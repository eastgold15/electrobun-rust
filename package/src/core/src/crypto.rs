//! Crypto module - AES-GCM encryption for secure WebSocket communication

use crate::error::ElectrobunError;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sha1::{Sha1, Digest};

/// AES-256-GCM key length (32 bytes)
pub const AES256_KEY_LENGTH: usize = 32;

/// Derive WebSocket accept key from challenge

pub fn derive_websocket_accept(challenge: &str) -> String {
    let combined = format!("{}{}", challenge, super::transport::WEBSOCKET_MAGIC);
    
    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    
    // Base64 encode
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(result)
}

/// Encrypt message with AES-256-GCM
pub fn encrypt_message(key: &[u8; AES256_KEY_LENGTH], nonce: &[u8; 12], plaintext: &[u8]) 
    -> Result<Vec<u8>, ElectrobunError> 
{
    if key.len() != AES256_KEY_LENGTH {
        return Err(ElectrobunError::CryptoError(format!(
            "Key must be {} bytes, got {}", AES256_KEY_LENGTH, key.len()
        )));
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| ElectrobunError::CryptoError(e.to_string()))?;
    
    let nonce = Nonce::from_slice(nonce);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| ElectrobunError::CryptoError(e.to_string()))?;
    
    Ok(ciphertext)
}

/// Decrypt message with AES-256-GCM
pub fn decrypt_message(key: &[u8; AES256_KEY_LENGTH], nonce: &[u8; 12], ciphertext: &[u8]) 
    -> Result<Vec<u8>, ElectrobunError> 
{
    if key.len() != AES256_KEY_LENGTH {
        return Err(ElectrobunError::CryptoError(format!(
            "Key must be {} bytes, got {}", AES256_KEY_LENGTH, key.len()
        )));
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| ElectrobunError::CryptoError(e.to_string()))?;
    
    let nonce = Nonce::from_slice(nonce);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| ElectrobunError::CryptoError(e.to_string()))?;
    
    Ok(plaintext)
}

/// Generate a random nonce
pub fn generate_nonce() -> [u8; 12] {
    use aes_gcm::aead::rand_core::RngCore;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Generate a random key
pub fn generate_key() -> [u8; AES256_KEY_LENGTH] {
    use aes_gcm::aead::rand_core::RngCore;
    let mut key = [0u8; AES256_KEY_LENGTH];
    OsRng.fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let nonce = generate_nonce();
        let plaintext = b"Hello, World!";
        
        let ciphertext = encrypt_message(&key, &nonce, plaintext).unwrap();
        let decrypted = decrypt_message(&key, &nonce, &ciphertext).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
