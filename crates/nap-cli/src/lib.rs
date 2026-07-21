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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install required dependencies.
    Install {
        /// Target to install (e.g., "lore").
        target: String,
    },

    /// Initialize a repository repository and/or configure the backend provider.
    ///
    /// When a repository name is provided, creates the repository structure
    /// (directories, config, repository manifest, initial commit).
    /// When --provider is given (or no provider is configured), sets up the
    /// backend provider. Both can be combined:
    ///
    ///   nap init starwars                     # create repository
    ///   nap init starwars --provider local    # create repository + configure provider
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

        /// Entity ID (slug). e.g., "lukeskywalker".
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
    ///   nap resolve nap://starwars/character/lukeskywalker#references.appears_in
    Resolve {
        /// NAP URI. e.g., "nap://starwars/character/lukeskywalker"
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

    /// Add a representation to an entity manifest.
    AddRepr {
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

    /// Compute the SHA-256 content hash of a file.
    ContentHash {
        /// Path to the file to hash.
        file: PathBuf,
    },
}
