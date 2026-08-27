#![allow(dead_code)]
//! NAP CLI — command-line interface for the Narrative Addressing Protocol.
//!
//! Commands:
//!   init         — Initialize a repository repository and/or configure provider
//!   choose       — Choose backend provider
//!   doctor       — Run diagnostics and repair
//!   publish      — Publish changes to remote
//!   status       — Show system status
//!   sync         — Sync with remote
//!   create       — Create an entity manifest
//!   resolve      — Resolve a NAP URI (with optional fragment query)
//!   query        — Query a subtree from a manifest
//!   commit       — Commit changes to a manifest
//!   history      — View commit history for an entity
//!   list         — List entities or repositories
//!   branch       — Create or list branches
//!   tag          — Create or list tags
//!   pull         — Clone or pull a repository from a remote
//!   push         — Push a repository to a remote
//!   remote       — Manage remotes on a repository
//!   sign         — Sign a manifest (stub for v0)
//!   verify       — Verify a manifest signature (stub for v0)

use anyhow::{Context, Result};
use clap::Parser;
use nap_cli::{AuthCmd, BackendCmd, ChooseCmd, Cli, Commands, RemoteCmd};
use nap_core::{
    commit::Change,
    error::NapError,
    manifest::Representation,
    provider::{ProviderFactory, ProviderManager, ProviderType},
    repository::Repository,
    resolver::{ResolveOptions, ResolveResult, Resolver},
    server::{LoreInstaller, NapDoctor, ServerManager},
    types::EntityType,
    uri::NapUri,
    vcs_lore::LoreBackend,
};
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Expand a path, resolving a leading `~` to the user's home directory.
/// Supports both Unix (HOME) and Windows (USERPROFILE) environments.
fn expand_path(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if s.starts_with('~') {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .unwrap_or_else(|| {
                // Fallback: keep path as-is if no home env var is available
                std::ffi::OsString::from("")
            });

        if !home.is_empty() {
            let rest = s.strip_prefix('~').unwrap_or("");
            return PathBuf::from(home).join(rest.trim_start_matches('/'));
        }
    }
    path.to_path_buf()
}

