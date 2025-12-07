use crate::models::{PendingTransaction, CreateTransactionRequest};
use crate::crypto::{create_transaction_payload, verify_signature, import_public_key_pem, sha256_hash, decrypt_private_key, import_private_key_pem, sign_data};
use crate::database::{DbPool, queries};
use crate::blockchain::calculate_wallet_balance;
use uuid::Uuid;
use chrono::Utc;
use std::env;

#[derive(Debug)]
pub enum TransactionError {
    InvalidWallet(String),
    InsufficientBalance,
    InvalidSignature,
    InvalidAmount,
    DatabaseError(String),
    CryptoError(String),
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransactionError::InvalidWallet(msg) => write!(f, "Invalid wallet: {}", msg),
            TransactionError::InsufficientBalance => write!(f, "Insufficient balance"),
            TransactionError::InvalidSignature => write!(f, "Invalid signature"),
            TransactionError::InvalidAmount => write!(f, "Invalid amount"),
            TransactionError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            TransactionError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
        }
    }
}

impl std::error::Error for TransactionError {}

/// Validate and create a new transaction
pub async fn create_transaction(
    pool: &DbPool,
    req: CreateTransactionRequest,
    aes_key: &[u8],
) -> Result<PendingTransaction, TransactionError> {
    let client = pool.get().await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

    // Validate amount
    if req.amount <= 0.0 {
        return Err(TransactionError::InvalidAmount);
    }

    // Check sender wallet exists
    let sender_wallet = queries::get_wallet(&client, &req.sender_wallet_id)
        .await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?
        .ok_or_else(|| TransactionError::InvalidWallet("Sender wallet not found".to_string()))?;

    // Check receiver wallet exists
    let _receiver_wallet = queries::get_wallet(&client, &req.receiver_wallet_id)
        .await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?
        .ok_or_else(|| TransactionError::InvalidWallet("Receiver wallet not found".to_string()))?;

    // Get transaction fee from environment
    let transaction_fee = env::var("TRANSACTION_FEE")
        .unwrap_or_else(|_| "0.1".to_string())
        .parse::<f64>()
        .unwrap_or(0.1);

    // Calculate sender's balance from UTXOs
    let sender_balance = calculate_wallet_balance(&client, &req.sender_wallet_id)
        .await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

    // Check if sender has enough balance for amount + fee
    let total_required = req.amount + transaction_fee;
    if sender_balance < total_required {
        return Err(TransactionError::InsufficientBalance);
    }

    // Get sender's user info for public key and encrypted private key
    let sender_user = queries::find_user_by_id(&client, sender_wallet.user_id.unwrap())
        .await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?
        .ok_or_else(|| TransactionError::InvalidWallet("Sender user not found".to_string()))?;

    // Decrypt and import private key from sender's user record
    let decrypted_private_key = decrypt_private_key(&sender_user.encrypted_private_key, aes_key)
        .map_err(|e| TransactionError::CryptoError(e.to_string()))?;
    
    let private_key = import_private_key_pem(&decrypted_private_key)
        .map_err(|e| TransactionError::CryptoError(e.to_string()))?;

    // Create transaction payload
    let timestamp = Utc::now().timestamp();
    let payload = create_transaction_payload(
        &req.sender_wallet_id,
        &req.receiver_wallet_id,
        req.amount,
        timestamp,
        &req.note,
    );

    // Sign transaction
    let signature = sign_data(&private_key, &payload)
        .map_err(|e| TransactionError::CryptoError(e.to_string()))?;

    // Verify signature with public key
    let public_key = import_public_key_pem(&sender_user.public_key)
        .map_err(|e| TransactionError::CryptoError(e.to_string()))?;

    let is_valid = verify_signature(&public_key, &payload, &signature)
        .map_err(|e| TransactionError::CryptoError(e.to_string()))?;

    if !is_valid {
        return Err(TransactionError::InvalidSignature);
    }

    // Create transaction hash
    let transaction_hash = sha256_hash(format!("{}{}", payload, signature).as_bytes());

    // Create pending transaction
    let pending_tx = PendingTransaction {
        id: Uuid::new_v4(),
        transaction_hash: transaction_hash.clone(),
        sender_wallet_id: req.sender_wallet_id.clone(),
        receiver_wallet_id: req.receiver_wallet_id.clone(),
        amount: req.amount,
        fee: transaction_fee,
        note: req.note.clone(),
        signature,
        timestamp,
        created_at: Utc::now(),
    };

    // Save to database
    queries::create_pending_transaction(&client, &pending_tx)
        .await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

    // Update sender's balance (will now reflect pending transaction deduction)
    let updated_sender_balance = calculate_wallet_balance(&client, &req.sender_wallet_id).await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;
    queries::update_wallet_balance(&client, &req.sender_wallet_id, updated_sender_balance).await
        .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

    log::info!("✅ Created pending transaction {} for {} coins (new available balance: {})", 
        transaction_hash, req.amount, updated_sender_balance);

    // Log transaction
    queries::create_transaction_log(
        &client,
        &req.sender_wallet_id,
        "sent",
        Some(transaction_hash.clone()),
        None,
        "pending",
        None,
        None,
        req.note.clone(),
    )
    .await
    .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

    queries::create_transaction_log(
        &client,
        &req.receiver_wallet_id,
        "received",
        Some(transaction_hash),
        None,
        "pending",
        None,
        None,
        req.note,
    )
    .await
    .map_err(|e| TransactionError::DatabaseError(e.to_string()))?;

    log::info!("✅ Transaction created: {} -> {} ({})", req.sender_wallet_id, req.receiver_wallet_id, req.amount);

    Ok(pending_tx)
}

