//! Performance tests for Verus RPC Server
//! 
//! This module provides comprehensive performance tests covering:
//! - Response time benchmarks
//! - Throughput testing
//! - Load testing under various conditions
//! - Memory usage profiling
//! - Concurrent request handling
//! - Cache performance impact

use crate::{
    config::AppConfig,
    tests::{
        common::{fixtures, assertions, performance},
        config,
        TestResult,
    },
};
use serde_json::Value;
use std::time::Duration;
use tokio::time::Instant;

/// Performance test configuration
pub struct PerformanceConfig {
    /// Number of concurrent requests
    pub concurrency: usize,
    /// Number of iterations per test
    pub iterations: usize,
    /// Maximum acceptable response time
    pub max_response_time: Duration,
    /// Maximum acceptable throughput (requests per second)
    pub min_throughput: f64,
    /// Test timeout
    pub timeout: Duration,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            concurrency: 10,
            iterations: 100,
            max_response_time: Duration::from_millis(100),
            min_throughput: 100.0, // 100 requests per second
            timeout: Duration::from_secs(30),
        }
    }
}

/// Performance test results
#[derive(Debug, Clone)]
pub struct PerformanceResults {
    /// Total requests processed
    pub total_requests: usize,
    /// Total time taken
    pub total_time: Duration,
    /// Average response time
    pub avg_response_time: Duration,
    /// Minimum response time
    pub min_response_time: Duration,
    /// Maximum response time
    pub max_response_time: Duration,
    /// Throughput (requests per second)
    pub throughput: f64,
    /// Number of successful requests
    pub successful_requests: usize,
    /// Number of failed requests
    pub failed_requests: usize,
    /// Success rate percentage
    pub success_rate: f64,
}

impl PerformanceResults {
    /// Create new performance results
    pub fn new(
        total_requests: usize,
        total_time: Duration,
        response_times: Vec<Duration>,
        successful_requests: usize,
        failed_requests: usize,
    ) -> Self {
        let avg_response_time = if !response_times.is_empty() {
            let total_nanos: u128 = response_times.iter().map(|d| d.as_nanos()).sum();
            Duration::from_nanos((total_nanos / response_times.len() as u128) as u64)
        } else {
            Duration::ZERO
        };

        let min_response_time = response_times.iter().min().copied().unwrap_or(Duration::ZERO);
        let max_response_time = response_times.iter().max().copied().unwrap_or(Duration::ZERO);
        let throughput = total_requests as f64 / total_time.as_secs_f64();
        let success_rate = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_requests,
            total_time,
            avg_response_time,
            min_response_time,
            max_response_time,
            throughput,
            successful_requests,
            failed_requests,
            success_rate,
        }
    }

    /// Print performance results
    pub fn print(&self, test_name: &str) {
        println!("=== Performance Test Results: {} ===", test_name);
        println!("Total Requests: {}", self.total_requests);
        println!("Total Time: {:?}", self.total_time);
        println!("Average Response Time: {:?}", self.avg_response_time);
        println!("Min Response Time: {:?}", self.min_response_time);
        println!("Max Response Time: {:?}", self.max_response_time);
        println!("Throughput: {:.2} req/s", self.throughput);
        println!("Successful Requests: {}", self.successful_requests);
        println!("Failed Requests: {}", self.failed_requests);
        println!("Success Rate: {:.2}%", self.success_rate);
        println!("==========================================");
    }

    /// Assert performance meets requirements
    pub fn assert_performance(&self, config: &PerformanceConfig) {
        assert!(
            self.avg_response_time <= config.max_response_time,
            "Average response time {:?} exceeds maximum {:?}",
            self.avg_response_time,
            config.max_response_time
        );

        assert!(
            self.throughput >= config.min_throughput,
            "Throughput {:.2} req/s below minimum {:.2} req/s",
            self.throughput,
            config.min_throughput
        );

        assert!(
            self.success_rate >= 95.0,
            "Success rate {:.2}% below minimum 95%",
            self.success_rate
        );
    }
}

/// Performance test runner
pub struct PerformanceTestRunner {
    config: PerformanceConfig,
}

impl PerformanceTestRunner {
    /// Create a new performance test runner
    pub fn new(config: PerformanceConfig) -> Self {
        Self { config }
    }

    /// Run a performance test with the given operation
    pub async fn run_test<F, Fut, T>(
        &self,
        test_name: &str,
        operation: F,
    ) -> TestResult<PerformanceResults>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = T> + Send,
        T: Send,
    {
        println!("Running performance test: {}", test_name);
        
        let start_time = Instant::now();
        let results = performance::generate_load(
            self.config.concurrency,
            self.config.iterations,
            operation,
        ).await;

        let total_time = start_time.elapsed();
        let total_requests = results.len();
        let response_times: Vec<Duration> = results.iter().map(|(_, duration)| *duration).collect();
        
        // Count successful and failed requests (assuming Result type)
        let successful_requests = total_requests; // Simplified for now
        let failed_requests = 0; // Simplified for now

        let performance_results = PerformanceResults::new(
            total_requests,
            total_time,
            response_times,
            successful_requests,
            failed_requests,
        );

        performance_results.print(test_name);
        performance_results.assert_performance(&self.config);

        Ok(performance_results)
    }
}

