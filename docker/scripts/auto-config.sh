#!/bin/bash

# Auto-Configuration Script for Verus RPC Server Docker Deployment
# Automatically sets up environment variables and configuration files

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$DOCKER_DIR")"
CONFIG_DIR="$PROJECT_ROOT/config"
DOCKER_CONFIG_DIR="$DOCKER_DIR/config"
BACKUP_DIR="$PROJECT_ROOT/backups"
ENV_FILE="$PROJECT_ROOT/.env"

# Logging functions
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] INFO: $1${NC}"
}

# Generate secure random string
generate_random_string() {
    local length=$1
    openssl rand -base64 $((length * 3 / 4)) | tr -d "=+/" | cut -c1-${length}
}

# Generate secure password
generate_secure_password() {
    local length=${1:-32}
    openssl rand -base64 $((length * 3 / 4)) | tr -d "=+/" | cut -c1-${length}
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        error "This script should not be run as root"
    fi
}

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check for required commands
    local required_commands=("openssl" "docker" "docker-compose")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            error "$cmd is not installed or not in PATH"
        fi
    done
    
    # Check for config directories
    if [[ ! -d "$CONFIG_DIR" ]]; then
        log "Creating config directory..."
        mkdir -p "$CONFIG_DIR"
    fi
    
    if [[ ! -d "$DOCKER_CONFIG_DIR" ]]; then
        log "Creating Docker config directory..."
        mkdir -p "$DOCKER_CONFIG_DIR"
    fi
    
    # Check for backup directory
    if [[ ! -d "$BACKUP_DIR" ]]; then
        log "Creating backup directory..."
        mkdir -p "$BACKUP_DIR"
    fi
    
    log "Prerequisites check completed"
}

# Backup existing configuration
backup_config() {
    local timestamp=$(date +%Y%m%d-%H%M%S)
    local backup_file="$BACKUP_DIR/config-backup-$timestamp.tar.gz"
    
    if [[ -f "$ENV_FILE" ]] || [[ -d "$CONFIG_DIR" ]]; then
        log "Creating backup of existing configuration..."
        tar -czf "$backup_file" -C "$PROJECT_ROOT" .env config/ 2>/dev/null || true
        log "Backup created: $backup_file"
    fi
}

# Generate environment variables
generate_env_vars() {
    log "Generating environment variables..."
    
    # Generate secure passwords and keys
    local redis_password=$(generate_secure_password 16)
    local jwt_secret=$(generate_secure_password 32)
    local rpc_user="verus_rpc_$(generate_random_string 8)"
    local rpc_password=$(generate_secure_password 16)
    
    # Create .env file
    cat > "$ENV_FILE" << EOF
# Auto-generated environment variables for Verus RPC Server
# Generated on: $(date)
# WARNING: Keep this file secure and do not commit to version control

# Redis Configuration
REDIS_PASSWORD=$redis_password

# JWT Configuration
JWT_SECRET_KEY=$jwt_secret

# Verus Daemon Configuration
VERUS_RPC_USER=$rpc_user
VERUS_RPC_PASSWORD=$rpc_password

# Server Configuration
VERUS_RPC_PORT=8080
TOKEN_SERVICE_PORT=8081

# Security Configuration
VERUS_RPC__SECURITY__DEVELOPMENT_MODE=false
VERUS_RPC__SECURITY__ENABLE_SECURITY_HEADERS=true

# Logging Configuration
RUST_LOG=info
RUST_BACKTRACE=0

# Docker Configuration
DOCKER_COMPOSE_PROJECT_NAME=verus-rpc-server
EOF
    
    # Set secure permissions
    chmod 600 "$ENV_FILE"
    log "Environment variables generated and saved to $ENV_FILE"
    
    # Export variables for current session
    export REDIS_PASSWORD="$redis_password"
    export JWT_SECRET_KEY="$jwt_secret"
    export VERUS_RPC_USER="$rpc_user"
    export VERUS_RPC_PASSWORD="$rpc_password"
}