/* DEPRECATED: No longer using UTXO reservation - balance calculation now uses pending transaction amounts directly
/// Reserve UTXOs for a pending transaction (lock coins until mined or failed)
async fn reserve_utxos_for_pending_transaction(
    client: &Client,
    transaction: &PendingTransaction,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get sender's unspent and unreserved UTXOs
    let sender_utxos = queries::get_unspent_utxos(client, &transaction.sender_wallet_id).await?;
    
    // Filter out already reserved or spent UTXOs
    let available_utxos: Vec<_> = sender_utxos.into_iter()
        .filter(|utxo| !utxo.is_spent && utxo.reserved_by.is_none())
        .collect();
    
    // Select UTXOs to reserve
    let mut total = 0.0;
    let mut utxos_to_reserve = Vec::new();
    
    for utxo in available_utxos {
        if total >= transaction.amount {
            break;
        }
        total += utxo.amount;
        utxos_to_reserve.push(utxo);
    }
    
    if total < transaction.amount {
        return Err("Insufficient unreserved UTXOs".into());
    }
    
    // Reserve selected UTXOs by linking them to this pending transaction
    for utxo in &utxos_to_reserve {
        client
            .execute(
                "UPDATE utxos SET reserved_by = $1 WHERE id = $2",
                &[&transaction.id, &utxo.id],
            )
            .await?;
    }
    
    log::info!("Reserved {} UTXOs (total: {}) for pending transaction {}", 
        utxos_to_reserve.len(), total, transaction.transaction_hash);
    
    Ok(())
}
*/

/* DEPRECATED: No longer needed with new balance calculation approach
/// Release reserved UTXOs when a pending transaction fails or is cancelled
pub async fn release_reserved_utxos(
    pool: &DbPool,
    pending_tx_id: Uuid,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    
    // Get the wallet_id before releasing
    let wallet_id_result = client
        .query_opt(
            "SELECT sender_wallet_id FROM pending_transactions WHERE id = $1",
            &[&pending_tx_id],
        )
        .await?;
    
    let wallet_id: String = match wallet_id_result {
        Some(row) => row.get(0),
        None => return Ok(()), // Transaction doesn't exist, nothing to release
    };
    
    // Release UTXOs
    client
        .execute(
            "UPDATE utxos SET reserved_by = NULL WHERE reserved_by = $1",
            &[&pending_tx_id],
        )
        .await?;
    
    // Update wallet balance (coins are now available again)
    let updated_balance = calculate_wallet_balance(&client, &wallet_id).await?;
    queries::update_wallet_balance(&client, &wallet_id, updated_balance).await?;
    
    log::info!("✅ Released reserved UTXOs for pending transaction {} (new balance: {})", 
        pending_tx_id, updated_balance);
    
    Ok(())
}
*/

/// Get pending transactions count
pub async fn get_pending_count(pool: &DbPool) -> Result<i32, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let row = client
        .query_one("SELECT COUNT(*) FROM pending_transactions", &[])
        .await?;
    let count: i64 = row.get(0);
    Ok(count as i32)
}
