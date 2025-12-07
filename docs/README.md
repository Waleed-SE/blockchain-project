# Blockchain Wallet - Complete Documentation

Welcome to the comprehensive documentation for the Blockchain Wallet application. This directory contains all technical documentation, architecture diagrams, and API references.

## 📚 Documentation Index

### Core Documentation

- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - System design, components, and technology stack
- **[DATABASE_SCHEMA.md](./DATABASE_SCHEMA.md)** - Database structure and relationships
- **[API_DOCUMENTATION.md](./API_DOCUMENTATION.md)** - Complete API reference with examples

### Additional Resources

- **[BLOCKCHAIN_DESIGN.md](./BLOCKCHAIN_DESIGN.md)** - Blockchain implementation details
- **[SECURITY.md](./SECURITY.md)** - Security considerations and best practices
- **[DEPLOYMENT.md](./DEPLOYMENT.md)** - Deployment guide and configuration

---

## 🎯 Quick Start

### For Developers

1. Start with [ARCHITECTURE.md](./ARCHITECTURE.md) to understand the system
2. Review [DATABASE_SCHEMA.md](./DATABASE_SCHEMA.md) for data structure
3. Check [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) for endpoints

### For DevOps/Deployment

1. Read [DEPLOYMENT.md](./DEPLOYMENT.md) for deployment steps
2. Check [SECURITY.md](./SECURITY.md) for security configuration
3. Review environment variables in deployment guide

### For Blockchain Understanding

1. Study [BLOCKCHAIN_DESIGN.md](./BLOCKCHAIN_DESIGN.md)
2. Understand UTXO model and PoW consensus
3. Review transaction flow

---

## 🏗️ High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Vercel)                        │
│              React 19 + Tailwind CSS + TypeScript           │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTPS
                       ↓
┌─────────────────────────────────────────────────────────────┐
│              Backend (Render - Rust/Actix-web)              │
│  - Authentication (Email + OTP)                             │
│  - Blockchain Management                                    │
│  - Transaction Processing                                   │
│  - Mining Operations                                        │
└──────────────────────┬──────────────────────────────────────┘
                       │ TCP
                       ↓
┌─────────────────────────────────────────────────────────────┐
│         Database (Supabase - PostgreSQL Serverless)         │
│  - Users & Wallets                                          │
│  - Transactions                                             │
│  - Blocks & UTXOs                                           │
│  - OTP Codes                                                │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔑 Key Technologies

### Backend

- **Language**: Rust 1.83+ (nightly)
- **Framework**: Actix-web 4.4
- **Database**: PostgreSQL (Supabase)
- **Cryptography**:
  - RSA-2048 (digital signatures)
  - SHA-256 (hashing)
  - AES-256-GCM (encryption)
  - HMAC-SHA256 (OTP generation)

### Frontend

- **Framework**: React 19 with TypeScript
- **Styling**: Tailwind CSS 4
- **Build Tool**: Vite
- **HTTP Client**: Axios

### Infrastructure

- **Backend Hosting**: Render.com (Container)
- **Frontend Hosting**: Vercel.com
- **Database**: Supabase (PostgreSQL)
- **Email**: Gmail SMTP

---

## 🌐 Deployment URLs

| Component | URL                                            | Status  |
| --------- | ---------------------------------------------- | ------- |
| Frontend  | https://blockchain-project-frontend.vercel.app | ✅ Live |
| Backend   | https://blockchain-project-f995.onrender.com   | ✅ Live |
| Database  | Supabase (Asia Pacific)                        | ✅ Live |

---

## 📊 Database Overview

### Main Tables

- **users** - User accounts and authentication
- **wallets** - Blockchain wallets
- **transactions** - All transactions
- **blocks** - Blockchain blocks
- **utxos** - Unspent transaction outputs
- **beneficiaries** - Saved recipients
- **otp_codes** - One-time passwords

_See [DATABASE_SCHEMA.md](./DATABASE_SCHEMA.md) for full details_

---

## 🔌 API Endpoints Overview

### Authentication

- `POST /api/auth/register` - Register new user
- `POST /api/auth/login` - Login user
- `POST /api/auth/send-otp` - Send OTP
- `POST /api/auth/verify-otp` - Verify OTP

### Wallet

- `GET /api/wallet/:wallet_id` - Get wallet info
- `GET /api/wallet/:wallet_id/balance` - Get balance
- `GET /api/wallet/:wallet_id/utxos` - Get UTXOs

### Transactions

- `POST /api/transactions/create` - Create transaction
- `GET /api/transactions/:wallet_id` - Get transactions
- `GET /api/transactions/:tx_hash` - Get transaction details

### Blockchain

- `GET /api/blockchain/info` - Get blockchain info
- `GET /api/blockchain/blocks` - Get all blocks
- `GET /api/blockchain/blocks/:block_id` - Get block details
- `POST /api/mining/mine-block` - Mine new block

_See [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) for full details with examples_

---

## 🔐 Security Features

