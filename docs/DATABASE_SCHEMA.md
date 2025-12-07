# Database Schema - Blockchain Wallet

## 📊 Database Overview

**Database System**: PostgreSQL (Supabase)  
**Region**: Asia Pacific  
**Connection Mode**: Transaction (Port 6543)  
**Max Connections**: 100 (production)

---

## 🗂️ Complete Database Schema

```sql
-- ============================================
-- USERS TABLE
-- ============================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    cnic VARCHAR(20),
    wallet_id VARCHAR(255) UNIQUE NOT NULL,
    public_key TEXT NOT NULL,
    encrypted_private_key TEXT NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- WALLETS TABLE
-- ============================================
CREATE TABLE wallets (
    wallet_id VARCHAR(255) PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE,
    balance NUMERIC(20, 8) DEFAULT 0,
    public_key TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- ============================================
-- TRANSACTIONS TABLE
-- ============================================
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_hash VARCHAR(255) UNIQUE NOT NULL,
    sender_wallet_id VARCHAR(255) NOT NULL,
    recipient_wallet_id VARCHAR(255),
    amount NUMERIC(20, 8) NOT NULL,
    fee NUMERIC(20, 8) NOT NULL DEFAULT 0,
    timestamp TIMESTAMP NOT NULL,
    signature TEXT NOT NULL,
    status VARCHAR(50) DEFAULT 'pending',
    block_id UUID,
    block_height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sender_wallet_id) REFERENCES wallets(wallet_id),
    FOREIGN KEY (recipient_wallet_id) REFERENCES wallets(wallet_id)
);

-- ============================================
-- BLOCKS TABLE
-- ============================================
CREATE TABLE blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    block_hash VARCHAR(255) UNIQUE NOT NULL,
    previous_hash VARCHAR(255),
    height INTEGER UNIQUE NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    nonce BIGINT NOT NULL,
    difficulty INTEGER NOT NULL,
    miner_wallet_id VARCHAR(255),
    miner_reward NUMERIC(20, 8),
    transaction_count INTEGER DEFAULT 0,
    merkle_root VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (miner_wallet_id) REFERENCES wallets(wallet_id)
);

-- ============================================
-- UTXOS TABLE (Unspent Transaction Outputs)
-- ============================================
CREATE TABLE utxos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_hash VARCHAR(255) NOT NULL,
    output_index INTEGER NOT NULL,
    wallet_id VARCHAR(255) NOT NULL,
    amount NUMERIC(20, 8) NOT NULL,
    is_spent BOOLEAN DEFAULT FALSE,
    spent_in_tx_hash VARCHAR(255),
    block_height INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(transaction_hash, output_index),
    FOREIGN KEY (wallet_id) REFERENCES wallets(wallet_id),
    FOREIGN KEY (transaction_hash) REFERENCES transactions(transaction_hash)
);

-- ============================================
-- BENEFICIARIES TABLE
-- ============================================
CREATE TABLE beneficiaries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    beneficiary_wallet_id VARCHAR(255) NOT NULL,
    beneficiary_name VARCHAR(255),
    nickname VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (beneficiary_wallet_id) REFERENCES wallets(wallet_id),
    UNIQUE(user_id, beneficiary_wallet_id)
);

-- ============================================
-- OTP CODES TABLE
-- ============================================
CREATE TABLE otp_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    otp_code VARCHAR(6) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    is_used BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- BLOCKCHAIN INFO TABLE
-- ============================================
CREATE TABLE blockchain_info (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    total_coins NUMERIC(20, 8) DEFAULT 21000000,
    circulating_coins NUMERIC(20, 8) DEFAULT 0,
    total_transactions BIGINT DEFAULT 0,
    total_blocks INTEGER DEFAULT 1,
    current_difficulty INTEGER DEFAULT 3,
    average_block_time NUMERIC(10, 2) DEFAULT 10,
    halving_height INTEGER DEFAULT 5,
    current_halving_number INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- LOGS TABLE
-- ============================================
CREATE TABLE logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);
```

