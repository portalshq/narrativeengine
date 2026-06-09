//! NAP CLI — command-line interface for the Narrative Addressing Protocol.
//!
//! Commands:
//!   init     — Initialize a universe repository
//!   create   — Create an entity manifest
//!   resolve  — Resolve a NAP URI (with optional fragment query)
//!   query    — Query a subtree from a manifest
//!   commit   — Commit changes to a manifest
//!   history  — View commit history for an entity
//!   list     — List entities or universes
//!   branch   — Create or list branches
//!   tag      — Create or list tags
//!   sign     — Sign a manifest (stub for v0)
//!   verify   — Verify a manifest signature (stub for v0)

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nap_core::{
    commit::Change,
    manifest::Representation,
    repository::Repository,
    resolver::{ResolveOptions, ResolveResult, Resolver},
    types::EntityType,
    uri::NapUri,
    vcs_git::GitBackend,
};
use std::io::IsTerminal;
use std::path::PathBuf;

/// NAP — Narrative Addressing Protocol CLI
///
/// Identity, addressing, resolution, and attribution for entertainment media.
#[derive(Parser, Debug)]
#[command(name = "nap", version, about, long_about = None)]
struct Cli {
    /// Base directory for universe repositories.
    /// Defaults to current directory.
    #[arg(long, short = 'd', global = true, default_value = ".")]
    base_dir: PathBuf,

    /// Enable verbose debug logging.
    #[arg(long, short = 'v', global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new universe repository.
    Init {
        /// Universe name (e.g., "starwars", "toystory").
        universe: String,
    },

    /// Create a new entity manifest.
    Create {
        /// Entity type: character, location, scene, prop, world.
        entity_type: String,

        /// Entity ID (slug). e.g., "lukeskywalker".
        entity_id: String,

        /// Universe name.
        #[arg(long, short = 'u')]
        universe: String,

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

        /// Resolve at a specific tag.
        #[arg(long)]
        tag: Option<String>,

        /// Output format: yaml, json.
        #[arg(long, short = 'f', default_value = "yaml")]
        format: String,
    },

    /// Query a subtree from a manifest.
    Query {
        /// NAP URI.
        uri: String,

        /// Dot-notation path. e.g., "appearances.audienceVotes".
        path: String,

        /// Output format: yaml, json.
        #[arg(long, short = 'f', default_value = "json")]
        format: String,
    },

