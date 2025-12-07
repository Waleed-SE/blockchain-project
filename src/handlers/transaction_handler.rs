use actix_web::{web, HttpResponse};
use crate::models::{ApiResponse, CreateTransactionRequest};
use crate::database::DbPool;
use crate::services::transaction_service;
use crate::config::Config;

pub async fn create_transaction(
    pool: web::Data<DbPool>,
    req: web::Json<CreateTransactionRequest>,
) -> HttpResponse {
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Config error: {}", e)),
            });
        }
    };

    match transaction_service::create_transaction(&pool, req.into_inner(), &config.aes_key).await {
        Ok(pending_tx) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(pending_tx),
            message: Some("Transaction created successfully and added to pending pool".to_string()),
        }),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn get_pending(pool: web::Data<DbPool>) -> HttpResponse {
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

    match crate::database::queries::get_pending_transactions(&client).await {
        Ok(transactions) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(transactions),
            message: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_transaction(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let tx_hash = path.into_inner();
    
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

    let result = client
        .query_opt(
            "SELECT id, transaction_hash, sender_wallet_id, receiver_wallet_id, amount, note, 
             signature, block_index, transaction_type, timestamp, created_at 
             FROM transactions WHERE transaction_hash = $1",
            &[&tx_hash],
        )
        .await;

    match result {
        Ok(Some(row)) => {
            let transaction = crate::models::Transaction {
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
            };
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(transaction),
                message: None,
            })
        }
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Transaction not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}
