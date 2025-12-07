use crate::models::{Block, Transaction, PendingTransaction};
use crate::crypto::sha256_hash;
use crate::database::DbPool;
use chrono::Utc;
use std::env;
use uuid::Uuid;

/// Calculate the block reward based on block height (halving mechanism)
pub fn calculate_block_reward(block_height: i32) -> f64 {
    let initial_reward = env::var("BLOCK_REWARD")
        .unwrap_or_else(|_| "50.0".to_string())
        .parse::<f64>()
        .unwrap_or(50.0);
    
    let halving_interval = env::var("HALVING_INTERVAL")
        .unwrap_or_else(|_| "210".to_string())
        .parse::<i32>()
        .unwrap_or(210);
    
    // Calculate number of halvings that have occurred
    let halvings = block_height / halving_interval;
    
    // Reward = initial_reward / (2 ^ halvings)
    // Using bit shift for efficiency: dividing by 2^n is same as right shift by n
    if halvings >= 64 {
        // After 64 halvings, reward becomes effectively 0
        return 0.0;
    }
    
    initial_reward / (2_u64.pow(halvings as u32) as f64)
}

/// Get total coins mined so far (sum of all coinbase rewards)
pub async fn get_total_mined_coins(client: &deadpool_postgres::Client) -> Result<f64, anyhow::Error> {
    let row = client.query_one(
        "SELECT COALESCE(SUM(amount)::float8, 0) 
         FROM utxos 
         WHERE transaction_hash LIKE 'coinbase_%'",
        &[],
    ).await?;
    
    let total: f64 = row.get(0);
    Ok(total)
}

/// Calculate merkle root from transactions
pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return sha256_hash(b"empty");
    }

    let mut hashes: Vec<String> = transactions
        .iter()
        .map(|tx| tx.transaction_hash.clone())
        .collect();

    while hashes.len() > 1 {
        let mut new_level = Vec::new();
        
        for i in (0..hashes.len()).step_by(2) {
            let left = &hashes[i];
            let right = if i + 1 < hashes.len() {
                &hashes[i + 1]
            } else {
                left // Duplicate if odd number
            };
            
            let combined = format!("{}{}", left, right);
            let hash = sha256_hash(combined.as_bytes());
            new_level.push(hash);
        }
        
        hashes = new_level;
    }

    hashes[0].clone()
}

/// Calculate block hash
pub fn calculate_block_hash(block: &Block) -> String {
    // Only serialize blockchain-relevant transaction fields for hash calculation
    // Exclude database-specific fields (id, created_at) to ensure consistent hashing
    let tx_hashes: Vec<&String> = block.transactions
        .iter()
        .map(|tx| &tx.transaction_hash)
        .collect();
    let transactions_data = serde_json::to_string(&tx_hashes).unwrap_or_default();
    
    let data = format!(
        "{}{}{}{}{}{}",
        block.index,
        block.timestamp,
        transactions_data,
        block.previous_hash,
        block.nonce,
        block.merkle_root.as_deref().unwrap_or("")
    );
    sha256_hash(data.as_bytes())
}

/// Legacy hash calculation for backward compatibility with old blocks
fn calculate_block_hash_legacy(block: &Block) -> String {
    let transactions_json = serde_json::to_string(&block.transactions).unwrap_or_default();
    let data = format!(
        "{}{}{}{}{}{}",
        block.index,
        block.timestamp,
        transactions_json,
        block.previous_hash,
        block.nonce,
        block.merkle_root.as_deref().unwrap_or("")
    );
    sha256_hash(data.as_bytes())
}

/// Proof of Work: Find nonce that produces hash with required difficulty (Multi-threaded)
pub fn proof_of_work(block: &mut Block, difficulty: usize) -> i64 {
    use std::sync::{Arc, atomic::{AtomicBool, AtomicI64, Ordering}};
    use std::thread;
    
    let target = "0".repeat(difficulty);
    let num_threads = num_cpus::get();
    let found = Arc::new(AtomicBool::new(false));
    let found_nonce = Arc::new(AtomicI64::new(0));
    let block_clone = Arc::new(block.clone());
    
    log::info!("Starting mining with {} threads", num_threads);
    
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let found = Arc::clone(&found);
            let found_nonce = Arc::clone(&found_nonce);
            let target = target.clone();
            let block = Arc::clone(&block_clone);
            
            thread::spawn(move || {
                let mut nonce = thread_id as i64;
                let step = num_threads as i64;
                
                loop {
                    if found.load(Ordering::Relaxed) {
                        break;
                    }
                    
                    let mut test_block = (*block).clone();
                    test_block.nonce = nonce;
                    let hash = calculate_block_hash(&test_block);
                    
                    if hash.starts_with(&target) {
                        found.store(true, Ordering::Relaxed);
                        found_nonce.store(nonce, Ordering::Relaxed);
                        log::info!("✅ Block mined! Thread {} found nonce: {}", thread_id, nonce);
                        break;
                    }
                    
                    nonce += step;
                    
                    // Log progress every 100k attempts per thread
                    if nonce % 100000 == 0 {
                        log::info!("Thread {} mining... nonce: {}", thread_id, nonce);
                    }
                }
            })
        })
        .collect();
    
    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
    
    let nonce = found_nonce.load(Ordering::Relaxed);
    block.nonce = nonce;
    block.hash = calculate_block_hash(block);
    
    nonce
}

