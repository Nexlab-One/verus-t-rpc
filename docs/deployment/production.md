# Production Deployment

This guide provides comprehensive instructions for deploying the Verus RPC Server in production environments with security, performance, and reliability considerations.

## 🚀 Production Architecture

### Recommended Production Setup

```
┌─────────────────────────────────────────────────────────────────┐
│                    Load Balancer (Optional)                     │
│                    • Nginx / HAProxy / Cloud Load Balancer      │
├─────────────────────────────────────────────────────────────────┤
│                    Reverse Proxy                                │
│                    • Caddy (recommended) / Nginx / Traefik      │
│                    • SSL/TLS Termination                        │
│                    • Rate Limiting                              │
│                    • Compression                                │
├─────────────────────────────────────────────────────────────────┤
│                    Verus RPC Server                             │
│                    • Containerized Application                  │
│                    • Health Checks                              │
│                    • Metrics Endpoint                           │
├─────────────────────────────────────────────────────────────────┤
│                    External Services                            │
│                    • Redis (Caching)                            │
│                    • Verus Daemon                               │
│                    • Monitoring Stack                           │
└─────────────────────────────────────────────────────────────────┘
```

## 📋 Prerequisites

### System Requirements

- **CPU**: 2+ cores (4+ recommended for high load)
- **Memory**: 4GB+ RAM (8GB+ recommended)
- **Storage**: 20GB+ SSD storage
- **Network**: Stable internet connection
- **OS**: Linux (Ubuntu 20.04+ / CentOS 8+ / Debian 11+)

### Software Requirements

- **Docker**: 20.10+ (for containerized deployment)
- **Docker Compose**: 2.0+ (for multi-service deployment)
- **Redis**: 6.0+ (for caching)
- **Verus Daemon**: Latest stable version
- **Reverse Proxy**: Caddy (recommended) / Nginx / Traefik

## 🔧 Configuration

### Production Configuration

Create a production `config.toml`:

```toml
[verus]
rpc_url = "http://verus-daemon:27486"
rpc_user = "your_production_rpc_user"
rpc_password = "your_production_rpc_password"
timeout_seconds = 30
max_retries = 3

[server]
bind_address = "0.0.0.0"  # Listen on all interfaces
port = 8080
max_request_size = 1048576
worker_threads = 8  # Adjust based on CPU cores

[security]
development_mode = false  # CRITICAL: Disable development mode
enable_security_headers = true
enable_custom_headers = true
cors_origins = ["https://yourdomain.com", "https://app.yourdomain.com"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "your-32-character-cryptographically-secure-secret-key"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"

[rate_limit]
enabled = true
requests_per_minute = 1000  # Higher limit for production
burst_size = 100

[cache]
enabled = true
redis_url = "redis://redis:6379"
default_ttl = 300
max_size = 104857600

[logging]
level = "info"
format = "json"
structured = true
```

### Environment Variables

Set production environment variables:

```bash
# Application
export VERUS_RPC_URL="http://verus-daemon:27486"
export VERUS_RPC_USER="your_production_user"
export VERUS_RPC_PASSWORD="your_production_password"
export SERVER_PORT="8080"
export JWT_SECRET_KEY="your-secure-jwt-secret"

# Security
export DEVELOPMENT_MODE="false"
export ENABLE_SECURITY_HEADERS="true"
export CORS_ORIGINS="https://yourdomain.com,https://app.yourdomain.com"

# Performance
export WORKER_THREADS="8"
export MAX_REQUEST_SIZE="1048576"
export RATE_LIMIT_REQUESTS_PER_MINUTE="1000"

# Caching
export REDIS_URL="redis://redis:6379"
export CACHE_ENABLED="true"
export CACHE_DEFAULT_TTL="300"
```

## 🐳 Docker Deployment

### Dockerfile

