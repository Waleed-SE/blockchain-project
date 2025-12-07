use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey, DecodePublicKey, LineEnding}, Pkcs1v15Sign};
use sha2::{Sha256, Digest};
use rand::rngs::OsRng;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng as AesOsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose};

const KEY_SIZE: usize = 2048;

#[derive(Debug)]
pub enum CryptoError {
    KeyGenerationError(String),
    EncryptionError(String),
    DecryptionError(String),
    SignatureError(String),
    VerificationError(String),
    EncodingError(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CryptoError::KeyGenerationError(msg) => write!(f, "Key generation error: {}", msg),
            CryptoError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            CryptoError::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            CryptoError::VerificationError(msg) => write!(f, "Verification error: {}", msg),
            CryptoError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Generate RSA-2048 keypair
pub fn generate_keypair() -> Result<(RsaPrivateKey, RsaPublicKey), CryptoError> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE)
        .map_err(|e| CryptoError::KeyGenerationError(e.to_string()))?;
    let public_key = RsaPublicKey::from(&private_key);
    
    Ok((private_key, public_key))
}

/// Export public key to PEM format
pub fn export_public_key_pem(public_key: &RsaPublicKey) -> Result<String, CryptoError> {
    public_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| CryptoError::EncodingError(e.to_string()))
}

/// Export private key to PEM format
pub fn export_private_key_pem(private_key: &RsaPrivateKey) -> Result<String, CryptoError> {
    private_key
        .to_pkcs8_pem(LineEnding::LF)
        .map(|pem| pem.to_string())
        .map_err(|e| CryptoError::EncodingError(e.to_string()))
}

/// Import public key from PEM format
pub fn import_public_key_pem(pem: &str) -> Result<RsaPublicKey, CryptoError> {
    RsaPublicKey::from_public_key_pem(pem)
        .map_err(|e| CryptoError::EncodingError(e.to_string()))
}

/// Import private key from PEM format
pub fn import_private_key_pem(pem: &str) -> Result<RsaPrivateKey, CryptoError> {
    RsaPrivateKey::from_pkcs8_pem(pem)
        .map_err(|e| CryptoError::EncodingError(e.to_string()))
}

/// Generate wallet ID from public key (SHA-256 hash)
pub fn generate_wallet_id(public_key: &RsaPublicKey) -> Result<String, CryptoError> {
    let pem = export_public_key_pem(public_key)?;
    let hash = sha256_hash(pem.as_bytes());
    Ok(hash)
}

/// SHA-256 hash function
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Sign data with private key
pub fn sign_data(private_key: &RsaPrivateKey, data: &str) -> Result<String, CryptoError> {
    // Hash the data first
    let hash = sha256_hash(data.as_bytes());
    let hash_bytes = hex::decode(&hash)
        .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
    
    // Sign the hash
    let signature = private_key
        .sign(Pkcs1v15Sign::new_unprefixed(), &hash_bytes)
        .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
    
    Ok(hex::encode(signature))
}

/// Verify signature with public key
pub fn verify_signature(public_key: &RsaPublicKey, data: &str, signature_hex: &str) -> Result<bool, CryptoError> {
    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| CryptoError::VerificationError(format!("Invalid hex signature: {}", e)))?;
    
    // Hash the data
    let hash = sha256_hash(data.as_bytes());
    let hash_bytes = hex::decode(&hash)
        .map_err(|e| CryptoError::VerificationError(e.to_string()))?;
    
    match public_key.verify(Pkcs1v15Sign::new_unprefixed(), &hash_bytes, &signature_bytes) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Encrypt private key with AES-256-GCM
pub fn encrypt_private_key(private_key_pem: &str, aes_key: &[u8]) -> Result<String, CryptoError> {
    if aes_key.len() != 32 {
        return Err(CryptoError::EncryptionError("AES key must be 32 bytes".to_string()));
    }

    let cipher = Aes256Gcm::new_from_slice(aes_key)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    
    // Generate random nonce
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, private_key_pem.as_bytes())
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    
    // Combine nonce + ciphertext and encode as base64
    let mut encrypted_data = nonce_bytes.to_vec();
    encrypted_data.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(&encrypted_data))
}

/// Decrypt private key with AES-256-GCM
pub fn decrypt_private_key(encrypted_base64: &str, aes_key: &[u8]) -> Result<String, CryptoError> {
    if aes_key.len() != 32 {
        return Err(CryptoError::DecryptionError("AES key must be 32 bytes".to_string()));
    }

    // Decode base64
    let encrypted_data = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|e| CryptoError::DecryptionError(format!("Invalid base64: {}", e)))?;
    
    if encrypted_data.len() < 12 {
        return Err(CryptoError::DecryptionError("Invalid encrypted data".to_string()));
    }
    
    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let cipher = Aes256Gcm::new_from_slice(aes_key)
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;
    
    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| CryptoError::DecryptionError(format!("Invalid UTF-8: {}", e)))
}

/// Create transaction payload for signing
pub fn create_transaction_payload(
    sender_id: &str,
    receiver_id: &str,
    amount: f64,
    timestamp: i64,
    note: &Option<String>,
) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        sender_id,
        receiver_id,
        amount,
        timestamp,
        note.as_deref().unwrap_or("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let result = generate_keypair();
        assert!(result.is_ok());
    }

    #[test]
    fn test_wallet_id_generation() {
        let (_, public_key) = generate_keypair().unwrap();
        let wallet_id = generate_wallet_id(&public_key).unwrap();
        assert_eq!(wallet_id.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_signature_verification() {
        let (private_key, public_key) = generate_keypair().unwrap();
        let data = "test transaction data";
        
        let signature = sign_data(&private_key, data).unwrap();
        let is_valid = verify_signature(&public_key, data, &signature).unwrap();
        
        assert!(is_valid);
    }

    #[test]
    fn test_signature_verification_fails_with_wrong_data() {
        let (private_key, public_key) = generate_keypair().unwrap();
        let data = "test transaction data";
        let wrong_data = "wrong transaction data";
        
        let signature = sign_data(&private_key, data).unwrap();
        let is_valid = verify_signature(&public_key, wrong_data, &signature).unwrap();
        
        assert!(!is_valid);
    }

    #[test]
    fn test_private_key_encryption_decryption() {
        let (private_key, _) = generate_keypair().unwrap();
        let private_key_pem = export_private_key_pem(&private_key).unwrap();
        
        let aes_key: [u8; 32] = rand::random();
        
        let encrypted = encrypt_private_key(&private_key_pem, &aes_key).unwrap();
        let decrypted = decrypt_private_key(&encrypted, &aes_key).unwrap();
        
        assert_eq!(private_key_pem, decrypted);
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"hello world";
        let hash = sha256_hash(data);
        assert_eq!(hash.len(), 64);
        
        // Hash should be deterministic
        let hash2 = sha256_hash(data);
        assert_eq!(hash, hash2);
    }
}
