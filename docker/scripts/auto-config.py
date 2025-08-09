#!/usr/bin/env python3
"""
Auto-Configuration Script for Verus RPC Server Docker Deployment
Cross-platform Python script for automatically setting up environment variables and configuration files
"""

import os
import sys
import json
import secrets
import string
import argparse
import subprocess
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# Colors for output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

def log(message: str, color: str = Colors.NC) -> None:
    """Print a timestamped log message with color."""
    timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
    print(f"{color}[{timestamp}] {message}{Colors.NC}")

def success(message: str) -> None:
    """Print a success message."""
    log(message, Colors.GREEN)

def warning(message: str) -> None:
    """Print a warning message."""
    log(f"WARNING: {message}", Colors.YELLOW)

def error(message: str) -> None:
    """Print an error message and exit."""
    log(f"ERROR: {message}", Colors.RED)
    sys.exit(1)

def info(message: str) -> None:
    """Print an info message."""
    log(message, Colors.BLUE)

class AutoConfig:
    def __init__(self, domain: str = "yourdomain.com", email: str = "your-email@domain.com"):
        self.domain = domain
        self.email = email
        
        # Get script directory and project root
        self.script_dir = Path(__file__).parent
        self.docker_dir = self.script_dir.parent
        self.project_root = self.docker_dir.parent
        self.config_dir = self.project_root / "config"
        self.docker_config_dir = self.docker_dir / "config"
        self.backup_dir = self.project_root / "backups"
        self.env_file = self.project_root / ".env"
        
        # Generated values
        self.redis_password = None
        self.jwt_secret = None
        self.rpc_user = None
        self.rpc_password = None

    def generate_secure_password(self, length: int = 32) -> str:
        """Generate a secure random password."""
        chars = string.ascii_letters + string.digits + "!@#$%^&*"
        return ''.join(secrets.choice(chars) for _ in range(length))

    def generate_random_string(self, length: int = 8) -> str:
        """Generate a random string for usernames."""
        chars = string.ascii_letters + string.digits
        return ''.join(secrets.choice(chars) for _ in range(length))

    def check_prerequisites(self) -> None:
        """Check if required tools are installed."""
        success("Checking prerequisites...")
        
        # Check for required commands
        required_commands = ["docker", "docker-compose"]
        for cmd in required_commands:
            try:
                subprocess.run([cmd, "--version"], capture_output=True, check=True)
            except (subprocess.CalledProcessError, FileNotFoundError):
                error(f"{cmd} is not installed or not in PATH")
        
        # Create directories if they don't exist
        for directory in [self.config_dir, self.docker_config_dir, self.backup_dir]:
            directory.mkdir(exist_ok=True)
            if not directory.exists():
                success(f"Creating directory: {directory}")
        
        success("Prerequisites check completed")

    def backup_config(self) -> None:
        """Backup existing configuration files."""
        timestamp = datetime.now().strftime('%Y%m%d-%H%M%S')
        backup_file = self.backup_dir / f"config-backup-{timestamp}.zip"
        
        files_to_backup = []
        if self.env_file.exists():
            files_to_backup.append(self.env_file)
        if self.config_dir.exists():
            files_to_backup.extend(self.config_dir.glob("*"))
        
        if files_to_backup:
            success("Creating backup of existing configuration...")
            with zipfile.ZipFile(backup_file, 'w', zipfile.ZIP_DEFLATED) as zipf:
                for file_path in files_to_backup:
                    if file_path.is_file():
                        zipf.write(file_path, file_path.relative_to(self.project_root))
            success(f"Backup created: {backup_file}")

    def generate_env_vars(self) -> None:
        """Generate environment variables."""
        success("Generating environment variables...")
        
        # Generate secure passwords and keys
        self.redis_password = self.generate_secure_password(16)
        self.jwt_secret = self.generate_secure_password(32)
        self.rpc_user = f"verus_rpc_{self.generate_random_string(8)}"
        self.rpc_password = self.generate_secure_password(16)
        
        # Create .env file content
        env_content = f"""# Auto-generated environment variables for Verus RPC Server
# Generated on: {datetime.now()}
# WARNING: Keep this file secure and do not commit to version control

# Redis Configuration
REDIS_PASSWORD={self.redis_password}

# JWT Configuration
JWT_SECRET_KEY={self.jwt_secret}

# Verus Daemon Configuration
VERUS_RPC_USER={self.rpc_user}
VERUS_RPC_PASSWORD={self.rpc_password}

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
"""
        
        # Write to file
        with open(self.env_file, 'w') as f:
            f.write(env_content)
        
        # Set secure permissions (Unix-like systems)
        if os.name != 'nt':  # Not Windows
            os.chmod(self.env_file, 0o600)
        
        success(f"Environment variables generated and saved to {self.env_file}")

    def generate_production_config(self) -> None:
        """Generate production configuration."""
        success("Generating production configuration...")
        
        production_config = f"""[verus]
rpc_url = "http://verus-daemon:27486"
rpc_user = "{self.rpc_user}"
rpc_password = "{self.rpc_password}"
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
cors_origins = ["https://{self.domain}", "https://app.{self.domain}"]
cors_methods = ["GET", "POST"]
cors_headers = ["Content-Type", "Authorization"]

[jwt]
secret_key = "{self.jwt_secret}"
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
"""
        
        production_file = self.config_dir / "production.toml"
        with open(production_file, 'w') as f:
            f.write(production_config)
        
        if os.name != 'nt':  # Not Windows
            os.chmod(production_file, 0o600)
        
        success("Production configuration generated")

    def generate_development_config(self) -> None:
        """Generate development configuration."""
        success("Generating development configuration...")
        
        development_config = f"""[verus]
rpc_url = "http://verus-daemon-dev:27486"
rpc_user = "{self.rpc_user}"
rpc_password = "{self.rpc_password}"
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
secret_key = "{self.jwt_secret}"
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
"""
        
        development_file = self.config_dir / "development.toml"
        with open(development_file, 'w') as f:
            f.write(development_config)
        
        if os.name != 'nt':  # Not Windows
            os.chmod(development_file, 0o600)
        
        success("Development configuration generated")

    def generate_verus_config(self) -> None:
        """Generate Verus daemon configuration."""
        success("Generating Verus daemon configuration...")
        
        verus_config = f"""# Verus Daemon Configuration for Docker
# Auto-generated on: {datetime.now()}

# RPC Configuration
rpcuser={self.rpc_user}
rpcpassword={self.rpc_password}
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
"""
        
        verus_file = self.docker_config_dir / "verus.conf"
        with open(verus_file, 'w') as f:
            f.write(verus_config)
        
        if os.name != 'nt':  # Not Windows
            os.chmod(verus_file, 0o600)
        
        success("Verus daemon configuration generated")

    def generate_caddy_config(self) -> None:
        """Generate Caddy configuration."""
        success("Generating Caddy configuration...")
        
        caddy_config = f"""# Production Caddyfile for Verus RPC Server
# Auto-generated on: {datetime.now()}

{self.domain} {{
    # Automatic HTTPS with Let's Encrypt
    tls {self.email}
    
    # Rate limiting
    rate_limit {{
        zone api
        events 1000
        window 1m
    }}
    
    # Security headers
    header {{
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        Permissions-Policy "geolocation=(), microphone=(), camera=()"
        -Server
        Cache-Control "no-cache, no-store, must-revalidate"
        Pragma "no-cache"
        Expires "0"
    }}
    
    # Gzip compression
    encode gzip
    
    # Reverse proxy to Verus RPC Server
    reverse_proxy verus-rpc-server:8080 {{
        health_uri /health
        health_interval 30s
        health_timeout 10s
        lb_policy round_robin
        timeout 30s
        header_up X-Real-IP {{remote_host}}
        header_up X-Forwarded-For {{remote_host}}
        header_up X-Forwarded-Proto {{scheme}}
    }}
    
    # Logging
    log {{
        output file /var/log/caddy/verus-rpc.log
        format json
        level INFO
    }}
}}

# Health check endpoint (internal only)
:8081 {{
    bind 127.0.0.1
    reverse_proxy verus-rpc-server:8080 {{
        health_uri /health
        health_interval 30s
        health_timeout 10s
    }}
}}

# Metrics endpoint (internal only)
:8082 {{
    bind 127.0.0.1
    reverse_proxy verus-rpc-server:8080 {{
        health_uri /metrics
        health_interval 30s
        health_timeout 10s
    }}
}}
"""
        
        caddy_file = self.docker_config_dir / "Caddyfile"
        with open(caddy_file, 'w') as f:
            f.write(caddy_config)
        
        success("Caddy configuration generated")

    def generate_caddy_dev_config(self) -> None:
        """Generate development Caddy configuration."""
        success("Generating development Caddy configuration...")
        
        caddy_dev_config = f"""# Development Caddyfile for Verus RPC Server
# Auto-generated on: {datetime.now()}

:80 {{
    # Rate limiting (relaxed for development)
    rate_limit {{
        zone api
        events 2000
        window 1m
    }}
    
    # Security headers
    header {{
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        -Server
        Cache-Control "no-cache, no-store, must-revalidate"
    }}
    
    # Gzip compression
    encode gzip
    
    # Reverse proxy to development server
    reverse_proxy verus-rpc-server-dev:8080 {{
        health_uri /health
        health_interval 30s
        health_timeout 10s
        timeout 30s
        header_up X-Real-IP {{remote_host}}
        header_up X-Forwarded-For {{remote_host}}
        header_up X-Forwarded-Proto {{scheme}}
    }}
    
    # Logging
    log {{
        output stdout
        format console
        level DEBUG
    }}
}}

# Token service endpoint
:8081 {{
    reverse_proxy token-service-dev:8081 {{
        health_uri /health
        health_interval 30s
        health_timeout 10s
    }}
}}
"""
        
        caddy_dev_file = self.docker_config_dir / "Caddyfile.dev"
        with open(caddy_dev_file, 'w') as f:
            f.write(caddy_dev_config)
        
        success("Development Caddy configuration generated")

    def generate_prometheus_config(self) -> None:
        """Generate Prometheus configuration."""
        success("Generating Prometheus configuration...")
        
        prometheus_config = """global:
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
"""
        
        prometheus_file = self.docker_config_dir / "prometheus.yml"
        with open(prometheus_file, 'w') as f:
            f.write(prometheus_config)
        
        success("Prometheus configuration generated")

    def validate_config(self) -> None:
        """Validate generated configuration."""
        success("Validating configuration...")
        
        errors = 0
        
        # Check if .env file exists
        if not self.env_file.exists():
            error(f"Environment file not found: {self.env_file}")
            errors += 1
        
        # Check if configuration files exist
        config_files = ["production.toml", "development.toml"]
        for config_file in config_files:
            config_path = self.config_dir / config_file
            if not config_path.exists():
                error(f"Configuration file not found: {config_path}")
                errors += 1
        
        if errors == 0:
            success("Configuration validation passed")
        else:
            error(f"Configuration validation failed with {errors} errors")

    def show_summary(self) -> None:
        """Show configuration summary."""
        success("Configuration Summary")
        print("====================")
        print(f"Environment file: {self.env_file}")
        print(f"Config directory: {self.config_dir}")
        print(f"Backup directory: {self.backup_dir}")
        print()
        print("Generated files:")
        print("- .env (environment variables)")
        print("- config/production.toml")
        print("- config/development.toml")
        print("- docker/config/verus.conf")
        print("- docker/config/Caddyfile")
        print("- docker/config/Caddyfile.dev")
        print("- docker/config/prometheus.yml")
        print()
        print("Next steps:")
        print("1. Review and customize the generated configuration files")
        print(f"2. Update domain names in docker/config/Caddyfile (currently set to: {self.domain})")
        print("3. Set up SSL certificates (if needed)")
        print("4. Run: docker-compose -f docker/compose/docker-compose.yml up -d")
        print()
        print("Security notes:")
        print("- Keep .env file secure and do not commit to version control")
        print("- Change default passwords in production")
        print("- Review and customize security settings")

    def run(self) -> None:
        """Run the complete auto-configuration process."""
        print("==========================================")
        print("Verus RPC Server Auto-Configuration Script")
        print("==========================================")
        print()
        
        # Check prerequisites
        self.check_prerequisites()
        
        # Backup existing configuration
        self.backup_config()
        
        # Generate all configurations
        self.generate_env_vars()
        self.generate_production_config()
        self.generate_development_config()
        self.generate_verus_config()
        self.generate_caddy_config()
        self.generate_caddy_dev_config()
        self.generate_prometheus_config()
        
        # Validate configuration
        self.validate_config()
        
        # Show summary
        self.show_summary()
        
        success("Auto-configuration completed successfully!")

def main():
    parser = argparse.ArgumentParser(
        description="Auto-Configuration Script for Verus RPC Server Docker Deployment",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python auto-config.py
  python auto-config.py --domain "api.mydomain.com" --email "admin@mydomain.com"
  python auto-config.py --help
        """
    )
    
    parser.add_argument(
        "--domain",
        default="yourdomain.com",
        help="Domain name for Caddy configuration (default: yourdomain.com)"
    )
    
    parser.add_argument(
        "--email",
        default="your-email@domain.com",
        help="Email for Let's Encrypt certificates (default: your-email@domain.com)"
    )
    
    args = parser.parse_args()
    
    # Run auto-configuration
    config = AutoConfig(domain=args.domain, email=args.email)
    config.run()

if __name__ == "__main__":
    main()
