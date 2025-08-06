# Architecture Documentation

System architecture and design documentation for the Rust Verus RPC Server.

## 📚 Available Documentation

### [System Architecture](system-architecture.md)
Overview of the system's clean architecture implementation including domain-driven design principles and layer separation.

## 🏗️ Architecture Overview

The Rust Verus RPC Server follows a **Clean Architecture** pattern with clear separation of concerns:

- **HTTP Layer**: Warp framework handling HTTP requests/responses
- **Infrastructure Layer**: External integrations, caching, monitoring
- **Application Layer**: Use cases, business logic orchestration
- **Domain Layer**: Core business models and validation

## 🔗 Quick Navigation

- **Getting Started**: See [../getting-started.md](../getting-started.md) for quick setup
- **API Reference**: See [../api/](api/) for complete API documentation
- **Security**: See [../security/](security/) for security architecture
- **Development**: See [../development/](development/) for development guidelines

## 📖 Related Documentation

- **Deployment**: [../deployment/](deployment/) - Production architecture considerations
- **Monitoring**: [../monitoring/](monitoring/) - Observability architecture
- **Configuration**: [../development/configuration-reference.md](development/configuration-reference.md) - Configuration architecture
