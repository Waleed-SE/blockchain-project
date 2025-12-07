-- Complete Database Schema for Blockchain Wallet System
-- Supabase PostgreSQL Schema
-- Execute this in Supabase SQL Editor

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    cnic VARCHAR(20) UNIQUE NOT NULL,
    wallet_id VARCHAR(64) UNIQUE NOT NULL,
    public_key TEXT NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Wallets table
CREATE TABLE IF NOT EXISTS wallets (
    wallet_id VARCHAR(64) PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    balance DECIMAL(20, 8) DEFAULT 0,
    last_zakat_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Blocks table
CREATE TABLE IF NOT EXISTS blocks (
    index BIGINT PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    previous_hash VARCHAR(64) NOT NULL,
    hash VARCHAR(64) UNIQUE NOT NULL,
    nonce BIGINT NOT NULL,
    merkle_root VARCHAR(64),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Pending transactions table (must be created before utxos for foreign key)
CREATE TABLE IF NOT EXISTS pending_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_hash VARCHAR(64) UNIQUE NOT NULL,
    sender_wallet_id VARCHAR(64) NOT NULL,
    receiver_wallet_id VARCHAR(64) NOT NULL,
    amount DECIMAL(20, 8) NOT NULL CHECK (amount > 0),
    fee DECIMAL(20, 8) NOT NULL DEFAULT 0.1,
    note TEXT,
    signature TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- UTXOs table (with reserved_by column)
CREATE TABLE IF NOT EXISTS utxos (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_id VARCHAR(64) REFERENCES wallets(wallet_id) ON DELETE CASCADE,
    amount DECIMAL(20, 8) NOT NULL CHECK (amount > 0),
    transaction_hash VARCHAR(64) NOT NULL,
    output_index INTEGER NOT NULL,
    is_spent BOOLEAN DEFAULT FALSE,
    reserved_by UUID REFERENCES pending_transactions(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    spent_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(transaction_hash, output_index)
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    transaction_hash VARCHAR(64) UNIQUE NOT NULL,
    sender_wallet_id VARCHAR(64) REFERENCES wallets(wallet_id),
    receiver_wallet_id VARCHAR(64) REFERENCES wallets(wallet_id),
    amount DECIMAL(20, 8) NOT NULL CHECK (amount > 0),
    note TEXT,
    signature TEXT NOT NULL,
    block_index BIGINT REFERENCES blocks(index),
    transaction_type VARCHAR(50) DEFAULT 'transfer',
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Beneficiaries table
CREATE TABLE IF NOT EXISTS beneficiaries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    beneficiary_wallet_id VARCHAR(64) NOT NULL,
    nickname VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Zakat records table
CREATE TABLE IF NOT EXISTS zakat_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_id VARCHAR(64) REFERENCES wallets(wallet_id) ON DELETE CASCADE,
    amount DECIMAL(20, 8) NOT NULL CHECK (amount > 0),
    transaction_hash VARCHAR(64),
    deduction_date TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transaction logs table
CREATE TABLE IF NOT EXISTS transaction_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_id VARCHAR(64) REFERENCES wallets(wallet_id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    transaction_hash VARCHAR(64),
    block_hash VARCHAR(64),
    status VARCHAR(50) NOT NULL,
    ip_address VARCHAR(45),
    user_agent TEXT,
    note TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- System logs table
CREATE TABLE IF NOT EXISTS system_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    log_type VARCHAR(50) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    message TEXT NOT NULL,
    ip_address VARCHAR(45),
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Email OTP verification table
CREATE TABLE IF NOT EXISTS email_otps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL,
    otp VARCHAR(6) NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================
-- INDEXES FOR PERFORMANCE
-- ============================================

-- Users indexes
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_wallet_id ON users(wallet_id);

-- Wallets indexes
CREATE INDEX IF NOT EXISTS idx_wallets_user_id ON wallets(user_id);

-- UTXOs indexes
CREATE INDEX IF NOT EXISTS idx_utxos_wallet_id ON utxos(wallet_id);
CREATE INDEX IF NOT EXISTS idx_utxos_is_spent ON utxos(is_spent);
CREATE INDEX IF NOT EXISTS idx_utxos_transaction_hash ON utxos(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_utxos_reserved_by ON utxos(reserved_by);

-- Blocks indexes
CREATE INDEX IF NOT EXISTS idx_blocks_hash ON blocks(hash);

-- Transactions indexes
CREATE INDEX IF NOT EXISTS idx_transactions_hash ON transactions(transaction_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_sender ON transactions(sender_wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_receiver ON transactions(receiver_wallet_id);
CREATE INDEX IF NOT EXISTS idx_transactions_block ON transactions(block_index);

-- Pending transactions indexes
CREATE INDEX IF NOT EXISTS idx_pending_tx_hash ON pending_transactions(transaction_hash);

-- Transaction logs indexes
CREATE INDEX IF NOT EXISTS idx_transaction_logs_wallet ON transaction_logs(wallet_id);
CREATE INDEX IF NOT EXISTS idx_transaction_logs_created ON transaction_logs(created_at);

-- System logs indexes
CREATE INDEX IF NOT EXISTS idx_system_logs_type ON system_logs(log_type);
CREATE INDEX IF NOT EXISTS idx_system_logs_user ON system_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_system_logs_created ON system_logs(created_at);

-- Zakat records indexes
CREATE INDEX IF NOT EXISTS idx_zakat_records_wallet ON zakat_records(wallet_id);

-- Beneficiaries indexes
CREATE INDEX IF NOT EXISTS idx_beneficiaries_user ON beneficiaries(user_id);

-- Email OTPs indexes
CREATE INDEX IF NOT EXISTS idx_email_otps_email ON email_otps(email);
CREATE INDEX IF NOT EXISTS idx_email_otps_expires ON email_otps(expires_at);

-- ============================================
-- TRIGGERS
-- ============================================

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply triggers to tables with updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_wallets_updated_at BEFORE UPDATE ON wallets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================

COMMENT ON TABLE users IS 'Stores user account information including wallet credentials';
COMMENT ON TABLE wallets IS 'Stores wallet information and cached balances';
COMMENT ON TABLE utxos IS 'Unspent Transaction Outputs for UTXO-based balance calculation';
COMMENT ON TABLE blocks IS 'Blockchain blocks with proof of work';
COMMENT ON TABLE transactions IS 'Mined transactions included in blocks';
COMMENT ON TABLE pending_transactions IS 'Transactions waiting to be mined';
COMMENT ON TABLE zakat_records IS 'Monthly zakat deduction records';
COMMENT ON TABLE transaction_logs IS 'User transaction activity logs';
COMMENT ON TABLE system_logs IS 'System-wide activity and error logs';
COMMENT ON TABLE email_otps IS 'Stores OTP codes for email verification';
COMMENT ON COLUMN utxos.reserved_by IS 'UUID of pending transaction that has reserved this UTXO, NULL if not reserved';
