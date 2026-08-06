// SHER AI Services: Comprehensive Tests

#[cfg(test)]
mod tests {
    use crate::anomaly_detection::*;
    use crate::predictive_allocation::*;
    use crate::adaptive_scheduling::*;
    use crate::continuous_learning::*;
    use crate::inference_engine::*;
    use crate::reinforcement_learning::*;
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

    // ========================================================================
    // ADAPTIVE SCHEDULING TESTS
    // ========================================================================

    #[test]
    fn test_adaptive_scheduler_new() {
        let scheduler = AdaptiveScheduler::new();
        assert_eq!(scheduler.total_decisions, 0);
        assert_eq!(scheduler.decisions.len(), 0);
    }

    #[test]
    fn test_scheduling_strategy_selection() {
        let mut scheduler = AdaptiveScheduler::new();
        let driver_id = ObjectId::new();

        // High CPU, low anomalies: aggressive
        let decision = scheduler.decide_scheduling(
            driver_id,
            SchedulingStrategy::Balanced,
            85.0, // high CPU
            50.0, // normal memory
            0,    // no anomalies
            50.0, // latency
            60.0, // SLO
        );
        assert_eq!(decision.strategy, SchedulingStrategy::Aggressive);

        // High anomalies: conservative
        let decision = scheduler.decide_scheduling(
            driver_id,
            SchedulingStrategy::Balanced,
            50.0,
            50.0,
            10,   // high anomalies
            50.0,
            60.0,
        );
        assert_eq!(decision.strategy, SchedulingStrategy::Conservative);
    }

    #[test]
    fn test_slo_tracking() {
        let mut scheduler = AdaptiveScheduler::new();
        let driver_id = ObjectId::new();

        // Record SLO achievement
        scheduler.record_slo_result(driver_id, 45.0, 60.0, 1000);
        scheduler.record_slo_result(driver_id, 55.0, 60.0, 1000);
        scheduler.record_slo_result(driver_id, 65.0, 60.0, 1000); // SLO miss

        let metrics = scheduler.get_metrics(driver_id).unwrap();
        assert_eq!(metrics.decisions_made, 3);
        assert_eq!(metrics.slo_violations, 1);
        assert!(metrics.slo_achievement_rate > 0.6);
    }

    #[test]
    fn test_workload_classification() {
        let mut classifier = WorkloadClassifier::new();
        let driver_id = ObjectId::new();

        // Classify as interactive (low CPU, higher latency variance)
        let workload = classifier.classify(driver_id, 10.0, 30.0, 500.0, 35.0);
        assert_eq!(workload, WorkloadType::Interactive);

        // Classify as ML (high CPU and memory)
        let workload = classifier.classify(driver_id, 85.0, 75.0, 2000.0, 50.0);
        assert_eq!(workload, WorkloadType::ML);

        // Classify as IO (high IO intensity)
        let workload = classifier.classify(driver_id, 40.0, 50.0, 8000.0, 50.0);
        assert_eq!(workload, WorkloadType::IO);

        // Classify as real-time (low latency variance)
        let workload = classifier.classify(driver_id, 60.0, 50.0, 2000.0, 5.0);
        assert_eq!(workload, WorkloadType::RealTime);
    }

    #[test]
    fn test_scheduler_statistics() {
        let mut scheduler = AdaptiveScheduler::new();

        for _ in 0..5 {
            let driver_id = ObjectId::new();
            scheduler.decide_scheduling(driver_id, SchedulingStrategy::Balanced, 50.0, 50.0, 0, 50.0, 60.0);
            scheduler.record_slo_result(driver_id, 55.0, 60.0, 1000);
        }

        let stats = scheduler.get_stats();
        assert_eq!(stats.total_drivers, 5);
        assert!(stats.avg_slo_achievement > 0.0);
    }

    // ========================================================================
    // CONTINUOUS LEARNING TESTS
    // ========================================================================

    #[test]
    fn test_learning_engine_new() {
        let engine = ContinuousLearningEngine::new();
        assert_eq!(engine.total_observations, 0);
        assert_eq!(engine.optimizations_applied, 0);
    }

    #[test]
    fn test_behavior_model_observation() {
        let mut model = DriverBehaviorModel::new(ObjectId::new());

        for i in 0..5 {
            let obs = RuntimeObservation {
                driver_id: model.driver_id,
                timestamp_ms: i * 1000,
                cpu_usage: 50.0 + (i as f64 * 5.0),
                memory_usage: 40.0,
                io_throughput: 1000.0,
                network_throughput: 100.0,
                latency_ms: 50.0,
                anomalies_detected: 0,
                task_count: 10,
            };
            model.observe(obs);
        }

        assert_eq!(model.samples, 5);
        assert!(model.avg_cpu_usage > 0.0);
        assert!(model.peak_cpu_usage > model.avg_cpu_usage);
    }

