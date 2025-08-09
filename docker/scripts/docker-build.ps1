# PowerShell script for building and running Verus RPC Server Docker containers
# Cross-platform compatible Docker build script for Windows

param(
    [string]$Environment = "development",
    [switch]$Build,
    [switch]$Run,
    [switch]$Stop,
    [switch]$Clean,
    [switch]$Help
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$White = "White"

# Logging functions
function Write-Log {
    param([string]$Message, [string]$Color = $White)
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] $Message" -ForegroundColor $Color
}

function Write-Success {
    param([string]$Message)
    Write-Log $Message $Green
}

function Write-Warning {
    param([string]$Message)
    Write-Log "WARNING: $Message" $Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Log "ERROR: $Message" $Red
    exit 1
}

# Check if Docker is installed
function Test-Docker {
    try {
        $null = docker --version
        Write-Success "Docker is installed"
    }
    catch {
        Write-Error "Docker is not installed or not in PATH"
    }
}

# Check if Docker Compose is installed
function Test-DockerCompose {
    try {
        $null = docker-compose --version
        Write-Success "Docker Compose is installed"
    }
    catch {
        Write-Error "Docker Compose is not installed or not in PATH"
    }
}

# Validate environment
function Test-Environment {
    param([string]$Env)
    
    if ($Env -eq "development") {
        if (-not (Test-Path "config/development.toml")) {
            Write-Error "Development configuration file not found: config/development.toml"
        }
        Write-Success "Development environment validated"
    }
    elseif ($Env -eq "production") {
        if (-not (Test-Path "config/production.toml")) {
            Write-Error "Production configuration file not found: config/production.toml"
        }
        Write-Success "Production environment validated"
    }
    else {
        Write-Error "Invalid environment: $Env. Use 'development' or 'production'"
    }
}

# Build Docker images
function Build-Images {
    param([string]$Env)
    
    Write-Log "Building Docker images for $Env environment..."
    
    if ($Env -eq "development") {
        docker-compose -f docker/compose/docker-compose.dev.yml build
    }
    else {
        docker-compose -f docker/compose/docker-compose.yml build
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Docker images built successfully"
    }
    else {
        Write-Error "Failed to build Docker images"
    }
}

# Run Docker containers
function Start-Containers {
    param([string]$Env)
    
    Write-Log "Starting Docker containers for $Env environment..."
    
    # Set environment variables
    $env:REDIS_PASSWORD = "default_secure_password_123"
    
    if ($Env -eq "development") {
        docker-compose -f docker/compose/docker-compose.dev.yml up -d
    }
    else {
        docker-compose -f docker/compose/docker-compose.yml up -d
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Docker containers started successfully"
        Write-Log "Waiting for services to be ready..."
        Start-Sleep -Seconds 30
        
        # Check health
        Test-Health
    }
    else {
        Write-Error "Failed to start Docker containers"
    }
}

# Stop Docker containers
function Stop-Containers {
    param([string]$Env)
    
    Write-Log "Stopping Docker containers for $Env environment..."
    
    if ($Env -eq "development") {
        docker-compose -f docker/compose/docker-compose.dev.yml down
    }
    else {
        docker-compose -f docker/compose/docker-compose.yml down
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Success "Docker containers stopped successfully"
    }
    else {
        Write-Error "Failed to stop Docker containers"
    }
}

# Clean Docker resources
function Clean-Docker {
    Write-Log "Cleaning Docker resources..."
    
    # Stop all containers
    docker-compose -f docker/compose/docker-compose.yml down
    docker-compose -f docker/compose/docker-compose.dev.yml down
    
    # Remove unused images
    docker image prune -f
    
    # Remove unused volumes
    docker volume prune -f
    
    # Remove unused networks
    docker network prune -f
    
    Write-Success "Docker resources cleaned successfully"
}

# Test service health
function Test-Health {
    Write-Log "Testing service health..."
    
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080/health" -UseBasicParsing -TimeoutSec 10
        if ($response.StatusCode -eq 200) {
            Write-Success "RPC Server is healthy"
        }
        else {
            Write-Warning "RPC Server health check failed with status: $($response.StatusCode)"
        }
    }
    catch {
        Write-Warning "RPC Server health check failed: $($_.Exception.Message)"
    }
    
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8081/health" -UseBasicParsing -TimeoutSec 10
        if ($response.StatusCode -eq 200) {
            Write-Success "Token Service is healthy"
        }
        else {
            Write-Warning "Token Service health check failed with status: $($response.StatusCode)"
        }
    }
    catch {
        Write-Warning "Token Service health check failed: $($_.Exception.Message)"
    }
}

# Show help
function Show-Help {
    Write-Host @"
Verus RPC Server Docker Management Script

Usage: .\docker-build.ps1 [Options]

Options:
    -Environment <string>    Environment to use (development|production) [default: development]
    -Build                   Build Docker images
    -Run                     Start Docker containers
    -Stop                    Stop Docker containers
    -Clean                   Clean Docker resources (images, volumes, networks)
    -Help                    Show this help message

Examples:
    .\docker-build.ps1 -Build -Environment development
    .\docker-build.ps1 -Run -Environment production
    .\docker-build.ps1 -Stop -Environment development
    .\docker-build.ps1 -Clean

"@
}

# Main execution
function Main {
    if ($Help) {
        Show-Help
        return
    }
    
    Write-Log "Starting Verus RPC Server Docker management..."
    
    # Validate prerequisites
    Test-Docker
    Test-DockerCompose
    Test-Environment $Environment
    
    # Execute requested actions
    if ($Build) {
        Build-Images $Environment
    }
    
    if ($Run) {
        Start-Containers $Environment
    }
    
    if ($Stop) {
        Stop-Containers $Environment
    }
    
    if ($Clean) {
        Clean-Docker
    }
    
    # Default action if no specific action provided
    if (-not ($Build -or $Run -or $Stop -or $Clean)) {
        Write-Log "No action specified. Building and running development environment..."
        Build-Images "development"
        Start-Containers "development"
    }
    
    Write-Success "Docker management completed successfully"
}

# Run main function
Main
