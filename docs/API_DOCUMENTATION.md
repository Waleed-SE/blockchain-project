# API Documentation - Blockchain Wallet

## 🔌 API Overview

**Base URL**: `https://blockchain-project-f995.onrender.com/api`  
**Version**: v1  
**Protocol**: REST/JSON  
**Authentication**: JWT Bearer Token  
**Content-Type**: `application/json`

---

## 📋 API Sections

1. [Authentication](#authentication)
2. [Wallet Management](#wallet-management)
3. [Transactions](#transactions)
4. [Blockchain](#blockchain)
5. [Mining](#mining)
6. [Beneficiaries](#beneficiaries)
7. [Error Responses](#error-responses)

---

## 🔐 Authentication

All protected endpoints require JWT token in Authorization header:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Register

**Endpoint**: `POST /auth/register`

**Description**: Create a new user account with automatic wallet generation

**Request**:

```json
{
  "email": "user@example.com",
  "full_name": "John Doe",
  "cnic": "12345-6789012-3",
  "password": "SecurePassword123"
}
```

**Response** (201 Created):

```json
{
  "success": true,
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "full_name": "John Doe",
      "cnic": "12345-6789012-3",
      "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
      "public_key": "-----BEGIN PUBLIC KEY-----...",
      "is_verified": false,
      "created_at": "2024-12-07T10:30:00Z"
    },
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "message": "User registered successfully"
}
```

**Errors**:

- `400`: Email already exists
- `400`: Invalid CNIC format
- `400`: Password too weak

---

### Login

**Endpoint**: `POST /auth/login`

**Description**: Authenticate user and get JWT token

**Request**:

```json
{
  "email": "user@example.com",
  "password": "SecurePassword123"
}
```

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "full_name": "John Doe",
      "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
      "is_verified": true
    },
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "message": "Login successful"
}
```

**Errors**:

- `401`: Invalid credentials
- `401`: User not found

---

### Send OTP

**Endpoint**: `POST /auth/send-otp`

**Description**: Send 6-digit OTP to user's email (alternative auth flow)

**Request**:

```json
{
  "email": "user@example.com"
}
```

**Response** (200 OK):

```json
{
  "success": true,
  "message": "OTP sent to your email",
  "data": {
    "expires_in_seconds": 600
  }
}
```

**Errors**:

- `404`: User not found
- `429`: Too many OTP requests (rate limited)

---

### Verify OTP

**Endpoint**: `POST /auth/verify-otp`

**Description**: Verify OTP and get JWT token

**Request**:

```json
{
  "email": "user@example.com",
  "otp": "123456"
}
```

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "email": "user@example.com",
      "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144"
    }
  },
  "message": "OTP verified successfully"
}
```

**Errors**:

- `400`: Invalid or expired OTP
- `404`: User not found

---

## 👤 User Profile

### Get Profile

**Endpoint**: `GET /auth/profile`  
**Auth**: Required (Bearer Token)

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "full_name": "John Doe",
    "cnic": "12345-6789012-3",
    "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
    "is_verified": true,
    "created_at": "2024-12-07T10:30:00Z"
  },
  "message": "Profile retrieved"
}
```

---

### Update Profile

**Endpoint**: `PUT /auth/profile`  
**Auth**: Required

**Request**:

```json
{
  "full_name": "John Smith",
  "email": "newemail@example.com"
}
```

**Response** (200 OK):

```json
{
  "success": true,
  "message": "Profile updated successfully"
}
```

---

## 💼 Wallet Management

### Get Wallet Info

**Endpoint**: `GET /wallet/:wallet_id`  
**Auth**: Required

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
    "balance": "1500.50000000",
    "public_key": "-----BEGIN PUBLIC KEY-----...",
    "created_at": "2024-12-07T10:30:00Z",
    "transaction_count": 42,
    "mining_reward_count": 5
  },
  "message": "Wallet data retrieved"
}
```

---

### Get Balance

**Endpoint**: `GET /wallet/:wallet_id/balance`  
**Auth**: Required

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "balance": "1500.50000000",
    "locked_balance": "0.00000000",
    "available_balance": "1500.50000000"
  },
  "message": "Balance retrieved"
}
```

---

### Get UTXOs

**Endpoint**: `GET /wallet/:wallet_id/utxos`  
**Auth**: Required

**Query Parameters**:

- `spent`: `true|false` - Filter by spent status (optional)
- `limit`: `50` - Max results (default: 100)

**Response** (200 OK):

```json
{
  "success": true,
  "data": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440003",
      "transaction_hash": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
      "output_index": 0,
      "amount": "500.00000000",
      "is_spent": false,
      "block_height": 102,
      "created_at": "2024-12-07T11:35:00Z"
    },
    {
      "id": "880e8400-e29b-41d4-a716-446655440004",
      "transaction_hash": "def456ghi789jkl012mno345pqr678stu901vwx234yzabc123",
      "output_index": 0,
      "amount": "1000.50000000",
      "is_spent": false,
      "block_height": 100,
      "created_at": "2024-12-07T11:20:00Z"
    }
  ],
  "message": "UTXOs retrieved",
  "count": 2
}
```

---

## 💸 Transactions

### Create Transaction

**Endpoint**: `POST /transactions/create`  
**Auth**: Required

**Request**:

```json
{
  "sender_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
  "recipient_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
  "amount": "50.00000000"
}
```

**Response** (201 Created):

```json
{
  "success": true,
  "data": {
    "transaction_hash": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
    "sender_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
    "recipient_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
    "amount": "50.00000000",
    "fee": "1.00000000",
    "total": "51.00000000",
    "status": "pending",
    "timestamp": "2024-12-07T11:30:00Z"
  },
  "message": "Transaction created successfully"
}
```

**Errors**:

- `400`: Insufficient balance
- `400`: Invalid recipient wallet
- `400`: Invalid amount
- `409`: Double-spend attempt

---

### Get Transaction Details

**Endpoint**: `GET /transactions/:tx_hash`  
**Auth**: Required

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "transaction_hash": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
    "sender_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
    "recipient_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
    "amount": "50.00000000",
    "fee": "1.00000000",
    "status": "confirmed",
    "block_height": 102,
    "block_hash": "00abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
    "confirmations": 10,
    "timestamp": "2024-12-07T11:30:00Z"
  },
  "message": "Transaction retrieved"
}
```

