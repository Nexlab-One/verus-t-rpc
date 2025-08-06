# Security Overview

Overview of the security features, architecture, and best practices implemented in the Verus RPC Server.

## 🛡️ Security Architecture

The Verus RPC Server implements a **multi-layered security approach** following the **Defense in Depth** principle to protect against various attack vectors.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Security Layers                              │
├─────────────────────────────────────────────────────────────────┤
│ 1. Network Security (Reverse Proxy)                             │
│    • SSL/TLS Termination                                        │
│    • DDoS Protection                                            │
│    • IP Whitelisting                                            │
├─────────────────────────────────────────────────────────────────┤
│ 2. Application Security (RPC Server)                            │
│    • Authentication & Authorization                             │
│    • Rate Limiting                                              │
│    • Input Validation                                           │
│    • Security Headers                                           │
├─────────────────────────────────────────────────────────────────┤
│ 3. Infrastructure Security                                      │
│    • Method Allowlist                                           │
│    • Parameter Validation                                       │
│    • Error Handling                                             │
└─────────────────────────────────────────────────────────────────┘
```

## 🔐 Authentication & Authorization

### JWT Token Authentication

The server uses **JSON Web Tokens (JWT)** for stateless authentication:

#### Token Structure
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "user_id",
    "iss": "verus-rpc-server",
    "aud": "verus-clients",
    "iat": 1640995200,
    "exp": 1640998800,
    "permissions": ["read", "write"]
  }
}
```

#### Configuration
```toml
[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"
```

#### Usage
```bash
# Include JWT token in requests
curl -X POST http://127.0.0.1:8080/ \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getinfo", "params": [], "id": 1}'
```

### Development Mode

For development purposes, authentication can be bypassed:

```toml
[security]
development_mode = true  # Disables authentication
```

**⚠️ Warning**: Never use development mode in production!

## 🚦 Rate Limiting

### IP-Based Rate Limiting

The server implements **IP-based rate limiting** to prevent abuse:

#### Configuration
```toml
[rate_limit]
enabled = true
requests_per_minute = 100
burst_size = 20
```

#### Rate Limit Headers
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
Retry-After: 60
```

#### Rate Limit Response
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32003,
    "message": "Rate limited",
    "data": {
      "retry_after": 60,
      "limit": 100,
      "window": "1m"
    }
  },
  "id": 1
}
```

### Rate Limiting Strategy

1. **Token Bucket Algorithm**: Allows burst traffic up to configured limit
2. **Per-IP Tracking**: Separate limits for each client IP
3. **Graceful Degradation**: Returns proper error responses instead of dropping requests
4. **Configurable Limits**: Different limits for different environments

## 🛡️ Security Headers

### HTTP Security Headers

The server applies comprehensive security headers to all responses:

#### Content Security Policy (CSP)
```
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';
```

**Purpose**: Prevents XSS attacks and controls resource loading

#### X-Content-Type-Options
```
X-Content-Type-Options: nosniff
```

**Purpose**: Prevents MIME type sniffing attacks

#### X-Frame-Options
```
X-Frame-Options: DENY
```

**Purpose**: Prevents clickjacking attacks

#### X-XSS-Protection
```
X-XSS-Protection: 1; mode=block
```

**Purpose**: Additional XSS protection for older browsers

#### Referrer Policy
```
Referrer-Policy: strict-origin-when-cross-origin
```

**Purpose**: Controls referrer information for privacy

#### Permissions Policy
```
Permissions-Policy: geolocation=(), microphone=(), camera=(), payment=()
```

**Purpose**: Controls browser feature access

#### Cache Control
```
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
Expires: 0
```

**Purpose**: Prevents sensitive data caching

### Custom Security Headers

Additional custom headers can be configured:

```toml
[security]
enable_custom_headers = true
custom_headers = [
  "X-Custom-Header: Value",
  "Server: Verus-RPC-Server"
]
```

## 🔍 Input Validation

### Method Allowlist

Only pre-approved RPC methods are allowed:

```rust
// src/allowlist.rs
pub const ALLOWED_METHODS: &[&str] = &[
    "getinfo",
    "getblockchaininfo",
    "getblock",
    "getblockhash",
    // ... 60+ methods
];
```

### Parameter Validation

Comprehensive parameter validation for each method:

#### Type Validation
```rust
// Validate parameter types
match method {
    "getblock" => {
        validate_string_param(params.get(0), "block_hash")?;
        validate_bool_param(params.get(1), "verbose")?;
    }
    "getrawtransaction" => {
        validate_string_param(params.get(0), "txid")?;
        validate_bool_param(params.get(1), "verbose")?;
    }
    // ... other methods
}
```

