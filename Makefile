# Makefile for Verus RPC Server Docker Operations
# Cross-platform compatible Docker management

.PHONY: help build build-dev run run-dev stop stop-dev clean clean-all logs logs-dev health security-validate security-build security-deploy security-monitor auto-config auto-config-windows auto-config-unix auto-config-python

# Default target
help:
	@echo "Verus RPC Server Docker Management"
	@echo ""
	@echo "Available commands:"
	@echo "  build          - Build production Docker images"
	@echo "  build-dev      - Build development Docker images"
	@echo "  run            - Start production containers"
	@echo "  run-dev        - Start development containers"
	@echo "  stop           - Stop production containers"
	@echo "  stop-dev       - Stop development containers"
	@echo "  clean          - Clean Docker resources"
	@echo "  clean-all      - Clean all Docker resources (including images)"
	@echo "  logs           - View production logs"
	@echo "  logs-dev       - View development logs"
	@echo "  health         - Check service health"
	@echo "  security-validate - Validate security configuration"
	@echo "  security-build - Build with security checks"
	@echo "  security-deploy - Deploy with security validation"
	@echo "  security-monitor - Monitor security status"
	@echo "  auto-config    - Auto-configure environment (cross-platform)"
	@echo "  auto-config-windows - Auto-configure using PowerShell"
	@echo "  auto-config-unix - Auto-configure using Bash"
	@echo "  auto-config-python - Auto-configure using Python"
	@echo ""

# Production commands
build:
	@echo "Building production Docker images..."
	docker-compose -f docker/compose/docker-compose.yml build

run:
	@echo "Starting production containers..."
	docker-compose -f docker/compose/docker-compose.yml up -d

stop:
	@echo "Stopping production containers..."
	docker-compose -f docker/compose/docker-compose.yml down

logs:
	@echo "Viewing production logs..."
	docker-compose -f docker/compose/docker-compose.yml logs -f

# Development commands
build-dev:
	@echo "Building development Docker images..."
	docker-compose -f docker/compose/docker-compose.dev.yml build

run-dev:
	@echo "Starting development containers..."
	docker-compose -f docker/compose/docker-compose.dev.yml up -d

stop-dev:
	@echo "Stopping development containers..."
	docker-compose -f docker/compose/docker-compose.dev.yml down

logs-dev:
	@echo "Viewing development logs..."
	docker-compose -f docker/compose/docker-compose.dev.yml logs -f

# Cleanup commands
clean:
	@echo "Cleaning Docker resources..."
	docker-compose -f docker/compose/docker-compose.yml down 2>/dev/null || true
	docker-compose -f docker/compose/docker-compose.dev.yml down 2>/dev/null || true
	docker system prune -f

clean-all:
	@echo "Cleaning all Docker resources..."
	docker-compose -f docker/compose/docker-compose.yml down 2>/dev/null || true
	docker-compose -f docker/compose/docker-compose.dev.yml down 2>/dev/null || true
	docker system prune -af
	docker volume prune -f
	docker network prune -f

# Health check
health:
	@echo "Checking service health..."
	@echo "RPC Server:"
	@curl -f -s http://localhost:8080/health || echo "RPC Server health check failed"
	@echo "Token Service:"
	@curl -f -s http://localhost:8081/health || echo "Token Service health check failed"

# Security commands
security-validate:
	@echo "Validating security configuration..."
	@if [ -f "docker/scripts/docker-security.sh" ]; then \
		chmod +x docker/scripts/docker-security.sh; \
		./docker/scripts/docker-security.sh validate; \
	else \
		echo "Security script not found"; \
	fi

security-build:
	@echo "Building with security checks..."
	@if [ -f "docker/scripts/docker-security.sh" ]; then \
		chmod +x docker/scripts/docker-security.sh; \
		./docker/scripts/docker-security.sh build; \
	else \
		echo "Security script not found"; \
	fi

security-deploy:
	@echo "Deploying with security validation..."
	@if [ -f "docker/scripts/docker-security.sh" ]; then \
		chmod +x docker/scripts/docker-security.sh; \
		./docker/scripts/docker-security.sh deploy; \
	else \
		echo "Security script not found"; \
	fi

security-monitor:
	@echo "Monitoring security status..."
	@if [ -f "docker/scripts/docker-security.sh" ]; then \
		chmod +x docker/scripts/docker-security.sh; \
		./docker/scripts/docker-security.sh monitor; \
	else \
		echo "Security script not found"; \
	fi

