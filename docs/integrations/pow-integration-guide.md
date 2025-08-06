# Proof of Work (PoW) Integration Guide

## Overview

The Verus RPC Server now supports Proof of Work (PoW) as an integrated payment mechanism for RPC access. This allows users to "mine" for access tokens by solving cryptographic challenges, providing a decentralized way to pay for RPC services.

## Architecture

### Components

1. **PoW Manager**: Handles challenge generation and proof validation
2. **Token Issuer**: Extended to support PoW-validated token issuance
3. **Configuration**: PoW settings managed via `PowConfig`
4. **API Endpoints**: New endpoints for challenge generation and PoW token issuance

### Token Issuance Modes

The system now supports three token issuance modes:

- **Anonymous**: Traditional anonymous token issuance (no PoW required)
- **Proof of Work**: Token issuance after successful PoW challenge completion
- **Partner**: Enhanced tokens for trusted partners (future implementation)

## Configuration

### PoW Configuration

Add the following to your `Conf.toml`:

```toml
[security.pow]
# Default difficulty for PoW challenges (hex string)
default_difficulty = "0000ffff"

# Challenge expiration time in minutes
challenge_expiration_minutes = 10

# Token duration for PoW-validated tokens (seconds)
token_duration_seconds = 14400  # 4 hours

# Rate limit multiplier for PoW-validated tokens
rate_limit_multiplier = 2.0

# Enable PoW challenges
enabled = true
```

### Difficulty Levels

- **Easy**: `"ffffffff"` - For testing and development
- **Medium**: `"0000ffff"` - Default production difficulty
- **Hard**: `"000000ff"` - High difficulty for premium access

## API Usage

### 1. Generate PoW Challenge

```bash
curl -X POST http://localhost:8081/pow/challenge \
  -H "Content-Type: application/json"
```

**Response:**
```json
{
  "id": "ff5e5136-505d-4fb8-8603-26c3ba82b497",
  "challenge": "verus_rpc_ff5e5136-505d-4fb8-8603-26c3ba82b497_1754496539",
  "target_difficulty": "0000ffff",
  "algorithm": "Sha256",
  "expires_at": "2025-08-06T16:18:59.571523900Z",
  "token_duration": 14400,
  "rate_limit_multiplier": 2.0
}
```

### 2. Solve PoW Challenge

The client must find a nonce that, when combined with the challenge and hashed, produces a hash that meets the target difficulty.

**Example Solution Process:**
```javascript
const challenge = "verus_rpc_ff5e5136-505d-4fb8-8603-26c3ba82b497_1754496539";
const targetDifficulty = "0000ffff";

// Try different nonces
for (let nonce = 0; nonce < 1000000; nonce++) {
  const input = challenge + nonce;
  const hash = sha256(input);
  
  // Check if hash meets difficulty (first 8 chars <= target)
  if (hash.substring(0, 8) <= targetDifficulty.substring(0, 8)) {
    console.log(`Found solution: nonce=${nonce}, hash=${hash}`);
    break;
  }
}
```

### 3. Issue PoW Token

```bash
curl -X POST http://localhost:8081/token/issue \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "pow_user_123",
    "permissions": ["read", "write"],
    "client_ip": "192.168.1.100",
    "user_agent": "PoWClient/1.0",
    "mode": "ProofOfWork",
    "pow_challenge": {
      "id": "ff5e5136-505d-4fb8-8603-26c3ba82b497",
      "challenge": "verus_rpc_ff5e5136-505d-4fb8-8603-26c3ba82b497_1754496539",
      "target_difficulty": "0000ffff",
      "algorithm": "Sha256",
      "expires_at": "2025-08-06T16:18:59.571523900Z",
      "token_duration": 14400,
      "rate_limit_multiplier": 2.0
    },
    "pow_proof": {
      "challenge_id": "ff5e5136-505d-4fb8-8603-26c3ba82b497",
      "nonce": "12345",
      "solution": "0000abcd1234567890abcdef1234567890abcdef1234567890abcdef12345678",
      "difficulty": "0000ffff",
      "submitted_at": "2025-08-06T16:15:00.000Z",
      "client_ip": "192.168.1.100"
    }
  }'
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 14400,
  "token_id": "3d3b6871-2e3b-40e4-9668-90304cebcda2",
  "user_id": "pow_user_123"
}
```

## Enhanced Permissions

PoW-validated tokens receive enhanced permissions:

- `pow_validated`: Indicates successful PoW completion
- `rate_multiplier_2.0`: 2x rate limit multiplier
- Extended token duration (4 hours vs 1 hour for anonymous)

## Security Considerations