- ✅ RSA-2048 Digital Signatures
- ✅ AES-256-GCM Encryption
- ✅ SHA-256 Hashing
- ✅ JWT Token Authentication
- ✅ OTP Email Verification
- ✅ CORS Protection
- ✅ Double-spend Prevention
- ✅ Password Hashing (Argon2)

_See [SECURITY.md](./SECURITY.md) for detailed security architecture_

---

## 💰 Blockchain Features

### Core Features

- ✅ Proof-of-Work Consensus (SHA-256 hashing)
- ✅ Bitcoin-style UTXO Model
- ✅ Digital Signature Verification
- ✅ Transaction Validation
- ✅ Mining with Difficulty Adjustment

### Advanced Features

- ✅ Bitcoin Halving Mechanism (every 5 blocks)
- ✅ Dynamic Transaction Fees
- ✅ Islamic Finance (Zakat - 2.5% charity)
- ✅ Block Explorer
- ✅ Chain Validation

_See [BLOCKCHAIN_DESIGN.md](./BLOCKCHAIN_DESIGN.md) for implementation details_

---

## 📈 Performance Metrics

| Metric               | Value                     |
| -------------------- | ------------------------- |
| Avg Block Time       | ~10 seconds               |
| Mining Difficulty    | 3 (configurable)          |
| Block Size Limit     | Unlimited                 |
| Transaction Limit    | ~500 per block            |
| Database Connections | 3 (Supabase Session Mode) |
| Frontend Build Size  | ~365KB gzipped            |

---

## 🚀 Quick Development Setup

### Prerequisites

- Rust 1.83+ (with nightly)
- Node.js 18+
- PostgreSQL (or Supabase account)

### Local Development

```bash
# Backend
cd backend
cargo run

# Frontend (new terminal)
cd frontend
npm install
npm run dev
```

Visit `http://localhost:5173`

---

## 📋 Environment Configuration

### Backend (.env)

```bash
DATABASE_URL=postgresql://user:pass@host/db
JWT_SECRET=your-secret-key
AES_ENCRYPTION_KEY=64-char-hex-key
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=app-password
# ... more variables in DEPLOYMENT.md
```

### Frontend (.env)

```bash
VITE_API_URL=http://localhost:8080/api
VITE_BLOCK_REWARD=50
VITE_MINING_DIFFICULTY=5
```

---

## 🔄 Data Flow

### User Registration Flow

```
1. User enters email, name, CNIC
2. Backend validates input
3. Generates wallet (private/public key pair)
4. Encrypts private key with AES-256-GCM
5. Stores user in database
6. Returns JWT token
7. Creates initial wallet with 0 balance
```

### Transaction Flow

```
1. User enters recipient and amount
2. Backend validates wallet has sufficient balance
3. Creates transaction object
4. Signs with sender's private key
5. Verifies receiver wallet exists
6. Adds to pending transactions pool
7. Miner includes in next block
8. Blockchain updates balance
```

### Mining Flow

```
1. Miner starts mining process
2. Collects pending transactions
3. Finds valid nonce (PoW)
4. Creates new block
5. Adds block to blockchain
6. Miner receives block reward
7. Transactions become confirmed
```

---

## 🐛 Troubleshooting

### Common Issues

**Database Connection Error**

- Check `DATABASE_URL` is correct
- Verify database is running
- Check connection pool limit (Supabase: 3 in Session Mode)

**CORS Error**

- Ensure frontend URL is in `ALLOWED_ORIGINS`
- Check environment variable on Render

**OTP Not Sending**

- Verify Gmail credentials
- Check "Less Secure Apps" setting
- Use App Password instead of account password

**Mining Errors**

- Check blockchain is initialized
- Verify pending transactions are valid
- Check available balance for block reward

_See individual documentation files for detailed troubleshooting_

---

## 📞 Support

### For Technical Issues

1. Check relevant documentation file
2. Review API logs in Render dashboard
3. Check frontend browser console
4. Review error messages in deployment logs

### Project Resources

- **GitHub Backend**: https://github.com/Waleed-SE/blockchain-project
- **GitHub Frontend**: https://github.com/Waleed-SE/blockchain-project-frontend
- **Live Demo**: https://blockchain-project-frontend.vercel.app

---

## 📄 Document Versions

| Document             | Version | Last Updated |
| -------------------- | ------- | ------------ |
| README.md            | 1.0     | Dec 7, 2024  |
| ARCHITECTURE.md      | 1.0     | Dec 7, 2024  |
| DATABASE_SCHEMA.md   | 1.0     | Dec 7, 2024  |
| API_DOCUMENTATION.md | 1.0     | Dec 7, 2024  |
| BLOCKCHAIN_DESIGN.md | 1.0     | Dec 7, 2024  |
| SECURITY.md          | 1.0     | Dec 7, 2024  |

---

## 📝 License

MIT License - This project is open source and available for educational purposes.

---

## 🙏 Acknowledgments

- Rust and Actix-web communities
- React team for amazing documentation
- Supabase for serverless database
- Tailwind CSS for utility-first styling
- All open-source contributors

---

**Last Updated**: December 7, 2024  
**Status**: Production Ready ✅
