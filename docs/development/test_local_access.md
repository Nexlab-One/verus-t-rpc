# Local Development Mode - No Authentication Required

## Overview

The Verus RPC server now supports a **development mode** that allows local access without authentication or JWT tokens. This is perfect for local development and testing.

## Configuration

### Enable Development Mode

In your `Conf.toml` file, set:

```toml
[security]
# Development mode - allows local access without authentication (WARNING: disable in production!)
development_mode = true
```

### Default Configuration

The default configuration already has development mode enabled for local development:

```toml
[security]
development_mode = true  # ✅ Enabled by default
```

## How It Works

### Security Bypass for Localhost

When `development_mode = true` and the client IP is localhost (`127.0.0.1`, `::1`, or `localhost`):

1. **Authentication checks are skipped**
2. **Permission checks are skipped**
3. **All RPC methods are accessible**
4. **No JWT token required**

### Production Security

When `development_mode = false` (production):
- Full authentication required
- JWT token validation enforced
- Permission-based access control active
- All security measures enabled

## Testing Local Access

### 1. Start the Server

```bash
cargo run
```

### 2. Test RPC Calls (No Authentication Required)

```bash
# Test getinfo method
curl -X POST http://127.0.0.1:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }'

# Test getblock method
curl -X POST http://127.0.0.1:8080 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getblock",
    "params": ["0000000000000000000000000000000000000000000000000000000000000000"],
    "id": 2
  }'
```

### 3. Test Health Check

```bash
curl http://127.0.0.1:8080/health
```

### 4. Test Metrics

```bash
curl http://127.0.0.1:8080/metrics
```

## Security Considerations

### ✅ Safe for Local Development
- Only works for localhost connections
- Authentication still required for external connections
- Easy to disable for production

### ⚠️ Production Deployment
- **ALWAYS** set `development_mode = false` in production
- Use proper JWT authentication
- Configure proper security policies

## Environment Variables

You can also control development mode via environment variables:

```bash
# Enable development mode
export VERUS_RPC__SECURITY__DEVELOPMENT_MODE=true

# Disable development mode (production)
export VERUS_RPC__SECURITY__DEVELOPMENT_MODE=false
```

## Code Example

The development mode is implemented in the security validator:

```rust
// In development mode, skip authentication and permission checks for localhost
if context.development_mode && self.is_localhost(&context.client_ip) {
    // Skip authentication and permission checks for local development
    return Ok(());
}
```

This ensures that local development is seamless while maintaining full security for production deployments. 