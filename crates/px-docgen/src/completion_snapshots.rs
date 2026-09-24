use anyhow::{Context, Result};
use clap::CommandFactory;
use clap_complete::Shell;
use px_cli::Cli;
use std::path::Path;

pub fn generate_all(workspace_root: &Path) -> Result<u32> {
    let completion_dir = workspace_root.join("completions");
    let mut files_written = 0;

    for (shell, filename) in [
        (Shell::Bash, "px.bash"),
        (Shell::Zsh, "px.zsh"),
        (Shell::Fish, "px.fish"),
    ] {
        let content = render(shell)?;
        if crate::filesystem::write_if_changed(&completion_dir.join(filename), &content)? {
            files_written += 1;
        }
    }

    Ok(files_written)
}

fn render(shell: Shell) -> Result<String> {
    let mut command = Cli::command();
    let mut output = Vec::new();
    clap_complete::generate(shell, &mut command, "px", &mut output);
    let content =
        String::from_utf8(output).context("completion generator returned non-UTF-8 output")?;
    Ok(match shell {
        Shell::Bash => bash_uri_completion(content),
        Shell::Zsh => zsh_uri_completion(content),
        Shell::Fish => fish_uri_completion(content),
        _ => content,
    })
}

const URI_COMMANDS: &str = "resolve query set unset add presign diff history";

fn bash_uri_completion(content: String) -> String {
    let uri_pattern = URI_COMMANDS.replace(' ', "|");
    let content = content.replacen("_px() {", "_px_generated() {", 1);
    let content = content.replace("complete -F _px ", "complete -F _px_generated ");
    format!(
        r#"{content}

_px_uri_candidates() {{
    local base="${{PX_DIR:-$HOME/.px}}" manifest path
    [[ -d "$base" ]] || return
    while IFS= read -r manifest; do
        path="${{manifest#$base/}}"
        [[ "$path" == */*/*.yaml ]] || continue
        path="${{path%.yaml}}"
        printf '%s\npx://%s\n' "$path" "$path"
    done < <(find "$base" -type f -name '*.yaml' ! -path '*/.px/*' 2>/dev/null)
}}

_px() {{
    local i command="" cur="${{COMP_WORDS[COMP_CWORD]}}"
    for ((i=1; i<COMP_CWORD; i++)); do
        case "${{COMP_WORDS[i]}}" in {uri_pattern}) command="${{COMP_WORDS[i]}}"; break;; esac
    done
    if [[ -n "$command" && "$cur" != -* ]]; then
        COMPREPLY=( $(compgen -W "$(_px_uri_candidates)" -- "$cur") )
        return 0
    fi
    _px_generated "$@"
}}
complete -F _px -o bashdefault -o default px
"#
    )
}

fn zsh_uri_completion(content: String) -> String {
    let uri_pattern = URI_COMMANDS.replace(' ', "|");
    let content = content.replacen("_px() {", "_px_generated() {", 1);
    let wrapper = format!(
        r#"
_px_uri_candidates() {{
    local base="${{PX_DIR:-$HOME/.px}}" manifest path
    [[ -d "$base" ]] || return
    while IFS= read -r manifest; do
        path="${{manifest#$base/}}"
        [[ "$path" == */*/*.yaml ]] || continue
        path="${{path%.yaml}}"
        print -r -- "$path"
        print -r -- "px://$path"
    done < <(find "$base" -type f -name '*.yaml' ! -path '*/.px/*' 2>/dev/null)
}}

_px() {{
    local word command=""
    for word in "${{words[@]}}"; do
        case "$word" in {uri_pattern}) command="$word"; break;; esac
    done
    if [[ -n "$command" && "$cur" != -* ]]; then
        local -a candidates
        candidates=("${{(@f)$(_px_uri_candidates)}}")
        _describe -t px-uri 'PX URI' candidates
        return
    fi
    _px_generated "$@"
}}
"#
    );
    content.replace(
        "\nif [ \"$funcstack[1]\" = \"_px\" ]; then",
        &format!("{wrapper}\nif [ \"$funcstack[1]\" = \"_px\" ]; then"),
    )
}

fn fish_uri_completion(content: String) -> String {
    format!(
        r#"{content}

function __fish_px_uri_candidates
    set -l base $PX_DIR
    test -n "$base"; or set base ~/.px
    test -d "$base"; or return
    for manifest in (find "$base" -type f -name '*.yaml' ! -path '*/.px/*' 2>/dev/null)
        set -l path (string replace -r "^$base/" '' -- $manifest)
        string match -qr '.+/.+/.+\.yaml$' -- $path; or continue
        set path (string replace -r '\\.yaml$' '' -- $path)
        echo $path
        echo px://$path
    end
end

complete -c px -n '__fish_px_using_subcommand resolve; or __fish_px_using_subcommand query; or __fish_px_using_subcommand set; or __fish_px_using_subcommand unset; or __fish_px_using_subcommand add; or __fish_px_using_subcommand presign; or __fish_px_using_subcommand diff; or __fish_px_using_subcommand history' -a '(__fish_px_uri_candidates)'
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_completion_scripts_for_supported_shells() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let content = render(shell).unwrap();
            assert!(content.contains("px"));
            assert!(content.contains("configure"));
            assert!(content.contains("uri_candidates"));
            assert!(content.contains("PX_DIR"));
            assert!(!content.contains("repository.yaml\n        path=\"${path%.yaml}\""));
        }
    }
}
