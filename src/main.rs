mod commands;
mod modules;
mod utils;

use colored::Colorize;
use commands::{developer_command, init_command};
use modules::termenu::Termenu;

#[tokio::main]
async fn main() {
    match Termenu::processor(Termenu::validate_commands(vec![
        developer_command::register(),
        init_command::register(),
    ]))
    .await
    {
        Ok(_) => {}
        Err(e) => eprintln!("[{}] {}", "Error".red().bold(), e),
    }
}
