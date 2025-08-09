# Auto-Configuration Script for Verus RPC Server Docker Deployment (Windows)
# Automatically sets up environment variables and configuration files

param(
    [switch]$Help,
    [switch]$Force,
    [string]$Domain = "yourdomain.com",
    [string]$Email = "your-email@domain.com"
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"
$White = "White"

# Configuration
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$DockerDir = Split-Path -Parent $ScriptDir
$ProjectRoot = Split-Path -Parent $DockerDir
$ConfigDir = Join-Path $ProjectRoot "config"
$DockerConfigDir = Join-Path $DockerDir "config"
$BackupDir = Join-Path $ProjectRoot "backups"
$EnvFile = Join-Path $ProjectRoot ".env"

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

function Write-Info {
    param([string]$Message)
    Write-Log $Message $Blue
}

# Generate secure random string
function Get-RandomString {
    param([int]$Length = 32)
    
    $chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
    $random = ""
    for ($i = 0; $i -lt $Length; $i++) {
        $random += $chars[(Get-Random -Maximum $chars.Length)]
    }
    return $random
}

# Generate secure password
function Get-SecurePassword {
    param([int]$Length = 32)
    
    $chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*"
    $password = ""
    for ($i = 0; $i -lt $Length; $i++) {
        $password += $chars[(Get-Random -Maximum $chars.Length)]
    }
    return $password
}

# Check if running as administrator
function Test-Administrator {
    if ([Security.Principal.WindowsIdentity]::GetCurrent().Groups -contains "S-1-5-32-544") {
        Write-Warning "This script is running as administrator. Consider running as a regular user."
    }
}

# Check prerequisites
function Test-Prerequisites {
    Write-Success "Checking prerequisites..."
    
    # Check for required commands
    $requiredCommands = @("docker", "docker-compose")
    foreach ($cmd in $requiredCommands) {
        try {
            $null = Get-Command $cmd -ErrorAction Stop
        }
        catch {
            Write-Error "$cmd is not installed or not in PATH"
        }
    }
    
    # Check for config directories
    if (-not (Test-Path $ConfigDir)) {
        Write-Success "Creating config directory..."
        New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
    }
    
    if (-not (Test-Path $DockerConfigDir)) {
        Write-Success "Creating Docker config directory..."
        New-Item -ItemType Directory -Path $DockerConfigDir -Force | Out-Null
    }
    
    # Check for backup directory
    if (-not (Test-Path $BackupDir)) {
        Write-Success "Creating backup directory..."
        New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
    }
    
    Write-Success "Prerequisites check completed"
}

# Backup existing configuration
function Backup-Config {
    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $backupFile = Join-Path $BackupDir "config-backup-$timestamp.zip"
    
    if ((Test-Path $EnvFile) -or (Test-Path $ConfigDir)) {
        Write-Success "Creating backup of existing configuration..."
        
        $filesToBackup = @()
        if (Test-Path $EnvFile) { $filesToBackup += $EnvFile }
        if (Test-Path $ConfigDir) { $filesToBackup += $ConfigDir }
        
        if ($filesToBackup.Count -gt 0) {
            Compress-Archive -Path $filesToBackup -DestinationPath $backupFile -Force
            Write-Success "Backup created: $backupFile"
        }
    }
}

# Generate environment variables
function New-EnvironmentVariables {
    Write-Success "Generating environment variables..."
    
    # Generate secure passwords and keys
    $redisPassword = Get-SecurePassword 16
    $jwtSecret = Get-SecurePassword 32
    $rpcUser = "verus_rpc_$(Get-RandomString 8)"
    $rpcPassword = Get-SecurePassword 16
    
    # Create .env file content
    $envContent = @"
# Auto-generated environment variables for Verus RPC Server
# Generated on: $(Get-Date)
# WARNING: Keep this file secure and do not commit to version control

# Redis Configuration
REDIS_PASSWORD=$redisPassword

# JWT Configuration
JWT_SECRET_KEY=$jwtSecret

# Verus Daemon Configuration
VERUS_RPC_USER=$rpcUser
VERUS_RPC_PASSWORD=$rpcPassword

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
"@
    
    # Write to file
    $envContent | Out-File -FilePath $EnvFile -Encoding UTF8 -Force
    
    # Set secure permissions (Windows equivalent)
    $acl = Get-Acl $EnvFile
    $acl.SetAccessRuleProtection($true, $false)
    $rule = New-Object System.Security.AccessControl.FileSystemAccessRule("$env:USERNAME", "FullControl", "Allow")
    $acl.AddAccessRule($rule)
    Set-Acl $EnvFile $acl
    
    Write-Success "Environment variables generated and saved to $EnvFile"
    
    # Set environment variables for current session
    $env:REDIS_PASSWORD = $redisPassword
    $env:JWT_SECRET_KEY = $jwtSecret
    $env:VERUS_RPC_USER = $rpcUser
    $env:VERUS_RPC_PASSWORD = $rpcPassword
}

# Generate production configuration
function New-ProductionConfig {
    Write-Success "Generating production configuration..."
    
    # Load environment variables
    if (Test-Path $EnvFile) {
        Get-Content $EnvFile | ForEach-Object {
            if ($_ -match '^([^#][^=]+)=(.*)$') {
                $env:$($matches[1]) = $matches[2]
            }
        }
    }
    
    $productionConfig = @"
[verus]
rpc_url = "http://verus-daemon:27486"
rpc_user = "$($env:VERUS_RPC_USER ?? 'verus_rpc_user')"
rpc_password = "$($env:VERUS_RPC_PASSWORD ?? 'verus_rpc_password')"
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
cors_origins = ["https://$Domain", "https://app.$Domain"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "$($env:JWT_SECRET_KEY ?? 'your-32-character-cryptographically-secure-secret-key')"
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
"@
    
    $productionConfig | Out-File -FilePath (Join-Path $ConfigDir "production.toml") -Encoding UTF8 -Force
    Write-Success "Production configuration generated"
}

# Generate development configuration
function New-DevelopmentConfig {
    Write-Success "Generating development configuration..."
    
    # Load environment variables
    if (Test-Path $EnvFile) {
        Get-Content $EnvFile | ForEach-Object {
            if ($_ -match '^([^#][^=]+)=(.*)$') {
                $env:$($matches[1]) = $matches[2]
            }
        }
    }
    
    $developmentConfig = @"
[verus]
rpc_url = "http://verus-daemon-dev:27486"
rpc_user = "$($env:VERUS_RPC_USER ?? 'verus_rpc_user')"
rpc_password = "$($env:VERUS_RPC_PASSWORD ?? 'verus_rpc_password')"
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
secret_key = "$($env:JWT_SECRET_KEY ?? 'dev-secret-key-32-chars-long-12345')"
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
"@
    
    $developmentConfig | Out-File -FilePath (Join-Path $ConfigDir "development.toml") -Encoding UTF8 -Force
    Write-Success "Development configuration generated"
}

# Generate Verus daemon configuration
function New-VerusConfig {
    Write-Success "Generating Verus daemon configuration..."
    
    # Load environment variables
    if (Test-Path $EnvFile) {
        Get-Content $EnvFile | ForEach-Object {
            if ($_ -match '^([^#][^=]+)=(.*)$') {
                $env:$($matches[1]) = $matches[2]
            }
        }
    }
    
    $verusConfig = @"
# Verus Daemon Configuration for Docker
# Auto-generated on: $(Get-Date)

# RPC Configuration
rpcuser=$($env:VERUS_RPC_USER ?? 'verus_rpc_user')
rpcpassword=$($env:VERUS_RPC_PASSWORD ?? 'verus_rpc_password')
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
"@
    
    $verusConfig | Out-File -FilePath (Join-Path $DockerConfigDir "verus.conf") -Encoding UTF8 -Force
    Write-Success "Verus daemon configuration generated"
}

# Generate Caddy configuration
function New-CaddyConfig {
    Write-Success "Generating Caddy configuration..."
    
    $caddyConfig = @"
# Production Caddyfile for Verus RPC Server
# Auto-generated on: $(Get-Date)

$Domain {
    # Automatic HTTPS with Let's Encrypt
    tls $Email
    
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
"@
    
    $caddyConfig | Out-File -FilePath (Join-Path $DockerConfigDir "Caddyfile") -Encoding UTF8 -Force
    Write-Success "Caddy configuration generated"
}

# Generate development Caddy configuration
function New-CaddyDevConfig {
    Write-Success "Generating development Caddy configuration..."
    
    $caddyDevConfig = @"
# Development Caddyfile for Verus RPC Server
# Auto-generated on: $(Get-Date)

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
"@
    
    $caddyDevConfig | Out-File -FilePath (Join-Path $DockerConfigDir "Caddyfile.dev") -Encoding UTF8 -Force
    Write-Success "Development Caddy configuration generated"
}

# Generate Prometheus configuration
function New-PrometheusConfig {
    Write-Success "Generating Prometheus configuration..."
    
    $prometheusConfig = @"
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
"@
    
    $prometheusConfig | Out-File -FilePath (Join-Path $DockerConfigDir "prometheus.yml") -Encoding UTF8 -Force
    Write-Success "Prometheus configuration generated"
}

# Validate configuration
function Test-Configuration {
    Write-Success "Validating configuration..."
    
    $errors = 0
    
    # Check if .env file exists
    if (-not (Test-Path $EnvFile)) {
        Write-Error "Environment file not found: $EnvFile"
        $errors++
    }
    
    # Check if configuration files exist
    $configFiles = @("production.toml", "development.toml")
    foreach ($configFile in $configFiles) {
        $configPath = Join-Path $ConfigDir $configFile
        if (-not (Test-Path $configPath)) {
            Write-Error "Configuration file not found: $configPath"
            $errors++
        }
    }
    
    if ($errors -eq 0) {
        Write-Success "Configuration validation passed"
    }
    else {
        Write-Error "Configuration validation failed with $errors errors"
    }
}

# Show configuration summary
function Show-Summary {
    Write-Success "Configuration Summary"
    Write-Host "===================="
    Write-Host "Environment file: $EnvFile"
    Write-Host "Config directory: $ConfigDir"
    Write-Host "Backup directory: $BackupDir"
    Write-Host ""
    Write-Host "Generated files:"
    Write-Host "- .env (environment variables)"
    Write-Host "- config/production.toml"
    Write-Host "- config/development.toml"
    Write-Host "- docker/config/verus.conf"
    Write-Host "- docker/config/Caddyfile"
    Write-Host "- docker/config/Caddyfile.dev"
    Write-Host "- docker/config/prometheus.yml"
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host "1. Review and customize the generated configuration files"
    Write-Host "2. Update domain names in docker/config/Caddyfile (currently set to: $Domain)"
    Write-Host "3. Set up SSL certificates (if needed)"
    Write-Host "4. Run: docker-compose -f docker/compose/docker-compose.yml up -d"
    Write-Host ""
    Write-Host "Security notes:"
    Write-Host "- Keep .env file secure and do not commit to version control"
    Write-Host "- Change default passwords in production"
    Write-Host "- Review and customize security settings"
}

# Show help
function Show-Help {
    Write-Host @"
Verus RPC Server Auto-Configuration Script (Windows)

Usage: .\auto-config.ps1 [Options]

Options:
    -Help                    Show this help message
    -Force                   Overwrite existing configuration files
    -Domain <string>         Domain name for Caddy configuration [default: yourdomain.com]
    -Email <string>          Email for Let's Encrypt certificates [default: your-email@domain.com]

Examples:
    .\auto-config.ps1
    .\auto-config.ps1 -Domain "api.mydomain.com" -Email "admin@mydomain.com"
    .\auto-config.ps1 -Force

"@
}

# Main function
function Main {
    if ($Help) {
        Show-Help
        return
    }
    
    Write-Host "=========================================="
    Write-Host "Verus RPC Server Auto-Configuration Script"
    Write-Host "=========================================="
    Write-Host ""
    
    # Check if running as administrator
    Test-Administrator
    
    # Check prerequisites
    Test-Prerequisites
    
    # Backup existing configuration
    Backup-Config
    
    # Generate all configurations
    New-EnvironmentVariables
    New-ProductionConfig
    New-DevelopmentConfig
    New-VerusConfig
    New-CaddyConfig
    New-CaddyDevConfig
    New-PrometheusConfig
    
    # Validate configuration
    Test-Configuration
    
    # Show summary
    Show-Summary
    
    Write-Success "Auto-configuration completed successfully!"
}

# Run main function
Main
