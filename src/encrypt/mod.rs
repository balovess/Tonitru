// Encryption module for Tonitru network native data format
//
// This module provides encryption and decryption capabilities for Tonitru data.
// It supports multiple encryption algorithms and field-level encryption.

use crate::internal::error::{Error, Result};
use std::fmt::Debug;

pub mod aes_gcm;
pub mod chacha20_poly1305;
pub mod kyber;
pub mod ecc;
pub mod field_level;
pub mod key_management;

/// Defines the encryption strategy to use.
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)] // Ensure enum variants have a fixed u8 representation
pub enum EncryptionStrategy {
    /// No encryption, data is stored as-is
    NoEncryption = 0,
    /// AES-GCM encryption
    AesGcm = 1,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305 = 2,
    /// Kyber768 post-quantum encryption
    Kyber = 3,
    /// Hybrid encryption (AES-GCM + Kyber)
    Hybrid = 4,
    /// Hybrid encryption (ChaCha20-Poly1305 + Kyber)
    ChaChaKyberHybrid = 5,
    /// ECC with AES-GCM
    EccAesGcm = 6,
    /// ECC with ChaCha20-Poly1305
    EccChaCha20Poly1305 = 7,
}

/// Trait for encryption algorithms.
pub trait Encryptor: Debug {
    /// Encrypts the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to encrypt
    /// * `key_id` - Optional key identifier for key management
    ///
    /// # Returns
    ///
    /// Returns the encrypted data or an error.
    fn encrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>>;

    /// Decrypts the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to decrypt
    /// * `key_id` - Optional key identifier for key management
    ///
    /// # Returns
    ///
    /// Returns the decrypted data or an error.
    fn decrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>>;
}

/// Returns an Encryptor implementation based on the given strategy.
pub fn get_encryptor(strategy: EncryptionStrategy) -> Result<Box<dyn Encryptor>> {
    match strategy {
        EncryptionStrategy::NoEncryption => Ok(Box::new(NoEncryptionEncryptor)),
        EncryptionStrategy::AesGcm => Ok(Box::new(aes_gcm::AesGcmEncryptor::new()?)),
        EncryptionStrategy::ChaCha20Poly1305 => Ok(Box::new(chacha20_poly1305::ChaCha20Poly1305Encryptor::new()?)),
        EncryptionStrategy::Kyber => Ok(Box::new(kyber::KyberEncryptor::new()?)),
        EncryptionStrategy::Hybrid => Ok(Box::new(HybridEncryptor::new()?)),
        EncryptionStrategy::ChaChaKyberHybrid => Ok(Box::new(ChaChaKyberHybridEncryptor::new()?)),
        EncryptionStrategy::EccAesGcm => Ok(Box::new(ecc::EccEncryptor::new(ecc::SymmetricAlgorithm::AesGcm)?)),
        EncryptionStrategy::EccChaCha20Poly1305 => Ok(Box::new(ecc::EccEncryptor::new(ecc::SymmetricAlgorithm::ChaCha20Poly1305)?)),
    }
}

/// A no-op encryptor that doesn't perform any encryption.
#[derive(Debug)]
pub struct NoEncryptionEncryptor;

impl Encryptor for NoEncryptionEncryptor {
    fn encrypt(&self, data: &[u8], _key_id: Option<&str>) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decrypt(&self, data: &[u8], _key_id: Option<&str>) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }
}

/// A hybrid encryptor that combines AES-GCM and Kyber for both
/// high-performance and post-quantum security.
#[derive(Debug)]
pub struct HybridEncryptor {
    aes_gcm: aes_gcm::AesGcmEncryptor,
    kyber: kyber::KyberEncryptor,
}

impl HybridEncryptor {
    /// Creates a new HybridEncryptor.
    pub fn new() -> Result<Self> {
        Ok(Self {
            aes_gcm: aes_gcm::AesGcmEncryptor::new()?,
            kyber: kyber::KyberEncryptor::new()?,
        })
    }
}

impl Encryptor for HybridEncryptor {
    fn encrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        // First encrypt with AES-GCM
        let aes_encrypted = self.aes_gcm.encrypt(data, key_id)?;

        // Then encrypt the result with Kyber
        self.kyber.encrypt(&aes_encrypted, key_id)
    }

    fn decrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        // First decrypt with Kyber
        let kyber_decrypted = self.kyber.decrypt(data, key_id)?;

        // Then decrypt with AES-GCM
        self.aes_gcm.decrypt(&kyber_decrypted, key_id)
    }
}

/// A hybrid encryptor that combines ChaCha20-Poly1305 and Kyber for both
/// high-performance and post-quantum security.
#[derive(Debug)]
pub struct ChaChaKyberHybridEncryptor {
    chacha: chacha20_poly1305::ChaCha20Poly1305Encryptor,
    kyber: kyber::KyberEncryptor,
}

impl ChaChaKyberHybridEncryptor {
    /// Creates a new ChaChaKyberHybridEncryptor.
    pub fn new() -> Result<Self> {
        Ok(Self {
            chacha: chacha20_poly1305::ChaCha20Poly1305Encryptor::new()?,
            kyber: kyber::KyberEncryptor::new()?,
        })
    }
}

impl Encryptor for ChaChaKyberHybridEncryptor {
    fn encrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        // First encrypt with ChaCha20-Poly1305
        let chacha_encrypted = self.chacha.encrypt(data, key_id)?;

        // Then encrypt the result with Kyber
        self.kyber.encrypt(&chacha_encrypted, key_id)
    }

    fn decrypt(&self, data: &[u8], key_id: Option<&str>) -> Result<Vec<u8>> {
        // First decrypt with Kyber
        let kyber_decrypted = self.kyber.decrypt(data, key_id)?;

        // Then decrypt with ChaCha20-Poly1305
        self.chacha.decrypt(&kyber_decrypted, key_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_encryption() {
        let encryptor = NoEncryptionEncryptor;
        let data = b"Test data";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_eq!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_aes_gcm_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::AesGcm).unwrap();
        let data = b"Test data for AES-GCM encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_chacha20_poly1305_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::ChaCha20Poly1305).unwrap();
        let data = b"Test data for ChaCha20-Poly1305 encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_kyber_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::Kyber).unwrap();
        let data = b"Test data for Kyber encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_hybrid_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::Hybrid).unwrap();
        let data = b"Test data for hybrid encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_chacha_kyber_hybrid_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::ChaChaKyberHybrid).unwrap();
        let data = b"Test data for ChaCha20-Poly1305 + Kyber hybrid encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_ecc_aes_gcm_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::EccAesGcm).unwrap();
        let data = b"Test data for ECC + AES-GCM encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }

    #[test]
    fn test_ecc_chacha20_poly1305_encryption() {
        let encryptor = get_encryptor(EncryptionStrategy::EccChaCha20Poly1305).unwrap();
        let data = b"Test data for ECC + ChaCha20-Poly1305 encryption";

        let encrypted = encryptor.encrypt(data, None).unwrap();
        assert_ne!(&encrypted, data);

        let decrypted = encryptor.decrypt(&encrypted, None).unwrap();
        assert_eq!(&decrypted, data);
    }
}