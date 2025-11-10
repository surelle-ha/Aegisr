use crate::modules::termenu::Termenu;
use crate::utils::constants::{aegisr, cmd_init};
use base64::{Engine as _, engine::general_purpose};
use dirs_next::home_dir;
use std::fs;
use std::path::PathBuf;

// TODO: Add DocType Comment.
fn generate_32_random_bytes(verbose_mode: Option<bool>) -> [u8; 32] {
    let verbose_mode = verbose_mode.unwrap_or(false);

    let key_bytes: [u8; 32] = rand::random();
    if verbose_mode {
        println!("  • Random 32-byte key (raw): {:?}", key_bytes);
    }
    key_bytes
}

// TODO: Add DocType Comment.
fn encode_base64(input: impl AsRef<[u8]>, verbose_mode: Option<bool>) -> String {
    let verbose_mode = verbose_mode.unwrap_or(false);

    let key_encoded: String = general_purpose::STANDARD.encode(input.as_ref());
    if verbose_mode {
        println!("  • Random 32-byte key (base64): {}", key_encoded);
    }
    key_encoded
}

// TODO: Add DocType Comment.
fn hash_blake3(target: String, verbose_mode: Option<bool>) -> String {
    let verbose_mode = verbose_mode.unwrap_or(false);

    let mut hasher = blake3::Hasher::new();
    hasher.update(target.as_bytes());
    let hash = hasher.finalize();
    if verbose_mode {
        println!("  • Hashed Blake3 key: {}", hash.to_hex());
    }
    hash.to_hex().to_string()
}

// TODO: Add DocType Comment.
fn build_authorization_key(verbose_mode: Option<bool>) -> String {
    let verbose_mode = verbose_mode.unwrap_or(false);

    let key_bytes: [u8; 32] = generate_32_random_bytes(Some(verbose_mode));
    let key_base64: String = encode_base64(key_bytes, Some(verbose_mode));
    let key_blake3: String = hash_blake3(key_base64, Some(verbose_mode));
    let authorization_key: String = encode_base64(key_blake3, Some(verbose_mode));
    if verbose_mode {
        println!(
            "  • Authorization Key successfully built: {}",
            authorization_key
        );
    }
    authorization_key
}

fn initialize_config(overwrite: Option<bool>, verbose_mode: Option<bool>) -> PathBuf {
    // TODO: Fix overwrite logic.
    let overwrite_mode = overwrite.unwrap_or(false);
    let verbose_mode = verbose_mode.unwrap_or(false);

    // Build config directory path
    let mut dir = home_dir().expect("Failed to get home directory");
    dir.push(aegisr::CONFIG_DIR);

    // Remove directory if overwrite is requested
    if overwrite_mode && dir.exists() {
        fs::remove_dir_all(&dir).expect("Failed to remove existing config directory");
    }

    // Create directory if it doesn't exist
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create config directory");
    }

    // Generate 'Authorization_Key'
    let authorization_key = build_authorization_key(Some(verbose_mode));

    // Store authorization_key in 'Authorization_Key' file
    let key_path = dir.join("Authorization_Key");
    fs::write(&key_path, authorization_key.as_bytes())
        .expect("Failed to write Authorization_Key file");

    dir
}

pub fn register() -> Termenu {
    // TODO: Hardcode cmd_init. Rename constant.rs to config.rs.
    let mut command = Termenu::new_command(
        cmd_init::COMMAND_KEY,
        "Initialize config file.",
        |options| {
            // Prepare Execution
            let verbose_mode: bool = options.get(cmd_init::VERBOSE_ATTR_KEY).is_some();
            let fresh_mode: bool = options.get(cmd_init::FRESH_ATTR_KEY).is_some();

            // Execute Command
            if verbose_mode {
                println!(
                    "Command `{}` executed: \n  • Verbose: {}\n  • Fresh Initialize: {}",
                    cmd_init::COMMAND_KEY,
                    verbose_mode,
                    fresh_mode
                );
            }

            // Initialize Config
            let config_dir: PathBuf = initialize_config(Some(fresh_mode), Some(verbose_mode));

            print!("✓ Aegisr config created at {:?}", config_dir);
            Ok(())
        },
    );

    command.add_option(cmd_init::VERBOSE_ATTR_KEY, "Run command in verbose mode.");
    command.add_option(cmd_init::FRESH_ATTR_KEY, "Initialize fresh config directory. Warning: This will replace your key and will not be able to open your encrypted files.");
    command
}
