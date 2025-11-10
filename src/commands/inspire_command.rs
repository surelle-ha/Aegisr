use crate::modules::termenu::{Termenu, TermenuError};
use colored::Colorize;
use reqwest;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

///
/// =========================================
/// Note: This is an internal command intended
/// for development use only. You may remove it
/// depending on your specific use case or how
/// you plan to use this tool.
/// =========================================
///
pub fn register() -> Termenu {
    let mut inspire_cmd = Termenu::new_async_command(
        "inspire",
        "Fetches and displays a random inspirational quote.",
        |_options: HashMap<String, Option<String>>| {
            Box::pin(async move {
                // Create a client that ignores SSL certificate validation
                let client = reqwest::Client::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .map_err(|e| {
                        TermenuError::connection_unknown_error(Some(json!({
                            "issue": format!("Client build failed: {}", e)
                        })))
                    })?;

                // Send request to public quotes API
                let resp = client
                    .get("https://api.quotable.io/random")
                    .send()
                    .await
                    .map_err(|e| {
                        TermenuError::connection_unknown_error(Some(json!({
                            "issue": format!("Request failed: {}", e)
                        })))
                    })?;

                // Convert response to text
                let text = resp.text().await.map_err(|e| {
                    TermenuError::connection_unknown_error(Some(json!({
                        "issue": format!("Failed to read response: {}", e)
                    })))
                })?;

                // Parse JSON
                let resp_json: Value = serde_json::from_str(&text).map_err(|e| {
                    TermenuError::input_unknown_error(Some(json!({
                        "issue": format!("Invalid JSON: {}", e)
                    })))
                })?;

                let quote = resp_json["content"].as_str().unwrap_or("No quote found.");
                let author = resp_json["author"].as_str().unwrap_or("Unknown");

                println!("💡 \"{}\"", quote.red());
                println!("   — {}", author);

                Ok(())
            }) as Pin<Box<dyn Future<Output = Result<(), TermenuError>> + Send>>
        },
    );

    inspire_cmd.add_option(
        "--category",
        "Specify a quote category (optional, not used in current API).",
    );

    inspire_cmd
}
