-- Drop All Tables for Blockchain Wallet System
-- WARNING: This will delete ALL data permanently!
-- Execute this in Supabase SQL Editor only if you want to completely reset the database

-- Drop tables in reverse order of dependencies to avoid foreign key conflicts

DROP TABLE IF EXISTS system_logs CASCADE;
DROP TABLE IF EXISTS transaction_logs CASCADE;
DROP TABLE IF EXISTS zakat_records CASCADE;
DROP TABLE IF EXISTS beneficiaries CASCADE;
DROP TABLE IF EXISTS email_otps CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS utxos CASCADE;
DROP TABLE IF EXISTS pending_transactions CASCADE;
DROP TABLE IF EXISTS blocks CASCADE;
DROP TABLE IF EXISTS wallets CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- Drop the trigger function
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Optional: Drop the UUID extension (uncomment if you want to remove it)
-- DROP EXTENSION IF EXISTS "uuid-ossp";

-- Confirmation message
DO $$
BEGIN
    RAISE NOTICE 'All tables have been dropped successfully.';
END $$;
