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
│    • HTTP Header-Based Authentication                           │
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

### HTTP Header-Based JWT Token Authentication

The server uses **JSON Web Tokens (JWT)** for stateless authentication, with tokens extracted from HTTP Authorization headers:

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

#### HTTP Header Extraction
```bash
# Include JWT token in Authorization header
curl -X POST http://127.0.0.1:8080/ \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "getinfo", "params": [], "id": 1}'
```

#### Authentication Flow
```
1. HTTP Request with Authorization Header
   ↓
2. Header Extraction in Route
   ├─ Authorization: Bearer <token>
   ├─ User-Agent: <client_info>
   └─ X-Forwarded-For: <client_ip>
   ↓
3. RequestContext Creation
   ├─ auth_token: from Authorization header
   ├─ user_agent: from User-Agent header
   └─ client_ip: validated IP address
   ↓
4. Domain Model Conversion
   ├─ ClientInfo with auth_token field
   └─ RpcRequest with complete client context
   ↓
5. RpcService Processing
   ├─ Extract auth_token from client_info
   ├─ Validate with AuthenticationAdapter
   ├─ Create SecurityContext with permissions
   └─ Apply security validation
```

#### Configuration
```toml
[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"
```

### Development Mode

For development purposes, authentication can be bypassed:

```toml
[security]
development_mode = true  # Disables authentication
```

**⚠️ Warning**: Never use development mode in production!

### Permission-Based Access Control

The server implements method-specific permission requirements:

```rust
pub struct RpcMethod {
    pub name: String,
    pub description: String,
    pub read_only: bool,
    pub required_permissions: Vec<String>,
    pub parameter_rules: Vec<ParameterRule>,
}
```

**Example Method Definitions**:
- `getinfo`: Requires `["read"]` permission
- `z_importviewingkey`: Requires `["write"]` permission
- `getblock`: Requires `["read"]` permission with hash validation

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

### Parameter Validation

The server implements comprehensive parameter validation:

#### Validation Rules
```rust
pub struct ParameterRule {
    pub index: usize,
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub constraints: Vec<Constraint>,
}
```

#### Supported Constraints
- **MinLength**: Minimum string length validation
- **MaxLength**: Maximum string length validation
- **Pattern**: Regex pattern matching
- **MinValue**: Minimum numeric value
- **MaxValue**: Maximum numeric value
- **Custom**: Custom validation rules

#### Example Validation
```rust
// Hash parameter validation
ParameterRule {
    index: 0,
    name: "hash".to_string(),
    param_type: ParameterType::String,
    required: true,
    constraints: vec![Constraint::MinLength(64)],
}

// Block type validation
ParameterRule {
    index: 1,
    name: "type".to_string(),
    param_type: ParameterType::String,
    required: false,
    constraints: vec![Constraint::Custom("sprout|sapling|orchard".to_string())],
}
```

### Method Allowlist

Only explicitly allowed RPC methods are processed:

```rust
// Supported methods with security rules
let method_registry = [
    ("getinfo", RpcMethod {
        name: "getinfo".to_string(),
        description: "Get blockchain information".to_string(),
        read_only: true,
        required_permissions: vec!["read".to_string()],
        parameter_rules: vec![],
    }),
    ("getblock", RpcMethod {
        name: "getblock".to_string(),
        description: "Get block information".to_string(),
        read_only: true,
        required_permissions: vec!["read".to_string()],
        parameter_rules: vec![
            ParameterRule {
                index: 0,
                name: "hash".to_string(),
                param_type: ParameterType::String,
                required: true,
                constraints: vec![Constraint::MinLength(64)],
            },
        ],
    }),
    // ... other methods
];
```

## 🔒 Error Handling

### Secure Error Responses

The server implements secure error handling to prevent information disclosure:

#### Error Response Format
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Authentication failed",
    "data": {
      "request_id": "req-123"
    }
  },
  "id": 1
}
```

#### Error Codes
- `-32001`: Authentication error
- `-32002`: Authorization error
- `-32003`: Rate limit exceeded
- `-32004`: Invalid parameters
- `-32005`: Method not allowed
- `-32006`: Internal server error

### Error Logging

Comprehensive error logging for security monitoring:

```rust
// Security event logging
warn!(
    request_id = %context.request_id,
    client_ip = %context.client_ip,
    method = %request.method,
    "Authentication failed: invalid token"
);
```

## 🔍 Security Monitoring

### Authentication Metrics

Track authentication success and failure rates:

```rust
// Authentication metrics
pub struct AuthMetrics {
    pub successful_auths: AtomicU64,
    pub failed_auths: AtomicU64,
    pub token_validations: AtomicU64,
    pub permission_checks: AtomicU64,
}
```

### Security Event Logging

Comprehensive security event logging:

```rust
// Security event types
pub enum SecurityEvent {
    AuthenticationSuccess { user_id: String, client_ip: String },
    AuthenticationFailure { client_ip: String, reason: String },
    AuthorizationFailure { user_id: String, method: String },
    RateLimitExceeded { client_ip: String },
    InvalidParameters { method: String, client_ip: String },
}
```

## 🛡️ Security Best Practices

### Token Management

1. **Secure Token Storage**: Tokens are never logged or stored in plain text
2. **Token Expiration**: Configurable token expiration with automatic renewal
3. **Token Validation**: Comprehensive token validation including signature, expiration, and audience
4. **Token Rotation**: Support for token rotation and revocation

### Input Sanitization

1. **Parameter Validation**: All input parameters are validated against defined rules
2. **Type Safety**: Strong typing with Rust's type system prevents type-related vulnerabilities
3. **Length Limits**: Configurable length limits prevent buffer overflow attacks
4. **Pattern Validation**: Regex pattern validation for format-specific parameters

### Access Control

1. **Method Allowlist**: Only explicitly allowed RPC methods are processed
2. **Permission-Based Access**: Method-specific permission requirements
3. **IP Restrictions**: Configurable IP address restrictions
4. **Development Mode**: Secure development mode for testing

### Error Handling

1. **Secure Error Messages**: No sensitive information in error responses
2. **Graceful Degradation**: Proper handling of authentication and authorization failures
3. **Comprehensive Logging**: Security events are logged for monitoring
4. **Rate Limiting**: Protection against abuse and DoS attacks

## 🔧 Security Configuration

### Complete Security Configuration

```toml
[security]
development_mode = false
enable_security_headers = true
enable_custom_headers = true
allowed_methods = ["getinfo", "getblock", "z_importviewingkey"]

[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"

[rate_limit]
enabled = true
requests_per_minute = 100
burst_size = 20

[cors]
enabled = true
allowed_origins = ["https://your-domain.com"]
allowed_methods = ["POST"]
allowed_headers = ["Authorization", "Content-Type"]
```

### Environment-Specific Configuration

#### Development Environment
```toml
[security]
development_mode = true  # Disables authentication for development

[rate_limit]
requests_per_minute = 1000  # Higher limits for development
```

#### Production Environment
```toml
[security]
development_mode = false
enable_security_headers = true

[rate_limit]
requests_per_minute = 100  # Stricter limits for production
```

## 🔗 Related Documentation

- [System Architecture](../architecture/system-architecture.md) - Overall system architecture
- [Application Services](../architecture/application-services.md) - Application services security
- [API Documentation](../api/) - API security considerations
- [Development Guide](../development/) - Security development guidelines
