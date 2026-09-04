use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nap", version, about, long_about = None)]
pub struct Cli {
    /// Base directory for repository repositories.
    /// Defaults to $NAP_DIR, or ~/.nap if unset.
    #[arg(long, short = 'd', global = true, env = "NAP_DIR")]
    pub base_dir: Option<PathBuf>,

    /// Enable verbose debug logging.
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Resolve repository reads through the configured Lore server (the default).
    #[arg(long, global = true, conflicts_with = "local")]
    pub remote: bool,

    /// Resolve repository reads from an explicitly checked-out local working tree.
    #[arg(long, global = true, conflicts_with = "remote")]
    pub local: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands for `nap remote`.
#[derive(Subcommand, Debug)]
pub enum RemoteCmd {
    /// Add a remote to a repository repository.
    Add {
        /// Repository name.
        repository: String,
        /// Remote name (e.g., "origin").
        name: String,
        /// Remote URL.
        url: String,
    },
    /// List remotes on a repository repository.
    Ls {
        /// Repository name.
        repository: String,
    },
    /// Remove a remote from a repository repository.
    Rm {
        /// Repository name.
        repository: String,
        /// Remote name to remove.
        name: String,
    },
}

/// Subcommands for `nap choose`.
#[derive(Subcommand, Debug)]
pub enum ChooseCmd {
    /// Choose backend provider.
    Backend {
        /// Provider type: local, portals-cloud, or remote.
        provider: String,

        /// Remote URL (required for remote provider).
        #[arg(long)]
        remote_url: Option<String>,

        /// Workspace ID (for remote provider).
        #[arg(long)]
        workspace_id: Option<String>,

        /// Reset the provider configuration file.
        #[arg(long)]
        reset: bool,
    },
}

/// Subcommands for `nap backend`.
#[derive(Subcommand, Debug)]
pub enum BackendCmd {
    /// Configure the version-control backend.
    ///
    /// After configuration, existing unversioned repositories in this NAP home
    /// are offered an initial commit so their current filesystem state becomes
    /// the repository baseline (unless --no-initial-commit is given).
    Configure {
        /// Backend type: local or remote.
        backend: String,

        /// Remote endpoint URL (required for remote backend).
        #[arg(long)]
        endpoint: Option<String>,

        /// Workspace ID (for remote backend).
        #[arg(long)]
        workspace_id: Option<String>,

        /// Bootstrap existing repositories with an initial commit without prompting.
        #[arg(long)]
        initial_commit: bool,

        /// Skip bootstrapping existing repositories with an initial commit.
        #[arg(long)]
        no_initial_commit: bool,
    },

    /// Show the current version-control backend configuration.
    Status,
}

/// Interactive authentication commands for Portals Cloud.
#[derive(Subcommand, Debug)]
pub enum AuthCmd {
    /// Sign in through the configured Lore authentication service.
    Login {
        /// Exchange a service-account API key instead of opening a browser.
        #[arg(long)]
        api_key: bool,

        /// Environment variable containing the API key.
        #[arg(long, default_value = "PORTALS_CLOUD_API_KEY", requires = "api_key")]
        api_key_env: String,

        /// Print the login URL without opening a browser.
        #[arg(long, conflicts_with = "api_key")]
        no_browser: bool,
    },
    /// Show the currently cached Lore identity without printing tokens.
    Status,
    /// Remove locally cached Lore credentials.
    Logout,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage secure Portals Cloud authentication.
    Auth {
        /// Authentication operation.
        #[command(subcommand)]
        cmd: AuthCmd,
    },

    /// Install required dependencies.
    Install {
        /// Target to install (e.g., "lore" or "mcp").
        target: String,
    },

    /// Initialize a repository repository and/or configure the backend provider.
    ///
    /// When a repository name is provided, creates the repository structure
    /// (directories, config, repository manifest, initial commit).
    /// When --provider is given (or no provider is configured), sets up the
    /// backend provider. Both can be combined:
    ///
    ///   nap init toystory                     # create repository
    ///   nap init toystory --provider local    # create repository + configure provider
    ///   nap init --provider local             # configure provider only
    Init {
        /// Repository name. If provided, initializes a new repository repository.
        repository: Option<String>,

        /// Provider type: local, portals-cloud, or remote.
        #[arg(long)]
        provider: Option<String>,

        /// Remote URL (required for remote provider).
        #[arg(long)]
        remote_url: Option<String>,

        /// Workspace ID (for remote provider).
        #[arg(long)]
        workspace_id: Option<String>,

        /// Remote URL to add as origin after init.
        #[arg(long)]
        remote: Option<String>,

        /// Reset the provider configuration file.
        #[arg(long)]
        reset: bool,
    },