# Generate production configuration
generate_production_config() {
    log "Generating production configuration..."
    
    # Source environment variables
    if [[ -f "$ENV_FILE" ]]; then
        source "$ENV_FILE"
    fi
    
    cat > "$CONFIG_DIR/production.toml" << EOF
[verus]
rpc_url = "http://verus-daemon:27486"
rpc_user = "${VERUS_RPC_USER:-verus_rpc_user}"
rpc_password = "${VERUS_RPC_PASSWORD:-verus_rpc_password}"
timeout_seconds = 30
max_retries = 3

[server]
bind_address = "0.0.0.0"
port = 8080
max_request_size = 1048576
worker_threads = 8

[security]
development_mode = false
enable_security_headers = true
enable_custom_headers = true
cors_origins = ["https://yourdomain.com", "https://app.yourdomain.com"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "${JWT_SECRET_KEY:-your-32-character-cryptographically-secure-secret-key}"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"

[rate_limit]
enabled = true
requests_per_minute = 1000
burst_size = 100

[cache]
enabled = true
redis_url = "redis://redis:6379"
default_ttl = 300
max_size = 104857600

[logging]
level = "info"
format = "json"
structured = true
EOF
    
    chmod 600 "$CONFIG_DIR/production.toml"
    log "Production configuration generated"
}

# Generate development configuration
generate_development_config() {
    log "Generating development configuration..."
    
    # Source environment variables
    if [[ -f "$ENV_FILE" ]]; then
        source "$ENV_FILE"
    fi
    
    cat > "$CONFIG_DIR/development.toml" << EOF
[verus]
rpc_url = "http://verus-daemon-dev:27486"
rpc_user = "${VERUS_RPC_USER:-verus_rpc_user}"
rpc_password = "${VERUS_RPC_PASSWORD:-verus_rpc_password}"
timeout_seconds = 30
max_retries = 3

[server]
bind_address = "0.0.0.0"
port = 8080
max_request_size = 1048576
worker_threads = 2

[security]
development_mode = true
enable_security_headers = true
enable_custom_headers = false
cors_origins = ["*"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "${JWT_SECRET_KEY:-dev-secret-key-32-chars-long-12345}"
expiration_seconds = 3600
issuer = "verus-rpc-server-dev"
audience = "verus-clients-dev"

[rate_limit]
enabled = true
requests_per_minute = 1000
burst_size = 100

[cache]
enabled = true
redis_url = "redis://redis-dev:6379"
default_ttl = 300
max_size = 104857600

[logging]
level = "debug"
format = "json"
structured = true
EOF
    
    chmod 600 "$CONFIG_DIR/development.toml"
    log "Development configuration generated"
}

# Generate Verus daemon configuration
generate_verus_config() {
    log "Generating Verus daemon configuration..."
    
    # Source environment variables
    if [[ -f "$ENV_FILE" ]]; then
        source "$ENV_FILE"
    fi
    
    cat > "$DOCKER_CONFIG_DIR/verus.conf" << EOF
# Verus Daemon Configuration for Docker
# Auto-generated on: $(date)

# RPC Configuration
rpcuser=${VERUS_RPC_USER:-verus_rpc_user}
rpcpassword=${VERUS_RPC_PASSWORD:-verus_rpc_password}
rpcport=27486
rpcbind=0.0.0.0
rpcallowip=0.0.0.0/0

# Network Configuration
listen=1
server=1
daemon=0
txindex=1

# Security Configuration
rpcworkqueue=16
rpcthreads=8

# Logging
debug=rpc
debug=net
debug=selectcoins

# Performance
dbcache=450
maxorphantx=10
maxmempool=50
par=2

# Wallet Configuration
wallet=1
walletnotify=echo "Wallet transaction: %s"
EOF
    
    chmod 600 "$DOCKER_CONFIG_DIR/verus.conf"
    log "Verus daemon configuration generated"
}

# Generate Caddy configuration
generate_caddy_config() {
    log "Generating Caddy configuration..."
    
    cat > "$DOCKER_CONFIG_DIR/Caddyfile" << EOF
# Production Caddyfile for Verus RPC Server
# Auto-generated on: $(date)

yourdomain.com {
    # Automatic HTTPS with Let's Encrypt
    tls your-email@domain.com
    
    # Rate limiting
    rate_limit {
        zone api
        events 1000
        window 1m
    }
    
    # Security headers
    header {
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        Permissions-Policy "geolocation=(), microphone=(), camera=()"
        -Server
        Cache-Control "no-cache, no-store, must-revalidate"
        Pragma "no-cache"
        Expires "0"
    }
    
    # Gzip compression
    encode gzip
    
    # Reverse proxy to Verus RPC Server
    reverse_proxy verus-rpc-server:8080 {
        health_uri /health
        health_interval 30s
        health_timeout 10s
        lb_policy round_robin
        timeout 30s
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
    
    # Logging
    log {
        output file /var/log/caddy/verus-rpc.log
        format json
        level INFO
    }
}

# Health check endpoint (internal only)
:8081 {
    bind 127.0.0.1
    reverse_proxy verus-rpc-server:8080 {
        health_uri /health
        health_interval 30s
        health_timeout 10s
    }
}

# Metrics endpoint (internal only)
:8082 {
    bind 127.0.0.1
    reverse_proxy verus-rpc-server:8080 {
        health_uri /metrics
        health_interval 30s
        health_timeout 10s
    }
}
EOF
    
    log "Caddy configuration generated"
}

# Generate development Caddy configuration
generate_caddy_dev_config() {
    log "Generating development Caddy configuration..."
    
    cat > "$DOCKER_CONFIG_DIR/Caddyfile.dev" << EOF
# Development Caddyfile for Verus RPC Server
# Auto-generated on: $(date)

:80 {
    # Rate limiting (relaxed for development)
    rate_limit {
        zone api
        events 2000
        window 1m
    }
    
    # Security headers
    header {
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        -Server
        Cache-Control "no-cache, no-store, must-revalidate"
    }
    
    # Gzip compression
    encode gzip
    
    # Reverse proxy to development server
    reverse_proxy verus-rpc-server-dev:8080 {
        health_uri /health
        health_interval 30s
        health_timeout 10s
        timeout 30s
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
    }
    
    # Logging
    log {
        output stdout
        format console
        level DEBUG
    }
}

# Token service endpoint
:8081 {
    reverse_proxy token-service-dev:8081 {
        health_uri /health
        health_interval 30s
        health_timeout 10s
    }
}
EOF
    
    log "Development Caddy configuration generated"
}

# Generate Prometheus configuration
generate_prometheus_config() {
    log "Generating Prometheus configuration..."
    
    cat > "$DOCKER_CONFIG_DIR/prometheus.yml" << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  - job_name: 'verus-rpc-server'
    static_configs:
      - targets: ['verus-rpc-server:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
    honor_labels: true

  - job_name: 'token-service'
    static_configs:
      - targets: ['token-service:8081']
    metrics_path: '/metrics'
    scrape_interval: 10s
    honor_labels: true

  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
EOF
    
    log "Prometheus configuration generated"
}

# Validate configuration
validate_config() {
    log "Validating configuration..."
    
    local errors=0
    
    # Check if .env file exists
    if [[ ! -f "$ENV_FILE" ]]; then
        error "Environment file not found: $ENV_FILE"
        ((errors++))
    fi
    
    # Check if configuration files exist
    local config_files=("production.toml" "development.toml")
    for config_file in "${config_files[@]}"; do
        if [[ ! -f "$CONFIG_DIR/$config_file" ]]; then
            error "Configuration file not found: $CONFIG_DIR/$config_file"
            ((errors++))
        fi
    done
    
    # Check file permissions
    if [[ -f "$ENV_FILE" ]] && [[ $(stat -c %a "$ENV_FILE") != "600" ]]; then
        warn "Environment file should have 600 permissions"
        chmod 600 "$ENV_FILE"
    fi
    
    if [[ $errors -eq 0 ]]; then
        log "Configuration validation passed"
    else
        error "Configuration validation failed with $errors errors"
    fi
}

# Show configuration summary
show_summary() {
    log "Configuration Summary"
    echo "===================="
    echo "Environment file: $ENV_FILE"
    echo "Config directory: $CONFIG_DIR"
    echo "Backup directory: $BACKUP_DIR"
    echo ""
    echo "Generated files:"
    echo "- .env (environment variables)"
    echo "- config/production.toml"
    echo "- config/development.toml"
    echo "- docker/config/verus.conf"
    echo "- docker/config/Caddyfile"
    echo "- docker/config/Caddyfile.dev"
    echo "- docker/config/prometheus.yml"
    echo ""
    echo "Next steps:"
    echo "1. Review and customize the generated configuration files"
    echo "2. Update domain names in docker/config/Caddyfile"
    echo "3. Set up SSL certificates (if needed)"
    echo "4. Run: docker-compose -f docker/compose/docker-compose.yml up -d"
    echo ""
    echo "Security notes:"
    echo "- Keep .env file secure and do not commit to version control"
    echo "- Change default passwords in production"
    echo "- Review and customize security settings"
}

# Main function
main() {
    echo "=========================================="
    echo "Verus RPC Server Auto-Configuration Script"
    echo "=========================================="
    echo ""
    
    # Check if running as root
    check_root
    
    # Check prerequisites
    check_prerequisites
    
    # Backup existing configuration
    backup_config
    
    # Generate all configurations
    generate_env_vars
    generate_production_config
    generate_development_config
    generate_verus_config
    generate_caddy_config
    generate_caddy_dev_config
    generate_prometheus_config
    
    # Validate configuration
    validate_config
    
    # Show summary
    show_summary
    
    log "Auto-configuration completed successfully!"
}

# Run main function
main "$@"