---

## 📋 Table Details

### 1. USERS Table

**Purpose**: Store user account information

| Column                | Type         | Constraints      | Description                       |
| --------------------- | ------------ | ---------------- | --------------------------------- |
| id                    | UUID         | PK, AUTO         | Unique user identifier            |
| email                 | VARCHAR(255) | UNIQUE, NOT NULL | User's email address              |
| full_name             | VARCHAR(255) | NOT NULL         | User's full name                  |
| cnic                  | VARCHAR(20)  |                  | National ID number                |
| wallet_id             | VARCHAR(255) | UNIQUE, NOT NULL | SHA256 hash as wallet ID          |
| public_key            | TEXT         | NOT NULL         | RSA public key (PEM)              |
| encrypted_private_key | TEXT         | NOT NULL         | AES-256-GCM encrypted private key |
| is_verified           | BOOLEAN      | DEFAULT FALSE    | Email verification status         |
| created_at            | TIMESTAMP    | DEFAULT NOW      | Account creation time             |
| updated_at            | TIMESTAMP    | DEFAULT NOW      | Last update time                  |

**Indexes**:

```sql
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_wallet_id ON users(wallet_id);
```

**Sample Data**:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "full_name": "John Doe",
  "cnic": "12345-6789012-3",
  "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
  "public_key": "-----BEGIN PUBLIC KEY-----...",
  "encrypted_private_key": "base64_encrypted_data",
  "is_verified": true,
  "created_at": "2024-12-07T10:30:00Z"
}
```

---

### 2. WALLETS Table

**Purpose**: Store blockchain wallet information

| Column     | Type          | Constraints | Description               |
| ---------- | ------------- | ----------- | ------------------------- |
| wallet_id  | VARCHAR(255)  | PK          | SHA256 hash of public key |
| user_id    | UUID          | UNIQUE, FK  | Reference to user         |
| balance    | NUMERIC(20,8) | DEFAULT 0   | Current wallet balance    |
| public_key | TEXT          | NOT NULL    | RSA-2048 public key       |
| created_at | TIMESTAMP     | DEFAULT NOW | Wallet creation time      |
| updated_at | TIMESTAMP     | DEFAULT NOW | Last update time          |

**Indexes**:

```sql
CREATE INDEX idx_wallets_user_id ON wallets(user_id);
CREATE INDEX idx_wallets_balance ON wallets(balance);
```

**Sample Data**:

```json
{
  "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "balance": "1500.50000000",
  "public_key": "-----BEGIN PUBLIC KEY-----...",
  "created_at": "2024-12-07T10:30:00Z"
}
```

---

### 3. TRANSACTIONS Table

**Purpose**: Store all blockchain transactions

| Column              | Type          | Constraints       | Description                |
| ------------------- | ------------- | ----------------- | -------------------------- |
| id                  | UUID          | PK, AUTO          | Internal transaction ID    |
| transaction_hash    | VARCHAR(255)  | UNIQUE, NOT NULL  | SHA256 hash of transaction |
| sender_wallet_id    | VARCHAR(255)  | NOT NULL, FK      | Sender's wallet ID         |
| recipient_wallet_id | VARCHAR(255)  | FK                | Recipient's wallet ID      |
| amount              | NUMERIC(20,8) | NOT NULL          | Transaction amount         |
| fee                 | NUMERIC(20,8) | DEFAULT 0         | Transaction fee            |
| timestamp           | TIMESTAMP     | NOT NULL          | Transaction time           |
| signature           | TEXT          | NOT NULL          | RSA signature              |
| status              | VARCHAR(50)   | DEFAULT 'pending' | pending/confirmed/failed   |
| block_id            | UUID          | FK                | Reference to block         |
| block_height        | INTEGER       |                   | Height in blockchain       |
| created_at          | TIMESTAMP     | DEFAULT NOW       | Record creation time       |

**Indexes**:

```sql
CREATE INDEX idx_tx_hash ON transactions(transaction_hash);
CREATE INDEX idx_tx_sender ON transactions(sender_wallet_id);
CREATE INDEX idx_tx_recipient ON transactions(recipient_wallet_id);
CREATE INDEX idx_tx_status ON transactions(status);
CREATE INDEX idx_tx_block_id ON transactions(block_id);
```

**Sample Data**:

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "transaction_hash": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
  "sender_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
  "recipient_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
  "amount": "50.00000000",
  "fee": "1.00000000",
  "timestamp": "2024-12-07T11:30:00Z",
  "signature": "signature_base64_data",
  "status": "confirmed",
  "block_height": 102,
  "created_at": "2024-12-07T11:30:00Z"
}
```

