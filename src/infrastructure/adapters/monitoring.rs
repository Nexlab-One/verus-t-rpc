//! Monitoring adapter for metrics and observability
//! 
//! This adapter handles Prometheus metrics collection and security event logging.

use crate::domain::security::SecurityEvent;
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use tracing::warn;

/// Adapter for monitoring and metrics services
pub struct MonitoringAdapter {
    prometheus_registry: prometheus::Registry,
    request_counter: prometheus::Counter,
    response_time_histogram: prometheus::Histogram,
    active_connections_gauge: prometheus::Gauge,
    rate_limited_requests: AtomicU64,
    total_response_time: AtomicU64,
    response_count: AtomicU64,
    active_connections: AtomicU32,
}

impl MonitoringAdapter {
    /// Create a new monitoring adapter
    pub fn new() -> Self {
        let registry = prometheus::Registry::new();
        
        // Create Prometheus metrics
        let request_counter = prometheus::Counter::new(
            "rpc_requests_total",
            "Total number of RPC requests"
        ).unwrap();
        
        let response_time_histogram = prometheus::Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "rpc_response_time_seconds",
                "RPC response time in seconds"
            )
        ).unwrap();
        
        let active_connections_gauge = prometheus::Gauge::new(
            "rpc_active_connections",
            "Number of active connections"
        ).unwrap();

        // Register metrics with registry
        registry.register(Box::new(request_counter.clone())).unwrap();
        registry.register(Box::new(response_time_histogram.clone())).unwrap();
        registry.register(Box::new(active_connections_gauge.clone())).unwrap();

        Self {
            prometheus_registry: registry,
            request_counter,
            response_time_histogram,
            active_connections_gauge,
            rate_limited_requests: AtomicU64::new(0),
            total_response_time: AtomicU64::new(0),
            response_count: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
        }
    }

    /// Log security event
    pub async fn log_security_event(&self, event: &SecurityEvent) {
        warn!(
            event_type = %event.event_type,
            client_ip = %event.client_ip,
            method = %event.method,
            details = %event.details,
            "Security event detected"
        );
        
        // TODO: In production, this would also send to:
        // - SIEM system
        // - Security monitoring dashboard
        // - Alert system
    }

    /// Record metrics for a request
    pub async fn record_metrics(&self, metrics: &MetricsEvent) {
        // Update Prometheus metrics
        self.request_counter.inc();
        self.response_time_histogram.observe(metrics.response_time_ms / 1000.0);
        
        // Update internal counters
        self.total_response_time.fetch_add(metrics.response_time_ms as u64, Ordering::Relaxed);
        self.response_count.fetch_add(1, Ordering::Relaxed);
        
        // TODO: In production, this would also send to:
        // - Time-series database (InfluxDB, Prometheus)
        // - APM system (DataDog, New Relic)
        // - Custom metrics dashboard
    }

    /// Get Prometheus metrics in text format
    pub fn get_prometheus_metrics(&self) -> String {
        use prometheus::Encoder;
        let mut buffer = Vec::new();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&self.prometheus_registry.gather(), &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    /// Update active connections count
    pub fn update_active_connections(&self, count: i64) {
        self.active_connections_gauge.set(count as f64);
        self.active_connections.store(count as u32, Ordering::Relaxed);
    }

    /// Record rate limited request
    pub fn record_rate_limited_request(&self) {
        self.rate_limited_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record response time
    pub fn record_response_time(&self, response_time_ms: f64) {
        self.total_response_time.fetch_add(response_time_ms as u64, Ordering::Relaxed);
        self.response_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn increment_active_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.active_connections_gauge.inc();
    }

    /// Decrement active connections
    pub fn decrement_active_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.active_connections_gauge.dec();
    }

    /// Get metrics summary
    pub fn get_metrics(&self) -> MetricsSummary {
        let total_requests = self.request_counter.get();
        let avg_response_time = if self.response_count.load(Ordering::Relaxed) > 0 {
            self.total_response_time.load(Ordering::Relaxed) as f64 / self.response_count.load(Ordering::Relaxed) as f64
        } else {
            0.0
        };

        MetricsSummary {
            total_requests,
            avg_response_time_ms: avg_response_time,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            rate_limited_requests: self.rate_limited_requests.load(Ordering::Relaxed),
        }
    }
}

impl Default for MonitoringAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics event for recording request metrics
pub struct MetricsEvent {
    pub request_count: u64,
    pub response_time_ms: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metrics summary for monitoring
pub struct MetricsSummary {
    pub total_requests: f64,
    pub avg_response_time_ms: f64,
    pub active_connections: u32,
    pub rate_limited_requests: u64,
} 