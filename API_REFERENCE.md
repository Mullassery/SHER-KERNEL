# SHER Kernel - API Reference

**Version**: 1.0.0  
**Target**: Rust 1.70+

---

## Core Crates

### sher_common

Core types and utilities shared across the system.

```rust
use sher_common::{ObjectId, Result, Error};

// Create unique identifiers
let id = ObjectId::new();

// Error handling
fn example() -> Result<String> {
    Err(Error::Device("Not found".to_string()))
}
```

### sher_kernel

Main kernel implementation with phases 0-10.

```rust
use sher_kernel::KernelObject;

// All kernel entities use the unified object model
let obj = KernelObject::new("process", "app_1");
```

---

## Phase 11: Hardware Integration

### HAL (Hardware Abstraction Layer)

```rust
use hal::{HardwareAbstractionLayer, DeviceType, DeviceInfo};

let mut hal = HardwareAbstractionLayer::new();

// Probe devices
let devices = hal.probe_devices()?;

// Get device by type
let gpus = hal.get_devices_by_type(DeviceType::Gpu);

// Map memory
let addr = hal.map_memory(&device_id, 0x1000, 4096)?;

// Register read/write
let value = hal.read_register(&device_id, 0)?;
hal.write_register(&device_id, 0, 0x1234)?;
```

### GPU Driver

```rust
use gpu_driver::{GPUDriver, Connector, DisplayMode, ConnectorType};

let mut driver = GPUDriver::new(256 * 1024 * 1024);

// Register connector
let connector = Connector {
    id: ObjectId::new(),
    connector_type: ConnectorType::HDMI,
    status: ConnectorStatus::Connected,
    supported_modes: vec![/* modes */],
    current_mode: None,
};
driver.register_connector(connector)?;

// Set display mode
driver.set_mode(&connector_id, mode)?;

// Allocate framebuffer
let fb = driver.allocate_framebuffer(1920, 1080)?;

// Page flip
driver.page_flip(&connector_id, &fb.id)?;
```

### Audio Driver

```rust
use audio_driver::{AudioDriver, AudioDevice, DeviceRole, AudioFormat};

let mut driver = AudioDriver::new();

// Register device
let device = AudioDevice {
    id: ObjectId::new(),
    name: "Speaker".to_string(),
    role: DeviceRole::Playback,
    sample_rates: vec![44100, 48000],
    current_sample_rate: 48000,
    formats: vec![AudioFormat::S16LE],
    current_format: AudioFormat::S16LE,
    channels: 2,
    is_active: false,
};
driver.register_device(device)?;

// Allocate buffer
let buffer = driver.allocate_buffer(&device_id, 4096)?;

// Start stream
driver.start_stream(&device_id)?;

// Stream operations
driver.write_buffer(&buffer.id, 2048)?;

// Stop stream
driver.stop_stream(&device_id)?;
```

### Input Driver

```rust
use input_driver::{InputDriver, InputDevice, InputDeviceType, KeyEvent, KeyEventType};

let mut driver = InputDriver::new();

// Register device
let keyboard = InputDevice {
    id: ObjectId::new(),
    name: "Keyboard".to_string(),
    device_type: InputDeviceType::Keyboard,
    is_active: false,
    has_buttons: true,
    has_axes: false,
    max_touches: 0,
};
driver.register_device(keyboard)?;

// Activate
driver.activate_device(&keyboard_id)?;

// Queue events
let event = KeyEvent {
    keycode: 30,
    event_type: KeyEventType::Press,
    timestamp: 1000,
};
driver.queue_key_event(event)?;

// Get events
if let Some(event) = driver.get_key_event() {
    println!("Key pressed: {}", event.keycode);
}

// Multitouch
driver.start_touch(0, 100, 200, 1000)?;
driver.update_touch(0, 150, 250, 950)?;
driver.end_touch(0)?;
```

---

## Phase 12: System Integration

### Wayland Compositor

