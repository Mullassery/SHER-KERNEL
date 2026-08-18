use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_common::ObjectId;
use sher_lki::interrupt_translation::{InterruptManager, IrqTrigger};
use sher_lki::memory_translation::LinuxMemoryAllocator;

fn benchmark_kmalloc_translation(c: &mut Criterion) {
    let mut allocator = LinuxMemoryAllocator::new();
    let driver_id = ObjectId::new();

    c.bench_function("lki_kmalloc_256bytes", |b| {
        b.iter(|| {
            let addr = allocator
                .kmalloc(driver_id, black_box(256), black_box(0))
                .unwrap();
            allocator.kfree(addr).unwrap();
        })
    });
}

fn benchmark_kmalloc_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("lki_allocation_sizes");
    let mut allocator = LinuxMemoryAllocator::new();
    let driver_id = ObjectId::new();

    for size in [256, 1024, 4096, 8192].iter() {
        group.bench_with_input(format!("kmalloc_{}", size), size, |b, &size| {
            b.iter(|| {
                let addr = allocator
                    .kmalloc(driver_id, black_box(size), black_box(0))
                    .unwrap();
                allocator.kfree(addr).unwrap();
            })
        });
    }
    group.finish();
}

fn benchmark_kfree_translation(c: &mut Criterion) {
    let mut allocator = LinuxMemoryAllocator::new();
    let driver_id = ObjectId::new();

    c.bench_function("lki_kfree", |b| {
        b.iter(|| {
            let addr = allocator.kmalloc(driver_id, 256, 0).unwrap();
            let _ = allocator.kfree(black_box(addr));
        })
    });
}

fn benchmark_validation_overhead(c: &mut Criterion) {
    let mut allocator = LinuxMemoryAllocator::new();
    let driver_id = ObjectId::new();

    c.bench_function("lki_validation_per_call", |b| {
        b.iter(|| {
            let addr = allocator
                .kmalloc(driver_id, black_box(256), black_box(0))
                .unwrap();
            allocator.kfree(addr).unwrap();
        })
    });
}

fn benchmark_request_irq_translation(c: &mut Criterion) {
    let mut manager = InterruptManager::new();
    let driver_id = ObjectId::new();

    c.bench_function("lki_request_irq_translation", |b| {
        let mut irq = 32u32;
        b.iter(|| {
            irq += 1;
            let _ =
                manager.request_irq(driver_id, black_box(irq), IrqTrigger::Rising, black_box(0));
        })
    });
}

criterion_group!(
    benches,
    benchmark_kmalloc_translation,
    benchmark_kmalloc_sizes,
    benchmark_kfree_translation,
    benchmark_validation_overhead,
    benchmark_request_irq_translation
);
criterion_main!(benches);
