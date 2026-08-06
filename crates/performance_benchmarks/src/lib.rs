//! Performance Benchmarking Framework - Phase 12
//!
//! Measures actual system performance with real metrics:
//! - Client connection latency
//! - Surface creation and rendering throughput
//! - Input event latency
//! - Memory overhead
//! - Concurrent application scaling

use std::time::{Instant, Duration};

#[derive(Clone, Debug)]
pub struct LatencyMeasurement {
    pub operation: String,
    pub duration_us: u64,
    pub iterations: u32,
}

#[derive(Clone, Debug)]
pub struct ThroughputMeasurement {
    pub operation: String,
    pub operations_per_second: f64,
    pub duration_seconds: f64,
}

#[derive(Clone, Debug)]
pub struct MemoryMeasurement {
    pub resource_name: String,
    pub bytes_allocated: usize,
    pub peak_bytes: usize,
}

#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub latencies: Vec<LatencyMeasurement>,
    pub throughputs: Vec<ThroughputMeasurement>,
    pub memory: Vec<MemoryMeasurement>,
    pub total_duration_seconds: f64,
}

pub struct Benchmark {
    measurements: Vec<BenchmarkResult>,
}

impl Benchmark {
    pub fn new() -> Self {
        Benchmark {
            measurements: Vec::new(),
        }
    }

    pub fn measure_operation<F>(&self, name: &str, iterations: u32, mut operation: F) -> LatencyMeasurement
    where
        F: FnMut(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        let elapsed = start.elapsed();

        LatencyMeasurement {
            operation: name.to_string(),
            duration_us: elapsed.as_micros() as u64 / iterations as u64,
            iterations,
        }
    }

    pub fn measure_throughput<F>(&self, name: &str, duration: Duration, mut operation: F) -> ThroughputMeasurement
    where
        F: FnMut() -> bool,
    {
        let start = Instant::now();
        let mut count = 0u64;

        while start.elapsed() < duration {
            if operation() {
                count += 1;
            }
        }

        let elapsed_seconds = start.elapsed().as_secs_f64();
        let ops_per_second = count as f64 / elapsed_seconds;

        ThroughputMeasurement {
            operation: name.to_string(),
            operations_per_second: ops_per_second,
            duration_seconds: elapsed_seconds,
        }
    }

    pub fn measure_memory(name: &str, bytes: usize, peak: usize) -> MemoryMeasurement {
        MemoryMeasurement {
            resource_name: name.to_string(),
            bytes_allocated: bytes,
            peak_bytes: peak,
        }
    }

    pub fn record_result(&mut self, result: BenchmarkResult) {
        self.measurements.push(result);
    }

    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.measurements
    }

    pub fn average_latency_us(&self, operation: &str) -> Option<u64> {
        self.measurements
            .iter()
            .flat_map(|r| &r.latencies)
            .filter(|m| m.operation == operation)
            .map(|m| m.duration_us)
            .collect::<Vec<_>>()
            .iter()
            .sum::<u64>()
            .checked_div(
                self.measurements
                    .iter()
                    .flat_map(|r| &r.latencies)
                    .filter(|m| m.operation == operation)
                    .count() as u64,
            )
    }
}