---

### 4. BLOCKS Table

**Purpose**: Store blockchain blocks

| Column            | Type          | Constraints      | Description              |
| ----------------- | ------------- | ---------------- | ------------------------ |
| id                | UUID          | PK, AUTO         | Internal block ID        |
| block_hash        | VARCHAR(255)  | UNIQUE, NOT NULL | SHA256 hash of block     |
| previous_hash     | VARCHAR(255)  |                  | Hash of previous block   |
| height            | INTEGER       | UNIQUE, NOT NULL | Block number in chain    |
| timestamp         | TIMESTAMP     | NOT NULL         | Block creation time      |
| nonce             | BIGINT        | NOT NULL         | Proof-of-Work nonce      |
| difficulty        | INTEGER       | NOT NULL         | Mining difficulty        |
| miner_wallet_id   | VARCHAR(255)  | FK               | Miner's wallet           |
| miner_reward      | NUMERIC(20,8) |                  | Reward amount            |
| transaction_count | INTEGER       | DEFAULT 0        | Tx count in block        |
| merkle_root       | VARCHAR(255)  |                  | Root of transaction tree |
| created_at        | TIMESTAMP     | DEFAULT NOW      | Record creation time     |

**Indexes**:

```sql
CREATE INDEX idx_blocks_hash ON blocks(block_hash);
CREATE INDEX idx_blocks_height ON blocks(height);
CREATE INDEX idx_blocks_miner ON blocks(miner_wallet_id);
```

