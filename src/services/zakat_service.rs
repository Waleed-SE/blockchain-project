use crate::database::{DbPool, queries};
use crate::models::PendingTransaction;
use crate::crypto::{create_transaction_payload, sha256_hash};
use chrono::Utc;
use uuid::Uuid;
use std::env;
use tokio::time::{interval, Duration as TokioDuration};

/// Calculate zakat amount (2.5% of balance)
fn calculate_zakat(balance: f64) -> f64 {
    let zakat_percentage = env::var("ZAKAT_PERCENTAGE")
        .unwrap_or_else(|_| "2.5".to_string())
        .parse::<f64>()
        .unwrap_or(2.5);
    
    balance * (zakat_percentage / 100.0)
}

/// Process zakat deduction for a single wallet
async fn process_wallet_zakat(
    client: &deadpool_postgres::Client,
    wallet_id: &str,
    zakat_pool_wallet_id: &str,
) -> Result<(), anyhow::Error> {
    // Get wallet
    let wallet = match queries::get_wallet(client, wallet_id).await? {
        Some(w) => w,
        None => return Ok(()), // Skip if wallet doesn't exist
    };

    // Skip if balance is 0 or negative
    if wallet.balance <= 0.0 {
        return Ok(());
    }

    // Check if balance meets the zakat threshold (nisab)
    let zakat_threshold = env::var("ZAKAT_THRESHOLD")
        .unwrap_or_else(|_| "100.0".to_string())
        .parse::<f64>()
        .unwrap_or(100.0);
    
    if wallet.balance < zakat_threshold {
        log::info!(
            "Wallet {} balance ({}) is below zakat threshold ({}), skipping zakat deduction",
            wallet_id,
            wallet.balance,
            zakat_threshold
        );
        return Ok(());
    }

    // Check if zakat was paid within the zakat period
    let zakat_period_seconds = env::var("ZAKAT_PERIOD")
        .unwrap_or_else(|_| "2592000".to_string()) // Default: 30 days
        .parse::<i64>()
        .unwrap_or(2592000);

    if let Some(last_zakat_date) = wallet.last_zakat_date {
        let now = Utc::now();
        let time_since_last_zakat = (now - last_zakat_date).num_seconds();
        
        // If zakat was paid within the zakat period, skip
        if time_since_last_zakat < zakat_period_seconds {
            log::info!(
                "Zakat already paid for wallet {} (last paid {} seconds ago, period is {} seconds)", 
                wallet_id, 
                time_since_last_zakat, 
                zakat_period_seconds
            );
            return Ok(());
        }
    }

    // Calculate zakat
    let zakat_amount = calculate_zakat(wallet.balance);
    
    if zakat_amount < 0.01 {
        return Ok(()); // Skip if zakat is too small
    }

    log::info!("Processing zakat for wallet {}: {} (balance: {})", wallet_id, zakat_amount, wallet.balance);

    // Create zakat transaction
    let timestamp = Utc::now().timestamp();
    let payload = create_transaction_payload(
        wallet_id,
        zakat_pool_wallet_id,
        zakat_amount,
        timestamp,
        &Some("Monthly Zakat Deduction (2.5%)".to_string()),
    );

    // For system transactions, we use a system signature
    let signature = sha256_hash(format!("SYSTEM_ZAKAT_{}", payload).as_bytes());
    let transaction_hash = sha256_hash(format!("{}{}", payload, signature).as_bytes());

    // Create pending transaction
    let pending_tx = PendingTransaction {
        id: Uuid::new_v4(),
        transaction_hash: transaction_hash.clone(),
        sender_wallet_id: wallet_id.to_string(),
        receiver_wallet_id: zakat_pool_wallet_id.to_string(),
        amount: zakat_amount,
        fee: 0.0, // Zakat transactions have no fee
        note: Some("Monthly Zakat Deduction (2.5%)".to_string()),
        signature: signature.clone(),
        timestamp,
        created_at: Utc::now(),
    };

    // Save pending transaction
    queries::create_pending_transaction(client, &pending_tx).await?;

    // Update sender's balance (will now reflect pending zakat deduction)
    let updated_balance = crate::blockchain::calculate_wallet_balance(client, wallet_id).await?;
    queries::update_wallet_balance(client, wallet_id, updated_balance).await?;

    log::info!("✅ Created zakat pending transaction {} for {} coins (new available balance: {})", 
        transaction_hash, zakat_amount, updated_balance);

    // Record zakat deduction
    client
        .execute(
            "INSERT INTO zakat_records (wallet_id, amount, transaction_hash, deduction_date) VALUES ($1, $2::float8, $3, $4)",
            &[&wallet_id, &zakat_amount, &transaction_hash, &Utc::now()],
        )
        .await?;

    // Update last zakat date
    client
        .execute(
            "UPDATE wallets SET last_zakat_date = $1 WHERE wallet_id = $2",
            &[&Utc::now(), &wallet_id],
        )
        .await?;

    // Log zakat deduction
    queries::create_system_log(
        client,
        "zakat_deduction",
        None,
        &format!("Zakat deducted from wallet {}: {}", wallet_id, zakat_amount),
        None,
        Some(serde_json::json!({
            "wallet_id": wallet_id,
            "amount": zakat_amount,
            "transaction_hash": transaction_hash,
        })),
    )
    .await?;

    queries::create_transaction_log(
        client,
        wallet_id,
        "zakat_deducted",
        Some(transaction_hash.clone()),
        None,
        "pending",
        None,
        None,
        Some("Monthly Zakat Deduction (2.5%)".to_string()),
    )
    .await?;

    log::info!("✅ Zakat deduction created for wallet {}: {}", wallet_id, zakat_amount);

    Ok(())
}

