# Application Services Architecture

Documentation of the application services layer architecture, including the recent refactoring for improved modularity and maintainability.

## Overview

The application services layer orchestrates business logic and coordinates between the domain layer and infrastructure adapters. It has been refactored to follow clean architecture principles with clear separation of concerns and improved testability.

## Module Structure

```
src/application/services/
├── mod.rs                    # Public API exports
├── rpc_service.rs            # Main RPC orchestration service
├── metrics_service.rs        # Metrics collection service
└── rpc/                      # RPC-specific submodules
    ├── mod.rs                # RPC module exports
    ├── token_extraction.rs   # HTTP header token extraction
    └── parameter_validation.rs # Parameter validation logic

# Domain validation (method registry, rules, and definitions) lives under:
src/domain/validation/
├── mod.rs
├── types.rs                  # RpcMethodDefinition, ParameterValidationRule, ParameterType, ValidationConstraint, SecurityLevel
├── registry.rs               # MethodRegistry (lookup and parameter validation)
├── domain_validator.rs       # DomainValidator facade
└── methods/                  # Modular method registrations (core, blocks, transactions, write, identity, currency, utility, additional)
```

## Service Components

### RpcService

**Purpose**: Main orchestrator for RPC request processing and security validation.

**Location**: `src/application/services/rpc_service.rs`

**Responsibilities**:
- Process RPC requests with authentication and authorization
- Coordinate with external RPC adapters
- Manage request lifecycle and error handling
- Create and validate security contexts
- Orchestrate parameter validation and method registry lookups

**Key Methods**:
```rust
impl RpcService {
    /// Standard constructor with internal dependency creation
    pub fn new(config: Arc<AppConfig>, security_validator: Arc<SecurityValidator>) -> Self
    
    /// Constructor for dependency injection (testing)
    pub fn new_with_dependencies(
        config: Arc<AppConfig>,
        security_validator: Arc<SecurityValidator>,
        external_rpc_adapter: Arc<ExternalRpcAdapter>,
        auth_adapter: Arc<AuthenticationAdapter>,
        comprehensive_validator: Arc<ComprehensiveValidator>,
    ) -> Self
    
    /// Main request processing method
    pub async fn process_request(&self, request: RpcRequest) -> AppResult<RpcResponse>
    
    /// Parameter validation orchestration
    pub fn validate_method_parameters(&self, method: &str, parameters: &Value) -> AppResult<()>
}
```

**Dependencies**:
- `AppConfig`: Application configuration
- `SecurityValidator`: Security policy validation
- `ExternalRpcAdapter`: Verus daemon communication
- `AuthenticationAdapter`: JWT token validation
- `ComprehensiveValidator`: Input validation

### MetricsService

**Purpose**: Application metrics collection and health monitoring.

**Location**: `src/application/services/metrics_service.rs`

**Responsibilities**:
- Record request success/failure rates
- Track performance metrics
- Provide health status information
- Monitor application behavior

**Key Methods**:
```rust
impl MetricsService {
    /// Record request outcome
    pub fn record_request(&self, success: bool)
    
    /// Get current metrics
    pub fn get_metrics(&self) -> MetricsData
    
    /// Check service health
    pub fn is_healthy(&self) -> bool
}
```

## RPC Submodules

### Token Extraction

**Purpose**: Extract and validate authentication tokens from HTTP headers.

**Location**: `src/application/services/rpc/token_extraction.rs`

**Functionality**:
- Extract Bearer tokens from HTTP Authorization headers
- Validate token format and structure
- Support for different authentication schemes

**Key Functions**:
```rust
/// Extract Bearer token from RPC request
pub fn extract_bearer_token_from_request(request: &RpcRequest) -> Option<String>
```

### Parameter Validation

**Purpose**: Validate RPC method parameters against defined rules and constraints.

**Location**: `src/application/services/rpc/parameter_validation.rs`

**Functionality**:
- Validate parameter types and formats
- Apply length constraints (min/max)
- Pattern matching validation
- Custom validation rules
- Support for both array and object parameter formats

**Key Functions**:
```rust
/// Validate parameter against defined rules
pub fn validate_parameter_rule(rule: &ParameterRule, parameters: &Value) -> AppResult<()>

/// Validate parameter value against constraints
pub fn validate_parameter_value(rule: &ParameterRule, value: &Value) -> AppResult<()>
```

**Supported Constraints**:
- `MinLength`: Minimum string length
- `MaxLength`: Maximum string length
- `Pattern`: Regex pattern matching
- `MinValue`: Minimum numeric value
- `MaxValue`: Maximum numeric value
- `Custom`: Custom validation rules

### Domain Validation (Method Registry)

**Purpose**: Define and manage RPC method definitions, security rules, and validation requirements.

**Location**: `src/domain/validation/registry.rs`

**Functionality**:
- Centralized RPC method definitions (category modules under `src/domain/validation/methods/*`)
- Security levels and permission requirements
- Parameter validation rules and constraints
- Method metadata and enabled/disabled flags

