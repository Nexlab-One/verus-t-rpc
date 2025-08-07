# System Architecture

Overview of the Verus RPC Server's system architecture, design principles, and component interactions.

## 🏗️ Architecture Overview

The Verus RPC Server follows **Clean Architecture** principles with a **layered design** that promotes separation of concerns, testability, and maintainability.

```
┌─────────────────────────────────────────────────────────────────┐
│                        HTTP Layer                               │
│                    (Warp Framework)                             │
├─────────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                         │
│  HTTP Server | Cache Adapter | Monitoring | External Services   │
├─────────────────────────────────────────────────────────────────┤
│                    Application Layer                            │
│  Use Cases | Services | Middleware | Validation | Authentication│
├─────────────────────────────────────────────────────────────────┤
│                      Domain Layer                               │
│  Entities | Value Objects | Business Rules | Domain Services    │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 Design Principles

### 1. **Clean Architecture**
- **Dependency Rule**: Dependencies point inward
- **Separation of Concerns**: Each layer has a specific responsibility
- **Testability**: Easy to unit test each component
- **Independence**: Framework and external dependencies are isolated

### 2. **Domain-Driven Design (DDD)**
- **Ubiquitous Language**: Consistent terminology across codebase
- **Bounded Contexts**: Clear boundaries between different domains
- **Value Objects**: Immutable objects representing domain concepts
- **Entities**: Objects with identity and lifecycle

### 3. **SOLID Principles**
- **Single Responsibility**: Each class has one reason to change
- **Open/Closed**: Open for extension, closed for modification
- **Liskov Substitution**: Subtypes are substitutable
- **Interface Segregation**: Clients depend only on interfaces they use
- **Dependency Inversion**: High-level modules don't depend on low-level modules

## 🏛️ Layer Architecture

### HTTP Layer (Infrastructure)

**Purpose**: Handle HTTP requests and responses using the Warp framework.

**Components**:
- **HTTP Server**: Warp-based async server with in-memory testing
- **Request Routing**: Route requests to appropriate handlers (`/`, `/health`, `/metrics`, `/metrics/prometheus`, `/pool/*`)
- **Response Formatting**: Format responses as JSON-RPC 2.0 with security headers
- **Middleware Integration**: Apply security, rate limiting, caching, validation

**Key Files**:
- `src/infrastructure/http/server.rs` - Main HTTP server
- `src/infrastructure/http/routes/*` - Route definitions
- `src/infrastructure/http/handlers/*` - Request handlers
- `src/infrastructure/http/processors/*` - Request processing (validation, rate limit, cache)
- `src/infrastructure/http/responses.rs` - Response formatting
- `src/infrastructure/http/utils.rs` - HTTP utilities

### Infrastructure Layer

**Purpose**: Provide external interfaces and adapters.

**Components**:
- **RPC Adapter**: Communicate with Verus daemon
- **Cache Adapter**: Redis-based caching
- **Monitoring**: Prometheus metrics and logging
- **Configuration**: Environment-based configuration

**Key Files**:
- `src/infrastructure/adapters/rpc.rs` - Verus daemon communication
- `src/infrastructure/adapters/cache.rs` - Redis caching
- `src/infrastructure/monitoring/` - Metrics and logging
- `src/config/` - Configuration management

### Application Layer

**Purpose**: Orchestrate use cases and coordinate between layers.

**Components**:
- **Use Cases**: Application-specific business logic
- **Services**: Cross-cutting concerns
- **Middleware**: Request/response processing
- **Validation**: Input validation and sanitization
- **Authentication**: JWT token validation

**Key Files**:
- `src/application/use_cases/` - Business use cases
- `src/application/services/` - Application services
- `src/middleware/` - Request processing middleware
- `src/application/validation/` - Input validation

### Domain Layer

**Purpose**: Core business logic and domain models.

**Components**:
- **Entities**: Core business objects
- **Value Objects**: Immutable domain concepts
- **Domain Services**: Business logic that doesn't belong to entities
- **Repositories**: Data access abstractions

**Key Files**:
- `src/domain/entities/` - Core business entities
- `src/domain/value_objects/` - Domain value objects
- `src/domain/services/` - Domain services
- `src/domain/repositories/` - Repository interfaces

## 🔄 Data Flow

### Request Processing Flow

```
1. HTTP Request
   ↓
2. Warp Router
   ↓
3. Middleware Chain
   ├─ Rate Limiting
   ├─ Authentication
   ├─ Security Headers
   └─ Caching
   ↓
4. Request Handler
   ↓
5. Use Case Execution
   ↓
6. Domain Logic
   ↓
7. Infrastructure Adapters
   ├─ RPC Adapter (Verus daemon)
   └─ Cache Adapter (Redis)
   ↓
8. Response Formatting
   ↓
9. HTTP Response
```

### Detailed Flow Example

#### 1. **Request Reception**
```rust
// src/infrastructure/http/handlers/rpc.rs
pub async fn handle_rpc_request(
    request: JsonRpcRequest,
    client_ip: String,
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
    config: AppConfig,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> Result<impl Reply, warp::reject::Rejection> {
    // Request processing with middleware chain
}
```

#### 2. **Middleware Processing**
```rust
// src/infrastructure/http/processors/base.rs
pub async fn check_rate_limit(
    client_ip: &str,
    context: &RequestContext,
    request: &JsonRpcRequest,
    rate_limit_middleware: &Arc<RateLimitMiddleware>,
    config: &AppConfig,
) -> Result<(), warp::reply::WithStatus<Box<dyn warp::Reply>>> {
    // Rate limiting validation
}
```

#### 3. **Use Case Execution**
```rust
// src/application/use_cases.rs
pub async fn execute(
    &self,
    request: RpcRequest,
) -> Result<RpcResponse, AppError> {
    // Business logic with domain services
}
```

#### 4. **Domain Logic**
```rust
// src/application/services.rs
pub struct RpcService {
    config: Arc<AppConfig>,
    security_validator: Arc<SecurityValidator>,
}
```

#### 5. **Infrastructure Interaction**
```rust
// src/infrastructure/adapters/external_rpc.rs
pub async fn call_method(
    &self,
    method: &str,
    params: &[serde_json::Value],
) -> Result<serde_json::Value, AppError> {
    // Verus daemon RPC call
}
```

## 🔧 Component Design

### Dependency Injection

The system uses dependency injection to manage component dependencies:

```rust
// Route creation with dependencies
pub fn create_rpc_route(
    config: AppConfig,
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // Route with middleware chain
}
```

### Error Handling

Comprehensive error handling across all layers:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("RPC error: {0}")]
    RpcError(#[from] RpcError),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    // ... other error types
}
```

### Configuration Management

Environment-based configuration with validation:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub verus: VerusConfig,
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub jwt: JwtConfig,
    pub rate_limit: RateLimitConfig,
    pub cache: CacheConfig,
    pub logging: LoggingConfig,
}
```

## 🛡️ Security Architecture

### Multi-Layer Security

1. **HTTP Security Headers**: Applied at response level
2. **Rate Limiting**: IP-based request throttling
3. **Authentication**: JWT token validation
4. **Input Validation**: Parameter sanitization
5. **Method Allowlist**: Restricted RPC method access

### Security Flow

```
Request → Rate Limiting → Authentication → Validation → Processing → Security Headers → Response
```

## 📊 Monitoring & Observability

### Metrics Collection

- **Request Metrics**: Count, duration, success/failure rates
- **Performance Metrics**: Response times, throughput
- **Error Metrics**: Error types and frequencies
- **Business Metrics**: Method usage, cache hit rates

### Logging Strategy

- **Structured Logging**: JSON format for easy parsing
- **Request/Response Logging**: Full request lifecycle
- **Security Event Logging**: Authentication and authorization events
- **Performance Logging**: Slow request identification

## 🔄 Caching Strategy

### Multi-Level Caching

1. **Application Cache**: In-memory caching for frequently accessed data
2. **Redis Cache**: Distributed caching for shared data
3. **Response Caching**: Cache entire RPC responses
4. **Method-Specific Caching**: Different TTL for different methods

### Cache Invalidation

- **Time-Based**: Automatic expiration
- **Event-Based**: Invalidation on specific events
- **Manual**: Explicit cache clearing

## 🚀 Performance Considerations

### Async Processing

- **Non-blocking I/O**: All operations are async
- **Connection Pooling**: Reuse connections to external services
- **Request Batching**: Batch multiple requests when possible
- **Response Streaming**: Stream large responses

### Resource Management

- **Memory Management**: Efficient memory usage with Rust's ownership system
- **Connection Limits**: Limit concurrent connections
- **Request Timeouts**: Prevent hanging requests
- **Graceful Shutdown**: Proper cleanup on shutdown

## 🔗 External Dependencies

### Verus Daemon

- **RPC Communication**: JSON-RPC 2.0 protocol
- **Connection Management**: Persistent connections with retry logic
- **Error Handling**: Comprehensive error handling and recovery

### Redis

- **Caching**: Response caching and session storage
- **Connection Pooling**: Efficient connection management
- **Failover**: Graceful handling of Redis failures

### Monitoring Stack

- **Prometheus**: Metrics collection and storage
- **Structured Logging**: JSON-formatted logs
- **Health Checks**: Service health monitoring

## 📈 Scalability Considerations

### Horizontal Scaling

- **Stateless Design**: No server-side state
- **Load Balancing**: Ready for load balancer deployment
- **Shared Caching**: Redis-based shared state
- **Containerization**: Docker-ready deployment

### Vertical Scaling

- **Async Processing**: Efficient resource utilization
- **Connection Pooling**: Optimized external connections
- **Memory Management**: Rust's memory safety and efficiency
- **CPU Optimization**: Multi-threaded processing

## 🔧 Configuration Management

### Environment-Based Configuration

```toml
[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_username"
rpc_password = "your_password"

[server]
bind_address = "127.0.0.1"
port = 8080

[security]
development_mode = false
enable_security_headers = true
```

### Configuration Validation

- **Schema Validation**: TOML schema validation
- **Environment Validation**: Runtime environment checks
- **Security Validation**: Security configuration validation

## 🔗 Related Documentation

- [Clean Architecture](./clean-architecture.md) - Detailed clean architecture principles
- [Component Design](./component-design.md) - Individual component documentation
- [Data Flow](./data-flow.md) - Detailed data flow diagrams
- [Security Architecture](../security/security-overview.md) - Security design and implementation
