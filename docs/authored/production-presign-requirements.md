# Production Presign Requirements

Source: incident where `px presign bears/location/setting1 reference_image`
returned a URL that rendered in the browser as a text file instead of a PNG.
Root cause: the bearer had expired (or died with a server restart), so Lore's
redeem endpoint returned `401 text/plain "invalid or expired token"` and the
browser rendered that 24-byte text. Bytes, signing path, and `px` were all
healthy — verified live: a fresh mint redeemed `200`, 2.8MB, PNG magic bytes.

The dev-server behavior below is acceptable for LAN development but must not
ship to production as-is. Owners: signing/errors/serving = Lore server repo
(`portalshq/lore`, external pinned binary); client items = `px` (this repo).

## 1. Signing-key management (Lore)

- [ ] Production key is provisioned explicitly (secret manager), never the
  auto-generated dev key. Presign stays disabled without one (already the
  behavior — keep it).
- [ ] Documented rotation story: overlapping key acceptance (`kid`-style key
  IDs already exist in the token payload) so rotation does not instantly kill
  outstanding URLs the way a restart does today.
- [ ] Constant-time HMAC comparison on redeem (verify, then state it).

## 2. Structured, communicative errors (Lore)

Today: `401 text/plain "invalid or expired token"` for every failure mode.

- [ ] Error body is RFC 9457 problem-JSON, e.g.
  `{code: presign_expired | presign_invalid | presign_disabled,
  message, expires_at, server_now}`.
- [ ] Keep one message for bad-signature (no oracle), but distinguish
  `expired` from `invalid` — expiry is not secret (it is base64-readable in
  the token) and `server_now` is the clock-skew detector.
- [ ] Status: `401` for invalid, `410 Gone` or `401` with `code=expired` for
  expired — pick one, document it, keep it stable for clients.

## 3. Safe content serving (Lore) — the highest-risk item

Observed on `200`: headers are `content-length` + `date` only. No
`Content-Type`, `Range` ignored (full 2.8MB re-served), no `Cache-Control`.

- [ ] Set `Content-Type` from stored representation metadata; fall back to
  `application/octet-stream`, never sniffable text.
- [ ] `X-Content-Type-Options: nosniff` on all redeem responses.
- [ ] `Content-Disposition`: `inline; filename="<repr>.<ext>"` for safe image
  types; `attachment` for everything else (a committed HTML/SVG served from
  the Lore origin with script execution is a stored-XSS vector, and bearer
  URLs live in that origin).
- [ ] `Cache-Control: private, no-store` (bearer capability) and
  `Referrer-Policy: no-referrer` on redeem responses.
- [ ] Honor `Range` (or document not to); full re-serves are a bandwidth/cost
  bug for large representations.
- [ ] Serve from a dedicated download origin, not the API origin, so rendered
  content never shares ambient authority with API routes.

## 4. Lifetime bounds (Lore)

- [ ] Document min/max/default TTL and clamp behavior server-side; reject (not
  silently rewrite) out-of-range `ttl_seconds`.
- [ ] One observed mint returned an already-expired `expires_at` (transient;
  suspected dev-server clock jump). Prod needs monotonic time source / NTP
  requirement stated in runbook.

## 5. `px` client follow-ups (this repo, optional, mostly blocked on §2)

- [x] Humanize terminal expiry (`Expires at: … (in 10m 0s)` / `ALREADY
  EXPIRED`) + warn that dead URLs render as 401 text. Done in
  `crates/px-cli/src/main.rs` (`format_presign_expiry`).
- [ ] Parse structured error codes once Lore emits them, instead of today's
  `"not enabled"` substring match in `resolver.rs`.
- [ ] Send filename/content-type hints at mint if Lore adds those fields to
  the presign request.
- [ ] Consider opt-in `--verify` (HEAD redeem at mint) for scripts that need
  fail-fast; not default (doubles traffic).

## Acceptance

Fresh `px presign` → open in browser → image renders before expiry; after
expiry the CLI (not the browser) is where the user learns why; security
headers verified with `curl -D`; rotation drill completes without mass
invalidating live URLs.
