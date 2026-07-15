use crate::model::{ArgModel, CommandModel, EnvVarModel};
use clap::CommandFactory;
use nap_cli::Cli;

pub fn extract_all_commands() -> Vec<CommandModel> {
    let mut cmd = Cli::command();
    let mut commands = Vec::new();

    for subcmd in cmd.get_subcommands_mut() {
        if !subcmd.is_hide_set() && subcmd.get_name() != "help" {
            commands.push(walk_command(subcmd, None));
        }
    }

    commands.sort_by(|a, b| a.name.cmp(&b.name));
    commands
}

pub fn extract_global_options() -> Vec<ArgModel> {
    let cmd = Cli::command();
    let mut options = Vec::new();

    for arg in cmd.get_arguments() {
        if arg.is_global_set() && !arg.is_hide_set() {
            options.push(extract_arg(arg));
        }
    }

    options.sort_by(|a, b| a.name.cmp(&b.name));
    options
}

fn walk_command(cmd: &mut clap::Command, parent_path: Option<String>) -> CommandModel {
    let name = cmd.get_name().to_string();
    let full_path = match &parent_path {
        Some(parent) => format!("{parent} {name}"),
        None => name.clone(),
    };

    let about = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();

    let long_about = cmd.get_long_about().map(|s| s.to_string());

    // render_usage requires &mut self
    let usage = cmd.render_usage().to_string();

    let aliases: Vec<String> = cmd.get_aliases().map(|s| s.to_string()).collect();

    let visible_aliases: Vec<String> = cmd.get_visible_aliases().map(|s| s.to_string()).collect();

    let mut arguments = Vec::new();
    let mut options = Vec::new();
    let mut flags = Vec::new();

    for arg in cmd.get_arguments() {
        if arg.is_hide_set() {
            continue;
        }
        let model = extract_arg(arg);
        if arg.is_positional() {
            arguments.push(model);
        } else if is_flag(arg) {
            flags.push(model);
        } else {
            options.push(model);
        }
    }

    let mut subcommands = Vec::new();
    for subcmd in cmd.get_subcommands_mut() {
        if !subcmd.is_hide_set() && subcmd.get_name() != "help" {
            subcommands.push(walk_command(subcmd, Some(full_path.clone())));
        }
    }
    subcommands.sort_by(|a, b| a.name.cmp(&b.name));

    let env_vars: Vec<EnvVarModel> = options
        .iter()
        .filter_map(|o| {
            o.env.as_ref().map(|env| EnvVarModel {
                name: env.clone(),
                about: format!("Override for --{}", o.long.as_deref().unwrap_or(&o.name)),
            })
        })
        .collect();

    CommandModel {
        name,
        parent_path,
        full_path,
        about,
        long_about,
        usage,
        aliases,
        visible_aliases,
        arguments,
        options,
        flags,
        subcommands,
        hidden: cmd.is_hide_set(),
        deprecated: false,
        env_vars,
        examples: Vec::new(),
    }
}

fn is_flag(arg: &clap::Arg) -> bool {
    arg.get_num_args()
        .map(|range| range.max_values() == 0)
        .unwrap_or(false)
}

fn extract_arg(arg: &clap::Arg) -> ArgModel {
    let name = arg.get_id().to_string();

    let about = arg.get_help().map(|s| s.to_string()).unwrap_or_default();

    let long_about = arg.get_long_help().map(|s| s.to_string());

    let required = arg.is_required_set();

    let takes_value = arg
        .get_num_args()
        .map(|range| range.min_values() > 0)
        .unwrap_or(false);

    let default_value = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    let value_name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|s| s.to_string());

    let possible_values: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|pv| pv.get_name().to_string())
        .collect();

    let short = arg.get_short();

    let long = arg.get_long().map(|s| s.to_string());

    let multiple = matches!(
        arg.get_action(),
        clap::ArgAction::Append | clap::ArgAction::Count
    );

    let hidden = arg.is_hide_set();

    let env = arg.get_env().map(|s| s.to_string_lossy().to_string());

    ArgModel {
        name,
        about,
        long_about,
        required,
        takes_value,
        default_value,
        value_name,
        possible_values,
        short,
        long,
        multiple,
        hidden,
        env,
    }
}