```dockerfile
# Multi-stage build for production
FROM rust:1.70 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Production stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false verus

# Copy binary
COPY --from=builder /app/target/release/verus-rpc-server /usr/local/bin/

# Create config directory
RUN mkdir -p /app/config && chown verus:verus /app/config

# Switch to non-root user
USER verus

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["verus-rpc-server"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  verus-rpc-server:
    build: .
    container_name: verus-rpc-server
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"  # Only bind to localhost
    volumes:
      - ./config:/app/config:ro
    environment:
      - RUST_LOG=info
      - RUST_BACKTRACE=0
    depends_on:
      - redis
      - verus-daemon
    networks:
      - verus-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  redis:
    image: redis:7-alpine
    container_name: verus-redis
    restart: unless-stopped
    command: redis-server --appendonly yes --maxmemory 256mb --maxmemory-policy allkeys-lru
    volumes:
      - redis-data:/data
    networks:
      - verus-network
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  verus-daemon:
    image: verus/verus:latest
    container_name: verus-daemon
    restart: unless-stopped
    volumes:
      - verus-data:/root/.verus
      - ./verus.conf:/root/.verus/verus.conf:ro
    networks:
      - verus-network
    command: verusd -conf=/root/.verus/verus.conf

  caddy:
    image: caddy:2-alpine
    container_name: verus-caddy
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy-data:/data
      - caddy-config:/config
    depends_on:
      - verus-rpc-server
    networks:
      - verus-network

volumes:
  redis-data:
  verus-data:
  caddy-data:
  caddy-config:

networks:
  verus-network:
    driver: bridge
```

### Caddy Configuration

Create a `Caddyfile` for Caddy reverse proxy:

```caddyfile
# Caddyfile
yourdomain.com {
    # Automatic HTTPS with Let's Encrypt
    tls your-email@domain.com
    
    # Rate limiting
    rate_limit {
        zone api
        events 1000
        window 1m
    }
    
    # Security headers
    header {
        # Security headers
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        Permissions-Policy "geolocation=(), microphone=(), camera=()"
        
        # Remove server header
        -Server
        
        # Cache control for API responses
        Cache-Control "no-cache, no-store, must-revalidate"
        Pragma "no-cache"
        Expires "0"
    }
    
    # Gzip compression
    encode gzip
    
    # Reverse proxy to Verus RPC Server
    reverse_proxy verus-rpc-server:8080 {
        # Health checks
        health_uri /health
        health_interval 30s
        health_timeout 10s
        
        # Load balancing (if multiple instances)
        lb_policy round_robin
        
        # Timeouts
        timeout 30s
        
        # Headers
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
    
    # Logging
    log {
        output file /var/log/caddy/verus-rpc.log
        format json
        level INFO
    }
}

# Health check endpoint (internal only)
:8081 {
    bind 127.0.0.1
    
    reverse_proxy verus-rpc-server:8080 {
        health_uri /health
        health_interval 30s
        health_timeout 10s
    }
}

# Metrics endpoint (internal only)
:8082 {
    bind 127.0.0.1
    
    reverse_proxy verus-rpc-server:8080 {
        health_uri /metrics
        health_interval 30s
        health_timeout 10s
    }
}
```

### Alternative Caddy Configuration (Simplified)

For a simpler setup without automatic HTTPS:

```caddyfile
# Simple Caddyfile
:80 {
    # Rate limiting
    rate_limit {
        zone api
        events 1000
        window 1m
    }
    
    # Security headers
    header {
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        -Server
        Cache-Control "no-cache, no-store, must-revalidate"
    }
    
    # Gzip compression
    encode gzip
    
    # Reverse proxy
    reverse_proxy verus-rpc-server:8080 {
        health_uri /health
        health_interval 30s
        health_timeout 10s
    }
}
```

## 🔒 Security Hardening

### Container Security

```bash
# Run container with security options
docker run --security-opt no-new-privileges \
    --cap-drop ALL \
    --read-only \
    --tmpfs /tmp \
    -p 127.0.0.1:8080:8080 \
    verus-rpc-server
```

### Network Security

```bash
# Create isolated network
docker network create --driver bridge --internal verus-internal

# Only expose necessary ports
docker run --network verus-internal \
    --publish 127.0.0.1:8080:8080 \
    verus-rpc-server
```

### File Permissions

```bash
# Secure file permissions
chmod 600 config.toml
chown verus:verus config.toml
chmod 700 /app/config
chown verus:verus /app/config
```

