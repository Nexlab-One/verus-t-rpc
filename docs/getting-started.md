# Getting Started

Welcome to the **Rust Verus RPC Server**! This guide will help you get up and running quickly.

## 🚀 Quick Start

### Prerequisites

- **Rust** (1.70 or higher)
- **Verus Daemon** (verusd) running and accessible
- **Redis** (optional, for caching)

### 1. Installation

```bash
# Clone the repository
git clone https://github.com/Nexlab-One/verus-t-rpc.git
cd verus-t-rpc

# Build the project
cargo build --release
```

### 2. Configuration

Create a `config.toml` file in the project root:

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_rpc_username"
rpc_password = "your_rpc_password"
timeout_seconds = 30
max_retries = 3

[server]
bind_address = "127.0.0.1"
port = 8080
max_request_size = 1048576
worker_threads = 4

[security]
development_mode = true  # Set to false for production
enable_security_headers = true
enable_custom_headers = false
cors_origins = ["*"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"

[rate_limit]
enabled = true
requests_per_minute = 100
burst_size = 20

[cache]
enabled = true
redis_url = "redis://127.0.0.1:6379"
default_ttl = 300
max_size = 104857600

[logging]
level = "info"
format = "json"
structured = true
```

### 3. Running the Server

```bash
# Development mode
cargo run

# Production mode
cargo run --release

# Or run the compiled binary
./target/release/verus-rpc-server
```

The server will start and listen on the configured address and port.

## 🔧 First Steps

### 1. Verify Server is Running

```bash
# Check if the server is responding
curl http://127.0.0.1:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "details": {
    "timestamp": "2024-12-06T15:30:00Z",
    "version": "1.0.0",
    "uptime": "0d 0h 5m",
    "daemon": {
      "available": true,
      "circuit_breaker": "Closed",
      "status": "connected"
    },
    "system": {
      "memory_usage": "N/A",
      "cpu_usage": "N/A",
      "active_connections": 0
    }
  }
}
```

### 2. Make Your First RPC Call

```bash
# Basic RPC call (development mode)
curl -X POST http://127.0.0.1:8080/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "version": 123456,
    "protocolversion": 123456,
    "walletversion": 123456,
    "balance": 0.0,
    "blocks": 123456,
    "timeoffset": 0,
    "connections": 8,
    "proxy": "",
    "difficulty": 123456.789,
    "testnet": false,
    "keypoololdest": 1234567890,
    "keypoolsize": 100,
    "unlocked_until": 0,
    "paytxfee": 0.0001,
    "relayfee": 0.00001,
    "errors": ""
  },
  "id": 1
}
```

### 3. Check Metrics

```bash
# View Prometheus metrics
curl http://127.0.0.1:8080/metrics
```

## 🔐 Authentication (Production)

For production use, you'll need to authenticate your requests:

### 1. Generate JWT Token

```bash
# This is typically done by your application
# Example token generation (you'll need to implement this)
curl -X POST http://127.0.0.1:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "your_username",
    "password": "your_password"
  }'
```

### 2. Use JWT Token in Requests

```bash
# Authenticated RPC call
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

## 📊 Monitoring

### Health Check
```bash
curl http://127.0.0.1:8080/health
```

### Metrics
```bash
curl http://127.0.0.1:8080/metrics
```

### Prometheus Format
```bash
curl http://127.0.0.1:8080/prometheus
```

## 🧪 Testing

Run the test suite to verify everything is working:

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test --verbose

# Run in release mode
cargo test --release
```

## 🚨 Common Issues

### 1. Connection Refused
- **Problem**: Can't connect to Verus daemon
- **Solution**: Ensure verusd is running and accessible at the configured RPC URL

### 2. Authentication Failed
- **Problem**: JWT token rejected
- **Solution**: Check JWT secret key and token expiration

### 3. Method Not Allowed
- **Problem**: RPC method not in allowlist
- **Solution**: Check the method allowlist in `src/allowlist.rs`

### 4. Rate Limited
- **Problem**: Too many requests
- **Solution**: Adjust rate limiting configuration or implement request throttling

## 📚 Next Steps

1. **Read the [API Reference](../api/request-response.md)** to understand all available endpoints
2. **Review [Security Best Practices](../security/best-practices.md)** for production deployment
3. **Check [Deployment Guide](../deployment/production.md)** for production setup
4. **Explore [Architecture Documentation](../architecture/system-architecture.md)** to understand the system design

## 🆘 Need Help?

- 📖 **Documentation**: Check the [main documentation](../README.md)
- 🐛 **Issues**: Create an issue on [GitHub](https://github.com/Nexlab/rust_verusd_rpc_server/issues)
- 💬 **Discussions**: Use [GitHub Discussions](https://github.com/Nexlab/rust_verusd_rpc_server/discussions)

---

**Ready to go?** Check out the [API Reference](../api/request-response.md) to start making RPC calls!
