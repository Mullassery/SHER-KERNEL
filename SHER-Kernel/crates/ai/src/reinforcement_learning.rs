// SHER AI Services: Reinforcement Learning
// Learn optimal policies through reward-based feedback and experience

use sher_common::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// REWARD TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardSignal {
    SloAchieved,           // Driver met SLO
    SloViolated,           // Driver missed SLO
    AnomalyDetected,       // System detected anomaly
    OptimizationSuccess,   // Optimization improved metrics
    OptimizationFailed,    // Optimization made things worse
    ResourceEfficient,     // Good resource utilization
    ResourceWasted,        // Poor resource utilization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardEvent {
    pub driver_id: ObjectId,
    pub signal: RewardSignal,
    pub magnitude: f64,    // 0.0-1.0 confidence/strength
    pub timestamp_ms: u64,
    pub action_taken: String,
}

// ============================================================================
// POLICY LEARNING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPolicy {
    pub action_name: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_reward: f64,
    pub avg_reward: f64,
    pub attempts: u64,
    pub learning_rate: f64,
}

impl ActionPolicy {
    pub fn new(action_name: String) -> Self {
        ActionPolicy {
            action_name,
            success_count: 0,
            failure_count: 0,
            total_reward: 0.0,
            avg_reward: 0.0,
            attempts: 0,
            learning_rate: 0.1,
        }
    }

    /// Update policy based on reward
    pub fn update(&mut self, reward: f64) {
        self.attempts += 1;
        self.total_reward += reward;

        // Update average with exponential moving average
        self.avg_reward = self.avg_reward * (1.0 - self.learning_rate) + reward * self.learning_rate;

        if reward > 0.0 {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            self.success_count as f64 / self.attempts as f64
        }
    }
}

// ============================================================================
// DRIVER STRATEGY LEARNER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverLearner {
    pub driver_id: ObjectId,
    pub policies: HashMap<String, ActionPolicy>,
    pub episodes: u64,
    pub cumulative_reward: f64,
    pub optimal_actions: Vec<String>,
    pub learning_history: Vec<f64>,
    pub max_history: usize,
}

impl DriverLearner {
    pub fn new(driver_id: ObjectId) -> Self {
        DriverLearner {
            driver_id,
            policies: HashMap::new(),
            episodes: 0,
            cumulative_reward: 0.0,
            optimal_actions: Vec::new(),
            learning_history: Vec::new(),
            max_history: 100,
        }
    }

    /// Record reward for an action
    pub fn record_reward(&mut self, action: String, reward: f64) {
        let policy = self.policies
            .entry(action.clone())
            .or_insert_with(|| ActionPolicy::new(action));

        policy.update(reward);
        self.cumulative_reward += reward;

        // Track learning progress
        if self.learning_history.len() >= self.max_history {
            self.learning_history.remove(0);
        }
        self.learning_history.push(reward);

        self.episodes += 1;
    }

    /// Get best action based on learned rewards
    pub fn get_best_action(&self) -> Option<String> {
        self.policies
            .iter()
            .max_by(|a, b| a.1.avg_reward.partial_cmp(&b.1.avg_reward).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }

    /// Get top N actions by reward
    pub fn get_top_actions(&self, n: usize) -> Vec<(String, f64)> {
        let mut actions: Vec<_> = self.policies
            .iter()
            .map(|(name, policy)| (name.clone(), policy.avg_reward))
            .collect();

        actions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        actions.into_iter().take(n).collect()
    }

    /// Get learning convergence metric (variance in recent rewards)
    pub fn convergence_metric(&self) -> f64 {
        if self.learning_history.len() < 2 {
            return 1.0;
        }

        let avg = self.learning_history.iter().sum::<f64>() / self.learning_history.len() as f64;
        let variance = self.learning_history
            .iter()
            .map(|r| (r - avg).powi(2))
            .sum::<f64>() / self.learning_history.len() as f64;

        variance.sqrt()
    }
}

// ============================================================================
// GLOBAL REINFORCEMENT LEARNING ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ReinforcementLearner {
    pub driver_learners: HashMap<ObjectId, DriverLearner>,
    pub global_policies: HashMap<String, ActionPolicy>,
    pub reward_history: Vec<RewardEvent>,
    pub max_history: usize,
    pub total_episodes: u64,
    pub total_reward: f64,
}

