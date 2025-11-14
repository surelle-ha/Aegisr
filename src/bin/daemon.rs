use aegisr_engine::{AegCore, AegFileSystem, AegisrCommand};
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;

use clap::Parser;
use hostname::get as get_hostname;
use tracing::{debug, error, info, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt};

/// Logger config
pub struct LoggerConfig {
    pub log_to_file: bool,
    pub level: String,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_to_file: true,
            level: "info".into(),
        }
    }
}

/// JSON config structure
#[derive(Debug, Deserialize)]
struct DaemonConfig {
    host: Option<String>,
    port: Option<u16>,
}

/// CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about = "AEGISR Daemon")]
struct CliArgs {
    /// Host to listen on
    #[arg(short = 'H', long)]
    host: Option<String>,

    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// Config JSON file
    #[arg(short, long)]
    config: Option<String>,
}

/// Initialize tracing subscriber
fn init_tracing(cfg: &LoggerConfig) {
    let env_filter = EnvFilter::try_new(&cfg.level).unwrap_or_else(|_| EnvFilter::new("info"));
    let console_layer = fmt::layer().pretty().with_target(false).with_ansi(true);
    let base_sub = Registry::default().with(env_filter).with(console_layer);

    if cfg.log_to_file {
        let mut log_dir = AegFileSystem::get_config_path();
        std::fs::create_dir_all(&log_dir).ok();
        log_dir.push("logs");
        std::fs::create_dir_all(&log_dir).ok();

        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "daemon.log");
        let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_writer(file_writer)
            .with_ansi(false)
            .with_target(true)
            .compact();

        tracing::subscriber::set_global_default(base_sub.with(file_layer))
            .expect("Failed setting tracing subscriber");
    } else {
        tracing::subscriber::set_global_default(base_sub)
            .expect("Failed setting tracing subscriber");
    }
}

pub struct AegDaemon {
    pub address: SocketAddr,
    pub pid: u32,
    pub hostname: String,
    pub logger_cfg: LoggerConfig,
}

impl AegDaemon {
    pub fn new(address: &str, logger_cfg: LoggerConfig) -> Self {
        let hostname = get_hostname()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or("unknown".into());

        Self {
            address: address.parse().expect("Invalid address"),
            pid: process::id(),
            hostname,
            logger_cfg,
        }
    }

    pub async fn start(&self) {
        AegFileSystem::validate_files();

        init_tracing(&self.logger_cfg);
        self.print_banner();
        self.spawn_background_worker();

        info!(
            "Daemon listening on {} (host: {}, pid: {})",
            self.address, self.hostname, self.pid
        );

        let listener = match TcpListener::bind(self.address).await {
            Ok(l) => l,
            Err(e) => {
                error!("Bind failed on {}: {}", self.address, e);
                return;
            }
        };

        loop {
            tokio::select! {
                Ok((mut socket, addr)) = listener.accept() => {
                    info!(%addr, "Client connected");

                    tokio::spawn(async move {
                        let mut buffer = vec![0u8; 4096];

                        match socket.read(&mut buffer).await {
                            Ok(n) if n > 0 => {
                                let data = &buffer[..n];

                                match serde_json::from_slice::<AegisrCommand>(data) {
                                    Ok(cmd) => {
                                        info!(?cmd, "Command received");

                                        let response = handle_command(cmd).await;

                                        if let Err(e) = socket.write_all(response.as_bytes()).await {
                                            error!(%e, "Failed sending response");
                                        }
                                    }
                                    Err(e) => {
                                        warn!(%e, "Invalid JSON command");
                                        let _ = socket.write_all(b"ERR: invalid command\n").await;
                                    }
                                }
                            }
                            Ok(_) => debug!("Connection closed without data"),
                            Err(e) => error!(%e, "Socket read error"),
                        }
                    });
                }

                _ = signal::ctrl_c() => {
                    info!("Ctrl+C detected — shutting down daemon");
                    break;
                }
            }
        }

        info!("Daemon shutdown complete");
    }

    fn spawn_background_worker(&self) {
        let pid = self.pid;

        thread::spawn(move || {
            loop {
                info!(pid, "Background worker heartbeat");
                thread::sleep(Duration::from_secs(10));
            }
        });
    }

