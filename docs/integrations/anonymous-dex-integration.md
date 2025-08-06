# Anonymous DEX Integration Guide

## Overview

This guide explains how to integrate with a free public Verus RPC service for anonymous DEX users. No registration or user accounts are required - users can get tokens instantly and start using the RPC service.

## Anonymous Token Issuance

### Simple Token Request

For anonymous users, simply request a token without providing a user ID:

```bash
# Request anonymous token
curl -X POST https://your-verus-rpc.com/token/issue \
  -H "Content-Type: application/json" \
  -d '{
    "permissions": ["read", "write"],
    "client_ip": "192.168.1.100",
    "user_agent": "MyDEX/1.0"
  }'
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "token_id": "anon_abc123-def456",
  "user_id": "anon_user_12345"
}
```

### JavaScript Example

```javascript
class AnonymousVerusClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
        this.token = null;
        this.userId = null;
    }

    async getToken(permissions = ['read', 'write']) {
        const response = await fetch(`${this.baseUrl}/token/issue`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                permissions: permissions,
                client_ip: await this.getClientIP(),
                user_agent: navigator.userAgent
            })
        });

        const data = await response.json();
        this.token = data.token;
        this.userId = data.user_id;
        
        console.log(`Anonymous user: ${data.user_id}`);
        return data.token;
    }

    async makeRPCRequest(method, params = []) {
        if (!this.token) {
            throw new Error('No token available. Call getToken() first.');
        }

        const response = await fetch(this.baseUrl, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${this.token}`
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: method,
                params: params,
                id: Date.now()
            })
        });

        return await response.json();
    }

    async getClientIP() {
        try {
            const response = await fetch('https://api.ipify.org?format=json');
            return (await response.json()).ip;
        } catch {
            return '127.0.0.1';
        }
    }
}

// Usage
const client = new AnonymousVerusClient('https://your-verus-rpc.com');

// Get anonymous token
await client.getToken(['read', 'write']);

// Make RPC calls
const balance = await client.makeRPCRequest('z_getbalance', ['address']);
const newAddress = await client.makeRPCRequest('z_getnewaddress', []);
```

## Rate Limiting for Anonymous Users

### Token Issuance Limits
- **10 tokens per hour** per IP address
- **1 token per minute** per IP address
- **Maximum 24-hour tokens** per IP address

### RPC Usage Limits
- **1000 requests per minute** per token
- **100 requests per second** per token
- **Method-specific limits** apply

## Security Features

### Anonymous User Security
1. **IP-based rate limiting** prevents abuse
2. **User agent validation** for DEX applications
3. **Token expiration** (1 hour default)
4. **Unique anonymous user IDs** for tracking
5. **Audit logging** for security monitoring

### Best Practices
1. **Store tokens securely** in memory only
2. **Handle token expiration** gracefully
3. **Implement retry logic** for failed requests
4. **Cache responses** when appropriate
5. **Monitor usage** to stay within limits

## Error Handling

### Common Error Responses

```json
// Rate limit exceeded
{
  "error": "rate_limit_exceeded",
  "message": "Too many token requests. Try again later.",
  "retry_after": 3600
}

// Invalid permissions
{
  "error": "invalid_permissions",
  "message": "Invalid permissions requested"
}

// Token expired
{
  "error": "token_expired",
  "message": "JWT token has expired"
}
```

### Error Handling Example

```javascript
async function handleRPCRequest(client, method, params) {
    try {
        return await client.makeRPCRequest(method, params);
    } catch (error) {
        if (error.message.includes('token_expired')) {
            // Get new token and retry
            await client.getToken();
            return await client.makeRPCRequest(method, params);
        } else if (error.message.includes('rate_limit')) {
            // Wait and retry
            await new Promise(resolve => setTimeout(resolve, 5000));
            return await client.makeRPCRequest(method, params);
        }
        throw error;
    }
}
```

## DEX Integration Examples

### React/Web3 Integration

```javascript
import { useState, useEffect } from 'react';

function useVerusRPC(baseUrl) {
    const [client, setClient] = useState(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        async function initializeClient() {
            try {
                const verusClient = new AnonymousVerusClient(baseUrl);
                await verusClient.getToken(['read', 'write']);
                setClient(verusClient);
            } catch (err) {
                setError(err.message);
            } finally {
                setLoading(false);
            }
        }

        initializeClient();
    }, [baseUrl]);

    return { client, loading, error };
}

// Usage in component
function DEXComponent() {
    const { client, loading, error } = useVerusRPC('https://your-verus-rpc.com');
    const [balance, setBalance] = useState(null);

    const checkBalance = async (address) => {
        if (client) {
            const result = await client.makeRPCRequest('z_getbalance', [address]);
            setBalance(result.result);
        }
    };

    if (loading) return <div>Connecting to Verus RPC...</div>;
    if (error) return <div>Error: {error}</div>;

    return (
        <div>
            <button onClick={() => checkBalance('address')}>
                Check Balance
            </button>
            {balance && <div>Balance: {balance}</div>}
        </div>
    );
}
```

### Node.js/Backend Integration

```javascript
const axios = require('axios');

class VerusRPCService {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
        this.token = null;
    }

    async initialize() {
        const response = await axios.post(`${this.baseUrl}/token/issue`, {
            permissions: ['read', 'write'],
            client_ip: await this.getClientIP(),
            user_agent: 'NodeJS-DEX/1.0'
        });

        this.token = response.data.token;
        console.log(`Anonymous user: ${response.data.user_id}`);
    }

    async makeRequest(method, params = []) {
        if (!this.token) {
            await this.initialize();
        }

        try {
            const response = await axios.post(this.baseUrl, {
                jsonrpc: '2.0',
                method: method,
                params: params,
                id: Date.now()
            }, {
                headers: {
                    'Authorization': `Bearer ${this.token}`
                }
            });

            return response.data;
        } catch (error) {
            if (error.response?.status === 401) {
                // Token expired, get new one
                await this.initialize();
                return this.makeRequest(method, params);
            }
            throw error;
        }
    }

    async getClientIP() {
        try {
            const response = await axios.get('https://api.ipify.org?format=json');
            return response.data.ip;
        } catch {
            return '127.0.0.1';
        }
    }
}

// Usage
const verusService = new VerusRPCService('https://your-verus-rpc.com');
await verusService.initialize();

const balance = await verusService.makeRequest('z_getbalance', ['address']);
```

## Configuration

### Public DEX Configuration

```toml
[security]
development_mode = false
enable_security_headers = true

[security.jwt]
secret_key = "your-super-secret-jwt-key-that-is-at-least-32-characters-long"
expiration_seconds = 3600  # 1 hour for anonymous users
issuer = "verus-rpc-server"
audience = "verus-dex-clients"

[security.token_issuance]
requests_per_hour = 10  # Limit anonymous token requests
enabled = true

[rate_limit]
requests_per_minute = 1000
burst_size = 100
enabled = true
```

## Benefits of Anonymous Access

1. **No Registration Required** - Users can start immediately
2. **Privacy Preserved** - No personal information collected
3. **Simple Integration** - Minimal setup required
4. **Rate Limited** - Prevents abuse while allowing legitimate use
5. **Secure** - JWT tokens with expiration and validation
6. **Scalable** - Can handle high volume of anonymous users

## Monitoring and Analytics

### Anonymous User Metrics
- Token issuance rates by IP
- RPC usage patterns
- Error rates and types
- Geographic distribution
- User agent analysis

### Abuse Prevention
- IP-based rate limiting
- Token usage monitoring
- Suspicious activity detection
- Automatic blocking of abusive IPs


