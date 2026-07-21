use crate::model::{CommandModel, DocMeta, WorkspaceMeta};
use serde::Serialize;

#[derive(Serialize)]
struct CommandsJson {
    generator: String,
    version: String,
    commands: Vec<CommandJson>,
}

#[derive(Serialize)]
struct CommandJson {
    name: String,
    full_path: String,
    about: String,
    long_about: Option<String>,
    usage: String,
    aliases: Vec<String>,
    arguments: Vec<ArgJson>,
    options: Vec<ArgJson>,
    flags: Vec<ArgJson>,
    subcommands: Vec<CommandJson>,
    hidden: bool,
}

#[derive(Serialize)]
struct ArgJson {
    name: String,
    about: String,
    required: bool,
    default_value: Option<String>,
    short: Option<char>,
    long: Option<String>,
    takes_value: bool,
}

pub fn render_commands_json(
    commands: &[CommandModel],
    _cargo_meta: &WorkspaceMeta,
    meta: &DocMeta,
) -> String {
    let json = CommandsJson {
        generator: meta.generator_name.clone(),
        version: meta.crate_version.clone(),
        commands: commands.iter().map(command_to_json).collect(),
    };

    serde_json::to_string_pretty(&json).unwrap_or_default()
}

fn command_to_json(cmd: &CommandModel) -> CommandJson {
    CommandJson {
        name: cmd.name.clone(),
        full_path: cmd.full_path.clone(),
        about: cmd.about.clone(),
        long_about: cmd.long_about.clone(),
        usage: cmd.usage.clone(),
        aliases: cmd.aliases.clone(),
        arguments: cmd.arguments.iter().map(arg_to_json).collect(),
        options: cmd.options.iter().map(arg_to_json).collect(),
        flags: cmd.flags.iter().map(arg_to_json).collect(),
        subcommands: cmd.subcommands.iter().map(command_to_json).collect(),
        hidden: cmd.hidden,
    }
}

fn arg_to_json(arg: &crate::model::ArgModel) -> ArgJson {
    ArgJson {
        name: arg.name.clone(),
        about: arg.about.clone(),
        required: arg.required,
        default_value: arg.default_value.clone(),
        short: arg.short,
        long: arg.long.clone(),
        takes_value: arg.takes_value,
    }
}
