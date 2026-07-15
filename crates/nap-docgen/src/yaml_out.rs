use crate::model::{CommandModel, DocMeta, WorkspaceMeta};

fn yaml_escape(s: &str) -> String {
    if s.contains('"') || s.contains('\\') || s.contains('\n') || s.is_empty() {
        format!(
            "\"{}\"",
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        )
    } else {
        format!("\"{}\"", s)
    }
}

pub fn render_commands_yaml(
    commands: &[CommandModel],
    _cargo_meta: &WorkspaceMeta,
    meta: &DocMeta,
) -> String {
    let mut yaml = String::new();
    yaml.push_str(&format!("generator: {}\n", meta.generator_name));
    yaml.push_str(&format!("version: \"{}\"\n", meta.crate_version));
    if let Some(ref sha) = meta.git_sha {
        yaml.push_str(&format!("git_sha: \"{sha}\"\n"));
    }
    yaml.push_str("commands:\n");

    for cmd in commands {
        render_command_yaml(cmd, &mut yaml, 1);
    }

    yaml
}

fn render_command_yaml(cmd: &CommandModel, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent * 2);
    out.push_str(&format!("{pad}- name: {}\n", yaml_escape(&cmd.name)));
    out.push_str(&format!(
        "{pad}  full_path: {}\n",
        yaml_escape(&cmd.full_path)
    ));
    out.push_str(&format!("{pad}  about: {}\n", yaml_escape(&cmd.about)));
    if let Some(ref long) = cmd.long_about {
        out.push_str(&format!("{pad}  long_about: |\n"));
        for line in long.lines() {
            out.push_str(&format!("{pad}    {line}\n"));
        }
    }
    out.push_str(&format!("{pad}  usage: {}\n", yaml_escape(&cmd.usage)));
    out.push_str(&format!(
        "{pad}  hidden: {}\n",
        if cmd.hidden { "true" } else { "false" }
    ));

    if !cmd.arguments.is_empty() {
        out.push_str(&format!("{pad}  arguments:\n"));
        for arg in &cmd.arguments {
            out.push_str(&format!("{pad}    - name: {}\n", yaml_escape(&arg.name)));
            out.push_str(&format!("{pad}      about: {}\n", yaml_escape(&arg.about)));
            out.push_str(&format!(
                "{pad}      required: {}\n",
                if arg.required { "true" } else { "false" }
            ));
            if let Some(ref def) = arg.default_value {
                out.push_str(&format!("{pad}      default: {}\n", yaml_escape(def)));
            }
        }
    }

    if !cmd.options.is_empty() {
        out.push_str(&format!("{pad}  options:\n"));
        for opt in &cmd.options {
            out.push_str(&format!("{pad}    - name: {}\n", yaml_escape(&opt.name)));
            out.push_str(&format!("{pad}      about: {}\n", yaml_escape(&opt.about)));
            if let Some(ref def) = opt.default_value {
                out.push_str(&format!("{pad}      default: {}\n", yaml_escape(def)));
            }
        }
    }

    if !cmd.flags.is_empty() {
        out.push_str(&format!("{pad}  flags:\n"));
        for flag in &cmd.flags {
            out.push_str(&format!("{pad}    - name: {}\n", yaml_escape(&flag.name)));
            out.push_str(&format!("{pad}      about: {}\n", yaml_escape(&flag.about)));
        }
    }

    if !cmd.subcommands.is_empty() {
        out.push_str(&format!("{pad}  subcommands:\n"));
        for sub in &cmd.subcommands {
            render_command_yaml(sub, out, indent + 2);
        }
    }
}