**Sample Data**:

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "block_hash": "00abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
  "previous_hash": "00aaabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567",
  "height": 102,
  "timestamp": "2024-12-07T11:35:00Z",
  "nonce": 45678934,
  "difficulty": 3,
  "miner_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
  "miner_reward": "500.00000000",
  "transaction_count": 5,
  "merkle_root": "merkle_root_hash_here",
  "created_at": "2024-12-07T11:35:00Z"
}
```

---

### 5. UTXOS Table

**Purpose**: Store Unspent Transaction Outputs (Bitcoin-style)

| Column           | Type          | Constraints   | Description               |
| ---------------- | ------------- | ------------- | ------------------------- |
| id               | UUID          | PK, AUTO      | Internal UTXO ID          |
| transaction_hash | VARCHAR(255)  | NOT NULL, FK  | Source transaction        |
| output_index     | INTEGER       | NOT NULL      | Output index in tx        |
| wallet_id        | VARCHAR(255)  | NOT NULL, FK  | Owner's wallet            |
| amount           | NUMERIC(20,8) | NOT NULL      | UTXO amount               |
| is_spent         | BOOLEAN       | DEFAULT FALSE | Spending status           |
| spent_in_tx_hash | VARCHAR(255)  |               | Transaction spending it   |
| block_height     | INTEGER       |               | Block height when created |
| created_at       | TIMESTAMP     | DEFAULT NOW   | Creation time             |
| updated_at       | TIMESTAMP     | DEFAULT NOW   | Last update time          |

**Indexes**:

```sql
CREATE INDEX idx_utxo_wallet ON utxos(wallet_id);
CREATE INDEX idx_utxo_is_spent ON utxos(is_spent);
CREATE INDEX idx_utxo_tx_hash ON utxos(transaction_hash);
```

**Sample Data**:

```json
{
  "id": "880e8400-e29b-41d4-a716-446655440003",
  "transaction_hash": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
  "output_index": 0,
  "wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
  "amount": "49.00000000",
  "is_spent": false,
  "spent_in_tx_hash": null,
  "block_height": 102,
  "created_at": "2024-12-07T11:35:00Z"
}
```

---

### 6. BENEFICIARIES Table

**Purpose**: Store frequently used recipient wallets

| Column                | Type         | Constraints  | Description           |
| --------------------- | ------------ | ------------ | --------------------- |
| id                    | UUID         | PK, AUTO     | Beneficiary record ID |
| user_id               | UUID         | NOT NULL, FK | User who saved        |
| beneficiary_wallet_id | VARCHAR(255) | NOT NULL, FK | Saved wallet          |
| beneficiary_name      | VARCHAR(255) |              | Beneficiary name      |
| nickname              | VARCHAR(255) |              | Custom nickname       |
| created_at            | TIMESTAMP    | DEFAULT NOW  | Save time             |

**Indexes**:

```sql
CREATE INDEX idx_beneficiary_user ON beneficiaries(user_id);
CREATE UNIQUE INDEX idx_beneficiary_unique ON beneficiaries(user_id, beneficiary_wallet_id);
```

---

### 7. OTP_CODES Table

**Purpose**: Store OTP codes for email verification

| Column     | Type         | Constraints   | Description          |
| ---------- | ------------ | ------------- | -------------------- |
| id         | UUID         | PK, AUTO      | OTP record ID        |
| email      | VARCHAR(255) | NOT NULL      | Email address        |
| otp_code   | VARCHAR(6)   | NOT NULL      | 6-digit OTP          |
| expires_at | TIMESTAMP    | NOT NULL      | Expiry time (10 min) |
| is_used    | BOOLEAN      | DEFAULT FALSE | Usage status         |
| used_at    | TIMESTAMP    |               | When OTP was used    |
| created_at | TIMESTAMP    | DEFAULT NOW   | Generation time      |

**Indexes**:

```sql
CREATE INDEX idx_otp_email ON otp_codes(email);
CREATE INDEX idx_otp_code ON otp_codes(otp_code);
```

**Auto-Cleanup**: Delete used/expired OTPs after 24 hours

---

### 8. BLOCKCHAIN_INFO Table

**Purpose**: Store blockchain global state

| Column                 | Type          | Description              |
| ---------------------- | ------------- | ------------------------ |
| total_coins            | NUMERIC(20,8) | Max supply (21M)         |
| circulating_coins      | NUMERIC(20,8) | Total issued coins       |
| total_transactions     | BIGINT        | Total transaction count  |
| total_blocks           | INTEGER       | Total blocks in chain    |
| current_difficulty     | INTEGER       | Current PoW difficulty   |
| average_block_time     | NUMERIC(10,2) | Avg block time (seconds) |
| halving_height         | INTEGER       | Next halving block       |
| current_halving_number | INTEGER       | Halving cycle number     |

---

### 9. LOGS Table

**Purpose**: Audit trail for security events

| Column        | Type         | Description                  |
| ------------- | ------------ | ---------------------------- |
| user_id       | UUID         | User performing action       |
| action        | VARCHAR(255) | Action type (login, tx, etc) |
| resource_type | VARCHAR(100) | wallet/transaction/block     |
| resource_id   | VARCHAR(255) | Affected resource ID         |
| details       | JSONB        | Additional details           |
| ip_address    | INET         | Source IP                    |
| user_agent    | TEXT         | Browser user agent           |
| created_at    | TIMESTAMP    | Log creation time            |

---

## 🔗 Entity Relationship Diagram

```
users (1) ──────── (1) wallets
  │
  ├─→ (1) ──────── (N) transactions (as sender)
  │
  └─→ (1) ──────── (N) beneficiaries


wallets (1) ──────── (1) users
  │
  ├─→ (1) ──────── (N) transactions (as sender/recipient)
  │
  ├─→ (1) ──────── (N) blocks (as miner)
  │
  └─→ (1) ──────── (N) utxos


