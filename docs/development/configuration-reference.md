# Configuration Reference

Reference for all configuration options available in the Verus RPC Server.

## Configuration File Structure

The configuration file uses TOML format and is typically named `Conf.toml`. Multiple configuration files can be used for different environments:

- `Conf.toml` - Default configuration
- `Conf.production.toml` - Production environment
- `Conf.public-dex.toml` - Public DEX deployment
- `Conf.pow-example.toml` - PoW-enabled example

## Configuration Sections

### [verus] - Verus Daemon Configuration

```toml
[verus]
# The URL and port of your Verus daemon RPC endpoint
rpc_url = "http://127.0.0.1:27486"
# RPC username (from your verus.conf file)
rpc_user = "your_rpc_username"
# RPC password (from your verus.conf file)
rpc_password = "your_rpc_password"
# Connection timeout in seconds
timeout_seconds = 30
# Maximum retry attempts
max_retries = 3
```

**Options:**
- `rpc_url`: URL of the Verus daemon RPC endpoint
- `rpc_user`: RPC username from verus.conf
- `rpc_password`: RPC password from verus.conf
- `timeout_seconds`: Connection timeout (1-300 seconds)
- `max_retries`: Maximum retry attempts (0-10)

### [server] - Server Configuration

```toml
[server]
# Server address to bind to
bind_address = "127.0.0.1"
# Server port
port = 8080
# Maximum request size in bytes
max_request_size = 1048576
# Worker threads (0 for auto-detect)
worker_threads = 0
# Enable SSL/TLS
ssl_enabled = false
# Enable response compression
compression_enabled = true
# Minimum response size for compression (bytes)
compression_min_size = 1024
```

**Options:**
- `bind_address`: Server bind address (use "0.0.0.0" for all interfaces)
- `port`: Server port (1-65535)
- `max_request_size`: Maximum request size (1KB-10MB)
- `worker_threads`: Worker threads (0-64, 0 for auto-detect)
- `ssl_enabled`: Enable SSL/TLS (should be handled by reverse proxy)
- `compression_enabled`: Enable response compression
- `compression_min_size`: Minimum size for compression

### [security] - Security Configuration

```toml
[security]
# Allowed CORS origins
cors_origins = ["*"]
# Allowed CORS methods
cors_methods = ["GET", "POST", "OPTIONS"]
# Allowed CORS headers
cors_headers = ["Content-Type", "Authorization", "Accept"]
# Enable request logging
enable_request_logging = true
# Enable security headers
enable_security_headers = true
# Trusted proxy headers
trusted_proxy_headers = ["X-Forwarded-For"]
# Enable custom security headers
enable_custom_headers = false
# Custom security header value
custom_security_header = ""
# Method-specific rate limits
method_rate_limits = {}
# Development mode - allows local access without authentication
development_mode = true
```

**Options:**
- `cors_origins`: Allowed CORS origins (use ["*"] for public access)
- `cors_methods`: Allowed HTTP methods
- `cors_headers`: Allowed CORS headers
- `enable_request_logging`: Enable request logging
- `enable_security_headers`: Enable security headers
- `trusted_proxy_headers`: Trusted proxy headers for IP detection
- `enable_custom_headers`: Enable custom security headers
- `custom_security_header`: Custom security header value
- `method_rate_limits`: Method-specific rate limits
- `development_mode`: Development mode (disable in production)

### [security.jwt] - JWT Configuration

```toml
[security.jwt]
# JWT secret key (generate a secure random key for production)
secret_key = "your-super-secret-jwt-key-that-is-at-least-32-characters-long"
# JWT token expiration time in seconds
expiration_seconds = 3600
# JWT issuer
issuer = "verus-rpc-server"
# JWT audience
audience = "verus-clients"
```

**Options:**
- `secret_key`: JWT secret key (minimum 32 characters)
- `expiration_seconds`: Token expiration time (60-86400 seconds)
- `issuer`: JWT issuer claim
- `audience`: JWT audience claim

### [security.pow] - Proof of Work Configuration

```toml
[security.pow]
# Default difficulty for PoW challenges (hex string)
default_difficulty = "0000ffff"
# Challenge expiration time in minutes
challenge_expiration_minutes = 10
# Token duration for PoW-validated tokens (seconds)
token_duration_seconds = 14400
# Rate limit multiplier for PoW-validated tokens
rate_limit_multiplier = 2.0
# Enable PoW challenges
enabled = true
```