### Challenge Security
- Challenges expire after 10 minutes (configurable)
- Each challenge has a unique ID to prevent replay attacks
- Client IP is tracked for rate limiting

### Proof Validation
- Solution hash must match the computed hash
- Hash must meet target difficulty
- Challenge must not be expired
- Challenge ID must match

### Rate Limiting
- PoW challenges are rate-limited per IP
- Successful PoW grants enhanced rate limits
- Failed attempts are logged for monitoring

## Mining Software Integration

### Example Mining Client

```javascript
class PoWMiner {
  constructor(tokenServiceUrl) {
    this.tokenServiceUrl = tokenServiceUrl;
  }
  
  async mineForToken(permissions = ['read', 'write']) {
    // Step 1: Get challenge
    const challenge = await this.getChallenge();
    
    // Step 2: Solve PoW
    const solution = await this.solveChallenge(challenge);
    
    // Step 3: Issue token
    const token = await this.issueToken(challenge, solution, permissions);
    
    return token;
  }
  
  async getChallenge() {
    const response = await fetch(`${this.tokenServiceUrl}/pow/challenge`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    });
    return response.json();
  }
  
  async solveChallenge(challenge) {
    const { challenge: challengeStr, target_difficulty } = challenge;
    
    // Simple mining loop (in production, use Web Workers or native mining)
    for (let nonce = 0; nonce < 1000000; nonce++) {
      const input = challengeStr + nonce;
      const hash = await this.sha256(input);
      
      if (hash.substring(0, 8) <= target_difficulty.substring(0, 8)) {
        return {
          challenge_id: challenge.id,
          nonce: nonce.toString(),
          solution: hash,
          difficulty: target_difficulty,
          submitted_at: new Date().toISOString(),
          client_ip: await this.getClientIP()
        };
      }
    }
    
    throw new Error('Could not find solution within iteration limit');
  }
  
  async issueToken(challenge, solution, permissions) {
    const response = await fetch(`${this.tokenServiceUrl}/token/issue`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        user_id: `pow_user_${Date.now()}`,
        permissions,
        client_ip: solution.client_ip,
        user_agent: navigator.userAgent,
        mode: 'ProofOfWork',
        pow_challenge: challenge,
        pow_proof: solution
      })
    });
    
    const result = await response.json();
    return result.token;
  }
  
  async sha256(input) {
    const encoder = new TextEncoder();
    const data = encoder.encode(input);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }
  
  async getClientIP() {
    // In production, get actual client IP
    return '127.0.0.1';
  }
}

// Usage
const miner = new PoWMiner('http://localhost:8081');
const token = await miner.mineForToken(['read', 'write']);
console.log('Mined token:', token);
```

## Testing

### Unit Tests

The implementation includes unit tests:

```bash
cargo test --lib
```

### Integration Tests

Run the PoW integration test:

```bash
cargo run --bin test-pow-integration
```

## Deployment Considerations

### Production Setup

1. **Configure appropriate difficulty levels**
2. **Set up monitoring for PoW attempts**
3. **Implement rate limiting for challenge generation**
4. **Use HTTPS for all API endpoints**
5. **Monitor for abuse patterns**

### Scaling

- PoW challenges can be cached in Redis
- Multiple token service instances can share challenge state
- Consider using a load balancer for high traffic

## Economic Model

### Difficulty Adjustment

The system can implement dynamic difficulty adjustment based on:

- Recent solve times
- Network load
- Economic factors
- User demand

### Cost Analysis

- **Easy difficulty**: ~1-10 seconds of CPU time
- **Medium difficulty**: ~10-60 seconds of CPU time  
- **Hard difficulty**: ~1-10 minutes of CPU time

## Future Enhancements

1. **Multiple algorithms**: Support for Blake3, Argon2, etc.
2. **Difficulty adjustment**: Dynamic difficulty based on solve times
3. **Mining pools**: Allow mining pools to issue tokens
4. **Staking**: Alternative to PoW using staked tokens
5. **Partner integration**: Direct integration with mining software

## Troubleshooting

### Common Issues

1. **Challenge expired**: Increase `challenge_expiration_minutes`
2. **Difficulty too high**: Lower `default_difficulty`
3. **Rate limiting**: Check IP-based rate limits
4. **Hash mismatch**: Verify nonce and challenge combination

### Debug Mode

Enable debug logging to troubleshoot PoW issues:

```toml
[logging]
level = "debug"
```

## Conclusion

The PoW integration provides a secure, decentralized way for users to pay for RPC access through computational work. This creates a fair economic model where users contribute computing resources in exchange for enhanced API access.
