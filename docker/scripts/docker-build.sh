#!/bin/bash

# Unix/Linux script for building and running Verus RPC Server Docker containers
# Cross-platform compatible Docker build script for Unix-based systems

set -euo pipefail

# Default values
ENVIRONMENT="development"
BUILD=false
RUN=false
STOP=false
CLEAN=false
HELP=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --environment|-e)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --build|-b)
            BUILD=true
            shift
            ;;
        --run|-r)
            RUN=true
            shift
            ;;
        --stop|-s)
            STOP=true
            shift
            ;;
        --clean|-c)
            CLEAN=true
            shift
            ;;
        --help|-h)
            HELP=true
            shift
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Show help
show_help() {
    cat << EOF
Verus RPC Server Docker Management Script

Usage: $0 [OPTIONS]

Options:
    -e, --environment ENV    Environment to use (development|production) [default: development]
    -b, --build             Build Docker images
    -r, --run               Start Docker containers
    -s, --stop              Stop Docker containers
    -c, --clean             Clean Docker resources (images, volumes, networks)
    -h, --help              Show this help message

Examples:
    $0 -b -e development
    $0 -r -e production
    $0 -s -e development
    $0 -c

EOF
}

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
    fi
    log "Docker is installed"
}

# Check if Docker Compose is installed
check_docker_compose() {
    if ! command -v docker-compose &> /dev/null; then
        error "Docker Compose is not installed or not in PATH"
    fi
    log "Docker Compose is installed"
}

# Validate environment
validate_environment() {
    local env="$1"
    
    if [[ "$env" == "development" ]]; then
        if [[ ! -f "config/development.toml" ]]; then
            error "Development configuration file not found: config/development.toml"
        fi
        log "Development environment validated"
    elif [[ "$env" == "production" ]]; then
        if [[ ! -f "config/production.toml" ]]; then
            error "Production configuration file not found: config/production.toml"
        fi
        log "Production environment validated"
    else
        error "Invalid environment: $env. Use 'development' or 'production'"
    fi
}

# Build Docker images
build_images() {
    local env="$1"
    
    log "Building Docker images for $env environment..."
    
    if [[ "$env" == "development" ]]; then
        docker-compose -f docker/compose/docker-compose.dev.yml build
    else
        docker-compose -f docker/compose/docker-compose.yml build
    fi
    
    if [[ $? -eq 0 ]]; then
        log "Docker images built successfully"
    else
        error "Failed to build Docker images"
    fi
}

# Run Docker containers
start_containers() {
    local env="$1"
    
    log "Starting Docker containers for $env environment..."
    
    # Set environment variables
    export REDIS_PASSWORD="default_secure_password_123"
    
    if [[ "$env" == "development" ]]; then
        docker-compose -f docker/compose/docker-compose.dev.yml up -d
    else
        docker-compose -f docker/compose/docker-compose.yml up -d
    fi
    
    if [[ $? -eq 0 ]]; then
        log "Docker containers started successfully"
        log "Waiting for services to be ready..."
        sleep 30
        
        # Check health
        test_health
    else
        error "Failed to start Docker containers"
    fi
}

# Stop Docker containers
stop_containers() {
    local env="$1"
    
    log "Stopping Docker containers for $env environment..."
    
    if [[ "$env" == "development" ]]; then
        docker-compose -f docker/compose/docker-compose.dev.yml down
    else
        docker-compose -f docker/compose/docker-compose.yml down
    fi
    
    if [[ $? -eq 0 ]]; then
        log "Docker containers stopped successfully"
    else
        error "Failed to stop Docker containers"
    fi
}

# Clean Docker resources
clean_docker() {
    log "Cleaning Docker resources..."
    
    # Stop all containers
    docker-compose -f docker/compose/docker-compose.yml down 2>/dev/null || true
    docker-compose -f docker/compose/docker-compose.dev.yml down 2>/dev/null || true
    
    # Remove unused images
    docker image prune -f
    
    # Remove unused volumes
    docker volume prune -f
    
    # Remove unused networks
    docker network prune -f
    
    log "Docker resources cleaned successfully"
}

# Test service health
test_health() {
    log "Testing service health..."
    
    # Test RPC Server
    if curl -f -s http://localhost:8080/health > /dev/null; then
        log "RPC Server is healthy"
    else
        warn "RPC Server health check failed"
    fi
    
    # Test Token Service
    if curl -f -s http://localhost:8081/health > /dev/null; then
        log "Token Service is healthy"
    else
        warn "Token Service health check failed"
    fi
}

# Main execution
main() {
    if [[ "$HELP" == "true" ]]; then
        show_help
        return
    fi
    
    log "Starting Verus RPC Server Docker management..."
    
    # Validate prerequisites
    check_docker
    check_docker_compose
    validate_environment "$ENVIRONMENT"
    
    # Execute requested actions
    if [[ "$BUILD" == "true" ]]; then
        build_images "$ENVIRONMENT"
    fi
    
    if [[ "$RUN" == "true" ]]; then
        start_containers "$ENVIRONMENT"
    fi
    
    if [[ "$STOP" == "true" ]]; then
        stop_containers "$ENVIRONMENT"
    fi
    
    if [[ "$CLEAN" == "true" ]]; then
        clean_docker
    fi
    
    # Default action if no specific action provided
    if [[ "$BUILD" == "false" && "$RUN" == "false" && "$STOP" == "false" && "$CLEAN" == "false" ]]; then
        log "No action specified. Building and running development environment..."
        build_images "development"
        start_containers "development"
    fi
    
    log "Docker management completed successfully"
}

# Run main function
main "$@"
