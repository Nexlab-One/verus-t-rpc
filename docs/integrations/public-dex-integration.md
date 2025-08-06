# Public DEX Integration Guide

## Overview

This guide explains how to integrate with a free public Verus RPC service for DEX applications, including JWT token issuance and usage.

## JWT Token Issuance Models

### 1. Self-Service Token Issuance (Recommended)

Users can request tokens directly through a public API endpoint.

#### API Endpoints

**Token Issuance:**
```http
POST /token/issue
Content-Type: application/json

{
  "user_id": "user_12345",
  "permissions": ["read", "write"],
  "client_ip": "192.168.1.100",
  "user_agent": "MyDEX/1.0",
  "custom_expiration": 3600
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "token_id": "abc123-def456"
}
```

**Token Validation:**
```http
POST /token/validate
Content-Type: application/json

{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "client_ip": "192.168.1.100"
}
```

#### Rate Limiting

- **Token Issuance**: 10 requests per hour per IP
- **Token Validation**: 100 requests per minute per IP
- **RPC Calls**: 1000 requests per minute per token

#### Security Measures

1. **IP-based rate limiting** to prevent abuse
2. **User agent validation** for DEX applications
3. **Token expiration** (default 1 hour, configurable)
4. **Permission-based access control**
5. **Audit logging** for security monitoring

### 2. Pre-approved Token Distribution

For trusted DEX partners, tokens can be pre-generated and distributed.

#### Partner Registration Process

1. **DEX Registration:**
```http
POST /partner/register
Content-Type: application/json

{
  "dex_name": "MyDEX",
  "contact_email": "admin@mydex.com",
  "expected_volume": "high",
  "security_requirements": ["audit_logging", "rate_limiting"]
}
```

2. **Token Generation:**
```http
POST /partner/tokens
Authorization: Bearer <admin_token>
Content-Type: application/json

{
  "partner_id": "mydex_001",
  "token_count": 1000,
  "permissions": ["read", "write"],
  "expiration_days": 30
}
```

### 3. OAuth2 Integration

For enterprise DEX applications, OAuth2 flow can be implemented.

#### OAuth2 Flow

1. **Authorization Request:**
```
GET /oauth/authorize?
  response_type=code&
  client_id=mydex_client&
  redirect_uri=https://mydex.com/callback&
  scope=verus_rpc&
  state=random_state
```

2. **Token Exchange:**
```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=received_code&
client_id=mydex_client&
client_secret=mydex_secret&
redirect_uri=https://mydex.com/callback
```

## DEX Integration Examples

### JavaScript/TypeScript Integration

```typescript
class VerusRPCClient {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async requestToken(userId: string, permissions: string[]): Promise<string> {
    const response = await fetch(`${this.baseUrl}/token/issue`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        user_id: userId,
        permissions: permissions,
        client_ip: await this.getClientIP(),
        user_agent: navigator.userAgent,
      }),
    });

    if (!response.ok) {
      throw new Error(`Token request failed: ${response.statusText}`);
    }

    const data = await response.json();
    this.token = data.token;
    return data.token;
  }

  async makeRPCRequest(method: string, params: any[]): Promise<any> {
    if (!this.token) {
      throw new Error('No valid token available');
    }

    const response = await fetch(this.baseUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.token}`,
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: method,
        params: params,
        id: Date.now(),
      }),
    });

    if (!response.ok) {
      throw new Error(`RPC request failed: ${response.statusText}`);
    }

    return await response.json();
  }

  private async getClientIP(): Promise<string> {
    // Use a service like ipify.org or similar
    const response = await fetch('https://api.ipify.org?format=json');
    const data = await response.json();
    return data.ip;
  }
}

// Usage Example
const verusClient = new VerusRPCClient('https://your-verus-rpc.com');

// Request token for DEX user
const token = await verusClient.requestToken('dex_user_123', ['read', 'write']);

// Make RPC calls
const balance = await verusClient.makeRPCRequest('z_getbalance', ['address']);
const newAddress = await verusClient.makeRPCRequest('z_getnewaddress', []);
```

### Python Integration

```python
import requests
import json
from typing import List, Dict, Any