/// Return `true` if `s` looks like a URL rather than a repository name.
///
/// Repository names are simple identifiers (`[a-zA-Z0-9_-]+`).
/// Everything else (contains `@`, `://`, `/`, special characters, etc.) is a URL.
fn looks_like_url(s: &str) -> bool {
    !s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
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
        println!(
            "{}",
            serde_json::to_string(&entry).unwrap_or_else(|_| msg.to_string())
        );
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let is_piped = !std::io::stdout().is_terminal();

    // Initialize tracing — silent by default, verbose with -v
    let filter = if cli.verbose {
        "nap_core=trace,nap_cli=trace"
    } else {
        "nap_core=warn,nap_cli=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    // Resolve base directory: -d flag > $NAP_DIR > ~/.nap
    let base_dir = cli
        .base_dir
        .or_else(|| std::env::var("NAP_DIR").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("~/.nap"));
    let base_dir = expand_path(&base_dir);

    // Ensure the base directory exists (e.g. ~/.nap/)
    std::fs::create_dir_all(&base_dir)
        .with_context(|| format!("failed to create base directory '{}'", base_dir.display()))?;

    let result = match cli.command {
        Commands::Auth { cmd } => cmd_auth(cmd),
        Commands::Init {
            repository,
            provider,
            remote_url,
            workspace_id,
            remote,
            reset,
        } => cmd_init(
            &base_dir,
            repository.as_deref(),
            provider,
            remote_url,
            workspace_id,
            remote.as_deref(),
            reset,
        ),
        Commands::Install { target } => cmd_install(&base_dir, &target),
        Commands::Choose { cmd } => cmd_choose(&base_dir, cmd),
        Commands::Backend { cmd } => cmd_backend(&base_dir, cmd),
        Commands::Doctor { repair } => cmd_doctor(&base_dir, repair),
        Commands::Publish { repository } => cmd_publish(&base_dir, &repository),
        Commands::Status => cmd_status(&base_dir),
        Commands::Sync { repository } => cmd_sync(&base_dir, &repository),
        Commands::Create {
            entity_type,
            entity_id,
            repository,
            name,
            author,
        } => cmd_create(
            &base_dir,
            &repository,
            &entity_type,
            &entity_id,
            &name,
            &author,
        ),
        Commands::Resolve {
            uri,
            branch,
            commit,
            //     tag,
            format,
            provenance,
            include_blobs,
        } => cmd_resolve(
            &base_dir,
            &uri,
            branch,
            commit,
            &format,
            provenance,
            include_blobs,
        ),
        Commands::Query { uri, path, format } => cmd_query(&base_dir, &uri, &path, &format),
        Commands::Commit {
            repository,
            message,
            author,
        } => cmd_commit(&base_dir, &repository, &message, &author),
        Commands::History { uri, limit } => cmd_history(&base_dir, &uri, limit),
        Commands::List {
            repository,
            entity_type,
        } => cmd_list(&base_dir, repository.as_deref(), entity_type.as_deref()),
        Commands::Branch { repository, name } => {
            cmd_branch(&base_dir, &repository, name.as_deref())
        }
        Commands::Set {
            uri,
            key,
            value,
            message,
            author,
        } => cmd_set(&base_dir, &uri, &key, &value, &message, &author),
        Commands::Add {
            uri,
            key,
            file,
            format,
            message,
            author,
        } => cmd_add_repr(&base_dir, &uri, &key, &file, &format, &message, &author),
        Commands::Revert {
            repository,
            commit,
            author,
        } => cmd_revert(&base_dir, &repository, &commit, &author),
        Commands::Pull { url_or_name } => cmd_pull(&base_dir, &url_or_name),
        Commands::Push {
            repository,
            remote,
            branch,
        } => cmd_push(&base_dir, &repository, &remote, branch.as_deref()),
        Commands::Remote(cmd) => cmd_remote(&base_dir, cmd),
        Commands::Sign { uri } => cmd_sign(&uri),
        Commands::Verify { uri } => cmd_verify(&uri),
        Commands::Switch { repository, name } => cmd_switch(&base_dir, &repository, &name),
        Commands::HeadHash { repository } => cmd_head_hash(&base_dir, &repository),
        Commands::Validate { uri, file } => {
            cmd_validate(&base_dir, uri.as_deref(), file.as_deref())
        }
        Commands::Schema { name, format } => cmd_schema(&name, &format),
        Commands::Diff {
            base_file,
            candidate_file,
            format,
        } => cmd_diff(&base_file, &candidate_file, &format),
        Commands::Merge {
            base,
            current,
            proposed,
            format,
        } => cmd_merge(&base, &current, &proposed, &format),
        Commands::ContentHash { file } => cmd_content_hash(&file),
    };

    if let Err(err) = result {
        if is_piped {
            let error_json = serde_json::json!({
                "level": "error",
                "error": format!("{err:#}"),
                "code": "CLI_ERROR",
            });
            eprintln!("{}", serde_json::to_string(&error_json).unwrap());
        } else {
            if cli.verbose {
                eprintln!("{:?}", err);
            } else {
                eprintln!("Error: {err:#}");
            }
        }
        std::process::exit(1);
    }
    Ok(())
}

/// Delegate authentication to Lore so Nap and Lore share one OS-keyring-backed
/// credential store. Login deliberately inherits stdio for browser/device-code
/// interaction; repository commands remain noninteractive.
fn cmd_auth(cmd: AuthCmd) -> Result<()> {
    const CLOUD_REMOTE: &str = "grpcs://lore.portals.works";
    let cloud_auth_url = std::env::var("NAP_AUTH_URL")
        .unwrap_or_else(|_| "ucs-auth://auth.portals.works".to_string());
    let (args, stdin_secret): (Vec<String>, Option<String>) = match cmd {
        AuthCmd::Login {
            api_key,
            api_key_env,
            no_browser,
        } => {
            if api_key {
                let token = std::env::var(&api_key_env).with_context(|| {
                    format!(
                        "{api_key_env} is not set; inject a revocable service-account API key into CI"
                    )
                })?;
                (
                    vec![
                        "auth".into(),
                        "login".into(),
                        "--token-type".into(),
                        "api-key".into(),
                        "--token-stdin".into(),
                        "--auth-url".into(),
                        cloud_auth_url.clone(),
                        CLOUD_REMOTE.into(),
                    ],
                    Some(token),
                )
            } else {
                let mut args = vec!["auth".into(), "login".into(), CLOUD_REMOTE.into()];
                if no_browser {
                    args.push("--no-browser".into());
                }
                (args, None)
            }
        }
        // `list` deliberately omits --with-token.
        AuthCmd::Status => (vec!["auth".into(), "list".into()], None),
        AuthCmd::Logout => (
            vec![
                "auth".into(),
                "logout".into(),
                "--auth-url".into(),
                cloud_auth_url,
            ],
            None,
        ),
    };
    let operation = args.get(1).map(String::as_str).unwrap_or("unknown");
    let binary = nap_core::vcs_lore::LoreProcessRunner::binary();
    let mut command = std::process::Command::new(&binary);
    command.args(&args);
    if stdin_secret.is_some() {
        command.stdin(std::process::Stdio::piped());
    }
    let mut child = command
        .spawn()
        .with_context(|| {
            format!(
                "failed to execute `{binary} auth {operation}`; install a compatible Lore CLI and ensure it is on PATH"
            )
        })?;
    if let Some(secret) = stdin_secret {
        let mut stdin = child
            .stdin
            .take()
            .context("failed to open Lore authentication stdin")?;
        stdin
            .write_all(secret.as_bytes())
            .context("failed to send API key to Lore over stdin")?;
    }
    let status = child
        .wait()
        .context("failed while waiting for Lore authentication")?;

    if status.success() {
        return Ok(());
    }

    anyhow::bail!(
        "Lore authentication failed (exit {}); run `nap auth login` in an interactive terminal and retry",
        status.code().unwrap_or(-1)
    )
}

/// Prompt the user to select a provider type
fn prompt_for_provider() -> Result<String> {
    println!("Select where to store your projects:\n");
    println!("  1. Local          Free. Installs local services.");
    println!("  2. Portals Cloud  Sync and collaborate online.\n");
    print!("Enter choice [1-2]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => Ok("local".to_string()),
        "2" => Ok("portals-cloud".to_string()),
        _ => {
            // Accept direct provider names as well
            if choice == "local" || choice == "portals-cloud" || choice == "remote" {
                Ok(choice.to_string())
            } else {
                anyhow::bail!("Invalid choice. Please enter 1, 2, or a provider name.")
            }
        }
    }
}

/// Get LoreBackend with precedence: env vars > provider config > defaults
fn get_lore_backend(_base_dir: &Path) -> LoreBackend {
    LoreBackend::from_env()
}

/// Open a repository, honoring the optional version-control backend.
///
/// When a version-control backend is configured (valid `provider.toml`), the
/// repository is opened in versioned mode and VCS operations are available.
/// Otherwise the repository is opened in unversioned mode: filesystem
/// reads/writes work, but VCS-only operations fail with an informative
/// [`NapError::BackendNotConfigured`] error.
fn open_repo(base_dir: &Path, repository: &str) -> Result<Repository> {
    let repo_path = base_dir.join(repository);
    let vcs: Option<Box<dyn nap_core::vcs::VcsBackend>> =
        if nap_core::provider::version_control_configured(base_dir) {
            Some(Box::new(get_lore_backend(base_dir)))
        } else {
            None
        };
    Repository::open_optional(&repo_path, vcs).map_err(|e| match e {
        NapError::RepositoryNotFound(_) => {
            anyhow::anyhow!("repository not found: '{}'", repository)
        }
        _ => anyhow::anyhow!(e),
    })
}

/// Return a configured VCS backend for CLI-initiated VCS operations, or a
/// clear error when none is configured.
fn require_backend(base_dir: &Path, operation: &str) -> Result<Box<dyn nap_core::vcs::VcsBackend>> {
    if nap_core::provider::version_control_configured(base_dir) {
        Ok(Box::new(get_lore_backend(base_dir)))
    } else {
        Err(NapError::BackendNotConfigured {
            operation: operation.to_string(),
        }
        .into())
    }
}

fn cmd_init(
    base_dir: &Path,
    repository: Option<&str>,
    provider_opt: Option<String>,
    remote_url: Option<String>,
    workspace_id: Option<String>,
    remote: Option<&str>,
    reset: bool,
) -> Result<()> {
    // ── Step 0: Reset provider config if requested ────────
    if reset {
        let config_path = base_dir.join("provider.toml");
        if config_path.exists() {
            std::fs::remove_file(&config_path).context("failed to reset provider configuration")?;
            emit("✓ Reset provider configuration.");
        }
    }

    // ── Step 1: Check if provider is configured ────────
    let mut provider_manager = ProviderManager::new(base_dir);
    let provider_configured = provider_manager.load_configured_provider()?.is_some();

    // ── Step 2: Configure provider if requested or on first run ────────
    let should_configure_provider = provider_opt.is_some()
        || (!provider_configured && (repository.is_some() || repository.is_none()));

    if should_configure_provider {
        let provider_str = if let Some(p) = provider_opt {
            p
        } else {
            prompt_for_provider()?
        };

        let provider_type = ProviderType::parse_from_str(&provider_str)
            .context(format!("invalid provider type '{provider_str}'"))?;

        let factory = ProviderFactory::new(base_dir);
        let provider = match provider_type {
            ProviderType::Local => factory.create_provider(ProviderType::Local)?,
            ProviderType::PortalsCloud => factory.create_provider(ProviderType::PortalsCloud)?,
            ProviderType::Remote => {
                let url = remote_url
                    .as_ref()
                    .context("remote provider requires --remote-url")?;
                let ws_id = workspace_id
                    .as_ref()
                    .context("remote provider requires --workspace-id")?;
                factory.create_remote_provider(url, ws_id)?
            }
        };

        provider_manager.set_active_provider(provider.clone());
        provider_manager
            .save_provider_config(provider.as_ref())
            .context("failed to save provider configuration")?;

        // Initialize and verify the provider
        match provider_type {
            ProviderType::Local => emit("Setting up local services..."),
            ProviderType::PortalsCloud => emit("Connecting to Portals Cloud..."),
            ProviderType::Remote => emit("Configuring remote provider..."),
        }

        let rt = get_tokio_runtime();
        rt.block_on(provider.initialize())
            .context("failed to initialize provider")?;

        emit(format!(
            "✓ Ready. NAP is configured with {}.",
            provider.name()
        ));
    }

    // ── Step 3: Initialize repository repository if name given ───────────
    if let Some(universe_name) = repository {
        let provider_type = provider_manager
            .active_provider()
            .map(|p| p.provider_type())
            .unwrap_or(ProviderType::Local);

        // Install dependencies based on provider type
        let installer = LoreInstaller::new(None);
        match provider_type {
            ProviderType::Local => {
                emit("Installing Lore dependencies...");
                installer.install_all()?;
                emit("✓ Lore CLI and server installed.");
            }
            _ => {
                emit("Installing Lore CLI...");
                installer.install_cli()?;
                emit("✓ Lore CLI installed.");
            }
        }

        // For Local provider, ensure the server is running before init
        if provider_type == ProviderType::Local {
            emit("Starting local Lore server...");
            let rt = get_tokio_runtime();
            let server_manager = ServerManager::new(base_dir);
            rt.block_on(server_manager.ensure_running())?;
            emit("✓ Local Lore server is running.");
        }

        cmd_init_universe(base_dir, universe_name, remote)?;
    } else if !should_configure_provider {
        // No repository, no --provider → nothing to do
        anyhow::bail!("Usage: nap init <repository>  or  nap init --provider <type>");
    }

    Ok(())
}

fn cmd_init_universe(base_dir: &Path, repository: &str, remote: Option<&str>) -> Result<()> {
    // 1. Create a temporary path for atomic initialization
    // Use a non-dot, valid resource_id prefix so LoreBackend::init (which derives
    // repo_id from path.file_name()) never creates grpcs://…/.__nap_init_… on the
    // remote — that name is rejected by store.validate_resource (dot-prefix).
    // The temp is still hidden from `nap list` because it is renamed atomically
    // to `base_dir/<repository>` on success; the dot was never needed for listing.
    let tmp_suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    // Use repository name in tmp so LoreBackend::init (which derives repo_id from
    // path.file_name()) creates the intended remote, not a generic nap_init_… .
    // Still atomic: tmp is created then renamed to final. Sanitize to valid id.
    let safe_repo = repository
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let tmp_path = base_dir.join(format!("{}_{}", safe_repo, tmp_suffix));

    // 2. Perform initialization in temporary path
    emit("Creating repository repository...");
    // Hint LoreBackend::from_env to read provider.toml from base_dir (not NAP_DIR) for portals-cloud
    let vcs: Option<Box<dyn nap_core::vcs::VcsBackend>> =
        if nap_core::provider::version_control_configured(base_dir) {
            // Set hint for vcs_lore to read the correct provider.toml
            unsafe { std::env::set_var("NAP_INIT_BASE_DIR", base_dir) };
            let backend = Box::new(LoreBackend::from_env());
            unsafe { std::env::remove_var("NAP_INIT_BASE_DIR") };
            Some(backend)
        } else {
            None
        };
    let result = Repository::init_optional(&tmp_path, repository, vcs);

    match result {
        Ok(repo) => {
            // 3. Success: rename to final destination
            let final_path = base_dir.join(repository);
            std::fs::rename(&tmp_path, &final_path).context(format!(
                "failed to move initialized repository to {}",
                final_path.display()
            ))?;

            emit(format!(
                "✓ Initialized repository '{repository}' at {}",
                final_path.display()
            ));

            if let Some(url) = remote {
                repo.add_remote("origin", url)
                    .context(format!("failed to add remote origin '{url}'"))?;
                emit(format!("  Added remote 'origin' → {url}"));
            }
            Ok(())
        }
        Err(e) => {
            // 4. Failure: clean up temporary path
            std::fs::remove_dir_all(&tmp_path).ok();
            Err(e.into())
        }
    }
}

fn cmd_install(_base_dir: &Path, target: &str) -> Result<()> {
    match target {
        "lore" => {
            let installer = LoreInstaller::new(None);
            emit("Installing Lore CLI and server...");
            installer.install_all()?;
            emit("✓ Lore CLI and server installed successfully.");
        }
        "mcp" => {
            emit("nap-mcp-server is bundled with the standard NAP installer.");
            emit("To repair a missing MCP server, rerun:");
            emit(
                "  curl -fsSL https://github.com/portalshq/narrativeengine/releases/latest/download/install.sh | bash",
            );
            emit("");
            emit("To use with Codex, add to your config:");
            emit(r#"  "mcpServers": { "nap": { "command": "nap-mcp-server" } }"#);
        }
        _ => anyhow::bail!("Unknown target '{}'. Available: 'lore', 'mcp'", target),
    }
    Ok(())
}

fn cmd_choose(base_dir: &Path, cmd: ChooseCmd) -> Result<()> {
    match cmd {
        ChooseCmd::Backend {
            provider,
            remote_url,
            workspace_id,
            reset,
        } => {
            if reset {
                let config_path = base_dir.join("provider.toml");
                if config_path.exists() {
                    std::fs::remove_file(&config_path)
                        .context("failed to reset provider configuration")?;
                    emit("✓ Reset provider configuration.");
                }
            }
            let provider_type = ProviderType::parse_from_str(&provider)
                .context(format!("invalid provider type '{provider}'"))?;

            let factory = ProviderFactory::new(base_dir);
            let provider = match provider_type {
                ProviderType::Local => factory.create_provider(ProviderType::Local)?,
                ProviderType::PortalsCloud => {
                    factory.create_provider(ProviderType::PortalsCloud)?
                }
                ProviderType::Remote => {
                    let url = remote_url
                        .as_ref()
                        .context("remote provider requires --remote-url")?;
                    let ws_id = workspace_id
                        .as_ref()
                        .context("remote provider requires --workspace-id")?;
                    factory.create_remote_provider(url, ws_id)?
                }
            };

            let mut provider_manager = ProviderManager::new(base_dir);
            provider_manager.set_active_provider(provider.clone());
            provider_manager
                .save_provider_config(provider.as_ref())
                .context("failed to save provider configuration")?;

            // Initialize and verify the provider
            let rt = get_tokio_runtime();
            rt.block_on(provider.initialize())
                .context("failed to initialize provider")?;

            emit(format!("✓ Switched to {}.", provider.name()));
            emit(format!("  Type: {}", provider_type.as_str()));
            if let Some(url) = &remote_url {
                emit(format!("  Remote URL: {}", url));
            }
            if let Some(ws_id) = &workspace_id {
                emit(format!("  Workspace ID: {}", ws_id));
            }
        }
    }
    Ok(())
}

fn cmd_backend(base_dir: &Path, cmd: BackendCmd) -> Result<()> {
    match cmd {
        BackendCmd::Configure {
            backend,
            endpoint,
            workspace_id,
            initial_commit,
            no_initial_commit,
        } => {
            if initial_commit && no_initial_commit {
                anyhow::bail!("--initial-commit and --no-initial-commit are mutually exclusive");
            }

            let provider_type = match backend.as_str() {
                "local" => ProviderType::Local,
                "remote" => ProviderType::Remote,
                other => {
                    anyhow::bail!("Unknown backend '{}'. Available: 'local', 'remote'", other)
                }
            };

            let factory = ProviderFactory::new(base_dir);
            let provider = match provider_type {
                ProviderType::Local => factory.create_provider(ProviderType::Local)?,
                ProviderType::Remote => {
                    let url = endpoint
                        .as_ref()
                        .context("remote backend requires --endpoint <url>")?;
                    let ws_id = workspace_id
                        .clone()
                        .unwrap_or_else(|| "default".to_string());
                    factory.create_remote_provider(url, &ws_id)?
                }
                ProviderType::PortalsCloud => {
                    anyhow::bail!("'portals-cloud' is not supported as a `nap backend` backend")
                }
            };

            let mut provider_manager = ProviderManager::new(base_dir);
            provider_manager.set_active_provider(provider.clone());
            provider_manager
                .save_provider_config(provider.as_ref())
                .context("failed to save provider configuration")?;

            let rt = get_tokio_runtime();
            rt.block_on(provider.initialize())
                .context("failed to initialize provider")?;

            emit(format!("✓ Configured {} backend.", provider_type.as_str()));
            if let Some(url) = &endpoint {
                emit(format!("  Endpoint: {}", url));
            }
            if let Some(ws_id) = &workspace_id {
                emit(format!("  Workspace ID: {}", ws_id));
            }

            if no_initial_commit {
                emit("  Skipped initial commit for existing repositories (--no-initial-commit).");
            } else {
                bootstrap_repositories(base_dir, initial_commit)?;
            }
        }
        BackendCmd::Status => {
            if nap_core::provider::version_control_configured(base_dir) {
                let mut provider_manager = ProviderManager::new(base_dir);
                if let Some(provider) = provider_manager.load_configured_provider()? {
                    emit(format!(
                        "✓ Version-control backend configured: {}",
                        provider.name()
                    ));
                    emit(format!("  Type: {}", provider.provider_type().as_str()));
                    if let Ok(url) = provider.lore_url_base() {
                        emit(format!("  Lore URL: {}", url));
                    }
                    emit(format!("  Workspace ID: {}", provider.workspace_id()));
                } else {
                    emit("Version-control backend configured (no provider details).");
                }
            } else {
                emit("No version-control backend configured. Run 'nap backend configure local'.");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

/// Offer to create an initial commit for repositories that exist on the
/// filesystem but were created before a version-control backend was configured.
fn bootstrap_repositories(base_dir: &Path, assume_yes: bool) -> Result<()> {
    if !nap_core::provider::version_control_configured(base_dir) {
        return Ok(());
    }

    let mut candidates = Vec::new();
    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("repository.yaml").exists() {
                candidates.push(path);
            }
        }
    }
    candidates.sort();

    if candidates.is_empty() {
        emit("  No existing repositories to bootstrap.");
        return Ok(());
    }

    for repo_dir in candidates {
        let name = repo_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("(unnamed)");

        let vcs: Box<dyn nap_core::vcs::VcsBackend> = Box::new(get_lore_backend(base_dir));
        let repo = match Repository::open_optional(&repo_dir, Some(vcs)) {
            Ok(repo) => repo,
            Err(e) => {
                emit(format!("  ⚠ Skipping '{}': {}", name, e));
                continue;
            }
        };

        // Skip repos that already have a VCS head.
        if repo.head_hash().is_ok() {
            emit(format!("  ✓ '{}' already versioned.", name));
            continue;
        }

        let proceed = if assume_yes {
            true
        } else {
            let mut input = String::new();
            print!("  Bootstrap '{}' with an initial commit? [Y/n] ", name);
            io::stdout().flush().ok();
            io::stdin().read_line(&mut input)?;
            let trimmed = input.trim().to_ascii_lowercase();
            !(trimmed == "n" || trimmed == "no")
        };

        if !proceed {
            emit(format!("  Skipped '{}'.", name));
            continue;
        }

        match repo.bootstrap_vcs(INITIAL_COMMIT_MESSAGE, "nap") {
            Ok(hash) => emit(format!("  ✓ Bootstrapped '{}' at {}.", name, hash)),
            Err(e) => emit(format!("  ⚠ Failed to bootstrap '{}': {}", name, e)),
        }
    }
    Ok(())
}

const INITIAL_COMMIT_MESSAGE: &str = "Initialize existing NAP repository";

fn cmd_doctor(base_dir: &Path, repair: bool) -> Result<()> {
    let doctor = NapDoctor::new(base_dir);

    // Check configured provider type
    let mut provider_manager = ProviderManager::new(base_dir);
    let provider_type = provider_manager
        .load_configured_provider()
        .map(|p| p.map(|p| p.provider_type()))
        .unwrap_or(None);

    // Block repair for non-local providers
    if repair {
        if let Some(ProviderType::Local) = provider_type {
            // Allow repair for local provider
        } else {
            anyhow::bail!(
                "nap doctor --repair is only available for the local provider. \
                 Your configured provider is {}. \
                 Repair operations are not needed for remote or cloud providers.",
                provider_type
                    .map(|t| t.as_str().to_string())
                    .unwrap_or("none".to_string())
            );
        }
    }

    // Use shared tokio runtime for async doctor operations
    let rt = get_tokio_runtime();

    let report = rt
        .block_on(doctor.diagnose())
        .context("failed to run diagnostics")?;

    if std::io::stdout().is_terminal() {
        emit("NAP Doctor Report");
        emit("==================");

        // Add provider context message
        if let Some(pt) = provider_type {
            match pt {
                ProviderType::Local => {
                    emit("Provider: local (doctor checks apply)");
                }
                ProviderType::Remote | ProviderType::PortalsCloud => {
                    emit(format!(
                        "Provider: {} (doctor checks apply only to local provider)",
                        pt.as_str()
                    ));
                }
            }
        } else {
            emit("Provider: not configured (doctor checks apply only to local provider)");
        }
        emit(String::new());

        for check in &report.checks {
            let status = if check.passed { "✓" } else { "✗" };
            let severity = match check.severity {
                nap_core::server::CheckSeverity::Info => "INFO",
                nap_core::server::CheckSeverity::Warning => "WARN",
                nap_core::server::CheckSeverity::Error => "ERROR",
            };
            emit(format!("{} [{}] {}", status, severity, check.name));
            if !check.message.is_empty() {
                emit(format!("  {}", check.message));
            }
        }

        let passed = report.checks.iter().filter(|c| c.passed).count();
        let total = report.checks.len();
        emit(String::new());
        emit(format!("Summary: {}/{} checks passed", passed, total));
    } else {
        // Manual JSON output for report since it doesn't implement Serialize
        let checks_json: Vec<serde_json::Value> = report
            .checks
            .iter()
            .map(|c| {
                serde_json::json!({
                    "name": c.name,
                    "passed": c.passed,
                    "severity": format!("{:?}", c.severity),
                    "message": c.message,
                })
            })
            .collect();
        let output = serde_json::json!({
            "nap_home": report.nap_home,
            "checks": checks_json,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    if repair {
        let repair_report = rt
            .block_on(doctor.repair(&report))
            .context("failed to run repairs")?;

        if std::io::stdout().is_terminal() {
            emit(String::new());
            emit("Repair Results");
            emit("===============");
            for repair_result in &repair_report.repairs {
                let status = if repair_result.success { "✓" } else { "✗" };
                emit(format!("{} {}", status, repair_result.check_name));
                emit(format!("  {}", repair_result.message));
            }
        } else {
            // Manual JSON output for repair report since it doesn't implement Serialize
            let repairs_json: Vec<serde_json::Value> = repair_report
                .repairs
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "check_name": r.check_name,
                        "success": r.success,
                        "message": r.message,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "repairs": repairs_json,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}

fn cmd_publish(base_dir: &Path, repository: &str) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    repo.push(Some("origin"), None)
        .context("failed to publish to remote")?;
    emit(format!("✓ Published '{repository}'."));
    Ok(())
}

fn cmd_status(base_dir: &Path) -> Result<()> {
    let mut provider_manager = ProviderManager::new(base_dir);

    if let Some(provider) = provider_manager.load_configured_provider()? {
        let rt = get_tokio_runtime();

        let status = rt
            .block_on(provider.status())
            .context("failed to get provider status")?;

        if std::io::stdout().is_terminal() {
            emit("NAP Status");
            emit("==========");
            emit(format!("Provider: {}", status.provider_type.as_str()));
            emit(format!(
                "Ready: {}",
                if status.ready { "Yes" } else { "No" }
            ));
            emit(format!(
                "Healthy: {}",
                if status.healthy { "Yes" } else { "No" }
            ));
            emit(format!("URL: {}", status.url_base));
            emit(format!("Workspace: {}", status.workspace_id));
            emit(format!("Message: {}", status.message));
        } else {
            // Manual JSON output for status since it doesn't implement Serialize
            let output = serde_json::json!({
                "provider_type": format!("{:?}", status.provider_type),
                "ready": status.ready,
                "healthy": status.healthy,
                "url_base": status.url_base,
                "workspace_id": status.workspace_id,
                "message": status.message,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    } else {
        emit("No provider configured. Run 'nap init' to setup.");
    }

    Ok(())
}

fn cmd_sync(base_dir: &Path, repository: &str) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    repo.pull(None, None)
        .context("failed to sync from remote")?;
    emit(format!("✓ Synced '{repository}'."));
    Ok(())
}

/// Get or create a shared tokio runtime for async operations
fn get_tokio_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().expect("failed to create tokio runtime"))
}

fn cmd_create(
    base_dir: &Path,
    repository: &str,
    entity_type_str: &str,
    entity_id: &str,
    name: &str,
    author: &str,
) -> Result<()> {
    let entity_type = EntityType::new(entity_type_str);
    let repo = open_repo(base_dir, repository)?;
    let (manifest, hash) = repo
        .create_entity(&entity_type, entity_id, name, author)
        .context("failed to create entity")?;
    emit(format!("✓ Created {entity_type} '{name}'."));
    emit(format!("  URI:    {}", manifest.id));
    emit(format!("  Commit: {}", &hash[..hash.len().min(12)]));
    Ok(())
}

fn cmd_resolve(
    base_dir: &Path,
    uri_str: &str,
    branch: Option<String>,
    commit: Option<String>,
    format: &str,
    provenance: bool,
    include_blobs: bool,
) -> Result<()> {
    let resolver = Resolver::new(base_dir);
    let wants_provenance = provenance || include_blobs;
    let options = ResolveOptions {
        branch,
        commit,
        path: None,
        recursive: Some(!wants_provenance),
        max_depth: None,
        provenance: Some(wants_provenance),
        include_blobs: Some(include_blobs),
    };
    let result = resolver
        .resolve(uri_str, &options)
        .context(format!("failed to resolve '{uri_str}'"))?;

    let fmt = resolve_output_format(format);
    match result {
        ResolveResult::Full(manifest) => match fmt.as_str() {
            "json" => println!("{}", serde_json::to_string_pretty(&manifest)?),
            _ => println!("{}", serde_yaml::to_string(&manifest)?),
        },
        ResolveResult::Provenance(envelope) => match fmt.as_str() {
            "json" => println!("{}", serde_json::to_string_pretty(&envelope)?),
            _ => println!("{}", serde_yaml::to_string(&envelope)?),
        },
        ResolveResult::Subtree(value) => match fmt.as_str() {
            "yaml" => {
                let yaml: serde_yaml::Value = serde_json::from_value(value)?;
                println!("{}", serde_yaml::to_string(&yaml)?);
            }
            _ => println!("{}", serde_json::to_string_pretty(&value)?),
        },
    }
    Ok(())
}

fn cmd_query(base_dir: &Path, uri_str: &str, path: &str, format: &str) -> Result<()> {
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

fn cmd_commit(base_dir: &Path, repository: &str, message: &str, author: &str) -> Result<()> {
    let repo_path = base_dir.join(repository);
    let vcs = require_backend(base_dir, "commit changes")?;
    let hash = nap_core::vcs::VcsBackend::commit(&*vcs, &repo_path, message, author)
        .context("failed to commit")?;
    emit(format!("✓ Committed: {} ({})", message, &hash[..12]));
    Ok(())
}

fn cmd_history(base_dir: &Path, uri_str: &str, limit: usize) -> Result<()> {
    let uri: NapUri = uri_str.parse().context("invalid URI")?;
    let repo = open_repo(base_dir, &uri.repository)?;
    let history = repo
        .history(&uri.entity_type, &uri.entity_id, limit)
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
            println!(
                "{} {} — {} ({})",
                short_hash, entry.timestamp, entry.message, entry.author
            );
        }
    } else {
        // Piped: emit full JSON array
        println!("{}", serde_json::to_string_pretty(&history)?);
    }
    Ok(())
}

fn cmd_list(base_dir: &Path, repository: Option<&str>, entity_type: Option<&str>) -> Result<()> {
    let is_piped = !std::io::stdout().is_terminal();

    match repository {
        None => {
            let resolver = Resolver::new(base_dir);
            let repositories = resolver
                .list_repositories()
                .context("failed to list repositories")?;
            if is_piped {
                println!("{}", serde_json::to_string_pretty(&repositories)?);
            } else if repositories.is_empty() {
                println!("No repositories found in {}", base_dir.display());
            } else {
                println!("Repositories:");
                for u in &repositories {
                    println!("  nap://{u}/");
                }
            }
        }
        Some(repository) => {
            let repo = open_repo(base_dir, repository)?;
            let is_piped = !std::io::stdout().is_terminal();
            match entity_type {
                Some(et_str) => {
                    let et = EntityType::new(et_str);
                    let entities = repo.list_entities(&et).context("failed to list entities")?;
                    if is_piped {
                        println!("{}", serde_json::to_string_pretty(&entities)?);
                    } else {
                        println!("{} in {repository}:", et_str);
                        for e in &entities {
                            println!("  nap://{repository}/{et}/{e}");
                        }
                    }
                }
                None => {
                    // Discover all entity types dynamically
                    let types = repo
                        .list_entity_types()
                        .context("failed to list entity types")?;
                    let mut all: Vec<serde_json::Value> = Vec::new();
                    for et in &types {
                        let entities = repo.list_entities(et).unwrap_or_default();
                        if is_piped && !entities.is_empty() {
                            for e in &entities {
                                all.push(serde_json::json!({
                                    "type": et.to_string(),
                                    "id": e,
                                    "uri": format!("nap://{repository}/{et}/{e}"),
                                }));
                            }
                        } else if !entities.is_empty() {
                            println!("{}:", et);
                            for e in &entities {
                                println!("  nap://{repository}/{et}/{e}");
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

fn cmd_branch(base_dir: &Path, repository: &str, name: Option<&str>) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    match name {
        Some(branch_name) => {
            repo.create_branch(branch_name)
                .context(format!("failed to create branch '{branch_name}'"))?;
            emit(format!("✓ Created branch '{branch_name}' in {repository}"));
        }
        None => {
            let branches = repo.list_branches().context("failed to list branches")?;
            if !std::io::stdout().is_terminal() {
                println!("{}", serde_json::to_string_pretty(&branches)?);
            } else {
                println!("Branches in {repository}:");
                for b in &branches {
                    println!("  {b}");
                }
            }
        }
    }
    Ok(())
}

fn cmd_pull(base_dir: &Path, url_or_name: &str) -> Result<()> {
    if looks_like_url(url_or_name) {
        // ── Clone from URL ──────────────────────────────────────
        require_backend(base_dir, "clone repository")?;

        let tmp_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_name = format!(".__nap_clone_{tmp_suffix}");
        let tmp_path = base_dir.join(&tmp_name);

        emit(format!("  Cloning from {url_or_name} …"));
        LoreBackend::clone_repo(url_or_name, &tmp_path).context("failed to clone repository")?;

        // Read the repository name — prefer .nap/config.yaml, fall back to
        // repository.yaml or URL last segment (lore 0.8.4-portals.8 creates .lore, not .nap).
        let config_path = tmp_path.join(".nap").join("config.yaml");
        let repo_yaml_path = tmp_path.join("repository.yaml");
        let name = if config_path.exists() {
            let config_content = std::fs::read_to_string(&config_path)
                .context("cloned repo is missing or corrupt .nap/config.yaml")?;
            let config_yaml: serde_yaml::Value = serde_yaml::from_str(&config_content)
                .context("invalid .nap/config.yaml in cloned repo")?;
            config_yaml["repository"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("missing 'repository' key in .nap/config.yaml"))?
        } else if repo_yaml_path.exists() {
            // Fallback: lore creates repository.yaml with id: nap://<repo>/world/<repo>
            let content = std::fs::read_to_string(&repo_yaml_path)
                .context("cloned repo missing repository.yaml")?;
            let yaml: serde_yaml::Value =
                serde_yaml::from_str(&content).context("invalid repository.yaml")?;
            yaml.get("id")
                .and_then(|id| id.as_str())
                .and_then(|id_str| id_str.strip_prefix("nap://"))
                .and_then(|rest| rest.split('/').next().map(|s| s.to_string()))
                .or_else(|| {
                    // Fallback to URL last segment
                    url_or_name.rsplit('/').next().map(|s| s.to_string())
                })
                .ok_or_else(|| anyhow::anyhow!("cannot determine repository name from clone"))?
        } else {
            // Final fallback: URL last segment
            url_or_name
                .rsplit('/')
                .next()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("cannot determine repository name from URL"))?
        };

        // Check if the target directory already exists
        let target = base_dir.join(&name);
        if target.exists() {
            // Clean up the temp clone
            std::fs::remove_dir_all(&tmp_path).context("failed to clean up temp clone")?;
            anyhow::bail!("repository '{name}' already exists at {}", target.display());
        }

        // Rename temp → final
        std::fs::rename(&tmp_path, &target)
            .context(format!("failed to rename {tmp_name} → {name}"))?;

        emit(format!(
            "✓ Cloned repository '{name}' to {}",
            target.display()
        ));
    } else {
        // ── Pull existing repo OR clone by name ───────────────────
        let target_dir = base_dir.join(url_or_name);
        if target_dir.exists() {
            // Already exists locally, pull latest changes
            let repo = open_repo(base_dir, url_or_name)?;
            repo.pull(None, None)
                .context("failed to pull latest changes")?;
            emit(format!("✓ Pulled latest changes for '{url_or_name}'"));
        } else {
            // Doesn't exist locally, construct URL and clone
            require_backend(base_dir, "clone repository")?;
            let backend_config = LoreBackend::from_env();
            let remote_url = format!("{}/{}", backend_config.remote_url(), url_or_name);

            // Perform clone
            let tmp_suffix = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let tmp_name = format!(".__nap_clone_{tmp_suffix}");
            let tmp_path = base_dir.join(&tmp_name);

            emit(format!("  Cloning from {remote_url} …"));
            LoreBackend::clone_repo(&remote_url, &tmp_path)
                .context("failed to clone repository")?;

            // Rename temp → final
            let target = base_dir.join(url_or_name);
            std::fs::rename(&tmp_path, &target)
                .context(format!("failed to rename {tmp_name} → {url_or_name}"))?;

            emit(format!(
                "✓ Cloned repository '{url_or_name}' to {}",
                target.display()
            ));
        }
    }

    Ok(())
}

fn cmd_push(base_dir: &Path, repository: &str, remote: &str, branch: Option<&str>) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    repo.push(Some(remote), branch)
        .context("failed to push to remote")?;
    match branch {
        Some(b) => emit(format!("✓ Pushed '{repository}' ({b}) → {remote}")),
        None => emit(format!("✓ Pushed '{repository}' → {remote}")),
    }
    Ok(())
}

fn cmd_remote(base_dir: &Path, cmd: RemoteCmd) -> Result<()> {
    match cmd {
        RemoteCmd::Add {
            repository,
            name,
            url,
        } => {
            let repo = open_repo(base_dir, &repository)?;
            repo.add_remote(&name, &url)
                .context(format!("failed to add remote '{name}'"))?;
            emit(format!("✓ Added remote '{name}' → {url} to '{repository}'"));
        }
        RemoteCmd::Ls { repository } => {
            let repo = open_repo(base_dir, &repository)?;
            let remotes = repo.list_remotes().context("failed to list remotes")?;
            if remotes.is_empty() {
                emit(format!("No remotes configured for '{repository}'"));
            } else {
                if std::io::stdout().is_terminal() {
                    println!("Remotes in '{repository}':");
                    for (name, url) in &remotes {
                        println!("  {name}\t{url}");
                    }
                } else {
                    let pairs: Vec<serde_json::Value> = remotes
                        .iter()
                        .map(|(n, u)| serde_json::json!({ "name": n, "url": u }))
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&pairs)?);
                }
            }
        }
        RemoteCmd::Rm { repository, name } => {
            let repo = open_repo(base_dir, &repository)?;
            repo.remove_remote(&name)
                .context(format!("failed to remove remote '{name}'"))?;
            emit(format!("✓ Removed remote '{name}' from '{repository}'"));
        }
    }
    Ok(())
}

fn cmd_set(
    base_dir: &Path,
    uri_str: &str,
    key: &str,
    value: &str,
    message: &str,
    author: &str,
) -> Result<()> {
    let uri: NapUri = uri_str.parse().context("invalid URI")?;
    let repo = open_repo(base_dir, &uri.repository)?;
    let mut manifest = repo
        .read_manifest(&uri.entity_type, &uri.entity_id)
        .context("failed to read manifest")?;

    // Parse value — try as YAML for structured values, fallback to string
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(value)
        .unwrap_or_else(|_| serde_yaml::Value::String(value.to_string()));

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
    base_dir: &Path,
    uri_str: &str,
    key: &str,
    file: &Path,
    format: &str,
    message: &str,
    author: &str,
) -> Result<()> {
    let uri: NapUri = uri_str.parse().context("invalid URI")?;
    let repo = open_repo(base_dir, &uri.repository)?;
    let mut manifest = repo
        .read_manifest(&uri.entity_type, &uri.entity_id)
        .context("failed to read manifest")?;

    // Compute content hash
    let hash = nap_core::ContentHash::from_file(file)
        .context(format!("failed to hash file '{}'", file.display()))?;

    // Copy file to repository and stage it for commit
    // Lore stores files in the immutable store when they're committed
    let entity_dir = repo
        .root
        .join(uri.entity_type.to_string())
        .join(&uri.entity_id);
    std::fs::create_dir_all(&entity_dir).context(format!(
        "failed to create entity directory '{}'",
        entity_dir.display()
    ))?;

    let asset_filename = format!("{}.{}", key, format);
    let asset_path = entity_dir.join(&asset_filename);
    std::fs::copy(file, &asset_path).context(format!(
        "failed to copy asset file to '{}'",
        asset_path.display()
    ))?;

    // Stage the asset file in the repository (versioned mode only; the copy
    // above is the durable change in unversioned mode).
    let asset_path_str = asset_path.display().to_string();
    if nap_core::provider::version_control_configured(base_dir) {
        let args = vec![
            "file",
            "stage",
            "--scan",
            &asset_path_str,
            "--non-interactive",
        ];
        nap_core::vcs_lore::LoreProcessRunner::run(&args, Some(&repo.root))
            .context("failed to stage asset file in repository")?;
    }

    // Store content hash directly (Lore's immutable store is content-addressed)
    let repr = Representation {
        hash: hash.as_str().to_string(),
        format: format.to_string(),
        uri: Some(asset_filename), // Store relative path to the asset file
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

    emit(format!(
        "✓ Added representation '{key}' ({format}) to {uri_str}"
    ));
    emit(format!("  Hash: {hash}"));
    emit(format!(
        "  Stored in Lore immutable store: {}",
        hash.as_str()
    ));
    Ok(())
}

fn cmd_revert(base_dir: &Path, repository: &str, commit: &str, author: &str) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    let new_hash = repo
        .revert_commit(commit, author)
        .context(format!("failed to revert commit '{commit}'"))?;
    let short_old = if commit.len() > 12 {
        &commit[..12]
    } else {
        commit
    };
    let short_new = &new_hash[..12.min(new_hash.len())];
    emit(format!(
        "✓ Reverted commit {short_old} — new commit: {short_new}"
    ));
    Ok(())
}

fn cmd_switch(base_dir: &Path, repository: &str, name: &str) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    repo.switch_branch(name)
        .context(format!("failed to switch to branch '{name}'"))?;
    emit(format!("✓ Switched to branch '{name}' in {repository}"));
    Ok(())
}

fn cmd_head_hash(base_dir: &Path, repository: &str) -> Result<()> {
    let repo = open_repo(base_dir, repository)?;
    let hash = repo.head_hash().context("failed to get HEAD hash")?;
    if !std::io::stdout().is_terminal() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "repository": repository,
                "head": hash,
            }))?
        );
    } else {
        emit(format!("HEAD: {hash}"));
    }
    Ok(())
}

fn cmd_validate(base_dir: &Path, uri: Option<&str>, file: Option<&Path>) -> Result<()> {
    match (uri, file) {
        (Some(uri_str), None) => {
            // Validate entity manifest by URI
            let uri_parsed: NapUri = uri_str.parse().context("invalid URI")?;
            let repo = open_repo(base_dir, &uri_parsed.repository)?;
            let manifest = repo
                .read_manifest(&uri_parsed.entity_type, &uri_parsed.entity_id)
                .context("failed to read manifest")?;
            match nap_core::schema::validate_manifest(&manifest) {
                Ok(()) => {
                    if !std::io::stdout().is_terminal() {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "valid": true,
                                "uri": uri_str,
                            }))?
                        );
                    } else {
                        emit(format!("✓ '{uri_str}' is valid"));
                    }
                }
                Err(errors) => {
                    if !std::io::stdout().is_terminal() {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "valid": false,
                                "uri": uri_str,
                                "errors": errors,
                            }))?
                        );
                    } else {
                        emit(format!("✗ '{uri_str}' is invalid:"));
                        for err in &errors {
                            emit(format!("  - {err}"));
                        }
                    }
                }
            }
        }
        (None, Some(file_path)) => {
            // Validate a YAML manifest file
            let manifest = nap_core::Manifest::from_file(file_path)
                .context("failed to parse manifest file")?;
            match nap_core::schema::validate_manifest(&manifest) {
                Ok(()) => {
                    if !std::io::stdout().is_terminal() {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "valid": true,
                                "file": file_path.to_string_lossy(),
                            }))?
                        );
                    } else {
                        emit(format!("✓ '{}' is valid", file_path.display()));
                    }
                }
                Err(errors) => {
                    if !std::io::stdout().is_terminal() {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "valid": false,
                                "file": file_path.to_string_lossy(),
                                "errors": errors,
                            }))?
                        );
                    } else {
                        emit(format!("✗ '{}' is invalid:", file_path.display()));
                        for err in &errors {
                            emit(format!("  - {err}"));
                        }
                    }
                }
            }
        }
        _ => {
            anyhow::bail!(
                "Provide either a NAP URI (nap validate <uri>) or a manifest file (nap validate --file <path>)"
            )
        }
    }
    Ok(())
}

