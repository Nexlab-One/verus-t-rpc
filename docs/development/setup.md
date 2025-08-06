# Development Setup

Instructions for setting up a development environment for the Verus RPC Server project.

## 🛠️ Prerequisites

### Required Software

- **Rust**: 1.70 or higher
- **Git**: Latest version
- **Docker**: 20.10+ (optional, for Redis)
- **Docker Compose**: 2.0+ (optional, for full stack)

### System Requirements

- **OS**: Linux, macOS, or Windows (WSL2 recommended for Windows)
- **Memory**: 4GB+ RAM
- **Storage**: 10GB+ free space
- **Network**: Internet connection for dependencies

## 🚀 Quick Setup

### 1. Clone the Repository

```bash
git clone https://github.com/Nexlab-One/verus-t-rpc.git
cd verus-t-rpc
```

### 2. Install Rust

If you don't have Rust installed:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source ~/.bashrc  # or source ~/.zshrc

# Verify installation
rustc --version
cargo --version
```

### 3. Install Development Tools

```bash
# Install Rust components
rustup component add rustfmt
rustup component add clippy

# Install additional tools
cargo install cargo-audit
cargo install cargo-watch
cargo install cargo-expand
```

### 4. Build the Project

```bash
# Build in development mode
cargo build

# Build in release mode
cargo build --release
```

## 🔧 Development Configuration

### Development Configuration File

Create `config.toml` for development:

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_dev_rpc_user"
rpc_password = "your_dev_rpc_password"
timeout_seconds = 30
max_retries = 3

[server]
bind_address = "127.0.0.1"
port = 8080
max_request_size = 1048576
worker_threads = 2

[security]
development_mode = true  # Enable development mode
enable_security_headers = true
enable_custom_headers = false
cors_origins = ["*"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "dev-secret-key-32-chars-long"
expiration_seconds = 3600
issuer = "verus-rpc-server-dev"
audience = "verus-clients-dev"

[rate_limit]
enabled = true
requests_per_minute = 1000
burst_size = 100

[cache]
enabled = true
redis_url = "redis://127.0.0.1:6379"
default_ttl = 300
max_size = 104857600

[logging]
level = "debug"  # More verbose for development
format = "json"
structured = true
```

### Environment Variables

Set development environment variables:

```bash
# Development environment
export RUST_LOG=debug
export RUST_BACKTRACE=1
export VERUS_RPC_URL="http://127.0.0.1:27486"
export VERUS_RPC_USER="your_dev_user"
export VERUS_RPC_PASSWORD="your_dev_password"
export DEVELOPMENT_MODE="true"
```

## 🗄️ Database Setup

### Redis Setup (Optional)

For caching functionality:

```bash
# Using Docker
docker run -d --name redis-dev \
    -p 6379:6379 \
    redis:7-alpine

# Or install locally
# Ubuntu/Debian
sudo apt-get install redis-server

# macOS
brew install redis
brew services start redis

# Test Redis connection
redis-cli ping
```

### Verus Daemon Setup

Set up a local Verus daemon for development:

```bash
# Create Verus configuration
mkdir -p ~/.verus
cat > ~/.verus/verus.conf << EOF
rpcuser=your_dev_user
rpcpassword=your_dev_password
rpcport=27486
rpcallowip=127.0.0.1
server=1
daemon=1
txindex=1
EOF

# Start Verus daemon
verusd -conf=~/.verus/verus.conf
```

## 🧪 Testing Setup

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test --verbose

# Run specific test
cargo test test_name

# Run tests in release mode
cargo test --release

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Test Configuration

Create test-specific configuration:

```toml
# test-config.toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "test_user"
rpc_password = "test_password"
timeout_seconds = 5
max_retries = 1

[server]
bind_address = "127.0.0.1"
port = 8081  # Different port for tests
max_request_size = 1048576
worker_threads = 1

[security]
development_mode = true
enable_security_headers = false  # Disable for faster tests
enable_custom_headers = false
cors_origins = ["*"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "test-secret-key-32-chars-long"
expiration_seconds = 3600
issuer = "verus-rpc-server-test"
audience = "verus-clients-test"

[rate_limit]
enabled = false  # Disable for tests
requests_per_minute = 1000
burst_size = 100

[cache]
enabled = false  # Disable for tests
redis_url = "redis://127.0.0.1:6379"
default_ttl = 300
max_size = 104857600

[logging]
level = "error"  # Minimal logging for tests
format = "json"
structured = true
```

## 🔍 Development Tools

### IDE Setup

#### VS Code

Install recommended extensions:

```json
// .vscode/extensions.json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb",
    "ms-vscode.vscode-json"
  ]
}
```

#### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Configure Rust toolchain
3. Set up run configurations

### Code Quality Tools

#### Rustfmt

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

#### Clippy

```bash
# Run linter
cargo clippy

# Run linter with all warnings
cargo clippy -- -W clippy::all

# Run linter in release mode
cargo clippy --release
```

#### Security Audit

```bash
# Check for security vulnerabilities
cargo audit

# Update dependencies
cargo update
```

### Development Workflow

#### Cargo Watch

