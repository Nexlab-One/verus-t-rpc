use crate::config::AppConfig;
use std::collections::HashMap;
use warp::http::{HeaderName, HeaderValue, StatusCode};

/// Security headers middleware for HTTP responses
pub struct SecurityHeadersMiddleware {
    config: AppConfig,
}

impl SecurityHeadersMiddleware {
    /// Create a new security headers middleware
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Check if security headers are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.security.enable_security_headers
    }

    /// Get security headers for responses
    pub fn get_security_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if !self.config.security.enable_security_headers {
            return headers;
        }

        // Content Security Policy
        headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' https:; connect-src 'self'; frame-ancestors 'none';".to_string(),
        );

        // X-Content-Type-Options
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());

        // X-Frame-Options
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());

        // X-XSS-Protection
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());

        // Referrer Policy
        headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());

        // Permissions Policy
        headers.insert(
            "Permissions-Policy".to_string(),
            "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=()".to_string(),
        );

        // Cache Control Headers
        headers.insert("Cache-Control".to_string(), "no-store, no-cache, must-revalidate, proxy-revalidate".to_string());
        headers.insert("Pragma".to_string(), "no-cache".to_string());
        headers.insert("Expires".to_string(), "0".to_string());

        // Custom security headers
        if self.config.security.enable_custom_headers {
            if let Some(custom_header) = &self.config.security.custom_security_header {
                if !custom_header.is_empty() {
                    let parts: Vec<&str> = custom_header.split(':').collect();
                    if parts.len() == 2 {
                        headers.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                    }
                }
            }
        }

        headers
    }
}

/// Add security headers to a response using warp's with_header approach
/// This function returns a boxed trait object to handle the type changes from with_header
pub fn add_security_headers_to_response<T: warp::Reply + Send + 'static>(
    response: T,
    middleware: &SecurityHeadersMiddleware,
) -> Box<dyn warp::Reply> {
    let headers = middleware.get_security_headers();
    
    if headers.is_empty() {
        return Box::new(response);
    }

    // Apply headers one by one using warp's with_header
    let mut response_with_headers: Box<dyn warp::Reply> = Box::new(response);
    
    for (key, value) in headers {
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::from_lowercase(key.to_lowercase().as_bytes()),
            HeaderValue::from_str(&value)
        ) {
            // Create a new response with the header
            let new_response = warp::reply::with_header(response_with_headers, header_name, header_value);
            response_with_headers = Box::new(new_response);
        }
    }

    response_with_headers
}

/// Create a response with security headers from a status code and body
pub fn create_response_with_security_headers(
    status: StatusCode,
    body: String,
    middleware: &SecurityHeadersMiddleware,
) -> Box<dyn warp::Reply> {
    let response = warp::reply::with_status(body, status);
    add_security_headers_to_response(response, middleware)
}