fn cmd_schema(name: &str, format: &str) -> Result<()> {
    let schema = match name {
        "manifest" => nap_core::schema::manifest_schema(),
        "commit" => nap_core::schema::commit_schema(),
        _ => anyhow::bail!("Unknown schema '{name}'. Available: 'manifest', 'commit'"),
    };

    let fmt = resolve_output_format(format);
    match fmt.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&schema)?),
        _ => {
            let yaml: serde_yaml::Value = serde_json::from_value(schema)?;
            println!("{}", serde_yaml::to_string(&yaml)?);
        }
    }
    Ok(())
}

fn cmd_diff(base_file: &Path, candidate_file: &Path, format: &str) -> Result<()> {
    // Read and parse both files
    let base_content = std::fs::read_to_string(base_file)
        .context(format!("failed to read '{}'", base_file.display()))?;
    let candidate_content = std::fs::read_to_string(candidate_file)
        .context(format!("failed to read '{}'", candidate_file.display()))?;

    // Parse as YAML then convert to JSON Value
    let base_yaml: serde_yaml::Value = serde_yaml::from_str(&base_content)
        .context(format!("failed to parse YAML in '{}'", base_file.display()))?;
    let candidate_yaml: serde_yaml::Value = serde_yaml::from_str(&candidate_content).context(
        format!("failed to parse YAML in '{}'", candidate_file.display()),
    )?;

    let base_value: serde_json::Value = serde_json::to_value(base_yaml)
        .map_err(|e| anyhow::anyhow!("YAML→JSON conversion failed: {e}"))?;
    let candidate_value: serde_json::Value = serde_json::to_value(candidate_yaml)
        .map_err(|e| anyhow::anyhow!("YAML→JSON conversion failed: {e}"))?;

    // Build a minimal SDL for diffing
    use nap_core::merge::sdl::SdlDocument;
    let sdl = SdlDocument::from_yaml(
        r#"schema:
  version: "1.0"
  required: []
  properties: {}
"#,
    )
    .context("failed to create default SDL")?;

    use nap_core::merge::diff::diff;
    let result = diff(&base_value, &candidate_value, &sdl);

    let fmt = resolve_output_format(format);
    match fmt.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&result)?),
        _ => {
            let yaml: serde_yaml::Value = serde_json::from_value(serde_json::to_value(&result)?)?;
            println!("{}", serde_yaml::to_string(&yaml)?);
        }
    }
    Ok(())
}