```bash
# Watch for changes and run tests
cargo watch -x test

# Watch for changes and run specific command
cargo watch -x 'run -- --config config.toml'

# Watch for changes and format code
cargo watch -x fmt
```

#### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with backtrace
RUST_BACKTRACE=1 cargo run

# Run specific binary
cargo run --bin verus-rpc-server
```

## 🐳 Docker Development

### Development Dockerfile

```dockerfile
# Development Dockerfile
FROM rust:1.70

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build

# Copy source code
COPY src ./src

# Build the application
RUN cargo build

# Expose port
EXPOSE 8080

# Run in development mode
CMD ["cargo", "run"]
```

### Docker Compose for Development

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  verus-rpc-server:
    build:
      context: .
      dockerfile: Dockerfile.dev
    container_name: verus-rpc-server-dev
    ports:
      - "8080:8080"
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/app/target
    environment:
      - RUST_LOG=debug
      - RUST_BACKTRACE=1
    depends_on:
      - redis
      - verus-daemon
    networks:
      - verus-dev-network

  redis:
    image: redis:7-alpine
    container_name: verus-redis-dev
    ports:
      - "6379:6379"
    networks:
      - verus-dev-network

  verus-daemon:
    image: verus/verus:latest
    container_name: verus-daemon-dev
    volumes:
      - verus-data:/root/.verus
      - ./verus.conf:/root/.verus/verus.conf:ro
    networks:
      - verus-dev-network
    command: verusd -conf=/root/.verus/verus.conf

volumes:
  cargo-cache:
  target-cache:
  verus-data:

networks:
  verus-dev-network:
    driver: bridge
```

## 🔧 Development Scripts

### Useful Scripts

Create development scripts in `scripts/` directory:

```bash
#!/bin/bash
# scripts/dev-setup.sh

echo "Setting up development environment..."

# Install Rust components
rustup component add rustfmt clippy

# Install cargo tools
cargo install cargo-audit cargo-watch cargo-expand

# Create development config
cp config.toml.example config.toml

# Start Redis (if using Docker)
docker run -d --name redis-dev -p 6379:6379 redis:7-alpine

echo "Development environment setup complete!"
```

```bash
#!/bin/bash
# scripts/dev-run.sh

echo "Starting development server..."

# Set development environment
export RUST_LOG=debug
export RUST_BACKTRACE=1
export DEVELOPMENT_MODE=true

# Run the server
cargo run -- --config config.toml
```

```bash
#!/bin/bash
# scripts/dev-test.sh

echo "Running development tests..."

# Run tests
cargo test --verbose

# Run linter
cargo clippy

# Run security audit
cargo audit

echo "Development tests complete!"
```

### Makefile

Create a `Makefile` for common development tasks:

```makefile
# Makefile

.PHONY: build test fmt clippy audit clean dev-setup dev-run

# Build the project
build:
	cargo build

# Build release version
build-release:
	cargo build --release

# Run tests
test:
	cargo test --verbose

# Format code
fmt:
	cargo fmt

# Run linter
clippy:
	cargo clippy

# Security audit
audit:
	cargo audit

# Clean build artifacts
clean:
	cargo clean

# Setup development environment
dev-setup:
	./scripts/dev-setup.sh

# Run development server
dev-run:
	./scripts/dev-run.sh

# Run all checks
check: fmt clippy test audit

# Install development dependencies
install-dev:
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-watch cargo-expand

# Start development services
dev-services:
	docker-compose -f docker-compose.dev.yml up -d

# Stop development services
dev-services-stop:
	docker-compose -f docker-compose.dev.yml down
```

## 📊 Development Monitoring

### Local Monitoring

```bash
# Monitor server logs
cargo run 2>&1 | tee server.log

# Monitor with structured logging
RUST_LOG=debug cargo run | jq '.'

# Monitor system resources
htop
```

### Development Metrics

Access development metrics:

```bash
# Health check
curl http://127.0.0.1:8080/health

# Metrics
curl http://127.0.0.1:8080/metrics

# Prometheus format
curl http://127.0.0.1:8080/prometheus
```

## 🚨 Troubleshooting

### Common Issues

#### Build Issues

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

#### Test Issues

```bash
# Run tests with more output
cargo test -- --nocapture

# Run specific test with output
cargo test test_name -- --nocapture

# Check test configuration
cargo test --help
```

#### Dependency Issues

```bash
# Update Rust
rustup update

# Check for outdated dependencies
cargo outdated

# Update specific dependency
cargo update -p dependency_name
```

### Debugging Tips

1. **Use debug logging**: Set `RUST_LOG=debug`
2. **Enable backtraces**: Set `RUST_BACKTRACE=1`
3. **Use cargo expand**: `cargo expand` to see macro expansions
4. **Use cargo tree**: `cargo tree` to see dependency tree
5. **Use cargo doc**: `cargo doc --open` to view documentation

## 🔗 Related Documentation

- [Testing Guide](./testing.md) - Comprehensive testing guide
- [Code Standards](./code-standards.md) - Coding standards and conventions
- [Contributing Guidelines](./contributing.md) - How to contribute
- [Architecture Documentation](../architecture/system-architecture.md) - System architecture

