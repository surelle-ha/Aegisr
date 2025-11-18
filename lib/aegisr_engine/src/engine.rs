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
use std::time::{SystemTime, Duration};
use zeroize::Zeroize;
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::sleep;

pub const RUNTIME_NAME: &str = "Aegisr";
pub const ENGINE_NAME: &str = "Aegisr Engine (Dusk)";
pub const ENGINE_DEVELOPER: &[&str] = &["surelle-ha"];
pub const ENGINE_VERSION: &str = "1.0.2-beta";
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

    pub fn reset_files() {
        let path = Self::get_config_path();
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to delete .aegisr configuration directory");
        }
        fs::create_dir_all(&path).expect("Failed to recreate config directory");
    }

    pub fn validate_files() {
        let path = Self::get_config_path();
        let collection_lock: PathBuf = path.join(STORE_COLLECTION);
        let config_file = path.join(STORE_CONFIG_AEG);
        let auth_file = path.join(STORE_AUTHORIZATION_KEY);
        if !config_file.exists() || !auth_file.exists() || !collection_lock.exists() {
            println!("Missing file. Running initialize config.");
            Self::initialize_config(None, None);
        } else {
            if let Err(e) = Self::maybe_migrate_collection_lock() {
                eprintln!("Migration failed: {}. Reinitializing.", e);
                Self::initialize_config(None, None);
            }
        }
    }

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

        let collection_path = dir.join(STORE_COLLECTION);
        if !collection_path.exists() {
            Self::write_collection_lock_default(&auth_key);
        }

        dir
    }

    pub fn write_collection_lock_json(data: &str, auth_key: &str) {
        let key_bytes = general_purpose::STANDARD.decode(auth_key).expect("Invalid base64");
        let key_arr: [u8; 32] =
            key_bytes.as_slice().try_into().expect("Auth key must be 32 bytes");
        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_arr[..12]);

        let encrypted = cipher.encrypt(nonce, data.as_bytes()).expect("Encrypt failed");
        let encoded = general_purpose::STANDARD.encode(&encrypted);

        let path = Self::get_config_path().join(STORE_COLLECTION);
        let mut file = fs::File::create(&path).expect("Failed to open file");
        use std::io::Write;
        file.write_all(encoded.as_bytes()).expect("Write failed");
        file.sync_all().expect("Flush failed");
    }

    pub fn read_collection_lock() -> String {
        let path = Self::get_config_path().join(STORE_COLLECTION);
        if !path.exists() {
            return String::new();
        }

        let auth_key = Self::read_authorization_key();
        let key_bytes = general_purpose::STANDARD.decode(auth_key).expect("Invalid auth key");

        let key_arr: [u8; 32] =
            key_bytes.as_slice().try_into().expect("Auth key must be 32 bytes");
        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_arr[..12]);

        let encrypted = fs::read_to_string(&path).unwrap_or_default();
        if encrypted.is_empty() {
            return String::new();
        }

        let encrypted_bytes =
            general_purpose::STANDARD.decode(encrypted).expect("Invalid base64 content");

        let decrypted = cipher
            .decrypt(nonce, encrypted_bytes.as_ref())
            .expect("Decrypt failed");

        String::from_utf8(decrypted).expect("Invalid UTF-8")
    }

    pub fn read_collection_lock_obj() -> CollectionLock {
        let json_str = Self::read_collection_lock();
        if json_str.trim().is_empty() {
            return CollectionLock {
                active: "default".to_string(),
                collections: vec!["default".to_string()],
            };
        }

        match serde_json::from_str::<CollectionLock>(&json_str) {
            Ok(lock) => lock,
            Err(_) => {
                let s = json_str.trim().trim_matches('"').to_string();
                let lock = CollectionLock {
                    active: s.clone(),
                    collections: vec![s],
                };

                let auth_key = Self::read_authorization_key();
                let serialized =
                    serde_json::to_string_pretty(&lock).expect("Serialize failed");
                Self::write_collection_lock_json(&serialized, &auth_key);
                lock
            }
        }
    }

    fn maybe_migrate_collection_lock() -> Result<(), String> {
        let _ = Self::read_collection_lock_obj();
        Ok(())
    }

    pub fn write_collection_lock_default(auth_key: &str) {
        let lock = CollectionLock {
            active: "default".to_string(),
            collections: vec!["default".to_string()],
        };
        let serialized =
            serde_json::to_string_pretty(&lock).expect("Serialize failed");
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
            serde_json::to_string_pretty(&lock).expect("Serialize failed");
        let auth_key = AegFileSystem::read_authorization_key();

        let path = AegFileSystem::get_config_path().join(STORE_COLLECTION);
        fs::write(&path, json.clone()).expect("Write failed");

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
        let mut core = Self::load();
        if core.collections.contains(&name.to_string()) {
            return format!("✗ Collection '{}' already exists", name);
        }

        core.collections.push(name.to_string());
        core.save();

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

    /// Insert into memory (non-blocking). Does not perform immediate disk save.
    /// Background saver (if started) will persist this later.
    pub fn put_value(key: &str, value: &str) -> String {
        let mut engine = AegMemoryEngine::load();
        engine.insert(key, value);
        // no engine.save() here - background saver will persist
        format!(
            "✓ Key '{}' saved in collection '{}' (in-memory)",
            key, engine.collection_name
        )
    }

    /// Read from memory (plaintext in RAM).
    pub fn get_value(key: &str) -> Option<String> {
        let engine = AegMemoryEngine::load();
        engine.get(key)
    }

    /// Delete in-memory (non-blocking). Background saver will persist deletion later.
    pub fn delete_value(key: &str) -> String {
        let mut engine = AegMemoryEngine::load();
        if engine.get(key).is_some() {
            engine.delete(key);
            // no engine.save() here
            format!(
                "✓ Key '{}' deleted from collection '{}' (in-memory)",
                key, engine.collection_name
            )
        } else {
            format!(
                "✗ Key '{}' not found in collection '{}' (in-memory)",
                key, engine.collection_name
            )
        }
    }

    /// Clear in-memory values (non-blocking). Background saver will persist later.
    pub fn clear_values() -> String {
        let mut engine = AegMemoryEngine::load();
        engine.clear();
        format!(
            "✓ All keys cleared from collection '{}' (in-memory)",
            engine.collection_name
        )
    }

    /// Force immediate flush (saves all collections to disk synchronously).
    pub fn flush_now() {
        AegMemoryEngine::save_all();
    }

    /// Start background saver thread. Safe to call multiple times.
    /// interval_seconds: how often to persist (e.g. 1).
    pub fn start_background_saver(interval_seconds: u64) {
        AegMemoryEngine::start_background_saver(interval_seconds);
    }

    /// Signal background saver to stop. Returns immediately.
    pub fn stop_background_saver() {
        AegMemoryEngine::stop_background_saver();
    }
}