    /// Choose backend provider.
    Choose {
        /// Subcommand for choose.
        #[command(subcommand)]
        cmd: ChooseCmd,
    },

    /// Configure or inspect the version-control backend.
    Backend {
        /// Subcommand for backend.
        #[command(subcommand)]
        cmd: BackendCmd,
    },

    /// Run diagnostics and repair.
    Doctor {
        /// Auto-repair detected issues.
        #[arg(long)]
        repair: bool,
    },

    /// Publish changes to remote.
    Publish {
        /// Repository name.
        repository: String,
    },

    /// Show system status.
    Status,

    /// Sync with remote.
    Sync {
        /// Repository name.
        repository: String,
    },

    /// Create a new entity manifest.
    Create {
        /// Entity type (any non-empty string, e.g. character, location, custom-type).
        entity_type: String,

        /// Entity ID (slug). e.g., "woody".
        entity_id: String,

        /// Repository name.
        #[arg(long, short = 'u')]
        repository: String,

        /// Human-readable name.
        #[arg(long, short = 'n')]
        name: String,

        /// Author identifier.
        #[arg(long, short = 'a', default_value = "nap-cli")]
        author: String,
    },

    /// Resolve a NAP URI to its manifest or a subtree.
    ///
    /// Fragment queries are supported via the URI:
    ///   nap resolve nap://toystory/character/woody#references.appears_in
    Resolve {
        /// NAP URI. e.g., "nap://toystory/character/woody"
        uri: String,

        /// Resolve at a specific branch.
        #[arg(long)]
        branch: Option<String>,

        /// Resolve at a specific commit hash.
        #[arg(long)]
        commit: Option<String>,

        /// Output format: yaml, json.
        #[arg(long, short = 'f', default_value = "yaml", env = "NAP_OUTPUT")]
        format: String,

        /// Include condensed per-file provenance for the manifest and direct representations.
        #[arg(long)]
        provenance: bool,

        /// Hydrate known readable provenance artifacts such as prompts and run records.
        #[arg(long)]
        include_blobs: bool,
    },

