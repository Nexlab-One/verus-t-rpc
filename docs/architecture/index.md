# Architecture Documentation

System architecture and design documentation for the Rust Verus RPC Server.

## 📚 Available Documentation

### [System Architecture](system-architecture.md)
Overview of the system's clean architecture implementation including domain-driven design principles and layer separation.

### [Application Services](application-services.md)
Detailed documentation of the application services layer architecture, including the recent refactoring for improved modularity and maintainability.

## 🏗️ Architecture Overview

The Rust Verus RPC Server follows a **Clean Architecture** pattern with clear separation of concerns:

- **HTTP Layer**: Warp framework handling HTTP requests/responses with header extraction
- **Infrastructure Layer**: External integrations, caching, monitoring
- **Application Layer**: Use cases, business logic orchestration with modular services
- **Domain Layer**: Core business models and domain validation (`src/domain/validation`)

## 🔗 Quick Navigation

- **Getting Started**: See [../getting-started.md](../getting-started.md) for quick setup
- **API Reference**: See [../api/](api/) for complete API documentation
- **Security**: See [../security/](security/) for security architecture
- **Development**: See [../development/](development/) for development guidelines

## 📖 Related Documentation

- **Deployment**: [../deployment/](deployment/) - Production architecture considerations
- **Monitoring**: [../monitoring/](monitoring/) - Observability architecture
- **Configuration**: [../development/configuration-reference.md](development/configuration-reference.md) - Configuration architecture

## 🏛️ Layer Architecture

### HTTP Layer (Infrastructure)

- **Framework**: `warp`
- **Routes**: `src/infrastructure/http/routes/*` (`/`, `/health`, `/metrics`, `/metrics/prometheus`, `/pool/*`)
- **Handlers**: `src/infrastructure/http/handlers/*` with authentication token processing
- **Processors**: `src/infrastructure/http/processors/*` (validation, rate limit, cache, security)
- **Responses**: `src/infrastructure/http/responses.rs` (JSON-RPC 2.0)
- **Header Processing**: Authorization, User-Agent, and X-Forwarded-For header extraction
- **Tests**: in-memory `warp::test::request()` covering status, headers, and body

### Application Layer

- **Use Cases**: `src/application/use_cases.rs` - Business logic orchestration
- **Services**: `src/application/services/` - Modular application services
  - `rpc_service.rs` - Main RPC orchestration service
  - `metrics_service.rs` - Metrics collection service
  - `rpc/` - RPC-specific submodules
    - `token_extraction.rs` - HTTP header token extraction
    - `parameter_validation.rs` - Parameter validation logic
    - `method_registry.rs` - RPC method definitions and rules
- **Middleware**: `src/middleware/` - Request processing middleware
- **Authentication**: HTTP header-based JWT token validation

### Domain Layer

- **Entities**: `src/domain/` - Core business entities and value objects
- **Security**: `src/domain/security.rs` - Security context and validation
- **RPC**: `src/domain/rpc.rs` - RPC request/response models with authentication support

## 🔗 Related Documentation

- **System Architecture**: [system-architecture.md](system-architecture.md) - Overall system design
- **Application Services**: [application-services.md](application-services.md) - Detailed service architecture
- **Security**: [../security/](security/) - Security implementation and best practices
- **API**: [../api/](api/) - API documentation and examples