// Key management for Tonitru encryption
//
// This module provides key management capabilities for encryption algorithms.
// It supports key generation, storage, rotation, and integration with external
// key management systems.

use crate::internal::error::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use rand_core::{OsRng, RngCore};
use aes_gcm::aead::KeyInit;
use chacha20poly1305::ChaCha20Poly1305;
use x25519_dalek::{StaticSecret, PublicKey};

/// Key types supported by the key manager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyType {
    /// AES-GCM key
    AesGcm,
    /// ChaCha20-Poly1305 key
    ChaCha20Poly1305,
    /// X25519 key pair
    X25519,
    /// Kyber768 key pair
    Kyber768,
}

/// Key metadata
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    /// Key ID
    pub id: String,
    /// Key type
    pub key_type: KeyType,
    /// Creation time
    pub created_at: SystemTime,
    /// Expiration time (if any)
    pub expires_at: Option<SystemTime>,
    /// Whether the key is the primary key for its type
    pub is_primary: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Key material (sensitive)
#[derive(Debug)]
enum KeyMaterial {
    /// AES-GCM key
    AesGcm([u8; 32]),
    /// ChaCha20-Poly1305 key
    ChaCha20Poly1305([u8; 32]),
    /// X25519 key pair (private, public)
    X25519(StaticSecret, PublicKey),
    /// Kyber768 key pair (public key, secret key)
    Kyber768([u8; 1184], [u8; 2400]),
}

/// A key entry in the key manager
#[derive(Debug)]
struct KeyEntry {
    /// Key metadata
    metadata: KeyMetadata,
    /// Key material
    material: KeyMaterial,
}

/// Key rotation policy
#[derive(Debug, Clone)]
pub struct KeyRotationPolicy {
    /// Key type
    pub key_type: KeyType,
    /// Key lifetime
    pub lifetime: Duration,
    /// Whether to keep old keys after rotation
    pub keep_old_keys: bool,
    /// Number of old keys to keep (if keep_old_keys is true)
    pub old_keys_to_keep: usize,
}

/// Key manager
///
/// Manages encryption keys for various algorithms, including generation,
/// storage, rotation, and integration with external key management systems.
#[derive(Debug)]
pub struct KeyManager {
    /// Keys by ID
    keys: Arc<RwLock<HashMap<String, KeyEntry>>>,
    /// Primary keys by type
    primary_keys: Arc<RwLock<HashMap<KeyType, String>>>,
    /// Key rotation policies by type
    rotation_policies: Arc<RwLock<HashMap<KeyType, KeyRotationPolicy>>>,
    /// External key provider (if any)
    external_provider: Option<Box<dyn ExternalKeyProvider>>,
}

/// Trait for external key providers
pub trait ExternalKeyProvider: Send + Sync + std::fmt::Debug {
    /// Gets a key from the external provider
    fn get_key(&self, key_id: &str, key_type: KeyType) -> Result<Vec<u8>>;
    /// Stores a key in the external provider
    fn store_key(&self, key_id: &str, key_type: KeyType, key_data: &[u8]) -> Result<()>;
    /// Lists keys in the external provider
    fn list_keys(&self, key_type: Option<KeyType>) -> Result<Vec<String>>;
    /// Deletes a key from the external provider
    fn delete_key(&self, key_id: &str) -> Result<()>;
}