/// Process zakat for all wallets
pub async fn process_monthly_zakat(pool: &DbPool) -> Result<(), anyhow::Error> {
    log::info!("🕌 Starting monthly zakat deduction process...");

    let client = pool.get().await?;

    let zakat_pool_wallet_id = env::var("ZAKAT_POOL_WALLET_ID")
        .unwrap_or_else(|_| "ZAKAT_POOL".to_string());

    // Ensure zakat pool wallet exists
    if queries::get_wallet(&client, &zakat_pool_wallet_id).await?.is_none() {
        log::info!("Creating zakat pool wallet...");
        queries::create_wallet(&client, &zakat_pool_wallet_id, None).await?;
    }

    // Get all wallets
    let rows = client
        .query("SELECT wallet_id FROM wallets WHERE wallet_id != $1", &[&zakat_pool_wallet_id])
        .await?;

    let mut processed_count = 0;
    let mut error_count = 0;

    for row in rows {
        let wallet_id: String = row.get(0);

        match process_wallet_zakat(&client, &wallet_id, &zakat_pool_wallet_id).await {
            Ok(_) => processed_count += 1,
            Err(e) => {
                error_count += 1;
                log::error!("Error processing zakat for wallet {}: {}", wallet_id, e);
            }
        }
    }

    log::info!("✅ Zakat deduction completed: {} wallets processed, {} error(s)", processed_count, error_count);

    Ok(())
}

/// Start zakat scheduler (configurable intervals)
pub async fn start_zakat_scheduler(pool: DbPool) {
    log::info!("🕌 Starting Zakat scheduler...");

    // Get configuration from environment
    // CHECK_INTERVAL: How often to check if zakat needs to be deducted (e.g., every 5 minutes for testing)
    let check_interval_seconds = env::var("ZAKAT_CHECK_INTERVAL")
        .unwrap_or_else(|_| "300".to_string()) // Default: 5 minutes
        .parse::<u64>()
        .unwrap_or(300);

    // ZAKAT_PERIOD: The actual period for zakat payment (e.g., 30 days)
    let zakat_period_seconds = env::var("ZAKAT_PERIOD")
        .unwrap_or_else(|_| "2592000".to_string()) // Default: 30 days
        .parse::<u64>()
        .unwrap_or(2592000);

    log::info!(
        "🕌 Zakat scheduler configured: Check interval: {} seconds ({} minutes), Zakat period: {} seconds ({} days)",
        check_interval_seconds,
        check_interval_seconds / 60,
        zakat_period_seconds,
        zakat_period_seconds / 86400
    );

    // Run checks at the check interval
    let mut interval = interval(TokioDuration::from_secs(check_interval_seconds));
    
    loop {
        interval.tick().await;
        
        log::info!("🕌 Running scheduled zakat check");
        
        if let Err(e) = process_monthly_zakat(&pool).await {
            log::error!("Error processing zakat: {}", e);
        }
    }
}

/// Manually trigger zakat deduction (for testing or admin purposes)
pub async fn trigger_zakat_deduction(pool: &DbPool) -> Result<(), anyhow::Error> {
    process_monthly_zakat(pool).await
}

/* DEPRECATED: No longer using UTXO reservation - balance calculation now uses pending transaction amounts directly
/// Reserve UTXOs for zakat transaction (helper function)
async fn reserve_utxos_for_zakat(
    client: &deadpool_postgres::Client,
    transaction: &PendingTransaction,
) -> Result<(), anyhow::Error> {
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
        return Err(anyhow::anyhow!("Insufficient unreserved UTXOs for zakat"));
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
    
    log::info!("Reserved {} UTXOs (total: {}) for zakat transaction {}", 
        utxos_to_reserve.len(), total, transaction.transaction_hash);
    
    Ok(())
}
*/
