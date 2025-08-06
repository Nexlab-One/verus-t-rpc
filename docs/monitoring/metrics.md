# Metrics & Monitoring

This document provides comprehensive information about the monitoring and observability features of the Verus RPC Server.

## 📊 Monitoring Overview

The Verus RPC Server provides comprehensive monitoring capabilities through:

- **Prometheus Metrics**: Detailed performance and business metrics
- **Health Checks**: Service health monitoring
- **Structured Logging**: JSON-formatted logs for analysis
- **Performance Monitoring**: Response time and throughput tracking

## 🔍 Available Metrics

### Request Metrics

#### Request Counters

```
# Total requests by method
verus_rpc_requests_total{method="getinfo"} 42
verus_rpc_requests_total{method="getblock"} 15
verus_rpc_requests_total{method="getrawtransaction"} 8

# Request duration histogram
verus_rpc_request_duration_seconds_bucket{method="getinfo",le="0.1"} 35
verus_rpc_request_duration_seconds_bucket{method="getinfo",le="0.5"} 42
verus_rpc_request_duration_seconds_bucket{method="getinfo",le="1.0"} 42
verus_rpc_request_duration_seconds_bucket{method="getinfo",le="+Inf"} 42
verus_rpc_request_duration_seconds_sum{method="getinfo"} 2.5
verus_rpc_request_duration_seconds_count{method="getinfo"} 42
```

#### Error Metrics

```
# Error counts by type
verus_rpc_errors_total{error_type="validation_error"} 3
verus_rpc_errors_total{error_type="authentication_error"} 1
verus_rpc_errors_total{error_type="rate_limit_error"} 5
verus_rpc_errors_total{error_type="rpc_error"} 2
```

### Performance Metrics

#### Response Time Metrics

```
# Response time percentiles
verus_rpc_response_time_p50_seconds{method="getinfo"} 0.05
verus_rpc_response_time_p95_seconds{method="getinfo"} 0.15
verus_rpc_response_time_p99_seconds{method="getinfo"} 0.25

# Response time by status
verus_rpc_response_time_seconds{status="success",method="getinfo"} 0.08
verus_rpc_response_time_seconds{status="error",method="getinfo"} 0.02
```

#### Throughput Metrics

```
# Requests per second
verus_rpc_requests_per_second{method="getinfo"} 2.1
verus_rpc_requests_per_second{method="getblock"} 0.8

# Concurrent requests
verus_rpc_concurrent_requests 5
```

### Security Metrics

#### Authentication Metrics

```
# Authentication attempts
verus_rpc_auth_attempts_total{result="success"} 95
verus_rpc_auth_attempts_total{result="failure"} 5

# Authentication failures by reason
verus_rpc_auth_failures_total{reason="invalid_token"} 3
verus_rpc_auth_failures_total{reason="expired_token"} 2
verus_rpc_auth_failures_total{reason="missing_token"} 1
```

#### Rate Limiting Metrics

```
# Rate limit hits
verus_rpc_rate_limit_hits_total{ip="192.168.1.100"} 10
verus_rpc_rate_limit_hits_total{ip="10.0.0.50"} 5

# Rate limit remaining
verus_rpc_rate_limit_remaining{ip="192.168.1.100"} 90
```

### Cache Metrics

#### Cache Performance

```
# Cache hits and misses
verus_rpc_cache_hits_total{method="getinfo"} 25
verus_rpc_cache_misses_total{method="getinfo"} 17
verus_rpc_cache_hit_ratio{method="getinfo"} 0.595

# Cache size and evictions
verus_rpc_cache_size_bytes 10485760
verus_rpc_cache_evictions_total 5
```

### System Metrics

#### Resource Usage

```
# Memory usage
verus_rpc_memory_usage_bytes 52428800
verus_rpc_memory_usage_percent 12.5

# CPU usage
verus_rpc_cpu_usage_percent 8.3

# Active connections
verus_rpc_active_connections 15
```

#### External Service Metrics

```
# Verus daemon health
verus_rpc_verus_daemon_healthy 1
verus_rpc_verus_daemon_response_time_seconds 0.05

# Redis health
verus_rpc_redis_healthy 1
verus_rpc_redis_response_time_seconds 0.001
```