# Auto-configuration commands
auto-config:
	@echo "Auto-configuring environment (cross-platform)..."
	@if command -v python3 >/dev/null 2>&1; then \
		echo "Using Python auto-config script..."; \
		python3 docker/scripts/auto-config.py; \
	elif command -v powershell >/dev/null 2>&1; then \
		echo "Using PowerShell auto-config script..."; \
		powershell -ExecutionPolicy Bypass -File docker/scripts/auto-config.ps1; \
	elif [ -f "docker/scripts/auto-config.sh" ]; then \
		echo "Using Bash auto-config script..."; \
		chmod +x docker/scripts/auto-config.sh; \
		./docker/scripts/auto-config.sh; \
	else \
		echo "No auto-config script found"; \
	fi

auto-config-windows:
	@echo "Auto-configuring using PowerShell..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File docker/scripts/auto-config.ps1; \
	else \
		echo "PowerShell not available"; \
	fi

auto-config-unix:
	@echo "Auto-configuring using Bash..."
	@if [ -f "docker/scripts/auto-config.sh" ]; then \
		chmod +x docker/scripts/auto-config.sh; \
		./docker/scripts/auto-config.sh; \
	else \
		echo "Bash auto-config script not found"; \
	fi

auto-config-python:
	@echo "Auto-configuring using Python..."
	@if command -v python3 >/dev/null 2>&1; then \
		python3 docker/scripts/auto-config.py; \
	else \
		echo "Python3 not available"; \
	fi

# Cross-platform build scripts
build-windows:
	@echo "Building using Windows PowerShell script..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File docker/scripts/docker-build.ps1 -Build -Environment production; \
	else \
		echo "PowerShell not available, using Docker Compose directly"; \
		docker-compose -f docker/compose/docker-compose.yml build; \
	fi

build-unix:
	@echo "Building using Unix shell script..."
	@if [ -f "docker/scripts/docker-build.sh" ]; then \
		chmod +x docker/scripts/docker-build.sh; \
		./docker/scripts/docker-build.sh -b -e production; \
	else \
		echo "Unix script not found, using Docker Compose directly"; \
		docker-compose -f docker/compose/docker-compose.yml build; \
	fi

run-windows:
	@echo "Running using Windows PowerShell script..."
	@if command -v powershell >/dev/null 2>&1; then \
		powershell -ExecutionPolicy Bypass -File docker/scripts/docker-build.ps1 -Run -Environment production; \
	else \
		echo "PowerShell not available, using Docker Compose directly"; \
		docker-compose -f docker/compose/docker-compose.yml up -d; \
	fi

run-unix:
	@echo "Running using Unix shell script..."
	@if [ -f "docker/scripts/docker-build.sh" ]; then \
		chmod +x docker/scripts/docker-build.sh; \
		./docker/scripts/docker-build.sh -r -e production; \
	else \
		echo "Unix script not found, using Docker Compose directly"; \
		docker-compose -f docker/compose/docker-compose.yml up -d; \
	fi

# Development with live reload
dev:
	@echo "Starting development environment with live reload..."
	docker-compose -f docker/compose/docker-compose.dev.yml up

# Production with monitoring
prod:
	@echo "Starting production environment with monitoring..."
	docker-compose -f docker/compose/docker-compose.yml up -d
	@echo "Waiting for services to be ready..."
	@sleep 30
	@make health

# Quick setup for new users
setup:
	@echo "Setting up Verus RPC Server Docker environment..."
	@echo "1. Creating configuration directories..."
	@mkdir -p config logs
	@echo "2. Auto-configuring environment..."
	@make auto-config
	@echo "3. Building development images..."
	@make build-dev
	@echo "Setup complete! Run 'make run-dev' to start development environment"

# Status check
status:
	@echo "Docker container status:"
	@docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep verus || echo "No Verus containers running"
	@echo ""
	@echo "Docker volumes:"
	@docker volume ls | grep verus || echo "No Verus volumes found"
	@echo ""
	@echo "Docker networks:"
	@docker network ls | grep verus || echo "No Verus networks found"

# Backup and restore
backup:
	@echo "Creating backup..."
	@mkdir -p backups
	@tar -czf backups/config-backup-$(shell date +%Y%m%d-%H%M%S).tar.gz config/ 2>/dev/null || echo "No config directory to backup"
	@docker run --rm -v verus-data:/data -v $(PWD)/backups:/backup alpine tar czf /backup/verus-data-backup-$(shell date +%Y%m%d-%H%M%S).tar.gz -C /data . 2>/dev/null || echo "No verus-data volume to backup"
	@echo "Backup completed in backups/ directory"

restore:
	@echo "Available backups:"
	@ls -la backups/ 2>/dev/null || echo "No backups found"
	@echo "To restore, manually extract the backup files to their respective locations"

# Update images
update:
	@echo "Updating Docker images..."
	docker-compose -f docker/compose/docker-compose.yml pull
	docker-compose -f docker/compose/docker-compose.dev.yml pull
	@echo "Images updated. Run 'make build' or 'make build-dev' to rebuild with latest changes"