    /// Commit changes to a universe repository.
    Commit {
        /// Universe name.
        universe: String,

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

    /// List universes or entities within a universe.
    List {
        /// Universe name. Omit to list all universes.
        universe: Option<String>,

        /// Entity type to list (if universe is specified).
        #[arg(long, short = 't')]
        entity_type: Option<String>,
    },

    /// Create or list branches.
    Branch {
        /// Universe name.
        universe: String,

        /// Branch name to create. Omit to list all branches.
        name: Option<String>,
    },

    /// Create or list tags.
    Tag {
        /// Universe name.
        universe: String,

        /// Tag name to create. Omit to list all tags.
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
        /// Universe name.
        universe: String,

        /// Commit hash to revert.
        #[arg(long, short = 'c')]
        commit: String,

        /// Author identifier.
        #[arg(long, short = 'a', default_value = "nap-cli")]
        author: String,
    },

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
}

/// Determine output format for a command.
///
/// Priority:
///   1. `--format` flag (honored when terminal; default remains as-set)
///   2. `NAP_OUTPUT` env var (explicit override regardless of terminal state)
///   3. Auto-detection: if stdout is not a terminal (piped), use JSON
fn resolve_output_format(requested: &str) -> String {
    // Env var takes highest priority
    if let Ok(env_val) = std::env::var("NAP_OUTPUT") {
        let val = env_val.trim().to_lowercase();
        if val == "json" || val == "yaml" {
            return val;
        }
    }

    // Auto-detect piped output
    if !std::io::stdout().is_terminal() {
        return "json".to_string();
    }

    // Default to what was requested (honor --format flag)
    requested.to_string()
}

/// Emit a human-friendly or machine-readable message depending on stdout.
fn emit(msg: impl AsRef<str>) {
    let msg = msg.as_ref();
    if std::io::stdout().is_terminal() {
        println!("{msg}");
    } else {
        // Piped/agent output → JSON structured log
        let entry = serde_json::json!({
            "level": "info",
            "message": msg,
        });
        println!("{}", serde_json::to_string(&entry).unwrap_or_else(|_| msg.to_string()));
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_piped = !std::io::stdout().is_terminal();

    // Initialize tracing
    let filter = if cli.verbose {
        "nap_core=trace,nap_cli=trace"
    } else {
        "nap_core=info,nap_cli=info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    let result = match cli.command {
        Commands::Init { universe } => cmd_init(&cli.base_dir, &universe),
        Commands::Create {
            entity_type,
            entity_id,
            universe,
            name,
            author,
        } => cmd_create(&cli.base_dir, &universe, &entity_type, &entity_id, &name, &author),
        Commands::Resolve {
            uri,
            branch,
            commit,
            tag,
            format,
        } => cmd_resolve(&cli.base_dir, &uri, branch, commit, tag, &format),
        Commands::Query { uri, path, format } => cmd_query(&cli.base_dir, &uri, &path, &format),
        Commands::Commit {
            universe,
            message,
            author,
        } => cmd_commit(&cli.base_dir, &universe, &message, &author),
        Commands::History { uri, limit } => cmd_history(&cli.base_dir, &uri, limit),
        Commands::List {
            universe,
            entity_type,
        } => cmd_list(&cli.base_dir, universe.as_deref(), entity_type.as_deref()),
        Commands::Branch { universe, name } => cmd_branch(&cli.base_dir, &universe, name.as_deref()),
        Commands::Tag { universe, name } => cmd_tag(&cli.base_dir, &universe, name.as_deref()),
        Commands::Set {
            uri,
            key,
            value,
            message,
            author,
        } => cmd_set(&cli.base_dir, &uri, &key, &value, &message, &author),
        Commands::AddRepr {
            uri,
            key,
            file,
            format,
            message,
            author,
        } => cmd_add_repr(&cli.base_dir, &uri, &key, &file, &format, &message, &author),
        Commands::Revert {
            universe,
            commit,
            author,
        } => cmd_revert(&cli.base_dir, &universe, &commit, &author),
        Commands::Sign { uri } => cmd_sign(&uri),
        Commands::Verify { uri } => cmd_verify(&uri),
    };

    if let Err(err) = result {
        if is_piped {
            let error_json = serde_json::json!({
                "level": "error",
                "error": err.to_string(),
                "code": "CLI_ERROR",
            });
            eprintln!("{}", serde_json::to_string(&error_json).unwrap());
        }
        return Err(err);
    }
    Ok(())
}

fn open_repo(base_dir: &PathBuf, universe: &str) -> Result<Repository> {
    let repo_path = base_dir.join(universe);
    Repository::open(&repo_path, Box::new(GitBackend::new()))
        .context(format!("failed to open universe '{universe}'"))
}

fn cmd_init(base_dir: &PathBuf, universe: &str) -> Result<()> {
    Repository::init(base_dir, universe, Box::new(GitBackend::new()))
        .context(format!("failed to initialize universe '{universe}'"))?;
    emit(format!("✓ Initialized universe '{universe}' at {}/{universe}", base_dir.display()));
    Ok(())
}

fn cmd_create(
    base_dir: &PathBuf,
    universe: &str,
    entity_type_str: &str,
    entity_id: &str,
    name: &str,
    author: &str,
) -> Result<()> {
    let entity_type: EntityType = entity_type_str
        .parse()
        .context(format!("unknown entity type '{entity_type_str}'"))?;
    let repo = open_repo(base_dir, universe)?;
    let (manifest, hash) = repo
        .create_entity(entity_type, entity_id, name, author)
        .context("failed to create entity")?;
    emit(format!("✓ Created {} '{}' ({})", entity_type, name, entity_id));
    emit(format!("  URI:    {}", manifest.id));
    emit(format!("  Commit: {}", &hash[..12]));
    Ok(())
}

fn cmd_resolve(
    base_dir: &PathBuf,
    uri_str: &str,
    branch: Option<String>,
    commit: Option<String>,
    tag: Option<String>,
    format: &str,
) -> Result<()> {
    let resolver = Resolver::new(base_dir);
    let options = ResolveOptions {
        branch,
        commit,
        tag,
        path: None,
    };
    let result = resolver
        .resolve(uri_str, &options)
        .context(format!("failed to resolve '{uri_str}'"))?;

    let fmt = resolve_output_format(format);
    match result {
        ResolveResult::Full(manifest) => {
            match fmt.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&manifest)?),
                _ => println!("{}", serde_yaml::to_string(&manifest)?),
            }
        }
        ResolveResult::Subtree(value) => {
            match fmt.as_str() {
                "yaml" => {
                    let yaml: serde_yaml::Value = serde_json::from_value(value)?;
                    println!("{}", serde_yaml::to_string(&yaml)?);
                }
                _ => println!("{}", serde_json::to_string_pretty(&value)?),
            }
        }
    }
    Ok(())
}

