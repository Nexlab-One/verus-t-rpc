# Rust Verus RPC Server

A **production-ready** RPC server for Verus blockchain built in Rust. Acts as a secure proxy between clients and the Verus daemon with comprehensive validation and security controls.

## 🚀 Status:

- ✅ **71/71 Tests Passing** (100% test coverage)
- ✅ **Zero Compiler Warnings** (Clean codebase)
- ✅ **Enterprise Security** (JWT auth, rate limiting, security headers)

## 🎯 Key Features

- **🔒 Security**: JWT authentication, rate limiting, security headers, method validation
- **⚡ Performance**: Redis caching, async processing
- **📊 Monitoring**: Prometheus metrics, structured logging, health checks
- **🏗️ Architecture**: Clean architecture with domain-driven design

## 📋 Prerequisites

- **Rust** (1.70+)
- **Redis** (optional, for caching)
- **Verus Daemon** (verusd) running
- **Git**

## 🚀 Quick Start

### 1. Installation

```bash
git clone https://github.com/Nexlab/verus-t-rpc.git
cd verus-t-rpc
cargo build --release
```

### 2. Configuration

Create `config.toml`:

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_rpc_username"
rpc_password = "your_rpc_password"

[server]
bind_address = "127.0.0.1"
port = 8080

[security]
development_mode = false
enable_security_headers = true

[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600

[rate_limit]
enabled = true
requests_per_minute = 100

[cache]
enabled = true
redis_url = "redis://127.0.0.1:6379"
```

### 3. Running

```bash
# Development
cargo run

# Production
cargo run --release
```

### 4. Making Requests

```bash
# Basic RPC call
curl -X POST http://127.0.0.1:8080/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }'

# With authentication
curl -X POST http://127.0.0.1:8080/ \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getblock",
    "params": ["block_hash", true],
    "id": 1
  }'
```

## 📚 Supported Methods

**60+ Verus RPC methods** with comprehensive validation:

- **Blockchain**: `getinfo`, `getblockchaininfo`, `getblock`, `getblockhash`
- **Transactions**: `getrawtransaction`, `sendrawtransaction`, `createrawtransaction`
- **Addresses**: `getaddressbalance`, `getaddressutxos`, `getaddressmempool`
- **Identities**: `getidentity`, `registeridentity`, `updateidentity`
- **Currencies**: `getcurrency`, `sendcurrency`, `listcurrencies`

## 🛡️ Security Features

- **JWT Authentication**: Token-based authentication with expiration
- **Rate Limiting**: IP-based request throttling
- **Security Headers**: CSP, XSS protection, clickjacking prevention
- **Method Validation**: Only pre-approved methods allowed
- **Input Validation**: Strict parameter type checking

## 📊 Monitoring

- **Health Check**: `/health`
- **Metrics**: `/metrics` (Prometheus format)
- **Structured Logging**: JSON format with request/response tracking

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test --verbose

# Run in release mode
cargo test --release
```

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│           HTTP Layer                │
│        (Warp Framework)             │
├─────────────────────────────────────┤
│        Infrastructure Layer         │
│  (HTTP Server, Cache, Monitoring)   │
├─────────────────────────────────────┤
│        Application Layer            │
│    (Use Cases, Services)            │
├─────────────────────────────────────┤
│          Domain Layer               │
│    (Business Logic, Models)         │
└─────────────────────────────────────┘
```

## 🔧 Development

```bash
# Build
cargo build --release

# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit
```

## 🚀 Production Deployment

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/verus-rpc-server /usr/local/bin/
EXPOSE 8080
CMD ["verus-rpc-server"]
```

### Environment Variables

```bash
export VERUS_RPC_URL="http://127.0.0.1:27486"
export VERUS_RPC_USER="your_username"
export VERUS_RPC_PASSWORD="your_password"
export SERVER_PORT="8080"
export JWT_SECRET_KEY="your-secret-key"
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 📖 Documentation

For comprehensive documentation, see the [docs/](docs/) directory:

- **[Getting Started](docs/getting-started.md)** - Quick start guide
- **[API Reference](docs/api/)** - Complete API documentation
- **[Architecture](docs/architecture/)** - System design and architecture
- **[Security](docs/security/)** - Security features and best practices
- **[Deployment](docs/deployment/)** - Production deployment guides
- **[Development](docs/development/)** - Development setup and guidelines
- **[Monitoring](docs/monitoring/)** - Monitoring and observability
- **[Secure Deployment Summary](docs/SECURE_DEPLOYMENT_SUMMARY.md)** - Complete implementation summary
- **[Local Development Guide](docs/test_local_access.md)** - Local development without authentication

## 📁 Project Structure

```
rust_verusd_rpc_server/
├── docs/                    # Documentation
│   ├── SECURE_DEPLOYMENT_SUMMARY.md
│   ├── test_local_access.md
│   └── ...
├── examples/                # Example code and scripts
│   ├── test_secure_deployment.sh
│   ├── test_project/        # Example Rust client
│   └── dex_client_example.js
├── src/                     # Source code
├── Conf.toml               # Development configuration
├── Conf.production.toml    # Production configuration
└── Conf.public-dex.toml    # Public DEX configuration
```

## 🆘 Support

- 📖 **Documentation**: Check the [documentation](docs/) directory
- 🐛 **Issues**: Create an issue on GitHub
- 💬 **Discussions**: Use GitHub Discussions

---

**Built with ❤️ by Nexlab-One**
If you find this project valuable, please consider supporting: zs10u0vvlchlv0yuew4a87cvpesrvdl2yc7dda0q9kjdg4lmezsx4nmj88nna2vcd0m4hmc2eg948c  
Every contribution helps us keep building. Thank you!