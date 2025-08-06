# Mining Pool Integration Guide

## Overview

This guide explains how to integrate external mining pools with the Verus RPC Server for enhanced Proof of Work (PoW) validation and security.

## Architecture

The mining pool integration follows a client-server architecture where:

- **RPC Server**: Acts as a client to the mining pool
- **Mining Pool**: Validates shares and provides enhanced security
- **Miners**: Submit shares to the pool for validation

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   RPC Server    │    │   Mining Pool    │    │   Client/Miner  │
│                 │    │                  │    │                 │
│ • Pool Client   │◄──►│ • Share Valid.   │◄──►│ • Mining Client │
│ • Validation    │    │ • Difficulty Adj │    │ • Hash Solving  │
│ • Rate Limiting │    │ • Miner Mgmt     │    │ • Share Submit  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Configuration

### Mining Pool Configuration

Add the following section to your configuration file:

```toml
[security.mining_pool]
# Pool server URL
pool_url = "https://pool.example.com"
# Pool API key for authentication
api_key = "your-pool-api-key"
# Pool public key for signature verification
public_key = "pool-public-key"
# Connection timeout in seconds
timeout_seconds = 30
# Maximum retry attempts
max_retries = 3
# Circuit breaker threshold (number of failures before opening)
circuit_breaker_threshold = 5
# Circuit breaker timeout in seconds
circuit_breaker_timeout = 60
# Rate limiting requests per minute
requests_per_minute = 100
# Enable pool integration
enabled = true
```

### Configuration Parameters

| Parameter | Description | Default | Required |
|-----------|-------------|---------|----------|
| `pool_url` | URL of the mining pool server | - | Yes |
| `api_key` | API key for pool authentication | - | Yes |
| `public_key` | Public key for signature verification | - | Yes |
| `timeout_seconds` | HTTP request timeout | 30 | No |
| `max_retries` | Maximum retry attempts | 3 | No |
| `circuit_breaker_threshold` | Failures before opening circuit | 5 | No |
| `circuit_breaker_timeout` | Circuit breaker timeout | 60 | No |
| `requests_per_minute` | Rate limit per miner | 100 | No |
| `enabled` | Enable pool integration | false | No |

## API Integration

### Pool Share Structure

```rust
pub struct PoolShare {
    pub challenge_id: String,
    pub miner_address: String,
    pub nonce: String,
    pub solution: String,
    pub difficulty: f64,
    pub timestamp: DateTime<Utc>,
    pub pool_signature: Option<String>,
}
```

### Token Issuance with Pool Validation

```rust
// Create a pool share
let share = PoolShare {
    challenge_id: "challenge-uuid".to_string(),
    miner_address: "miner-address".to_string(),
    nonce: "nonce-value".to_string(),
    solution: "hash-solution".to_string(),
    difficulty: 1.5,
    timestamp: Utc::now(),
    pool_signature: Some("pool-signature".to_string()),
};

// Create token issuance request
let request = TokenIssuanceRequest {
    user_id: "user-id".to_string(),
    permissions: vec!["read".to_string(), "write".to_string()],
    client_ip: Some("127.0.0.1".to_string()),
    user_agent: Some("MiningClient/1.0".to_string()),
    custom_expiration: None,
    mode: TokenIssuanceMode::PoolValidated(share),
    pow_challenge: None,
};

// Issue token with pool validation
let response = token_issuer.issue_token(request).await?;
```

## Security Features

### 1. Circuit Breaker Pattern

The integration includes a circuit breaker to handle pool failures gracefully:

- **Closed**: Normal operation
- **Open**: Pool is failing, requests fail fast
- **Half-Open**: Testing if pool is back online

### 2. Rate Limiting

- Per-miner rate limiting
- Configurable limits
- Automatic window management

### 3. Enhanced Permissions

Pool-validated tokens receive enhanced permissions:

- `pool_validated`: Indicates pool validation
- `miner_{address}`: Miner-specific permission
- `rate_multiplier_2.0`: Enhanced rate limits

### 4. Cryptographic Signatures

- Pool signatures for validation
- Public key verification
- Tamper-proof share validation

## Error Handling

### Common Error Scenarios

1. **Pool Unavailable**
   ```
   Error: Mining pool service is temporarily unavailable
   ```

2. **Invalid Share**
   ```
   Error: Pool share validation failed
   ```

