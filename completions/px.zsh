#compdef px

autoload -U is-at-least

_px() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_px_commands" \
"*::: :->px" \
&& ret=0
    case $state in
    (px)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-command-$line[1]:"
        case $line[1] in
            (auth)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_px__subcmd__auth_commands" \
"*::: :->auth" \
&& ret=0

    case $state in
    (auth)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-auth-command-$line[1]:"
        case $line[1] in
            (login)
_arguments "${_arguments_options[@]}" : \
'--api-key-env=[Environment variable containing the API key]:API_KEY_ENV:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--api-key[Exchange a service-account API key instead of opening a browser]' \
'(--api-key)--no-browser[Print the login URL without opening a browser]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(logout)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__auth__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-auth-help-command-$line[1]:"
        case $line[1] in
            (login)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(logout)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(install)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':target -- Target to install (e.g., "lore" or "mcp"):_default' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
'--provider=[Provider type\: local, portals-cloud, or remote]:PROVIDER:_default' \
'--remote-url=[Remote URL (required for remote provider)]:REMOTE_URL:_default' \
'--workspace-id=[Workspace ID (for remote provider)]:WORKSPACE_ID:_default' \
'--origin=[Remote URL to add as origin after init]:REMOTE:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--reset[Reset the provider configuration file]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::repository -- Repository name. If provided, initializes a new repository repository:_default' \
&& ret=0
;;
(configure)
_arguments "${_arguments_options[@]}" : \
'()--provider=[Provider type flag (alternative to positional \`PROVIDER\`)]:PROVIDER:_default' \
'--remote-url=[Remote URL (required for \`remote\`). Aliases\: --endpoint, --remote_url]:URL:_default' \
'--workspace-id=[Workspace ID (for \`remote\` and \`portals-cloud\`)]:ID:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--reset[Reset provider configuration before (re)configuring]' \
'--initial-commit[Bootstrap existing unversioned repositories with an initial commit without prompting]' \
'--no-initial-commit[Skip bootstrapping existing repositories]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'::provider -- Provider type\: local, portals-cloud, or remote. Positional for ergonomics; omit to show current config (or use `px configure status`):_default' \
":: :_px__subcmd__configure_commands" \
"*::: :->configure" \
&& ret=0

    case $state in
    (configure)
        words=($line[2] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-configure-command-$line[2]:"
        case $line[2] in
            (status)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__configure__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-configure-help-command-$line[1]:"
        case $line[1] in
            (status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(choose)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_px__subcmd__choose_commands" \
"*::: :->choose" \
&& ret=0

    case $state in
    (choose)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-choose-command-$line[1]:"
        case $line[1] in
            (backend)
_arguments "${_arguments_options[@]}" : \
'--remote-url=[Remote URL (required for remote provider)]:REMOTE_URL:_default' \
'--workspace-id=[Workspace ID (for remote provider)]:WORKSPACE_ID:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--reset[Reset the provider configuration file]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':provider -- Provider type\: local, portals-cloud, or remote:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__choose__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-choose-help-command-$line[1]:"
        case $line[1] in
            (backend)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(backend)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_px__subcmd__backend_commands" \
"*::: :->backend" \
&& ret=0

    case $state in
    (backend)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-backend-command-$line[1]:"
        case $line[1] in
            (configure)
_arguments "${_arguments_options[@]}" : \
'--endpoint=[Remote endpoint URL (required for remote backend)]:ENDPOINT:_default' \
'--workspace-id=[Workspace ID (for remote backend)]:WORKSPACE_ID:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--initial-commit[Bootstrap existing repositories with an initial commit without prompting]' \
'--no-initial-commit[Skip bootstrapping existing repositories with an initial commit]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':backend -- Backend type\: local or remote:_default' \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__backend__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-backend-help-command-$line[1]:"
        case $line[1] in
            (configure)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':shell -- Shell to generate completions for:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(doctor)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--repair[Auto-repair detected issues]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(sync)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
&& ret=0
;;
(create)
_arguments "${_arguments_options[@]}" : \
'-u+[Repository name]:REPOSITORY:_default' \
'--repository=[Repository name]:REPOSITORY:_default' \
'-n+[Human-readable name]:NAME:_default' \
'--name=[Human-readable name]:NAME:_default' \
'-a+[Author identifier]:AUTHOR:_default' \
'--author=[Author identifier]:AUTHOR:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':entity_type -- Entity type (any non-empty string, e.g. character, location, custom-type):_default' \
':entity_id -- Entity ID (slug). e.g., "woody":_default' \
&& ret=0
;;
(resolve)
_arguments "${_arguments_options[@]}" : \
'--branch=[Resolve at a specific branch]:BRANCH:_default' \
'--commit=[Resolve at a specific commit hash]:COMMIT:_default' \
'-f+[Output format\: yaml, json]:FORMAT:_default' \
'--format=[Output format\: yaml, json]:FORMAT:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--provenance[Include condensed per-file provenance for the manifest and direct representations]' \
'--include-blobs[Hydrate known readable provenance artifacts such as prompts and run records]' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':uri -- PX URI. e.g., "px\://toystory/character/woody":_default' \
&& ret=0
;;
(presign)
_arguments "${_arguments_options[@]}" : \
'(--commit)--branch=[Resolve at a specific branch]:BRANCH:_default' \
'(--branch)--commit=[Resolve at a specific commit hash]:COMMIT:_default' \
'--ttl-seconds=[Requested lifetime in seconds; Lore enforces its configured bounds]:TTL_SECONDS:_default' \
'--http-url=[Explicit Lore HTTP origin, such as http\://127.0.0.1\:41339]:HTTP_URL:_default' \
'--token-env=[Environment variable containing a repository-scoped bearer token]:TOKEN_ENV:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':uri -- Entity ID, e.g. 25th-chapter/character/nathan-gunn. The px\:// prefix is optional; fragments are not supported:_default' \
':representation -- Representation name (manifest key), e.g. item. Its URI is relative to the entity'\''s asset directory:_default' \
&& ret=0
;;
(query)
_arguments "${_arguments_options[@]}" : \
'-f+[Output format\: yaml, json]:FORMAT:_default' \
'--format=[Output format\: yaml, json]:FORMAT:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':uri -- PX URI:_default' \
':path -- Dot-notation path. e.g., "appearances.audienceVotes":_default' \
&& ret=0
;;
(commit)
_arguments "${_arguments_options[@]}" : \
'-m+[Commit message]:MESSAGE:_default' \
'--message=[Commit message]:MESSAGE:_default' \
'-a+[Author identifier]:AUTHOR:_default' \
'--author=[Author identifier]:AUTHOR:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
&& ret=0
;;
(history)
_arguments "${_arguments_options[@]}" : \
'-n+[Maximum number of commits to show]:LIMIT:_default' \
'--limit=[Maximum number of commits to show]:LIMIT:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':uri -- PX URI:_default' \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
'-t+[Entity type to list (if repository is specified)]:ENTITY_TYPE:_default' \
'--entity-type=[Entity type to list (if repository is specified)]:ENTITY_TYPE:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
'::repository -- Repository name. Omit to list all repositories:_default' \
&& ret=0
;;
(branch)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
'::name -- Branch name to create. Omit to list all branches:_default' \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
'-m+[Commit message]:MESSAGE:_default' \
'--message=[Commit message]:MESSAGE:_default' \
'-a+[Author identifier]:AUTHOR:_default' \
'--author=[Author identifier]:AUTHOR:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':uri -- PX URI:_default' \
':key -- Property key (dot-notation):_default' \
':value -- Property value:_default' \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
'--format=[Asset format. e.g., "png", "glb"]:FORMAT:_default' \
'-m+[Commit message]:MESSAGE:_default' \
'--message=[Commit message]:MESSAGE:_default' \
'-a+[Author identifier]:AUTHOR:_default' \
'--author=[Author identifier]:AUTHOR:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':uri -- PX URI:_default' \
':key -- Representation key. e.g., "reference_image":_default' \
':file -- File path to the asset:_files' \
&& ret=0
;;
(revert)
_arguments "${_arguments_options[@]}" : \
'-c+[Commit hash to revert]:COMMIT:_default' \
'--commit=[Commit hash to revert]:COMMIT:_default' \
'-a+[Author identifier]:AUTHOR:_default' \
'--author=[Author identifier]:AUTHOR:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
&& ret=0
;;
(pull)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':url_or_name -- URL (clone) or repository name (pull existing):_default' \
&& ret=0
;;
(push)
_arguments "${_arguments_options[@]}" : \
'--remote-name=[Remote name (default\: tracking branch'\''s remote, or "origin")]:REMOTE:_default' \
'--branch=[Branch to push (default\: current branch)]:BRANCH:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
':repository -- Repository name:_default' \
&& ret=0
;;
(remote)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
":: :_px__subcmd__remote_commands" \
"*::: :->remote" \
&& ret=0

    case $state in
    (remote)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-remote-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