```rust
use wayland_server::{WaylandCompositor, WaylandClient, Surface};

let mut compositor = WaylandCompositor::new();
compositor.start()?;

// Client management
let client = WaylandClient {
    id: ObjectId::new(),
    name: "my_app".to_string(),
    is_connected: false,
};
compositor.connect_client(client)?;

// Surface management
let surface = compositor.create_surface(&client_id)?;
compositor.configure_surface(&surface.id, 1920, 1080)?;

// Buffer management
let buffer = compositor.create_buffer(1920, 1080, 0x34325241)?;
compositor.attach_buffer(&surface.id, &buffer.id)?;
compositor.commit_surface(&surface.id)?;

// Input routing
let event = PointerEvent {
    surface_id: Some(surface.id.clone()),
    event_type: PointerEventType::Motion,
    x: 960,
    y: 540,
    button: None,
};
compositor.route_pointer_event(event)?;

compositor.stop()?;
```

### Unified Device Manager

```rust
use unified_device_manager::UnifiedDeviceManager;

let mut manager = UnifiedDeviceManager::new();
manager.initialize()?;

// Register devices
manager.register_gpu_device(device_id, "NVIDIA RTX".to_string())?;
manager.register_audio_device(device_id, "Speaker".to_string())?;
manager.register_input_device(device_id, "Keyboard".to_string())?;

// Device tracking
let gpu = manager.get_gpu_device(&device_id);
let health_count = manager.get_healthy_device_count();

// Error handling
manager.report_device_error(&device_id, "Error occurred".to_string())?;
manager.mark_device_healthy(&device_id)?;

// Hot-plug events
manager.broadcast_hotplug(device_id, true)?;

// Status
let status = manager.all_devices_healthy();
```

---

## Phase 13: Production Hardening

### Security Audit

```rust
use security_audit::{SecurityAudit, InputValidator, ValidationRule, SecurityEvent, ThreatLevel};

// Input validation
let mut validator = InputValidator::new();
let rule = ValidationRule {
    name: "api_input".to_string(),
    max_length: Some(256),
    min_length: Some(1),
    allowed_chars: None,
    reject_patterns: vec!["../".to_string()],
};
validator.add_rule(rule);

if validator.validate("api_input", user_input).is_ok() {
    // Safe to process
}

// Capability control
let mut cv = CapabilityValidator::new();
cv.grant_capability(subject_id, "read_file".to_string(), expiration_time);

if cv.has_capability(&subject_id, "read_file", current_time) {
    // Allow access
}

// Audit logging
let mut audit = SecurityAudit::new();
let event = SecurityEvent {
    event_id: ObjectId::new(),
    timestamp: 12345,
    event_type: "login_attempt".to_string(),
    source: "client_1".to_string(),
    threat_level: ThreatLevel::Low,
    description: "User login".to_string(),
    remediation: None,
};
audit.log_event(event);

let score = audit.get_threat_score();
let status = audit.security_status(); // "SECURE", "MEDIUM", "HIGH", "CRITICAL"
```

### Performance Optimization

```rust
use performance_optimization::{ObjectPool, Cache, Batch};

// Object pooling
let mut pool: ObjectPool<u32> = ObjectPool::new(10);
let id = pool.acquire(42)?;
pool.release(&id)?;

let utilization = pool.utilization();

// Caching
let mut cache: Cache<String, u32> = Cache::new(100);
cache.put("key".to_string(), 100, current_time);

if let Some(value) = cache.get(&"key".to_string(), current_time) {
    println!("Cache hit: {}", value);
}

let hit_rate = cache.hit_rate();

// Batch processing
let mut batch: Batch<u32> = Batch::new(32);
for item in items {
    if batch.add(item) {
        if batch.is_full() {
            let flushed = batch.flush();
            process_batch(flushed);
        }
    }
}

if !batch.is_empty() {
    process_batch(batch.flush());
}
```

### Release Engineering

