// SHER AI Services: Comprehensive Tests

#[cfg(test)]
mod tests {
    use crate::anomaly_detection::*;
    use crate::predictive_allocation::*;
    use sher_common::ObjectId;

    // ========================================================================
    // ANOMALY DETECTION TESTS
    // ========================================================================

    #[test]
    fn test_memory_leak_detector_new() {
        let detector = MemoryLeakDetector::new();
        assert_eq!(detector.detections, 0);
        assert_eq!(detector.threshold_mb_per_second, 50);
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut detector = MemoryLeakDetector::new();
        let driver_id = ObjectId::new();

        // Record initial state
        detector.record_allocation(driver_id, 100 * 1024 * 1024, 0);

        // Record rapid growth
        detector.record_allocation(driver_id, 6000 * 1024 * 1024, 10_000); // 6GB in 10 seconds = 600MB/s

        let anomaly = detector.detect_leak(driver_id, 10_000);
        assert!(anomaly.is_some());

        let anom = anomaly.unwrap();
        assert_eq!(anom.anomaly_type, AnomalyType::MemoryLeak);
        assert_eq!(anom.severity, AnomalySeverity::Critical);
        assert!(anom.confidence > 0.8);
    }

    #[test]
    fn test_interrupt_storm_detector_new() {
        let detector = InterruptStormDetector::new();
        assert_eq!(detector.detections, 0);
        assert_eq!(detector.threshold_per_second, 10_000);
    }

    #[test]
    fn test_interrupt_storm_detection() {
        let mut detector = InterruptStormDetector::new();

        // Generate 15,000 interrupts with timestamps in same second
        let base_time = 5000u64;
        for i in 0..15_000u32 {
            detector.record_interrupt(32, base_time + (i as u64 % 1000));
        }

        let anomaly = detector.detect_storm(32, base_time + 500);
        assert!(anomaly.is_some());

        let anom = anomaly.unwrap();
        assert_eq!(anom.anomaly_type, AnomalyType::InterruptStorm);
        assert!(anom.severity >= AnomalySeverity::High);
    }

    #[test]
    fn test_dma_abuse_detector_new() {
        let detector = DmaAbuseDetector::new();
        assert_eq!(detector.detections, 0);
        assert_eq!(detector.threshold_concurrent, 100);
    }

    #[test]
    fn test_dma_abuse_detection() {
        let mut detector = DmaAbuseDetector::new();
        let driver_id = ObjectId::new();

        // Record 150 concurrent DMA operations (exceeds threshold of 100)
        // Use 1MB per op to stay under 1GB/s threshold (150 * 1MB = 150MB < 1GB)
        for i in 0..150 {
            detector.record_dma(driver_id, 1 * 1024 * 1024, true, i);
        }

        let anomaly = detector.detect_abuse(driver_id, 150);
        assert!(anomaly.is_some());

        let anom = anomaly.unwrap();
        assert_eq!(anom.anomaly_type, AnomalyType::DmaAbuse);
        assert_eq!(anom.severity, AnomalySeverity::High);
    }

    #[test]
    fn test_anomaly_engine_new() {
        let engine = AnomalyEngine::new();
        assert_eq!(engine.total_anomalies, 0);
        assert_eq!(engine.recent_anomalies.len(), 0);
    }

    #[test]
    fn test_anomaly_engine_memory_detection() {
        let mut engine = AnomalyEngine::new();
        let driver_id = ObjectId::new();

        engine.record_memory_state(driver_id, 100 * 1024 * 1024, 0);
        engine.record_memory_state(driver_id, 6000 * 1024 * 1024, 10_000);

        assert_eq!(engine.total_anomalies, 1);
        assert_eq!(engine.recent_anomalies.len(), 1);
    }