// ===================== IN-MEMORY ENGINE =====================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AegMemoryEngine {
    pub store: HashMap<String, String>,
    pub collection_name: String,
}

/// SAFE GLOBAL IN-MEMORY CACHE (OnceLock + Mutex)
static MEMORY_CACHE: OnceLock<Mutex<HashMap<String, AegMemoryEngine>>> = OnceLock::new();

/// Background saver control
static SAVER_RUNNING: OnceLock<AtomicBool> = OnceLock::new();
static SAVER_STARTED: OnceLock<AtomicBool> = OnceLock::new();

impl AegMemoryEngine {
    /// Returns a reference to the global Mutex<HashMap<...>>.
    fn global_memory_mutex() -> &'static Mutex<HashMap<String, AegMemoryEngine>> {
        MEMORY_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn new(collection_name: &str) -> Self {
        Self {
            store: HashMap::new(),
            collection_name: collection_name.to_string(),
        }
    }

    fn engine_file_path(collection_name: &str) -> PathBuf {
        let mut path = AegFileSystem::get_config_path();
        path.push(format!("collection_{}.aekv", collection_name));
        path
    }

    /// Insert into current engine and update global in-memory cache (fast).
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.store.insert(key.into(), value.into());
        // persist to global in-memory cache (only memory)
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(self.collection_name.clone(), self.clone());
        // intentionally not calling self.save() here
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.store.get(key).cloned()
    }

    pub fn delete(&mut self, key: &str) {
        self.store.remove(key);
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(self.collection_name.clone(), self.clone());
        // intentionally not calling self.save()
    }

    pub fn list(&self) -> Vec<(String, String)> {
        self.store
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.store.clear();
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(self.collection_name.clone(), self.clone());
    }

    /// Persist single engine to disk (synchronous) — same encryption as before.
    pub fn save_to_disk(engine: &AegMemoryEngine) -> Result<(), String> {
        let path = Self::engine_file_path(&engine.collection_name);

        let json = serde_json::to_string_pretty(engine)
            .map_err(|e| format!("serialize error: {}", e))?;

        let auth_key = AegFileSystem::read_authorization_key();
        let key_bytes = general_purpose::STANDARD
            .decode(auth_key)
            .map_err(|e| format!("base64 decode auth key: {}", e))?;

        let key: &aes_gcm::Key<Aes256Gcm> = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&key_bytes[..12]);

        let encrypted = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("encrypt error: {:?}", e))?;

        let encoded = general_purpose::STANDARD.encode(&encrypted);

        fs::write(&path, encoded).map_err(|e| format!("write error: {}", e))?;

        Ok(())
    }

    /// Save ALL collections currently in memory to disk.
    /// This function clones the cache under the mutex and performs expensive work outside the lock.
    pub fn save_all() {
        // 1) Clone the memory map under the lock (minimize lock time)
        let snapshot: HashMap<String, AegMemoryEngine> = {
            let mutex = Self::global_memory_mutex();
            let guard = mutex.lock().expect("Failed to lock global memory mutex");
            guard.clone()
        };

        // 2) For each collection, perform serialization/encryption/write outside the lock
        for (_name, engine) in snapshot.into_iter() {
            // best-effort: log errors but continue
            if let Err(e) = Self::save_to_disk(&engine) {
                eprintln!("Failed to save collection '{}': {}", engine.collection_name, e);
            }
        }
    }

    /// Load engine from memory cache; otherwise load from disk; otherwise fresh engine.
    pub fn load() -> Self {
        let core = AegCore::load();
        let collection_name = core.active_collection.clone();

        // First try in-memory (global cache)
        {
            let mutex = Self::global_memory_mutex();
            let guard = mutex.lock().expect("Failed to lock global memory mutex");
            if let Some(engine) = guard.get(&collection_name).cloned() {
                return engine;
            }
        }

        // If not in memory, load from disk
        let path = Self::engine_file_path(&collection_name);

        if path.exists() {
            let encrypted = fs::read_to_string(&path).unwrap_or_default();
            if encrypted.trim().is_empty() {
                let engine = Self::new(&collection_name);
                // store in memory
                let mutex = Self::global_memory_mutex();
                let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
                guard.insert(collection_name.clone(), engine.clone());
                return engine;
            }

            let auth_key = AegFileSystem::read_authorization_key();
            let key_bytes =
                general_purpose::STANDARD.decode(auth_key).expect("Invalid base64");

            let key: &aes_gcm::Key<Aes256Gcm> =
                aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
            let cipher = Aes256Gcm::new(key);

            let nonce = Nonce::from_slice(&key_bytes[..12]);

            let decoded =
                general_purpose::STANDARD.decode(encrypted).expect("Invalid base64");

            let decrypted = cipher
                .decrypt(nonce, decoded.as_ref())
                .expect("Decrypt failed");

            let engine: AegMemoryEngine =
                serde_json::from_slice(&decrypted).unwrap_or(Self::new(&collection_name));

            // Store to in-memory cache
            let mutex = Self::global_memory_mutex();
            let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
            guard.insert(collection_name.clone(), engine.clone());

            return engine;
        }

        // Fresh engine
        let engine = Self::new(&collection_name);
        let mutex = Self::global_memory_mutex();
        let mut guard = mutex.lock().expect("Failed to lock global memory mutex");
        guard.insert(collection_name.clone(), engine.clone());
        engine
    }

    /// Start a background thread to periodically save memory to disk.
    /// If already started, this is a no-op.
    pub fn start_background_saver(interval_seconds: u64) {
        // initialize the running flag (if not already)
        let running = SAVER_RUNNING.get_or_init(|| AtomicBool::new(false));
        let started_flag = SAVER_STARTED.get_or_init(|| AtomicBool::new(false));

        // if already started, do nothing
        if started_flag.load(Ordering::SeqCst) {
            return;
        }

        // mark running
        running.store(true, Ordering::SeqCst);
        // mark started
        started_flag.store(true, Ordering::SeqCst);

        // spawn detached thread
        let running_ref: &'static AtomicBool = running;
        thread::spawn(move || {
            let interval = Duration::from_secs(interval_seconds.max(1));
            while running_ref.load(Ordering::SeqCst) {
                // save snapshot
                Self::save_all();
                // sleep for interval (cooperative)
                sleep(interval);
            }
            // final flush on exit attempt
            Self::save_all();
        });
    }

    /// Signal the background saver to stop. Thread is detached so we can't join; this just signals termination.
    pub fn stop_background_saver() {
        if let Some(running) = SAVER_RUNNING.get() {
            running.store(false, Ordering::SeqCst);
        }
        if let Some(started) = SAVER_STARTED.get() {
            started.store(false, Ordering::SeqCst);
        }
    }
}

// ===================== USAGE GUIDE =====================
//
// During startup:
// AegFileSystem::initialize_config(None, None);   // prepares configuration files
// AegCore::start_background_saver(1);             // enables automatic persistence (1-second interval)
//
// Normal operations use:
// AegCore::put_value(...);
// AegCore::get_value(...);
//
// For an immediate write to disk:
// AegCore::flush_now();
//
// At application shutdown:
// AegCore::stop_background_saver();               // stops the background thread
// AegCore::flush_now();                           // optional final save
