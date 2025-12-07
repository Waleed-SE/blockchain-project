use actix_web::{web, HttpResponse, HttpRequest};
use crate::models::{ApiResponse, BlockchainInfo, MiningStats};
use crate::database::DbPool;
use crate::blockchain;
use crate::services::{transaction_service, auth_service};
use uuid::Uuid;
use std::env;

pub async fn get_blocks(
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(50);
    let offset = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    
    let client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    match crate::database::queries::get_all_blocks(&client, limit, offset).await {
        Ok(blocks) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(blocks),
            message: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_block(
    pool: web::Data<DbPool>,
    path: web::Path<i64>,
) -> HttpResponse {
    let block_index = path.into_inner();
    
    let client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    match crate::database::queries::get_block_by_index(&client, block_index).await {
        Ok(Some(block)) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(block),
            message: None,
        }),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Block not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn validate_chain(pool: web::Data<DbPool>) -> HttpResponse {
    match blockchain::validate_blockchain(&pool).await {
        Ok(is_valid) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "is_valid": is_valid
            })),
            message: Some(if is_valid {
                "Blockchain is valid".to_string()
            } else {
                "Blockchain validation failed".to_string()
            }),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Validation error: {}", e)),
        }),
    }
}

pub async fn mine_block(pool: web::Data<DbPool>, req: HttpRequest) -> HttpResponse {
    // Extract token from Authorization header
    let token = match req.headers().get("Authorization") {
        Some(header) => {
            match header.to_str() {
                Ok(auth_str) => {
                    auth_str.strip_prefix("Bearer ").unwrap_or("")
                }
                Err(_) => {
                    return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        message: Some("Invalid authorization header".to_string()),
                    });
                }
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Missing authorization header".to_string()),
            });
        }
    };

    // Verify token and get user_id
    let claims = match auth_service::verify_token(token) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid or expired token".to_string()),
            });
        }
    };

    // Get user from database to retrieve wallet_id
    let client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    // Parse user_id from claims
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid user ID in token".to_string()),
            });
        }
    };

    let user_row = match client
        .query_one("SELECT wallet_id FROM users WHERE id = $1", &[&user_id])
        .await
    {
        Ok(row) => row,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("User not found: {}", e)),
            });
        }
    };

    let wallet_id: String = user_row.get(0);

    match blockchain::mine_block(&pool, &wallet_id).await {
        Ok(block) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "block_index": block.index,
                "block_hash": block.hash,
                "transactions_count": block.transactions.len(),
                "nonce": block.nonce,
                "timestamp": block.timestamp,
            })),
            message: Some("Block mined successfully".to_string()),
        }),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn get_info(pool: web::Data<DbPool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let total_blocks_result = client
        .query_one("SELECT COUNT(*) FROM blocks", &[])
        .await;

    let total_transactions_result = client
        .query_one("SELECT COUNT(*) FROM transactions", &[])
        .await;

    let total_wallets_result = client
        .query_one("SELECT COUNT(*) FROM wallets", &[])
        .await;

    let latest_block_result = crate::database::queries::get_latest_block(&client).await;

    let pending_count_result = transaction_service::get_pending_count(&pool).await;

    match (total_blocks_result, total_transactions_result, total_wallets_result, latest_block_result, pending_count_result) {
        (Ok(blocks_row), Ok(tx_row), Ok(wallets_row), Ok(latest_block), Ok(pending_count)) => {
            let total_blocks: i64 = blocks_row.get(0);
            let total_transactions: i64 = tx_row.get(0);
            let total_wallets: i64 = wallets_row.get(0);
            
            // Get mining configuration
            let mining_difficulty: i32 = env::var("MINING_DIFFICULTY")
                .unwrap_or("3".to_string())
                .parse()
                .unwrap_or(3);
            
            let current_block_height = latest_block.as_ref().map(|b| b.index).unwrap_or(0);
            let current_block_reward = blockchain::calculate_block_reward(current_block_height as i32);
            
            let transaction_fee: f64 = env::var("TRANSACTION_FEE")
                .unwrap_or("0.1".to_string())
                .parse()
                .unwrap_or(0.1);
            
            let info = BlockchainInfo {
                total_blocks,
                latest_block,
                pending_transactions: pending_count,
                total_transactions,
                total_wallets,
                mining_difficulty,
                current_block_reward,
                transaction_fee,
            };

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(info),
                message: None,
            })
        }
        _ => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Failed to retrieve blockchain info".to_string()),
        }),
    }
}

pub async fn get_mining_stats(pool: web::Data<DbPool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    // Get latest block to determine current height
    let latest_block = match crate::database::queries::get_latest_block(&client).await {
        Ok(Some(block)) => block,
        Ok(None) => {
            return HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(MiningStats {
                    current_block_height: 0,
                    current_block_reward: blockchain::calculate_block_reward(0),
                    next_halving_block: env::var("HALVING_INTERVAL").unwrap_or("210".to_string()).parse().unwrap_or(210) as i64,
                    blocks_until_halving: env::var("HALVING_INTERVAL").unwrap_or("210".to_string()).parse().unwrap_or(210) as i64,
                    total_mined_coins: 0.0,
                    max_coin_supply: env::var("MAX_COIN_SUPPLY").unwrap_or("21000000.0".to_string()).parse().unwrap_or(21000000.0),
                    remaining_coins: env::var("MAX_COIN_SUPPLY").unwrap_or("21000000.0".to_string()).parse().unwrap_or(21000000.0),
                    halving_interval: env::var("HALVING_INTERVAL").unwrap_or("210".to_string()).parse().unwrap_or(210),
                    percentage_mined: 0.0,
                }),
                message: Some("No blocks mined yet".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Failed to get latest block: {}", e)),
            });
        }
    };

    let current_height = latest_block.index;
    
    // Get configuration
    let halving_interval: i32 = env::var("HALVING_INTERVAL")
        .unwrap_or("210".to_string())
        .parse()
        .unwrap_or(210);
    
    let max_supply: f64 = env::var("MAX_COIN_SUPPLY")
        .unwrap_or("21000000.0".to_string())
        .parse()
        .unwrap_or(21000000.0);
    
    // Calculate current reward
    let current_reward = blockchain::calculate_block_reward(current_height as i32);
    
    // Calculate next halving block
    let next_halving_block = ((current_height / halving_interval as i64) + 1) * halving_interval as i64;
    let blocks_until_halving = next_halving_block - current_height;
    
    // Get total mined coins
    let total_mined = match blockchain::get_total_mined_coins(&client).await {
        Ok(total) => total,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Failed to calculate total mined coins: {}", e)),
            });
        }
    };
    
    let remaining = (max_supply - total_mined).max(0.0);
    let percentage_mined = (total_mined / max_supply * 100.0).min(100.0);
    
    let stats = MiningStats {
        current_block_height: current_height,
        current_block_reward: current_reward,
        next_halving_block,
        blocks_until_halving,
        total_mined_coins: total_mined,
        max_coin_supply: max_supply,
        remaining_coins: remaining,
        halving_interval,
        percentage_mined,
    };

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(stats),
        message: None,
    })
}