':name -- Remote name (e.g., "origin"):_default' \
':url -- Remote URL:_default' \
&& ret=0
;;
(ls)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
&& ret=0
;;
(rm)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
':name -- Remote name to remove:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__remote__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-remote-help-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(ls)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rm)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(sign)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':uri -- PX URI:_default' \
&& ret=0
;;
(verify)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':uri -- PX URI:_default' \
&& ret=0
;;
(switch)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
':name -- Branch name to switch to:_default' \
&& ret=0
;;
(head)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':repository -- Repository name:_default' \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
'--file=[Path to a manifest YAML file to validate]:FILE:_files' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
'::uri -- PX URI of the entity to validate:_default' \
&& ret=0
;;
(schema)
_arguments "${_arguments_options[@]}" : \
'-f+[Output format\: json, yaml]:FORMAT:_default' \
'--format=[Output format\: json, yaml]:FORMAT:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':name -- Schema name\: '\''manifest'\'' or '\''commit'\'':_default' \
&& ret=0
;;
(diff)
_arguments "${_arguments_options[@]}" : \
'-f+[Output format\: json, yaml]:FORMAT:_default' \
'--format=[Output format\: json, yaml]:FORMAT:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':base_file -- Base (left) manifest file:_files' \
':candidate_file -- Candidate (right) manifest file:_files' \
&& ret=0
;;
(merge)
_arguments "${_arguments_options[@]}" : \
'-f+[Output format\: json, yaml]:FORMAT:_default' \
'--format=[Output format\: json, yaml]:FORMAT:_default' \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':base -- Base (common ancestor) file:_files' \
':current -- Current (ours) file:_files' \
':proposed -- Proposed (theirs) file:_files' \
&& ret=0
;;
(content-hash)
_arguments "${_arguments_options[@]}" : \
'-d+[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'--base-dir=[Base directory for repository repositories. Defaults to \$PX_DIR, or ~/.px if unset]:BASE_DIR:_files' \
'-v[Enable verbose debug logging]' \
'--verbose[Enable verbose debug logging]' \
'(--local)--remote[Resolve repository reads through the configured Lore server (the default)]' \
'(--remote)--local[Resolve repository reads from an explicitly checked-out local working tree]' \
'-h[Print help]' \
'--help[Print help]' \
':file -- Path to the file to hash:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-help-command-$line[1]:"
        case $line[1] in
            (auth)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__help__subcmd__auth_commands" \
"*::: :->auth" \
&& ret=0

    case $state in
    (auth)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-help-auth-command-$line[1]:"
        case $line[1] in
            (login)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(logout)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(install)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(configure)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__help__subcmd__configure_commands" \
"*::: :->configure" \
&& ret=0

    case $state in
    (configure)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-help-configure-command-$line[1]:"
        case $line[1] in
            (status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(choose)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__help__subcmd__choose_commands" \
"*::: :->choose" \
&& ret=0

    case $state in
    (choose)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-help-choose-command-$line[1]:"
        case $line[1] in
            (backend)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(backend)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__help__subcmd__backend_commands" \
"*::: :->backend" \
&& ret=0

    case $state in
    (backend)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-help-backend-command-$line[1]:"
        case $line[1] in
            (configure)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(doctor)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(status)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(sync)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(create)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(resolve)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(presign)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(query)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(commit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(history)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(branch)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(set)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(revert)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(pull)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(push)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(remote)
_arguments "${_arguments_options[@]}" : \
":: :_px__subcmd__help__subcmd__remote_commands" \
"*::: :->remote" \
&& ret=0

    case $state in
    (remote)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:px-help-remote-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(ls)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rm)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(sign)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(switch)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(head)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(schema)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(diff)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(merge)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(content-hash)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_px_commands] )) ||
