use crate::modules::termenu::{Termenu, TermenuError};
use colored::Colorize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

/// =========================================
/// Note: This is an internal command intended
/// for development use only.
/// =========================================
pub fn register() -> Termenu {
    let mut command: Termenu = Termenu::new_command(
        "developer",
        "Add new command for development purposes. Will only work in debug mode.",
        |options: &HashMap<String, Option<String>>| {
            if cfg!(debug_assertions) {
                let verbose_mode: bool = options.contains_key("--verbose");

                let command_name: Option<String> = options
                    .get("--add")
                    .and_then(|v: &Option<String>| v.as_ref())
                    .map(|v: &String| v.trim().to_string());

                let command_name = match command_name {
                    Some(name) if !name.is_empty() => name,
                    _ => {
                        return Err(TermenuError::input_missing_error(Some(json!( {
                            "issue": "Action is required. Use `--add=<command_name>` to continue."
                        }))));
                    }
                };

                let file_name = format!("src/commands/{}_command.rs", command_name.to_lowercase());
                let path = Path::new(&file_name);

                if path.exists() {
                    return Err(TermenuError::input_unknown_error(Some(json!( {
                        "issue": format!("Command file already exists: {}", file_name)
                    }))));
                }

                // Create the command file
                fs::write(path, generate_command_template(&command_name)).map_err(|e| {
                    TermenuError::connection_unknown_error(Some(json!( {
                        "issue": format!("Failed to create file: {}", e)
                    })))
                })?;

                // Update mod.rs automatically
                let mod_file_path = Path::new("src/commands/mod.rs");
                let mod_line = format!("pub mod {}_command;\n", command_name.to_lowercase());

                if mod_file_path.exists() {
                    let content = fs::read_to_string(mod_file_path).unwrap_or_default();
                    if !content.contains(&mod_line) {
                        let mut file = fs::OpenOptions::new()
                            .append(true)
                            .open(mod_file_path)
                            .map_err(|e| {
                                TermenuError::connection_unknown_error(Some(json!( {
                                    "issue": format!("Failed to open mod.rs: {}", e)
                                })))
                            })?;
                        file.write_all(mod_line.as_bytes()).map_err(|e| {
                            TermenuError::connection_unknown_error(Some(json!( {
                                "issue": format!("Failed to write to mod.rs: {}", e)
                            })))
                        })?;
                        if verbose_mode {
                            println!(
                                "{} Registered command in mod.rs: {}",
                                "✔".green(),
                                mod_line.trim()
                            );
                        }
                    } else if verbose_mode {
                        println!("{} Command already registered in mod.rs.", "ℹ".cyan());
                    }
                } else if verbose_mode {
                    println!("{} mod.rs not found in src/commands/", "⚠".yellow());
                }

                if verbose_mode {
                    println!(
                        "{} Command file created successfully at: {}",
                        "✔".green(),
                        file_name
                    );
                }

                Ok(())
            } else {
                Err(TermenuError::framework_forbidden_error(Some(json!( {
                    "issue": "This command is only available in debug mode."
                }))))
            }
        },
    );

    command.add_option(
        "--verbose",
        "--verbose | Display detailed usage information.",
    );
    command.add_option(
        "--add",
        "--add=<command_name> | Specify the name of the command to add.",
    );

    command
}

#[warn(unused_variables)]
fn generate_command_template(name: &str) -> String {
    let struct_name = format!("{}Command", capitalize_first_letter(&name.to_lowercase()));
    format!(
        "use crate::modules::termenu::Termenu;\n\n\
        pub fn register() -> Termenu {{\n    \
            let command: Termenu = Termenu::new_command(\n        \
                \"{}\",\n        \
                \"Describe what this command does.\",\n        \
                |_options| {{\n            \
                    println!(\"Command '{}' executed.\");\n            \
                    Ok(())\n        \
                }},\n    );\n\n    \
            command\n\
        }}\n",
        name.to_lowercase(),
        name
    )
}

fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
