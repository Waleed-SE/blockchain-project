use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;
use std::env;

pub type DbPool = Pool;

pub async fn create_pool() -> Result<DbPool, Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;
    
    let mut cfg = Config::new();
    cfg.url = Some(database_url);
    
    // Limit pool size for Supabase free tier (max 10 connections in session mode)
    cfg.pool = Some(deadpool_postgres::PoolConfig::new(10));
    
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    
    log::info!("✅ Database connection pool created");
    
    Ok(pool)
}

pub mod queries {
    use crate::models::*;
    use crate::models::Transaction as TxModel;
    use deadpool_postgres::Client;
    use tokio_postgres::Transaction;
    use uuid::Uuid;
    use chrono::{Utc, DateTime};

    // User queries
    pub async fn create_user(
        client: &Client,
        email: &str,
        full_name: &str,
        cnic: &str,
        wallet_id: &str,
        public_key: &str,
        encrypted_private_key: &str,
    ) -> Result<User, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO users (email, full_name, cnic, wallet_id, public_key, encrypted_private_key) 
                 VALUES ($1, $2, $3, $4, $5, $6) 
                 RETURNING id, email, full_name, cnic, wallet_id, public_key, encrypted_private_key, is_verified, created_at, updated_at",
                &[&email, &full_name, &cnic, &wallet_id, &public_key, &encrypted_private_key],
            )
            .await?;

        Ok(User {
            id: row.get(0),
            email: row.get(1),
            full_name: row.get(2),
            cnic: row.get(3),
            wallet_id: row.get(4),
            public_key: row.get(5),
            encrypted_private_key: row.get(6),
            is_verified: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        })
    }

    pub async fn find_user_by_email(client: &Client, email: &str) -> Result<Option<User>, tokio_postgres::Error> {
        let result = client
            .query_opt(
                "SELECT id, email, full_name, cnic, wallet_id, public_key, encrypted_private_key, is_verified, created_at, updated_at 
                 FROM users WHERE email = $1",
                &[&email],
            )
            .await?;

        Ok(result.map(|row| User {
            id: row.get(0),
            email: row.get(1),
            full_name: row.get(2),
            cnic: row.get(3),
            wallet_id: row.get(4),
            public_key: row.get(5),
            encrypted_private_key: row.get(6),
            is_verified: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        }))
    }

    pub async fn find_user_by_id(client: &Client, user_id: Uuid) -> Result<Option<User>, tokio_postgres::Error> {
        let result = client
            .query_opt(
                "SELECT id, email, full_name, cnic, wallet_id, public_key, encrypted_private_key, is_verified, created_at, updated_at 
                 FROM users WHERE id = $1",
                &[&user_id],
            )
            .await?;

        Ok(result.map(|row| User {
            id: row.get(0),
            email: row.get(1),
            full_name: row.get(2),
            cnic: row.get(3),
            wallet_id: row.get(4),
            public_key: row.get(5),
            encrypted_private_key: row.get(6),
            is_verified: row.get(7),
            created_at: row.get(8),
            updated_at: row.get(9),
        }))
    }

    // Wallet queries
    pub async fn create_wallet(
        client: &Client,
        wallet_id: &str,
        user_id: Option<Uuid>,
    ) -> Result<Wallet, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO wallets (wallet_id, user_id, balance) 
                 VALUES ($1, $2, 0) 
                 RETURNING wallet_id, user_id, balance::float8, last_zakat_date, created_at, updated_at",
                &[&wallet_id, &user_id],
            )
            .await?;

        Ok(Wallet {
            wallet_id: row.get(0),
            user_id: row.get(1),
            balance: row.get(2),
            last_zakat_date: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
        })
    }

    pub async fn get_wallet(client: &Client, wallet_id: &str) -> Result<Option<Wallet>, tokio_postgres::Error> {
        let result = client
            .query_opt(
                "SELECT wallet_id, user_id, balance::float8, last_zakat_date, created_at, updated_at 
                 FROM wallets WHERE wallet_id = $1",
                &[&wallet_id],
            )
            .await?;

        Ok(result.map(|row| Wallet {
            wallet_id: row.get(0),
            user_id: row.get(1),
            balance: row.get(2),
            last_zakat_date: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
        }))
    }

    pub async fn update_wallet_balance(
        client: &Client,
        wallet_id: &str,
        new_balance: f64,
    ) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "UPDATE wallets SET balance = $1::float8, updated_at = $2 WHERE wallet_id = $3",
                &[&new_balance, &Utc::now(), &wallet_id],
            )
            .await?;
        Ok(())
    }

    // UTXO queries
    pub async fn create_utxo(
        client: &Client,
        wallet_id: &str,
        amount: f64,
        transaction_hash: &str,
        output_index: i32,
    ) -> Result<UTXO, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO utxos (wallet_id, amount, transaction_hash, output_index) 
                 VALUES ($1, $2::float8, $3, $4) 
                 RETURNING id, wallet_id, amount::float8, transaction_hash, output_index, is_spent, created_at, spent_at",
                &[&wallet_id, &amount, &transaction_hash, &output_index],
            )
            .await?;

        Ok(UTXO {
            id: row.get(0),
            wallet_id: row.get(1),
            amount: row.get(2),
            transaction_hash: row.get(3),
            output_index: row.get(4),
            is_spent: row.get(5),
            created_at: row.get(6),
            spent_at: row.get(7),
            reserved_by: None, // New UTXOs are not reserved
        })
    }

    pub async fn get_unspent_utxos(client: &Client, wallet_id: &str) -> Result<Vec<UTXO>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, wallet_id, amount::float8, transaction_hash, output_index, is_spent, created_at, spent_at, reserved_by 
                 FROM utxos WHERE wallet_id = $1 AND is_spent = false 
                 ORDER BY created_at ASC",
                &[&wallet_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| UTXO {
                id: row.get(0),
                wallet_id: row.get(1),
                amount: row.get(2),
                transaction_hash: row.get(3),
                output_index: row.get(4),
                is_spent: row.get(5),
                created_at: row.get(6),
                spent_at: row.get(7),
                reserved_by: row.get(8),
            })
            .collect())
    }

    pub async fn mark_utxo_spent(client: &Client, utxo_id: Uuid) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "UPDATE utxos SET is_spent = true, spent_at = $1 WHERE id = $2",
                &[&Utc::now(), &utxo_id],
            )
            .await?;
        Ok(())
    }

    // Block queries
    pub async fn create_block(client: &Client, block: &Block) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "INSERT INTO blocks (\"index\", timestamp, previous_hash, hash, nonce, merkle_root) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &block.index,
                    &block.timestamp,
                    &block.previous_hash,
                    &block.hash,
                    &block.nonce,
                    &block.merkle_root,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn get_latest_block(client: &Client) -> Result<Option<Block>, tokio_postgres::Error> {
        let result = client
            .query_opt(
                "SELECT index, timestamp, previous_hash, hash, nonce, merkle_root 
                 FROM blocks ORDER BY index DESC LIMIT 1",
                &[],
            )
            .await?;

        if let Some(row) = result {
            let index: i64 = row.get(0);
            
            // Get transactions for this block
            let tx_rows = client
                .query(
                    "SELECT id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount::float8, note, 
                     signature, block_index, transaction_type, timestamp, created_at 
                     FROM transactions WHERE block_index = $1",
                    &[&index],
                )
                .await?;

            let transactions = tx_rows
                .into_iter()
                .map(|tx_row| TxModel {
                    id: tx_row.get(0),
                    transaction_hash: tx_row.get(1),
                    sender_wallet_id: tx_row.get(2),
                    receiver_wallet_id: tx_row.get(3),
                    amount: tx_row.get(4),
                    note: tx_row.get(5),
                    signature: tx_row.get(6),
                    block_index: tx_row.get(7),
                    transaction_type: tx_row.get(8),
                    timestamp: tx_row.get(9),
                    created_at: tx_row.get(10),
                })
                .collect();

            Ok(Some(Block {
                index,
                timestamp: row.get(1),
                previous_hash: row.get(2),
                hash: row.get(3),
                nonce: row.get(4),
                merkle_root: row.get(5),
                transactions,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_block_by_index(client: &Client, block_index: i64) -> Result<Option<Block>, tokio_postgres::Error> {
        let result = client
            .query_opt(
                "SELECT index, timestamp, previous_hash, hash, nonce, merkle_root 
                 FROM blocks WHERE index = $1",
                &[&block_index],
            )
            .await?;

        if let Some(row) = result {
            let index: i64 = row.get(0);
            
            let tx_rows = client
                .query(
                    "SELECT id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount::float8, note, 
                     signature, block_index, transaction_type, timestamp, created_at 
                     FROM transactions WHERE block_index = $1",
                    &[&index],
                )
                .await?;

            let transactions = tx_rows
                .into_iter()
                .map(|tx_row| TxModel {
                    id: tx_row.get(0),
                    transaction_hash: tx_row.get(1),
                    sender_wallet_id: tx_row.get(2),
                    receiver_wallet_id: tx_row.get(3),
                    amount: tx_row.get(4),
                    note: tx_row.get(5),
                    signature: tx_row.get(6),
                    block_index: tx_row.get(7),
                    transaction_type: tx_row.get(8),
                    timestamp: tx_row.get(9),
                    created_at: tx_row.get(10),
                })
                .collect();

            Ok(Some(Block {
                index,
                timestamp: row.get(1),
                previous_hash: row.get(2),
                hash: row.get(3),
                nonce: row.get(4),
                merkle_root: row.get(5),
                transactions,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_blocks(client: &Client, limit: i64, offset: i64) -> Result<Vec<Block>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT index, timestamp, previous_hash, hash, nonce, merkle_root 
                 FROM blocks ORDER BY index DESC LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await?;

        let mut blocks = Vec::new();
        for row in rows {
            let index: i64 = row.get(0);
            
            let tx_rows = client
                .query(
                    "SELECT id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount::float8, note, 
                     signature, block_index, transaction_type, timestamp, created_at 
                     FROM transactions WHERE block_index = $1",
                    &[&index],
                )
                .await?;

            let transactions = tx_rows
                .into_iter()
                .map(|tx_row| TxModel {
                    id: tx_row.get(0),
                    transaction_hash: tx_row.get(1),
                    sender_wallet_id: tx_row.get(2),
                    receiver_wallet_id: tx_row.get(3),
                    amount: tx_row.get(4),
                    note: tx_row.get(5),
                    signature: tx_row.get(6),
                    block_index: tx_row.get(7),
                    transaction_type: tx_row.get(8),
                    timestamp: tx_row.get(9),
                    created_at: tx_row.get(10),
                })
                .collect();

            blocks.push(Block {
                index,
                timestamp: row.get(1),
                previous_hash: row.get(2),
                hash: row.get(3),
                nonce: row.get(4),
                merkle_root: row.get(5),
                transactions,
            });
        }

        Ok(blocks)
    }

    // Transaction queries
    pub async fn create_pending_transaction(
        client: &Client,
        transaction: &PendingTransaction,
    ) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "INSERT INTO pending_transactions (id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount, fee, note, signature, timestamp) 
                 VALUES ($1, $2, $3, $4, $5::float8, $6::float8, $7, $8, $9)",
                &[
                    &transaction.id,
                    &transaction.transaction_hash,
                    &transaction.sender_wallet_id,
                    &transaction.receiver_wallet_id,
                    &transaction.amount,
                    &transaction.fee,
                    &transaction.note,
                    &transaction.signature,
                    &transaction.timestamp,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn get_pending_transactions(client: &Client) -> Result<Vec<PendingTransaction>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount::float8, fee::float8, note, signature, timestamp, created_at 
                 FROM pending_transactions ORDER BY created_at ASC",
                &[],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| PendingTransaction {
                id: row.get(0),
                transaction_hash: row.get(1),
                sender_wallet_id: row.get(2),
                receiver_wallet_id: row.get(3),
                amount: row.get(4),
                fee: row.get(5),
                note: row.get(6),
                signature: row.get(7),
                timestamp: row.get(8),
                created_at: row.get(9),
            })
            .collect())
    }

    pub async fn delete_pending_transaction(client: &Client, tx_id: Uuid) -> Result<(), tokio_postgres::Error> {
        client
            .execute("DELETE FROM pending_transactions WHERE id = $1", &[&tx_id])
            .await?;
        Ok(())
    }

    pub async fn create_transaction(
        client: &Client,
        pending_tx: &PendingTransaction,
        block_index: i64,
        transaction_type: &str,
    ) -> Result<TxModel, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO transactions (transaction_hash, sender_wallet_id, receiver_wallet_id, amount, note, signature, block_index, transaction_type, timestamp) 
                 VALUES ($1, $2, $3, $4::float8, $5, $6, $7, $8, $9) 
                 RETURNING id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount::float8, note, signature, block_index, transaction_type, timestamp, created_at",
                &[
                    &pending_tx.transaction_hash,
                    &pending_tx.sender_wallet_id,
                    &pending_tx.receiver_wallet_id,
                    &pending_tx.amount,
                    &pending_tx.note,
                    &pending_tx.signature,
                    &block_index,
                    &transaction_type,
                    &pending_tx.timestamp,
                ],
            )
            .await?;

        Ok(TxModel {
            id: row.get(0),
            transaction_hash: row.get(1),
            sender_wallet_id: row.get(2),
            receiver_wallet_id: row.get(3),
            amount: row.get(4),
            note: row.get(5),
            signature: row.get(6),
            block_index: row.get(7),
            transaction_type: row.get(8),
            timestamp: row.get(9),
            created_at: row.get(10),
        })
    }

    pub async fn get_wallet_transactions(
        client: &Client,
        wallet_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TxModel>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount::float8, note, 
                 signature, block_index, transaction_type, timestamp, created_at 
                 FROM transactions 
                 WHERE sender_wallet_id = $1 OR receiver_wallet_id = $1 
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&wallet_id, &limit, &offset],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| TxModel {
                id: row.get(0),
                transaction_hash: row.get(1),
                sender_wallet_id: row.get(2),
                receiver_wallet_id: row.get(3),
                amount: row.get(4),
                note: row.get(5),
                signature: row.get(6),
                block_index: row.get(7),
                transaction_type: row.get(8),
                timestamp: row.get(9),
                created_at: row.get(10),
            })
            .collect())
    }

    // OTP queries
    pub async fn create_otp(
        client: &Client,
        email: &str,
        otp: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<EmailOtp, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO email_otps (email, otp, expires_at) 
                 VALUES ($1, $2, $3) 
                 RETURNING id, email, otp, is_verified, expires_at, created_at",
                &[&email, &otp, &expires_at],
            )
            .await?;

        Ok(EmailOtp {
            id: row.get(0),
            email: row.get(1),
            otp: row.get(2),
            is_verified: row.get(3),
            expires_at: row.get(4),
            created_at: row.get(5),
        })
    }

    pub async fn verify_otp(
        client: &Client,
        email: &str,
        otp: &str,
    ) -> Result<bool, tokio_postgres::Error> {
        let result = client
            .query_opt(
                "UPDATE email_otps 
                 SET is_verified = TRUE 
                 WHERE email = $1 AND otp = $2 AND is_verified = FALSE AND expires_at > NOW() 
                 RETURNING id",
                &[&email, &otp],
            )
            .await?;

        Ok(result.is_some())
    }

    pub async fn mark_user_verified(
        client: &Client,
        email: &str,
    ) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "UPDATE users SET is_verified = TRUE WHERE email = $1",
                &[&email],
            )
            .await?;
        Ok(())
    }

    // System logs
    pub async fn create_system_log(
        client: &Client,
        log_type: &str,
        user_id: Option<Uuid>,
        message: &str,
        ip_address: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "INSERT INTO system_logs (log_type, user_id, message, ip_address, metadata) 
                 VALUES ($1, $2, $3, $4, $5)",
                &[&log_type, &user_id, &message, &ip_address, &metadata],
            )
            .await?;
        Ok(())
    }

    // Transaction logs
    pub async fn create_transaction_log(
        client: &Client,
        wallet_id: &str,
        action: &str,
        transaction_hash: Option<String>,
        block_hash: Option<String>,
        status: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
        note: Option<String>,
    ) -> Result<(), tokio_postgres::Error> {
        client
            .execute(
                "INSERT INTO transaction_logs (wallet_id, action, transaction_hash, block_hash, status, ip_address, user_agent, note) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &wallet_id,
                    &action,
                    &transaction_hash,
                    &block_hash,
                    &status,
                    &ip_address,
                    &user_agent,
                    &note,
                ],
            )
            .await?;
        Ok(())
    }

    // Beneficiary queries
    pub async fn get_user_beneficiaries(
        client: &Client,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::Beneficiary>, tokio_postgres::Error> {
        let rows = client
            .query(
                "SELECT id, user_id, beneficiary_wallet_id, nickname, created_at 
                 FROM beneficiaries WHERE user_id = $1 ORDER BY created_at DESC",
                &[&user_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| crate::models::Beneficiary {
                id: row.get(0),
                user_id: row.get(1),
                beneficiary_wallet_id: row.get(2),
                nickname: row.get(3),
                created_at: row.get(4),
            })
            .collect())
    }

    pub async fn add_beneficiary(
        client: &Client,
        user_id: Uuid,
        beneficiary_wallet_id: &str,
        nickname: Option<String>,
    ) -> Result<crate::models::Beneficiary, tokio_postgres::Error> {
        let row = client
            .query_one(
                "INSERT INTO beneficiaries (user_id, beneficiary_wallet_id, nickname) 
                 VALUES ($1, $2, $3) 
                 RETURNING id, user_id, beneficiary_wallet_id, nickname, created_at",
                &[&user_id, &beneficiary_wallet_id, &nickname],
            )
            .await?;

        Ok(crate::models::Beneficiary {
            id: row.get(0),
            user_id: row.get(1),
            beneficiary_wallet_id: row.get(2),
            nickname: row.get(3),
            created_at: row.get(4),
        })
    }

    pub async fn delete_beneficiary(
        client: &Client,
        beneficiary_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        let result = client
            .execute(
                "DELETE FROM beneficiaries WHERE id = $1 AND user_id = $2",
                &[&beneficiary_id, &user_id],
            )
            .await?;
        Ok(result)
    }
}