    fn print_banner(&self) {
        let reset = "\x1b[0m";
        let green = "\x1b[32m";
        let cyan = "\x1b[36m";
        let yellow = "\x1b[33m";

        println!("{}======================================{}", green, reset);
        println!("{}        AEGISR DAEMON STARTED         {}", cyan, reset);
        println!("{}--------------------------------------{}", yellow, reset);
        println!(
            "{}Daemon Info{} \n \t host: {} \n \t pid: {} \n \t listen: {}",
            green, reset, self.hostname, self.pid, self.address
        );
        println!("{}======================================{}", green, reset);
    }
}

async fn handle_command(cmd: AegisrCommand) -> String {
    match cmd {
        AegisrCommand::New { verbose, name } => {
            let resp = AegCore::create_collection(&name);
            if verbose {
                info!("Verbose: {}", resp);
            }
            resp
        }
        AegisrCommand::List => {
            let engine = AegCore::load();
            if engine.collections.is_empty() {
                "No collections found".to_string()
            } else {
                let mut output = String::from("✓ Collections:\n");
                for collection in engine.collections {
                    output.push_str(&format!("• {}\n", collection));
                }
                output
            }
        }
        AegisrCommand::Delete { verbose, name } => {
            let resp = AegCore::delete_collection(&name);
            if verbose {
                info!("Verbose: {}", resp);
            }
            resp
        }
        AegisrCommand::Rename {
            verbose,
            name,
            new_name,
        } => {
            let resp = AegCore::rename_collection(&name, &new_name);
            if verbose {
                info!("Verbose: {}", resp);
            }
            resp
        }
        AegisrCommand::Use { verbose, name } => {
            // always reload fresh
            let mut engine = AegCore::load();
            debug!("Collections loaded from disk: {:?}", engine.collections);

            match engine.set_active_collection(&name) {
                Ok(_) => {
                    if verbose {
                        info!("Verbose: switched to '{}'", name);
                    }
                    format!("✓ Active Collection: {}", name)
                }
                Err(e) => {
                    error!("Failed to switch to '{}': {}", name, e);
                    format!("✗ Failed to switch to '{}': {}", name, e)
                }
            }
        }

        AegisrCommand::Init { verbose, reset } => {
            if reset {
                warn!("Reset requested — clearing engine files");
                AegFileSystem::reset_files();
            }

            info!("Initializing engine files");
            let config_path = AegFileSystem::initialize_config(Some(true), Some(verbose));

            // Always reload fresh after initialization
            let mut engine = AegCore::load();

            // Ensure at least the default collection exists
            if engine.collections.is_empty() {
                engine.collections.push("default".to_string());
            }
            if engine.active_collection.is_empty() {
                engine.active_collection = engine.collections[0].clone();
            }

            // Save back to collection.lock
            engine.save();

            if verbose {
                info!("Verbose: init completed at {}", config_path.display());
            }

            format!(
                "✓ Engine initialized. Active Collection: {}",
                engine.get_active_collection()
            )
        }
    }
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    // Load config file if provided
    let file_config = if let Some(cfg_path) = &args.config {
        let cfg_str = fs::read_to_string(cfg_path)
            .unwrap_or_else(|_| panic!("Failed to read config file: {}", cfg_path));
        serde_json::from_str::<DaemonConfig>(&cfg_str).unwrap_or(DaemonConfig {
            host: None,
            port: None,
        })
    } else {
        DaemonConfig {
            host: None,
            port: None,
        }
    };

    // Determine host and port: CLI > config file > default
    let host = args.host.or(file_config.host).unwrap_or("127.0.0.1".into());
    let port = args.port.or(file_config.port).unwrap_or(1211);

    let address = format!("{}:{}", host, port);

    let logger_cfg = LoggerConfig {
        log_to_file: true,
        level: std::env::var("AEGISR_LOG_LEVEL").unwrap_or("info".into()),
    };

    AegFileSystem::validate_files();

    let daemon = AegDaemon::new(&address, logger_cfg);
    daemon.start().await;
}
