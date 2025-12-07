# System Architecture - Blockchain Wallet

## 📐 Overview

The Blockchain Wallet is a full-stack application built with:

- **Backend**: Rust with Actix-web framework
- **Frontend**: React 19 with TypeScript
- **Database**: PostgreSQL (Supabase)
- **Infrastructure**: Render (backend), Vercel (frontend)

---

## 🏛️ Architectural Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     PRESENTATION LAYER (Frontend)               │
│                    https://vercel.app (React 19)                │
│                                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │Dashboard │  │ Wallet   │  │ Send $   │  │ Mining   │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │Blocks    │  │History   │  │Profile   │  │Explorer  │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└────────────────────────┬────────────────────────────────────────┘
                         │ REST/JSON (HTTPS)
                         │ Axios HTTP Client
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│                    API LAYER (Backend)                          │
│            https://render.com:8080/api (Rust/Actix-web)        │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Routes & Handlers                                        │  │
│  │ - Auth Handlers (login, register, OTP)                  │  │
│  │ - Wallet Handlers (get balance, UTXOs)                  │  │
│  │ - Transaction Handlers (create, get)                    │  │
│  │ - Blockchain Handlers (blocks, info)                    │  │
│  │ - Mining Handlers (start, stop)                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                         ↓                                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Business Logic Layer                                     │  │
│  │ - Auth Service (JWT, password hashing)                  │  │
│  │ - Wallet Service (balance calculation)                  │  │
│  │ - Transaction Service (validation, signing)             │  │
│  │ - Blockchain Service (mining, block creation)           │  │
│  │ - OTP Service (email generation)                        │  │
│  └──────────────────────────────────────────────────────────┘  │
│                         ↓                                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Data Access Layer                                        │  │
│  │ - Database Queries                                       │  │
│  │ - Connection Pool Management                            │  │
│  │ - Transaction Management                                │  │
│  └──────────────────────────────────────────────────────────┘  │
└────────────────────────┬────────────────────────────────────────┘
                         │ TCP (Port 5432/6543)
                         │ PostgreSQL Protocol
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│                   DATA LAYER (Database)                         │
│         PostgreSQL Serverless (Supabase - Asia Pacific)         │
│                                                                 │
│  Tables:                                                        │
│  - users, wallets, transactions, blocks, utxos                │
│  - beneficiaries, otp_codes, logs                             │
│                                                                 │
│  Features:                                                      │
│  - Connection Pooling (3 connections - Session Mode)          │
│  - Automatic Backups                                          │
│  - SSL/TLS Encryption                                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔀 Component Interaction Flow

### Request/Response Cycle

```
User Action (Frontend)
        ↓
Axios HTTP Request
        ↓
CORS Validation
        ↓
Route Handler (Backend)
        ↓
Authentication Check (JWT)
        ↓
Business Logic Processing
        ↓
Database Query Execution
        ↓
Response Formation
        ↓
JSON Response
        ↓
Frontend Update
        ↓
UI Render
```

---

## 🔐 Authentication Architecture

```
┌─────────────────────────────────────────┐
│      Login Request (email + password)   │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│   Find User by Email in Database        │
└──────────────┬──────────────────────────┘
               ↓
        ┌──────┴──────┐
        ↓             ↓
   User Found    No User
        │         │
        │         └─→ Return 401
        │
        ↓
┌─────────────────────────────────────────┐
│   Generate JWT Token                    │
│   - User ID (sub)                       │
│   - Email (email)                       │
│   - Expiry (exp)                        │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│   Return Token + User Data              │
└─────────────────────────────────────────┘

Subsequent Requests:
Bearer Token → Validate JWT → Extract Claims → Authorize
```

---

## 💰 Transaction Flow Architecture

