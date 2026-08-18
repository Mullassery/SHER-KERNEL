use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_memory::allocator::MemoryAllocator;

fn benchmark_small_allocation(c: &mut Criterion) {
    let mut allocator = MemoryAllocator::new(1024 * 1024);

    c.bench_function("allocate_256_bytes", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(256)).unwrap();
            allocator.deallocate(256).unwrap();
        })
    });
}

fn benchmark_large_allocation(c: &mut Criterion) {
    let mut allocator = MemoryAllocator::new(10 * 1024 * 1024);

    c.bench_function("allocate_4096_bytes", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(4096)).unwrap();
            allocator.deallocate(4096).unwrap();
        })
    });
}

fn benchmark_allocation_sequence(c: &mut Criterion) {
    let mut allocator = MemoryAllocator::new(10 * 1024 * 1024);

    c.bench_function("allocate_deallocate_sequence", |b| {
        b.iter(|| {
            for _ in 0..100 {
                allocator.allocate(black_box(256)).unwrap();
            }
            for _ in 0..100 {
                allocator.deallocate(256).unwrap();
            }
        })
    });
}

fn linux_baseline_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("linux_comparison");
    let mut allocator = MemoryAllocator::new(10 * 1024 * 1024);

    group.bench_function("sher_256b_allocation", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(256));
            allocator.deallocate(256).unwrap();
        })
    });

    group.bench_function("sher_4k_allocation", |b| {
        b.iter(|| {
            let _ = allocator.allocate(black_box(4096));
            allocator.deallocate(4096).unwrap();
        })
    });

    group.bench_function("sher_usage_percent", |b| {
        b.iter(|| {
            allocator.allocate(black_box(256)).unwrap();
            let _ = black_box(allocator.usage_percent());
            allocator.deallocate(256).unwrap();
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
