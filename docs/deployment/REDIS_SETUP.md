# Redis Setup Guide for Verus RPC Server

This guide explains how to set up Redis for the Verus RPC server's caching functionality.

## Usage in Payments

- Payment sessions are stored under keys `payments:{payment_id}` (JSON-serialized sessions)
- Revoked JWT IDs are stored under keys `jwt:revoked:{jti}` with TTL
- If Redis is unavailable, both stores fall back to in-memory, preserving functionality for a single instance

## Quick Start (No Authentication)

### 1. Install Redis

**Windows:**
```bash
# Using Chocolatey
choco install redis-64

# Or download from https://github.com/microsoftarchive/redis/releases
```

**macOS:**
```bash
# Using Homebrew
brew install redis
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install redis-server
```

### 2. Start Redis Server

**Windows:**
```bash
redis-server
```

**macOS/Linux:**
```bash
# Start Redis service
sudo systemctl start redis-server

# Or start manually
redis-server
```

### 3. Test Redis Connection

```bash
redis-cli ping
# Should return: PONG
```

### 4. Enable Caching in Configuration

Edit `Conf.toml`:
```toml
[cache]
enabled = true
redis_url = "redis://127.0.0.1:6379"
```

## Redis with Authentication

### 1. Configure Redis Password

Edit Redis configuration file (`redis.conf`):

```conf
# Set a password
requirepass your_secure_password

# Optional: Set username (Redis 6.0+)
user default on >your_secure_password ~* &* +@all
```

### 2. Restart Redis

```bash
# Stop Redis
redis-cli shutdown

# Start with config
redis-server redis.conf
```

### 3. Test Authentication

```bash
redis-cli
AUTH your_secure_password
ping
```

### 4. Update Configuration

Edit `Conf.toml`:
```toml
[cache]
enabled = true
redis_url = "redis://:your_secure_password@127.0.0.1:6379"
```

## Redis with Username/Password (ACL)

### 1. Create Redis User

```bash
redis-cli
ACL SETUSER cacheuser on >cache_password ~* &* +@all
```

### 2. Update Configuration

```toml
[cache]
enabled = true
redis_url = "redis://cacheuser:cache_password@127.0.0.1:6379"
```

## Redis with SSL/TLS

### 1. Generate SSL Certificate

```bash
# Generate self-signed certificate
openssl req -x509 -newkey rsa:4096 -keyout redis.key -out redis.crt -days 365 -nodes
```

### 2. Configure Redis SSL

Edit `redis.conf`:
```conf
tls-port 6380
tls-cert-file redis.crt
tls-key-file redis.key
tls-ca-cert-file redis.crt
```

### 3. Update Configuration

```toml
[cache]
enabled = true
redis_url = "rediss://127.0.0.1:6380"
```

## Troubleshooting

### Redis Connection Failed

1. **Check if Redis is running:**
   ```bash
   redis-cli ping
   ```

2. **Check Redis port:**
   ```bash
   netstat -an | grep 6379
   ```

3. **Check Redis logs:**
   ```bash
   tail -f /var/log/redis/redis-server.log
   ```

### Authentication Failed

1. **Verify password:**
   ```bash
   redis-cli
   AUTH your_password
   ```

2. **Check Redis configuration:**
   ```bash
   redis-cli CONFIG GET requirepass
   ```

### Memory Issues

1. **Check Redis memory usage:**
   ```bash
   redis-cli INFO memory
   ```

2. **Configure memory limits in `redis.conf`:**
   ```conf
   maxmemory 100mb
   maxmemory-policy allkeys-lru
   ```

## Performance Tuning

### 1. Optimize Redis Configuration

Edit `redis.conf`:
```conf
# Enable persistence
save 900 1
save 300 10
save 60 10000

# Memory optimization
maxmemory 100mb
maxmemory-policy allkeys-lru

# Network optimization
tcp-keepalive 300
```

### 2. Monitor Performance

```bash
# Monitor Redis commands
redis-cli MONITOR

# Check cache statistics
redis-cli INFO stats
```

## Security Best Practices

1. **Use strong passwords**
2. **Enable SSL/TLS in production**
3. **Restrict network access**
4. **Regular security updates**
5. **Monitor access logs**

## Production Deployment

### Docker

```bash
# Run Redis with Docker
docker run -d --name redis-cache \
  -p 6379:6379 \
  -v redis-data:/data \
  redis:7-alpine redis-server --requirepass your_password
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis-cache
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis-cache
  template:
    metadata:
      labels:
        app: redis-cache
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        ports:
        - containerPort: 6379
        command: ["redis-server", "--requirepass", "your_password"]
```

## Configuration Examples

### Development (No Auth)
```toml
[cache]
enabled = true
redis_url = "redis://127.0.0.1:6379"
default_ttl = 300
max_size = 104857600
```

### Production (With Auth)
```toml
[cache]
enabled = true
redis_url = "redis://cacheuser:secure_password@redis.example.com:6379"
default_ttl = 600
max_size = 1073741824
```

### Production (With SSL)
```toml
[cache]
enabled = true
redis_url = "rediss://cacheuser:secure_password@redis.example.com:6380"
default_ttl = 600
max_size = 1073741824
```

## Fallback Behavior

If Redis is unavailable, the server will:
1. Log a warning message
2. Continue operating with in-memory cache only
3. Provide helpful setup instructions
4. Maintain full functionality

The in-memory cache provides basic caching functionality even without Redis, ensuring the server remains operational. 