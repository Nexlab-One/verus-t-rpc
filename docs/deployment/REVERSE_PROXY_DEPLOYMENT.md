# Verus RPC Server - Reverse Proxy Deployment Guide

## 📋 **Configuration for Reverse Proxy Deployment**

### **Application Configuration (Conf.toml):**
```toml
[server]
bind_address = "127.0.0.1"  # Only bind to localhost
port = 8080
ssl_enabled = false  # Let reverse proxy handle SSL
compression_enabled = false  # Let reverse proxy handle compression

[security]
# Trust proxy headers for proper client IP handling
trusted_proxy_headers = ["X-Forwarded-For", "X-Real-IP"]
# CORS configuration (for reference only - handled by reverse proxy)
cors_origins = ["https://yourdomain.com"]
cors_methods = ["GET", "POST", "OPTIONS"]
cors_headers = ["Content-Type", "Authorization"]

[rate_limit]
enabled = true
requests_per_minute = 1000
burst_size = 100
```

### **Caddy Configuration (Caddyfile):**
```caddyfile
verus.example.com {
    # SSL/TLS termination
    tls your-email@example.com
    
    # Reverse proxy to Verus RPC server
    reverse_proxy 127.0.0.1:8080 {
        # Health check
        health_uri /health
        health_interval 30s
        health_timeout 10s
        
        # Headers to forward
        header_up X-Real-IP {remote_host}
        header_up X-Forwarded-For {remote_host}
        header_up X-Forwarded-Proto {scheme}
        header_up Host {host}
    }
    
    # Compression
    encode gzip
    
    # CORS headers
    header {
        Access-Control-Allow-Origin https://yourdomain.com
        Access-Control-Allow-Methods "GET, POST, OPTIONS"
        Access-Control-Allow-Headers "Content-Type, Authorization"
        Access-Control-Max-Age 3600
    }
    
    # Security headers
    header {
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy strict-origin-when-cross-origin
        Permissions-Policy "geolocation=(), microphone=(), camera=()"
    }
}
```

### **Nginx Configuration:**
```nginx
server {
    listen 443 ssl;
    server_name verus.example.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # Compression
    gzip on;
    gzip_types application/json text/plain;
    
    # CORS headers
    add_header Access-Control-Allow-Origin https://yourdomain.com;
    add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
    add_header Access-Control-Allow-Headers "Content-Type, Authorization";
    add_header Access-Control-Max-Age 3600;
    
    # Security headers
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy strict-origin-when-cross-origin;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Health check
        proxy_connect_timeout 10s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
}
```

## 🐳 **Docker Deployment**

### **Docker Compose Example:**
```yaml
version: '3.8'
services:
  verus-rpc:
    build: .
    ports:
      - "127.0.0.1:8080:8080"  # Only expose locally
    environment:
      - VERUS_RPC__SERVER__BIND_ADDRESS=0.0.0.0
      - VERUS_RPC__SERVER__SSL_ENABLED=false
      - VERUS_RPC__SECURITY__TRUSTED_PROXY_HEADERS=["X-Forwarded-For", "X-Real-IP"]
    volumes:
      - ./Conf.toml:/app/Conf.toml:ro
    restart: unless-stopped

  caddy:
    image: caddy:2-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    restart: unless-stopped

volumes:
  caddy_data:
  caddy_config:
```

## 🔧 **Migration Guide**

### **From Standalone to Reverse Proxy:**

1. **Update Configuration:**
   ```bash
   # Set SSL and compression to false
   sed -i 's/ssl_enabled = true/ssl_enabled = false/' Conf.toml
   sed -i 's/compression_enabled = true/compression_enabled = false/' Conf.toml
   ```

2. **Configure Reverse Proxy:**
   - Set up nginx/Caddy with SSL certificates
   - Configure compression and CORS
   - Set up health checks

3. **Update Client Configuration:**
   - Point clients to the reverse proxy URL
   - Update any hardcoded HTTP URLs to HTTPS

4. **Test Deployment:**
   ```bash
   # Test health endpoint
   curl https://verus.example.com/health
   
   # Test RPC endpoint
   curl -X POST https://verus.example.com/ \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"getinfo","params":[],"id":1}'
   ```

## 📊 **Monitoring and Observability**

### **Application Metrics:**
- `/metrics` - Application-specific metrics
- `/prometheus` - Prometheus-formatted metrics
- `/health` - Health check endpoint

### **Reverse Proxy Metrics:**
- nginx: `stub_status` module
- Caddy: Built-in metrics endpoint
- Prometheus: Export metrics for monitoring

### **Logging:**
- Application logs: Structured JSON logging
- Reverse proxy logs: Access and error logs
- Centralized logging: ELK stack or similar

## 🚀 **Production Checklist**

- [ ] SSL certificates configured in reverse proxy
- [ ] Compression enabled in reverse proxy
- [ ] CORS headers configured in reverse proxy
- [ ] Security headers configured in reverse proxy
- [ ] Health checks configured
- [ ] Rate limiting configured
- [ ] Monitoring and alerting set up
- [ ] Logging centralized
- [ ] Backup and recovery procedures
- [ ] Load balancing configured (if needed)
