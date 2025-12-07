use crate::models::{RegisterRequest, User};
use crate::database::{DbPool, queries};
use crate::services::wallet_service::generate_wallet_keypair;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;
use std::ops::DerefMut;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: i64,
}

#[derive(Debug)]
pub enum AuthError {
    UserAlreadyExists,
    InvalidCredentials,
    TokenError(String),
    DatabaseError(String),
    WalletError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthError::UserAlreadyExists => write!(f, "User already exists"),
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::TokenError(msg) => write!(f, "Token error: {}", msg),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AuthError::WalletError(msg) => write!(f, "Wallet error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// Register a new user
pub async fn register_user(
    pool: &DbPool,
    req: RegisterRequest,
    aes_key: &[u8],
) -> Result<User, AuthError> {
    let mut client = pool.get().await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Start a transaction on the underlying tokio-postgres client
    let transaction = client.deref_mut().transaction().await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

    // Check if user already exists
    let existing_user_check = transaction
        .query_opt(
            "SELECT id FROM users WHERE email = $1",
            &[&req.email],
        )
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if existing_user_check.is_some() {
        return Err(AuthError::UserAlreadyExists);
    }

    // Generate wallet keypair
    let keypair = generate_wallet_keypair(aes_key)
        .map_err(|e| AuthError::WalletError(e.to_string()))?;

    // Create user
    let user_row = transaction
        .query_one(
            "INSERT INTO users (email, full_name, cnic, wallet_id, public_key, encrypted_private_key) 
             VALUES ($1, $2, $3, $4, $5, $6) 
             RETURNING id, email, full_name, cnic, wallet_id, public_key, encrypted_private_key, is_verified, created_at, updated_at",
            &[&req.email, &req.full_name, &req.cnic, &keypair.wallet_id, &keypair.public_key, &keypair.private_key],
        )
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to create user: {}", e)))?;

    let user = User {
        id: user_row.get(0),
        email: user_row.get(1),
        full_name: user_row.get(2),
        cnic: user_row.get(3),
        wallet_id: user_row.get(4),
        public_key: user_row.get(5),
        encrypted_private_key: user_row.get(6),
        is_verified: user_row.get(7),
        created_at: user_row.get(8),
        updated_at: user_row.get(9),
    };

    // Create wallet
    transaction
        .execute(
            "INSERT INTO wallets (wallet_id, user_id, balance) VALUES ($1, $2, 0)",
            &[&keypair.wallet_id, &user.id],
        )
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to create wallet: {}", e)))?;

    // Log registration
    transaction
        .execute(
            "INSERT INTO system_logs (log_type, user_id, message) VALUES ($1, $2, $3)",
            &[&"user_registration", &user.id, &format!("New user registered: {}", req.email)],
        )
        .await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to create log: {}", e)))?;

    // Commit the transaction
    transaction.commit().await
        .map_err(|e| AuthError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

    log::info!("✅ User registered: {} (wallet: {})", req.email, keypair.wallet_id);

    Ok(user)
}

/// Generate JWT token
pub fn generate_token(user_id: &str, email: &str) -> Result<String, AuthError> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|e| AuthError::TokenError(e.to_string()))
}

/// Verify JWT token
pub fn verify_token(token: &str) -> Result<Claims, AuthError> {
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AuthError::TokenError(e.to_string()))
}
