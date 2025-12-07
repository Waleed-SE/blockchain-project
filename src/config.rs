use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub aes_key: Vec<u8>,
    pub mining_difficulty: usize,
    pub block_reward: f64,
    pub zakat_percentage: f64,
    pub zakat_pool_wallet_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let aes_key_hex = env::var("AES_ENCRYPTION_KEY")?;
        let aes_key = hex::decode(aes_key_hex)?;

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            aes_key,
            mining_difficulty: env::var("MINING_DIFFICULTY")
                .unwrap_or_else(|_| "5".to_string())
                .parse()?,
            block_reward: env::var("BLOCK_REWARD")
                .unwrap_or_else(|_| "50.0".to_string())
                .parse()?,
            zakat_percentage: env::var("ZAKAT_PERCENTAGE")
                .unwrap_or_else(|_| "2.5".to_string())
                .parse()?,
            zakat_pool_wallet_id: env::var("ZAKAT_POOL_WALLET_ID")
                .unwrap_or_else(|_| "ZAKAT_POOL".to_string()),
        })
    }
}
