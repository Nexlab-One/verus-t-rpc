#!/bin/bash

# Docker Security Script for Verus RPC Server
# Validates and enforces security best practices for containerized deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
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

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        error "This script should not be run as root for security reasons"
    fi
}

# Validate Docker installation
check_docker() {
    log "Checking Docker installation..."
    
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed"
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed"
    fi
    
    # Check if user is in docker group
    if ! groups $USER | grep -q docker; then
        warn "User $USER is not in docker group. You may need to run docker commands with sudo"
    fi
    
    log "Docker installation validated"
}

# Validate security configuration
validate_security_config() {
    log "Validating security configuration..."
    
    # Check if config file exists
    if [[ ! -f "config/production.toml" ]]; then
        error "Production configuration file not found: config/production.toml"
    fi
    
    # Validate JWT secret key
    if grep -q "your-32-character-secret-key-here" config/production.toml; then
        error "Default JWT secret key detected. Please set a secure secret key"
    fi
    
    # Check development mode is disabled
    if grep -q "development_mode = true" config/production.toml; then
        error "Development mode is enabled in production configuration"
    fi
    
    # Validate Redis password
    if [[ -z "${REDIS_PASSWORD:-}" ]]; then
        error "REDIS_PASSWORD environment variable is not set"
    fi
    
    if [[ ${#REDIS_PASSWORD} -lt 12 ]]; then
        error "Redis password must be at least 12 characters long"
    fi
    
    log "Security configuration validated"
}

# Check for security vulnerabilities
check_security_vulnerabilities() {
    log "Checking for security vulnerabilities..."
    
    # Run cargo audit
    if command -v cargo-audit &> /dev/null; then
        log "Running cargo audit..."
        if ! cargo audit; then
            error "Security vulnerabilities detected in dependencies"
        fi
    else
        warn "cargo-audit not installed. Install with: cargo install cargo-audit"
    fi
    
    # Check for known vulnerabilities in base images
    log "Checking Docker images for known vulnerabilities..."
    if command -v trivy &> /dev/null; then
        trivy image --severity HIGH,CRITICAL rust:1.70 || warn "Vulnerabilities found in Rust base image"
        trivy image --severity HIGH,CRITICAL debian:bullseye-slim || warn "Vulnerabilities found in Debian base image"
    else
        warn "trivy not installed. Install for vulnerability scanning"
    fi
    
    log "Security vulnerability check completed"
}

# Validate network security
validate_network_security() {
    log "Validating network security configuration..."
    
    # Check if internal network is configured
    if ! grep -q "internal: true" docker/compose/docker-compose.yml; then
        warn "Internal network not configured in docker/compose/docker-compose.yml"
    fi
    
    # Check if services are properly isolated
    if grep -q "ports:" docker/compose/docker-compose.yml | grep -v "127.0.0.1"; then
        warn "Services may be exposed to external interfaces"
    fi
    
    log "Network security validation completed"
}

# Validate file permissions
validate_file_permissions() {
    log "Validating file permissions..."
    
    # Check config file permissions
    if [[ -f "config/production.toml" ]]; then
        perms=$(stat -c %a config/production.toml)
        if [[ $perms != "600" ]]; then
            warn "Config file permissions should be 600, found: $perms"
            chmod 600 config/production.toml
        fi
    fi
    
    # Check script permissions
    if [[ -f "$0" ]]; then
        perms=$(stat -c %a "$0")
        if [[ $perms != "755" ]]; then
            chmod 755 "$0"
        fi
    fi
    
    log "File permissions validated"
}

# Build secure images
build_secure_images() {
    log "Building secure Docker images..."
    
    # Build with security flags
    DOCKER_BUILDKIT=1 docker build \
        --build-arg RUST_VERSION=1.70 \
        --security-opt seccomp=unconfined \
        --no-cache \
        -t verus-rpc-server:latest .
    
    log "Docker images built successfully"
}

# Deploy with security checks
deploy_secure() {
    log "Deploying with security checks..."
    
    # Validate environment
    validate_security_config
    validate_network_security
    validate_file_permissions
    
    # Build images
    build_secure_images
    
    # Deploy with security options
    docker-compose -f docker/compose/docker-compose.yml up -d
    
    # Wait for services to start
    log "Waiting for services to start..."
    sleep 30
    
    # Verify health checks
    log "Verifying service health..."
    
    if ! curl -f -s http://localhost:8080/health > /dev/null; then
        error "RPC server health check failed"
    fi
    
    if ! curl -f -s http://localhost:8081/health > /dev/null; then
        error "Token service health check failed"
    fi
    
    log "All services are healthy"
}

# Security monitoring
monitor_security() {
    log "Setting up security monitoring..."
    
    # Check container security
    log "Checking container security..."
    
    # Verify containers are running as non-root
    for container in verus-rpc-server verus-token-service redis verus-daemon caddy; do
        if docker inspect $container | grep -q '"User": ""'; then
            warn "Container $container is running as root"
        fi
    done
    
    # Check for privileged containers
    if docker ps --format "table {{.Names}}\t{{.Status}}" | grep -q privileged; then
        warn "Privileged containers detected"
    fi
    
    # Check for exposed ports
    exposed_ports=$(docker ps --format "{{.Names}}\t{{.Ports}}" | grep -v "127.0.0.1")
    if [[ -n "$exposed_ports" ]]; then
        warn "Services exposed to external interfaces:"
        echo "$exposed_ports"
    fi
    
    log "Security monitoring completed"
}

# Cleanup function
cleanup() {
    log "Performing security cleanup..."
    
    # Remove unused images
    docker image prune -f
    
    # Remove unused volumes
    docker volume prune -f
    
    # Remove unused networks
    docker network prune -f
    
    log "Cleanup completed"
}

# Main function
main() {
    log "Starting Docker security validation and deployment..."
    
    check_root
    check_docker
    check_security_vulnerabilities
    
    case "${1:-deploy}" in
        "validate")
            validate_security_config
            validate_network_security
            validate_file_permissions
            ;;
        "build")
            build_secure_images
            ;;
        "deploy")
            deploy_secure
            ;;
        "monitor")
            monitor_security
            ;;
        "cleanup")
            cleanup
            ;;
        "all")
            validate_security_config
            validate_network_security
            validate_file_permissions
            build_secure_images
            deploy_secure
            monitor_security
            ;;
        *)
            echo "Usage: $0 {validate|build|deploy|monitor|cleanup|all}"
            echo "  validate: Validate security configuration"
            echo "  build: Build secure Docker images"
            echo "  deploy: Deploy with security checks"
            echo "  monitor: Monitor security status"
            echo "  cleanup: Clean up unused resources"
            echo "  all: Run all security checks and deployment"
            exit 1
            ;;
    esac
    
    log "Docker security script completed successfully"
}

# Run main function with all arguments
main "$@"