---

### Get Wallet Transactions

**Endpoint**: `GET /transactions/:wallet_id`  
**Auth**: Required

**Query Parameters**:

- `status`: `pending|confirmed|failed` - Filter by status
- `type`: `sent|received` - Filter by type
- `limit`: `50` - Max results (default: 100)
- `offset`: `0` - Pagination offset

**Response** (200 OK):

```json
{
  "success": true,
  "data": [
    {
      "transaction_hash": "abc123def456...",
      "sender_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
      "recipient_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
      "amount": "50.00000000",
      "fee": "1.00000000",
      "type": "sent",
      "status": "confirmed",
      "timestamp": "2024-12-07T11:30:00Z"
    }
  ],
  "pagination": {
    "total": 42,
    "limit": 50,
    "offset": 0,
    "has_more": false
  },
  "message": "Transactions retrieved"
}
```

---

## ⛓️ Blockchain

### Get Blockchain Info

**Endpoint**: `GET /blockchain/info`  
**Auth**: Optional

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "total_coins": "21000000.00000000",
    "circulating_coins": "2500.00000000",
    "total_transactions": 125,
    "total_blocks": 105,
    "current_difficulty": 3,
    "average_block_time": 9.8,
    "next_halving_height": 110,
    "current_halving_number": 0,
    "halving_percentage": 47.6,
    "block_reward": "500.00000000",
    "transaction_fee": "1.00000000"
  },
  "message": "Blockchain info retrieved"
}
```

---

### Get All Blocks

**Endpoint**: `GET /blockchain/blocks`  
**Auth**: Optional

**Query Parameters**:

- `limit`: `50` - Results per page (default: 20)
- `page`: `1` - Page number

**Response** (200 OK):

```json
{
  "success": true,
  "data": [
    {
      "block_hash": "00abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
      "height": 105,
      "previous_hash": "00aaabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567",
      "timestamp": "2024-12-07T11:35:00Z",
      "miner_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
      "miner_reward": "500.00000000",
      "transaction_count": 5,
      "difficulty": 3,
      "confirmations": 0
    }
  ],
  "pagination": {
    "total": 105,
    "limit": 20,
    "page": 1,
    "total_pages": 6
  },
  "message": "Blocks retrieved"
}
```

---

### Get Block Details

**Endpoint**: `GET /blockchain/blocks/:block_id`  
**Auth**: Optional

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "block_hash": "00abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
    "height": 105,
    "previous_hash": "00aaabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567",
    "timestamp": "2024-12-07T11:35:00Z",
    "nonce": 45678934,
    "difficulty": 3,
    "miner_wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
    "miner_reward": "500.00000000",
    "transaction_count": 5,
    "merkle_root": "merkle_root_hash_here",
    "size_bytes": 1024,
    "confirmations": 0,
    "transactions": [
      {
        "transaction_hash": "abc123def456ghi789jkl012mno345pqr678stu901vwx234yz",
        "amount": "50.00000000",
        "fee": "1.00000000"
      }
    ]
  },
  "message": "Block retrieved"
}
```

---

## ⛏️ Mining

### Get Mining Stats