    #[test]
    fn test_anomaly_engine_critical_detection() {
        let mut engine = AnomalyEngine::new();
        let driver_id = ObjectId::new();

        engine.record_memory_state(driver_id, 100 * 1024 * 1024, 0);
        engine.record_memory_state(driver_id, 10_000 * 1024 * 1024, 5_000); // 2GB/s growth

        let critical = engine.get_critical_anomalies();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].severity, AnomalySeverity::Critical);
    }

    #[test]
    fn test_anomaly_engine_filter_by_type() {
        let mut engine = AnomalyEngine::new();
        let driver_id = ObjectId::new();

        // Test memory leak anomaly detection
        engine.record_memory_state(driver_id, 100 * 1024 * 1024, 0);
        engine.record_memory_state(driver_id, 6000 * 1024 * 1024, 10_000);

        let memory_anomalies = engine.get_anomalies_by_type(AnomalyType::MemoryLeak);
        assert!(memory_anomalies.len() > 0);
        assert_eq!(memory_anomalies[0].anomaly_type, AnomalyType::MemoryLeak);
    }

    #[test]
    fn test_anomaly_engine_statistics() {
        let mut engine = AnomalyEngine::new();
        let driver_id = ObjectId::new();

        engine.record_memory_state(driver_id, 100 * 1024 * 1024, 0);
        engine.record_memory_state(driver_id, 6000 * 1024 * 1024, 10_000);

        let stats = engine.get_stats();
        assert_eq!(stats.total_detected, 1);
        assert_eq!(stats.memory_leaks, 1);
    }

    // ========================================================================
    // PREDICTIVE ALLOCATION TESTS
    // ========================================================================

    #[test]
    fn test_predictive_allocator_new() {
        let allocator = PredictiveAllocator::new();
        assert_eq!(allocator.profiles.len(), 0);
        assert_eq!(allocator.prediction_horizon_ms, 1000);
    }

    #[test]
    fn test_update_profile_memory() {
        let mut allocator = PredictiveAllocator::new();
        let driver_id = ObjectId::new();

        let observation = ResourceObservation {
            allocated_bytes: 100 * 1024 * 1024,
            cpu_usage_percent: 50.0,
            io_ops_per_sec: 1000.0,
            network_bandwidth_mbps: 10.0,
            timestamp_ms: 1000,
        };

        allocator.update_profile(driver_id, &observation);

        assert_eq!(allocator.profiles.len(), 1);
        let profile = allocator.profiles.get(&driver_id).unwrap();
        assert_eq!(profile.samples, 1);
        assert_eq!(profile.memory_usage_pattern.peak_allocation_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_profile_learning() {
        let mut allocator = PredictiveAllocator::new();
        let driver_id = ObjectId::new();

        // Simulate multiple observations
        for i in 0..5 {
            let observation = ResourceObservation {
                allocated_bytes: (100 + i * 20) * 1024 * 1024,
                cpu_usage_percent: 50.0 + (i as f64 * 5.0),
                io_ops_per_sec: 1000.0,
                network_bandwidth_mbps: 10.0,
                timestamp_ms: (i as u64 + 1) * 1000,
            };

            allocator.update_profile(driver_id, &observation);
        }

        let profile = allocator.profiles.get(&driver_id).unwrap();
        assert_eq!(profile.samples, 5);
        assert!(profile.confidence > 0.4); // After 5 samples, confidence should increase
    }

    #[test]
    fn test_prediction() {
        let mut allocator = PredictiveAllocator::new();
        let driver_id = ObjectId::new();

        let observation = ResourceObservation {
            allocated_bytes: 500 * 1024 * 1024,
            cpu_usage_percent: 80.0,
            io_ops_per_sec: 5000.0,
            network_bandwidth_mbps: 50.0,
            timestamp_ms: 1000,
        };

        allocator.update_profile(driver_id, &observation);

        let prediction = allocator.predict_resources(driver_id);
        assert!(prediction.is_some());

        let pred = prediction.unwrap();
        assert!(pred.predicted_memory_bytes > 0);
        assert!(pred.predicted_cpu_percent > 0.0);
    }

    #[test]
    fn test_allocation_recommendation() {
        let mut allocator = PredictiveAllocator::new();
        let driver_id = ObjectId::new();

        let observation = ResourceObservation {
            allocated_bytes: 500 * 1024 * 1024,
            cpu_usage_percent: 85.0,
            io_ops_per_sec: 5000.0,
            network_bandwidth_mbps: 50.0,
            timestamp_ms: 1000,
        };

        // Add 10+ samples to build confidence > 0.9 (confidence = min(samples/10, 1.0))
        for _ in 0..12 {
            allocator.update_profile(driver_id, &observation);
        }

        let recommendation = allocator.get_allocation_recommendation(driver_id);
        assert!(recommendation.is_some());

        let rec = recommendation.unwrap();
        assert_eq!(rec.priority, TaskPriority::High); // 85% CPU should be High priority
        assert!(rec.recommended_memory_bytes > 500 * 1024 * 1024); // Should allocate with headroom
    }

    #[test]
    fn test_allocation_priority_levels() {
        assert_eq!(IoSchedulingClass::from_io_rate(15_000.0), IoSchedulingClass::Realtime);
        assert_eq!(IoSchedulingClass::from_io_rate(5_000.0), IoSchedulingClass::BestEffort);
        assert_eq!(IoSchedulingClass::from_io_rate(500.0), IoSchedulingClass::Idle);
    }

    #[test]
    fn test_prediction_statistics() {
        let mut allocator = PredictiveAllocator::new();

        for _ in 0..5 {
            let driver_id = ObjectId::new();
            let observation = ResourceObservation {
                allocated_bytes: 100 * 1024 * 1024,
                cpu_usage_percent: 50.0,
                io_ops_per_sec: 1000.0,
                network_bandwidth_mbps: 10.0,
                timestamp_ms: 1000,
            };

            for _ in 0..10 {
                allocator.update_profile(driver_id, &observation);
            }
        }

        let stats = allocator.get_stats();
        assert_eq!(stats.total_drivers, 5);
        assert!(stats.avg_confidence > 0.9); // After 10 samples each, confidence should be high
    }

    #[test]
    fn test_profile_sorting() {
        let mut allocator = PredictiveAllocator::new();

        // Create drivers with different CPU utilizations
        for i in 0..3 {
            let driver_id = ObjectId::new();
            let observation = ResourceObservation {
                allocated_bytes: 100 * 1024 * 1024,
                cpu_usage_percent: (i as f64 + 1.0) * 30.0, // 30%, 60%, 90%
                io_ops_per_sec: 1000.0,
                network_bandwidth_mbps: 10.0,
                timestamp_ms: 1000,
            };

            allocator.update_profile(driver_id, &observation);
        }

        let by_cpu = allocator.get_profiles_by_cpu_load();
        assert!(by_cpu[0].cpu_usage_pattern.avg_utilization_percent >= by_cpu[1].cpu_usage_pattern.avg_utilization_percent);
    }
}
