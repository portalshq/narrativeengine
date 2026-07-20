use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "nap", version, about, long_about = None)]
pub struct Cli {
    #[arg(long, short = 'd', global = true, env = "NAP_DIR")]
    pub base_dir: Option<PathBuf>,
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... I need to copy ALL command definitions here!
}