/// Validate a single block
pub fn validate_block(block: &Block, previous_block: Option<&Block>) -> bool {
    // Check if hash is correct - try both old and new hash calculation methods
    let calculated_hash_new = calculate_block_hash(block);
    let calculated_hash_old = calculate_block_hash_legacy(block);
    
    if calculated_hash_new != block.hash && calculated_hash_old != block.hash {
        log::error!("Invalid block hash (tried both new and legacy methods)");
        return false;
    }

    // Check previous hash
    if let Some(prev) = previous_block {
        if block.previous_hash != prev.hash {
            log::error!("Invalid previous hash");
            return false;
        }
        
        if block.index != prev.index + 1 {
            log::error!("Invalid block index");
            return false;
        }
    }

    // Check merkle root
    let calculated_merkle = calculate_merkle_root(&block.transactions);
    if let Some(merkle) = &block.merkle_root {
        if merkle != &calculated_merkle {
            log::error!("Invalid merkle root");
            return false;
        }
    }

    // Check difficulty
    let difficulty = env::var("MINING_DIFFICULTY")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(5);
    
    let target = "0".repeat(difficulty);
    if !block.hash.starts_with(&target) {
        log::error!("Hash doesn't meet difficulty requirement");
        return false;
    }

    true
}

/// Validate entire blockchain
pub async fn validate_blockchain(pool: &DbPool) -> Result<bool, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    
    // Get all blocks
    let rows = client
        .query("SELECT index FROM blocks ORDER BY index ASC", &[])
        .await?;
    
    let mut previous_block: Option<Block> = None;
    
    for row in rows {
        let index: i64 = row.get(0);
        let block = crate::database::queries::get_block_by_index(&client, index)
            .await?
            .ok_or("Block not found")?;
        
        if !validate_block(&block, previous_block.as_ref()) {
            log::error!("Blockchain validation failed at block {}", index);
            return Ok(false);
        }
        
        previous_block = Some(block);
    }
    
    log::info!("✅ Blockchain validation successful");
    Ok(true)
}

/// Create genesis block
pub fn create_genesis_block() -> Block {
    let transactions = vec![];
    let merkle_root = calculate_merkle_root(&transactions);
    
    let mut block = Block {
        index: 0,
        timestamp: Utc::now().timestamp(),
        transactions,
        previous_hash: "0".to_string(),
        hash: String::new(),
        nonce: 0,
        merkle_root: Some(merkle_root),
    };

    // Mine genesis block
    let difficulty = env::var("MINING_DIFFICULTY")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(5);
    
    proof_of_work(&mut block, difficulty);
    
    block
}

/// Initialize blockchain (create genesis block if needed)
pub async fn initialize_blockchain(pool: DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    
    // Check if genesis block exists
    let result = client
        .query_opt("SELECT index FROM blocks WHERE index = 0", &[])
        .await?;
    
    if result.is_none() {
        log::info!("Creating genesis block...");
        let genesis = create_genesis_block();
        
        crate::database::queries::create_block(&client, &genesis).await?;
        
        log::info!("✅ Genesis block created: {}", genesis.hash);
    } else {
        log::info!("✅ Blockchain already initialized");
    }
    
    Ok(())
}