```rust
use release_engineering::{ReleaseManager, Version, ChangeLogEntry, ChangeType, ReleaseArtifact, ReleaseStatus};

let mut manager = ReleaseManager::new(1, 0, 0);

// Version management
let new_version = manager.get_version().bump_minor();
manager.set_version(new_version);

// Changelog
let entry = ChangeLogEntry {
    version: Version::new(1, 1, 0),
    date: "2026-08-07".to_string(),
    change_type: ChangeType::Feature,
    description: "New feature".to_string(),
    breaking: false,
};
manager.add_changelog_entry(entry);

// Quality gates
manager.set_quality_gate("tests_passing".to_string(), true);
manager.set_quality_gate("security_audit".to_string(), true);

// Release artifacts
let artifact = ReleaseArtifact {
    version: Version::new(1, 1, 0),
    artifact_type: "binary".to_string(),
    size_bytes: 1024 * 1024,
    checksum: "abc123".to_string(),
    download_url: "https://releases.example.com/sher-1.1.0".to_string(),
};
manager.register_artifact(artifact);

// Release readiness
if manager.is_release_ready() {
    manager.set_status(ReleaseStatus::Stable);
}
```

---

## Error Handling

```rust
use sher_common::{Result, Error};

// Common error types
fn example() -> Result<()> {
    // Device errors
    Err(Error::Device("Device not found".to_string()))?;
    
    // Memory errors
    Err(Error::Memory("Allocation failed".to_string()))?;
    
    // Security errors
    Err(Error::Security("Unauthorized access".to_string()))?;
    
    // Generic errors
    Err(Error::Other("Unknown error".to_string()))?;
    
    Ok(())
}

// Pattern matching
match some_operation() {
    Ok(value) => println!("Success: {}", value),
    Err(Error::Device(msg)) => eprintln!("Device error: {}", msg),
    Err(Error::Memory(msg)) => eprintln!("Memory error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

## Testing

```rust
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = AudioDevice { /* ... */ };
        assert_eq!(device.channels, 2);
    }
}

// Integration tests
#[test]
fn test_full_workflow() {
    let mut stack = ApplicationStack::new();
    assert!(stack.initialize().is_ok());
    assert!(stack.shutdown().is_ok());
}

// Run tests
// cargo test --lib
// cargo test --lib --package security_audit
// cargo test --lib -- --nocapture
```

---

## Performance APIs

```rust
use performance_benchmarks::{Benchmark, LatencyMeasurement, ThroughputMeasurement};

let benchmark = Benchmark::new();

// Measure latency
let measurement = benchmark.measure_operation("operation", 100, || {
    // operation
});
println!("Latency: {} µs", measurement.duration_us);

// Measure throughput
let measurement = benchmark.measure_throughput("operation", Duration::from_secs(1), || {
    // return true if operation succeeded
    true
});
println!("Throughput: {}/s", measurement.operations_per_second);
```

---

## Full Example

```rust
use system_integration::ApplicationStack;
use wayland_server::WaylandClient;
use sher_common::ObjectId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize stack
    let mut stack = ApplicationStack::new();
    stack.initialize()?;

    // Create client
    let client = WaylandClient {
        id: ObjectId::new(),
        name: "demo_app".to_string(),
        is_connected: false,
    };
    
    let client_id = client.id.clone();
    stack.get_compositor_mut().connect_client(client)?;

    // Create surface
    let surface = stack.get_compositor_mut().create_surface(&client_id)?;

    // Allocate buffer
    let buffer = stack.get_compositor_mut().create_buffer(1920, 1080, 0x34325241)?;

    // Attach and commit
    stack.get_compositor_mut().attach_buffer(&surface.id, &buffer.id)?;
    stack.get_compositor_mut().commit_surface(&surface.id)?;

    println!("Application created successfully!");

    // Shutdown
    stack.shutdown()?;
    Ok(())
}
```

---

**SHER Kernel API v1.0.0**  
*Complete API Reference for Production Use*
