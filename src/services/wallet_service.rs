use crate::models::{KeyPair, WalletBalance};
use crate::crypto::{generate_keypair, export_public_key_pem, export_private_key_pem, generate_wallet_id, encrypt_private_key};
use crate::database::{DbPool, queries};
use crate::blockchain::calculate_wallet_balance;

#[derive(Debug)]
pub enum WalletError {
    KeyGenerationError(String),
    EncryptionError(String),
    DatabaseError(String),
    WalletNotFound,
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WalletError::KeyGenerationError(msg) => write!(f, "Key generation error: {}", msg),
            WalletError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            WalletError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            WalletError::WalletNotFound => write!(f, "Wallet not found"),
        }
    }
}

impl std::error::Error for WalletError {}

/// Generate a new wallet with keypair
pub fn generate_wallet_keypair(aes_key: &[u8]) -> Result<KeyPair, WalletError> {
    // Generate RSA keypair
    let (private_key, public_key) = generate_keypair()
        .map_err(|e| WalletError::KeyGenerationError(e.to_string()))?;

    // Export keys to PEM
    let public_key_pem = export_public_key_pem(&public_key)
        .map_err(|e| WalletError::KeyGenerationError(e.to_string()))?;
    
    let private_key_pem = export_private_key_pem(&private_key)
        .map_err(|e| WalletError::KeyGenerationError(e.to_string()))?;

    // Generate wallet ID
    let wallet_id = generate_wallet_id(&public_key)
        .map_err(|e| WalletError::KeyGenerationError(e.to_string()))?;

    // Encrypt private key
    let encrypted_private_key = encrypt_private_key(&private_key_pem, aes_key)
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    Ok(KeyPair {
        public_key: public_key_pem,
        private_key: encrypted_private_key,
        wallet_id,
    })
}

/// Get wallet balance with UTXO count
pub async fn get_wallet_balance(pool: &DbPool, wallet_id: &str) -> Result<WalletBalance, WalletError> {
    let client = pool.get().await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

    // Check if wallet exists
    let wallet = queries::get_wallet(&client, wallet_id)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?
        .ok_or(WalletError::WalletNotFound)?;

    // Get all unspent UTXOs
    let utxos = queries::get_unspent_utxos(&client, wallet_id)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

    // Calculate total balance from all unspent UTXOs
    let total_balance: f64 = utxos.iter().map(|u| u.amount).sum();
    
    // Calculate amount locked in pending outgoing transactions
    let pending_amount_result = client.query_one(
        "SELECT COALESCE(SUM(amount)::float8, 0) 
         FROM pending_transactions 
         WHERE sender_wallet_id = $1",
        &[&wallet_id],
    ).await;
    
    let pending_amount: f64 = match pending_amount_result {
        Ok(row) => row.get(0),
        Err(_) => 0.0,
    };
    
    // Available balance = total balance - pending sends
    let balance = total_balance - pending_amount;
    let utxo_count = utxos.len() as i32;

    // Update cached balance in wallet table
    queries::update_wallet_balance(&client, wallet_id, balance)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

    Ok(WalletBalance {
        wallet_id: wallet_id.to_string(),
        balance,
        utxo_count,
    })
}

/// Check if wallet exists
pub async fn wallet_exists(pool: &DbPool, wallet_id: &str) -> Result<bool, WalletError> {
    let client = pool.get().await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

    let wallet = queries::get_wallet(&client, wallet_id)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

    Ok(wallet.is_some())
}
