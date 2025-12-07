use crate::database::{DbPool, queries};
use chrono::{Utc, Duration};
use rand::Rng;
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::env;

#[derive(Debug)]
pub enum OtpError {
    DatabaseError(String),
    InvalidOtp,
    ExpiredOtp,
    SendError(String),
}

impl std::fmt::Display for OtpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OtpError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            OtpError::InvalidOtp => write!(f, "Invalid or already used OTP"),
            OtpError::ExpiredOtp => write!(f, "OTP has expired"),
            OtpError::SendError(msg) => write!(f, "Failed to send OTP: {}", msg),
        }
    }
}

impl std::error::Error for OtpError {}

/// Generate a 6-digit OTP
pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..=999999))
}

/// Send email with OTP
async fn send_email(to_email: &str, otp: &str) -> Result<(), String> {
    let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);
    let smtp_username = env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set in .env")?;
    let smtp_password = env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set in .env")?;
    let from_email = env::var("SMTP_FROM_EMAIL").unwrap_or_else(|_| smtp_username.clone());
    let from_name = env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "BlockWallet".to_string());

    // Create email body
    let html_body = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; padding: 20px; }}
                .container {{ max-width: 600px; margin: 0 auto; background-color: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                .header {{ text-align: center; margin-bottom: 30px; }}
                .header h1 {{ color: #4F46E5; margin: 0; }}
                .otp-code {{ font-size: 32px; font-weight: bold; color: #4F46E5; text-align: center; padding: 20px; background-color: #F3F4F6; border-radius: 8px; letter-spacing: 8px; margin: 20px 0; }}
                .content {{ color: #374151; line-height: 1.6; }}
                .footer {{ margin-top: 30px; padding-top: 20px; border-top: 1px solid #E5E7EB; text-align: center; color: #6B7280; font-size: 14px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="header">
                    <h1>🔗 BlockWallet</h1>
                </div>
                <div class="content">
                    <h2>Email Verification</h2>
                    <p>Hello,</p>
                    <p>Thank you for registering with BlockWallet. Please use the following One-Time Password (OTP) to verify your email address:</p>
                    <div class="otp-code">{}</div>
                    <p><strong>This code will expire in 10 minutes.</strong></p>
                    <p>If you didn't request this verification code, please ignore this email.</p>
                </div>
                <div class="footer">
                    <p>This is an automated email. Please do not reply.</p>
                    <p>&copy; 2025 BlockWallet. All rights reserved.</p>
                </div>
            </div>
        </body>
        </html>
        "#,
        otp
    );

    // Build email
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to_email.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("BlockWallet - Email Verification Code")
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| format!("Failed to build email: {}", e))?;

    // Create SMTP credentials
    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());

    // Create SMTP transport with STARTTLS
    let mailer = SmtpTransport::starttls_relay(&smtp_host)
        .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
        .credentials(creds)
        .port(smtp_port)
        .timeout(Some(std::time::Duration::from_secs(10)))
        .build();

    // Send email
    mailer.send(&email)
        .map_err(|e| format!("Failed to send email: {}", e))?;

    log::info!("✅ Email sent successfully to {}", to_email);
    
    Ok(())
}

/// Send OTP to email and store in database
pub async fn send_otp(pool: &DbPool, email: &str) -> Result<String, OtpError> {
    let client = pool.get().await
        .map_err(|e| OtpError::DatabaseError(e.to_string()))?;

    // Generate OTP
    let otp = generate_otp();
    let expires_at = Utc::now() + Duration::minutes(10);

    // Store OTP in database
    queries::create_otp(&client, email, &otp, expires_at)
        .await
        .map_err(|e| OtpError::DatabaseError(e.to_string()))?;

    // Send email with OTP
    send_email(email, &otp)
        .await
        .map_err(|e| OtpError::SendError(e))?;

    log::info!("📧 OTP sent to {}", email);

    Ok(otp) // In production, consider not returning OTP for security
}

/// Verify OTP and mark user as verified
pub async fn verify_otp(pool: &DbPool, email: &str, otp: &str) -> Result<(), OtpError> {
    let client = pool.get().await
        .map_err(|e| OtpError::DatabaseError(e.to_string()))?;

    // Verify OTP
    let is_valid = queries::verify_otp(&client, email, otp)
        .await
        .map_err(|e| OtpError::DatabaseError(e.to_string()))?;

    if !is_valid {
        return Err(OtpError::InvalidOtp);
    }

    // Mark user as verified
    queries::mark_user_verified(&client, email)
        .await
        .map_err(|e| OtpError::DatabaseError(e.to_string()))?;

    log::info!("✅ Email verified: {}", email);

    Ok(())
}