fn cmd_query(base_dir: &PathBuf, uri_str: &str, path: &str, format: &str) -> Result<()> {
    let resolver = Resolver::new(base_dir);
    let result = resolver
        .query(uri_str, path)
        .context(format!("failed to query '{uri_str}#{path}'"))?;

    let fmt = resolve_output_format(format);
    match fmt.as_str() {
        "yaml" => {
            let yaml: serde_yaml::Value = serde_json::from_value(result)?;
            println!("{}", serde_yaml::to_string(&yaml)?);
        }
        _ => println!("{}", serde_json::to_string_pretty(&result)?),
    }
    Ok(())
}

fn cmd_commit(base_dir: &PathBuf, universe: &str, message: &str, author: &str) -> Result<()> {
    let repo_path = base_dir.join(universe);
    let vcs = GitBackend::new();
    let hash = nap_core::vcs::VcsBackend::commit(&vcs, &repo_path, message, author)
        .context("failed to commit")?;
    emit(format!("✓ Committed: {} ({})", message, &hash[..12]));
    Ok(())
}

fn cmd_history(base_dir: &PathBuf, uri_str: &str, limit: usize) -> Result<()> {
    let uri: NapUri = uri_str.parse().context("invalid URI")?;
    let repo = open_repo(base_dir, &uri.universe)?;
    let history = repo
        .history(uri.entity_type, &uri.entity_id, limit)
        .context("failed to get history")?;

    if history.is_empty() {
        emit(format!("No history found for {uri_str}"));
        return Ok(());
    }

    if std::io::stdout().is_terminal() {
        for entry in &history {
            let short_hash = if entry.id.len() > 12 {
                &entry.id[..12]
            } else {
                &entry.id
            };
            println!("{} {} — {} ({})", short_hash, entry.timestamp, entry.message, entry.author);
        }
    } else {
        // Piped: emit full JSON array
        println!("{}", serde_json::to_string_pretty(&history)?);
    }
    Ok(())
}

fn cmd_list(base_dir: &PathBuf, universe: Option<&str>, entity_type: Option<&str>) -> Result<()> {
    let is_piped = !std::io::stdout().is_terminal();

    match universe {
        None => {
            let resolver = Resolver::new(base_dir);
            let universes = resolver.list_universes().context("failed to list universes")?;
            if is_piped {
                println!("{}", serde_json::to_string_pretty(&universes)?);
            } else if universes.is_empty() {
                println!("No universes found in {}", base_dir.display());
            } else {
                println!("Universes:");
                for u in &universes {
                    println!("  nap://{u}/");
                }
            }
        }
        Some(universe) => {
            let repo = open_repo(base_dir, universe)?;
            let is_piped = !std::io::stdout().is_terminal();
            match entity_type {
                Some(et_str) => {
                    let et: EntityType = et_str.parse().context("unknown entity type")?;
                    let entities = repo.list_entities(et).context("failed to list entities")?;
                    if is_piped {
                        println!("{}", serde_json::to_string_pretty(&entities)?);
                    } else {
                        println!("{} in {universe}:", et_str);
                        for e in &entities {
                            println!("  nap://{universe}/{et}/{e}");
                        }
                    }
                }
                None => {
                    // Collect all entities grouped by type → JSON for piped, human for terminal
                    let mut all: Vec<serde_json::Value> = Vec::new();
                    for et in EntityType::subdirectory_types() {
                        let entities = repo.list_entities(*et).unwrap_or_default();
                        if is_piped && !entities.is_empty() {
                            for e in &entities {
                                all.push(serde_json::json!({
                                    "type": et.to_string(),
                                    "id": e,
                                    "uri": format!("nap://{universe}/{et}/{e}"),
                                }));
                            }
                        } else if !entities.is_empty() {
                            println!("{}:", et);
                            for e in &entities {
                                println!("  nap://{universe}/{et}/{e}");
                            }
                        }
                    }
                    if is_piped {
                        println!("{}", serde_json::to_string_pretty(&all)?);
                    }
                }
            }
        }
    }
    Ok(())
}