_px_commands() {
    local commands; commands=(
'auth:Manage secure authentication for the configured Lore provider' \
'install:Install required dependencies' \
'init:Initialize a repository repository and/or configure the backend provider' \
'configure:Configure version-control backend (unified\: replaces \`px choose\` + \`px backend\`)' \
'choose:Choose backend provider (deprecated\: use \`px configure\`)' \
'backend:Configure or inspect the version-control backend (deprecated\: use \`px configure\`)' \
'completions:Generate shell completions for \`px\`' \
'doctor:Run diagnostics and repair' \
'status:Show system status' \
'sync:Sync with remote' \
'create:Create a new entity manifest' \
'resolve:Resolve a PX URI to its manifest or a subtree' \
'presign:Create a time-limited public URL for a committed representation' \
'query:Query a subtree from a manifest' \
'commit:Commit changes to a repository repository' \
'history:View commit history for an entity' \
'list:List repositories or entities within a repository' \
'branch:Create or list branches' \
'set:Set a property on an entity manifest' \
'add:Add a file representation to an entity manifest' \
'revert:Revert a commit by hash (undoes all changes in that commit)' \
'pull:Clone or pull a repository from a remote' \
'push:Push the current branch to its configured upstream remote' \
'remote:Manage remotes on a repository' \
'sign:Sign a manifest (stub for v0)' \
'verify:Verify a manifest signature (stub for v0)' \
'switch:Switch to a branch' \
'head:Show the current HEAD commit hash' \
'validate:Validate a manifest against the PX schema' \
'schema:Print a JSON Schema for manifest or commit types' \
'diff:Show diff between two manifest files or versions' \
'merge:Three-way merge of JSON/YAML values' \
'content-hash:Compute the BLAKE3 content hash of a file' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px commands' commands "$@"
}
(( $+functions[_px__subcmd__add_commands] )) ||
_px__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'px add commands' commands "$@"
}
(( $+functions[_px__subcmd__auth_commands] )) ||
_px__subcmd__auth_commands() {
    local commands; commands=(
'login:Sign in through the configured Lore authentication service' \
'status:Show the currently cached Lore identity without printing tokens' \
'logout:Remove locally cached Lore credentials' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px auth commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__help_commands] )) ||
_px__subcmd__auth__subcmd__help_commands() {
    local commands; commands=(
'login:Sign in through the configured Lore authentication service' \
'status:Show the currently cached Lore identity without printing tokens' \
'logout:Remove locally cached Lore credentials' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px auth help commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__help__subcmd__help_commands] )) ||
_px__subcmd__auth__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'px auth help help commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__help__subcmd__login_commands] )) ||
_px__subcmd__auth__subcmd__help__subcmd__login_commands() {
    local commands; commands=()
    _describe -t commands 'px auth help login commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__help__subcmd__logout_commands] )) ||
_px__subcmd__auth__subcmd__help__subcmd__logout_commands() {
    local commands; commands=()
    _describe -t commands 'px auth help logout commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__help__subcmd__status_commands] )) ||
_px__subcmd__auth__subcmd__help__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px auth help status commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__login_commands] )) ||
_px__subcmd__auth__subcmd__login_commands() {
    local commands; commands=()
    _describe -t commands 'px auth login commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__logout_commands] )) ||
_px__subcmd__auth__subcmd__logout_commands() {
    local commands; commands=()
    _describe -t commands 'px auth logout commands' commands "$@"
}
(( $+functions[_px__subcmd__auth__subcmd__status_commands] )) ||
_px__subcmd__auth__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px auth status commands' commands "$@"
}
(( $+functions[_px__subcmd__backend_commands] )) ||
_px__subcmd__backend_commands() {
    local commands; commands=(
'configure:Configure the version-control backend' \
'status:Show the current version-control backend configuration' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px backend commands' commands "$@"
}
(( $+functions[_px__subcmd__backend__subcmd__configure_commands] )) ||
_px__subcmd__backend__subcmd__configure_commands() {
    local commands; commands=()
    _describe -t commands 'px backend configure commands' commands "$@"
}
(( $+functions[_px__subcmd__backend__subcmd__help_commands] )) ||
_px__subcmd__backend__subcmd__help_commands() {
    local commands; commands=(
'configure:Configure the version-control backend' \
'status:Show the current version-control backend configuration' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px backend help commands' commands "$@"
}
(( $+functions[_px__subcmd__backend__subcmd__help__subcmd__configure_commands] )) ||
_px__subcmd__backend__subcmd__help__subcmd__configure_commands() {
    local commands; commands=()
    _describe -t commands 'px backend help configure commands' commands "$@"
}
(( $+functions[_px__subcmd__backend__subcmd__help__subcmd__help_commands] )) ||
_px__subcmd__backend__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'px backend help help commands' commands "$@"
}
(( $+functions[_px__subcmd__backend__subcmd__help__subcmd__status_commands] )) ||
_px__subcmd__backend__subcmd__help__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px backend help status commands' commands "$@"
}
(( $+functions[_px__subcmd__backend__subcmd__status_commands] )) ||
_px__subcmd__backend__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px backend status commands' commands "$@"
}
(( $+functions[_px__subcmd__branch_commands] )) ||
_px__subcmd__branch_commands() {
    local commands; commands=()
    _describe -t commands 'px branch commands' commands "$@"
}
(( $+functions[_px__subcmd__choose_commands] )) ||
_px__subcmd__choose_commands() {
    local commands; commands=(
'backend:Choose backend provider' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px choose commands' commands "$@"
}
(( $+functions[_px__subcmd__choose__subcmd__backend_commands] )) ||
_px__subcmd__choose__subcmd__backend_commands() {
    local commands; commands=()
    _describe -t commands 'px choose backend commands' commands "$@"
}
(( $+functions[_px__subcmd__choose__subcmd__help_commands] )) ||
_px__subcmd__choose__subcmd__help_commands() {
    local commands; commands=(
'backend:Choose backend provider' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px choose help commands' commands "$@"
}
(( $+functions[_px__subcmd__choose__subcmd__help__subcmd__backend_commands] )) ||
_px__subcmd__choose__subcmd__help__subcmd__backend_commands() {
    local commands; commands=()
    _describe -t commands 'px choose help backend commands' commands "$@"
}
(( $+functions[_px__subcmd__choose__subcmd__help__subcmd__help_commands] )) ||
_px__subcmd__choose__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'px choose help help commands' commands "$@"
}
(( $+functions[_px__subcmd__commit_commands] )) ||
_px__subcmd__commit_commands() {
    local commands; commands=()
    _describe -t commands 'px commit commands' commands "$@"
}
(( $+functions[_px__subcmd__completions_commands] )) ||
_px__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'px completions commands' commands "$@"
}
(( $+functions[_px__subcmd__configure_commands] )) ||
_px__subcmd__configure_commands() {
    local commands; commands=(
'status:Show current backend configuration and connectivity (default when no provider is given)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px configure commands' commands "$@"
}
(( $+functions[_px__subcmd__configure__subcmd__help_commands] )) ||
_px__subcmd__configure__subcmd__help_commands() {
    local commands; commands=(
'status:Show current backend configuration and connectivity (default when no provider is given)' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px configure help commands' commands "$@"
}
(( $+functions[_px__subcmd__configure__subcmd__help__subcmd__help_commands] )) ||
_px__subcmd__configure__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'px configure help help commands' commands "$@"
}
(( $+functions[_px__subcmd__configure__subcmd__help__subcmd__status_commands] )) ||
_px__subcmd__configure__subcmd__help__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px configure help status commands' commands "$@"
}
(( $+functions[_px__subcmd__configure__subcmd__status_commands] )) ||
_px__subcmd__configure__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px configure status commands' commands "$@"
}
(( $+functions[_px__subcmd__content-hash_commands] )) ||
_px__subcmd__content-hash_commands() {
    local commands; commands=()
    _describe -t commands 'px content-hash commands' commands "$@"
}
(( $+functions[_px__subcmd__create_commands] )) ||
_px__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'px create commands' commands "$@"
}
(( $+functions[_px__subcmd__diff_commands] )) ||
_px__subcmd__diff_commands() {
    local commands; commands=()
    _describe -t commands 'px diff commands' commands "$@"
}
(( $+functions[_px__subcmd__doctor_commands] )) ||
_px__subcmd__doctor_commands() {
    local commands; commands=()
    _describe -t commands 'px doctor commands' commands "$@"
}
(( $+functions[_px__subcmd__head_commands] )) ||
_px__subcmd__head_commands() {
    local commands; commands=()
    _describe -t commands 'px head commands' commands "$@"
}
(( $+functions[_px__subcmd__help_commands] )) ||
_px__subcmd__help_commands() {
    local commands; commands=(
'auth:Manage secure authentication for the configured Lore provider' \
'install:Install required dependencies' \
'init:Initialize a repository repository and/or configure the backend provider' \
'configure:Configure version-control backend (unified\: replaces \`px choose\` + \`px backend\`)' \
'choose:Choose backend provider (deprecated\: use \`px configure\`)' \
'backend:Configure or inspect the version-control backend (deprecated\: use \`px configure\`)' \
'completions:Generate shell completions for \`px\`' \
'doctor:Run diagnostics and repair' \
'status:Show system status' \
'sync:Sync with remote' \
'create:Create a new entity manifest' \
'resolve:Resolve a PX URI to its manifest or a subtree' \
'presign:Create a time-limited public URL for a committed representation' \
'query:Query a subtree from a manifest' \
'commit:Commit changes to a repository repository' \
'history:View commit history for an entity' \
'list:List repositories or entities within a repository' \
'branch:Create or list branches' \
'set:Set a property on an entity manifest' \
'add:Add a file representation to an entity manifest' \
'revert:Revert a commit by hash (undoes all changes in that commit)' \
'pull:Clone or pull a repository from a remote' \
'push:Push the current branch to its configured upstream remote' \
'remote:Manage remotes on a repository' \
'sign:Sign a manifest (stub for v0)' \
'verify:Verify a manifest signature (stub for v0)' \
'switch:Switch to a branch' \
'head:Show the current HEAD commit hash' \
'validate:Validate a manifest against the PX schema' \
'schema:Print a JSON Schema for manifest or commit types' \
'diff:Show diff between two manifest files or versions' \
'merge:Three-way merge of JSON/YAML values' \
'content-hash:Compute the BLAKE3 content hash of a file' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px help commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__add_commands] )) ||
_px__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'px help add commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__auth_commands] )) ||
_px__subcmd__help__subcmd__auth_commands() {
    local commands; commands=(
'login:Sign in through the configured Lore authentication service' \
'status:Show the currently cached Lore identity without printing tokens' \
'logout:Remove locally cached Lore credentials' \
    )
    _describe -t commands 'px help auth commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__auth__subcmd__login_commands] )) ||
_px__subcmd__help__subcmd__auth__subcmd__login_commands() {
    local commands; commands=()
    _describe -t commands 'px help auth login commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__auth__subcmd__logout_commands] )) ||
_px__subcmd__help__subcmd__auth__subcmd__logout_commands() {
    local commands; commands=()
    _describe -t commands 'px help auth logout commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__auth__subcmd__status_commands] )) ||
_px__subcmd__help__subcmd__auth__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px help auth status commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__backend_commands] )) ||
_px__subcmd__help__subcmd__backend_commands() {
    local commands; commands=(
'configure:Configure the version-control backend' \
'status:Show the current version-control backend configuration' \
    )
    _describe -t commands 'px help backend commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__backend__subcmd__configure_commands] )) ||
_px__subcmd__help__subcmd__backend__subcmd__configure_commands() {
    local commands; commands=()
    _describe -t commands 'px help backend configure commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__backend__subcmd__status_commands] )) ||
_px__subcmd__help__subcmd__backend__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px help backend status commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__branch_commands] )) ||
_px__subcmd__help__subcmd__branch_commands() {
    local commands; commands=()
    _describe -t commands 'px help branch commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__choose_commands] )) ||
_px__subcmd__help__subcmd__choose_commands() {
    local commands; commands=(
'backend:Choose backend provider' \
    )
    _describe -t commands 'px help choose commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__choose__subcmd__backend_commands] )) ||
_px__subcmd__help__subcmd__choose__subcmd__backend_commands() {
    local commands; commands=()
    _describe -t commands 'px help choose backend commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__commit_commands] )) ||
_px__subcmd__help__subcmd__commit_commands() {
    local commands; commands=()
    _describe -t commands 'px help commit commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__completions_commands] )) ||
_px__subcmd__help__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'px help completions commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__configure_commands] )) ||
_px__subcmd__help__subcmd__configure_commands() {
    local commands; commands=(
'status:Show current backend configuration and connectivity (default when no provider is given)' \
    )
    _describe -t commands 'px help configure commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__configure__subcmd__status_commands] )) ||
_px__subcmd__help__subcmd__configure__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px help configure status commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__content-hash_commands] )) ||
_px__subcmd__help__subcmd__content-hash_commands() {
    local commands; commands=()
    _describe -t commands 'px help content-hash commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__create_commands] )) ||
_px__subcmd__help__subcmd__create_commands() {
    local commands; commands=()
    _describe -t commands 'px help create commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__diff_commands] )) ||
_px__subcmd__help__subcmd__diff_commands() {
    local commands; commands=()
    _describe -t commands 'px help diff commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__doctor_commands] )) ||
_px__subcmd__help__subcmd__doctor_commands() {
    local commands; commands=()
    _describe -t commands 'px help doctor commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__head_commands] )) ||
_px__subcmd__help__subcmd__head_commands() {
    local commands; commands=()
    _describe -t commands 'px help head commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__help_commands] )) ||
_px__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'px help help commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__history_commands] )) ||
_px__subcmd__help__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'px help history commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__init_commands] )) ||
_px__subcmd__help__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'px help init commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__install_commands] )) ||
_px__subcmd__help__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'px help install commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__list_commands] )) ||
_px__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'px help list commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__merge_commands] )) ||
_px__subcmd__help__subcmd__merge_commands() {
    local commands; commands=()
    _describe -t commands 'px help merge commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__presign_commands] )) ||
_px__subcmd__help__subcmd__presign_commands() {
    local commands; commands=()
    _describe -t commands 'px help presign commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__pull_commands] )) ||
_px__subcmd__help__subcmd__pull_commands() {
    local commands; commands=()
    _describe -t commands 'px help pull commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__push_commands] )) ||
_px__subcmd__help__subcmd__push_commands() {
    local commands; commands=()
    _describe -t commands 'px help push commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__query_commands] )) ||
_px__subcmd__help__subcmd__query_commands() {
    local commands; commands=()
    _describe -t commands 'px help query commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__remote_commands] )) ||
_px__subcmd__help__subcmd__remote_commands() {
    local commands; commands=(
'add:Add a remote to a repository repository' \
'ls:List remotes on a repository repository' \
'rm:Remove a remote from a repository repository' \
    )
    _describe -t commands 'px help remote commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__remote__subcmd__add_commands] )) ||
_px__subcmd__help__subcmd__remote__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'px help remote add commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__remote__subcmd__ls_commands] )) ||
_px__subcmd__help__subcmd__remote__subcmd__ls_commands() {
    local commands; commands=()
    _describe -t commands 'px help remote ls commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__remote__subcmd__rm_commands] )) ||
_px__subcmd__help__subcmd__remote__subcmd__rm_commands() {
    local commands; commands=()
    _describe -t commands 'px help remote rm commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__resolve_commands] )) ||
_px__subcmd__help__subcmd__resolve_commands() {
    local commands; commands=()
    _describe -t commands 'px help resolve commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__revert_commands] )) ||
_px__subcmd__help__subcmd__revert_commands() {
    local commands; commands=()
    _describe -t commands 'px help revert commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__schema_commands] )) ||
_px__subcmd__help__subcmd__schema_commands() {
    local commands; commands=()
    _describe -t commands 'px help schema commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__set_commands] )) ||
_px__subcmd__help__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'px help set commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__sign_commands] )) ||
_px__subcmd__help__subcmd__sign_commands() {
    local commands; commands=()
    _describe -t commands 'px help sign commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__status_commands] )) ||
_px__subcmd__help__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px help status commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__switch_commands] )) ||
_px__subcmd__help__subcmd__switch_commands() {
    local commands; commands=()
    _describe -t commands 'px help switch commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__sync_commands] )) ||
_px__subcmd__help__subcmd__sync_commands() {
    local commands; commands=()
    _describe -t commands 'px help sync commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__validate_commands] )) ||
_px__subcmd__help__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'px help validate commands' commands "$@"
}
(( $+functions[_px__subcmd__help__subcmd__verify_commands] )) ||
_px__subcmd__help__subcmd__verify_commands() {
    local commands; commands=()
    _describe -t commands 'px help verify commands' commands "$@"
}
(( $+functions[_px__subcmd__history_commands] )) ||
_px__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'px history commands' commands "$@"
}
(( $+functions[_px__subcmd__init_commands] )) ||
_px__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'px init commands' commands "$@"
}
(( $+functions[_px__subcmd__install_commands] )) ||
_px__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'px install commands' commands "$@"
}
(( $+functions[_px__subcmd__list_commands] )) ||
_px__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'px list commands' commands "$@"
}
(( $+functions[_px__subcmd__merge_commands] )) ||
_px__subcmd__merge_commands() {
    local commands; commands=()
    _describe -t commands 'px merge commands' commands "$@"
}
(( $+functions[_px__subcmd__presign_commands] )) ||
_px__subcmd__presign_commands() {
    local commands; commands=()
    _describe -t commands 'px presign commands' commands "$@"
}
(( $+functions[_px__subcmd__pull_commands] )) ||
_px__subcmd__pull_commands() {
    local commands; commands=()
    _describe -t commands 'px pull commands' commands "$@"
}
(( $+functions[_px__subcmd__push_commands] )) ||
_px__subcmd__push_commands() {
    local commands; commands=()
    _describe -t commands 'px push commands' commands "$@"
}
(( $+functions[_px__subcmd__query_commands] )) ||
_px__subcmd__query_commands() {
    local commands; commands=()
    _describe -t commands 'px query commands' commands "$@"
}
(( $+functions[_px__subcmd__remote_commands] )) ||
_px__subcmd__remote_commands() {
    local commands; commands=(
'add:Add a remote to a repository repository' \
'ls:List remotes on a repository repository' \
'rm:Remove a remote from a repository repository' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px remote commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__add_commands] )) ||
_px__subcmd__remote__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'px remote add commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__help_commands] )) ||
_px__subcmd__remote__subcmd__help_commands() {
    local commands; commands=(
'add:Add a remote to a repository repository' \
'ls:List remotes on a repository repository' \
'rm:Remove a remote from a repository repository' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'px remote help commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__help__subcmd__add_commands] )) ||
_px__subcmd__remote__subcmd__help__subcmd__add_commands() {
    local commands; commands=()
    _describe -t commands 'px remote help add commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__help__subcmd__help_commands] )) ||
_px__subcmd__remote__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'px remote help help commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__help__subcmd__ls_commands] )) ||
_px__subcmd__remote__subcmd__help__subcmd__ls_commands() {
    local commands; commands=()
    _describe -t commands 'px remote help ls commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__help__subcmd__rm_commands] )) ||
_px__subcmd__remote__subcmd__help__subcmd__rm_commands() {
    local commands; commands=()
    _describe -t commands 'px remote help rm commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__ls_commands] )) ||
_px__subcmd__remote__subcmd__ls_commands() {
    local commands; commands=()
    _describe -t commands 'px remote ls commands' commands "$@"
}
(( $+functions[_px__subcmd__remote__subcmd__rm_commands] )) ||
_px__subcmd__remote__subcmd__rm_commands() {
    local commands; commands=()
    _describe -t commands 'px remote rm commands' commands "$@"
}
(( $+functions[_px__subcmd__resolve_commands] )) ||
_px__subcmd__resolve_commands() {
    local commands; commands=()
    _describe -t commands 'px resolve commands' commands "$@"
}
(( $+functions[_px__subcmd__revert_commands] )) ||
_px__subcmd__revert_commands() {
    local commands; commands=()
    _describe -t commands 'px revert commands' commands "$@"
}
(( $+functions[_px__subcmd__schema_commands] )) ||
_px__subcmd__schema_commands() {
    local commands; commands=()
    _describe -t commands 'px schema commands' commands "$@"
}
(( $+functions[_px__subcmd__set_commands] )) ||
_px__subcmd__set_commands() {
    local commands; commands=()
    _describe -t commands 'px set commands' commands "$@"
}
(( $+functions[_px__subcmd__sign_commands] )) ||
_px__subcmd__sign_commands() {
    local commands; commands=()
    _describe -t commands 'px sign commands' commands "$@"
}
(( $+functions[_px__subcmd__status_commands] )) ||
_px__subcmd__status_commands() {
    local commands; commands=()
    _describe -t commands 'px status commands' commands "$@"
}
(( $+functions[_px__subcmd__switch_commands] )) ||
_px__subcmd__switch_commands() {
    local commands; commands=()
    _describe -t commands 'px switch commands' commands "$@"
}
(( $+functions[_px__subcmd__sync_commands] )) ||
_px__subcmd__sync_commands() {
    local commands; commands=()
    _describe -t commands 'px sync commands' commands "$@"
}
(( $+functions[_px__subcmd__validate_commands] )) ||
_px__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'px validate commands' commands "$@"
}
(( $+functions[_px__subcmd__verify_commands] )) ||
_px__subcmd__verify_commands() {
    local commands; commands=()
    _describe -t commands 'px verify commands' commands "$@"
}

if [ "$funcstack[1]" = "_px" ]; then
    _px "$@"
else
    compdef _px px
fi
