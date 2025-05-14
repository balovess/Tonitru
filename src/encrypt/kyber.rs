// Kyber post-quantum encryption implementation for Tonitru
//
// This module provides Kyber768 post-quantum encryption and decryption functionality.

use crate::internal::error::{Error, Result};
use kyber_rust::{
    kyber768::{
        decapsulate as kyber_decapsulate, encapsulate as kyber_encapsulate,
        keypair as kyber_keypair,
    },
    KYBER_CIPHERTEXTBYTES, KYBER_PUBLICKEYBYTES, KYBER_SECRETKEYBYTES, KYBER_SYMBYTES,
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// KyberEncryptor implementation
///
/// This encryptor uses Kyber768 for key encapsulation and AES-GCM for data encryption.
/// Kyber is a post-quantum key encapsulation mechanism (KEM) that is believed to be
/// secure against attacks by quantum computers.
#[derive(Debug)]
pub struct KyberEncryptor {
    // Default keypair used when no key_id is provided
    default_public_key: [u8; KYBER_PUBLICKEYBYTES],
    default_secret_key: [u8; KYBER_SECRETKEYBYTES],
    // Cache of keypairs for different key_ids
    keypair_cache: Arc<Mutex<HashMap<String, ([u8; KYBER_PUBLICKEYBYTES], [u8; KYBER_SECRETKEYBYTES])>>>,
}

impl KyberEncryptor {
    /// Creates a new KyberEncryptor with a randomly generated default keypair.
    pub fn new() -> Result<Self> {
        let (public_key, secret_key) = kyber_keypair();
        
        Ok(Self {
            default_public_key: public_key,
            default_secret_key: secret_key,
            keypair_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Creates a new KyberEncryptor with the provided keypair.
    pub fn with_keypair(
        public_key: [u8; KYBER_PUBLICKEYBYTES],
        secret_key: [u8; KYBER_SECRETKEYBYTES],
    ) -> Result<Self> {
        Ok(Self {
            default_public_key: public_key,
            default_secret_key: secret_key,
            keypair_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Adds a keypair to the cache.
    pub fn add_keypair(
        &self,
        key_id: &str,
        public_key: [u8; KYBER_PUBLICKEYBYTES],
        secret_key: [u8; KYBER_SECRETKEYBYTES],
    ) -> Result<()> {
        let mut cache = self.keypair_cache.lock().map_err(|_| {
            Error::EncryptionError("Failed to acquire lock on keypair cache".to_string())
        })?;
        
        cache.insert(key_id.to_string(), (public_key, secret_key));
        
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
        let (public_key, secret_key) = kyber_keypair();
        self.add_keypair(key_id, public_key, secret_key)
    }
    
    /// Gets the keypair for the given key_id, or the default keypair if None.
    fn get_keypair(
        &self,
        key_id: Option<&str>,
    ) -> Result<([u8; KYBER_PUBLICKEYBYTES], [u8; KYBER_SECRETKEYBYTES])> {
        match key_id {
            Some(id) => {
                let cache = self.keypair_cache.lock().map_err(|_| {
                    Error::EncryptionError("Failed to acquire lock on keypair cache".to_string())
                })?;
                
                cache.get(id).cloned().ok_or_else(|| {
                    Error::EncryptionError(format!("Key ID '{}' not found in cache", id))
                })
            }
            None => Ok((self.default_public_key, self.default_secret_key)),
        }
    }
}

impl super::Encryptor for KyberEncryptor {
    fn encrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        let (public_key, _) = self.get_keypair(key_id)?;
        
        // Encapsulate a shared secret using Kyber
        let (ciphertext, shared_secret) = kyber_encapsulate(&public_key);
        
        // Use the shared secret as an AES key
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&shared_secret));
        
        // Use a fixed nonce for simplicity (in a real system, this should be random)
        let nonce = Nonce::from_slice(&[0u8; 12]);
        
        // Encrypt the data with AES-GCM
        let encrypted_data = cipher.encrypt(nonce, data).map_err(|e| {
            Error::EncryptionError(format!("AES-GCM encryption failed: {}", e))
        })?;
        
        // Combine Kyber ciphertext and encrypted data
        let mut result = Vec::with_capacity(KYBER_CIPHERTEXTBYTES + encrypted_data.len());
        result.extend_from_slice(&ciphertext);
        result.extend_from_slice(&encrypted_data);
        
        Ok(result)
    }
    
    fn decrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        if data.len() < KYBER_CIPHERTEXTBYTES {
            return Err(Error::EncryptionError(
                "Data too short to contain Kyber ciphertext".to_string(),
            ));
        }
        
        let (_, secret_key) = self.get_keypair(key_id)?;
        
        // Split data into Kyber ciphertext and encrypted data
        let kyber_ciphertext = {
            let mut ct = [0u8; KYBER_CIPHERTEXTBYTES];
            ct.copy_from_slice(&data[..KYBER_CIPHERTEXTBYTES]);
            ct
        };
        let encrypted_data = &data[KYBER_CIPHERTEXTBYTES..];
        
        // Decapsulate the shared secret using Kyber
        let shared_secret = kyber_decapsulate(&kyber_ciphertext, &secret_key);
        
        // Use the shared secret as an AES key
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&shared_secret));
        
        // Use the same fixed nonce as in encryption
        let nonce = Nonce::from_slice(&[0u8; 12]);
        
        // Decrypt the data with AES-GCM
        let decrypted_data = cipher.decrypt(nonce, encrypted_data).map_err(|e| {
            Error::EncryptionError(format!("AES-GCM decryption failed: {}", e))
        })?;
        
        Ok(decrypted_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kyber_encrypt_decrypt() {
        let encryptor = KyberEncryptor::new().unwrap();
        let data = b"Test data for Kyber encryption";
        
        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted[KYBER_CIPHERTEXTBYTES..], data);
        
        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }
    
    #[test]
    fn test_kyber_with_keypair() {
        let (public_key, secret_key) = kyber_keypair();
        let encryptor = KyberEncryptor::with_keypair(public_key, secret_key).unwrap();
        let data = b"Test data with custom keypair";
        
        let encrypted = encryptor.encrypt(data, None).unwrap();
        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }
    
    #[test]
    fn test_kyber_key_management() {
        let encryptor = KyberEncryptor::new().unwrap();
        let key_id = "test-key-1";
        let data = b"Test data with key management";
        
        // Generate and add a keypair
        encryptor.generate_keypair(key_id).unwrap();
        
        // Encrypt with the keypair
        let encrypted = encryptor.encrypt(data, Some(key_id)).unwrap();
        
        // Decrypt with the keypair
        let decrypted = encryptor.decrypt(&encrypted, Some(key_id)).unwrap();
        assert_eq!(&decrypted, data);
        
        // Decrypt with wrong keypair should fail
        encryptor.generate_keypair("wrong-key").unwrap();
        assert!(encryptor.decrypt(&encrypted, Some("wrong-key")).is_err());
        
        // Remove the keypair
        encryptor.remove_keypair(key_id).unwrap();
        
        // Encrypt/decrypt with removed keypair should fail
        assert!(encryptor.encrypt(data, Some(key_id)).is_err());
        assert!(encryptor.decrypt(&encrypted, Some(key_id)).is_err());
    }
}
