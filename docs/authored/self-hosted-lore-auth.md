# Self-hosted Lore authentication

This is the canonical guide for consumer authentication against a self-hosted
Lore server. The generated `px auth` reference documents command syntax; this
guide defines the self-hosted deployment contract.

## For PX users

Sign in once in an interactive terminal:

```bash
px auth login
```

PX opens the configured SSO or email sign-in page. After a successful sign-in,
return to PX and continue working normally:

```bash
px sync 25th-chapter
px push 25th-chapter
```

Users never create, copy, refresh, or supply JWTs. PX and Lore perform the
following automatically:

1. Store the renewable user session in the OS credential store.
2. Refresh that session when needed without reopening a browser.
3. Exchange it for a short-lived token scoped to exactly the repository being
   accessed.
4. Refresh the repository token before it expires.

If the renewable session has expired, been revoked, or cannot be refreshed,
PX prompts the user to run `px auth login` again. `px auth logout` removes the
local session.

## Deployment contract

Development and production use the same three-role protocol:

| Role | Responsibility |
| --- | --- |
| PX/Lore client | Owns the encrypted user session and renews it automatically. |
| Auth gateway | Runs the SSO/email browser flow, refreshes the user session, and issues repository-scoped tokens after checking permissions. |
| Lore server | Verifies signed, short-lived repository tokens using the gateway JWKS. |

The auth gateway is the only component that talks to the configured identity
provider. The Lore server never receives user passwords, refresh tokens, or
provider credentials. Consumers never receive the gateway signing key.

The server advertises the gateway through its environment endpoint. PX must
use that advertised endpoint rather than a consumer-provided token URL; this
keeps local and production behavior mechanically identical.

## Strict-mode JWKS trust

Lore strict mode (`LORE_SECURITY_MODE=strict`) requires:

```toml
[server.auth]
jwt_issuer = "https://auth.dev.example.com"
jwt_audience = ["lore", "portals.works"]

[server.auth.jwk]
endpoint = "https://auth.dev.example.com/.well-known/jwks.json"
refresh_interval_seconds = 60
max_stale_seconds = 600
# Omit ca_file when the JWKS hostname uses a publicly trusted cert.
# For a private dev CA, point at the CA bundle (never disable TLS):
ca_file = "/Users/vibrantceo/Library/Application Support/Portals/Lore/auth/dev-root-ca.pem"
```

A configuration-only change is sufficient when the JWKS hostname already
presents a publicly trusted TLS certificate. For a private LAN CA (e.g.
`andresb.local` or `auth.dev.example.com` with a self-signed CA), a Lore
source change is required — the `ca_file` option adds the private root with
`reqwest::Certificate::from_pem` + `add_root_certificate` without disabling
hostname or certificate validation. `SSL_CERT_FILE` does not fix the running
Lore binary's `UnknownIssuer` because Lore builds its own `reqwest::Client`.

Generate the private CA and cert with:

```bash
control-plane/auth-gateway/scripts/setup-dev-auth.sh
# adds 127.0.0.1 auth.dev.example.com to /etc/hosts
```

The script keeps the CA private key only on the TLS proxy/gateway host and
makes `dev-root-ca.pem` admin-owned, non-user-writable.

## Refresh-token security

The gateway issues its own opaque, rotating refresh credential — never a raw
Cognito/IdP refresh token. Security properties:

- **Rotation:** every `RefreshAuthSession` call atomically consumes the
  presented token and returns a new one (`rotated_at` + `replaced_by`).
- **Replay detection:** reuse of a consumed token revokes the entire family
  (`family_id`) immediately.
- **Revocation:** `RevokeAuthSession` (called by `lore auth logout`) revokes
  the family; `disable_user` / `disable_service_account` revokes all families.
- **Expiration:** 30-day absolute TTL; expired tokens revoke their family.
- **No logging:** tokens are SHA-256 hashed at rest and never logged.
- **Concurrency:** Lore's `refreshed_authentication_token` uses a per
  `(auth_url, identity)` `Mutex` and a 120 s pre-expiry window; after waiting,
  it reloads the token store before touching the one-time credential.

## OIDC production flexibility

Set a generic issuer to run the same gateway against Keycloak, ZITADEL, or
Ory without changing PX or Lore:

```bash
OIDC_ISSUER=https://auth.dev.example.com/realms/portals
OIDC_CLIENT_ID=portals-dev
OIDC_DOMAIN=https://auth.dev.example.com
```

The gateway discovers `/.well-known/openid-configuration`, enforces exact
issuer, `aud` = client ID, `RS256`, `PKCE S256`, `state`/`nonce`, and TLS,
and maps only standard claims (`sub`, `email`, `name`,
`preferred_username`). Repository authorization, JWT signing (`RS256`,
`kid`), and opaque rotation stay provider-independent.

For genuine production parity, run the gateway container with a **dedicated
non-production Cognito user pool** (provisioned via `AuthFoundation`); use
Keycloak only when you accept the OIDC-provider adapter boundary.

## Development container parity

`control-plane/auth-gateway/docker-compose.dev.yml` runs the full stack
locally (Postgres + Keycloak + Mailpit + Caddy TLS + Auth Gateway) while
Cognito itself stays AWS-managed — a local Cognito emulator would not give
parity. Start it with `docker compose -f docker-compose.dev.yml up -d` and
point Lore strict at `https://auth.dev.example.com`.

## Andres lab status

The durable Lore store is active at
`/Users/andresb/Library/Application Support/Portals/Lore/store`. Strict-mode
JWKS verification is now supported via `server.auth.jwk.ca_file`; the staged
lab signer/JWKS is not production infrastructure and should be replaced with
the gateway + IdP above.
