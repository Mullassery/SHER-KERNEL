use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sher_common::ObjectId;
use sher_lki::audit::{AuditEntry, AuditLog};
use sher_lki::enforcement::SecurityEnforcer;
use sher_lki::security::{
    Capability, CapabilityGrant, PermissionTier, SecurityLevel, SecurityPolicy,
};

fn setup_enforcer_with_grant(capability: Capability) -> (SecurityEnforcer, ObjectId, ObjectId) {
    let driver_id = ObjectId::new();
    let policy = SecurityPolicy::new(driver_id, SecurityLevel::Balanced);
    let mut enforcer = SecurityEnforcer::new();
    let context_id = enforcer.register_driver(driver_id, policy).unwrap();

    let grant = CapabilityGrant::new(driver_id, capability, PermissionTier::Low);
    enforcer
        .get_context_mut(context_id)
        .unwrap()
        .capability_manager
        .grant(grant)
        .unwrap();

    (enforcer, driver_id, context_id)
}

fn benchmark_capability_check(c: &mut Criterion) {
    let (mut enforcer, _driver_id, context_id) = setup_enforcer_with_grant(Capability::ReadMemory);

    c.bench_function("capability_check_hit", |b| {
        b.iter(|| {
            let _ = enforcer.enforce(black_box(context_id), black_box(Capability::ReadMemory), 0);
        })
    });
}

fn benchmark_capability_check_miss(c: &mut Criterion) {
    let mut enforcer = SecurityEnforcer::new();
    let unknown_context = ObjectId::new();

    c.bench_function("capability_check_miss", |b| {
        b.iter(|| {
            let _ = enforcer.enforce(
                black_box(unknown_context),
                black_box(Capability::ModifyPolicy),
                0,
            );
        })
    });
}

fn benchmark_multiple_capability_checks(c: &mut Criterion) {
    let driver_id = ObjectId::new();
    let policy = SecurityPolicy::new(driver_id, SecurityLevel::Balanced);
    let mut enforcer = SecurityEnforcer::new();
    let context_id = enforcer.register_driver(driver_id, policy).unwrap();

    let caps = [
        Capability::ReadMemory,
        Capability::WriteMemory,
        Capability::NetworkAccess,
    ];
    for cap in caps {
        let grant = CapabilityGrant::new(driver_id, cap, PermissionTier::Medium);
        enforcer
            .get_context_mut(context_id)
            .unwrap()
            .capability_manager
            .grant(grant)
            .unwrap();
    }

    c.bench_function("multiple_capability_checks", |b| {
        b.iter(|| {
            for cap in caps {
                let _ = enforcer.enforce(black_box(context_id), black_box(cap), 0);
            }
        })
    });
}

fn benchmark_expiration_check(c: &mut Criterion) {
    // A Low-tier grant recommends a 1h lifetime; checking right at the edge
    // of that window exercises the same expiration-comparison path enforce()
    // always takes, just with a value chosen to land close to the boundary.
    let (mut enforcer, _driver_id, context_id) = setup_enforcer_with_grant(Capability::ReadMemory);
    let near_expiry_ms = PermissionTier::Low.recommended_duration_ms() - 1;

    c.bench_function("expiration_check_per_capability", |b| {
        b.iter(|| {
            let _ = enforcer.enforce(
                black_box(context_id),
                black_box(Capability::ReadMemory),
                black_box(near_expiry_ms),
            );
        })
    });
}

fn benchmark_audit_logging(c: &mut Criterion) {
    let mut log = AuditLog::new(10_000);
    let driver_id = ObjectId::new();

    c.bench_function("audit_log_entry", |b| {
        b.iter(|| {
            log.log(
                AuditEntry::new(
                    black_box(driver_id),
                    black_box("kmalloc"),
                    black_box("allocate"),
                )
                .with_result("granted"),
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