## 📊 Monitoring & Observability

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'verus-rpc-server'
    static_configs:
      - targets: ['verus-rpc-server:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
```

### Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Verus RPC Server",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_requests_total[5m])",
            "legendFormat": "{{method}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(verus_rpc_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_errors_total[5m])",
            "legendFormat": "{{error_type}}"
          }
        ]
      }
    ]
  }
}
```

### Logging Configuration

```yaml
# logrotate configuration
/var/log/verus-rpc-server/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 verus verus
    postrotate
        docker exec verus-rpc-server kill -USR1 1
    endscript
}
```

## 🔄 High Availability

### Load Balancer Configuration

```yaml
# HAProxy configuration
global
    maxconn 4096
    log stdout format raw local0 info

defaults
    mode http
    timeout connect 5s
    timeout client 50s
    timeout server 50s

frontend verus_api
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/verus.pem
    redirect scheme https if !{ ssl_fc }
    
    acl is_health path /health
    acl is_metrics path /metrics
    
    # Health check (allow internal)
    use_backend health if is_health { src 127.0.0.1 }
    use_backend health if is_health { src 10.0.0.0/8 }
    
    # Metrics (allow internal)
    use_backend metrics if is_metrics { src 127.0.0.1 }
    use_backend metrics if is_metrics { src 10.0.0.0/8 }
    
    # API requests
    default_backend verus_api

backend verus_api
    balance roundrobin
    option httpchk GET /health
    server verus1 10.0.1.10:8080 check
    server verus2 10.0.1.11:8080 check
    server verus3 10.0.1.12:8080 check

backend health
    server verus1 10.0.1.10:8080 check

backend metrics
    server verus1 10.0.1.10:8080 check
```

### Auto-Scaling

```yaml
# Kubernetes HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: verus-rpc-server
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: verus-rpc-server
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

## 🚨 Backup & Recovery

### Configuration Backup

```bash
#!/bin/bash
# backup-config.sh

BACKUP_DIR="/backup/verus-rpc"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup configuration
cp config.toml "$BACKUP_DIR/config_$DATE.toml"

# Backup SSL certificates
cp -r ssl "$BACKUP_DIR/ssl_$DATE"

# Compress backup
tar -czf "$BACKUP_DIR/backup_$DATE.tar.gz" -C "$BACKUP_DIR" .

# Clean old backups (keep last 7 days)
find "$BACKUP_DIR" -name "backup_*.tar.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/backup_$DATE.tar.gz"
```

### Recovery Procedure

```bash
#!/bin/bash
# restore-config.sh

BACKUP_FILE="$1"
BACKUP_DIR="/tmp/restore_$$"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# Extract backup
mkdir -p "$BACKUP_DIR"
tar -xzf "$BACKUP_FILE" -C "$BACKUP_DIR"

# Stop services
docker-compose down

# Restore configuration
cp "$BACKUP_DIR"/config_*.toml config.toml
cp -r "$BACKUP_DIR"/ssl_* ssl

# Start services
docker-compose up -d

# Cleanup
rm -rf "$BACKUP_DIR"

echo "Recovery completed"
```

## 🔧 Maintenance

### Regular Maintenance Tasks

```bash
#!/bin/bash
# maintenance.sh

# Update containers
docker-compose pull
docker-compose up -d

# Clean up old images
docker image prune -f

# Check disk space
df -h

# Check logs
docker-compose logs --tail=100

# Health check
curl -f http://localhost:8080/health
```

### Performance Tuning

```toml
# Performance tuning in config.toml
[server]
worker_threads = 8  # Match CPU cores
max_request_size = 2097152  # 2MB for larger requests

[rate_limit]
requests_per_minute = 2000  # Increase for high load
burst_size = 200

[cache]
default_ttl = 600  # Increase cache TTL
max_size = 209715200  # 200MB cache
```

## 🔗 Related Documentation

- [Docker Deployment](./docker.md) - Detailed Docker setup
- [Reverse Proxy Setup](./reverse-proxy.md) - Caddy/Nginx configuration
- [Environment Configuration](./environment.md) - Environment variables
- [Monitoring & Logging](./monitoring.md) - Monitoring setup
- [Redis Setup](./redis-setup.md) - Redis configuration
