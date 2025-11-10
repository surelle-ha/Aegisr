mod commands;
mod modules;

use colored::Colorize;
use commands::{developer_command, hello_world_command, inspire_command};
use modules::termenu::Termenu;

#[tokio::main]
async fn main() {
    match Termenu::processor(Termenu::validate_commands(vec![
        developer_command::register(),
        hello_world_command::register(),
        inspire_command::register(),
    ]))
    .await
    {
        Ok(_) => {}
        Err(e) => eprintln!("[{}] {}", "Error".red().bold(), e),
    }
}
