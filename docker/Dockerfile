# Multi-stage build for production
FROM rust:latest AS builder

# Set working directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create dummy files for dependency caching
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs && echo "fn main() {}" > src/bin/token_service.rs

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Build dependencies (this layer will be cached)
RUN cargo build --release

# Copy source code
COPY src ./src

# Build the application (main binary only first)
RUN cargo build --release --bin verus-rpc-server && \
    find /app -name "verus-rpc-server" -type f && \
    ls -la /app/target/release/ && \
    echo "Main binary build completed successfully"

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r verus && useradd -r -g verus -s /bin/bash verus

# Create app directory
WORKDIR /app

# Copy main binary from builder
COPY --from=builder /app/target/release/verus-rpc-server /usr/local/bin/

# Copy startup script
COPY docker/scripts/wait-for-verus.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/wait-for-verus.sh

# Create config directory
RUN mkdir -p /app/config && chown -R verus:verus /app

# Switch to non-root user
USER verus

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f -H "User-Agent: Docker-Health-Check" http://localhost:8080/health || exit 1

# Default command
CMD ["verus-rpc-server"]
