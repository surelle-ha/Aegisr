use crate::modules::termenu::Banner;
use colored::Colorize;
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::exceptions::TermenuError;

/// Constants
pub const MAX_COMMAND: i32 = 100;
pub const MIN_COMMAND: i32 = 0;

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
    dependencies: Option<toml::value::Table>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    authors: Vec<String>,
}

#[derive(Clone)]
pub struct Termenu {
    pub command: String,
    pub description: String,
    pub options: Vec<(String, String)>,
    #[allow(clippy::type_complexity)]
    pub handler: Option<
        Arc<dyn Fn(&HashMap<String, Option<String>>) -> Result<(), TermenuError> + Send + Sync>,
    >,
    #[allow(clippy::type_complexity)]
    pub async_handler: Option<
        Arc<
            dyn Fn(
                    HashMap<String, Option<String>>,
                )
                    -> Pin<Box<dyn Future<Output = Result<(), TermenuError>> + Send>>
                + Send
                + Sync,
        >,
    >,
}

impl Termenu {
    /// Create a new synchronous command with a handler
    pub fn new_command<F>(command: &str, description: &str, handler: F) -> Self
    where
        F: Fn(&HashMap<String, Option<String>>) -> Result<(), TermenuError> + Send + Sync + 'static,
    {
        Self {
            command: command.to_string(),
            description: description.to_string(),
            options: Vec::new(),
            handler: Some(Arc::new(handler)),
            async_handler: None,
        }
    }

    /// Create a new asynchronous command with a handler
    pub fn new_async_command<F, Fut>(command: &str, description: &str, handler: F) -> Self
    where
        F: Fn(HashMap<String, Option<String>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), TermenuError>> + Send + 'static,
    {
        Self {
            command: command.to_string(),
            description: description.to_string(),
            options: Vec::new(),
            handler: None,
            async_handler: Some(Arc::new(move |opts| Box::pin(handler(opts)))),
        }
    }

    /// Add an option with a description
    pub fn add_option(&mut self, option: &str, description: &str) {
        self.options
            .push((option.to_string(), description.to_string()));
    }

    /// Parse raw arguments into key/value map
    pub fn parse_options(
        &self,
        raw_args: &[String],
    ) -> Result<HashMap<String, Option<String>>, TermenuError> {
        let mut parsed: HashMap<String, Option<String>> = HashMap::new();

        for arg in raw_args {
            if let Some((key, value)) = arg.split_once('=') {
                if self.options.iter().any(|(opt, _)| opt == key) {
                    parsed.insert(key.to_string(), Some(value.to_string()));
                } else {
                    return Err(TermenuError::invalid_command_error(Some(json!({
                        "issue": format!("Unknown option: '{}'", key)
                    }))));
                }
            } else if self.options.iter().any(|(opt, _)| opt == arg) {
                parsed.insert(arg.clone(), None);
            } else {
                return Err(TermenuError::invalid_command_error(Some(json!({
                    "issue": format!("Unknown option: '{}'", arg)
                }))));
            }
        }

        Ok(parsed)
    }

    /// Execute either sync or async handler automatically
    pub async fn execute(
        &self,
        options: HashMap<String, Option<String>>,
    ) -> Result<(), TermenuError> {
        if let Some(handler) = &self.handler {
            handler(&options)
        } else if let Some(async_handler) = &self.async_handler {
            async_handler(options).await
        } else {
            Err(TermenuError::invalid_command_error(Some(json!({
                "issue": "No handler found for this command."
            }))))
        }
    }

    /// Validate a list of registered commands
    pub fn validate_commands(commands: Vec<Termenu>) -> Vec<Termenu> {
        if commands.len() as i32 > MAX_COMMAND {
            panic!(
                "{} {} (max allowed: {})",
                "Too many commands registered:".red().bold(),
                commands.len(),
                MAX_COMMAND
            );
        }

        if commands.len() as i32 <= MIN_COMMAND {
            panic!(
                "{} {} (min allowed: {})",
                "Minimum number of commands not reached:".red().bold(),
                commands.len(),
                MIN_COMMAND
            );
        }

        let mut seen: HashSet<String> = HashSet::new();
        for cmd in &commands {
            if !seen.insert(cmd.command.clone()) {
                panic!(
                    "{} '{}' ",
                    "Duplicate command detected:".red().bold(),
                    cmd.command
                );
            }
        }

        for cmd in &commands {
            if cmd.command.trim().is_empty() {
                panic!("{}", "Command name cannot be empty".red().bold());
            }

            if cmd.description.trim().is_empty() {
                panic!(
                    "{} '{}' ",
                    "Command description missing for".red().bold(),
                    cmd.command
                );
            }
        }

        if cfg!(debug_assertions) {
            println!(
                "{} {} registered successfully.\n",
                "✔".green(),
                format!("{} commands", commands.len()).bold()
            );
        }

        commands
    }

    /// Built-in help system: display all registered commands
    fn show_help(
        commands: &[Termenu],
        specific: Option<&str>,
        verbose: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string("Cargo.toml")?;
        let cargo_toml: CargoToml = toml::from_str(&content)?;

        Banner::render("Termenu");
        println!(
            "{} {}",
            "Developer:".green(),
            cargo_toml.package.authors[0].green().bold()
        );
        println!(
            "{} {}",
            "Version:".green(),
            cargo_toml.package.version.green().bold()
        );
        println!(
            "{}\n  {} {}",
            "Usage:".bold(),
            cargo_toml.package.name.green().bold(),
            "[Command] <options>".bold()
        );
        println!(
            "  Note: Check `{} help --command=<command_name>` to view guide.",
            cargo_toml.package.name.bold()
        );
        println!("{}", "Available Commands:".yellow().bold());

        for cmd in commands {
            if let Some(spec) = specific {
                if cmd.command != spec {
                    continue;
                }
            }
            println!("  {:<10} • {}", cmd.command.green(), cmd.description);
            if verbose || specific.is_some() {
                for (opt, desc) in &cmd.options {
                    println!("    {:<12} {}", opt.blue(), desc);
                }
                println!();
            }
        }

        Ok(())
    }

    /// Process CLI input and execute matching command
    pub async fn processor(mut commands: Vec<Termenu>) -> Result<(), TermenuError> {
        // --- Clone commands for use inside the help closure ---
        let help_commands = commands.clone();

        // --- Add built-in `help` dynamically ---
        let mut help_bi: Termenu = Termenu::new_command(
            "help",
            "Show usage guide and command descriptions.",
            move |options| {
                let verbose = options.contains_key("--verbose");
                let specific = options.get("--command").and_then(|v| v.clone());
                Termenu::show_help(&help_commands, specific.as_deref(), verbose);
                Ok(())
            },
        );

        // ✅ add options properly (mut required)
        help_bi.add_option("--verbose", "Display detailed usage information.");
        help_bi.add_option(
            "--command",
            "Show help for a specific command (e.g. --command=test).",
        );

        commands.push(help_bi);

        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            Self::show_help(&commands, None, false);
            return Ok(());
        }

        let command_name = args[1].clone();

        if let Some(termenu) = commands.iter().find(|t| t.command == command_name) {
            let raw_options: &[String] = &args[2..];
            let parsed_options = termenu.parse_options(raw_options)?;

            if let Err(err) = termenu.execute(parsed_options).await {
                return Err(err); // ✅ keep original error (no double wrap)
            }
        } else {
            return Err(TermenuError::invalid_command_error(Some(json!({
                "issue": format!(
                    "invalid command '{}'. Run with 'help' to view available commands.",
                    command_name
                )
            }))));
        }

        Ok(())
    }
}
