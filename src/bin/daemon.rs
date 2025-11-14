use aegisr_engine::{AegCore, AegFileSystem, AegisrCommand};
use std::net::SocketAddr;
use std::process;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;

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

/// Initialize tracing subscriber:
/// - Console logs
/// - Optional daily-rotating file logs (non-blocking writer)
fn init_tracing(cfg: &LoggerConfig) {
    let env_filter = EnvFilter::try_new(&cfg.level).unwrap_or_else(|_| EnvFilter::new("info"));

    // Console pretty layer
    let console_layer = fmt::layer().pretty().with_target(false).with_ansi(true);

    let base_sub = Registry::default().with(env_filter).with(console_layer);

    if cfg.log_to_file {
        // Create daemon log directory
        let mut log_dir = AegFileSystem::get_config_path();
        std::fs::create_dir_all(&log_dir).ok();
        log_dir.push("logs");
        std::fs::create_dir_all(&log_dir).ok();

        // Daily rotation
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "daemon.log");

        // IMPORTANT: use non_blocking() instead of clone()
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
        info!("======================================");
        info!("        AEGISR DAEMON STARTED         ");
        info!("--------------------------------------");
        info!(host = %self.hostname, pid = self.pid, listen = %self.address, "Daemon Info");
        info!("======================================");
    }
}

/// Command handler
async fn handle_command(cmd: AegisrCommand) -> String {
    match cmd {
        AegisrCommand::Init { verbose, reset } => {
            if reset {
                warn!("Reset requested — clearing engine files");
                AegFileSystem::reset_files();
            }
            info!("Validating engine files");
            AegFileSystem::validate_files();

            let engine: AegCore = AegCore::load();
            engine.save();

            if verbose {
                info!("Verbose: init completed");
            }

            format!(
                "✓ Engine initialized. Active Collection: {}",
                AegCore::load().get_active_collection()
            )
        }

        AegisrCommand::Use { verbose, name } => {
            info!(%name, "Switching active collection");

            let mut engine = AegCore::load();
            engine.set_active_collection(&name);
            engine.save();

            if verbose {
                info!("Verbose: collection switched");
            }

            format!("✓ Active Collection: {}", engine.get_active_collection())
        }
    }
}

#[tokio::main]
async fn main() {
    let logger_cfg = LoggerConfig {
        log_to_file: true,
        level: std::env::var("AEGISR_LOG_LEVEL").unwrap_or("info".into()),
    };

    let daemon = AegDaemon::new("127.0.0.1:8080", logger_cfg);
    daemon.start().await;
}
