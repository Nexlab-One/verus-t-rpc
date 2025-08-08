# System Architecture

Overview of the Verus RPC Server's system architecture, design principles, and component interactions.

## 🏗️ Architecture Overview

The Verus RPC Server follows **Clean Architecture** principles with a **layered design** that promotes separation of concerns, testability, and maintainability.

```
┌───────────────────────────────────────────────────────────────────┐
│                        HTTP Layer                                 │
│                    (Warp Framework)                               │
│  Headers: Authorization, User-Agent, X-Forwarded-For              │
├───────────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                           │
│  HTTP Server | Cache Adapter | Monitoring | External Services     │
├───────────────────────────────────────────────────────────────────┤
│                    Application Layer                              │
│  Use Cases | Services | Middleware | Validation | Authentication  │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              Application Services                            │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │ │
│  │  │ RPC Service │ │Metrics Svc  │ │     RPC Submodules      │ │ │
│  │  │             │ │             │ │ ┌─────────┐ ┌─────────┐ │ │ │
│  │  │ • Process   │ │ • Collect   │ │ │ Token   │ │ Param   │ │ │ │
│  │  │   Requests  │ │   Metrics   │ │ │ Extract │ │ Valid   │ │ │ │
│  │  │ • Auth      │ │ • Monitor   │ │ └─────────┘ └─────────┘ │ │ │
│  │  │   Validation│ │   Health    │ │ ┌───────────┐           │ │ │
│  │  │ • Security  │ │ • Report    │ │ │ Domain    │           │ │ │
│  │  │   Context   │ │   Status    │ │ │Validation │           │ │ │
│  │  │             │ │             │ │ └───────────┘           │ │ │
│  │  └─────────────┘ └─────────────┘ └─────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────────┤
│                      Domain Layer                                 │
│  Entities | Value Objects | Business Rules | Domain Services      │
└───────────────────────────────────────────────────────────────────┘
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
- **Header Processing**: Extract and validate `Authorization`, `User-Agent`, and `X-Forwarded-For` headers

**Key Files**:
- `src/infrastructure/http/server.rs` - Main HTTP server
- `src/infrastructure/http/routes/*` - Route definitions with header extraction
- `src/infrastructure/http/handlers/*` - Request handlers with auth token processing
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
- **Services**: Cross-cutting concerns with modular design
- **Middleware**: Request/response processing
- **Validation**: Input validation and sanitization
- **Authentication**: HTTP header-based JWT token validation

**Key Files**:
- `src/application/use_cases/` - Business use cases
- `src/application/services/` - Modular application services
- `src/middleware/` - Request processing middleware
- `src/domain/validation/` - Domain method registry and parameter validation

#### Application Services Architecture

The application services layer has been refactored for better modularity and maintainability:

```
src/application/services/
├── mod.rs                    # Public API exports
├── rpc_service.rs            # Main RPC orchestration service
├── metrics_service.rs        # Metrics collection service
└── rpc/                      # RPC-specific submodules
    ├── mod.rs                # RPC module exports
    ├── token_extraction.rs   # HTTP header token extraction
    ├── parameter_validation.rs # Parameter validation logic
    └── method_registry.rs    # RPC method definitions and rules
```

**Service Responsibilities**:

- **RpcService**: Main orchestrator for RPC requests
  - Processes authentication tokens from HTTP headers
  - Validates security context and permissions
  - Coordinates with external RPC adapters
  - Manages request lifecycle

- **MetricsService**: Application metrics collection
  - Records request success/failure rates
  - Tracks performance metrics
  - Provides health status information

- **RPC Submodules**: Specialized functionality
  - **Token Extraction**: HTTP header-based authentication
  - **Parameter Validation**: Input sanitization and validation
  - **Method Registry**: RPC method definitions and security rules

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
1. HTTP Request (with Authorization header)
   ↓
2. Warp Router (extracts headers)
   ↓
3. Middleware Chain
   ├─ Rate Limiting
   ├─ Header Validation
   ├─ Security Headers
   └─ Caching
   ↓
4. Request Handler (processes auth token)
   ↓
5. Use Case Execution
   ↓
6. Application Services
   ├─ RPC Service (validates auth, creates security context)
   ├─ Token Extraction (from HTTP headers)
   ├─ Parameter Validation (domain validation)
   └─ Method Registry (domain layer)
   ↓
7. Domain Logic
   ↓
8. Infrastructure Adapters
   ├─ RPC Adapter (Verus daemon)
   └─ Cache Adapter (Redis)
   ↓
9. Response Formatting
   ↓
10. HTTP Response
```

### Authentication Flow

```
1. HTTP Request with Authorization Header
   ↓
2. Header Extraction in Route
   ├─ Authorization: Bearer <token>
   ├─ User-Agent: <client_info>
   └─ X-Forwarded-For: <client_ip>
   ↓
3. RequestContext Creation
   ├─ client_ip: validated IP address
   ├─ user_agent: from User-Agent header
   ├─ auth_token: from Authorization header
   └─ timestamp: request timestamp
   ↓
4. Domain Model Conversion
   ├─ ClientInfo with auth_token
   └─ RpcRequest with client context
   ↓
5. RpcService Processing
   ├─ Extract auth_token from client_info
   ├─ Validate token with AuthenticationAdapter
   ├─ Create SecurityContext with permissions
   └─ Validate request against security policy
   ↓
6. Security Validation
   ├─ Check authentication requirements
   ├─ Validate user permissions
   ├─ Apply method-specific rules
   └─ Enforce IP restrictions
```

### Detailed Flow Example

#### 1. **Request Reception with Headers**
```rust
// src/infrastructure/http/handlers/rpc.rs
pub async fn handle_rpc_request(
    request: JsonRpcRequest,
    client_ip: String,
    auth_header: Option<String>,        // Authorization header
    user_agent_header: Option<String>,  // User-Agent header
    rpc_use_case: Arc<ProcessRpcRequestUseCase>,
    config: AppConfig,
    cache_middleware: Arc<CacheMiddleware>,
    rate_limit_middleware: Arc<RateLimitMiddleware>,
) -> Result<impl Reply, warp::reject::Rejection> {
    // Create RequestContext with auth token
    let mut context = RequestContext::new(
        validated_client_ip.clone(),
        request.method.clone(),
        request.params.clone(),
    );
    if let Some(ua) = user_agent_header {
        context = context.with_user_agent(ua);
    }
    if let Some(auth) = auth_header {
        context = context.with_auth_token(auth);
    }
    // Process request...
}
```

#### 2. **Application Service Processing**
```rust
// src/application/services/rpc_service.rs
pub async fn process_request(&self, request: RpcRequest) -> AppResult<RpcResponse> {
    // Extract auth token from HTTP headers (now in client_info)
    let auth_token: Option<String> = request.client_info.auth_token.clone();
    
    // Validate token and get user permissions
    let user_permissions = if let Some(token) = &auth_token {
        match self.auth_adapter.validate_token(token).await {
            Ok(permissions) => permissions,
            Err(e) => {
                warn!("Authentication failed: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    };

    // Create security context with auth token
    let security_context = SecurityContext {
        client_ip: request.client_info.ip_address.clone(),
        user_agent: request.client_info.user_agent.clone(),
        auth_token,
        user_permissions,
        timestamp: request.client_info.timestamp,
        request_id: request.client_info.timestamp.timestamp_millis().to_string(),
        development_mode: self._config.security.development_mode,
    };

    // Validate request against security policy
    self.security_validator.validate_request(&request.method, &security_context)?;
    // Continue processing...
}
```

#### 3. **Use Case Execution**
```rust
// src/application/use_cases.rs
pub async fn execute(
    &self,
    request: RpcRequest,  // Now includes auth_token in client_info
) -> Result<RpcResponse, AppError> {
    // Business logic with domain services
    let result = self.rpc_service.process_request(request).await;
    
    // Record metrics
    self.metrics_service.record_request(result.is_ok());
    
    result
}
```

#### 4. **Domain Logic**
```rust
// src/application/services/rpc_service.rs
pub struct RpcService {
    _config: Arc<AppConfig>,
    security_validator: Arc<SecurityValidator>,
    external_rpc_adapter: Arc<crate::infrastructure::adapters::ExternalRpcAdapter>,
    auth_adapter: Arc<crate::infrastructure::adapters::AuthenticationAdapter>,
    comprehensive_validator: Arc<ComprehensiveValidator>,
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
    // Route with middleware chain and header extraction
    warp::path::end()
        .and(warp::post())
        .and(warp::body::content_length_limit(config.server.max_request_size as u64))
        .and(warp::body::json())
        .and(warp::header::<String>("x-forwarded-for"))
        .and(warp::header::optional::<String>("authorization"))
        .and(warp::header::optional::<String>("user-agent"))
        .and(with_rpc_use_case(rpc_use_case))
        .and(with_config(config))
        .and(with_cache_middleware(cache_middleware))
        .and(with_rate_limit_middleware(rate_limit_middleware))
        .and_then(handle_rpc_request)
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
3. **Authentication**: HTTP header-based JWT token validation
4. **Input Validation**: Parameter sanitization
5. **Method Allowlist**: Restricted RPC method access

### Security Flow

```
Request → Header Extraction → Rate Limiting → Authentication → Validation → Processing → Security Headers → Response
```

### Authentication Flow

```
Authorization Header → RequestContext → ClientInfo → SecurityContext → Token Validation → Permission Check → Request Processing
```

## 📊 Monitoring & Observability

### Metrics Collection

- **Request Metrics**: Count, duration, success/failure rates
- **Performance Metrics**: Response times, throughput
- **Error Metrics**: Error types and frequencies
- **Business Metrics**: Method usage, cache hit rates
- **Authentication Metrics**: Token validation success/failure rates

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

[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"
```

### Configuration Validation

- **Schema Validation**: TOML schema validation
- **Environment Validation**: Runtime environment checks
- **Security Validation**: Security configuration validation

## 🔗 Related Documentation

- [Application Services](./application-services.md) - Detailed application services architecture
- [Clean Architecture](./clean-architecture.md) - Detailed clean architecture principles
- [Component Design](./component-design.md) - Individual component documentation
- [Data Flow](./data-flow.md) - Detailed data flow diagrams
- [Security Architecture](../security/security-overview.md) - Security design and implementation
