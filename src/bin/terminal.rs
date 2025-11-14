use aegisr_engine::{AegisrCommand, Commands, ENGINE_DEVELOPER, ENGINE_NAME, ENGINE_VERSION};
use clap::Parser;
use colored::Colorize;
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

        // Connect to daemon
        let mut stream = match TcpStream::connect_timeout(
            &"127.0.0.1:8080".parse().unwrap(),
            Duration::from_secs(1),
        ) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("{}", "Error: Daemon is not running.".red().bold());
                std::process::exit(1);
            }
        };

        // Convert CLI → Shared command enum
        let cmd = match &cli.command {
            Commands::Init(args) => AegisrCommand::Init {
                verbose: args.verbose,
                reset: args.reset,
            },
            Commands::Use(args) => AegisrCommand::Use {
                verbose: args.verbose,
                name: args.name.clone(),
            },
        };

        // Send command
        let cmd_bytes = serde_json::to_vec(&cmd).unwrap();
        stream.write_all(&cmd_bytes).unwrap();

        // Read daemon response
        let mut response = vec![0; 1024];
        let n = stream.read(&mut response).unwrap();
        println!("{}", String::from_utf8_lossy(&response[..n]).green());
    }
}

fn main() {
    AegTerminal::start();
}
