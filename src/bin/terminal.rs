use aegisr_engine::{AegisrCommand, Commands, ENGINE_DEVELOPER, ENGINE_NAME, ENGINE_VERSION};
use clap::Parser;
use colored::Colorize;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Parser)]
#[command(name = ENGINE_NAME, author = ENGINE_DEVELOPER[0], version = ENGINE_VERSION)]
pub struct AegTerminal {
    #[command(subcommand)]
    command: Commands,
}

impl AegTerminal {
    pub fn start() {
        let cli = AegTerminal::parse();
        let mut stream = match TcpStream::connect_timeout(
            &"127.0.0.1:1211".parse().unwrap(),
            Duration::from_secs(1),
        ) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("{}", "Error: daemon not running.".red());
                std::process::exit(1);
            }
        };

        let cmd = match &cli.command {
            Commands::Init(args) => AegisrCommand::Init {
                verbose: args.verbose,
                reset: args.reset,
            },
            Commands::List => AegisrCommand::List,
            Commands::Use(args) => AegisrCommand::Use {
                verbose: args.verbose,
                name: args.name.clone(),
            },
            Commands::New(args) => AegisrCommand::New {
                verbose: args.verbose,
                name: args.name.clone(),
            },
            Commands::Delete(args) => AegisrCommand::Delete {
                verbose: args.verbose,
                name: args.name.clone(),
            },
            Commands::Rename(args) => AegisrCommand::Rename {
                verbose: args.verbose,
                name: args.name.clone(),
                new_name: args.new_name.clone(),
            },
            Commands::Status => AegisrCommand::Status,
            Commands::Put(args) => AegisrCommand::Put {
                verbose: args.verbose,
                key: args.key.clone(),
                value: args.value.clone(),
            },
            Commands::Get(args) => AegisrCommand::Get {
                verbose: args.verbose,
                key: args.key.clone(),
            },
            Commands::Del(args) => AegisrCommand::Del {
                verbose: args.verbose,
                key: args.key.clone(),
            },
            Commands::Clear(args) => AegisrCommand::Clear {
                verbose: args.verbose,
            },
        };

        let cmd_bytes = serde_json::to_vec(&cmd).unwrap();
        stream.write_all(&cmd_bytes).unwrap();

        let mut response = vec![0; 4096];
        let n = stream.read(&mut response).unwrap();

        if let Ok(value) = serde_json::from_slice::<Value>(&response[..n]) {
            println!("{}", serde_json::to_string_pretty(&value).unwrap().green());
        } else {
            println!("{}", String::from_utf8_lossy(&response[..n]).red());
        }
    }
}

fn main() {
    AegTerminal::start();
}
