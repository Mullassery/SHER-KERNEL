use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_common::ObjectId;
use sher_device_manager::policy::{DriverDatabase, DriverEntry, DriverMatcher, DriverPolicy};
use sher_device_manager::registry::{DeviceRegistry, RegisteredDevice};

fn make_device(name: &str) -> RegisteredDevice {
    RegisteredDevice::new(ObjectId::new(), name.to_string(), "pci".to_string(), 0)
}

fn benchmark_device_registration(c: &mut Criterion) {
    let mut registry = DeviceRegistry::new();

    c.bench_function("register_single_device", |b| {
        b.iter(|| {
            registry.register(black_box(make_device("Intel Network Controller")));
        })
    });
}

fn benchmark_device_lookup(c: &mut Criterion) {
    let mut registry = DeviceRegistry::new();
    let mut ids = Vec::new();

    // Register 100 devices
    for i in 0..100 {
        let device = make_device(&format!("Test Device {}", i));
        ids.push(device.id);
        registry.register(device);
    }
    let target = ids[50];

    c.bench_function("lookup_device_by_id", |b| {
        b.iter(|| {
            let _ = registry.get_device(black_box(target));
        })
    });
}

fn benchmark_device_enumeration(c: &mut Criterion) {
    let mut registry = DeviceRegistry::new();

    // Register multiple devices
    for i in 0..1000 {
        registry.register(make_device(&format!("Test Device {}", i)));
    }

    c.bench_function("enumerate_1000_devices", |b| {
        b.iter(|| {
            let devices = registry.list_devices();
            black_box(devices);
        })
    });
}

fn benchmark_driver_matching(c: &mut Criterion) {
    let mut matcher = DriverMatcher::new(DriverPolicy::default());
    let mut database = DriverDatabase::new();

    // Register 100 drivers, each targeting a distinct vendor/device pair
    for i in 0..100u16 {
        database.register_driver(DriverEntry {
            id: format!("driver-{}", i),
            name: format!("Driver {}", i),
            vendor_id: Some(0x8086),
            device_id: Some(i),
            device_class: Some(0x02),
            device_subclass: Some(0x00),
            native: true,
            version: "1.0".to_string(),
            required_capabilities: vec![],
        });
    }
    matcher.database = database;

    c.bench_function("match_drivers_100_devices", |b| {
        b.iter(|| {
            for i in 0..100u16 {
                let _ = matcher.find_best_match(black_box(0x8086), black_box(i), 0x02, 0x00);
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_device_registration,
    benchmark_device_lookup,
    benchmark_device_enumeration,
    benchmark_driver_matching
);
criterion_main!(benches);