#### Format Validation
```rust
// Validate block hash format
fn validate_block_hash(hash: &str) -> Result<(), ValidationError> {
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) || hash.len() != 64 {
        return Err(ValidationError::InvalidFormat(
            "Block hash must be 64 character hex string".to_string()
        ));
    }
    Ok(())
}
```

#### Size Limits
```rust
// Validate parameter sizes
const MAX_PARAM_LENGTH: usize = 1024 * 1024; // 1MB

if param.len() > MAX_PARAM_LENGTH {
    return Err(ValidationError::TooLarge(
        format!("Parameter exceeds maximum size of {} bytes", MAX_PARAM_LENGTH)
    ));
}
```

### Injection Prevention

#### SQL Injection Prevention
- No direct SQL queries in the application
- All data is passed through parameterized interfaces

#### Command Injection Prevention
- No shell command execution
- All external calls use safe APIs

#### XSS Prevention
- Input sanitization
- Output encoding
- CSP headers

## 🔒 CORS Configuration

### Cross-Origin Resource Sharing

Configurable CORS settings for web applications:

```toml
[security]
cors_origins = ["https://yourdomain.com", "https://app.yourdomain.com"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]
```

### CORS Headers
```
Access-Control-Allow-Origin: https://yourdomain.com
Access-Control-Allow-Methods: GET, POST
Access-Control-Allow-Headers: Content-Type, Authorization
Access-Control-Max-Age: 86400
```

## 🚨 Error Handling

### Secure Error Responses

Error responses don't leak sensitive information:

#### Good Error Response
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32004,
    "message": "Validation error",
    "data": {
      "field": "params[0]",
      "reason": "Invalid format"
    }
  },
  "id": 1
}
```

#### Bad Error Response (Avoided)
```json
{
  "error": "Database connection failed: user=admin, password=secret123"
}
```

### Error Logging

Security events are logged for monitoring:

```rust
// Log authentication failures
tracing::warn!(
    "Authentication failed for IP: {}, reason: {}",
    client_ip,
    reason
);

// Log rate limit violations
tracing::info!(
    "Rate limit exceeded for IP: {}, limit: {}, window: {}",
    client_ip,
    limit,
    window
);
```

## 📊 Security Monitoring

### Security Metrics

Prometheus metrics for security monitoring:

```
# Authentication metrics
verus_rpc_auth_failures_total{reason="invalid_token"} 5
verus_rpc_auth_failures_total{reason="expired_token"} 2

# Rate limiting metrics
verus_rpc_rate_limit_hits_total{ip="192.168.1.100"} 10

# Validation metrics
verus_rpc_validation_errors_total{field="block_hash"} 3
verus_rpc_validation_errors_total{field="txid"} 1
```

### Security Logging

Structured logging for security events:

```json
{
  "timestamp": "2024-12-06T15:30:00Z",
  "level": "warn",
  "event": "authentication_failure",
  "ip": "192.168.1.100",
  "reason": "invalid_token",
  "user_agent": "curl/7.68.0"
}
```

## 🔧 Security Configuration

### Environment-Specific Settings

#### Development
```toml
[security]
development_mode = true
enable_security_headers = true
cors_origins = ["*"]
```

#### Production
```toml
[security]
development_mode = false
enable_security_headers = true
cors_origins = ["https://yourdomain.com"]
```

### Security Headers Configuration

```toml
[security]
enable_security_headers = true
enable_custom_headers = false
custom_headers = [
  "X-Custom-Security: enabled"
]
```

## 🚀 Security Best Practices

### 1. **Use Strong JWT Secrets**
```toml
# Use cryptographically secure random strings
jwt_secret = "your-32-character-cryptographically-secure-secret"
```

### 2. **Enable All Security Features**
```toml
[security]
development_mode = false
enable_security_headers = true
enable_custom_headers = true
```

### 3. **Configure Rate Limiting**
```toml
[rate_limit]
enabled = true
requests_per_minute = 100  # Adjust based on your needs
burst_size = 20
```

### 4. **Use HTTPS in Production**
- Deploy behind a reverse proxy with SSL/TLS termination
- Never expose the server directly to the internet

### 5. **Monitor Security Events**
- Set up alerts for authentication failures
- Monitor rate limit violations
- Track validation errors

### 6. **Regular Security Updates**
- Keep dependencies updated
- Monitor security advisories
- Regular security audits

## 🔗 Related Documentation

- [Authentication](./authentication.md) - Detailed authentication guide
- [Security Headers](./security-headers.md) - Security headers implementation
- [Input Validation](./input-validation.md) - Input validation details
- [Rate Limiting](./rate-limiting.md) - Rate limiting configuration
- [Security Best Practices](./best-practices.md) - Production security guidelines