impl ReinforcementLearner {
    pub fn new() -> Self {
        ReinforcementLearner {
            driver_learners: HashMap::new(),
            global_policies: HashMap::new(),
            reward_history: Vec::new(),
            max_history: 10000,
            total_episodes: 0,
            total_reward: 0.0,
        }
    }

    /// Process reward event
    pub fn reward(&mut self, event: RewardEvent) {
        // Convert signal to reward value first
        let reward_value = self.signal_to_reward(event.signal, event.magnitude);
        let action_name = event.action_taken.clone();

        // Update driver learner
        {
            let driver_learner = self.driver_learners
                .entry(event.driver_id)
                .or_insert_with(|| DriverLearner::new(event.driver_id));

            driver_learner.record_reward(action_name.clone(), reward_value);
        }

        // Update global policy
        let global_policy = self.global_policies
            .entry(action_name)
            .or_insert_with(|| ActionPolicy::new(event.action_taken.clone()));

        global_policy.update(reward_value);

        self.total_episodes += 1;
        self.total_reward += reward_value;

        // Keep history
        if self.reward_history.len() >= self.max_history {
            self.reward_history.remove(0);
        }
        self.reward_history.push(event);
    }

    pub fn signal_to_reward(&self, signal: RewardSignal, magnitude: f64) -> f64 {
        match signal {
            RewardSignal::SloAchieved => magnitude.max(0.5),
            RewardSignal::SloViolated => -magnitude.min(0.5),
            RewardSignal::AnomalyDetected => -0.3 * magnitude,
            RewardSignal::OptimizationSuccess => magnitude.max(0.6),
            RewardSignal::OptimizationFailed => -magnitude.min(0.4),
            RewardSignal::ResourceEfficient => magnitude * 0.3,
            RewardSignal::ResourceWasted => -magnitude * 0.2,
        }
    }

    /// Get best global policy
    pub fn get_best_global_policy(&self) -> Option<String> {
        self.global_policies
            .iter()
            .max_by(|a, b| a.1.avg_reward.partial_cmp(&b.1.avg_reward).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| name.clone())
    }

    /// Get best actions for specific driver
    pub fn get_best_actions_for_driver(&self, driver_id: ObjectId) -> Vec<(String, f64)> {
        self.driver_learners
            .get(&driver_id)
            .map(|learner| learner.get_top_actions(5))
            .unwrap_or_default()
    }

    /// Get learning stats
    pub fn get_stats(&self) -> RLearningStats {
        let avg_reward = if self.total_episodes > 0 {
            self.total_reward / self.total_episodes as f64
        } else {
            0.0
        };

        let best_action = self.get_best_global_policy();

        RLearningStats {
            total_episodes: self.total_episodes,
            total_drivers_learned: self.driver_learners.len() as u64,
            avg_reward_per_episode: avg_reward,
            cumulative_reward: self.total_reward,
            best_global_policy: best_action.unwrap_or_default(),
            total_policies: self.global_policies.len() as u64,
        }
    }

    /// Get convergence analysis
    pub fn get_convergence_status(&self) -> Vec<(ObjectId, f64)> {
        let mut convergence: Vec<_> = self.driver_learners
            .iter()
            .map(|(id, learner)| (*id, learner.convergence_metric()))
            .collect();

        convergence.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        convergence
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLearningStats {
    pub total_episodes: u64,
    pub total_drivers_learned: u64,
    pub avg_reward_per_episode: f64,
    pub cumulative_reward: f64,
    pub best_global_policy: String,
    pub total_policies: u64,
}
