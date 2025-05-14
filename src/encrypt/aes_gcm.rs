// AES-GCM encryption implementation for Tonitru
//
// This module provides AES-GCM encryption and decryption functionality.

use crate::internal::error::{Error, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The length of the AES-GCM key in bytes (256 bits)
const KEY_SIZE: usize = 32;

/// The length of the nonce in bytes
const NONCE_SIZE: usize = 12;

/// AES-GCM encryptor implementation
#[derive(Debug)]
pub struct AesGcmEncryptor {
    // Default key used when no key_id is provided
    default_key: Key<Aes256Gcm>,
    // Cache of cipher instances for different keys
    cipher_cache: Arc<Mutex<HashMap<String, Aes256Gcm>>>,
}

impl AesGcmEncryptor {
    /// Creates a new AesGcmEncryptor with a randomly generated default key.
    pub fn new() -> Result<Self> {
        let default_key = Aes256Gcm::generate_key(&mut OsRng);
        
        Ok(Self {
            default_key,
            cipher_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Creates a new AesGcmEncryptor with the provided key.
    pub fn with_key(key: &[u8]) -> Result<Self> {
        if key.len() != KEY_SIZE {
            return Err(Error::EncryptionError(format!(
                "Invalid AES-GCM key size: expected {} bytes, got {} bytes",
                KEY_SIZE,
                key.len()
            )));
        }
        
        let default_key = *Key::<Aes256Gcm>::from_slice(key);
        
        Ok(Self {
            default_key,
            cipher_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Adds a key to the cipher cache.
    pub fn add_key(&self, key_id: &str, key: &[u8]) -> Result<()> {
        if key.len() != KEY_SIZE {
            return Err(Error::EncryptionError(format!(
                "Invalid AES-GCM key size: expected {} bytes, got {} bytes",
                KEY_SIZE,
                key.len()
            )));
        }
        
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        
        let mut cache = self.cipher_cache.lock().map_err(|_| {
            Error::EncryptionError("Failed to acquire lock on cipher cache".to_string())
        })?;
        
        cache.insert(key_id.to_string(), cipher);
        
        Ok(())
    }
    
    /// Removes a key from the cipher cache.
    pub fn remove_key(&self, key_id: &str) -> Result<()> {
        let mut cache = self.cipher_cache.lock().map_err(|_| {
            Error::EncryptionError("Failed to acquire lock on cipher cache".to_string())
        })?;
        
        cache.remove(key_id);
        
        Ok(())
    }
    
    /// Gets the cipher for the given key_id, or the default cipher if None.
    fn get_cipher(&self, key_id: Option<&str>) -> Result<Aes256Gcm> {
        match key_id {
            Some(id) => {
                let cache = self.cipher_cache.lock().map_err(|_| {
                    Error::EncryptionError("Failed to acquire lock on cipher cache".to_string())
                })?;
                
                cache.get(id).cloned().ok_or_else(|| {
                    Error::EncryptionError(format!("Key ID '{}' not found in cache", id))
                })
            }
            None => Ok(Aes256Gcm::new(&self.default_key)),
        }
    }
}

impl super::Encryptor for AesGcmEncryptor {
    fn encrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        let cipher = self.get_cipher(key_id)?;
        
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the data
        let ciphertext = cipher.encrypt(&nonce, data).map_err(|e| {
            Error::EncryptionError(format!("AES-GCM encryption failed: {}", e))
        })?;
        
        // Combine nonce and ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    fn decrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        if data.len() < NONCE_SIZE {
            return Err(Error::EncryptionError(
                "Data too short to contain nonce".to_string(),
            ));
        }
        
        let cipher = self.get_cipher(key_id)?;
        
        // Split data into nonce and ciphertext
        let nonce = Nonce::from_slice(&data[..NONCE_SIZE]);
        let ciphertext = &data[NONCE_SIZE..];
        
        // Decrypt the data
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            Error::EncryptionError(format!("AES-GCM decryption failed: {}", e))
        })?;
        
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let encryptor = AesGcmEncryptor::new().unwrap();
        let data = b"Test data for AES-GCM encryption";
        
        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted[NONCE_SIZE..], data);
        
        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }
    
    #[test]
    fn test_aes_gcm_with_key() {
        let key = [0u8; KEY_SIZE];
        let encryptor = AesGcmEncryptor::with_key(&key).unwrap();
        let data = b"Test data with custom key";
        
        let encrypted = encryptor.encrypt(data, None).unwrap();
        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }
    
    #[test]
    fn test_aes_gcm_key_management() {
        let encryptor = AesGcmEncryptor::new().unwrap();
        let key_id = "test-key-1";
        let key = [1u8; KEY_SIZE];
        let data = b"Test data with key management";
        
        // Add a key
        encryptor.add_key(key_id, &key).unwrap();
        
        // Encrypt with the key
        let encrypted = encryptor.encrypt(data, Some(key_id)).unwrap();
        
        // Decrypt with the key
        let decrypted = encryptor.decrypt(&encrypted, Some(key_id)).unwrap();
        assert_eq!(&decrypted, data);
        
        // Decrypt with wrong key should fail
        encryptor.add_key("wrong-key", &[2u8; KEY_SIZE]).unwrap();
        assert!(encryptor.decrypt(&encrypted, Some("wrong-key")).is_err());
        
        // Remove the key
        encryptor.remove_key(key_id).unwrap();
        
        // Encrypt/decrypt with removed key should fail
        assert!(encryptor.encrypt(data, Some(key_id)).is_err());
        assert!(encryptor.decrypt(&encrypted, Some(key_id)).is_err());
    }
}
