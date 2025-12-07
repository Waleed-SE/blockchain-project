use actix_web::{web, HttpResponse};
use crate::models::ApiResponse;
use crate::database::DbPool;

pub async fn get_transaction_logs(
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

    let result = client
        .query(
            "SELECT id, wallet_id, action, transaction_hash, block_hash, status, ip_address, 
             user_agent, note, created_at 
             FROM transaction_logs 
             WHERE wallet_id = $1 
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            &[&wallet_id, &limit, &offset],
        )
        .await;

    match result {
        Ok(rows) => {
            let logs: Vec<crate::models::TransactionLog> = rows
                .iter()
                .map(|row| crate::models::TransactionLog {
                    id: row.get(0),
                    wallet_id: row.get(1),
                    action: row.get(2),
                    transaction_hash: row.get(3),
                    block_hash: row.get(4),
                    status: row.get(5),
                    ip_address: row.get(6),
                    user_agent: row.get(7),
                    note: row.get(8),
                    created_at: row.get(9),
                })
                .collect();

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(logs),
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

pub async fn get_system_logs(
    pool: web::Data<DbPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(100);
    let offset = query.get("offset").and_then(|o| o.parse().ok()).unwrap_or(0);
    let log_type = query.get("type");

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

    let result = if let Some(lt) = log_type {
        client
            .query(
                "SELECT id, log_type, user_id, message, ip_address, metadata, created_at 
                 FROM system_logs 
                 WHERE log_type = $1 
                 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                &[&lt, &limit, &offset],
            )
            .await
    } else {
        client
            .query(
                "SELECT id, log_type, user_id, message, ip_address, metadata, created_at 
                 FROM system_logs 
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                &[&limit, &offset],
            )
            .await
    };

    match result {
        Ok(rows) => {
            let logs: Vec<crate::models::SystemLog> = rows
                .iter()
                .map(|row| crate::models::SystemLog {
                    id: row.get(0),
                    log_type: row.get(1),
                    user_id: row.get(2),
                    message: row.get(3),
                    ip_address: row.get(4),
                    metadata: row.get(5),
                    created_at: row.get(6),
                })
                .collect();

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(logs),
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

pub async fn get_monthly_report(
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

    // Get monthly statistics
    let result = client
        .query_one(
            "SELECT 
                COUNT(*) as total_transactions,
                COALESCE(SUM(CASE WHEN sender_wallet_id = $1 THEN amount ELSE 0 END)::float8, 0) as total_sent,
                COALESCE(SUM(CASE WHEN receiver_wallet_id = $1 THEN amount ELSE 0 END)::float8, 0) as total_received
             FROM transactions 
             WHERE (sender_wallet_id = $1 OR receiver_wallet_id = $1)
             AND created_at >= NOW() - INTERVAL '30 days'",
            &[&wallet_id],
        )
        .await;

    // Get zakat paid in last 30 days
    let zakat_result = client
        .query_one(
            "SELECT COALESCE(SUM(amount)::float8, 0) as total_zakat
             FROM zakat_records
             WHERE wallet_id = $1
             AND deduction_date >= NOW() - INTERVAL '30 days'",
            &[&wallet_id],
        )
        .await;

    // Get current wallet balance
    let wallet_result = client
        .query_opt(
            "SELECT balance::float8 FROM wallets WHERE wallet_id = $1",
            &[&wallet_id],
        )
        .await;

    // Get total transactions count (all time)
    let all_time_tx_result = client
        .query_one(
            "SELECT COUNT(*) FROM transactions WHERE sender_wallet_id = $1 OR receiver_wallet_id = $1",
            &[&wallet_id],
        )
        .await;

    match (result, zakat_result, wallet_result, all_time_tx_result) {
        (Ok(row), Ok(zakat_row), Ok(wallet_row), Ok(all_tx_row)) => {
            let transaction_count: i64 = row.get(0);
            let total_sent: f64 = row.get::<_, Option<f64>>(1).unwrap_or(0.0);
            let total_received: f64 = row.get::<_, Option<f64>>(2).unwrap_or(0.0);
            let zakat_paid: f64 = zakat_row.get::<_, Option<f64>>(0).unwrap_or(0.0);
            let current_balance: f64 = wallet_row.and_then(|r| r.get::<_, Option<f64>>(0)).unwrap_or(0.0);
            let all_time_transactions: i64 = all_tx_row.get(0);

            let report = serde_json::json!({
                "wallet_id": wallet_id,
                "period": "30_days",
                "transaction_count": transaction_count,
                "total_sent": total_sent,
                "total_received": total_received,
                "net_change": total_received - total_sent,
                "zakat_paid": zakat_paid,
                "current_balance": current_balance,
                "all_time_transactions": all_time_transactions,
            });

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(report),
                message: None,
            })
        }
        _ => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Failed to fetch report data".to_string()),
        }),
    }
}

pub async fn get_analytics(pool: web::Data<DbPool>) -> HttpResponse {
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

    let blocks_result = client.query_one("SELECT COUNT(*) FROM blocks", &[]).await;
    let transactions_result = client.query_one("SELECT COUNT(*) FROM transactions", &[]).await;
    let wallets_result = client.query_one("SELECT COUNT(*) FROM wallets", &[]).await;
    let users_result = client.query_one("SELECT COUNT(*) FROM users", &[]).await;

    match (blocks_result, transactions_result, wallets_result, users_result) {
        (Ok(b), Ok(t), Ok(w), Ok(u)) => {
            let analytics = serde_json::json!({
                "total_blocks": b.get::<_, i64>(0),
                "total_transactions": t.get::<_, i64>(0),
                "total_wallets": w.get::<_, i64>(0),
                "total_users": u.get::<_, i64>(0),
            });

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(analytics),
                message: None,
            })
        }
        _ => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Failed to retrieve analytics".to_string()),
        }),
    }
}
