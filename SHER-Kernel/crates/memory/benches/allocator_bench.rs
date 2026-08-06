// SHER Kernel Memory Allocator Benchmark Suite
// Phase 1 Week 1: Benchmark Framework
// Status: Skeleton - Ready for Week 2 Implementation
//
// This file contains the benchmark harness for testing allocation performance
// against Linux baselines and measuring improvement targets.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::alloc::{alloc, dealloc, Layout};
use std::time::Instant;

// ============================================================================
// BENCHMARK 1: Allocation Speed by Size Class
// ============================================================================

fn benchmark_allocation_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_latency");

    // Test sizes: 8B, 16B, 32B, 64B, 128B, 256B, 512B, 1KB, 8KB, 64KB
    let test_sizes = vec![
        ("8B", 8),
        ("16B", 16),
        ("32B", 32),
        ("64B", 64),
        ("128B", 128),
        ("256B", 256),
        ("512B", 512),
        ("1KB", 1024),
        ("8KB", 8192),
        ("64KB", 65536),
    ];

    for (name, size) in test_sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            b.iter(|| {
                let layout = Layout::from_size_align(size, 8).unwrap();
                unsafe {
                    let ptr = alloc(layout);
                    dealloc(ptr, layout);
                }
            })
        });
    }

    group.finish();
}

// ============================================================================
// BENCHMARK 2: Deallocation Speed by Size Class
// ============================================================================

fn benchmark_deallocation_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("deallocation_latency");

    let test_sizes = vec![
        ("8B", 8),
        ("64B", 64),
        ("256B", 256),
        ("1KB", 1024),
        ("8KB", 8192),
    ];

    for (name, size) in test_sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            b.iter_batched(
                || {
                    // Setup: allocate
                    let layout = Layout::from_size_align(size, 8).unwrap();
                    unsafe { alloc(layout) }
                },
                |ptr| {
                    // Benchmark: deallocate
                    let layout = Layout::from_size_align(size, 8).unwrap();
                    unsafe { dealloc(ptr, layout) }
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

// ============================================================================
// BENCHMARK 3: Allocation + Deallocation Cycle
// ============================================================================

fn benchmark_alloc_free_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc_free_cycle");

    let test_sizes = vec![
        ("64B", 64),
        ("1KB", 1024),
        ("64KB", 65536),
    ];

    for (name, size) in test_sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            b.iter(|| {
                let layout = Layout::from_size_align(size, 8).unwrap();
                unsafe {
                    let ptr = black_box(alloc(layout));
                    black_box(dealloc(ptr, layout));
                }
            })
        });
    }

    group.finish();
}

// ============================================================================
// BENCHMARK 4: Per-CPU Cache Efficiency (Single Thread)
// ============================================================================

fn benchmark_per_cpu_cache_hit(c: &mut Criterion) {
    c.bench_function("per_cpu_cache_hit_1000_allocs", |b| {
        b.iter(|| {
            let layout = Layout::from_size_align(64, 8).unwrap();
            for _ in 0..1000 {
                unsafe {
                    let ptr = alloc(layout);
                    dealloc(ptr, layout);
                }
            }
        })
    });
}

// ============================================================================
// BENCHMARK 5: Fragmentation Test
// ============================================================================

fn benchmark_fragmentation(c: &mut Criterion) {
    c.bench_function("fragmentation_1000_varied_allocs", |b| {
        b.iter(|| {
            let mut ptrs = Vec::new();

            // Allocate varied sizes
            for i in 0..1000 {
                let size = 64 + (i % 256) * 8;  // Varied sizes to induce fragmentation
                let layout = Layout::from_size_align(size, 8).unwrap();
                unsafe {
                    ptrs.push((alloc(layout), layout));
                }
            }

            // Free in random order (worst case for fragmentation)
            for (ptr, layout) in ptrs {
                unsafe {
                    dealloc(ptr, layout);
                }
            }
        })
    });
}

// ============================================================================
// BENCHMARK 6: Contention Under Multiple Threads (Simulated)
// ============================================================================

fn benchmark_contention_single_size_class(c: &mut Criterion) {
    // Note: Criterion doesn't support multi-threaded benchmarks directly
    // This is a placeholder for manual multi-threaded testing

    c.bench_function("single_thread_1KB_allocs", |b| {
        b.iter(|| {
            let layout = Layout::from_size_align(1024, 8).unwrap();
            for _ in 0..100 {
                unsafe {
                    let ptr = alloc(layout);
                    dealloc(ptr, layout);
                }
            }
        })
    });
}

// ============================================================================
// BENCHMARK 7: Large Allocation Performance
// ============================================================================

fn benchmark_large_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_allocations");

    let test_sizes = vec![
        ("256KB", 262144),
        ("1MB", 1048576),
        ("16MB", 16777216),
    ];

    for (name, size) in test_sizes {
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            b.iter(|| {
                let layout = Layout::from_size_align(size, 4096).unwrap();
                unsafe {
                    let ptr = alloc(layout);
                    dealloc(ptr, layout);
                }
            })
        });
    }

    group.finish();
}

// ============================================================================
// BENCHMARK 8: Throughput Test (Operations Per Second)
// ============================================================================

fn benchmark_allocation_throughput(c: &mut Criterion) {
    c.bench_function("allocation_throughput_64B_1sec", |b| {
        b.iter(|| {
            let layout = Layout::from_size_align(64, 8).unwrap();
            let start = Instant::now();
            let mut count = 0;

            while start.elapsed().as_secs_f64() < 0.1 {  // 100ms test
                unsafe {
                    let ptr = alloc(layout);
                    dealloc(ptr, layout);
                    count += 1;
                }
            }

            black_box(count)
        })
    });
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    benches,
    benchmark_allocation_latency,
    benchmark_deallocation_latency,
    benchmark_alloc_free_cycle,
    benchmark_per_cpu_cache_hit,
    benchmark_fragmentation,
    benchmark_contention_single_size_class,
    benchmark_large_allocations,
    benchmark_allocation_throughput,
);

criterion_main!(benches);

// ============================================================================
// MEASUREMENT NOTES
// ============================================================================
//
// To run benchmarks:
//   $ cd crates/memory
//   $ cargo bench
//
// To run specific benchmark:
//   $ cargo bench allocation_latency
//
// To compare with baseline:
//   $ cargo bench -- --baseline=linux
//
// These benchmarks establish baselines against which SHER implementation
// will be measured. Target: 10x improvement on most metrics.
//
// During Phase 1 Week 2, implement SHER allocator and re-run these
// benchmarks to measure improvement against Linux baselines.