fn cmd_merge(
    base_file: &Path,
    current_file: &Path,
    proposed_file: &Path,
    format: &str,
) -> Result<()> {
    // Read and parse all three files
    let read_file = |path: &Path| -> Result<serde_json::Value> {
        let content = std::fs::read_to_string(path)
            .context(format!("failed to read '{}'", path.display()))?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .context(format!("failed to parse YAML in '{}'", path.display()))?;
        serde_json::to_value(yaml).map_err(|e| anyhow::anyhow!("YAML→JSON conversion failed: {e}"))
    };

    let base = read_file(base_file)?;
    let current = read_file(current_file)?;
    let proposed = read_file(proposed_file)?;

    // Build minimal SDL and merge engine
    use nap_core::merge::merge_engine::MergeEngine;
    use nap_core::merge::sdl::SdlDocument;
    let sdl = SdlDocument::from_yaml(
        r#"schema:
  version: "1.0"
  required: []
  properties: {}
"#,
    )
    .context("failed to create default SDL")?;
    let engine = MergeEngine::new(sdl);

    use nap_core::merge::conflict::MergeResult;
    match engine.merge(base, current, proposed) {
        MergeResult::Merged(merged) => {
            let fmt = resolve_output_format(format);
            match fmt.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&merged)?),
                _ => {
                    let yaml: serde_yaml::Value = serde_json::from_value(merged)?;
                    println!("{}", serde_yaml::to_string(&yaml)?);
                }
            }
        }
        MergeResult::Conflicts(conflicts) => {
            if !std::io::stdout().is_terminal() {
                println!("{}", serde_json::to_string_pretty(&conflicts)?);
            } else {
                emit(format!("✗ Merge conflicts detected ({}):", conflicts.len()));
                for c in &conflicts {
                    emit(format!("  - {}: {:?}", c.path, c.conflict_type));
                }
            }
        }
    }
    Ok(())
}

fn cmd_content_hash(file: &Path) -> Result<()> {
    let hash = nap_core::ContentHash::from_file(file)
        .context(format!("failed to hash file '{}'", file.display()))?;
    if !std::io::stdout().is_terminal() {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "file": file.to_string_lossy(),
                "hash": hash.as_str(),
                "algorithm": "blake3",
            }))?
        );
    } else {
        emit(format!("{}  {}", hash, file.display()));
    }
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
