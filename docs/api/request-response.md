# API Reference - Request/Response Format

This document describes the API endpoints, request/response formats, and usage patterns for the Verus RPC Server.

## API Overview

The server provides a JSON-RPC 2.0 compliant API with the following endpoints:

- `POST /` – JSON-RPC 2.0 endpoint
- `GET /health` – Health check (JSON)
- `GET /metrics` – Metrics (JSON)
- `GET /metrics/prometheus` – Prometheus exposition format (text/plain)
- `POST /payments/request` – Request a payment quote and shielded address
- `POST /payments/submit` – Submit raw transaction hex for your payment
- `GET /payments/status/{payment_id}` – Check payment status and obtain tokens

## Request Format

### JSON-RPC 2.0 Standard

All RPC requests must follow the JSON-RPC 2.0 specification:

```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": ["param1", "param2", ...],
  "id": 1
}
```

### Required Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `jsonrpc` | string | ✅ | Must be "2.0" |
| `method` | string | ✅ | RPC method name |
| `params` | array | ❌ | Method parameters |
| `id` | number/string | ✅ | Request identifier |

### Example Request

```bash
curl -X POST http://127.0.0.1:8080/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }'
```

## Response Format

### Success Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    // Method-specific result data
  },
  "id": 1
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "details": "Additional error information"
    }
  },
  "id": 1
}
```

### Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32600 | Invalid Request | Request is not valid JSON-RPC 2.0 |
| -32601 | Method not found | Method does not exist |
| -32602 | Invalid params | Invalid method parameters |
| -32700 | Parse error | Invalid JSON |
| -32000 | Server error | Internal server error |
| -32001 | Method not allowed | Method not in allowlist |
| -32002 | Authentication required | JWT token required |
| -32003 | Rate limited | Too many requests |
| -32004 | Validation error | Parameter validation failed |

## Authentication

### Development Mode

In development mode (`development_mode = true`), authentication is not required.

### Production Mode

In production mode, include a JWT token in the Authorization header:

```bash
curl -X POST http://127.0.0.1:8080/ \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }'
```

## Health Check Endpoint

### GET /health

Request:
```bash
curl http://127.0.0.1:8080/health
```

Response:
```json
{
  "status": "healthy",
  "timestamp": "2024-12-06T15:30:00Z",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

## Metrics Endpoints

### GET /metrics (JSON)

Request:
```bash
curl http://127.0.0.1:8080/metrics
```

Response (example):
```json
{
  "total_requests": 100,
  "successful_requests": 98,
  "failed_requests": 2,
  "rate_limited_requests": 0,
  "avg_response_time_ms": 12.3,
  "active_connections": 5,
  "uptime_seconds": 3600
}
```

### GET /metrics/prometheus (text/plain)

Request:
```bash
curl http://127.0.0.1:8080/metrics/prometheus
```

Response (example):
```
# HELP verus_rpc_requests_total Total number of RPC requests
# TYPE verus_rpc_requests_total counter
verus_rpc_requests_total{method="getinfo"} 42
```

## Rate Limiting

The API implements rate limiting based on IP address. When rate limited, you'll receive:

Response (429 Too Many Requests):
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

Headers:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1640995200
Retry-After: 60
```

## Security Headers

All responses include security headers:

```
Content-Security-Policy: default-src 'self'; script-src 'self'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
Expires: 0
```

## Common RPC Methods

### getinfo

Request:
```json
{
  "jsonrpc": "2.0",
  "method": "getinfo",
  "params": [],
  "id": 1
}
```

Response:
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

### getblock

Request:
```json
{
  "jsonrpc": "2.0",
  "method": "getblock",
  "params": ["0000000000000000000000000000000000000000000000000000000000000000", true],
  "id": 1
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "confirmations": 123456,
    "size": 1234,
    "height": 123456,
    "version": 1,
    "merkleroot": "0000000000000000000000000000000000000000000000000000000000000000",
    "tx": ["txid1", "txid2"],
    "time": 1234567890,
    "mediantime": 1234567890,
    "nonce": 1234567890,
    "bits": "1d00ffff",
    "difficulty": 123456.789,
    "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
    "previousblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
    "nextblockhash": "0000000000000000000000000000000000000000000000000000000000000000"
  },
  "id": 1
}
```

## Error Handling

### Validation Errors

When parameters fail validation:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32004,
    "message": "Validation error",
    "data": {
      "field": "params[0]",
      "reason": "Invalid block hash format",
      "expected": "64 character hex string"
    }
  },
  "id": 1
}
```

### Method Not Allowed

When requesting a method not in the allowlist:

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Method not allowed",
    "data": {
      "method": "invalid_method",
      "allowed_methods": ["getinfo", "getblock", "getblockhash"]
    }
  },
  "id": 1
}
```

## Complete Method List

For a complete list of supported RPC methods, see [RPC Methods](./rpc-methods.md).

## Related Documentation

- [Authentication](./authentication.md) - Detailed authentication guide
- [Error Handling](./error-handling.md) - Comprehensive error handling
- [Rate Limiting](./rate-limiting.md) - Rate limiting configuration
- [RPC Methods](./rpc-methods.md) - Complete method reference
