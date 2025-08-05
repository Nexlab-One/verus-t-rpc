use verus_t_rpc::{AppConfig, VerusRpcServer};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    // Initialize logging
    if let Err(e) = initialize_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Starting Verus RPC Server...");

    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Create and start server
    let server = match VerusRpcServer::new(config).await {
        Ok(server) => {
            info!("Server initialized successfully");
            server
        }
        Err(e) => {
            error!("Failed to initialize server: {}", e);
            std::process::exit(1);
        }
    };

    // Start the server
    info!("Server starting on {}", server.config().server_address());
    
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}

fn initialize_logging() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    
    Ok(())
}
