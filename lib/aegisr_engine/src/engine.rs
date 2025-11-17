use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose};
use dirs_next::home_dir;
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use zeroize::Zeroize;

pub const RUNTIME_NAME: &str = "Aegisr";
pub const ENGINE_NAME: &str = "Aegisr Engine (Dusk)";
pub const ENGINE_DEVELOPER: &[&str] = &["surelle-ha"];
pub const ENGINE_VERSION: &str = "1.0.1-beta";
pub const STORE_DIR: &str = ".aegisr";
pub const STORE_COLLECTION: &str = "collection.lock";
pub const STORE_CONFIG_AEG: &str = "config.aeg";
pub const STORE_AUTHORIZATION_KEY: &str = "AUTHORIZATION_KEY";

// ===================== FILESYSTEM =====================
pub struct AegFileSystem;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectionLock {
    pub active: String,
    pub collections: Vec<String>,
}

impl AegFileSystem {
    pub fn get_config_path() -> PathBuf {
        let mut config_path = home_dir().expect("Failed to get home directory");
        config_path.push(STORE_DIR);
        if !config_path.exists() {
            fs::create_dir_all(&config_path).expect("Failed to create config directory");
        }
        config_path
    }

    /// Remove the whole config directory and recreate it (keeps it empty).
    pub fn reset_files() {
        let path = Self::get_config_path();
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to delete .aegisr configuration directory");
        }
        fs::create_dir_all(&path).expect("Failed to recreate config directory");
    }

    /// Ensure required files exist; if not, initialize them.
    pub fn validate_files() {
        let path = Self::get_config_path();
        let collection_lock: PathBuf = path.join(STORE_COLLECTION);
        let config_file = path.join(STORE_CONFIG_AEG);
        let auth_file = path.join(STORE_AUTHORIZATION_KEY);
        if !config_file.exists() || !auth_file.exists() || !collection_lock.exists() {
            println!("Missing file. Running initialize config.");
            Self::initialize_config(None, None);
        } else {
            // Attempt migration if collection.lock is not JSON (old format).
            if let Err(e) = Self::maybe_migrate_collection_lock() {
                // If migration fails, re-init to stable state.
                eprintln!("Collection lock migration failed: {}. Reinitializing.", e);
                Self::initialize_config(None, None);
            }
        }
    }

    /// Create or re-create configuration files. Returns config dir path.
    pub fn initialize_config(overwrite: Option<bool>, verbose_mode: Option<bool>) -> PathBuf {
        let overwrite_mode = overwrite.unwrap_or(false);
        let _verbose_mode = verbose_mode.unwrap_or(false);
        let dir = Self::get_config_path();

        if overwrite_mode && dir.exists() {
            fs::remove_dir_all(&dir).expect("Failed to remove existing config directory");
        }

        if !dir.exists() {
            fs::create_dir_all(&dir).expect("Failed to create config directory");
        }

        let key_path = dir.join(STORE_AUTHORIZATION_KEY);
        let auth_key = if key_path.exists() {
            fs::read_to_string(&key_path).expect("Failed to read AUTHORIZATION_KEY")
        } else {
            let k = AegCrypto::create_authorization_key(Some(_verbose_mode));
            fs::write(&key_path, &k).expect("Failed to write AUTHORIZATION_KEY");
            k
        };

        // Initialize default collection.lock (JSON format) if missing
        let collection_path = dir.join(STORE_COLLECTION);
        if !collection_path.exists() {
            Self::write_collection_lock_default(&auth_key);
        }

        dir
    }

    /// Write an arbitrary JSON string (plaintext JSON) encrypted into collection.lock
    pub fn write_collection_lock_json(data: &str, auth_key: &str) {
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .expect("Invalid base64 auth key");
        let key_arr: [u8; 32] = key_bytes
            .as_slice()
            .try_into()
            .expect("Authorization key must be 32 bytes after base64 decoding");

        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_arr[..12]);

        let encrypted = cipher
            .encrypt(nonce, data.as_bytes())
            .expect("Failed to encrypt collection lock");
        let encoded = general_purpose::STANDARD.encode(&encrypted);

        let path = Self::get_config_path().join(STORE_COLLECTION);

        // write and flush immediately
        let mut file = fs::File::create(&path).expect("Failed to open collection.lock for writing");
        use std::io::Write;
        file.write_all(encoded.as_bytes())
            .expect("Failed to write collection.lock");
        file.sync_all().expect("Failed to flush collection.lock");
    }

    /// Read collection.lock and return decrypted JSON string.
    /// Returns empty string if file missing or empty.
    pub fn read_collection_lock() -> String {
        let path = Self::get_config_path().join(STORE_COLLECTION);
        if !path.exists() {
            return String::new();
        }

        let auth_key = Self::read_authorization_key();
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .expect("Invalid auth key");

        let key_arr: [u8; 32] = key_bytes
            .as_slice()
            .try_into()
            .expect("Authorization key must be 32 bytes after base64 decoding");
        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce = Nonce::from_slice(&key_arr[..12]);

        let encrypted = fs::read_to_string(&path).unwrap_or_default();
        if encrypted.is_empty() {
            return String::new();
        }

        let encrypted_bytes = general_purpose::STANDARD
            .decode(encrypted)
            .expect("Invalid base64 in collection.lock");
        let decrypted = cipher
            .decrypt(nonce, encrypted_bytes.as_ref())
            .expect("Failed to decrypt collection.lock");
        String::from_utf8(decrypted).expect("Collection lock decrypted to invalid UTF-8")
    }

    /// Read collection lock and parse into CollectionLock. This function will perform
    /// migration if the file contains the old single-string format.
    pub fn read_collection_lock_obj() -> CollectionLock {
        // If file missing or empty, return default lock
        let json_str = Self::read_collection_lock();
        if json_str.trim().is_empty() {
            return CollectionLock {
                active: "default".to_string(),
                collections: vec!["default".to_string()],
            };
        }

        // Try to parse JSON object
        match serde_json::from_str::<CollectionLock>(&json_str) {
            Ok(lock) => lock,
            Err(_) => {
                // Attempt to treat json_str as an old plain string (unquoted or quoted)
                let s = json_str.trim().trim_matches('"').to_string();
                let lock = CollectionLock {
                    active: s.clone(),
                    collections: vec![s],
                };
                // migrate by overwriting collection.lock with the new JSON structure
                let auth_key = Self::read_authorization_key();
                let serialized =
                    serde_json::to_string_pretty(&lock).expect("Failed to serialize migrated lock");
                Self::write_collection_lock_json(&serialized, &auth_key);
                lock
            }
        }
    }

    /// Helper used by validate_files to migrate if necessary.
    fn maybe_migrate_collection_lock() -> Result<(), String> {
        let path = Self::get_config_path().join(STORE_COLLECTION);
        if !path.exists() {
            return Ok(());
        }

        // Attempt to decrypt and parse; read_collection_lock_obj already migrates on failure,
        // so just calling it is sufficient.
        let _ = Self::read_collection_lock_obj();
        Ok(())
    }

    pub fn write_collection_lock_default(auth_key: &str) {
        let lock = CollectionLock {
            active: "default".to_string(),
            collections: vec!["default".to_string()],
        };
        let serialized =
            serde_json::to_string_pretty(&lock).expect("Failed to serialize default lock");
        Self::write_collection_lock_json(&serialized, auth_key);
    }

    pub fn read_authorization_key() -> String {
        let path = Self::get_config_path().join(STORE_AUTHORIZATION_KEY);
        fs::read_to_string(&path).expect("Failed to read authorization key")
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
#[derive(Serialize, Deserialize, Debug)]
pub struct AegCore {
    pub active_collection: String,
    pub collections: Vec<String>,
}

impl AegCore {
    pub fn load() -> Self {
        let lock = AegFileSystem::read_collection_lock_obj();
        Self {
            active_collection: lock.active,
            collections: lock.collections,
        }
    }

    pub fn save(&self) {
        let lock = CollectionLock {
            active: self.active_collection.clone(),
            collections: self.collections.clone(),
        };
        let json =
            serde_json::to_string_pretty(&lock).expect("Failed to serialize collection lock");
        let auth_key = AegFileSystem::read_authorization_key();

        // Write JSON to disk first
        let path = AegFileSystem::get_config_path().join(STORE_COLLECTION);
        fs::write(&path, json.clone()).expect("Failed to write collection.lock");

        // Then encrypt
        AegFileSystem::write_collection_lock_json(&json, &auth_key);
    }

    pub fn get_active_collection(&self) -> &str {
        &self.active_collection
    }

    pub fn set_active_collection(&mut self, name: &str) -> Result<(), String> {
        if !self.collections.contains(&name.to_string()) {
            return Err(format!("Collection '{}' does not exist", name));
        }
        self.active_collection = name.to_string();
        self.save();
        Ok(())
    }

    pub fn create_collection(name: &str) -> String {
        // Load fresh
        let mut core = Self::load();
        if core.collections.contains(&name.to_string()) {
            return format!("✗ Collection '{}' already exists", name);
        }

        core.collections.push(name.to_string());
        core.save(); // persist

        // reload to ensure Use sees the new collection
        let _ = Self::load();

        format!("✓ Collection '{}' created", name)
    }

    pub fn delete_collection(name: &str) -> String {
        let mut core = Self::load();
        if core.collections.len() == 1 {
            return "✗ Cannot delete the last collection".into();
        }
        if let Some(pos) = core.collections.iter().position(|x| x == name) {
            core.collections.remove(pos);
            if core.active_collection == name {
                core.active_collection = core.collections[0].clone();
            }
            core.save();
            format!("✓ Collection '{}' deleted", name)
        } else {
            format!("✗ Collection '{}' does not exist", name)
        }
    }

    pub fn rename_collection(name: &str, new_name: &str) -> String {
        let mut core = Self::load();
        if core.collections.contains(&new_name.to_string()) {
            return format!("✗ Collection '{}' already exists", new_name);
        }
        if let Some(pos) = core.collections.iter().position(|x| x == name) {
            core.collections[pos] = new_name.to_string();
            if core.active_collection == name {
                core.active_collection = new_name.to_string();
            }
            core.save();
            format!("✓ Collection '{}' renamed to '{}'", name, new_name)
        } else {
            format!("✗ Collection '{}' does not exist", name)
        }
    }

    pub fn put_value(key: &str, value: &str) -> String {
        let mut engine = AegMemoryEngine::load();
        engine.insert(key, value);
        engine.save();
        format!("✓ Key '{}' saved in collection '{}'", key, engine.collection_name)
    }

    pub fn get_value(key: &str) -> Option<String> {
        let engine = AegMemoryEngine::load();
        engine.get(key)
    }

    pub fn delete_value(key: &str) -> String {
        let mut engine = AegMemoryEngine::load();
        if engine.get(key).is_some() {
            engine.delete(key);
            engine.save();
            format!("✓ Key '{}' deleted from collection '{}'", key, engine.collection_name)
        } else {
            format!("✗ Key '{}' not found in collection '{}'", key, engine.collection_name)
        }
    }

    pub fn clear_values() -> String {
        let mut engine = AegMemoryEngine::load();
        engine.clear();
        format!("✓ All keys cleared from collection '{}'", engine.collection_name)
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

// ===================== IN-MEMORY ENGINE =====================
// Simple in-memory key/value engine with JSON persistence to STORE_ENGINE_STATE.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AegMemoryEngine {
    pub store: HashMap<String, String>,
    pub collection_name: String,
}

impl AegMemoryEngine {
    /// Create a new empty engine for a specific collection
    pub fn new(collection_name: &str) -> Self {
        Self {
            store: HashMap::new(),
            collection_name: collection_name.to_string(),
        }
    }

    /// Path to engine_state file, including collection name
    fn engine_file_path(collection_name: &str) -> PathBuf {
        let mut path = AegFileSystem::get_config_path();
        path.push(format!("collection_{}.aekv", collection_name));
        path
    }

    /// Insert or update a key
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.store.insert(key.into(), value.into());
    }

    /// Get value by key (cloned)
    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).cloned()
    }

    /// Delete a key
    pub fn delete(&mut self, key: &str) {
        self.store.remove(key);
    }

    /// List all key/value pairs
    pub fn list(&self) -> Vec<(String, String)> {
        self.store.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Save current engine state to disk (JSON) encrypted
    pub fn save(&self) {
        let path = Self::engine_file_path(&self.collection_name);
        let json = serde_json::to_string_pretty(&self).expect("Failed to serialize engine state");

        // Encrypt with authorization key
        let auth_key = AegFileSystem::read_authorization_key();
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .expect("Invalid base64 auth key");
        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_bytes[..12]);

        let encrypted = cipher.encrypt(nonce, json.as_bytes())
            .expect("Failed to encrypt engine state");
        let encoded = general_purpose::STANDARD.encode(&encrypted);

        fs::write(&path, encoded).expect("Failed to write engine state file");
    }

    /// Load engine state from disk. If missing or invalid, returns new engine.
    pub fn load() -> Self {
        let core = AegCore::load();
        let collection_name = core.active_collection.clone();
        let path = Self::engine_file_path(&collection_name);

        if path.exists() {
            let encrypted = fs::read_to_string(&path).unwrap_or_default();
            if encrypted.trim().is_empty() {
                return Self::new(&collection_name);
            }

            let auth_key = AegFileSystem::read_authorization_key();
            let key_bytes = general_purpose::STANDARD
                .decode(auth_key)
                .expect("Invalid base64 auth key");
            let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);
            let nonce = Nonce::from_slice(&key_bytes[..12]);

            if let Ok(encrypted_bytes) = general_purpose::STANDARD.decode(encrypted) {
                if let Ok(decrypted) = cipher.decrypt(nonce, encrypted_bytes.as_ref()) {
                    if let Ok(engine) = serde_json::from_slice::<AegMemoryEngine>(&decrypted) {
                        return engine;
                    }
                }
            }
        }

        Self::new(&collection_name)
    }

    /// Clear the engine and persist an empty state
    pub fn clear(&mut self) {
        self.store.clear();
        self.save();
    }
}

// Small convenience wrapper for common operations used by the daemon. This keeps
// API calls short and mirrors the pattern used in earlier Aeg* structs.
impl Default for AegMemoryEngine {
    fn default() -> Self {
        Self::load()
    }
}

// Example helpers that tie collections to engine filenames could be added here.
// For example, if you want per-collection engine files, you could change
// engine_file_path to include the active collection name from AegCore.

