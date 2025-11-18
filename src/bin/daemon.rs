use aegisr_engine::{AegCore, AegFileSystem, AegisrCommand};
use clap::Parser;
use hostname::get as get_hostname;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;
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
    #[arg(short = 'H', long)]
    host: Option<String>,
    #[arg(short, long)]
    port: Option<u16>,
    #[arg(short, long)]
    config: Option<String>,
}

/// Standard JSON response
#[derive(Serialize)]
struct JsonResponse<T: Serialize> {
    status: String,
    message: String,
    data: Option<T>,
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
        AegCore::start_background_saver(1);
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
                                let response_json = match serde_json::from_slice::<AegisrCommand>(data) {
                                Ok(cmd) => handle_command(cmd).await.to_json(),
                                    Err(_) => serde_json::to_string(&json!({
                                        "status": "error",
                                        "message": "Invalid command"
                                    })).unwrap(),
                                };
                                if let Err(e) = socket.write_all(response_json.as_bytes()).await {
                                    error!(%e, "Failed sending response");
                                }
                            }
                            Ok(_) => debug!("Connection closed without data"),
                            Err(e) => error!(%e, "Socket read error"),
                        }
                    });
                }

                _ = signal::ctrl_c() => {
                    info!("Ctrl+C detected — shutting down daemon");
                    AegCore::stop_background_saver();
                    AegCore::flush_now();

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
        println!("==== AEGISR DAEMON ====");
        println!(
            "Host: {} | PID: {} | Listening: {}",
            self.hostname, self.pid, self.address
        );
        println!("=======================");
    }
}

enum CommandResult {
    Text { message: String, success: bool },
    List { items: Vec<String>, success: bool },
}

impl CommandResult {
    fn to_json(&self) -> String {
        match self {
            CommandResult::Text { message, success } => serde_json::to_string(&json!({
                "status": if *success { "ok" } else { "error" },
                "message": message
            }))
                .unwrap(),
            CommandResult::List { items, success } => serde_json::to_string(&json!({
                "status": if *success { "ok" } else { "error" },
                "data": items
            }))
                .unwrap(),
        }
    }
}

async fn handle_command(cmd: AegisrCommand) -> CommandResult {
    match cmd {
        AegisrCommand::New { verbose, name } => {
            let resp = AegCore::create_collection(&name);
            if verbose {
                info!("Verbose: {}", resp);
            }
            CommandResult::Text {
                message: resp,
                success: true,
            }
        }
        AegisrCommand::List => {
            let engine: AegCore = AegCore::load();
            if engine.collections.is_empty() {
                CommandResult::Text {
                    message: "No collections found".into(),
                    success: true,
                }
            } else {
                CommandResult::List {
                    items: engine.collections.clone(),
                    success: true,
                }
            }
        }
        AegisrCommand::Delete { verbose, name } => {
            let resp = AegCore::delete_collection(&name);
            if verbose {
                info!("Verbose: {}", resp);
            }
            CommandResult::Text {
                message: resp,
                success: true,
            }
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
            CommandResult::Text {
                message: resp,
                success: true,
            }
        }
        AegisrCommand::Use { verbose, name } => {
            let mut engine = AegCore::load();
            match engine.set_active_collection(&name) {
                Ok(_) => {
                    if verbose {
                        info!("Verbose: switched to '{}'", name);
                    }
                    CommandResult::Text {
                        message: format!("Active Collection set to '{}'", name),
                        success: true,
                    }
                }
                Err(e) => {
                    error!("Failed to switch to '{}': {}", name, e);
                    CommandResult::Text {
                        message: format!("Failed to switch to '{}': {}", name, e),
                        success: false,
                    }
                }
            }
        }
        AegisrCommand::Init { verbose, reset } => {
            if reset {
                warn!("Reset requested — clearing engine files");
                AegFileSystem::reset_files();
            }
            let config_path = AegFileSystem::initialize_config(Some(reset), Some(verbose));
            let mut engine = AegCore::load();
            if engine.collections.is_empty() {
                engine.collections.push("default".to_string());
            }
            if engine.active_collection.is_empty() {
                engine.active_collection = engine.collections[0].clone();
            }
            engine.save();
            if verbose {
                info!("Verbose: init completed at {}", config_path.display());
            }
            CommandResult::Text {
                message: format!(
                    "Engine initialized. Active Collection: {}",
                    engine.get_active_collection()
                ),
                success: true,
            }
        }
        AegisrCommand::Status => {
            let engine = AegCore::load();
            CommandResult::Text {
                message: engine.get_active_collection().to_string(),
                success: true,
            }
        }
        AegisrCommand::Put {
            verbose,
            key,
            value,
        } => {
            let resp = AegCore::put_value(&key, &value);
            if verbose {
                info!("Verbose: PUT {} = {}", key, value);
            }
            CommandResult::Text {
                message: resp,
                success: true,
            }
        }
        AegisrCommand::Get { verbose, key } => match AegCore::get_value(&key) {
            Some(v) => {
                if verbose {
                    info!("Verbose: GET {} = {}", key, v);
                }
                CommandResult::Text {
                    message: v,
                    success: true,
                }
            }
            None => {
                if verbose {
                    warn!("Verbose: GET {} not found", key);
                }
                CommandResult::Text {
                    message: "Key not found".into(),
                    success: false,
                }
            }
        },
        AegisrCommand::Del { verbose, key } => {
            let resp = AegCore::delete_value(&key);
            if verbose {
                info!("Verbose: DEL {}", key);
            }
            CommandResult::Text {
                message: resp,
                success: true,
            }
        }
        AegisrCommand::Clear { verbose } => {
            let resp = AegCore::clear_values();
            if verbose {
                info!("Verbose: CLEAR all values");
            }
            CommandResult::Text {
                message: resp,
                success: true,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    let file_config = if let Some(cfg_path) = &args.config {
        let cfg_str = fs::read_to_string(cfg_path).unwrap_or_default();
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

    let host = args.host.or(file_config.host).unwrap_or("127.0.0.1".into());
    let port = args.port.or(file_config.port).unwrap_or(1211);
    let address = format!("{}:{}", host, port);

    let logger_cfg = LoggerConfig {
        log_to_file: true,
        level: std::env::var("AEGISR_LOG_LEVEL").unwrap_or("info".into()),
    };

    let daemon = AegDaemon::new(&address, logger_cfg);
    daemon.start().await;
}