**Options:**
- `default_difficulty`: Default difficulty for PoW challenges
  - Easy: `"ffffffff"` (for testing)
  - Medium: `"0000ffff"` (default production)
  - Hard: `"000000ff"` (premium access)
- `challenge_expiration_minutes`: Challenge expiration time (1-60 minutes)
- `token_duration_seconds`: Token duration for PoW-validated tokens (3600-86400 seconds)
- `rate_limit_multiplier`: Rate limit multiplier (1.0-10.0)
- `enabled`: Enable PoW challenges

### [security.token_issuance] - Token Issuance Rate Limiting

```toml
[security.token_issuance]
# Limit token requests per IP per hour
requests_per_hour = 10
# Enable token issuance rate limiting
enabled = true
```

**Options:**
- `requests_per_hour`: Token requests per IP per hour
- `enabled`: Enable token issuance rate limiting

### [rate_limit] - Rate Limiting Configuration

```toml
[rate_limit]
# Requests per minute per IP
requests_per_minute = 1000
# Burst size
burst_size = 100
# Enable rate limiting
enabled = true

# Method-specific rate limits
[rate_limit.methods]
getinfo = 1000
z_getbalance = 500
z_getnewaddress = 100
z_sendmany = 50
getblock = 200
```

**Options:**
- `requests_per_minute`: Requests per minute per IP (1-10000)
- `burst_size`: Burst size (1-1000)
- `enabled`: Enable rate limiting
- `methods`: Method-specific rate limits

### [logging] - Logging Configuration

```toml
[logging]
# Log level (trace, debug, info, warn, error)
level = "info"
# Log format (json, text)
format = "json"
# Enable structured logging
structured = true
```

**Options:**
- `level`: Log level (trace, debug, info, warn, error)
- `format`: Log format (json, text)
- `structured`: Enable structured logging

### [cache] - Caching Configuration

```toml
[cache]
# Enable response caching
enabled = false
# Redis connection URL
redis_url = "redis://127.0.0.1:6379"
# Default TTL in seconds
default_ttl = 300
# Maximum cache size in bytes
max_size = 104857600
```

**Options:**
- `enabled`: Enable response caching
- `redis_url`: Redis connection URL
  - No auth: `redis://127.0.0.1:6379`
  - With password: `redis://:password@127.0.0.1:6379`
  - With username/password: `redis://username:password@127.0.0.1:6379`
  - With SSL: `rediss://127.0.0.1:6379`
- `default_ttl`: Default cache TTL (1-86400 seconds)
- `max_size`: Maximum cache size (1KB-1GB)

### [token_service] - Token Service Configuration

```toml
[token_service]
# Token service port
port = 8081
# Token service bind address
bind_address = "127.0.0.1"
# Enable CORS for token service
enable_cors = true
# Allowed origins for token service
allowed_origins = ["*"]
# Rate limit for token service requests per minute
rate_limit_requests_per_minute = 100
# Enable request logging for token service
enable_request_logging = true
```

**Options:**
- `port`: Token service port (1-65535)
- `bind_address`: Token service bind address
- `enable_cors`: Enable CORS for token service
- `allowed_origins`: Allowed origins for token service
- `rate_limit_requests_per_minute`: Rate limit for token service
- `enable_request_logging`: Enable request logging for token service

### [payments] - Payments Configuration

```toml
[payments]
enabled = true
address_types = ["orchard", "sapling"]
default_address_type = "orchard"
min_confirmations = 1
session_ttl_minutes = 30
require_viewing_key = false
viewing_keys = []
viewing_key_rescan = "whenkeyisnew"  # "yes", "no", or "whenkeyisnew"

[[payments.tiers]]
id = "basic"
amount_vrsc = 1.0
permissions = ["read"]

[[payments.tiers]]
id = "pro"
amount_vrsc = 5.0
permissions = ["read", "write"]
```