transactions (1) ──────── (N) blocks
  │
  └─→ (1) ──────── (N) utxos


blocks (1) ──────── (N) transactions
  │
  └─→ (1) ──────── (1) wallets (as miner)
```

---

## 🔐 Data Types

### Numeric Precision

- **Balance**: `NUMERIC(20, 8)` - Max ~99,999,999.99999999 coins
- **Fees**: `NUMERIC(20, 8)` - Maximum precision for micro-transactions

### Cryptographic Hashes

- **Hash fields**: `VARCHAR(255)` - SHA256 (64 hex chars)
- **Signatures**: `TEXT` - Base64 encoded RSA signatures

### Timestamps

- **All timestamps**: `TIMESTAMP` (UTC) - ISO 8601 format
- **Server timezone**: UTC
- **Client displays**: Local timezone conversion

---

## 🔄 Database Relationships

### Foreign Keys with Cascading

```sql
-- Users to Wallets (1:1)
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
-- When user deleted, wallet deleted too

-- Transactions to Wallets (N:1)
FOREIGN KEY (sender_wallet_id) REFERENCES wallets(wallet_id)
-- Prevents orphaned transactions

-- Beneficiaries to Users (N:1)
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
-- Clean up beneficiaries when user deleted
```

---

## 📊 Query Examples

### Get User with Wallet Balance

```sql
SELECT u.*, w.balance
FROM users u
JOIN wallets w ON u.wallet_id = w.wallet_id
WHERE u.email = 'user@example.com';
```

### Get Recent Transactions for Wallet

```sql
SELECT * FROM transactions
WHERE sender_wallet_id = $1 OR recipient_wallet_id = $1
ORDER BY timestamp DESC
LIMIT 50;
```

### Calculate Total Balance with UTXOs

```sql
SELECT w.wallet_id, SUM(u.amount) as balance
FROM wallets w
LEFT JOIN utxos u ON w.wallet_id = u.wallet_id
WHERE u.is_spent = FALSE
GROUP BY w.wallet_id;
```

### Get Latest Block

```sql
SELECT * FROM blocks
ORDER BY height DESC
LIMIT 1;
```

### Get Pending Transactions

```sql
SELECT * FROM transactions
WHERE status = 'pending'
ORDER BY created_at ASC;
```

---

## 📈 Performance Optimization

### Indexes Created

1. **Users**: email, wallet_id
2. **Transactions**: tx_hash, sender, recipient, status, block_id
3. **Blocks**: hash, height, miner
4. **UTXOs**: wallet_id, is_spent, tx_hash
5. **Beneficiaries**: user_id
6. **OTP**: email, code

### Query Optimization Tips

1. Always filter by indexed columns first
2. Use LIMIT on large result sets
3. Batch similar queries
4. Consider connection pooling
5. Monitor slow query log

---

## 🛡️ Data Security

### Encryption at Rest

- **Private keys**: AES-256-GCM encrypted
- **Passwords**: Argon2 hashing (not stored plain)
- **Sensitive data**: Encrypted in application layer

### Encryption in Transit

- All database connections: SSL/TLS
- All API calls: HTTPS/TLS

### Access Control

- Database users with limited permissions
- Row-level security policies (PostgreSQL)
- Application-level authorization checks

---

## 🔧 Maintenance Tasks

### Regular Backups

- Supabase: Automatic daily backups
- Retention: 30 days
- Recovery: Point-in-time restore available

### Monitoring

- Query performance monitoring
- Connection pool utilization
- Disk space usage
- Transaction throughput

### Cleanup Tasks

```sql
-- Delete expired OTP codes (older than 24 hours)
DELETE FROM otp_codes
WHERE expires_at < NOW() - INTERVAL '24 hours';

-- Delete old logs (older than 90 days)
DELETE FROM logs
WHERE created_at < NOW() - INTERVAL '90 days';
```

---

**Document Version**: 1.0  
**Last Updated**: December 7, 2024
