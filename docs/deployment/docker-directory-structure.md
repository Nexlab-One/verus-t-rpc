# Docker Directory Structure

This document explains the organization of Docker-related files in the Verus RPC Server project.

## 📁 Directory Organization

```
docker/
├── README.md                    # Docker implementation guide
├── Dockerfile                   # Production container definition
├── Dockerfile.dev              # Development container definition
├── .dockerignore               # Docker build exclusions
├── compose/
│   ├── docker-compose.yml      # Production services
│   └── docker-compose.dev.yml  # Development services
├── scripts/
│   ├── auto-config.py          # Cross-platform auto-config script
│   ├── auto-config.ps1         # Windows PowerShell auto-config script
│   ├── auto-config.sh          # Unix/Linux/macOS auto-config script
│   ├── docker-build.ps1        # Windows PowerShell build script
│   ├── docker-build.sh         # Unix/Linux/macOS build script
│   └── docker-security.sh      # Security validation script
└── config/
    ├── Caddyfile               # Production reverse proxy configuration
    ├── Caddyfile.dev          # Development reverse proxy configuration
    ├── prometheus.yml         # Monitoring configuration
    └── verus.conf             # Verus daemon configuration
```

## Usage

### Quick Start Commands

#### Windows Users
```powershell
# Auto-configure and start development environment
.\docker\scripts\auto-config.ps1
.\docker\scripts\docker-build.ps1 -Build -Run -Environment development

# Or use the Makefile
make auto-config-windows
make run-dev
```

#### Unix/Linux/macOS Users
```bash
# Auto-configure and start development environment
chmod +x docker/scripts/auto-config.sh
./docker/scripts/auto-config.sh
./docker/scripts/docker-build.sh -b -r -e development

# Or use the Makefile
make auto-config-unix
make run-dev
```

#### Cross-Platform (Python)
```bash
# Auto-configure using Python (works on all platforms)
python3 docker/scripts/auto-config.py
make run-dev
```

### Manual Docker Commands

#### Development Environment
```bash
# Build development images
docker-compose -f docker/compose/docker-compose.dev.yml build

# Start development services
docker-compose -f docker/compose/docker-compose.dev.yml up -d

# View logs
docker-compose -f docker/compose/docker-compose.dev.yml logs -f

# Stop development services
docker-compose -f docker/compose/docker-compose.dev.yml down
```

#### Production Environment
```bash
# Build production images
docker-compose -f docker/compose/docker-compose.yml build

# Start production services
docker-compose -f docker/compose/docker-compose.yml up -d

# View logs
docker-compose -f docker/compose/docker-compose.yml logs -f

# Stop production services
docker-compose -f docker/compose/docker-compose.yml down
```

## 🛠️ Makefile Integration

```makefile
# Production commands
build:
	docker-compose -f docker/compose/docker-compose.yml build

run:
	docker-compose -f docker/compose/docker-compose.yml up -d

# Development commands
build-dev:
	docker-compose -f docker/compose/docker-compose.dev.yml build

run-dev:
	docker-compose -f docker/compose/docker-compose.dev.yml up -d

# Auto-configuration commands
auto-config:
	python3 docker/scripts/auto-config.py

auto-config-windows:
	powershell -ExecutionPolicy Bypass -File docker/scripts/auto-config.ps1

auto-config-unix:
	./docker/scripts/auto-config.sh
```

## 🔧 Configuration Files

### Auto-Configuration Output

```
project-root/
├── .env                          # Environment variables (secure)
├── config/
│   ├── production.toml          # Production configuration
│   └── development.toml         # Development configuration
├── docker/
│   └── config/
│       ├── verus.conf           # Verus daemon configuration
│       ├── Caddyfile            # Production reverse proxy
│       ├── Caddyfile.dev        # Development reverse proxy
│       └── prometheus.yml       # Monitoring configuration
└── backups/                     # Configuration backups
```

### Environment Variables

The `.env` file is still created in the project root for easy access:

```bash
# Unix/Linux/macOS
export REDIS_PASSWORD="your-secure-12-char-password"
export JWT_SECRET_KEY="your-32-character-cryptographically-secure-key"

# Windows PowerShell
$env:REDIS_PASSWORD="your-secure-12-char-password"
$env:JWT_SECRET_KEY="your-32-character-cryptographically-secure-key"
```

## 🔒 Security Features

### Security Validation

```bash
# Unix/Linux/macOS
./docker/scripts/docker-security.sh validate
./docker/scripts/docker-security.sh build
./docker/scripts/docker-security.sh deploy
./docker/scripts/docker-security.sh monitor

# Windows (using WSL or Git Bash)
./docker/scripts/docker-security.sh validate
./docker/scripts/docker-security.sh build
./docker/scripts/docker-security.sh deploy
./docker/scripts/docker-security.sh monitor
```

### Security Checks

The security script now validates:
- Internal network configuration in `docker/compose/docker-compose.yml`
- Service isolation and port exposure
- Container security settings
- File permissions and access controls

## 📚 Documentation

Documentation

- **`docker/README.md`**: Complete Docker implementation guide
- **`docs/deployment/docker-directory-structure.md`**: This document
- **`docs/deployment/docker-security.md`**: Security considerations
- **`docs/deployment/production.md`**: Production deployment guide

### Help Commands

All scripts include built-in help:

```bash
# Python script
python3 docker/scripts/auto-config.py --help

# PowerShell script
.\docker\scripts\auto-config.ps1 -Help

# Bash script
./docker/scripts/auto-config.sh --help

# Build scripts
.\docker\scripts\docker-build.ps1 -Help
./docker/scripts/docker-build.sh -h
```


## 🤝 Contributing

When contributing to the Docker implementation:

1. **Follow the structure**: Place new Docker files in appropriate subdirectories
2. **Update scripts**: Ensure all scripts work with the new paths
3. **Update documentation**: Keep this guide and related docs current
4. **Test thoroughly**: Verify changes work on all supported platforms

## 📄 Related Documentation

- [Docker Implementation Guide](../docker/README.md)
- [Security Overview](../security/security-overview.md)
- [Production Deployment](production.md)
- [Docker Security](docker-security.md)
