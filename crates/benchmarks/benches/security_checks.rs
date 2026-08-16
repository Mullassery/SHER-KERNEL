use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_security::capability::{Capability, CapabilityGrant, PermissionTier};
use sher_security::enforcer::SecurityEnforcer;
use std::time::{SystemTime, UNIX_EPOCH};

fn benchmark_capability_check(c: &mut Criterion) {
    let mut enforcer = SecurityEnforcer::new();
    let driver_id = "driver_001".to_string();

    let grant = CapabilityGrant {
        capability: Capability::Read,
        tier: PermissionTier::Low,
        granted_at: 1000,
        expires_at: Some(2000),
    };
    enforcer.grant_capability(driver_id.clone(), grant).unwrap();

    c.bench_function("capability_check_hit", |b| {
        b.iter(|| {
            let _ = enforcer.check_capability(black_box(&driver_id), black_box(&Capability::Read));
        })
    });
}

fn benchmark_capability_check_miss(c: &mut Criterion) {
    let enforcer = SecurityEnforcer::new();

    c.bench_function("capability_check_miss", |b| {
        b.iter(|| {
            let _ = enforcer
                .check_capability(black_box(&"unknown_driver"), black_box(&Capability::Admin));
        })
    });
}

fn benchmark_multiple_capability_checks(c: &mut Criterion) {
    let mut enforcer = SecurityEnforcer::new();
    let driver_id = "driver_001".to_string();

    for cap in &[Capability::Read, Capability::Write, Capability::Execute] {
        let grant = CapabilityGrant {
            capability: cap.clone(),
            tier: PermissionTier::Medium,
            granted_at: 1000,
            expires_at: Some(5000),
        };
        let _ = enforcer.grant_capability(driver_id.clone(), grant);
    }

    c.bench_function("multiple_capability_checks", |b| {
        b.iter(|| {
            for cap in &[Capability::Read, Capability::Write, Capability::Execute] {
                let _ = enforcer.check_capability(black_box(&driver_id), black_box(cap));
            }
        })
    });
}

fn benchmark_expiration_check(c: &mut Criterion) {
    let mut enforcer = SecurityEnforcer::new();
    let driver_id = "driver_001".to_string();

    let grant = CapabilityGrant {
        capability: Capability::Read,
        tier: PermissionTier::Low,
        granted_at: 1000,
        expires_at: Some(2000),
    };
    enforcer.grant_capability(driver_id.clone(), grant).unwrap();

    c.bench_function("expiration_check_per_capability", |b| {
        b.iter(|| {
            let _ = enforcer.check_expiration(black_box(&driver_id), black_box(&Capability::Read));
        })
    });
}

fn benchmark_audit_logging(c: &mut Criterion) {
    let mut enforcer = SecurityEnforcer::new();

    c.bench_function("audit_log_entry", |b| {
        b.iter(|| {
            enforcer.log_access_check(
                black_box("driver_001"),
                black_box(&Capability::Read),
                black_box(true),
            );
        })
    });
}

criterion_group!(
    benches,
    benchmark_capability_check,
    benchmark_capability_check_miss,
    benchmark_multiple_capability_checks,
    benchmark_expiration_check,
    benchmark_audit_logging
);
criterion_main!(benches);