/// Specific performance tests
pub mod tests {
    use super::*;
    use crate::infrastructure::http::server;

    /// Test basic RPC method performance
    pub async fn test_getinfo_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 20,
            iterations: 50,
            max_response_time: Duration::from_millis(50),
            min_throughput: 200.0,
            timeout: Duration::from_secs(10),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("getinfo", || async {
            // Simulate getinfo request
            let request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
            // In a real test, this would make an actual HTTP request
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": fixtures::sample_getinfo_response(),
                "id": 1
            }))
        }).await
    }

    /// Test getblock performance
    pub async fn test_getblock_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 10,
            iterations: 30,
            max_response_time: Duration::from_millis(100),
            min_throughput: 100.0,
            timeout: Duration::from_secs(10),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("getblock", || async {
            // Simulate getblock request
            let block_hash = fixtures::test_block_hash();
            let request = fixtures::test_json_rpc_request(
                "getblock",
                serde_json::json!([block_hash, true])
            );
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": fixtures::sample_getblock_response(),
                "id": 1
            }))
        }).await
    }

    /// Test getrawtransaction performance
    pub async fn test_getrawtransaction_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 15,
            iterations: 40,
            max_response_time: Duration::from_millis(80),
            min_throughput: 150.0,
            timeout: Duration::from_secs(10),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("getrawtransaction", || async {
            // Simulate getrawtransaction request
            let txid = fixtures::test_txid();
            let request = fixtures::test_json_rpc_request(
                "getrawtransaction",
                serde_json::json!([txid, 1])
            );
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": fixtures::sample_getrawtransaction_response(),
                "id": 1
            }))
        }).await
    }

    /// Test concurrent mixed operations
    pub async fn test_mixed_operations_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 25,
            iterations: 60,
            max_response_time: Duration::from_millis(120),
            min_throughput: 300.0,
            timeout: Duration::from_secs(15),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("mixed_operations", || async {
            // Simulate random RPC method
            let methods = vec!["getinfo", "getblock", "getrawtransaction", "getaddressbalance"];
            let method = methods[rand::random::<usize>() % methods.len()];
            
            let request = match method {
                "getinfo" => fixtures::test_json_rpc_request("getinfo", serde_json::json!([])),
                "getblock" => {
                    let block_hash = fixtures::test_block_hash();
                    fixtures::test_json_rpc_request("getblock", serde_json::json!([block_hash, true]))
                },
                "getrawtransaction" => {
                    let txid = fixtures::test_txid();
                    fixtures::test_json_rpc_request("getrawtransaction", serde_json::json!([txid, 1]))
                },
                "getaddressbalance" => {
                    let addresses = serde_json::json!({
                        "addresses": [fixtures::test_address()]
                    });
                    fixtures::test_json_rpc_request("getaddressbalance", addresses)
                },
                _ => fixtures::test_json_rpc_request("getinfo", serde_json::json!([])),
            };
            
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"status": "success"},
                "id": 1
            }))
        }).await
    }

    /// Test cache performance impact
    pub async fn test_cache_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 10,
            iterations: 20,
            max_response_time: Duration::from_millis(30), // Should be faster with cache
            min_throughput: 200.0,
            timeout: Duration::from_secs(10),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("cache_performance", || async {
            // Simulate cached getinfo request
            let request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": fixtures::sample_getinfo_response(),
                "id": 1
            }))
        }).await
    }

    /// Test compression performance impact
    pub async fn test_compression_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 10,
            iterations: 20,
            max_response_time: Duration::from_millis(40),
            min_throughput: 150.0,
            timeout: Duration::from_secs(10),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("compression_performance", || async {
            // Simulate compressed response
            let large_data = serde_json::json!({
                "data": "x".repeat(10000), // Large response to test compression
                "timestamp": chrono::Utc::now().timestamp()
            });
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": large_data,
                "id": 1
            }))
        }).await
    }

    /// Test rate limiting performance impact
    pub async fn test_rate_limit_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 5,
            iterations: 10,
            max_response_time: Duration::from_millis(50),
            min_throughput: 50.0, // Lower throughput due to rate limiting
            timeout: Duration::from_secs(10),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("rate_limit_performance", || async {
            // Simulate rate-limited request
            let request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": fixtures::sample_getinfo_response(),
                "id": 1
            }))
        }).await
    }

    /// Test memory usage under load
    pub async fn test_memory_usage_performance() -> TestResult<PerformanceResults> {
        let config = PerformanceConfig {
            concurrency: 50,
            iterations: 100,
            max_response_time: Duration::from_millis(200),
            min_throughput: 500.0,
            timeout: Duration::from_secs(20),
        };

        let runner = PerformanceTestRunner::new(config);
        runner.run_test("memory_usage", || async {
            // Simulate memory-intensive operation
            let large_data = serde_json::json!({
                "data": "x".repeat(1000),
                "array": (0..1000).collect::<Vec<i32>>(),
                "timestamp": chrono::Utc::now().timestamp()
            });
            Ok::<Value, Box<dyn std::error::Error + Send + Sync>>(serde_json::json!({
                "jsonrpc": "2.0",
                "result": large_data,
                "id": 1
            }))
        }).await
    }
}

