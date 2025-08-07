//! Metrics service for collecting application metrics

use serde_json::Value;

/// Metrics service for collecting application metrics
pub struct MetricsService {
    total_requests: std::sync::atomic::AtomicU64,
    successful_requests: std::sync::atomic::AtomicU64,
    failed_requests: std::sync::atomic::AtomicU64,
    rate_limited_requests: std::sync::atomic::AtomicU64,
    total_response_time: std::sync::atomic::AtomicU64,
    response_count: std::sync::atomic::AtomicU64,
    active_connections: std::sync::atomic::AtomicU32,
    start_time: std::time::Instant,
}

impl MetricsService {
    /// Create a new metrics service
    pub fn new() -> Self {
        Self {
            total_requests: std::sync::atomic::AtomicU64::new(0),
            successful_requests: std::sync::atomic::AtomicU64::new(0),
            failed_requests: std::sync::atomic::AtomicU64::new(0),
            rate_limited_requests: std::sync::atomic::AtomicU64::new(0),
            total_response_time: std::sync::atomic::AtomicU64::new(0),
            response_count: std::sync::atomic::AtomicU64::new(0),
            active_connections: std::sync::atomic::AtomicU32::new(0),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a request
    pub fn record_request(&self, success: bool) {
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Record a rate limited request
    pub fn record_rate_limited_request(&self) {
        self.rate_limited_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Record response time
    pub fn record_response_time(&self, duration_ms: u64) {
        self.total_response_time.fetch_add(duration_ms, std::sync::atomic::Ordering::Relaxed);
        self.response_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn increment_active_connections(&self) {
        self.active_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn decrement_active_connections(&self) {
        self.active_connections.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Value {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        let successful = self.successful_requests.load(std::sync::atomic::Ordering::Relaxed);
        let failed = self.failed_requests.load(std::sync::atomic::Ordering::Relaxed);
        let rate_limited = self.rate_limited_requests.load(std::sync::atomic::Ordering::Relaxed);
        let total_response_time = self.total_response_time.load(std::sync::atomic::Ordering::Relaxed);
        let response_count = self.response_count.load(std::sync::atomic::Ordering::Relaxed);
        let active_connections = self.active_connections.load(std::sync::atomic::Ordering::Relaxed);
        let uptime = self.start_time.elapsed().as_secs();

        let avg_response_time_ms = if response_count > 0 {
            total_response_time as f64 / response_count as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_requests": total,
            "successful_requests": successful,
            "failed_requests": failed,
            "rate_limited_requests": rate_limited,
            "avg_response_time_ms": avg_response_time_ms,
            "active_connections": active_connections,
            "uptime_seconds": uptime,
        })
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}