/// Mine pending transactions into a new block with coinbase reward
pub async fn mine_block(pool: &DbPool, miner_wallet_id: &str) -> Result<Block, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    
    // Get latest block
    let latest_block = crate::database::queries::get_latest_block(&client)
        .await?
        .ok_or("No blocks found")?;
    
    // Get pending transactions
    let pending_transactions = crate::database::queries::get_pending_transactions(&client).await?;
    
    log::info!("Mining block with {} pending transactions", pending_transactions.len());
    
    // Convert pending transactions to transactions
    let transactions: Vec<Transaction> = pending_transactions
        .iter()
        .map(|pt| Transaction {
            id: pt.id,
            transaction_hash: pt.transaction_hash.clone(),
            sender_wallet_id: pt.sender_wallet_id.clone(),
            receiver_wallet_id: pt.receiver_wallet_id.clone(),
            amount: pt.amount,
            note: pt.note.clone(),
            signature: pt.signature.clone(),
            block_index: Some(latest_block.index + 1),
            transaction_type: "transfer".to_string(),
            timestamp: pt.timestamp,
            created_at: pt.created_at,
        })
        .collect();
    
    // Create new block
    let merkle_root = calculate_merkle_root(&transactions);
    
    let mut new_block = Block {
        index: latest_block.index + 1,
        timestamp: Utc::now().timestamp(),
        transactions: transactions.clone(),
        previous_hash: latest_block.hash.clone(),
        hash: String::new(),
        nonce: 0,
        merkle_root: Some(merkle_root),
    };
    
    // Proof of Work
    let difficulty = env::var("MINING_DIFFICULTY")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(5);
    
    log::info!("Starting Proof of Work with difficulty {}...", difficulty);
    proof_of_work(&mut new_block, difficulty);
    log::info!("✅ Block mined! Hash: {}", new_block.hash);
    
    // Save block to database
    log::info!("Saving block to database: index={}, timestamp={}, hash={}", 
        new_block.index, new_block.timestamp, new_block.hash);
    
    match crate::database::queries::create_block(&client, &new_block).await {
        Ok(_) => log::info!("✅ Block saved to database"),
        Err(e) => {
            log::error!("❌ Failed to save block: {:?}", e);
            return Err(Box::new(e));
        }
    }
    
    // Process each pending transaction and collect fees
    let mut total_fees = 0.0;
    
    for pending_tx in &pending_transactions {
        // Move to transactions table
        match crate::database::queries::create_transaction(
            &client,
            pending_tx,
            new_block.index,
            "transfer",
        )
        .await {
            Ok(_) => {},
            Err(e) => {
                log::error!("❌ Failed to create transaction for {}: {:?}", pending_tx.transaction_hash, e);
                // Release reserved UTXOs on failure
                if let Err(release_err) = release_reserved_utxos_internal(&client, pending_tx.id, &pending_tx.sender_wallet_id).await {
                    log::error!("Failed to release UTXOs for failed transaction {}: {}", pending_tx.id, release_err);
                }
                continue; // Skip this transaction but continue with others
            }
        }
        
        // Update UTXOs: mark spent and create new ones, collect fee
        match update_utxos_for_transaction(&client, pending_tx).await {
            Ok(fee) => {
                total_fees += fee;
                log::info!("✅ Collected fee: {} for transaction {}", fee, pending_tx.transaction_hash);
            },
            Err(e) => {
                log::error!("❌ Failed to update UTXOs for {}: {:?}", pending_tx.transaction_hash, e);
                // Release reserved UTXOs on failure
                if let Err(release_err) = release_reserved_utxos_internal(&client, pending_tx.id, &pending_tx.sender_wallet_id).await {
                    log::error!("Failed to release UTXOs for failed transaction {}: {}", pending_tx.id, release_err);
                }
                continue;
            }
        }
        
        // Delete from pending only after successful processing
        crate::database::queries::delete_pending_transaction(&client, pending_tx.id).await?;
    }
    
    // Calculate block reward with halving mechanism
    let block_reward = calculate_block_reward(new_block.index as i32);
    
    // Check if we've reached max supply
    let max_supply = env::var("MAX_COIN_SUPPLY")
        .unwrap_or_else(|_| "21000000.0".to_string())
        .parse::<f64>()
        .unwrap_or(21000000.0);
    
    let total_mined = get_total_mined_coins(&client).await?;
    
    let actual_reward = if total_mined + block_reward > max_supply {
        // If adding full reward would exceed max supply, only give remaining amount
        let remaining = max_supply - total_mined;
        if remaining > 0.0 {
            log::warn!("⚠️ Approaching max supply! Reward reduced from {} to {}", block_reward, remaining);
            remaining
        } else {
            log::warn!("⚠️ Max coin supply reached! No mining reward for block {}", new_block.index);
            0.0
        }
    } else {
        block_reward
    };
    
    // Add transaction fees to block reward
    let total_reward = actual_reward + total_fees;
    
    // Only create coinbase UTXO if there's a reward to give
    if total_reward > 0.0 {
        let coinbase_hash = sha256_hash(format!("coinbase_{}_{}", new_block.index, miner_wallet_id).as_bytes());
        
        // Create UTXO for mining reward + fees
        crate::database::queries::create_utxo(
            &client,
            miner_wallet_id,
            total_reward,
            &coinbase_hash,
            0,
        )
        .await?;
        
        log::info!("✅ Block {} mined! Reward: {} coins (Block reward: {}, Fees: {}, Block height: {}, Total mined: {}/{})", 
            new_block.index, total_reward, actual_reward, total_fees, new_block.index, total_mined + actual_reward, max_supply);
    } else {
        log::info!("✅ Block {} mined! No reward (max supply reached)", new_block.index);
    }
    
    // Update miner's wallet balance
    let miner_balance = calculate_wallet_balance(&client, miner_wallet_id).await?;
    crate::database::queries::update_wallet_balance(&client, miner_wallet_id, miner_balance).await?;
    
    Ok(new_block)
}

