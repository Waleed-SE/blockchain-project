use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub cnic: String,
    pub wallet_id: String,
    pub public_key: String,
    pub encrypted_private_key: String,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub wallet_id: String,
    pub user_id: Option<Uuid>,
    pub balance: f64,
    pub last_zakat_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXO {
    pub id: Uuid,
    pub wallet_id: String,
    pub amount: f64,
    pub transaction_hash: String,
    pub output_index: i32,
    pub is_spent: bool,
    pub created_at: DateTime<Utc>,
    pub spent_at: Option<DateTime<Utc>>,
    pub reserved_by: Option<Uuid>,  // Pending transaction ID that reserved this UTXO
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: i64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: i64,
    pub merkle_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_hash: String,
    pub sender_wallet_id: String,
    pub receiver_wallet_id: String,
    pub amount: f64,
    pub note: Option<String>,
    pub signature: String,
    pub block_index: Option<i64>,
    pub transaction_type: String,
    pub timestamp: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub id: Uuid,
    pub transaction_hash: String,
    pub sender_wallet_id: String,
    pub receiver_wallet_id: String,
    pub amount: f64,
    pub fee: f64,
    pub note: Option<String>,
    pub signature: String,
    pub timestamp: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beneficiary {
    pub id: Uuid,
    pub user_id: Uuid,
    pub beneficiary_wallet_id: String,
    pub nickname: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZakatRecord {
    pub id: Uuid,
    pub wallet_id: String,
    pub amount: f64,
    pub transaction_hash: Option<String>,
    pub deduction_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    pub id: Uuid,
    pub wallet_id: String,
    pub action: String,
    pub transaction_hash: Option<String>,
    pub block_hash: Option<String>,
    pub status: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLog {
    pub id: Uuid,
    pub log_type: String,
    pub user_id: Option<Uuid>,
    pub message: String,
    pub ip_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub full_name: String,
    pub cnic: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailOtp {
    pub id: Uuid,
    pub email: String,
    pub otp: String,
    pub is_verified: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String,
}

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub sender_wallet_id: String,
    pub receiver_wallet_id: String,
    pub amount: f64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddBeneficiaryRequest {
    pub beneficiary_wallet_id: String,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WalletBalance {
    pub wallet_id: String,
    pub balance: f64,
    pub utxo_count: i32,
}

#[derive(Debug, Serialize)]
pub struct BlockchainInfo {
    pub total_blocks: i64,
    pub latest_block: Option<Block>,
    pub pending_transactions: i32,
    pub total_transactions: i64,
    pub total_wallets: i64,
    pub mining_difficulty: i32,
    pub current_block_reward: f64,
    pub transaction_fee: f64,
}

#[derive(Debug, Serialize)]
pub struct MiningStats {
    pub current_block_height: i64,
    pub current_block_reward: f64,
    pub next_halving_block: i64,
    pub blocks_until_halving: i64,
    pub total_mined_coins: f64,
    pub max_coin_supply: f64,
    pub remaining_coins: f64,
    pub halving_interval: i32,
    pub percentage_mined: f64,
}

#[derive(Debug, Serialize)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
    pub wallet_id: String,
}