**Key Functions**:
```rust
// MethodRegistry
pub fn get_method(&self, name: &str) -> Option<&RpcMethodDefinition>
pub fn is_method_allowed(&self, name: &str) -> bool
pub fn validate_method_parameters(&self, method_name: &str, params: &[Box<RawValue>]) -> AppResult<()>

// DomainValidator facade
pub fn validate_method_call(&self, method: &str, params: &Option<Value>) -> AppResult<()>
```

**Method Definition Structure**:
```rust
pub struct RpcMethodDefinition {
    pub name: String,
    pub description: String,
    pub read_only: bool,
    pub required_permissions: Vec<String>,
    pub parameter_rules: Vec<ParameterValidationRule>,
    pub security_level: SecurityLevel,
    pub enabled: bool,
}
```

## Data Flow

### Request Processing Flow

```
1. RPC Request (with auth_token in client_info)
   ↓
2. RpcService.process_request()
   ├─ Extract auth_token from client_info
   ├─ Validate token with AuthenticationAdapter
   ├─ Create SecurityContext with permissions
   └─ Validate request against security policy
   ↓
3. Parameter Validation
   ├─ Lookup method in MethodRegistry
   ├─ Validate parameters against rules
   └─ Apply constraint validation
   ↓
4. External RPC Call
   ├─ Call Verus daemon via ExternalRpcAdapter
   ├─ Handle response and errors
   └─ Format response
   ↓
5. Return RpcResponse
```

### Authentication Flow

```
1. HTTP Request with Authorization Header
   ↓
2. Header Extraction in HTTP Layer
   ├─ Authorization: Bearer <token>
   ├─ User-Agent: <client_info>
   └─ X-Forwarded-For: <client_ip>
   ↓
3. RequestContext Creation
   ├─ auth_token: from Authorization header
   ├─ user_agent: from User-Agent header
   └─ client_ip: validated IP address
   ↓
4. Domain Model Conversion
   ├─ ClientInfo with auth_token field
   └─ RpcRequest with complete client context
   ↓
5. RpcService Processing
   ├─ Extract auth_token from client_info
   ├─ Validate with AuthenticationAdapter
   ├─ Create SecurityContext with permissions
   └─ Apply security validation
```

## Testing Strategy

### Unit Testing

Each service component has comprehensive unit tests:

**RpcService Tests**:
- Request processing with valid/invalid tokens
- Security context creation and validation
- Error handling scenarios
- Dependency injection testing

**Token Extraction Tests**:
- Valid Bearer token extraction
- Invalid token handling
- Missing token scenarios

**Parameter Validation Tests**:
- Valid parameter validation
- Constraint violation testing
- Array and object parameter formats
- Custom validation rules

**Method Registry Tests**:
- Method lookup functionality
- Security rule validation
- Parameter rule verification

### Integration Testing

- End-to-end request processing
- Authentication flow testing
- Error handling and recovery
- Performance and load testing

## Configuration

### Service Configuration

```toml
[security]
development_mode = false
enable_security_headers = true

[jwt]
secret_key = "your-32-character-secret-key-here"
expiration_seconds = 3600
issuer = "verus-rpc-server"
audience = "verus-clients"

[verus]
rpc_url = "http://127.0.0.1:27486"
rpc_user = "your_username"
rpc_password = "your_password"
```

### Development Mode

For development purposes, authentication can be bypassed:

```toml
[security]
development_mode = true  # Disables authentication
```

**Warning**: Never use development mode in production!

## Security Considerations

### Authentication

- **JWT Token Validation**: Secure token validation with proper error handling
- **Permission-Based Access**: Method-specific permission requirements
- **IP Address Validation**: Client IP validation and restrictions
- **Development Mode**: Secure development mode with proper warnings

### Input Validation

- **Parameter Sanitization**: All input parameters are validated
- **Type Safety**: Strong typing with Rust's type system
- **Constraint Validation**: Length, pattern, and value constraints
- **Method Allowlist**: Only allowed RPC methods are processed

### Error Handling

- **Secure Error Messages**: No sensitive information in error responses
- **Graceful Degradation**: Proper handling of authentication failures
- **Logging**: Comprehensive security event logging
- **Rate Limiting**: Protection against abuse

## Monitoring and Observability

### Metrics Collection

- **Request Success/Failure Rates**: Track authentication and processing success
- **Performance Metrics**: Response times and throughput
- **Security Metrics**: Authentication failures and security violations
- **Method Usage**: Track which RPC methods are most used

### Logging

- **Structured Logging**: JSON format for easy parsing
- **Security Events**: Authentication and authorization events
- **Request Lifecycle**: Complete request processing logs
- **Error Tracking**: Detailed error information for debugging

## Related Documentation

- [System Architecture](./system-architecture.md) - Overall system architecture
- [Security Overview](../security/security-overview.md) - Security implementation details
- [API Documentation](../api/) - API reference and examples
- [Development Guide](../development/) - Development setup and guidelines