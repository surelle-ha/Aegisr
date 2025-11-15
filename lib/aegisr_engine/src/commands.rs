use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

// ================
// INIT COMMAND
// ================
#[derive(Args, Debug)]
#[command(about = "Initialize the Aegisr engine")]
pub struct InitArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(short, long, help = "Reset existing data before initialization")]
    pub reset: bool,
}

// ================
// USE COMMAND
// ================
#[derive(Args, Debug)]
#[command(about = "Switch the active collection")]
pub struct UseArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(help = "Name of the collection to use")]
    pub name: String,
}

// ================
// NEW COMMAND
// ================
#[derive(Args, Debug)]
#[command(about = "Create a new collection")]
pub struct NewArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(help = "Name of the new collection")]
    pub name: String,
}

// ================
// DELETE COMMAND
// ================
#[derive(Args, Debug)]
#[command(about = "Delete an existing collection")]
pub struct DeleteArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(help = "Name of the collection to delete")]
    pub name: String,
}

// ================
// RENAME COMMAND
// ================
#[derive(Args, Debug)]
#[command(about = "Rename a collection")]
pub struct RenameArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(help = "Current name of the collection")]
    pub name: String,

    #[arg(help = "New name for the collection")]
    pub new_name: String,
}

// ========================================================

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Initialize the Aegisr engine")]
    Init(InitArgs),

    #[command(about = "List all collections")]
    List,

    #[command(about = "Switch the active collection")]
    Use(UseArgs),

    #[command(about = "Create a new collection")]
    New(NewArgs),

    #[command(about = "Delete an existing collection")]
    Delete(DeleteArgs),

    #[command(about = "Rename a collection")]
    Rename(RenameArgs),

    #[command(about = "Show the currently active collection")]
    Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AegisrCommand {
    Init {
        verbose: bool,
        reset: bool,
    },
    List,
    Use {
        verbose: bool,
        name: String,
    },
    New {
        verbose: bool,
        name: String,
    },
    Delete {
        verbose: bool,
        name: String,
    },
    Rename {
        verbose: bool,
        name: String,
        new_name: String,
    },
    Status,
}