```
User Input (recipient, amount)
        ↓
┌──────────────────────────────────────┐
│  Validate Input                      │
│  - Amount > 0                        │
│  - Recipient wallet exists           │
│  - Sufficient balance                │
└──────────────┬───────────────────────┘
               ↓
┌──────────────────────────────────────┐
│  Create Transaction Object           │
│  - sender_wallet_id                  │
│  - recipient_wallet_id               │
│  - amount                            │
│  - fee                               │
│  - timestamp                         │
└──────────────┬───────────────────────┘
               ↓
┌──────────────────────────────────────┐
│  Sign Transaction                    │
│  - Use sender's private key          │
│  - RSA-2048 signature                │
│  - SHA-256 hash                      │
└──────────────┬───────────────────────┘
               ↓
┌──────────────────────────────────────┐
│  Add to Pending Pool                 │
│  - Broadcast to network (local)      │
│  - Store in pending_transactions     │
└──────────────┬───────────────────────┘
               ↓
        (Awaiting Miner)
               ↓
┌──────────────────────────────────────┐
│  Miner Includes in Block             │
│  - Validates signature               │
│  - Checks balance                    │
│  - Creates UTXO                      │
└──────────────┬───────────────────────┘
               ↓
┌──────────────────────────────────────┐
│  Block Added to Chain                │
│  - Transaction confirmed             │
│  - Balances updated                  │
│  - UTXOs created/consumed            │
└──────────────────────────────────────┘
```

---

## ⛏️ Mining Architecture

```
Mining Job Request
        ↓
┌──────────────────────────────────┐
│  Collect Pending Transactions    │
│  - Up to 500 per block          │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  Create Block Header             │
│  - Previous block hash           │
│  - Merkle root of transactions   │
│  - Timestamp                     │
│  - Difficulty target             │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  Proof-of-Work                   │
│  - Increment nonce              │
│  - Hash block header            │
│  - Check if < target            │
│  - Repeat until found (avg 10s) │
└──────────────┬───────────────────┘
               ↓
        ┌──────┴──────┐
        ↓             ↓
   Valid PoW    Invalid PoW
        │         │
        │         └─→ Try again
        │
        ↓
┌──────────────────────────────────┐
│  Create Block                    │
│  - Serialize transactions        │
│  - Add miner reward              │
│  - Calculate block hash          │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  Validate Block                  │
│  - Check PoW                     │
│  - Verify signatures             │
│  - Check balances                │
│  - No double-spending            │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  Add to Blockchain               │
│  - Link to previous block        │
│  - Update chain length           │
│  - Clear pending pool            │
│  - Update UTXOs                  │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│  Emit Mining Reward              │
│  - 500 coins initially           │
│  - Halved every 5 blocks         │
│  - Goes to miner's wallet        │
└──────────────────────────────────┘
```

---

## 🗄️ Database Connection Architecture

```
Frontend Request
        ↓
Backend Handler
        ↓
┌─────────────────────────────────────┐
│   Connection Pool                   │
│   (Max 3 connections - Session Mode)│
│   OR (Max 100 - Transaction Mode)   │
└──────────────┬──────────────────────┘
               ↓
        ┌──────┴──────┐
        ↓             ↓
   Conn Avail   Conn Unavail
        │         │
        ↓         └─→ Queue request
        │
┌──────────────────────────────────────┐
│   Execute Query                      │
│   - Validate input                   │
│   - Execute SQL                      │
│   - Return results                   │
└──────────────┬───────────────────────┘
               ↓
┌──────────────────────────────────────┐
│   Release Connection Back to Pool    │
└──────────────────────────────────────┘
```

---

## 🔄 State Management (Frontend)

```
┌─────────────────────────────────────┐
│      Auth Context                   │
│  - user: User | null                │
│  - token: string | null             │
│  - login()                          │
│  - register()                       │
│  - logout()                         │
└─────────────────────────────────────┘

Provides auth state to all components:
└─→ Protected Routes
└─→ Header (show user)
└─→ Wallet Pages
└─→ Transaction Pages
```

---

## 🛡️ Security Layers

```
┌─────────────────────────────────────┐
│   Layer 1: Transport Security       │
│   - HTTPS/TLS encryption           │
│   - Secure WebSocket (WSS)         │
└─────────────────────────────────────┘
               ↓
┌─────────────────────────────────────┐
│   Layer 2: API Security             │
│   - CORS validation                 │
│   - Rate limiting                   │
│   - Input validation                │
└─────────────────────────────────────┘
               ↓
┌─────────────────────────────────────┐
│   Layer 3: Authentication           │
│   - JWT token validation            │
│   - OTP verification                │
│   - Password hashing (Argon2)       │
└─────────────────────────────────────┘
               ↓
┌─────────────────────────────────────┐
│   Layer 4: Authorization            │
│   - User ownership checks           │
│   - Wallet access control           │
│   - Signature verification          │
└─────────────────────────────────────┘
               ↓
┌─────────────────────────────────────┐
│   Layer 5: Cryptography             │
│   - RSA-2048 signatures             │
│   - AES-256-GCM encryption          │
│   - SHA-256 hashing                 │
└─────────────────────────────────────┘
               ↓
┌─────────────────────────────────────┐
│   Layer 6: Data Integrity           │
│   - Double-spend prevention         │
│   - Balance validation              │
│   - Chain validation                │
└─────────────────────────────────────┘
```