3. **Rate Limit Exceeded**
   ```
   Error: Rate limit exceeded for miner: {address}
   ```

4. **Configuration Missing**
   ```
   Error: Mining pool configuration not found
   ```

### Error Recovery

- Circuit breaker automatically retries after timeout
- Graceful degradation when pool is unavailable
- Fallback to standalone PoW validation

## Testing

### Unit Tests

```bash
# Run all tests
cargo test --lib

# Run mining pool specific tests
cargo test mining_pool

# Run token issuer tests
cargo test token_issuer
```

### Integration Tests

```bash
# Test with mock pool server
cargo test --test integration_tests
```

## Deployment

### 1. Pool Server Setup

Ensure your mining pool server provides:

- `/api/v1/share/validate` endpoint
- `/api/v1/health` endpoint
- Proper authentication
- Rate limiting

### 2. RPC Server Configuration

1. Copy `Conf.mining-pool.toml` to your deployment
2. Update pool configuration with your pool details
3. Set appropriate rate limits and timeouts
4. Enable pool integration

### 3. Monitoring

Monitor the following metrics:

- Pool connectivity status
- Share validation success rate
- Circuit breaker state
- Rate limiting events

## Example Usage

### JavaScript Client

```javascript
// Submit share to pool and get token
async function getPoolValidatedToken(share) {
    const response = await fetch('/token/issue', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({
            user_id: 'user-id',
            permissions: ['read', 'write'],
            client_ip: '127.0.0.1',
            user_agent: 'MiningClient/1.0',
            mode: 'PoolValidated',
            pool_share: share
        })
    });
    
    return response.json();
}

// Use the token for RPC calls
async function makeRpcCall(token, method, params) {
    const response = await fetch('/rpc', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: method,
            params: params
        })
    });
    
    return response.json();
}
```

### Python Client

```python
import requests
import json

def get_pool_validated_token(share):
    response = requests.post('/token/issue', json={
        'user_id': 'user-id',
        'permissions': ['read', 'write'],
        'client_ip': '127.0.0.1',
        'user_agent': 'MiningClient/1.0',
        'mode': 'PoolValidated',
        'pool_share': share
    })
    return response.json()

def make_rpc_call(token, method, params):
    response = requests.post('/rpc', 
        headers={
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {token}'
        },
        json={
            'jsonrpc': '2.0',
            'id': 1,
            'method': method,
            'params': params
        }
    )
    return response.json()
```

## Troubleshooting

### Common Issues

1. **Pool Connection Failed**
   - Check pool URL and network connectivity
   - Verify API key and authentication
   - Check firewall settings

2. **Share Validation Fails**
   - Verify share format and fields
   - Check pool signature validation
   - Ensure difficulty requirements are met

3. **Rate Limiting Issues**
   - Monitor rate limit configuration
   - Check miner address format
   - Verify rate limit windows

4. **Circuit Breaker Issues**
   - Check pool health endpoint
   - Monitor failure thresholds
   - Verify timeout settings

### Debug Mode

Enable debug logging to troubleshoot issues:

```toml
[logging]
level = "debug"
format = "text"
structured = false
```

## Security Considerations

### 1. API Key Security
- Store API keys securely
- Use environment variables
- Rotate keys regularly

### 2. Network Security
- Use HTTPS for pool communication
- Implement proper TLS configuration
- Monitor for suspicious activity

### 3. Rate Limiting
- Configure appropriate limits
- Monitor for abuse
- Implement IP-based restrictions

### 4. Signature Verification
- Verify pool signatures
- Use secure key management
- Implement signature validation

## Performance Optimization

### 1. Connection Pooling
- Configure HTTP client pooling
- Optimize connection reuse
- Monitor connection metrics

### 2. Caching
- Cache pool responses
- Implement share validation caching
- Use Redis for distributed caching

### 3. Async Processing
- Use async/await for I/O operations
- Implement concurrent share validation
- Optimize for high throughput

## Conclusion

The mining pool integration provides enhanced security and validation for the Verus RPC Server. By leveraging external mining pools, you can:

- Improve share validation accuracy
- Implement advanced rate limiting
- Add miner reputation tracking
- Enhance overall security posture

Follow this guide to successfully deploy and maintain mining pool integration in your Verus RPC Server environment.
