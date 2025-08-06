use verus_rpc_server::{AppConfig, VerusRpcServer};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Verus RPC Server (Reverse Proxy Mode)");
    info!("SSL/TLS, compression, and CORS should be handled by the reverse proxy");

    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };

    // Validate configuration for reverse proxy deployment
    if let Err(e) = config.validate_config() {
        error!("Configuration validation failed: {}", e);
        return Err(format!("Configuration validation failed: {}", e).into());
    }

    // Print deployment recommendations
    info!("=== Reverse Proxy Deployment Recommendations ===");
    info!("1. Configure SSL/TLS termination in your reverse proxy (nginx, Caddy, etc.)");
    info!("2. Configure compression (gzip/brotli) in your reverse proxy");
    info!("3. Configure CORS headers in your reverse proxy");
    info!("4. Set trusted_proxy_headers for proper client IP handling");
    info!("5. Configure rate limiting based on real client IPs from proxy headers");
    info!("================================================");

    // Create and run the server
    let server = match VerusRpcServer::new(config).await {
        Ok(server) => {
            info!("Server initialized successfully");
            server
        }
        Err(e) => {
            error!("Failed to initialize server: {}", e);
            return Err(e.into());
        }
    };

    // Run the server
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
