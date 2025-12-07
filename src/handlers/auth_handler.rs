use actix_web::{web, HttpResponse, HttpRequest};
use crate::models::{RegisterRequest, LoginRequest, VerifyOtpRequest, SendOtpRequest, ApiResponse};
use crate::database::DbPool;
use crate::services::{auth_service, otp_service};
use crate::config::Config;

pub async fn register(
    pool: web::Data<DbPool>,
    req: web::Json<RegisterRequest>,
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

    match auth_service::register_user(&pool, req.into_inner(), &config.aes_key).await {
        Ok(user) => {
            match auth_service::generate_token(&user.id.to_string(), &user.email) {
                Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "data": {
                        "user": user,
                        "token": token
                    },
                    "message": "User registered successfully"
                })),
                Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some(format!("Token generation failed: {}", e)),
                }),
            }
        }
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn login(
    pool: web::Data<DbPool>,
    req: web::Json<LoginRequest>,
) -> HttpResponse {
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

    match crate::database::queries::find_user_by_email(&client, &req.email).await {
        Ok(Some(user)) => {
            // Note: In production, you should verify password hash here
            match auth_service::generate_token(&user.id.to_string(), &user.email) {
                Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "data": {
                        "user": user,
                        "token": token
                    },
                    "message": "Login successful"
                })),
                Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some(format!("Token generation failed: {}", e)),
                }),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Invalid credentials".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn get_profile(
    pool: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    // Extract JWT from Authorization header
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("No token provided".to_string()),
            });
        }
    };

    let claims = match auth_service::verify_token(&token) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Invalid token: {}", e)),
            });
        }
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Invalid user ID: {}", e)),
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

    match crate::database::queries::find_user_by_id(&client, user_id).await {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(user),
            message: Some("Profile retrieved successfully".to_string()),
        }),
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("User not found".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Database error: {}", e)),
        }),
    }
}

pub async fn send_otp(
    pool: web::Data<DbPool>,
    req: web::Json<SendOtpRequest>,
) -> HttpResponse {
    match otp_service::send_otp(&pool, &req.email).await {
        Ok(otp) => {
            // In production, don't send OTP in response
            // This is only for testing/development
            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "message": "OTP sent successfully",
                    "otp": otp // Remove in production!
                })),
                message: Some("Check your email for verification code".to_string()),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn verify_otp(
    pool: web::Data<DbPool>,
    req: web::Json<VerifyOtpRequest>,
) -> HttpResponse {
    match otp_service::verify_otp(&pool, &req.email, &req.otp).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({"verified": true})),
            message: Some("Email verified successfully".to_string()),
        }),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(e.to_string()),
        }),
    }
}

pub async fn update_profile(
    pool: web::Data<DbPool>,
    req: HttpRequest,
    body: web::Json<crate::models::UpdateProfileRequest>,
) -> HttpResponse {
    // Extract JWT token
    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            return HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("No authorization token provided".to_string()),
            });
        }
    };

    // Verify token
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

    // Get current user
    let current_user = match crate::database::queries::find_user_by_id(&client, user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some("User not found".to_string()),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: Some(format!("Database error: {}", e)),
            });
        }
    };

    let mut updates = Vec::new();
    let params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
    let mut param_count = 1;

    // Build dynamic UPDATE query
    if let Some(ref full_name) = body.full_name {
        updates.push(format!("full_name = ${}", param_count));
        param_count += 1;
    }

    if let Some(ref email) = body.email {
        if email != &current_user.email {
            // Check if email already exists
            let email_exists = client
                .query_opt("SELECT id FROM users WHERE email = $1 AND id != $2", &[email, &user_id])
                .await;

            if let Ok(Some(_)) = email_exists {
                return HttpResponse::BadRequest().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: Some("Email already in use".to_string()),
                });
            }

            updates.push(format!("email = ${}", param_count));
            updates.push(format!("is_verified = ${}", param_count + 1));
            param_count += 2;
        }
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("No fields to update".to_string()),
        });
    }

    updates.push(format!("updated_at = ${}", param_count));

    let query = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING id, email, full_name, cnic, wallet_id, public_key, encrypted_private_key, is_verified, created_at, updated_at",
        updates.join(", "),
        param_count + 1
    );

    // Build params vector
    let mut query_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = vec![];
    
    if let Some(ref full_name) = body.full_name {
        query_params.push(Box::new(full_name.clone()));
    }
    
    if let Some(ref email) = body.email {
        if email != &current_user.email {
            query_params.push(Box::new(email.clone()));
            query_params.push(Box::new(false)); // Reset is_verified
        }
    }
    
    query_params.push(Box::new(chrono::Utc::now()));
    query_params.push(Box::new(user_id));

    let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = query_params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    match client.query_one(&query, &params_refs).await {
        Ok(row) => {
            let updated_user = crate::models::User {
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
            };

            HttpResponse::Ok().json(ApiResponse {
                success: true,
                data: Some(serde_json::json!({
                    "id": updated_user.id,
                    "email": updated_user.email,
                    "full_name": updated_user.full_name,
                    "wallet_id": updated_user.wallet_id,
                    "public_key": updated_user.public_key,
                    "is_verified": updated_user.is_verified,
                })),
                message: Some("Profile updated successfully".to_string()),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some(format!("Failed to update profile: {}", e)),
        }),
    }
}

fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}
