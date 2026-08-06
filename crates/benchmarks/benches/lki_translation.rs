use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_lki::interface::LinuxKernelInterface;
use std::sync::Arc;

fn benchmark_kmalloc_translation(c: &mut Criterion) {
    let mut lki = LinuxKernelInterface::new();
    lki.initialize().unwrap();

    c.bench_function("lki_kmalloc_256bytes", |b| {
        b.iter(|| {
            let _ = lki.kmalloc(black_box(256), black_box(0));
        })
    });
}

fn benchmark_kmalloc_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("lki_allocation_sizes");
    let mut lki = LinuxKernelInterface::new();
    lki.initialize().unwrap();

    for size in [256, 1024, 4096, 8192].iter() {
        group.bench_with_input(
            format!("kmalloc_{}", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _ = lki.kmalloc(black_box(size), black_box(0));
                })
            },
        );
    }
    group.finish();
}

fn benchmark_kfree_translation(c: &mut Criterion) {
    let mut lki = LinuxKernelInterface::new();
    lki.initialize().unwrap();

    let ptr = lki.kmalloc(256, 0).unwrap();
    c.bench_function("lki_kfree", |b| {
        b.iter(|| {
            let _ = lki.kfree(black_box(ptr));
        })
    });
}

fn benchmark_validation_overhead(c: &mut Criterion) {
    let mut lki = LinuxKernelInterface::new();
    lki.initialize().unwrap();

    c.bench_function("lki_validation_per_call", |b| {
        b.iter(|| {
            let _ = lki.kmalloc(black_box(256), black_box(0));
        })
    });
}

fn benchmark_request_irq_translation(c: &mut Criterion) {
    let mut lki = LinuxKernelInterface::new();
    lki.initialize().unwrap();

    c.bench_function("lki_request_irq_translation", |b| {
        b.iter(|| {
            let _ = lki.request_irq(black_box(32), None, black_box(0));
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
