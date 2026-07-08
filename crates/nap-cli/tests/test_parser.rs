use clap::Parser;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    base_dir: std::path::PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    Set {
        uri: String,
        key: String,
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_for_set_command() {
        let args = vec![
            "nap",
            "--base-dir",
            ".",
            "set",
            "star-ocean/character/hiro",
            "name",
            "Hiro",
        ];

        let cli = Cli::try_parse_from(args).expect("Failed to parse arguments");

        assert_eq!(cli.base_dir.to_str().unwrap(), ".");

        match cli.command {
            Commands::Set { uri, key, value } => {
                assert_eq!(uri, "star-ocean/character/hiro");
                assert_eq!(key, "name");
                assert_eq!(value, "Hiro");
            }
        }
    }
}