fn cmd_branch(base_dir: &PathBuf, universe: &str, name: Option<&str>) -> Result<()> {
    let repo = open_repo(base_dir, universe)?;
    match name {
        Some(branch_name) => {
            repo.create_branch(branch_name)
                .context(format!("failed to create branch '{branch_name}'"))?;
            emit(format!("✓ Created branch '{branch_name}' in {universe}"));
        }
        None => {
            let branches = repo.list_branches().context("failed to list branches")?;
            if !std::io::stdout().is_terminal() {
                println!("{}", serde_json::to_string_pretty(&branches)?);
            } else {
                println!("Branches in {universe}:");
                for b in &branches {
                    println!("  {b}");
                }
            }
        }
    }
    Ok(())
}

fn cmd_tag(base_dir: &PathBuf, universe: &str, name: Option<&str>) -> Result<()> {
    let repo = open_repo(base_dir, universe)?;
    match name {
        Some(tag_name) => {
            repo.create_tag(tag_name)
                .context(format!("failed to create tag '{tag_name}'"))?;
            emit(format!("✓ Created tag '{tag_name}' in {universe}"));
        }
        None => {
            let tags = repo.list_tags().context("failed to list tags")?;
            if !std::io::stdout().is_terminal() {
                println!("{}", serde_json::to_string_pretty(&tags)?);
            } else if tags.is_empty() {
                println!("No tags in {universe}");
            } else {
                println!("Tags in {universe}:");
                for t in &tags {
                    println!("  {t}");
                }
            }
        }
    }
    Ok(())
}

fn cmd_set(
    base_dir: &PathBuf,
    uri_str: &str,
    key: &str,
    value: &str,
    message: &str,
    author: &str,
) -> Result<()> {
    let uri: NapUri = uri_str.parse().context("invalid URI")?;
    let repo = open_repo(base_dir, &uri.universe)?;
    let mut manifest = repo
        .read_manifest(uri.entity_type, &uri.entity_id)
        .context("failed to read manifest")?;

    // Parse value — try as YAML for structured values, fallback to string
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(value).unwrap_or_else(|_| {
        serde_yaml::Value::String(value.to_string())
    });

    manifest.set_property(key, yaml_value);
    let changes = vec![Change::set(
        &format!("properties.{key}"),
        None,
        value.to_string(),
    )];

    repo.commit_manifest(&mut manifest, message, author, changes)
        .context("failed to commit property change")?;

    emit(format!("✓ Set {key} = {value} on {uri_str}"));
    Ok(())
}

fn cmd_add_repr(
    base_dir: &PathBuf,
    uri_str: &str,
    key: &str,
    file: &PathBuf,
    format: &str,
    message: &str,
    author: &str,
) -> Result<()> {
    let uri: NapUri = uri_str.parse().context("invalid URI")?;
    let repo = open_repo(base_dir, &uri.universe)?;
    let mut manifest = repo
        .read_manifest(uri.entity_type, &uri.entity_id)
        .context("failed to read manifest")?;

    // Compute content hash
    let hash = nap_core::ContentHash::from_file(file)
        .context(format!("failed to hash file '{}'", file.display()))?;

    let repr = Representation {
        hash: hash.as_str().to_string(),
        format: format.to_string(),
        uri: Some(file.display().to_string()),
        tier: None,
    };

    manifest.set_representation(key, repr);
    let changes = vec![Change::set(
        &format!("representations.{key}"),
        None,
        hash.as_str().to_string(),
    )];

    repo.commit_manifest(&mut manifest, message, author, changes)
        .context("failed to commit representation")?;

    emit(format!("✓ Added representation '{key}' ({format}) to {uri_str}"));
    emit(format!("  Hash: {hash}"));
    Ok(())
}

fn cmd_revert(base_dir: &PathBuf, universe: &str, commit: &str, author: &str) -> Result<()> {
    let repo = open_repo(base_dir, universe)?;
    let new_hash = repo
        .revert_commit(commit, author)
        .context(format!("failed to revert commit '{commit}'"))?;
    let short_old = if commit.len() > 12 { &commit[..12] } else { commit };
    let short_new = &new_hash[..12.min(new_hash.len())];
    emit(format!("✓ Reverted commit {short_old} — new commit: {short_new}"));
    Ok(())
}

fn cmd_sign(uri_str: &str) -> Result<()> {
    emit(format!("⚠ Sign not implemented in v0. URI: {uri_str}"));
    emit("  Future: Ed25519 signing of manifest content hash.");
    Ok(())
}

fn cmd_verify(uri_str: &str) -> Result<()> {
    emit(format!("⚠ Verify not implemented in v0. URI: {uri_str}"));
    emit("  Future: Ed25519 signature verification.");
    Ok(())
}