---

## 📊 Scalability Considerations

### Current Limitations

- Single backend instance
- 3 database connections (Supabase Session Mode)
- No caching layer
- Synchronous mining

### Future Improvements

1. **Horizontal Scaling**

   - Load balancer
   - Multiple backend instances
   - Shared session storage

2. **Database Optimization**

   - Read replicas
   - Query optimization
   - Indexing strategy

3. **Caching Layer**

   - Redis for blockchain data
   - Token caching
   - Transaction pool caching

4. **Async Mining**
   - Background job queue
   - Worker processes
   - Real-time updates (WebSocket)

---

## 🚀 Performance Optimizations

### Backend

- Connection pooling
- Query optimization
- Async/await non-blocking I/O
- Cargo release build with LTO

### Frontend

- Code splitting with Vite
- Lazy loading routes
- Component memoization
- CSS minification

### Database

- Indexed lookups on wallet_id, email
- Materialized views for stats
- Connection pooling on transaction mode

---

## 🔄 Deployment Architecture

```
Git Repository (GitHub)
        ↓
┌───────────────────────┐     ┌───────────────────────┐
│  Backend Deployment   │     │ Frontend Deployment   │
│  (Render)             │     │ (Vercel)              │
├───────────────────────┤     ├───────────────────────┤
│ Trigger: git push     │     │ Trigger: git push     │
│ Build: cargo build    │     │ Build: npm run build  │
│ Start: ./target/...   │     │ Output: dist/         │
│ Env: 20+ variables    │     │ Env: VITE_API_URL     │
│ Health: /api/health   │     │ Auto: redeploy        │
└───────────────────────┘     └───────────────────────┘
        ↓                               ↓
        └───────────┬───────────────────┘
                    ↓
        ┌───────────────────────┐
        │  Supabase Database    │
        │  PostgreSQL (pooler)  │
        │  Backups: Daily       │
        │  Monitoring: Built-in │
        └───────────────────────┘
```

---

## 📈 Monitoring & Observability

### Backend Logging

```
[2025-12-07T13:07:51Z INFO actix_web::middleware::logger]
127.0.0.1 "POST /api/auth/login HTTP/1.1" 200
```

### Metrics Tracked

- Request latency
- Error rates
- Database connection pool usage
- Mining performance
- Transaction throughput

### Tools

- Render logs dashboard
- Vercel analytics
- Supabase monitoring
- Browser DevTools

---

## 🔗 API Communication

### REST Endpoints

```
Base URL: https://blockchain-project-f995.onrender.com/api

Request Structure:
POST /auth/login
Content-Type: application/json
Authorization: Bearer <token>

{
  "email": "user@example.com",
  "password": "password"
}

Response Structure:
{
  "success": true,
  "data": { ... },
  "message": "Success"
}
```

### Error Handling

```
200 OK - Success
400 Bad Request - Invalid input
401 Unauthorized - Auth failed
404 Not Found - Resource missing
500 Server Error - Internal error

Error Response:
{
  "success": false,
  "message": "Error description",
  "data": null
}
```

---

## 🎯 Architectural Principles

1. **Separation of Concerns**

   - Handlers ↔ Services ↔ Database
   - Clean architecture pattern

2. **Security First**

   - Encryption at rest & in transit
   - Input validation everywhere
   - Principle of least privilege

3. **Scalability**

   - Stateless backend
   - Database connection pooling
   - Async processing

4. **Reliability**

   - Error handling at all layers
   - Retry logic for failed requests
   - Data consistency checks

5. **Maintainability**
   - Modular code structure
   - Clear naming conventions
   - Comprehensive documentation

---

**Document Version**: 1.0  
**Last Updated**: December 7, 2024