## 🏥 Health Checks

### Health Check Endpoint

The server provides a health check endpoint at `/health`:

```bash
curl http://127.0.0.1:8080/health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2024-12-06T15:30:00Z",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "checks": {
    "verus_daemon": "healthy",
    "redis": "healthy",
    "memory": "healthy",
    "disk": "healthy"
  }
}
```

### Health Check Components

#### Verus Daemon Health

```rust
// Check Verus daemon connectivity
async fn check_verus_daemon() -> HealthStatus {
    match rpc_adapter.call_method("getinfo", &[]).await {
        Ok(_) => HealthStatus::Healthy,
        Err(_) => HealthStatus::Unhealthy,
    }
}
```

#### Redis Health

```rust
// Check Redis connectivity
async fn check_redis() -> HealthStatus {
    match cache_adapter.ping().await {
        Ok(_) => HealthStatus::Healthy,
        Err(_) => HealthStatus::Unhealthy,
    }
}
```

#### System Health

```rust
// Check system resources
fn check_system_health() -> HealthStatus {
    let memory_usage = get_memory_usage();
    let disk_usage = get_disk_usage();
    
    if memory_usage > 90.0 || disk_usage > 95.0 {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Healthy
    }
}
```

## 📈 Prometheus Integration

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'verus-rpc-server'
    static_configs:
      - targets: ['verus-rpc-server:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
    scrape_timeout: 5s
    
    # Relabeling rules
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        regex: '([^:]+)(?::\d+)?'
        replacement: '${1}'
```

### Metrics Endpoint

Access Prometheus metrics:

```bash
# Prometheus format
curl http://127.0.0.1:8080/metrics

# JSON format
curl http://127.0.0.1:8080/prometheus
```

### Custom Metrics

The server exposes custom metrics for business logic:

```rust
// Custom business metrics
pub struct BusinessMetrics {
    requests_by_method: CounterVec,
    errors_by_type: CounterVec,
    cache_performance: HistogramVec,
    authentication_events: CounterVec,
}

impl BusinessMetrics {
    pub fn new() -> Self {
        let requests_by_method = register_counter_vec!(
            "verus_rpc_requests_total",
            "Total number of RPC requests",
            &["method"]
        ).unwrap();
        
        let errors_by_type = register_counter_vec!(
            "verus_rpc_errors_total",
            "Total number of errors",
            &["error_type"]
        ).unwrap();
        
        // ... other metrics
        
        Self {
            requests_by_method,
            errors_by_type,
            cache_performance,
            authentication_events,
        }
    }
}
```

## 📊 Grafana Dashboards

### Main Dashboard

```json
{
  "dashboard": {
    "title": "Verus RPC Server",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_requests_total[5m])",
            "legendFormat": "{{method}}"
          }
        ]
      },
      {
        "title": "Response Time (95th percentile)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(verus_rpc_request_duration_seconds_bucket[5m]))",
            "legendFormat": "{{method}}"
          }
        ]
      },
      {
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_errors_total[5m])",
            "legendFormat": "{{error_type}}"
          }
        ]
      },
      {
        "title": "Cache Hit Ratio",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_cache_hits_total[5m]) / (rate(verus_rpc_cache_hits_total[5m]) + rate(verus_rpc_cache_misses_total[5m]))",
            "legendFormat": "{{method}}"
          }
        ]
      }
    ]
  }
}
```

### Security Dashboard

```json
{
  "dashboard": {
    "title": "Security Monitoring",
    "panels": [
      {
        "title": "Authentication Failures",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_auth_failures_total[5m])",
            "legendFormat": "{{reason}}"
          }
        ]
      },
      {
        "title": "Rate Limit Hits",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_rate_limit_hits_total[5m])",
            "legendFormat": "{{ip}}"
          }
        ]
      },
      {
        "title": "Validation Errors",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(verus_rpc_errors_total{error_type=\"validation_error\"}[5m])",
            "legendFormat": "Validation Errors"
          }
        ]
      }
    ]
  }
}
```

## 📝 Logging

### Structured Logging

The server uses structured JSON logging:

```json
{
  "timestamp": "2024-12-06T15:30:00Z",
  "level": "info",
  "target": "verus_rpc_server",
  "message": "Request processed",
  "request_id": "req-123",
  "method": "getinfo",
  "duration_ms": 45,
  "status": "success",
  "client_ip": "192.168.1.100",
  "user_agent": "curl/7.68.0"
}
```

### Log Configuration

```toml
[logging]
level = "info"
format = "json"
structured = true
output = "stdout"  # or "file" for file logging
file_path = "/var/log/verus-rpc-server/app.log"
max_file_size = "100MB"
max_files = 10
```

### Log Levels

- **error**: Errors that need immediate attention
- **warn**: Warning conditions
- **info**: General information about application flow
- **debug**: Detailed information for debugging
- **trace**: Very detailed information

### Log Categories

```rust
// Different log categories
tracing::info!(target: "verus_rpc_server", "Server started");
tracing::debug!(target: "verus_rpc_request", "Processing request");
tracing::warn!(target: "verus_rpc_security", "Authentication failed");
tracing::error!(target: "verus_rpc_error", "RPC call failed");
```

## 🚨 Alerting

### Alert Rules

```yaml
# alerting.yml
groups:
  - name: verus_rpc_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(verus_rpc_errors_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"

      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(verus_rpc_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time detected"
          description: "95th percentile response time is {{ $value }} seconds"

      - alert: ServiceDown
        expr: up{job="verus-rpc-server"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Verus RPC Server is down"
          description: "The server has been down for more than 1 minute"

      - alert: HighMemoryUsage
        expr: verus_rpc_memory_usage_percent > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}%"

      - alert: AuthenticationFailures
        expr: rate(verus_rpc_auth_failures_total[5m]) > 0.05
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High authentication failure rate"
          description: "Authentication failure rate is {{ $value }} failures per second"
```

### Alert Notifications

```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alertmanager@example.com'

route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'team-verus'

receivers:
  - name: 'team-verus'
    email_configs:
      - to: 'team-verus@example.com'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/...'
        channel: '#verus-alerts'
```

## 🔧 Monitoring Configuration

### Environment Variables

```bash
# Monitoring configuration
export METRICS_ENABLED="true"
export METRICS_PORT="8080"
export HEALTH_CHECK_INTERVAL="30s"
export LOG_LEVEL="info"
export LOG_FORMAT="json"
```

### Configuration File

```toml
[monitoring]
enabled = true
metrics_port = 8080
health_check_interval = "30s"
prometheus_enabled = true
custom_metrics_enabled = true

[logging]
level = "info"
format = "json"
structured = true
output = "stdout"
file_path = "/var/log/verus-rpc-server/app.log"
max_file_size = "100MB"
max_files = 10

[alerts]
enabled = true
error_rate_threshold = 0.1
response_time_threshold = 1.0
memory_usage_threshold = 80.0
```

## 📊 Performance Monitoring

### Key Performance Indicators (KPIs)

1. **Request Rate**: Requests per second by method
2. **Response Time**: 95th and 99th percentile response times
3. **Error Rate**: Errors per second by type
4. **Cache Hit Ratio**: Cache performance
5. **Resource Usage**: CPU, memory, and disk usage
6. **External Service Health**: Verus daemon and Redis health

### Performance Baselines

```yaml
# performance_baselines.yml
baselines:
  request_rate:
    getinfo: 100  # requests per second
    getblock: 50
    getrawtransaction: 30
  
  response_time:
    p95:
      getinfo: 0.1  # seconds
      getblock: 0.2
      getrawtransaction: 0.15
    p99:
      getinfo: 0.2
      getblock: 0.5
      getrawtransaction: 0.3
  
  error_rate:
    max: 0.01  # 1% error rate
  
  cache_hit_ratio:
    min: 0.8  # 80% cache hit ratio
```

## 🔗 Related Documentation

- [Health Checks](./health-checks.md) - Detailed health check configuration
- [Logging](./logging.md) - Comprehensive logging guide
- [Performance Monitoring](./performance.md) - Performance optimization
- [Production Deployment](../deployment/production.md) - Production monitoring setup
