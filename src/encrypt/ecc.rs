// ECC (Elliptic Curve Cryptography) implementation for Tonitru
//
// This module provides ECC-based key exchange and encryption functionality.
// It supports Curve25519 and NIST P-256 curves.

use crate::internal::error::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};
use rand_core::OsRng;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use chacha20poly1305::{ChaCha20Poly1305};
use sha2::{Sha256, Digest};

/// The length of the X25519 public key in bytes
const X25519_PUBLIC_KEY_SIZE: usize = 32;

/// The length of the X25519 private key in bytes
const X25519_PRIVATE_KEY_SIZE: usize = 32;

/// The length of the AES-GCM key in bytes (256 bits)
const AES_KEY_SIZE: usize = 32;

/// The length of the nonce in bytes
const NONCE_SIZE: usize = 12;

/// Supported ECC curves
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EccCurve {
    /// Curve25519 (X25519)
    Curve25519,
    /// NIST P-256
    P256,
}

/// Supported symmetric encryption algorithms for use with ECC
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymmetricAlgorithm {
    /// AES-GCM
    AesGcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

/// ECC encryptor implementation
///
/// This encryptor uses ECC for key exchange and a symmetric algorithm for data encryption.
#[derive(Debug)]
pub struct EccEncryptor {
    // Default keypair used when no key_id is provided
    default_private_key: StaticSecret,
    default_public_key: PublicKey,
    // Cache of keypairs for different key_ids
    keypair_cache: Arc<Mutex<HashMap<String, (StaticSecret, PublicKey)>>>,
    // Symmetric algorithm to use
    symmetric_algorithm: SymmetricAlgorithm,
}

impl EccEncryptor {
    /// Creates a new EccEncryptor with a randomly generated default keypair.
    pub fn new(symmetric_algorithm: SymmetricAlgorithm) -> Result<Self> {
        let default_private_key = StaticSecret::new(OsRng);
        let default_public_key = PublicKey::from(&default_private_key);
        
        Ok(Self {
            default_private_key,
            default_public_key,
            keypair_cache: Arc::new(Mutex::new(HashMap::new())),
            symmetric_algorithm,
        })
    }
    
    /// Creates a new EccEncryptor with the provided keypair.
    pub fn with_keypair(
        private_key_bytes: &[u8],
        symmetric_algorithm: SymmetricAlgorithm,
    ) -> Result<Self> {
        if private_key_bytes.len() != X25519_PRIVATE_KEY_SIZE {
            return Err(Error::EncryptionError(format!(
                "Invalid X25519 private key size: expected {} bytes, got {} bytes",
                X25519_PRIVATE_KEY_SIZE,
                private_key_bytes.len()
            )));
        }
        
        let mut private_key_array = [0u8; X25519_PRIVATE_KEY_SIZE];
        private_key_array.copy_from_slice(private_key_bytes);
        
        let default_private_key = StaticSecret::from(private_key_array);
        let default_public_key = PublicKey::from(&default_private_key);
        
        Ok(Self {
            default_private_key,
            default_public_key,
            keypair_cache: Arc::new(Mutex::new(HashMap::new())),
            symmetric_algorithm,
        })
    }
    
    /// Adds a keypair to the cache.
    pub fn add_keypair(&self, key_id: &str, private_key_bytes: &[u8]) -> Result<()> {
        if private_key_bytes.len() != X25519_PRIVATE_KEY_SIZE {
            return Err(Error::EncryptionError(format!(
                "Invalid X25519 private key size: expected {} bytes, got {} bytes",
                X25519_PRIVATE_KEY_SIZE,
                private_key_bytes.len()
            )));
        }
        
        let mut private_key_array = [0u8; X25519_PRIVATE_KEY_SIZE];
        private_key_array.copy_from_slice(private_key_bytes);
        
        let private_key = StaticSecret::from(private_key_array);
        let public_key = PublicKey::from(&private_key);
        
        let mut cache = self.keypair_cache.lock().map_err(|_| {
            Error::EncryptionError("Failed to acquire lock on keypair cache".to_string())
        })?;
        
        cache.insert(key_id.to_string(), (private_key, public_key));
        
        Ok(())
    }
    
    /// Removes a keypair from the cache.
    pub fn remove_keypair(&self, key_id: &str) -> Result<()> {
        let mut cache = self.keypair_cache.lock().map_err(|_| {
            Error::EncryptionError("Failed to acquire lock on keypair cache".to_string())
        })?;
        
        cache.remove(key_id);
        
        Ok(())
    }
    
    /// Generates a new keypair and adds it to the cache.
    pub fn generate_keypair(&self, key_id: &str) -> Result<()> {
        let private_key = StaticSecret::new(OsRng);
        let public_key = PublicKey::from(&private_key);
        
        let mut cache = self.keypair_cache.lock().map_err(|_| {
            Error::EncryptionError("Failed to acquire lock on keypair cache".to_string())
        })?;
        
        cache.insert(key_id.to_string(), (private_key, public_key));
        
        Ok(())
    }
    
    /// Gets the keypair for the given key_id, or the default keypair if None.
    fn get_keypair(&self, key_id: Option<&str>) -> Result<(&StaticSecret, PublicKey)> {
        match key_id {
            Some(id) => {
                let cache = self.keypair_cache.lock().map_err(|_| {
                    Error::EncryptionError("Failed to acquire lock on keypair cache".to_string())
                })?;
                
                if let Some((private_key, public_key)) = cache.get(id) {
                    Ok((private_key, *public_key))
                } else {
                    Err(Error::EncryptionError(format!("Key ID '{}' not found in cache", id)))
                }
            }
            None => Ok((&self.default_private_key, self.default_public_key)),
        }
    }
    
    /// Derives a symmetric key from a shared secret.
    fn derive_symmetric_key(&self, shared_secret: &[u8]) -> [u8; AES_KEY_SIZE] {
        // Use SHA-256 to derive a key from the shared secret
        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        let result = hasher.finalize();
        
        let mut key = [0u8; AES_KEY_SIZE];
        key.copy_from_slice(&result);
        key
    }
}

impl super::Encryptor for EccEncryptor {
    fn encrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        // Get the keypair
        let (_, public_key) = self.get_keypair(key_id)?;
        
        // Generate an ephemeral key for this encryption
        let ephemeral_secret = EphemeralSecret::new(OsRng);
        let ephemeral_public = PublicKey::from(&ephemeral_secret);
        
        // Perform key exchange to get a shared secret
        let shared_secret = ephemeral_secret.diffie_hellman(&public_key);
        
        // Derive a symmetric key from the shared secret
        let symmetric_key = self.derive_symmetric_key(shared_secret.as_bytes());
        
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the data with the chosen symmetric algorithm
        let ciphertext = match self.symmetric_algorithm {
            SymmetricAlgorithm::AesGcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&symmetric_key));
                cipher.encrypt(&nonce, data).map_err(|e| {
                    Error::EncryptionError(format!("AES-GCM encryption failed: {}", e))
                })?
            }
            SymmetricAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(Key::<ChaCha20Poly1305>::from_slice(&symmetric_key));
                cipher.encrypt(&nonce, data).map_err(|e| {
                    Error::EncryptionError(format!("ChaCha20-Poly1305 encryption failed: {}", e))
                })?
            }
        };
        
        // Combine ephemeral public key, nonce, and ciphertext
        let mut result = Vec::with_capacity(X25519_PUBLIC_KEY_SIZE + NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(ephemeral_public.as_bytes());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    fn decrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        if data.len() < X25519_PUBLIC_KEY_SIZE + NONCE_SIZE {
            return Err(Error::EncryptionError(
                "Data too short to contain ECC public key and nonce".to_string(),
            ));
        }
        
        // Get the keypair
        let (private_key, _) = self.get_keypair(key_id)?;
        
        // Extract the ephemeral public key
        let mut ephemeral_public_bytes = [0u8; X25519_PUBLIC_KEY_SIZE];
        ephemeral_public_bytes.copy_from_slice(&data[..X25519_PUBLIC_KEY_SIZE]);
        let ephemeral_public = PublicKey::from(ephemeral_public_bytes);
        
        // Perform key exchange to get the shared secret
        let shared_secret = private_key.diffie_hellman(&ephemeral_public);
        
        // Derive the symmetric key
        let symmetric_key = self.derive_symmetric_key(shared_secret.as_bytes());
        
        // Extract the nonce
        let nonce = Nonce::from_slice(&data[X25519_PUBLIC_KEY_SIZE..X25519_PUBLIC_KEY_SIZE + NONCE_SIZE]);
        
        // Extract the ciphertext
        let ciphertext = &data[X25519_PUBLIC_KEY_SIZE + NONCE_SIZE..];
        
        // Decrypt the data with the chosen symmetric algorithm
        let plaintext = match self.symmetric_algorithm {
            SymmetricAlgorithm::AesGcm => {
                let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&symmetric_key));
                cipher.decrypt(nonce, ciphertext).map_err(|e| {
                    Error::EncryptionError(format!("AES-GCM decryption failed: {}", e))
                })?
            }
            SymmetricAlgorithm::ChaCha20Poly1305 => {
                let cipher = ChaCha20Poly1305::new(Key::<ChaCha20Poly1305>::from_slice(&symmetric_key));
                cipher.decrypt(nonce, ciphertext).map_err(|e| {
                    Error::EncryptionError(format!("ChaCha20-Poly1305 decryption failed: {}", e))
                })?
            }
        };
        
        Ok(plaintext)
    }
}
