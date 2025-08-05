# Rust Verus RPC Server

A secure, high-performance RPC server for Verus blockchain built in Rust. This server acts as a proxy between clients and the Verus daemon, providing method validation and security controls.

## Features

- **Method Validation**: Only allows specific RPC methods through a comprehensive allowlist
- **CORS Support**: Built-in CORS headers for web applications
- **Security**: Input validation and parameter type checking
- **High Performance**: Built with Rust and Warp for optimal performance
- **Easy Configuration**: Simple TOML configuration file

## Prerequisites

- Rust programming language (latest stable version)
- A running Verus daemon (verusd)
- Git

## Installation

1. Clone the repository:
```bash
git clone https://github.com/VerusCoin/rust_verusd_rpc_server.git
cd rust_verusd_rpc_server
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

1. Edit the `Conf.toml` file with your Verus daemon settings:

```toml
# The URL and port of your Verus daemon RPC endpoint
rpc_url = "http://127.0.0.1:27486"

# RPC username (from your verus.conf file)
rpc_user = "your_rpc_username"

# RPC password (from your verus.conf file)
rpc_password = "your_rpc_password"

# Port for the HTTP server to listen on
server_port = 8080

# IP address to bind the server to (use "0.0.0.0" for all interfaces)
server_addr = "127.0.0.1"
```

2. Make sure your Verus daemon is running and accessible at the configured RPC endpoint.

## Usage

### Running the Server

```bash
cargo run --release
```

Or run the compiled binary:
```bash
./target/release/rust_verusd_rpc_server
```

The server will start and listen on the configured address and port.

### Making RPC Calls

The server accepts JSON-RPC requests at the root endpoint (`/`). Example:

```bash
curl -X POST http://127.0.0.1:8080/ \
  -H "Content-Type: application/json" \
  -d '{
    "method": "getinfo",
    "params": []
  }'
```

### Supported Methods

The server supports a comprehensive list of Verus RPC methods including:

- **Blockchain Info**: `getinfo`, `getblockchaininfo`, `getblockcount`, etc.
- **Block Operations**: `getblock`, `getblockhash`, `getblockheader`, etc.
- **Transaction Operations**: `getrawtransaction`, `sendrawtransaction`, etc.
- **Address Operations**: `getaddressbalance`, `getaddressutxos`, etc.
- **Identity Operations**: `getidentity`, `registeridentity`, `updateidentity`, etc.
- **Currency Operations**: `getcurrency`, `sendcurrency`, etc.

For a complete list, see the `src/allowlist.rs` file.

## Security Features

- **Method Allowlist**: Only pre-approved RPC methods are allowed
- **Parameter Validation**: Strict type checking for all parameters
- **Input Sanitization**: All inputs are validated before being passed to the daemon
- **CORS Protection**: Configurable CORS headers for web security

## Development

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

## Architecture

The server is built with a modular architecture:

- **HTTP Layer**: Warp web framework for handling HTTP requests
- **RPC Proxy**: JSON-RPC client for communicating with Verus daemon
- **Method Validation**: Allowlist system for method and parameter validation
- **Configuration**: TOML-based configuration system

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:
- Create an issue on GitHub
- Check the Verus documentation
- Join the Verus community channels

## Security Considerations

- Always run the server behind a firewall
- Use HTTPS in production environments
- Regularly update dependencies
- Monitor server logs for suspicious activity
- Consider rate limiting for production deployments
