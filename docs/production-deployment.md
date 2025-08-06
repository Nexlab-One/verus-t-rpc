# Production Deployment Guide

## Overview

This guide covers deploying the Verus RPC Server as a secure, publicly available service with a separate token issuance service.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Client Apps   │     │  Token Service  │     │  Verus Daemon   │
│                 │     │   (Port 8081)   │     │   (Port 27486)  │
└─────────┬───────┘     └─────────┬───────┘     └─────────┬───────┘
          │                       │                       │
          │ 1. Request Token      │                       │
          │─────────────────────▶│                       │
          │                       │                       │
          │ 2. Return JWT         │                       │
          │◀─────────────────────│                       │
          │                       │                       │
          │ 3. RPC Request        │                       │
          │   + JWT Token         │                       │
          │─────────────────────▶│                       │
          │                       │                       │
          │                       │ 4. Forward to         │
          │                       │    Verus Daemon       │
          │                       │─────────────────────▶│
          │                       │                       │
          │                       │ 5. Response           │
          │                       │◀─────────────────────│
          │                       │                       │
          │ 6. RPC Response       │                       │
          │◀─────────────────────│                       │
          │                       │                       │
```

## Prerequisites

- **Rust 1.70+**
- **Redis** (for caching)
- **Verus Daemon** (verusd) running
- **Reverse Proxy** (nginx/Caddy) for SSL termination
- **Firewall** configured

## Step 1: Build the Services

```bash
# Build both services
cargo build --release

# The binaries will be:
# - target/release/verus-rpc-server (main RPC server)
# - target/release/token-service (token issuance service)
```

## Step 2: Configure the Token Service

Create `token-service.conf`:

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_rpc_username"
rpc_password = "your_rpc_password"

[server]
bind_address = "127.0.0.1"  # Internal only
port = 8081

[security]
development_mode = false

[security.jwt]
secret_key = "your-super-secret-jwt-key-that-is-at-least-32-characters-long"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"
```

## Step 3: Configure the Main RPC Server

Use the provided `Conf.production.toml` and update:

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_rpc_username"
rpc_password = "your_rpc_password"

[server]
bind_address = "0.0.0.0"  # Public access
port = 8080

[security]
development_mode = false  # CRITICAL: Must be false

[security.jwt]
# Use the SAME secret key as token service
secret_key = "your-super-secret-jwt-key-that-is-at-least-32-characters-long"
```

## Step 4: Set Up Reverse Proxy (nginx)

Create `/etc/nginx/sites-available/verus-rpc`:

```nginx
# Upstream for RPC server
upstream verus_rpc {
    server 127.0.0.1:8080;
}

