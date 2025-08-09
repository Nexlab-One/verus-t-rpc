# Docker Implementation for Verus RPC Server

## Overview

This Docker implementation provides a complete containerized environment for the Verus RPC Server, including the Verus daemon integration for blockchain operations.

## Architecture

The system consists of several interconnected services:

- **Verus RPC Server**: Rust-based API server for blockchain operations
- **Token Service**: JWT token management service
- **Verus Daemon**: Official VerusCoin blockchain node
- **Redis**: Caching and session storage
- **Caddy**: Reverse proxy with SSL termination
- **Prometheus**: Monitoring and metrics collection
- **Grafana**: Visualization and dashboards
- **Alertmanager**: Alerting and notifications

## Resilience Features

### Circuit Breaker Pattern
The RPC server implements a circuit breaker pattern to handle daemon connectivity issues:

- **Closed State**: Normal operation, requests pass through
- **Open State**: Daemon failing, requests fail fast with fallback responses
- **Half-Open State**: Testing if daemon has recovered

### Graceful Degradation
When the Verus daemon is unavailable, the RPC server provides:

- **Fallback Responses**: Cached or default data for common RPC methods
- **Degraded Health Status**: Service remains available but with warnings
- **Automatic Recovery**: Circuit breaker automatically tests daemon recovery

### Enhanced Health Checks
- **Multi-level Health Status**: Healthy, Degraded, Unhealthy
- **Daemon Connectivity Monitoring**: Real-time daemon availability tracking
- **Circuit Breaker Status**: Current circuit breaker state and metrics

## Quick Start

### Prerequisites

- Docker and Docker Compose installed
- At least 4GB RAM available
- 50GB+ disk space for blockchain data

### 1. Auto-Configuration

Run the auto-configuration script to set up environment variables and configuration files:

```bash
# Windows
.\docker\scripts\auto-config.ps1

# Unix/Linux/macOS
./docker/scripts/auto-config.sh

# Cross-platform (Python)
python3 docker/scripts/auto-config.py
```

### 2. Build and Start Services

```bash
# Build all images
docker-compose -f docker/compose/docker-compose.yml build

# Start all services
docker-compose -f docker/compose/docker-compose.yml up -d
```

### 3. Verify Deployment

```bash
# Check service status
docker-compose -f docker/compose/docker-compose.yml ps

# View logs
docker-compose -f docker/compose/docker-compose.yml logs -f verus-daemon
docker-compose -f docker/compose/docker-compose.yml logs -f verus-rpc-server
```

## Verus Daemon Integration

### Overview

The Verus daemon is the official VerusCoin blockchain node that provides:
- Blockchain synchronization
- Transaction processing
- Zero-knowledge proof support
- RPC interface for blockchain operations

### Configuration

The Verus daemon is configured via `docker/config/verus.conf`:

```ini
# RPC Configuration
rpcuser=verus_rpc_user
rpcpassword=verus_rpc_password
rpcport=27486
rpcbind=0.0.0.0

# Network Configuration
server=1
daemon=1
listen=1
txindex=1
```

### Initial Sync

The Verus daemon requires initial blockchain synchronization:

1. **Bootstrap Data**: Pre-synced blockchain data can be downloaded using the bootstrap script
2. **Zcash Parameters**: Zero-knowledge proof parameters are automatically downloaded during build
3. **Sync Time**: Initial sync may take several hours depending on network conditions

### Health Monitoring

The daemon includes health checks to ensure proper operation:

```bash
# Check daemon status
docker exec verus-daemon verus-cli -conf=/home/verus/.komodo/VRSC/verus.conf getinfo

# View sync progress
docker exec verus-daemon verus-cli -conf=/home/verus/.komodo/VRSC/verus.conf getblockchaininfo
```

## Development Environment

For development with live code reloading:

```bash
# Start development environment
docker-compose -f docker/compose/docker-compose.dev.yml up -d

# View development logs
docker-compose -f docker/compose/docker-compose.dev.yml logs -f
```

## Testing

### Verus Integration Test

Test the complete system with Verus daemon integration:

```bash
# Start test environment
docker-compose -f docker/compose/docker-compose.verus-test.yml up -d

# Test RPC endpoints
curl http://localhost:8080/health
curl http://localhost:27486 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"1.0","id":"test","method":"getinfo","params":[]}'
```

### Circuit Breaker Testing

Test the circuit breaker functionality:

```bash
# Check circuit breaker status
curl http://localhost:8080/admin/circuit-breaker/status

# Reset circuit breaker (admin only)
curl -X POST http://localhost:8080/admin/circuit-breaker/reset

# Test fallback responses when daemon is down
# Stop the daemon and observe fallback responses
docker-compose -f docker/compose/docker-compose.verus-test.yml stop verus-daemon-test
curl http://localhost:8080/health  # Should show "degraded" status
```

### Unit Tests

```bash
# Run tests in container
docker exec verus-rpc-server-test cargo test
```

## Monitoring & Alerting

### Start Monitoring Stack