/// Create a JSON response with security headers
pub fn create_json_response_with_security_headers<T: serde::Serialize>(
    data: &T,
    middleware: &SecurityHeadersMiddleware,
) -> Box<dyn warp::Reply> {
    let response = warp::reply::json(data);
    add_security_headers_to_response(response, middleware)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_headers_middleware_creation() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        assert!(middleware.is_enabled());
    }

    #[test]
    fn test_security_headers_generation() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        let headers = middleware.get_security_headers();
        
        // Check that security headers are generated
        assert!(headers.contains_key("Content-Security-Policy"));
        assert!(headers.contains_key("X-Content-Type-Options"));
        assert!(headers.contains_key("X-Frame-Options"));
        assert!(headers.contains_key("X-XSS-Protection"));
        assert!(headers.contains_key("Referrer-Policy"));
        assert!(headers.contains_key("Permissions-Policy"));
        assert!(headers.contains_key("Cache-Control"));
        assert!(headers.contains_key("Pragma"));
        assert!(headers.contains_key("Expires"));
    }

    #[test]
    fn test_custom_security_headers() {
        let mut config = AppConfig::default();
        config.security.enable_custom_headers = true;
        config.security.custom_security_header = Some("X-Custom-Security-Header:custom-value".to_string());
        
        let middleware = SecurityHeadersMiddleware::new(config);
        let headers = middleware.get_security_headers();
        
        assert!(headers.contains_key("X-Custom-Security-Header"));
        assert_eq!(headers.get("X-Custom-Security-Header").unwrap(), "custom-value");
    }

    #[test]
    fn test_security_headers_disabled() {
        let mut config = AppConfig::default();
        config.security.enable_security_headers = false;
        
        let middleware = SecurityHeadersMiddleware::new(config);
        let headers = middleware.get_security_headers();
        
        assert!(headers.is_empty());
    }

    #[test]
    fn test_add_security_headers_to_response() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        
        let test_response = warp::reply::json(&serde_json::json!({"test": "data"}));
        let _response_with_headers = add_security_headers_to_response(test_response, &middleware);
        
        // The response should be a boxed trait object
        assert!(std::any::type_name::<Box<dyn warp::Reply>>() == std::any::type_name::<Box<dyn warp::Reply>>());
    }

    #[test]
    fn test_create_json_response_with_security_headers() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        
        let test_data = serde_json::json!({"test": "data"});
        let _response = create_json_response_with_security_headers(&test_data, &middleware);
        
        // The response should be a boxed trait object
        assert!(std::any::type_name::<Box<dyn warp::Reply>>() == std::any::type_name::<Box<dyn warp::Reply>>());
    }

    #[test]
    fn test_create_response_with_security_headers() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        
        let _response = create_response_with_security_headers(
            warp::http::StatusCode::OK,
            "test body".to_string(),
            &middleware
        );
        
        // The response should be a boxed trait object
        assert!(std::any::type_name::<Box<dyn warp::Reply>>() == std::any::type_name::<Box<dyn warp::Reply>>());
    }

    #[test]
    fn test_security_headers_content() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        let headers = middleware.get_security_headers();
        
        // Test specific header values
        assert_eq!(headers.get("X-Content-Type-Options").unwrap(), "nosniff");
        assert_eq!(headers.get("X-Frame-Options").unwrap(), "DENY");
        assert_eq!(headers.get("X-XSS-Protection").unwrap(), "1; mode=block");
        assert_eq!(headers.get("Referrer-Policy").unwrap(), "strict-origin-when-cross-origin");
        assert_eq!(headers.get("Cache-Control").unwrap(), "no-store, no-cache, must-revalidate, proxy-revalidate");
        assert_eq!(headers.get("Pragma").unwrap(), "no-cache");
        assert_eq!(headers.get("Expires").unwrap(), "0");
    }

    #[test]
    fn test_permissions_policy_content() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        let headers = middleware.get_security_headers();
        
        let permissions_policy = headers.get("Permissions-Policy").unwrap();
        assert!(permissions_policy.contains("geolocation=()"));
        assert!(permissions_policy.contains("microphone=()"));
        assert!(permissions_policy.contains("camera=()"));
        assert!(permissions_policy.contains("payment=()"));
        assert!(permissions_policy.contains("usb=()"));
        assert!(permissions_policy.contains("magnetometer=()"));
        assert!(permissions_policy.contains("gyroscope=()"));
        assert!(permissions_policy.contains("accelerometer=()"));
    }

    #[test]
    fn test_content_security_policy_content() {
        let config = AppConfig::default();
        let middleware = SecurityHeadersMiddleware::new(config);
        let headers = middleware.get_security_headers();
        
        let csp = headers.get("Content-Security-Policy").unwrap();
        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("script-src 'self' 'unsafe-inline'"));
        assert!(csp.contains("style-src 'self' 'unsafe-inline'"));
        assert!(csp.contains("img-src 'self' data: https:"));
        assert!(csp.contains("font-src 'self' https:"));
        assert!(csp.contains("connect-src 'self'"));
        assert!(csp.contains("frame-ancestors 'none'"));
    }
} 