    /// Create a time-limited public URL for a committed representation.
    #[command(
        long_about = r#"Create a time-limited public URL for a committed representation.

Pass the entity ID first and the representation name second:

```bash
nap presign 25th-chapter/character/nathan-gunn item
```

- `25th-chapter/character/nathan-gunn` identifies the repository, entity type,
  and entity ID. The `nap://` prefix is optional.
- `item` is the exact key under the entity manifest's `representations` map.
  It is not a file path or the entity's display name.

The equivalent fully qualified command is:

```bash
nap presign nap://25th-chapter/character/nathan-gunn item
```

### How the representation is located

NAP reads the entity manifest at the selected revision and looks up
`representations.item`. For example:

```yaml
representations:
  item:
    hash: blake3:<content hash>
    format: jpg
    uri: item.jpg
```

Representation URIs are relative to the entity's asset directory, matching
`nap add`. For this entity, `uri: item.jpg` resolves to
`character/nathan-gunn/item.jpg` within the repository. Keep `uri: item.jpg`;
there is no need to put the entity ID into the representation URI.

### Revision and lifetime

```bash
nap presign 25th-chapter/character/nathan-gunn item \
  --branch main \
  --ttl-seconds 900
```

Use either `--branch` or `--commit`, never both. When neither is supplied, NAP
uses the repository's configured default branch, falling back to the global
default branch. Branches are pinned to a commit before NAP reads the manifest
and content address. Lore applies its configured lifetime bounds and defaults
when `--ttl-seconds` is omitted.

The manifest and representation file must be committed at the selected
revision, and the content must have been pushed to the Lore server.
External URLs, linked repositories, absolute paths, path traversal, URI
fragments, and unversioned working-tree files are not supported.

### Output

In a terminal, the command prints the URL, expiration, and pinned revision.
When piped or redirected, it emits JSON with `url`, `expires_at`, `revision`,
`repository_id`, `address`, `representation`, and `format`.

The returned URL is a bearer capability: anyone who has it can download the
immutable bytes until it expires. Do not place it in logs, analytics, exception
messages, source control, or long-lived storage.

### Automatic configuration

NAP records the Lore HTTP origin in `provider.toml` during backend setup and
backfills older provider configurations automatically. Local Lore uses
`http://127.0.0.1:41339`; standard remote Lore uses the same host on port 41339;
TLS deployments behind port 443 use the same HTTPS origin. Portals Cloud uses
`https://lore.portals.works`. The normal command needs no additional flags:

```bash
nap presign 25th-chapter/character/nathan-gunn item
```

Authenticated requests reuse the active `nap auth login` / Lore identity.
Only unexpired repository-scoped tokens authorized for the HTTP recipient are
used. Automatic credential reuse requires HTTPS for remote servers; loopback
HTTP is supported for development. No separate HTTP token setup is needed.

Operators with custom proxy layouts can set `http_url` in `provider.toml`.
Explicit `--http-url` or `NAP_LORE_HTTP_URL` overrides take precedence.
Bearer-token environment overrides remain available for automation.

### Server setup and signing-key security

New NAP-managed local installations create a unique 32-byte signing key in
owner-only server configuration and bind to loopback. Existing managed configs
receive a missing key without replacing existing keys or other settings.
Restart an already running server after its configuration changes.

Standalone development Lore provisions a persistent owner-only `presign.key`
in its configuration directory when no signing key is supplied. Persist this
directory across restarts. Never copy that key into client configuration.

Only Lore uses the key, to sign and validate download capabilities. It is
independent of login tokens, JWT signing keys, and API-key peppers; NAP clients
never need it. Keep the key stable across restarts and private to the server.
Server logs omit signing keys and signed query tokens. Signed responses prevent
caching and referrer leakage. Development URLs require network access to the host.

Production / Portals Cloud presign is WIP. NAP derives the Cloud HTTP origin,
but deployment still needs a dedicated shared signing key, scoped HTTPS routes,
and query-token-safe logging. Without a supplied production key, presign stays
disabled. These are operator concerns, not end-user flags or secrets.

### SDK methods

All three methods take the entity ID and representation name as their first
two arguments, using the same lookup as the CLI:

- Rust: `Resolver::presign_representation(entity_id, representation, &options)`
  is asynchronous.
- Python: `presign_representation(entity_id, representation, **options)` is
  synchronous.
- TypeScript: `presignRepresentation(entityId, representation, options)` is
  asynchronous.

Python:

```python
from nap_sdk import presign_representation

result = presign_representation(
    "25th-chapter/character/nathan-gunn", "item", branch="main", ttl_seconds=900
)
```

TypeScript:

```typescript
import { presignRepresentation } from "@portalshq/nap-sdk";

const result = await presignRepresentation(
  "25th-chapter/character/nathan-gunn",
  "item",
  { branch: "main", ttlSeconds: 900 },
);
```

The SDKs return the same fields as the CLI JSON output.
"#
    )]
    Presign {
        /// Entity ID, e.g. 25th-chapter/character/nathan-gunn. The nap:// prefix is optional; fragments are not supported.
        #[arg(value_name = "ENTITY_ID")]
        uri: String,

        /// Representation name (manifest key), e.g. item. Its URI is relative to the entity's asset directory.
        representation: String,

        /// Resolve at a specific branch.
        #[arg(long, conflicts_with = "commit")]
        branch: Option<String>,

        /// Resolve at a specific commit hash.
        #[arg(long, conflicts_with = "branch")]
        commit: Option<String>,

        /// Requested lifetime in seconds; Lore enforces its configured bounds.
        #[arg(long)]
        ttl_seconds: Option<u64>,

        /// Explicit Lore HTTP origin, such as http://127.0.0.1:41339.
        #[arg(long)]
        http_url: Option<String>,

        /// Environment variable containing a repository-scoped bearer token.
        #[arg(long)]
        token_env: Option<String>,
    },

    /// Query a subtree from a manifest.
    Query {
        /// NAP URI.
        uri: String,

        /// Dot-notation path. e.g., "appearances.audienceVotes".
        path: String,

        /// Output format: yaml, json.
        #[arg(long, short = 'f', default_value = "json", env = "NAP_OUTPUT")]
        format: String,
    },

    /// Commit changes to a repository repository.
    Commit {
        /// Repository name.
        repository: String,

        /// Commit message.
        #[arg(long, short = 'm')]
        message: String,

        /// Author identifier.
        #[arg(long, short = 'a', default_value = "nap-cli")]
        author: String,
    },

    /// View commit history for an entity.
    History {
        /// NAP URI.
        uri: String,

        /// Maximum number of commits to show.
        #[arg(long, short = 'n', default_value = "20")]
        limit: usize,
    },

    /// List repositories or entities within a repository.
    List {
        /// Repository name. Omit to list all repositories.
        repository: Option<String>,

        /// Entity type to list (if repository is specified).
        #[arg(long, short = 't')]
        entity_type: Option<String>,
    },

    /// Create or list branches.
    Branch {
        /// Repository name.
        repository: String,

        /// Branch name to create. Omit to list all branches.
        name: Option<String>,
    },

    /// Set a property on an entity manifest.
    Set {
        /// NAP URI.
        uri: String,

        /// Property key (dot-notation).
        key: String,

        /// Property value.
        value: String,

        /// Commit message.
        #[arg(long, short = 'm', default_value = "set property")]
        message: String,

        /// Author identifier.
        #[arg(long, short = 'a', default_value = "nap-cli")]
        author: String,
    },

    /// Add a file representation to an entity manifest.
    #[command(alias = "add-repr")]
    Add {
        /// NAP URI.
        uri: String,

        /// Representation key. e.g., "reference_image".
        key: String,

        /// File path to the asset.
        file: PathBuf,

        /// Asset format. e.g., "png", "glb".
        #[arg(long)]
        format: String,

        /// Commit message.
        #[arg(long, short = 'm', default_value = "add representation")]
        message: String,

        /// Author identifier.
        #[arg(long, short = 'a', default_value = "nap-cli")]
        author: String,
    },

    /// Revert a commit by hash (undoes all changes in that commit).
    Revert {
        /// Repository name.
        repository: String,

        /// Commit hash to revert.
        #[arg(long, short = 'c')]
        commit: String,

        /// Author identifier.
        #[arg(long, short = 'a', default_value = "nap-cli")]
        author: String,
    },

    /// Clone or pull a repository from a remote.
    ///
    /// If the argument is a URL, the repo is cloned (name is read from the
    /// repo's own config).  If it's a repository name, the repo must already
    /// exist locally and will be updated via pull.
    Pull {
        /// URL (clone) or repository name (pull existing).
        url_or_name: String,
    },

    /// Push the current branch to its configured upstream remote.
    Push {
        /// Repository name.
        repository: String,

        /// Remote name (default: tracking branch's remote, or "origin").
        #[arg(long, default_value = "origin")]
        remote: String,

        /// Branch to push (default: current branch).
        #[arg(long)]
        branch: Option<String>,
    },

    /// Manage remotes on a repository.
    #[command(subcommand)]
    Remote(RemoteCmd),

    /// Sign a manifest (stub for v0).
    Sign {
        /// NAP URI.
        uri: String,
    },

    /// Verify a manifest signature (stub for v0).
    Verify {
        /// NAP URI.
        uri: String,
    },

    /// Switch to a branch.
    Switch {
        /// Repository name.
        repository: String,
        /// Branch name to switch to.
        name: String,
    },

    /// Show the current HEAD commit hash.
    HeadHash {
        /// Repository name.
        repository: String,
    },

    /// Validate a manifest against the NAP schema.
    Validate {
        /// NAP URI of the entity to validate.
        uri: Option<String>,
        /// Path to a manifest YAML file to validate.
        #[arg(long)]
        file: Option<PathBuf>,
    },

    /// Print a JSON Schema for manifest or commit types.
    Schema {
        /// Schema name: 'manifest' or 'commit'.
        name: String,
        /// Output format: json, yaml.
        #[arg(long, short = 'f', default_value = "json")]
        format: String,
    },

    /// Show diff between two manifest files or versions.
    Diff {
        /// Base (left) manifest file.
        base_file: PathBuf,
        /// Candidate (right) manifest file.
        candidate_file: PathBuf,
        /// Output format: json, yaml.
        #[arg(long, short = 'f', default_value = "yaml")]
        format: String,
    },

    /// Three-way merge of JSON/YAML values.
    Merge {
        /// Base (common ancestor) file.
        base: PathBuf,
        /// Current (ours) file.
        current: PathBuf,
        /// Proposed (theirs) file.
        proposed: PathBuf,
        /// Output format: json, yaml.
        #[arg(long, short = 'f', default_value = "yaml")]
        format: String,
    },

    /// Compute the BLAKE3 content hash of a file.
    ContentHash {
        /// Path to the file to hash.
        file: PathBuf,
    },
}
