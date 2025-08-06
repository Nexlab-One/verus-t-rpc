# Security Documentation

Security documentation for the Rust Verus RPC Server.

## 📚 Available Documentation

### [Security Overview](security-overview.md)
Security architecture overview including JWT authentication, rate limiting, security headers, and method validation.

## 🛡️ Security Features

The Rust Verus RPC Server implements security features:

- **JWT Authentication**: Token-based authentication with expiration
- **Rate Limiting**: IP-based request throttling
- **Security Headers**: CSP, XSS protection, clickjacking prevention
- **Method Validation**: Only pre-approved methods allowed
- **Input Validation**: Strict parameter type checking

## 🔗 Quick Navigation

- **Getting Started**: See [../getting-started.md](../getting-started.md) for quick setup
- **API Reference**: See [../api/](api/) for API security considerations
- **Architecture**: See [../architecture/](architecture/) for security architecture
- **Deployment**: See [../deployment/](deployment/) for secure deployment

## 📖 Related Documentation

- **Development**: [../development/](development/) - Security development guidelines
- **Configuration**: [../development/configuration-reference.md](development/configuration-reference.md) - Security configuration options