# Upstream for token service (internal only)
upstream token_service {
    server 127.0.0.1:8081;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    # SSL Configuration
    ssl_certificate /path/to/your/certificate.crt;
    ssl_certificate_key /path/to/your/private.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    # Security Headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

    # Rate Limiting
    limit_req_zone $binary_remote_addr zone=rpc:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=token:10m rate=5r/s;

    # Main RPC endpoint
    location / {
        limit_req zone=rpc burst=20 nodelay;
        
        proxy_pass http://verus_rpc;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # Token service (internal only)
    location /auth/ {
        # Only allow internal access
        allow 127.0.0.1;
        deny all;
        
        limit_req zone=token burst=10 nodelay;
        
        proxy_pass http://token_service/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Health check
    location /health {
        proxy_pass http://verus_rpc/health;
    }

    # Metrics (restrict access)
    location /metrics {
        allow 127.0.0.1;
        allow 10.0.0.0/8;
        deny all;
        
        proxy_pass http://verus_rpc/metrics;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/verus-rpc /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## Step 5: Create Systemd Services

### Token Service

Create `/etc/systemd/system/verus-token.service`:

```ini
[Unit]
Description=Verus Token Issuance Service
After=network.target

[Service]
Type=simple
User=verus
Group=verus
WorkingDirectory=/opt/verus
ExecStart=/opt/verus/token-service
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

### RPC Server

Create `/etc/systemd/system/verus-rpc.service`:

```ini
[Unit]
Description=Verus RPC Server
After=network.target redis.service
Requires=redis.service

[Service]
Type=simple
User=verus
Group=verus
WorkingDirectory=/opt/verus
ExecStart=/opt/verus/verus-rpc-server
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

## Step 6: Deploy and Start Services

```bash
# Create user
sudo useradd -r -s /bin/false verus

# Create directory
sudo mkdir -p /opt/verus
sudo chown verus:verus /opt/verus

# Copy binaries
sudo cp target/release/verus-rpc-server /opt/verus/
sudo cp target/release/token-service /opt/verus/
sudo chown verus:verus /opt/verus/*

# Copy configs
sudo cp Conf.production.toml /opt/verus/
sudo cp token-service.conf /opt/verus/
sudo chown verus:verus /opt/verus/*.toml

# Start services
sudo systemctl daemon-reload
sudo systemctl enable verus-token
sudo systemctl enable verus-rpc
sudo systemctl start verus-token
sudo systemctl start verus-rpc

# Check status
sudo systemctl status verus-token
sudo systemctl status verus-rpc
```

## Step 7: Client Integration

### Getting a Token

```bash
# Request a token from your authentication service
curl -X POST https://yourdomain.com/auth/issue \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "client_app_1",
    "permissions": ["read", "write"],
    "client_ip": "203.0.113.1",
    "user_agent": "MyApp/1.0"
  }'
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "token_id": "uuid-here"
}
```

### Making RPC Calls

```bash
# Use the token for RPC calls
curl -X POST https://yourdomain.com/ \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }'
```

## Step 8: Security Hardening

### Firewall Configuration

```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 80/tcp    # HTTP (redirect)
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 6379/tcp  # Redis (if external)
sudo ufw deny 8080/tcp   # Block direct RPC access
sudo ufw deny 8081/tcp   # Block direct token access
sudo ufw enable
```

### Redis Security

```bash
# Edit /etc/redis/redis.conf
bind 127.0.0.1
requirepass your_redis_password
maxmemory 256mb
maxmemory-policy allkeys-lru
```

### File Permissions

```bash
# Secure config files
sudo chmod 600 /opt/verus/*.toml
sudo chown verus:verus /opt/verus/*.toml

# Secure binaries
sudo chmod 755 /opt/verus/verus-rpc-server
sudo chmod 755 /opt/verus/token-service
```

## Step 9: Monitoring and Logging

### Log Rotation

Create `/etc/logrotate.d/verus-rpc`:

```
/var/log/verus-rpc.log {
    daily
    missingok
    rotate 52
    compress
    delaycompress
    notifempty
    create 644 verus verus
    postrotate
        systemctl reload verus-rpc
    endscript
}
```

### Health Checks

```bash
# Check RPC server health
curl https://yourdomain.com/health

# Check token service health
curl http://127.0.0.1:8081/health

# Monitor logs
sudo journalctl -u verus-rpc -f
sudo journalctl -u verus-token -f
```

## Step 10: Backup and Recovery

### Configuration Backup

```bash
# Backup configs
sudo cp /opt/verus/*.toml /backup/verus/
sudo cp /etc/nginx/sites-available/verus-rpc /backup/nginx/

# Backup systemd services
sudo cp /etc/systemd/system/verus-*.service /backup/systemd/
```

### Recovery Procedure

```bash
# Stop services
sudo systemctl stop verus-rpc verus-token

# Restore from backup
sudo cp /backup/verus/*.toml /opt/verus/
sudo cp /backup/systemd/verus-*.service /etc/systemd/system/

# Restart services
sudo systemctl daemon-reload
sudo systemctl start verus-token verus-rpc
```

## Troubleshooting

### Common Issues

1. **Token Validation Fails**
   - Check JWT secret key matches between services
   - Verify token expiration
   - Check issuer/audience settings

2. **Rate Limiting**
   - Adjust rate limits in nginx and application config
   - Monitor logs for rate limit violations

3. **Connection Issues**
   - Verify Verus daemon is running
   - Check firewall rules
   - Test connectivity to Redis

### Debug Mode

For debugging, temporarily enable debug logging:

```bash
# Edit systemd service files
Environment=RUST_LOG=debug

# Reload and restart
sudo systemctl daemon-reload
sudo systemctl restart verus-rpc verus-token
```

## Performance Tuning

### Redis Optimization

```bash
# Edit /etc/redis/redis.conf
maxmemory 512mb
maxmemory-policy allkeys-lru
save 900 1
save 300 10
save 60 10000
```

### Application Tuning

```toml
[server]
worker_threads = 8  # Adjust based on CPU cores

[rate_limit]
requests_per_minute = 2000  # Adjust based on capacity
burst_size = 200
```

## Security Checklist

- [ ] Development mode disabled
- [ ] JWT secret key is 32+ characters
- [ ] SSL/TLS enabled with strong ciphers
- [ ] Firewall configured
- [ ] Rate limiting enabled
- [ ] Security headers configured
- [ ] File permissions secured
- [ ] Redis password set
- [ ] Logging enabled
- [ ] Monitoring configured
- [ ] Backup strategy in place

This deployment provides a secure, scalable, and production-ready Verus RPC server with proper authentication and authorization.
