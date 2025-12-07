mod models;
mod services;
mod handlers;
mod crypto;
mod blockchain;
mod database;
mod utils;
mod middleware;
mod config;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("{}:{}", host, port);

    log::info!("🚀 Starting Blockchain Wallet Backend on {}", address);

    // Initialize database pool
    let db_pool = database::create_pool().await.expect("Failed to create database pool");

    // Initialize blockchain
    blockchain::initialize_blockchain(db_pool.clone())
        .await
        .expect("Failed to initialize blockchain");

    // Start Zakat scheduler
    tokio::spawn(services::zakat_service::start_zakat_scheduler(db_pool.clone()));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                let allowed_origins = env::var("ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:5173".to_string());
                let origins: Vec<&str> = allowed_origins.split(',').collect();
                origins.iter().any(|&o| origin.as_bytes() == o.as_bytes())
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .configure(handlers::configure_routes)
    })
    .bind(address)?
    .run()
    .await
}