    #[test]
    fn test_correlation_calculation() {
        let mut model = DriverBehaviorModel::new(ObjectId::new());

        // Create correlated observations
        for i in 0..15 {
            let obs = RuntimeObservation {
                driver_id: model.driver_id,
                timestamp_ms: i * 100,
                cpu_usage: 30.0 + (i as f64 * 4.0),
                memory_usage: 20.0 + (i as f64 * 3.0),
                io_throughput: 500.0,
                network_throughput: 50.0,
                latency_ms: 40.0,
                anomalies_detected: 0,
                task_count: 5,
            };
            model.observe(obs);
        }

        assert!(model.cpu_memory_correlation > 0.5); // Should be positively correlated
    }

    #[test]
    fn test_trend_analysis() {
        let mut model = DriverBehaviorModel::new(ObjectId::new());

        // Increasing CPU trend
        for i in 0..20 {
            let obs = RuntimeObservation {
                driver_id: model.driver_id,
                timestamp_ms: i * 100,
                cpu_usage: 20.0 + (i as f64 * 2.0),
                memory_usage: 40.0,
                io_throughput: 1000.0,
                network_throughput: 100.0,
                latency_ms: 50.0,
                anomalies_detected: 0,
                task_count: 10,
            };
            model.observe(obs);
        }

        // Trend should be positive (increasing)
        assert!(model.cpu_trend > 0.0);
    }

    #[test]
    fn test_anomaly_detection_from_model() {
        let mut model = DriverBehaviorModel::new(ObjectId::new());

        // Build baseline
        for _ in 0..15 {
            let obs = RuntimeObservation {
                driver_id: model.driver_id,
                timestamp_ms: 1000,
                cpu_usage: 50.0,
                memory_usage: 40.0,
                io_throughput: 1000.0,
                network_throughput: 100.0,
                latency_ms: 50.0,
                anomalies_detected: 0,
                task_count: 10,
            };
            model.observe(obs);
        }

        // Normal observation
        let normal = RuntimeObservation {
            driver_id: model.driver_id,
            timestamp_ms: 2000,
            cpu_usage: 55.0,
            memory_usage: 42.0,
            io_throughput: 1000.0,
            network_throughput: 100.0,
            latency_ms: 52.0,
            anomalies_detected: 0,
            task_count: 10,
        };
        assert!(!model.is_anomalous(&normal));

        // Anomalous observation
        let anomalous = RuntimeObservation {
            driver_id: model.driver_id,
            timestamp_ms: 2000,
            cpu_usage: 150.0,
            memory_usage: 40.0,
            io_throughput: 1000.0,
            network_throughput: 100.0,
            latency_ms: 50.0,
            anomalies_detected: 0,
            task_count: 10,
        };
        assert!(model.is_anomalous(&anomalous));
    }

    #[test]
    fn test_continuous_learning_observations() {
        let mut engine = ContinuousLearningEngine::new();
        let driver_id = ObjectId::new();

        for i in 0..10 {
            let obs = RuntimeObservation {
                driver_id,
                timestamp_ms: i * 1000,
                cpu_usage: 40.0 + (i as f64 * 3.0),
                memory_usage: 30.0,
                io_throughput: 500.0,
                network_throughput: 50.0,
                latency_ms: 45.0,
                anomalies_detected: 0,
                task_count: 8,
            };
            engine.observe(obs);
        }

        assert_eq!(engine.total_observations, 10);
        let model = engine.get_model(driver_id).unwrap();
        assert_eq!(model.samples, 10);
    }

