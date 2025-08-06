//! Metrics utilities module
//! 
//! This module provides centralized metrics functionality and utilities.

use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::time::{SystemTime, Duration};
use serde::{Deserialize, Serialize};

/// Metrics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Total requests processed
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// Rate limited requests
    pub rate_limited_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Current active connections
    pub active_connections: u32,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Metrics utilities for the application
pub struct MetricsUtils {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    rate_limited_requests: AtomicU64,
    total_response_time: AtomicU64,
    response_count: AtomicU64,
    active_connections: AtomicU32,
    start_time: SystemTime,
}

impl MetricsUtils {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            rate_limited_requests: AtomicU64::new(0),
            total_response_time: AtomicU64::new(0),
            response_count: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
            start_time: SystemTime::now(),
        }
    }

    /// Increment total requests
    pub fn increment_total_requests(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment successful requests
    pub fn increment_successful_requests(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment failed requests
    pub fn increment_failed_requests(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment rate limited requests
    pub fn increment_rate_limited_requests(&self) {
        self.rate_limited_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record response time
    pub fn record_response_time(&self, duration_ms: u64) {
        self.total_response_time.fetch_add(duration_ms, Ordering::Relaxed);
        self.response_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn increment_active_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn decrement_active_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Metrics {
        let total_response_time = self.total_response_time.load(Ordering::Relaxed);
        let response_count = self.response_count.load(Ordering::Relaxed);
        
        let avg_response_time_ms = if response_count > 0 {
            total_response_time as f64 / response_count as f64
        } else {
            0.0
        };

        let uptime = SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Metrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            rate_limited_requests: self.rate_limited_requests.load(Ordering::Relaxed),
            avg_response_time_ms,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            uptime_seconds: uptime,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.rate_limited_requests.store(0, Ordering::Relaxed);
        self.total_response_time.store(0, Ordering::Relaxed);
        self.response_count.store(0, Ordering::Relaxed);
        self.active_connections.store(0, Ordering::Relaxed);
    }
}

impl Default for MetricsUtils {
    fn default() -> Self {
        Self::new()
    }
} 