impl KeyManager {
    /// Creates a new KeyManager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            primary_keys: Arc::new(RwLock::new(HashMap::new())),
            rotation_policies: Arc::new(RwLock::new(HashMap::new())),
            external_provider: None,
        }
    }
    
    /// Sets an external key provider
    pub fn set_external_provider(&mut self, provider: Box<dyn ExternalKeyProvider>) {
        self.external_provider = Some(provider);
    }
    
    /// Sets a key rotation policy
    pub fn set_rotation_policy(&self, policy: KeyRotationPolicy) -> Result<()> {
        let mut policies = self.rotation_policies.write().map_err(|_| {
            Error::EncryptionError("Failed to acquire write lock on rotation policies".to_string())
        })?;
        
        policies.insert(policy.key_type, policy);
        
        Ok(())
    }
    
    /// Generates a new key
    pub fn generate_key(&self, key_type: KeyType, make_primary: bool) -> Result<String> {
        // Generate a random key ID
        let key_id = self.generate_key_id();
        
        // Generate key material based on type
        let material = match key_type {
            KeyType::AesGcm => {
                let key = aes_gcm::Aes256Gcm::generate_key(&mut OsRng);
                KeyMaterial::AesGcm(*key.as_slice().try_into().unwrap())
            }
            KeyType::ChaCha20Poly1305 => {
                let key = ChaCha20Poly1305::generate_key(&mut OsRng);
                KeyMaterial::ChaCha20Poly1305(*key.as_slice().try_into().unwrap())
            }
            KeyType::X25519 => {
                let private_key = StaticSecret::new(OsRng);
                let public_key = PublicKey::from(&private_key);
                KeyMaterial::X25519(private_key, public_key)
            }
            KeyType::Kyber768 => {
                let (public_key, secret_key) = kyber_rust::kyber768::keypair();
                KeyMaterial::Kyber768(public_key, secret_key)
            }
        };
        
        // Create key metadata
        let metadata = KeyMetadata {
            id: key_id.clone(),
            key_type,
            created_at: SystemTime::now(),
            expires_at: self.get_expiration_time(key_type),
            is_primary: make_primary,
            metadata: HashMap::new(),
        };
        
        // Create key entry
        let entry = KeyEntry {
            metadata: metadata.clone(),
            material,
        };
        
        // Store the key
        let mut keys = self.keys.write().map_err(|_| {
            Error::EncryptionError("Failed to acquire write lock on keys".to_string())
        })?;
        
        keys.insert(key_id.clone(), entry);
        
        // Update primary key if needed
        if make_primary {
            let mut primary_keys = self.primary_keys.write().map_err(|_| {
                Error::EncryptionError("Failed to acquire write lock on primary keys".to_string())
            })?;
            
            // If there was a previous primary key, update its is_primary flag
            if let Some(old_primary_id) = primary_keys.get(&key_type) {
                if let Some(old_entry) = keys.get_mut(old_primary_id) {
                    old_entry.metadata.is_primary = false;
                }
            }
            
            primary_keys.insert(key_type, key_id.clone());
        }
        
        // Store in external provider if available
        if let Some(provider) = &self.external_provider {
            match key_type {
                KeyType::AesGcm => {
                    if let KeyMaterial::AesGcm(key_data) = &material {
                        provider.store_key(&key_id, key_type, key_data)?;
                    }
                }
                KeyType::ChaCha20Poly1305 => {
                    if let KeyMaterial::ChaCha20Poly1305(key_data) = &material {
                        provider.store_key(&key_id, key_type, key_data)?;
                    }
                }
                KeyType::X25519 => {
                    if let KeyMaterial::X25519(private_key, _) = &material {
                        let private_bytes = private_key.to_bytes();
                        provider.store_key(&key_id, key_type, &private_bytes)?;
                    }
                }
                KeyType::Kyber768 => {
                    if let KeyMaterial::Kyber768(_, secret_key) = &material {
                        provider.store_key(&key_id, key_type, secret_key)?;
                    }
                }
            }
        }
        
        Ok(key_id)
    }
    
    /// Gets a key by ID
    pub fn get_key(&self, key_id: &str) -> Result<KeyMetadata> {
        // Try to get from local cache first
        let keys = self.keys.read().map_err(|_| {
            Error::EncryptionError("Failed to acquire read lock on keys".to_string())
        })?;
        
        if let Some(entry) = keys.get(key_id) {
            return Ok(entry.metadata.clone());
        }
        
        // If not found and we have an external provider, try to get from there
        if let Some(provider) = &self.external_provider {
            // We need to know the key type to fetch from external provider
            // This is a limitation of this implementation
            return Err(Error::EncryptionError(format!(
                "Key ID '{}' not found in local cache and cannot determine key type for external fetch",
                key_id
            )));
        }
        
        Err(Error::EncryptionError(format!("Key ID '{}' not found", key_id)))
    }
    
    /// Gets the primary key for a key type
    pub fn get_primary_key(&self, key_type: KeyType) -> Result<KeyMetadata> {
        let primary_keys = self.primary_keys.read().map_err(|_| {
            Error::EncryptionError("Failed to acquire read lock on primary keys".to_string())
        })?;
        
        if let Some(key_id) = primary_keys.get(&key_type) {
            return self.get_key(key_id);
        }
        
        Err(Error::EncryptionError(format!(
            "No primary key found for key type {:?}",
            key_type
        )))
    }
    
    /// Rotates keys according to the rotation policy
    pub fn rotate_keys(&self) -> Result<()> {
        let policies = self.rotation_policies.read().map_err(|_| {
            Error::EncryptionError("Failed to acquire read lock on rotation policies".to_string())
        })?;
        
        for (key_type, policy) in policies.iter() {
            self.rotate_key_type(*key_type, policy)?;
        }
        
        Ok(())
    }
    
    /// Rotates keys for a specific key type
    fn rotate_key_type(&self, key_type: KeyType, policy: &KeyRotationPolicy) -> Result<()> {
        // Generate a new primary key
        let new_key_id = self.generate_key(key_type, true)?;
        
        // If we don't keep old keys, delete them
        if !policy.keep_old_keys {
            let mut keys = self.keys.write().map_err(|_| {
                Error::EncryptionError("Failed to acquire write lock on keys".to_string())
            })?;
            
            // Collect keys to remove
            let keys_to_remove: Vec<String> = keys
                .iter()
                .filter(|(id, entry)| {
                    entry.metadata.key_type == key_type && **id != new_key_id
                })
                .map(|(id, _)| id.clone())
                .collect();
            
            // Remove keys
            for id in keys_to_remove {
                keys.remove(&id);
                
                // Remove from external provider if available
                if let Some(provider) = &self.external_provider {
                    let _ = provider.delete_key(&id); // Ignore errors
                }
            }
        } else if policy.old_keys_to_keep > 0 {
            // Keep only the specified number of old keys
            let mut keys = self.keys.write().map_err(|_| {
                Error::EncryptionError("Failed to acquire write lock on keys".to_string())
            })?;
            
            // Collect keys of this type
            let mut type_keys: Vec<(String, SystemTime)> = keys
                .iter()
                .filter(|(id, entry)| {
                    entry.metadata.key_type == key_type && **id != new_key_id
                })
                .map(|(id, entry)| (id.clone(), entry.metadata.created_at))
                .collect();
            
            // Sort by creation time (newest first)
            type_keys.sort_by(|a, b| b.1.cmp(&a.1));
            
            // Remove excess keys
            if type_keys.len() > policy.old_keys_to_keep {
                for (id, _) in type_keys.iter().skip(policy.old_keys_to_keep) {
                    keys.remove(id);
                    
                    // Remove from external provider if available
                    if let Some(provider) = &self.external_provider {
                        let _ = provider.delete_key(id); // Ignore errors
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Generates a random key ID
    fn generate_key_id(&self) -> String {
        let mut bytes = [0u8; 16];
        OsRng.fill_bytes(&mut bytes);
        
        hex::encode(bytes)
    }
    
    /// Gets the expiration time for a key type based on rotation policy
    fn get_expiration_time(&self, key_type: KeyType) -> Option<SystemTime> {
        let policies = match self.rotation_policies.read() {
            Ok(p) => p,
            Err(_) => return None,
        };
        
        if let Some(policy) = policies.get(&key_type) {
            return Some(SystemTime::now() + policy.lifetime);
        }
        
        None
    }
}