/// Update UTXOs for a transaction and return the transaction fee
async fn update_utxos_for_transaction(
    client: &deadpool_postgres::Client,
    transaction: &PendingTransaction,
) -> Result<f64, anyhow::Error> {
    // Get sender's unspent UTXOs
    let sender_utxos = crate::database::queries::get_unspent_utxos(client, &transaction.sender_wallet_id).await?;
    
    // Select UTXOs to cover the transaction amount + fee
    let total_required = transaction.amount + transaction.fee;
    let mut total = 0.0;
    let mut utxos_to_spend = Vec::new();
    
    for utxo in sender_utxos {
        if total >= total_required {
            break;
        }
        total += utxo.amount;
        utxos_to_spend.push(utxo);
    }
    
    if total < total_required {
        return Err(anyhow::anyhow!("Insufficient UTXOs to cover transaction amount + fee"));
    }
    
    // Mark selected UTXOs as spent
    for utxo in &utxos_to_spend {
        crate::database::queries::mark_utxo_spent(client, utxo.id).await?;
    }
    
    log::info!("✅ Spent {} UTXOs (total: {}) for transaction {}", 
        utxos_to_spend.len(), total, transaction.transaction_hash);
    
    // Create new UTXO for receiver
    crate::database::queries::create_utxo(
        client,
        &transaction.receiver_wallet_id,
        transaction.amount,
        &transaction.transaction_hash,
        0,
    )
    .await?;
    
    // Create change UTXO if needed (after deducting amount + fee)
    let change = total - transaction.amount - transaction.fee;
    if change > 0.0 {
        crate::database::queries::create_utxo(
            client,
            &transaction.sender_wallet_id,
            change,
            &transaction.transaction_hash,
            1,
        )
        .await?;
    }
    
    // Update wallet balances
    let sender_balance = calculate_wallet_balance(client, &transaction.sender_wallet_id).await?;
    let receiver_balance = calculate_wallet_balance(client, &transaction.receiver_wallet_id).await?;
    
    crate::database::queries::update_wallet_balance(client, &transaction.sender_wallet_id, sender_balance).await?;
    crate::database::queries::update_wallet_balance(client, &transaction.receiver_wallet_id, receiver_balance).await?;
    
    // Return the fee for this transaction
    Ok(transaction.fee)
}

/// Release reserved UTXOs when mining fails (internal helper)
async fn release_reserved_utxos_internal(
    client: &deadpool_postgres::Client,
    pending_tx_id: Uuid,
    wallet_id: &str,
) -> Result<(), anyhow::Error> {
    // Release UTXOs reserved by this pending transaction
    client
        .execute(
            "UPDATE utxos SET reserved_by = NULL WHERE reserved_by = $1",
            &[&pending_tx_id],
        )
        .await?;
    
    // Update wallet balance (coins are now available again)
    let updated_balance = calculate_wallet_balance(client, wallet_id).await?;
    crate::database::queries::update_wallet_balance(client, wallet_id, updated_balance).await?;
    
    log::info!("✅ Released reserved UTXOs for failed transaction {} (balance restored: {})", 
        pending_tx_id, updated_balance);
    
    Ok(())
}

/// Calculate wallet balance from UTXOs
pub async fn calculate_wallet_balance(
    client: &deadpool_postgres::Client,
    wallet_id: &str,
) -> Result<f64, anyhow::Error> {
    let utxos = crate::database::queries::get_unspent_utxos(client, wallet_id).await?;
    
    // Calculate total balance from all unspent UTXOs
    let total_balance: f64 = utxos.iter()
        .filter(|u| !u.is_spent)
        .map(|u| u.amount)
        .sum();
    
    // Get amount locked in pending outgoing transactions
    let pending_amount: f64 = match client.query_one(
        "SELECT COALESCE(SUM(amount)::float8, 0) 
         FROM pending_transactions 
         WHERE sender_wallet_id = $1",
        &[&wallet_id],
    ).await {
        Ok(row) => row.get(0),
        Err(_) => 0.0,
    };
    
    // Available balance = total balance - pending sends
    Ok(total_balance - pending_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block_creation() {
        let genesis = create_genesis_block();
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert!(!genesis.hash.is_empty());
    }

    #[test]
    fn test_merkle_root_empty() {
        let root = calculate_merkle_root(&[]);
        assert!(!root.is_empty());
    }

    #[test]
    fn test_block_hash_calculation() {
        let block = Block {
            index: 1,
            timestamp: 1234567890,
            transactions: vec![],
            previous_hash: "previous".to_string(),
            hash: String::new(),
            nonce: 0,
            merkle_root: None,
        };
        
        let hash = calculate_block_hash(&block);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }
}
