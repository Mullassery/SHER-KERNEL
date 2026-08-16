use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sher_memory::allocator::MemoryAllocator;
use std::time::Instant;

fn benchmark_small_allocation(c: &mut Criterion) {
    let mut allocator = MemoryAllocator::new();
    allocator.initialize(1024 * 1024).unwrap();

    c.bench_function("allocate_256_bytes", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(256)).unwrap();
        })
    });
}

fn benchmark_large_allocation(c: &mut Criterion) {
    let mut allocator = MemoryAllocator::new();
    allocator.initialize(10 * 1024 * 1024).unwrap();

    c.bench_function("allocate_4096_bytes", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(4096)).unwrap();
        })
    });
}

fn benchmark_allocation_sequence(c: &mut Criterion) {
    let mut allocator = MemoryAllocator::new();
    allocator.initialize(10 * 1024 * 1024).unwrap();

    c.bench_function("allocate_deallocate_sequence", |b| {
        b.iter(|| {
            let ptrs: Vec<_> = (0..100)
                .map(|_| allocator.allocate(black_box(256)).unwrap())
                .collect();
            for ptr in ptrs {
                let _ = allocator.deallocate(ptr);
            }
        })
    });
}

fn linux_baseline_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("linux_comparison");
    let mut allocator = MemoryAllocator::new();
    allocator.initialize(10 * 1024 * 1024).unwrap();

    group.bench_function("sher_256b_allocation", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(256));
        })
    });

    group.bench_function("sher_4k_allocation", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(4096));
        })
    });

    group.bench_function("sher_with_validation", |b| {
        b.iter(|| {
            let ptr = allocator.allocate(black_box(256)).unwrap();
            let _ = allocator.validate_allocation(ptr);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_small_allocation,
    benchmark_large_allocation,
    benchmark_allocation_sequence,
    linux_baseline_comparison
);
criterion_main!(benches);