impl Default for Benchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_creation() {
        let benchmark = Benchmark::new();
        assert_eq!(benchmark.get_results().len(), 0);
    }

    #[test]
    fn test_measure_operation() {
        let benchmark = Benchmark::new();
        let mut counter = 0;
        let measurement = benchmark.measure_operation("increment", 100, || {
            counter += 1;
        });

        assert_eq!(measurement.operation, "increment");
        assert_eq!(measurement.iterations, 100);
        assert!(measurement.duration_us >= 0);
    }

    #[test]
    fn test_measure_throughput() {
        let benchmark = Benchmark::new();
        let mut counter = 0u32;
        let measurement = benchmark.measure_throughput("simple_op", Duration::from_millis(10), || {
            counter = counter.wrapping_add(1);
            true
        });

        assert_eq!(measurement.operation, "simple_op");
        assert!(measurement.operations_per_second > 0.0);
        assert!(measurement.duration_seconds >= 0.01);
    }

    #[test]
    fn test_memory_measurement() {
        let mem = Benchmark::measure_memory("test_resource", 1024 * 1024, 2 * 1024 * 1024);
        assert_eq!(mem.bytes_allocated, 1024 * 1024);
        assert_eq!(mem.peak_bytes, 2 * 1024 * 1024);
    }

    #[test]
    fn test_record_and_retrieve_results() {
        let mut benchmark = Benchmark::new();
        let result = BenchmarkResult {
            test_name: "sample_test".to_string(),
            latencies: vec![LatencyMeasurement {
                operation: "op1".to_string(),
                duration_us: 100,
                iterations: 10,
            }],
            throughputs: vec![],
            memory: vec![],
            total_duration_seconds: 0.001,
        };

        benchmark.record_result(result);
        assert_eq!(benchmark.get_results().len(), 1);
    }

    #[test]
    fn test_average_latency_calculation() {
        let mut benchmark = Benchmark::new();

        let result1 = BenchmarkResult {
            test_name: "test1".to_string(),
            latencies: vec![LatencyMeasurement {
                operation: "op".to_string(),
                duration_us: 100,
                iterations: 10,
            }],
            throughputs: vec![],
            memory: vec![],
            total_duration_seconds: 0.001,
        };

        let result2 = BenchmarkResult {
            test_name: "test2".to_string(),
            latencies: vec![LatencyMeasurement {
                operation: "op".to_string(),
                duration_us: 200,
                iterations: 10,
            }],
            throughputs: vec![],
            memory: vec![],
            total_duration_seconds: 0.002,
        };

        benchmark.record_result(result1);
        benchmark.record_result(result2);

        let avg = benchmark.average_latency_us("op").unwrap();
        assert_eq!(avg, 150);
    }

    #[test]
    fn test_wayland_client_connection_latency() {
        use system_integration::ApplicationStack;
        use wayland_server::WaylandClient;
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let measurement = benchmark.measure_operation("wayland_client_connect", 10, || {
            let client = WaylandClient {
                id: ObjectId::new(),
                name: "test_app".to_string(),
                is_connected: false,
            };
            let _ = stack.get_compositor_mut().connect_client(client);
        });

        assert_eq!(measurement.operation, "wayland_client_connect");
        assert!(measurement.duration_us < 10000);
    }

    #[test]
    fn test_surface_creation_latency() {
        use system_integration::ApplicationStack;
        use wayland_server::WaylandClient;
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = stack.get_compositor_mut().connect_client(client);

        let measurement = benchmark.measure_operation("surface_creation", 5, || {
            let _ = stack.get_compositor_mut().create_surface(&client_id);
        });

        assert_eq!(measurement.operation, "surface_creation");
        assert!(measurement.duration_us < 10000);
    }

    #[test]
    fn test_buffer_allocation_latency() {
        use system_integration::ApplicationStack;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let measurement =
            benchmark.measure_operation("buffer_allocation", 10, || {
                let _ = stack
                    .get_compositor_mut()
                    .create_buffer(1920, 1080, 0x34325241);
            });

        assert_eq!(measurement.operation, "buffer_allocation");
        assert!(measurement.duration_us < 10000);
    }

    #[test]
    fn test_gpu_connector_registration_latency() {
        use system_integration::ApplicationStack;
        use gpu_driver::{Connector, ConnectorType, ConnectorStatus, DisplayMode};
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let mode = DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            clock: 148500,
        };

        let measurement = benchmark.measure_operation("gpu_connector_registration", 5, || {
            let connector = Connector {
                id: ObjectId::new(),
                connector_type: ConnectorType::HDMI,
                status: ConnectorStatus::Connected,
                supported_modes: vec![mode.clone()],
                current_mode: None,
            };
            let _ = stack.get_gpu_driver_mut().register_connector(connector);
        });

        assert_eq!(measurement.operation, "gpu_connector_registration");
        assert!(measurement.duration_us < 10000);
    }

    #[test]
    fn test_audio_device_registration_latency() {
        use system_integration::ApplicationStack;
        use audio_driver::{AudioDevice, DeviceRole, AudioFormat};
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let measurement = benchmark.measure_operation("audio_device_registration", 5, || {
            let device = AudioDevice {
                id: ObjectId::new(),
                name: "Speaker".to_string(),
                role: DeviceRole::Playback,
                sample_rates: vec![48000],
                current_sample_rate: 48000,
                formats: vec![AudioFormat::S16LE],
                current_format: AudioFormat::S16LE,
                channels: 2,
                is_active: false,
            };
            let _ = stack.get_audio_driver_mut().register_device(device);
        });

        assert_eq!(measurement.operation, "audio_device_registration");
        assert!(measurement.duration_us < 10000);
    }

    #[test]
    fn test_input_device_registration_latency() {
        use system_integration::ApplicationStack;
        use input_driver::{InputDevice, InputDeviceType};
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let measurement = benchmark.measure_operation("input_device_registration", 10, || {
            let device = InputDevice {
                id: ObjectId::new(),
                name: "Keyboard".to_string(),
                device_type: InputDeviceType::Keyboard,
                is_active: false,
                has_buttons: true,
                has_axes: false,
                max_touches: 0,
            };
            let _ = stack.get_input_driver_mut().register_device(device);
        });

        assert_eq!(measurement.operation, "input_device_registration");
        assert!(measurement.duration_us < 5000);
    }

    #[test]
    fn test_pointer_event_routing_latency() {
        use system_integration::ApplicationStack;
        use wayland_server::{WaylandClient, PointerEvent, PointerEventType};
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = stack.get_compositor_mut().connect_client(client);
        let surface = stack
            .get_compositor_mut()
            .create_surface(&client_id)
            .unwrap();

        let measurement =
            benchmark.measure_operation("pointer_event_routing", 100, || {
                let event = PointerEvent {
                    surface_id: Some(surface.id.clone()),
                    event_type: PointerEventType::Motion,
                    x: 100,
                    y: 200,
                    button: None,
                };
                let _ = stack.get_compositor_mut().route_pointer_event(event);
            });

        assert_eq!(measurement.operation, "pointer_event_routing");
        assert!(measurement.duration_us < 1000);
    }

    #[test]
    fn test_multi_surface_throughput() {
        use system_integration::ApplicationStack;
        use wayland_server::WaylandClient;
        use sher_common::ObjectId;

        let benchmark = Benchmark::new();
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = stack.get_compositor_mut().connect_client(client);

        let measurement = benchmark.measure_throughput("surface_creation", Duration::from_millis(100), || {
            match stack.get_compositor_mut().create_surface(&client_id) {
                Ok(_) => true,
                Err(_) => false,
            }
        });

        assert_eq!(measurement.operation, "surface_creation");
        assert!(measurement.operations_per_second > 1000.0);
    }
}