**Options:**
- `enabled`: Enable the payments REST API
- `address_types`: Allowed shielded address types ("orchard", "sapling")
- `default_address_type`: Default shielded address type
- `min_confirmations`: Confirmations required before issuing a provisional token
- `session_ttl_minutes`: Minutes before a quote/session expires
- `require_viewing_key`: If true, server must have viewing keys and will not create new addresses
- `viewing_keys`: List of viewing keys to import on startup
- `viewing_key_rescan`: Rescan policy for viewing key import ("yes", "no", "whenkeyisnew")
- `tiers`: Payment tiers (id, amount_vrsc, optional description, permissions)

Notes:
- With `require_viewing_key=true` and empty `viewing_keys`, the server will warn and reject quotes
- Tokens are provisional at `min_confirmations` and finalized at deeper confirmations (≥2)
 - When `[cache].enabled = true`, `PaymentsStore` and `RevocationStore` use Redis; otherwise they use in-memory fallbacks
 - Set `payments.enabled=false` to disable payments endpoints and service behavior

## Environment-Specific Configurations

### Development Configuration

```toml
[security]
development_mode = true
cors_origins = ["*"]

[security.pow]
enabled = false  # Disable PoW for development
```

### Production Configuration

```toml
[security]
development_mode = false
cors_origins = ["https://yourdomain.com"]

[security.pow]
enabled = true
default_difficulty = "0000ffff"
```

### Public DEX Configuration

```toml
[security]
cors_origins = ["*"]
development_mode = false

[security.pow]
enabled = true
default_difficulty = "0000ffff"

[security.token_issuance]
requests_per_hour = 10
enabled = true
```

## Configuration Validation

The configuration is automatically validated on startup. Common validation errors:

- **Invalid URLs**: Ensure RPC and Redis URLs are properly formatted
- **Invalid ranges**: Check that numeric values are within allowed ranges
- **Missing required fields**: Ensure all required fields are present
- **Invalid difficulty**: PoW difficulty must be a valid hex string

## Security Considerations

### Production Security

1. **JWT Secret**: Use a cryptographically secure random key
2. **Development Mode**: Always disable in production
3. **CORS Origins**: Restrict to specific domains
4. **Rate Limiting**: Enable and configure appropriately
5. **SSL/TLS**: Use reverse proxy for SSL termination

### PoW Security

1. **Difficulty Levels**: Adjust based on your economic model
2. **Expiration Times**: Balance security with usability
3. **Rate Limiting**: Prevent abuse of challenge generation
4. **Monitoring**: Monitor for unusual patterns

## Configuration Examples

### Complete Example

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_rpc_username"
rpc_password = "your_rpc_password"
timeout_seconds = 30
max_retries = 3

[server]
bind_address = "0.0.0.0"
port = 8080
max_request_size = 1048576
worker_threads = 0
ssl_enabled = false
compression_enabled = true
compression_min_size = 1024

[security]
cors_origins = ["*"]
cors_methods = ["GET", "POST", "OPTIONS"]
cors_headers = ["Content-Type", "Authorization", "Accept"]
enable_request_logging = true
enable_security_headers = true
trusted_proxy_headers = ["X-Forwarded-For", "X-Real-IP"]
enable_custom_headers = true
custom_security_header = "X-Verus-RPC-Server"
method_rate_limits = {}
development_mode = false

[security.jwt]
secret_key = "your-super-secret-jwt-key-that-is-at-least-32-characters-long"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"

[security.pow]
default_difficulty = "0000ffff"
challenge_expiration_minutes = 10
token_duration_seconds = 14400
rate_limit_multiplier = 2.0
enabled = true

[security.token_issuance]
requests_per_hour = 10
enabled = true

[rate_limit]
requests_per_minute = 1000
burst_size = 100
enabled = true

[rate_limit.methods]
getinfo = 1000
z_getbalance = 500
z_getnewaddress = 100
z_sendmany = 50
getblock = 200

[logging]
level = "info"
format = "json"
structured = true

[cache]
enabled = true
redis_url = "redis://127.0.0.1:6379"
default_ttl = 300
max_size = 104857600

[token_service]
port = 8081
bind_address = "127.0.0.1"
enable_cors = true
allowed_origins = ["*"]
rate_limit_requests_per_minute = 100
enable_request_logging = true
```

This configuration provides a complete reference for all available options and their recommended values for different deployment scenarios.