    #[test]
    fn test_optimization_tracking() {
        let mut engine = ContinuousLearningEngine::new();
        let driver_id = ObjectId::new();

        let result = OptimizationResult {
            driver_id,
            change_type: "cpu_affinity".to_string(),
            before_metric: 100.0,
            after_metric: 85.0,
            improvement_percent: 15.0,
            applied_at_ms: 1000,
        };

        engine.record_optimization(result);

        let stats = engine.get_optimization_stats();
        assert_eq!(stats.total_optimizations, 1);
        assert_eq!(stats.successful_optimizations, 1);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[test]
    fn test_learning_stats() {
        let mut engine = ContinuousLearningEngine::new();

        for i in 0..3 {
            let driver_id = ObjectId::new();
            for j in 0..10 {
                let obs = RuntimeObservation {
                    driver_id,
                    timestamp_ms: j * 1000,
                    cpu_usage: 40.0 + (i as f64 * 10.0),
                    memory_usage: 30.0,
                    io_throughput: 500.0,
                    network_throughput: 50.0,
                    latency_ms: 45.0,
                    anomalies_detected: 0,
                    task_count: 8,
                };
                engine.observe(obs);
            }
        }

        let stats = engine.get_stats();
        assert_eq!(stats.total_drivers_observed, 3);
        assert_eq!(stats.total_observations, 30);
    }

    // ========================================================================
    // INFERENCE ENGINE TESTS
    // ========================================================================

    #[test]
    fn test_inference_engine_new() {
        let engine = InferenceEngine::new();
        assert_eq!(engine.decisions_made, 0);
        assert_eq!(engine.high_confidence_decisions, 0);
    }

    #[test]
    fn test_feature_vector_creation() {
        let driver_id = ObjectId::new();
        let mut context = std::collections::HashMap::new();
        context.insert("cpu_usage".to_string(), 75.0);
        context.insert("memory_usage".to_string(), 60.0);
        context.insert("io_ops_per_sec".to_string(), 5000.0);
        context.insert("latency_ms".to_string(), 80.0);

        let features = FeatureVector::from_context(driver_id, &context);
        assert!(features.cpu_usage_normalized > 0.7);
        assert!(features.memory_usage_normalized > 0.5);
        assert!(features.latency_ms > 0.0);
    }

    #[test]
    fn test_resource_allocation_inference() {
        let mut engine = InferenceEngine::new();
        let driver_id = ObjectId::new();
        let request_id = ObjectId::new();

        let mut context = std::collections::HashMap::new();
        context.insert("cpu_usage".to_string(), 50.0);
        context.insert("memory_usage".to_string(), 85.0);
        context.insert("anomalies".to_string(), 2.0);

        let features = FeatureVector::from_context(driver_id, &context);

        let request = InferenceRequest {
            request_id,
            driver_id,
            inference_type: InferenceType::ResourceAllocation,
            context: context.clone(),
            timestamp_ms: 1000,
        };

        let decision = engine.infer(&request, &features, 0.8);
        assert_eq!(decision.action, "allocate_resources");
        assert!(decision.confidence > 0.5);
        assert!(decision.parameters.contains_key("memory_scale"));
    }

    #[test]
    fn test_scheduling_strategy_inference() {
        let mut engine = InferenceEngine::new();
        let driver_id = ObjectId::new();

        let mut context = std::collections::HashMap::new();
        context.insert("cpu_usage".to_string(), 85.0);
        context.insert("anomalies".to_string(), 1.0);
        context.insert("latency_ms".to_string(), 40.0);

        let features = FeatureVector::from_context(driver_id, &context);

        let request = InferenceRequest {
            request_id: ObjectId::new(),
            driver_id,
            inference_type: InferenceType::SchedulingStrategy,
            context,
            timestamp_ms: 1000,
        };

        let decision = engine.infer(&request, &features, 0.8);
        assert!(decision.action.contains("strategy"));
        assert!(decision.confidence > 0.5);
    }

    #[test]
    fn test_anomaly_response_inference() {
        let mut engine = InferenceEngine::new();
        let driver_id = ObjectId::new();

        let mut context = std::collections::HashMap::new();
        context.insert("anomalies".to_string(), 8.0);
        context.insert("slo_violation_rate".to_string(), 0.5);

        let features = FeatureVector::from_context(driver_id, &context);

        let request = InferenceRequest {
            request_id: ObjectId::new(),
            driver_id,
            inference_type: InferenceType::AnomalyResponse,
            context,
            timestamp_ms: 1000,
        };

        let decision = engine.infer(&request, &features, 0.7);
        assert!(decision.action.contains("isolate") || decision.action.contains("throttle"));
    }

    #[test]
    fn test_inference_statistics() {
        let mut engine = InferenceEngine::new();
        let driver_id = ObjectId::new();

        for _ in 0..5 {
            let request = InferenceRequest {
                request_id: ObjectId::new(),
                driver_id,
                inference_type: InferenceType::ResourceAllocation,
                context: std::collections::HashMap::new(),
                timestamp_ms: 1000,
            };

            let features = FeatureVector::new(driver_id);
            engine.infer(&request, &features, 0.8);
        }

        let stats = engine.get_stats();
        assert_eq!(stats.total_decisions, 5);
        assert!(stats.avg_latency_ms >= 0.0);
    }

    // ========================================================================
    // REINFORCEMENT LEARNING TESTS
    // ========================================================================

    #[test]
    fn test_action_policy_new() {
        let policy = ActionPolicy::new("test_action".to_string());
        assert_eq!(policy.attempts, 0);
        assert_eq!(policy.avg_reward, 0.0);
    }

    #[test]
    fn test_action_policy_update() {
        let mut policy = ActionPolicy::new("test_action".to_string());

        policy.update(0.8);
        policy.update(0.9);
        policy.update(-0.1);

        assert_eq!(policy.attempts, 3);
        assert_eq!(policy.success_count, 2);
        assert_eq!(policy.failure_count, 1);
        assert!(policy.avg_reward > 0.0);
    }

    #[test]
    fn test_driver_learner_new() {
        let driver_id = ObjectId::new();
        let learner = DriverLearner::new(driver_id);
        assert_eq!(learner.episodes, 0);
        assert_eq!(learner.cumulative_reward, 0.0);
    }

    #[test]
    fn test_driver_learner_reward() {
        let driver_id = ObjectId::new();
        let mut learner = DriverLearner::new(driver_id);

        learner.record_reward("action_a".to_string(), 0.9);
        learner.record_reward("action_a".to_string(), 0.85);
        learner.record_reward("action_b".to_string(), 0.5);

        assert_eq!(learner.episodes, 3);
        let best = learner.get_best_action().unwrap();
        assert_eq!(best, "action_a");
    }

    #[test]
    fn test_driver_learner_convergence() {
        let driver_id = ObjectId::new();
        let mut learner = DriverLearner::new(driver_id);

        // Add consistent rewards
        for _ in 0..10 {
            learner.record_reward("action".to_string(), 0.8);
        }

        let convergence = learner.convergence_metric();
        assert!(convergence < 0.5); // Should be low variance (converged)
    }

    #[test]
    fn test_reinforcement_learner_new() {
        let rl = ReinforcementLearner::new();
        assert_eq!(rl.total_episodes, 0);
        assert_eq!(rl.total_reward, 0.0);
    }

    #[test]
    fn test_reinforcement_learner_reward() {
        let mut rl = ReinforcementLearner::new();
        let driver_id = ObjectId::new();

        let event = RewardEvent {
            driver_id,
            signal: RewardSignal::SloAchieved,
            magnitude: 0.9,
            timestamp_ms: 1000,
            action_taken: "allocate_resources".to_string(),
        };

        rl.reward(event);

        assert_eq!(rl.total_episodes, 1);
        assert!(rl.total_reward > 0.0);
    }

    #[test]
    fn test_reinforcement_learner_policy_selection() {
        let mut rl = ReinforcementLearner::new();
        let driver_id = ObjectId::new();

        // Reward successful action
        for _ in 0..3 {
            let event = RewardEvent {
                driver_id,
                signal: RewardSignal::OptimizationSuccess,
                magnitude: 0.8,
                timestamp_ms: 1000,
                action_taken: "aggressive_strategy".to_string(),
            };
            rl.reward(event);
        }

        // Reward less successful action
        for _ in 0..2 {
            let event = RewardEvent {
                driver_id,
                signal: RewardSignal::OptimizationFailed,
                magnitude: 0.3,
                timestamp_ms: 1000,
                action_taken: "conservative_strategy".to_string(),
            };
            rl.reward(event);
        }

        let best = rl.get_best_global_policy().unwrap();
        assert_eq!(best, "aggressive_strategy");
    }

    #[test]
    fn test_reinforcement_learner_stats() {
        let mut rl = ReinforcementLearner::new();
        let driver_id = ObjectId::new();

        for i in 0..10 {
            let signal = if i < 7 {
                RewardSignal::SloAchieved
            } else {
                RewardSignal::SloViolated
            };

            let event = RewardEvent {
                driver_id,
                signal,
                magnitude: 0.7,
                timestamp_ms: 1000,
                action_taken: "action".to_string(),
            };
            rl.reward(event);
        }

        let stats = rl.get_stats();
        assert_eq!(stats.total_episodes, 10);
        assert!(stats.avg_reward_per_episode > 0.0);
    }

    #[test]
    fn test_reward_signal_conversion() {
        let rl = ReinforcementLearner::new();

        let slo_achieved = rl.signal_to_reward(RewardSignal::SloAchieved, 0.8);
        let slo_violated = rl.signal_to_reward(RewardSignal::SloViolated, 0.8);

        assert!(slo_achieved > 0.0);
        assert!(slo_violated < 0.0);
    }
}