**Endpoint**: `GET /blockchain/mining-stats`  
**Auth**: Optional

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "total_blocks": 105,
    "pending_transactions": 3,
    "estimated_next_block_time": "~10 seconds",
    "total_miners": 1,
    "current_difficulty": 3,
    "network_hashrate": "calculated_hashrate",
    "top_miners": [
      {
        "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
        "blocks_mined": 25,
        "total_reward": "12500.00000000"
      }
    ]
  },
  "message": "Mining stats retrieved"
}
```

---

### Start Mining

**Endpoint**: `POST /mining/start-mining`  
**Auth**: Required

**Request**:

```json
{
  "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144"
}
```

**Response** (200 OK):

```json
{
  "success": true,
  "data": {
    "mining_started": true,
    "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144",
    "pending_transactions": 3,
    "estimated_time": "~10 seconds"
  },
  "message": "Mining started"
}
```

---

### Mine Block

**Endpoint**: `POST /mining/mine-block`  
**Auth**: Required

**Request**:

```json
{
  "wallet_id": "5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144"
}
```

**Response** (201 Created):

```json
{
  "success": true,
  "data": {
    "block_hash": "00abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678",
    "height": 105,
    "timestamp": "2024-12-07T11:35:00Z",
    "transactions_included": 5,
    "reward": "500.00000000",
    "nonce": 45678934,
    "mining_time_seconds": 9.8
  },
  "message": "Block mined successfully"
}
```

**Errors**:

- `400`: Insufficient pending transactions
- `409`: Mining already in progress

---

## 📋 Beneficiaries

### Get Beneficiaries

**Endpoint**: `GET /beneficiaries`  
**Auth**: Required

**Response** (200 OK):

```json
{
  "success": true,
  "data": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440005",
      "beneficiary_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
      "beneficiary_name": "Alice Smith",
      "nickname": "Sister",
      "created_at": "2024-12-07T10:00:00Z"
    }
  ],
  "message": "Beneficiaries retrieved"
}
```

---

### Add Beneficiary

**Endpoint**: `POST /beneficiaries`  
**Auth**: Required

**Request**:

```json
{
  "beneficiary_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
  "beneficiary_name": "Alice Smith",
  "nickname": "Sister"
}
```

**Response** (201 Created):

```json
{
  "success": true,
  "data": {
    "id": "990e8400-e29b-41d4-a716-446655440005",
    "beneficiary_wallet_id": "6d698281c666g97c384gee5931358gbf8be3gf82bffe8e0e4eg994cff87g255",
    "beneficiary_name": "Alice Smith",
    "nickname": "Sister"
  },
  "message": "Beneficiary added successfully"
}
```

---

### Delete Beneficiary

**Endpoint**: `DELETE /beneficiaries/:id`  
**Auth**: Required

**Response** (200 OK):

```json
{
  "success": true,
  "message": "Beneficiary removed successfully"
}
```

---

## ❌ Error Responses

All errors follow this format:

```json
{
  "success": false,
  "message": "Error description",
  "data": null
}
```

### HTTP Status Codes

| Code | Meaning           | Example                    |
| ---- | ----------------- | -------------------------- |
| 200  | OK                | Request successful         |
| 201  | Created           | Resource created           |
| 400  | Bad Request       | Invalid input              |
| 401  | Unauthorized      | Missing/invalid token      |
| 403  | Forbidden         | Access denied              |
| 404  | Not Found         | Resource not found         |
| 409  | Conflict          | Double-spend, email exists |
| 429  | Too Many Requests | Rate limited               |
| 500  | Server Error      | Internal error             |

### Common Errors

**Invalid Token** (401):

```json
{
  "success": false,
  "message": "Invalid token",
  "data": null
}
```

**Insufficient Balance** (400):

```json
{
  "success": false,
  "message": "Insufficient balance for transaction",
  "data": null
}
```

**User Not Found** (404):

```json
{
  "success": false,
  "message": "User not found",
  "data": null
}
```

---

## 🔄 Request/Response Examples

### Complete Login Flow

**Step 1: Register**

```bash
curl -X POST https://blockchain-project-f995.onrender.com/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "full_name": "John Doe",
    "cnic": "12345-6789012-3",
    "password": "SecurePass123"
  }'
```

**Step 2: Login**

```bash
curl -X POST https://blockchain-project-f995.onrender.com/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123"
  }'
```

**Step 3: Get Wallet**

```bash
curl -X GET https://blockchain-project-f995.onrender.com/api/wallet/5c587170b555f96b273fdd4820247faf7ad2fe71aeed7d9d9d3df884bee76144 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## ⏱️ Rate Limiting

- **OTP Requests**: 3 per hour per email
- **Login Attempts**: 5 per 15 minutes per IP
- **API Calls**: 100 per minute per token

---

## 📚 Additional Resources

- **Blockchain Design**: See BLOCKCHAIN_DESIGN.md
- **Database Schema**: See DATABASE_SCHEMA.md
- **Architecture**: See ARCHITECTURE.md
- **Error Troubleshooting**: See TROUBLESHOOTING.md

---

**Document Version**: 1.0  
**Last Updated**: December 7, 2024
