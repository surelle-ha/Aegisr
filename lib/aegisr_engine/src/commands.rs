use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    Use(UseArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub reset: bool,
}

#[derive(Args, Debug)]
pub struct UseArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AegisrCommand {
    Init { verbose: bool, reset: bool },
    Use { verbose: bool, name: String },
}
