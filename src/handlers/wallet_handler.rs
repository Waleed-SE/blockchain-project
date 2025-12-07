use actix_web::{web, HttpResponse};
use crate::models::{ApiResponse, AddBeneficiaryRequest};
use crate::database::DbPool;
use crate::services::{wallet_service, zakat_service};
use crate::config::Config;
use uuid::Uuid;
use std::env;

pub async fn generate_wallet(_pool: web::Data<DbPool>) -> HttpResponse {
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

    match wallet_service::generate_wallet_keypair(&config.aes_key) {
        Ok(keypair) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(keypair),
            message: Some("Wallet generated successfully".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn get_wallet(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let wallet_id = path.into_inner();
    
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

    match crate::database::queries::get_wallet(&client, &wallet_id).await {
        Ok(Some(wallet)) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(wallet),
            message: None,
        }),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Wallet not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_balance(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let wallet_id = path.into_inner();

    match wallet_service::get_wallet_balance(&pool, &wallet_id).await {
        Ok(balance) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(balance),
            message: None,
        }),
        Err(e) => {
            // Return 404 if wallet not found, otherwise 400
            let mut status_code = if e.to_string().contains("Wallet not found") {
                HttpResponse::NotFound()
            } else {
                HttpResponse::BadRequest()
            };
            status_code.json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(e.to_string()),
            })
        }
    }
}

pub async fn get_utxos(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let wallet_id = path.into_inner();
    
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

    match crate::database::queries::get_unspent_utxos(&client, &wallet_id).await {
        Ok(utxos) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(utxos),
            message: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_transactions(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let wallet_id = path.into_inner();
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

    match crate::database::queries::get_wallet_transactions(&client, &wallet_id, limit, offset).await {
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

pub async fn get_beneficiaries(
    pool: web::Data<DbPool>,
    req: actix_web::HttpRequest,
) -> HttpResponse {
    // Extract user_id from JWT token
    let token = match req.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(t) => t.trim_start_matches("Bearer ").to_string(),
            Err(_) => {
                return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some("Invalid authorization header".to_string()),
                });
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("No authorization token provided".to_string()),
            });
        }
    };

    let claims = match crate::services::auth_service::verify_token(&token) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid or expired token".to_string()),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid user ID in token".to_string()),
            });
        }
    };

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

    match crate::database::queries::get_user_beneficiaries(&client, user_id).await {
        Ok(beneficiaries) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(beneficiaries),
            message: None,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Failed to fetch beneficiaries: {}", e)),
        }),
    }
}

pub async fn add_beneficiary(
    pool: web::Data<DbPool>,
    req: actix_web::HttpRequest,
    body: web::Json<AddBeneficiaryRequest>,
) -> HttpResponse {
    // Extract user_id from JWT token
    let token = match req.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(t) => t.trim_start_matches("Bearer ").to_string(),
            Err(_) => {
                return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some("Invalid authorization header".to_string()),
                });
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("No authorization token provided".to_string()),
            });
        }
    };

    let claims = match crate::services::auth_service::verify_token(&token) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid or expired token".to_string()),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid user ID in token".to_string()),
            });
        }
    };

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

    // Verify the beneficiary wallet exists
    match crate::database::queries::get_wallet(&client, &body.beneficiary_wallet_id).await {
        Ok(None) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Beneficiary wallet not found".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Failed to verify wallet: {}", e)),
            });
        }
        Ok(Some(_)) => {}
    }

    match crate::database::queries::add_beneficiary(
        &client,
        user_id,
        &body.beneficiary_wallet_id,
        body.nickname.clone(),
    )
    .await
    {
        Ok(beneficiary) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(beneficiary),
            message: Some("Beneficiary added successfully".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Failed to add beneficiary: {}", e)),
        }),
    }
}

pub async fn delete_beneficiary(
    pool: web::Data<DbPool>,
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let beneficiary_id = path.into_inner();

    // Extract user_id from JWT token
    let token = match req.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(t) => t.trim_start_matches("Bearer ").to_string(),
            Err(_) => {
                return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some("Invalid authorization header".to_string()),
                });
            }
        },
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("No authorization token provided".to_string()),
            });
        }
    };

    let claims = match crate::services::auth_service::verify_token(&token) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid or expired token".to_string()),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("Invalid user ID in token".to_string()),
            });
        }
    };

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

    match crate::database::queries::delete_beneficiary(&client, beneficiary_id, user_id).await {
        Ok(rows) if rows > 0 => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({"deleted": rows})),
            message: Some("Beneficiary deleted successfully".to_string()),
        }),
        Ok(_) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Beneficiary not found or not owned by user".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Failed to delete beneficiary: {}", e)),
        }),
    }
}

pub async fn get_zakat_records(
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let wallet_id = match query.get("wallet_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("wallet_id is required".to_string()),
            });
        }
    };

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
        .query(
            "SELECT id, wallet_id, amount::float8, transaction_hash, deduction_date, created_at 
             FROM zakat_records WHERE wallet_id = $1 ORDER BY deduction_date DESC",
            &[&wallet_id],
        )
        .await;

    match result {
        Ok(rows) => {
            let records: Vec<crate::models::ZakatRecord> = rows
                .iter()
                .map(|row| crate::models::ZakatRecord {
                    id: row.get(0),
                    wallet_id: row.get(1),
                    amount: row.get(2),
                    transaction_hash: row.get(3),
                    deduction_date: row.get(4),
                    created_at: row.get(5),
                })
                .collect();
            
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(records),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_zakat_pool(pool: web::Data<DbPool>) -> HttpResponse {
    let zakat_pool_wallet_id = env::var("ZAKAT_POOL_WALLET_ID")
        .unwrap_or_else(|_| "ZAKAT_POOL".to_string());

    match wallet_service::get_wallet_balance(&pool, &zakat_pool_wallet_id).await {
        Ok(balance) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(balance),
            message: None,
        }),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn trigger_zakat(pool: web::Data<DbPool>) -> HttpResponse {
    match zakat_service::trigger_zakat_deduction(&pool).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({"message": "Zakat deduction triggered"})),
            message: Some("Zakat deduction process completed".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Zakat deduction failed: {}", e)),
        }),
    }
}
