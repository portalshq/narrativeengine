# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_px_global_optspecs
    string join \n d/base-dir= v/verbose remote local h/help V/version
end

function __fish_px_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_px_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_px_using_subcommand
    set -l cmd (__fish_px_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c px -n "__fish_px_needs_command" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_needs_command" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_needs_command" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_needs_command" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_needs_command" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_needs_command" -s V -l version -d 'Print version'
complete -c px -n "__fish_px_needs_command" -f -a "auth" -d 'Manage secure authentication for the configured Lore provider'
complete -c px -n "__fish_px_needs_command" -f -a "install" -d 'Install required dependencies'
complete -c px -n "__fish_px_needs_command" -f -a "init" -d 'Initialize a repository repository and/or configure the backend provider'
complete -c px -n "__fish_px_needs_command" -f -a "configure" -d 'Configure version-control backend (unified: replaces `px choose` + `px backend`)'
complete -c px -n "__fish_px_needs_command" -f -a "choose" -d 'Choose backend provider (deprecated: use `px configure`)'
complete -c px -n "__fish_px_needs_command" -f -a "backend" -d 'Configure or inspect the version-control backend (deprecated: use `px configure`)'
complete -c px -n "__fish_px_needs_command" -f -a "completions" -d 'Generate shell completions for `px`'
complete -c px -n "__fish_px_needs_command" -f -a "doctor" -d 'Run diagnostics and repair'
complete -c px -n "__fish_px_needs_command" -f -a "status" -d 'Show system status'
complete -c px -n "__fish_px_needs_command" -f -a "sync" -d 'Sync with remote'
complete -c px -n "__fish_px_needs_command" -f -a "create" -d 'Create a new entity manifest'
complete -c px -n "__fish_px_needs_command" -f -a "resolve" -d 'Resolve a PX URI to its manifest or a subtree'
complete -c px -n "__fish_px_needs_command" -f -a "presign" -d 'Create a time-limited public URL for a committed representation'
complete -c px -n "__fish_px_needs_command" -f -a "query" -d 'Query a subtree from a manifest'
complete -c px -n "__fish_px_needs_command" -f -a "commit" -d 'Commit changes to a repository repository'
complete -c px -n "__fish_px_needs_command" -f -a "history" -d 'View commit history for an entity'
complete -c px -n "__fish_px_needs_command" -f -a "list" -d 'List repositories or entities within a repository'
complete -c px -n "__fish_px_needs_command" -f -a "branch" -d 'Create or list branches'
complete -c px -n "__fish_px_needs_command" -f -a "set" -d 'Set a property on an entity manifest'
complete -c px -n "__fish_px_needs_command" -f -a "add" -d 'Add a file representation to an entity manifest'
complete -c px -n "__fish_px_needs_command" -f -a "revert" -d 'Revert a commit by hash (undoes all changes in that commit)'
complete -c px -n "__fish_px_needs_command" -f -a "pull" -d 'Clone or pull a repository from a remote'
complete -c px -n "__fish_px_needs_command" -f -a "push" -d 'Push the current branch to its configured upstream remote'
complete -c px -n "__fish_px_needs_command" -f -a "remote" -d 'Manage remotes on a repository'
complete -c px -n "__fish_px_needs_command" -f -a "sign" -d 'Sign a manifest (stub for v0)'
complete -c px -n "__fish_px_needs_command" -f -a "verify" -d 'Verify a manifest signature (stub for v0)'
complete -c px -n "__fish_px_needs_command" -f -a "switch" -d 'Switch to a branch'
complete -c px -n "__fish_px_needs_command" -f -a "head" -d 'Show the current HEAD commit hash'
complete -c px -n "__fish_px_needs_command" -f -a "validate" -d 'Validate a manifest against the PX schema'
complete -c px -n "__fish_px_needs_command" -f -a "schema" -d 'Print a JSON Schema for manifest or commit types'
complete -c px -n "__fish_px_needs_command" -f -a "diff" -d 'Show diff between two manifest files or versions'
complete -c px -n "__fish_px_needs_command" -f -a "merge" -d 'Three-way merge of JSON/YAML values'
complete -c px -n "__fish_px_needs_command" -f -a "content-hash" -d 'Compute the BLAKE3 content hash of a file'
complete -c px -n "__fish_px_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -f -a "login" -d 'Sign in through the configured Lore authentication service'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -f -a "status" -d 'Show the currently cached Lore identity without printing tokens'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -f -a "logout" -d 'Remove locally cached Lore credentials'
complete -c px -n "__fish_px_using_subcommand auth; and not __fish_seen_subcommand_from login status logout help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -l api-key-env -d 'Environment variable containing the API key' -r
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -l api-key -d 'Exchange a service-account API key instead of opening a browser'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -l no-browser -d 'Print the login URL without opening a browser'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from login" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from status" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from status" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from status" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from logout" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from logout" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from logout" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from logout" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from logout" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "login" -d 'Sign in through the configured Lore authentication service'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show the currently cached Lore identity without printing tokens'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "logout" -d 'Remove locally cached Lore credentials'
complete -c px -n "__fish_px_using_subcommand auth; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand install" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand install" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand install" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand install" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand install" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand init" -l provider -d 'Provider type: local, portals-cloud, or remote' -r
complete -c px -n "__fish_px_using_subcommand init" -l remote-url -d 'Remote URL (required for remote provider)' -r
complete -c px -n "__fish_px_using_subcommand init" -l workspace-id -d 'Workspace ID (for remote provider)' -r
complete -c px -n "__fish_px_using_subcommand init" -l origin -d 'Remote URL to add as origin after init' -r
complete -c px -n "__fish_px_using_subcommand init" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand init" -l reset -d 'Reset the provider configuration file'
complete -c px -n "__fish_px_using_subcommand init" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand init" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand init" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand init" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l provider -d 'Provider type flag (alternative to positional `PROVIDER`)' -r
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l remote-url -d 'Remote URL (required for `remote`). Aliases: --endpoint, --remote_url' -r
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l workspace-id -d 'Workspace ID (for `remote` and `portals-cloud`)' -r
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l reset -d 'Reset provider configuration before (re)configuring'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l initial-commit -d 'Bootstrap existing unversioned repositories with an initial commit without prompting'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l no-initial-commit -d 'Skip bootstrapping existing repositories'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -a "status" -d 'Show current backend configuration and connectivity (default when no provider is given)'
complete -c px -n "__fish_px_using_subcommand configure; and not __fish_seen_subcommand_from status help" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from status" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from status" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from status" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show current backend configuration and connectivity (default when no provider is given)'
complete -c px -n "__fish_px_using_subcommand configure; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -f -a "backend" -d 'Choose backend provider'
complete -c px -n "__fish_px_using_subcommand choose; and not __fish_seen_subcommand_from backend help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -l remote-url -d 'Remote URL (required for remote provider)' -r
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -l workspace-id -d 'Workspace ID (for remote provider)' -r
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -l reset -d 'Reset the provider configuration file'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from backend" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from help" -f -a "backend" -d 'Choose backend provider'
complete -c px -n "__fish_px_using_subcommand choose; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -f -a "configure" -d 'Configure the version-control backend'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -f -a "status" -d 'Show the current version-control backend configuration'
complete -c px -n "__fish_px_using_subcommand backend; and not __fish_seen_subcommand_from configure status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -l endpoint -d 'Remote endpoint URL (required for remote backend)' -r
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -l workspace-id -d 'Workspace ID (for remote backend)' -r
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -l initial-commit -d 'Bootstrap existing repositories with an initial commit without prompting'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -l no-initial-commit -d 'Skip bootstrapping existing repositories with an initial commit'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from configure" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from status" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from status" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from status" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from help" -f -a "configure" -d 'Configure the version-control backend'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from help" -f -a "status" -d 'Show the current version-control backend configuration'
complete -c px -n "__fish_px_using_subcommand backend; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand completions" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand completions" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand completions" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand completions" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand doctor" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand doctor" -l repair -d 'Auto-repair detected issues'
complete -c px -n "__fish_px_using_subcommand doctor" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand doctor" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand doctor" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand doctor" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand status" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand status" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand status" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand status" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand status" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand sync" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand sync" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand sync" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand sync" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand sync" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand create" -s u -l repository -d 'Repository name' -r
complete -c px -n "__fish_px_using_subcommand create" -s n -l name -d 'Human-readable name' -r
complete -c px -n "__fish_px_using_subcommand create" -s a -l author -d 'Author identifier' -r
complete -c px -n "__fish_px_using_subcommand create" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand create" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand create" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand create" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand create" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand resolve" -l branch -d 'Resolve at a specific branch' -r
complete -c px -n "__fish_px_using_subcommand resolve" -l commit -d 'Resolve at a specific commit hash' -r
complete -c px -n "__fish_px_using_subcommand resolve" -s f -l format -d 'Output format: yaml, json' -r
complete -c px -n "__fish_px_using_subcommand resolve" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand resolve" -l provenance -d 'Include condensed per-file provenance for the manifest and direct representations'
complete -c px -n "__fish_px_using_subcommand resolve" -l include-blobs -d 'Hydrate known readable provenance artifacts such as prompts and run records'
complete -c px -n "__fish_px_using_subcommand resolve" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand resolve" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand resolve" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand resolve" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand presign" -l branch -d 'Resolve at a specific branch' -r
complete -c px -n "__fish_px_using_subcommand presign" -l commit -d 'Resolve at a specific commit hash' -r
complete -c px -n "__fish_px_using_subcommand presign" -l ttl-seconds -d 'Requested lifetime in seconds; Lore enforces its configured bounds' -r
complete -c px -n "__fish_px_using_subcommand presign" -l http-url -d 'Explicit Lore HTTP origin, such as http://127.0.0.1:41339' -r
complete -c px -n "__fish_px_using_subcommand presign" -l token-env -d 'Environment variable containing a repository-scoped bearer token' -r
complete -c px -n "__fish_px_using_subcommand presign" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand presign" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand presign" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand presign" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand presign" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand query" -s f -l format -d 'Output format: yaml, json' -r
complete -c px -n "__fish_px_using_subcommand query" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand query" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand query" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand query" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand query" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand commit" -s m -l message -d 'Commit message' -r
complete -c px -n "__fish_px_using_subcommand commit" -s a -l author -d 'Author identifier' -r
complete -c px -n "__fish_px_using_subcommand commit" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand commit" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand commit" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand commit" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand commit" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand history" -s n -l limit -d 'Maximum number of commits to show' -r
complete -c px -n "__fish_px_using_subcommand history" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand history" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand history" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand history" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand history" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand list" -s t -l entity-type -d 'Entity type to list (if repository is specified)' -r
complete -c px -n "__fish_px_using_subcommand list" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand list" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand list" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand list" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand list" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand branch" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand branch" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand branch" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand branch" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand branch" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand set" -s m -l message -d 'Commit message' -r
complete -c px -n "__fish_px_using_subcommand set" -s a -l author -d 'Author identifier' -r
complete -c px -n "__fish_px_using_subcommand set" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand set" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand set" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand set" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand set" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand add" -l format -d 'Asset format. e.g., "png", "glb"' -r
complete -c px -n "__fish_px_using_subcommand add" -s m -l message -d 'Commit message' -r
complete -c px -n "__fish_px_using_subcommand add" -s a -l author -d 'Author identifier' -r
complete -c px -n "__fish_px_using_subcommand add" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand add" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand add" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand add" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand add" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand revert" -s c -l commit -d 'Commit hash to revert' -r
complete -c px -n "__fish_px_using_subcommand revert" -s a -l author -d 'Author identifier' -r
complete -c px -n "__fish_px_using_subcommand revert" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand revert" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand revert" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand revert" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand revert" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand pull" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand pull" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand pull" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand pull" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand pull" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand push" -l remote-name -d 'Remote name (default: tracking branch\'s remote, or "origin")' -r
complete -c px -n "__fish_px_using_subcommand push" -l branch -d 'Branch to push (default: current branch)' -r
complete -c px -n "__fish_px_using_subcommand push" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand push" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand push" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand push" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand push" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -f -a "add" -d 'Add a remote to a repository repository'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -f -a "ls" -d 'List remotes on a repository repository'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -f -a "rm" -d 'Remove a remote from a repository repository'
complete -c px -n "__fish_px_using_subcommand remote; and not __fish_seen_subcommand_from add ls rm help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from add" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from add" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from add" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from ls" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from ls" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from ls" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from ls" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from ls" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from rm" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from rm" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from rm" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from rm" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from rm" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from help" -f -a "add" -d 'Add a remote to a repository repository'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from help" -f -a "ls" -d 'List remotes on a repository repository'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from help" -f -a "rm" -d 'Remove a remote from a repository repository'
complete -c px -n "__fish_px_using_subcommand remote; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand sign" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand sign" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand sign" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand sign" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand sign" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand verify" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand verify" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand verify" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand verify" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand verify" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand switch" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand switch" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand switch" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand switch" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand switch" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand head" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand head" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand head" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand head" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand head" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand validate" -l file -d 'Path to a manifest YAML file to validate' -r -F
complete -c px -n "__fish_px_using_subcommand validate" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand validate" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand validate" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand validate" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand validate" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand schema" -s f -l format -d 'Output format: json, yaml' -r
complete -c px -n "__fish_px_using_subcommand schema" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand schema" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand schema" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand schema" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand schema" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand diff" -s f -l format -d 'Output format: json, yaml' -r
complete -c px -n "__fish_px_using_subcommand diff" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand diff" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand diff" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand diff" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand diff" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand merge" -s f -l format -d 'Output format: json, yaml' -r
complete -c px -n "__fish_px_using_subcommand merge" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand merge" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand merge" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand merge" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand merge" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand content-hash" -s d -l base-dir -d 'Base directory for repository repositories. Defaults to $PX_DIR, or ~/.px if unset' -r -F
complete -c px -n "__fish_px_using_subcommand content-hash" -s v -l verbose -d 'Enable verbose debug logging'
complete -c px -n "__fish_px_using_subcommand content-hash" -l remote -d 'Resolve repository reads through the configured Lore server (the default)'
complete -c px -n "__fish_px_using_subcommand content-hash" -l local -d 'Resolve repository reads from an explicitly checked-out local working tree'
complete -c px -n "__fish_px_using_subcommand content-hash" -s h -l help -d 'Print help'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "auth" -d 'Manage secure authentication for the configured Lore provider'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "install" -d 'Install required dependencies'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "init" -d 'Initialize a repository repository and/or configure the backend provider'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "configure" -d 'Configure version-control backend (unified: replaces `px choose` + `px backend`)'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "choose" -d 'Choose backend provider (deprecated: use `px configure`)'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "backend" -d 'Configure or inspect the version-control backend (deprecated: use `px configure`)'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "completions" -d 'Generate shell completions for `px`'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "doctor" -d 'Run diagnostics and repair'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "status" -d 'Show system status'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "sync" -d 'Sync with remote'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "create" -d 'Create a new entity manifest'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "resolve" -d 'Resolve a PX URI to its manifest or a subtree'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "presign" -d 'Create a time-limited public URL for a committed representation'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "query" -d 'Query a subtree from a manifest'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "commit" -d 'Commit changes to a repository repository'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "history" -d 'View commit history for an entity'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "list" -d 'List repositories or entities within a repository'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "branch" -d 'Create or list branches'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "set" -d 'Set a property on an entity manifest'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "add" -d 'Add a file representation to an entity manifest'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "revert" -d 'Revert a commit by hash (undoes all changes in that commit)'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "pull" -d 'Clone or pull a repository from a remote'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "push" -d 'Push the current branch to its configured upstream remote'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "remote" -d 'Manage remotes on a repository'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "sign" -d 'Sign a manifest (stub for v0)'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "verify" -d 'Verify a manifest signature (stub for v0)'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "switch" -d 'Switch to a branch'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "head" -d 'Show the current HEAD commit hash'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "validate" -d 'Validate a manifest against the PX schema'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "schema" -d 'Print a JSON Schema for manifest or commit types'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "diff" -d 'Show diff between two manifest files or versions'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "merge" -d 'Three-way merge of JSON/YAML values'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "content-hash" -d 'Compute the BLAKE3 content hash of a file'
complete -c px -n "__fish_px_using_subcommand help; and not __fish_seen_subcommand_from auth install init configure choose backend completions doctor status sync create resolve presign query commit history list branch set add revert pull push remote sign verify switch head validate schema diff merge content-hash help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "login" -d 'Sign in through the configured Lore authentication service'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "status" -d 'Show the currently cached Lore identity without printing tokens'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from auth" -f -a "logout" -d 'Remove locally cached Lore credentials'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from configure" -f -a "status" -d 'Show current backend configuration and connectivity (default when no provider is given)'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from choose" -f -a "backend" -d 'Choose backend provider'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from backend" -f -a "configure" -d 'Configure the version-control backend'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from backend" -f -a "status" -d 'Show the current version-control backend configuration'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from remote" -f -a "add" -d 'Add a remote to a repository repository'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from remote" -f -a "ls" -d 'List remotes on a repository repository'
complete -c px -n "__fish_px_using_subcommand help; and __fish_seen_subcommand_from remote" -f -a "rm" -d 'Remove a remote from a repository repository'