class VerusRPCClient:
    def __init__(self, base_url: str):
        self.base_url = base_url
        self.token = None
        self.session = requests.Session()

    def request_token(self, user_id: str, permissions: List[str]) -> str:
        """Request a JWT token for RPC access"""
        payload = {
            "user_id": user_id,
            "permissions": permissions,
            "client_ip": self._get_client_ip(),
            "user_agent": "PythonDEX/1.0"
        }

        response = self.session.post(
            f"{self.base_url}/token/issue",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        response.raise_for_status()

        data = response.json()
        self.token = data["token"]
        return data["token"]

    def make_rpc_request(self, method: str, params: List[Any]) -> Dict[str, Any]:
        """Make an RPC request with JWT authentication"""
        if not self.token:
            raise ValueError("No valid token available")

        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        }

        headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {self.token}"
        }

        response = self.session.post(
            self.base_url,
            json=payload,
            headers=headers
        )
        response.raise_for_status()

        return response.json()

    def _get_client_ip(self) -> str:
        """Get client IP address"""
        try:
            response = self.session.get("https://api.ipify.org?format=json")
            return response.json()["ip"]
        except:
            return "127.0.0.1"

# Usage Example
client = VerusRPCClient("https://your-verus-rpc.com")

# Request token
token = client.request_token("dex_user_456", ["read", "write"])

# Make RPC calls
balance = client.make_rpc_request("z_getbalance", ["address"])
new_address = client.make_rpc_request("z_getnewaddress", [])
```

## Security Best Practices

### For DEX Developers

1. **Token Management:**
   - Store tokens securely (encrypted at rest)
   - Implement token refresh logic
   - Handle token expiration gracefully

2. **Rate Limiting:**
   - Implement client-side rate limiting
   - Cache responses when appropriate
   - Use connection pooling

3. **Error Handling:**
   - Handle authentication failures
   - Implement retry logic with exponential backoff
   - Log errors for debugging

4. **Security:**
   - Validate all RPC responses
   - Sanitize user inputs
   - Use HTTPS for all communications

### For RPC Service Providers

1. **Monitoring:**
   - Monitor token issuance patterns
   - Track RPC usage per token
   - Alert on suspicious activity

2. **Rate Limiting:**
   - Implement IP-based rate limiting
   - Token-based rate limiting
   - Method-specific rate limiting

3. **Security:**
   - Regular security audits
   - JWT secret rotation
   - DDoS protection

## Configuration Examples

### Production Configuration

```toml
[security]
development_mode = false
enable_security_headers = true

[security.jwt]
secret_key = "your-super-secret-jwt-key-that-is-at-least-32-characters-long"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-dex-clients"

[rate_limit]
requests_per_minute = 1000
burst_size = 100
enabled = true

[rate_limit.token_issuance]
requests_per_hour = 10
enabled = true
```

### Development Configuration

```toml
[security]
development_mode = true

[security.jwt]
secret_key = "dev-secret-key-for-testing-only"
expiration_seconds = 86400  # 24 hours for development
issuer = "verus-rpc-server-dev"
audience = "verus-dex-clients-dev"

[rate_limit]
requests_per_minute = 10000  # Higher limits for development
burst_size = 1000
enabled = true
```

## Deployment Considerations

### Load Balancing

- Use multiple RPC server instances
- Implement sticky sessions for token validation
- Use Redis for shared rate limiting state

### Monitoring

- Monitor token issuance rates
- Track RPC method usage
- Alert on authentication failures
- Monitor response times

### Scaling

- Horizontal scaling of RPC servers
- Database sharding for high volume
- CDN for static assets
- Caching layer for frequent requests

## Support and Documentation

### API Documentation

- Swagger/OpenAPI documentation
- Code examples in multiple languages
- Integration guides for popular frameworks

### Support Channels

- Technical documentation
- Community forums
- Email support for enterprise clients
- Status page for service updates