```bash
# Start monitoring services
docker-compose -f docker/compose/docker-compose.monitoring.yml up -d

# Access monitoring dashboards
# Grafana: http://localhost:3000 (admin/admin123)
# Prometheus: http://localhost:9090
# Alertmanager: http://localhost:9093
```

### Key Metrics

- **Circuit Breaker State**: Current state (Closed/Open/Half-Open)
- **Daemon Availability**: Real-time daemon connectivity status
- **Request Success Rate**: Percentage of successful RPC requests
- **Response Times**: Average and percentile response times
- **Error Rates**: Rate of different error types

### Alerts

The monitoring stack includes alerts for:

- **Daemon Down**: Verus daemon becomes unavailable
- **Circuit Breaker Open**: Circuit breaker opens due to failures
- **High Error Rate**: Error rate exceeds threshold
- **High Response Time**: Response times exceed acceptable limits
- **Low Success Rate**: Success rate drops below threshold

## Security Considerations

### Network Security

- All services run on internal Docker networks
- Only necessary ports are exposed
- RPC access is restricted to internal network

### Container Security

- Non-root user execution
- Read-only filesystems where possible
- Dropped capabilities
- Security headers enabled

### Circuit Breaker Security

- Admin endpoints protected by authentication
- Circuit breaker state changes logged
- Fallback responses clearly marked as degraded

## Troubleshooting

### Common Issues

1. **Daemon Not Starting**
   ```bash
   # Check daemon logs
   docker-compose logs verus-daemon
   
   # Check configuration
   docker exec verus-daemon cat /home/verus/.komodo/VRSC/verus.conf
   ```

2. **Circuit Breaker Stuck Open**
   ```bash
   # Check circuit breaker status
   curl http://localhost:8080/admin/circuit-breaker/status
   
   # Manually reset if needed
   curl -X POST http://localhost:8080/admin/circuit-breaker/reset
   ```

3. **Health Check Failing**
   ```bash
   # Check health endpoint
   curl http://localhost:8080/health
   
   # Check service logs
   docker-compose logs verus-rpc-server
   ```

### Performance Tuning

1. **Circuit Breaker Configuration**
   ```toml
   [verus.circuit_breaker]
   failure_threshold = 5              # Number of failures before opening
   recovery_timeout_seconds = 60      # Time to wait before testing recovery
   half_open_max_requests = 3         # Max requests in half-open state
   ```

2. **Redis Configuration**
   ```toml
   [cache]
   enabled = true
   redis_url = "redis://redis:6379"
   default_ttl = 300                  # Cache TTL in seconds
   max_size = 104857600               # Max cache size in bytes
   ```

## Production Deployment

### High Availability

For production deployments, consider:

1. **Multiple RPC Server Instances**: Load balance across multiple containers
2. **Redis Cluster**: Use Redis cluster for high availability caching
3. **Database Backups**: Regular backups of blockchain data
4. **Monitoring**: Comprehensive monitoring and alerting
5. **Logging**: Centralized logging with log aggregation

### Scaling

```bash
# Scale RPC server instances
docker-compose -f docker/compose/docker-compose.yml up -d --scale verus-rpc-server=3

# Scale with load balancer
docker-compose -f docker/compose/docker-compose.yml up -d --scale verus-rpc-server=5
```

## Maintenance

### Regular Tasks

1. **Update Dependencies**: Regular security updates
2. **Monitor Logs**: Check for errors and warnings
3. **Backup Data**: Regular backups of configuration and data
4. **Performance Review**: Monitor metrics and optimize as needed

### Emergency Procedures

1. **Daemon Recovery**: Restart daemon if it becomes unresponsive
2. **Circuit Breaker Reset**: Reset circuit breaker if stuck
3. **Service Restart**: Restart services if they become unhealthy
4. **Rollback**: Rollback to previous version if issues occur

## Directory Structure

```
docker/
├── compose/
│   ├── docker-compose.yml (production)
│   ├── docker-compose.dev.yml (development)
│   ├── docker-compose.test.yml (testing)
│   └── docker-compose.verus-test.yml (verus integration test)
├── scripts/
│   ├── auto-config.ps1 (Windows)
│   ├── auto-config.sh (Unix)
│   ├── auto-config.py (cross-platform)
│   ├── verus-bootstrap.sh (blockchain bootstrap)
│   ├── docker-build.ps1 (Windows)
│   ├── docker-build.sh (Unix)
│   └── docker-security.sh (security validation)
├── config/
│   ├── verus.conf (verus daemon configuration)
│   ├── production.toml (production config)
│   ├── development.toml (development config)
│   ├── Caddyfile (reverse proxy)
│   └── prometheus.yml (monitoring)
├── Dockerfile (production)
├── Dockerfile.dev (development)
└── Dockerfile.verus (verus daemon)
```

## Support

For issues related to:
- **VerusCoin**: Visit [VerusCoin GitHub](https://github.com/Veruscoin/VerusCoin)
- **Docker**: Check Docker documentation
- **RPC Server**: Review application logs and configuration

## License

This implementation follows the same license as the VerusCoin project.
