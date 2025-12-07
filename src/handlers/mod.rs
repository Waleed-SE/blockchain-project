pub mod auth_handler;
pub mod wallet_handler;
pub mod transaction_handler;
pub mod blockchain_handler;
pub mod logs_handler;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth_handler::register))
                    .route("/login", web::post().to(auth_handler::login))
                    .route("/send-otp", web::post().to(auth_handler::send_otp))
                    .route("/verify-otp", web::post().to(auth_handler::verify_otp))
                    .route("/profile", web::get().to(auth_handler::get_profile))
                    .route("/profile", web::put().to(auth_handler::update_profile))
            )
            .service(
                web::scope("/wallet")
                    .route("/generate", web::post().to(wallet_handler::generate_wallet))
                    .route("/{wallet_id}", web::get().to(wallet_handler::get_wallet))
                    .route("/{wallet_id}/balance", web::get().to(wallet_handler::get_balance))
                    .route("/{wallet_id}/utxos", web::get().to(wallet_handler::get_utxos))
                    .route("/{wallet_id}/transactions", web::get().to(wallet_handler::get_transactions))
            )
            .service(
                web::scope("/transaction")
                    .route("/create", web::post().to(transaction_handler::create_transaction))
                    .route("/pending", web::get().to(transaction_handler::get_pending))
                    .route("/{tx_hash}", web::get().to(transaction_handler::get_transaction))
            )
            .service(
                web::scope("/blockchain")
                    .route("/blocks", web::get().to(blockchain_handler::get_blocks))
                    .route("/block/{index}", web::get().to(blockchain_handler::get_block))
                    .route("/validate", web::get().to(blockchain_handler::validate_chain))
                    .route("/mine", web::post().to(blockchain_handler::mine_block))
                    .route("/info", web::get().to(blockchain_handler::get_info))
                    .route("/mining-stats", web::get().to(blockchain_handler::get_mining_stats))
            )
            .service(
                web::scope("/beneficiaries")
                    .route("", web::get().to(wallet_handler::get_beneficiaries))
                    .route("", web::post().to(wallet_handler::add_beneficiary))
                    .route("/{id}", web::delete().to(wallet_handler::delete_beneficiary))
            )
            .service(
                web::scope("/zakat")
                    .route("/records", web::get().to(wallet_handler::get_zakat_records))
                    .route("/pool", web::get().to(wallet_handler::get_zakat_pool))
                    .route("/trigger", web::post().to(wallet_handler::trigger_zakat))
            )
            .service(
                web::scope("/logs")
                    .route("/transaction", web::get().to(logs_handler::get_transaction_logs))
                    .route("/system", web::get().to(logs_handler::get_system_logs))
            )
            .service(
                web::scope("/reports")
                    .route("/monthly/{wallet_id}", web::get().to(logs_handler::get_monthly_report))
                    .route("/analytics", web::get().to(logs_handler::get_analytics))
            )
    );
}
