use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_device_manager::manager::DeviceManager;
use std::collections::HashMap;

fn benchmark_device_registration(c: &mut Criterion) {
    let mut manager = DeviceManager::new();

    c.bench_function("register_single_device", |b| {
        b.iter(|| {
            let _ = manager.register_device(
                black_box("pci:0:1:0"),
                black_box("Intel Network Controller"),
            );
        })
    });
}

fn benchmark_device_lookup(c: &mut Criterion) {
    let mut manager = DeviceManager::new();

    // Register 100 devices
    for i in 0..100 {
        let _ = manager.register_device(&format!("pci:0:1:{}", i), "Test Device");
    }

    c.bench_function("lookup_device_hashmap", |b| {
        b.iter(|| {
            let _ = manager.get_device(black_box("pci:0:1:50"));
        })
    });
}

fn benchmark_device_enumeration(c: &mut Criterion) {
    let mut manager = DeviceManager::new();

    // Register multiple devices
    for i in 0..1000 {
        let _ = manager.register_device(&format!("pci:0:1:{}", i), "Test Device");
    }

    c.bench_function("enumerate_1000_devices", |b| {
        b.iter(|| {
            let devices = manager.enumerate_all();
            black_box(devices);
        })
    });
}

fn benchmark_driver_matching(c: &mut Criterion) {
    let mut manager = DeviceManager::new();

    // Register devices
    for i in 0..100 {
        let _ = manager.register_device(&format!("pci:0:1:{}", i), "Test Device");
    }

    c.bench_function("match_drivers_100_devices", |b| {
        b.iter(|| {
            let _ = manager.match_drivers();
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
