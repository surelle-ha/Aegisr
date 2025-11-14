use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

// ================
// INIT COMMAND
// ================
#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg(short, long)]
    pub reset: bool,
}

// ================
// USE COMMAND
// ================
#[derive(Args, Debug)]
pub struct UseArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg()]
    pub name: String,
}

// ================
// NEW COMMAND
// ================
#[derive(Args, Debug)]
pub struct NewArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg()]
    pub name: String,
}

// ================
// DELETE COMMAND
// ================
#[derive(Args, Debug)]
pub struct DeleteArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg()]
    pub name: String,
}

// ================
// RENAME COMMAND
// ================
#[derive(Args, Debug)]
pub struct RenameArgs {
    #[arg(short, long)]
    pub verbose: bool,

    #[arg()]
    pub name: String,

    #[arg()]
    pub new_name: String,
}

// ========================================================

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    List,
    Use(UseArgs),
    New(NewArgs),
    Delete(DeleteArgs),
    Rename(RenameArgs),
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
}