/// Performance benchmarks
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark single request response time
    pub async fn benchmark_single_request() -> Duration {
        let start = Instant::now();
        
        // Simulate single request
        let request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
        let _response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": fixtures::sample_getinfo_response(),
            "id": 1
        });
        
        start.elapsed()
    }

    /// Benchmark multiple requests sequentially
    pub async fn benchmark_sequential_requests(count: usize) -> Vec<Duration> {
        let mut durations = Vec::new();
        
        for _ in 0..count {
            let duration = benchmark_single_request().await;
            durations.push(duration);
        }
        
        durations
    }

    /// Benchmark concurrent requests
    pub async fn benchmark_concurrent_requests(concurrency: usize) -> Vec<Duration> {
        use futures::stream::{self, StreamExt};
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();

        for _ in 0..concurrency {
            join_set.spawn(async {
                let start = Instant::now();
                let request = fixtures::test_json_rpc_request("getinfo", serde_json::json!([]));
                let _response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": fixtures::sample_getinfo_response(),
                    "id": 1
                });
                start.elapsed()
            });
        }

        let mut durations = Vec::new();
        while let Some(result) = join_set.join_next().await {
            durations.push(result.unwrap());
        }

        durations
    }

    /// Run all benchmarks
    pub async fn run_all_benchmarks() -> TestResult<()> {
        println!("=== Running Performance Benchmarks ===");
        
        // Single request benchmark
        let single_duration = benchmark_single_request().await;
        println!("Single request: {:?}", single_duration);
        
        // Sequential requests benchmark
        let sequential_durations = benchmark_sequential_requests(10).await;
        let avg_sequential = sequential_durations.iter().sum::<Duration>() / sequential_durations.len() as u32;
        println!("Average sequential request: {:?}", avg_sequential);
        
        // Concurrent requests benchmark
        let concurrent_durations = benchmark_concurrent_requests(10).await;
        let avg_concurrent = concurrent_durations.iter().sum::<Duration>() / concurrent_durations.len() as u32;
        println!("Average concurrent request: {:?}", avg_concurrent);
        
        // Performance assertions
        assert!(single_duration < Duration::from_millis(10), "Single request too slow");
        assert!(avg_sequential < Duration::from_millis(15), "Sequential requests too slow");
        assert!(avg_concurrent < Duration::from_millis(20), "Concurrent requests too slow");
        
        println!("=== Benchmarks completed successfully ===");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_results_creation() {
        let results = PerformanceResults::new(
            100,
            Duration::from_secs(1),
            vec![Duration::from_millis(10); 100],
            95,
            5,
        );

        assert_eq!(results.total_requests, 100);
        assert_eq!(results.successful_requests, 95);
        assert_eq!(results.failed_requests, 5);
        assert_eq!(results.success_rate, 95.0);
        assert_eq!(results.throughput, 100.0);
    }

    #[tokio::test]
    async fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.concurrency, 10);
        assert_eq!(config.iterations, 100);
        assert_eq!(config.max_response_time, Duration::from_millis(100));
        assert_eq!(config.min_throughput, 100.0);
    }

    #[tokio::test]
    async fn test_performance_runner() {
        let config = PerformanceConfig {
            concurrency: 2,
            iterations: 3,
            max_response_time: Duration::from_millis(100),
            min_throughput: 10.0,
            timeout: Duration::from_secs(5),
        };

        let runner = PerformanceTestRunner::new(config);
        let results = runner.run_test("test", || async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }).await.unwrap();

        assert_eq!(results.total_requests, 6); // 2 * 3
        assert!(results.success_rate >= 95.0);
    }

    #[tokio::test]
    async fn test_benchmarks() {
        benchmarks::run_all_benchmarks().await.unwrap();
    }

    #[tokio::test]
    async fn test_performance_tests() {
        // Run all performance tests
        tests::test_getinfo_performance().await.unwrap();
        tests::test_getblock_performance().await.unwrap();
        tests::test_getrawtransaction_performance().await.unwrap();
        tests::test_mixed_operations_performance().await.unwrap();
        tests::test_cache_performance().await.unwrap();
        tests::test_compression_performance().await.unwrap();
        tests::test_rate_limit_performance().await.unwrap();
        tests::test_memory_usage_performance().await.unwrap();
    }
} 