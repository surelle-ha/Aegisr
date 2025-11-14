use base64::{Engine as _, engine::general_purpose};
use dirs_next::home_dir;
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use zeroize::Zeroize;

pub const RUNTIME_NAME: &str = "Aegisr";
pub const ENGINE_NAME: &str = "Aegisr Engine (Dusk)";
pub const ENGINE_DEVELOPER: &[&str] = &["surelle-ha"];
pub const ENGINE_VERSION: &str = "1.0.0-beta";
pub const STORE_DIR: &str = ".aegisr";
pub const STORE_CONFIG_AEG: &str = "config.aeg";
pub const STORE_ENGINE_STATE: &str = "engine_state.json";
pub const STORE_AUTHORIZATION_KEY: &str = "AUTHORIZATION_KEY";

// ===================== FILESYSTEM =====================
pub struct AegFileSystem;

impl AegFileSystem {
    pub fn get_config_path() -> PathBuf {
        let mut config_path = home_dir().expect("Failed to get home directory");
        config_path.push(STORE_DIR);
        if !config_path.exists() {
            fs::create_dir_all(&config_path).expect("Failed to create config directory");
        }
        config_path
    }

    pub fn reset_files() {
        let path = Self::get_config_path();
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to delete .aegisr configuration directory");
        }
        fs::create_dir_all(&path).expect("Failed to recreate config directory");
    }

    pub fn validate_files() {
        let path = Self::get_config_path();
        let config_file = path.join(STORE_CONFIG_AEG);
        let auth_file = path.join(STORE_AUTHORIZATION_KEY);
        if !config_file.exists() || !auth_file.exists() {
            Self::initialize_config(None, None);
        }
    }

    pub fn initialize_config(overwrite: Option<bool>, verbose_mode: Option<bool>) -> PathBuf {
        let overwrite_mode = overwrite.unwrap_or(false);
        let verbose_mode = verbose_mode.unwrap_or(false);
        let dir = Self::get_config_path();

        if overwrite_mode && dir.exists() {
            fs::remove_dir_all(&dir).expect("Failed to remove existing config directory");
        }

        if !dir.exists() {
            fs::create_dir_all(&dir).expect("Failed to create config directory");
        }

        let auth_key = AegCrypto::create_authorization_key(Some(verbose_mode));
        let key_path = dir.join(STORE_AUTHORIZATION_KEY);
        fs::write(&key_path, auth_key.as_bytes()).expect("Failed to write AUTHORIZATION_KEY");

        dir
    }
}

// ===================== CRYPTO =====================
pub struct AegCrypto;

impl AegCrypto {
    pub fn generate_random_bytes(_verbose: Option<bool>) -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.try_fill_bytes(&mut key).unwrap();
        key
    }

    pub fn encode_base64(input: impl AsRef<[u8]>, _verbose: Option<bool>) -> String {
        general_purpose::STANDARD.encode(input.as_ref())
    }

    pub fn create_authorization_key(_verbose: Option<bool>) -> String {
        let mut bytes = Self::generate_random_bytes(None);
        let hash = blake3::hash(&bytes);
        bytes.zeroize();
        Self::encode_base64(hash.as_bytes(), None)
    }
}

// ===================== ENGINE CORE =====================
#[derive(Serialize, Deserialize)]
pub struct AegCore {
    pub active_collection: String,
}

impl AegCore {
    pub fn config_path() -> PathBuf {
        let mut path = AegFileSystem::get_config_path();
        path.push(STORE_ENGINE_STATE);
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let data = fs::read_to_string(&path).expect("Failed to read engine state file");
            serde_json::from_str(&data).expect("Failed to parse engine state")
        } else {
            Self {
                active_collection: "default".to_string(),
            }
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        let data = serde_json::to_string_pretty(&self).expect("Failed to serialize engine state");
        fs::write(&path, data).expect("Failed to write engine state file");
    }

    pub fn set_active_collection(&mut self, name: &str) {
        self.active_collection = name.to_string();
    }

    pub fn get_active_collection(&self) -> &str {
        &self.active_collection
    }
}

// ===================== PERSISTENT CACHE =====================
#[derive(Serialize, Deserialize, Debug)]
pub struct CacheEntry {
    pub value: String,
    pub expires_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AegCache {
    pub store: HashMap<String, CacheEntry>,
}

impl AegCache {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    fn cache_file_path() -> PathBuf {
        let mut path = AegFileSystem::get_config_path();
        path.push("cache.json");
        path
    }

    pub fn set(&mut self, key: &str, value: &str, ttl_seconds: Option<u64>) {
        let expires_at = ttl_seconds.map(|ttl| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + ttl
        });
        self.store.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                expires_at,
            },
        );
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(entry) = self.store.get(key) {
            if let Some(expiry) = entry.expires_at {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now > expiry {
                    self.store.remove(key);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub fn save(&self) {
        let path = Self::cache_file_path();
        let data = serde_json::to_string_pretty(&self).expect("Failed to serialize cache");
        fs::write(&path, data).expect("Failed to write cache file");
    }

    pub fn load() -> Self {
        let path = Self::cache_file_path();
        if path.exists() {
            let data = fs::read_to_string(&path).expect("Failed to read cache file");
            serde_json::from_str(&data).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }
}
