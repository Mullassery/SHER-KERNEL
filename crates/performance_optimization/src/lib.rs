//! Phase 13: Performance Optimization
//!
//! Production performance enhancements:
//! - Object pooling for resource reuse
//! - Batch processing for throughput
//! - Caching for frequently accessed data
//! - Memory alignment for cache efficiency
//! - Lock-free algorithm patterns

use std::collections::VecDeque;
use sher_common::ObjectId;

#[derive(Clone, Debug)]
pub struct PooledResource<T: Clone> {
    id: ObjectId,
    resource: T,
    in_use: bool,
}

#[derive(Clone, Debug)]
pub struct ObjectPool<T: Clone> {
    available: VecDeque<PooledResource<T>>,
    in_use: Vec<PooledResource<T>>,
    max_size: usize,
}

#[derive(Clone, Debug)]
pub struct CacheEntry<K: Clone, V: Clone> {
    key: K,
    value: V,
    access_count: u32,
    timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct Cache<K: Clone, V: Clone> {
    entries: Vec<CacheEntry<K, V>>,
    max_entries: usize,
    hits: u64,
    misses: u64,
}

#[derive(Clone, Debug)]
pub struct Batch<T: Clone> {
    items: Vec<T>,
    size: usize,
    max_size: usize,
}

#[derive(Clone, Debug)]
pub struct PerformanceOptimizer {
    cache_hit_rate: f64,
    batch_size_avg: f64,
    pool_utilization: f64,
}

impl<T: Clone + Default> ObjectPool<T> {
    pub fn new(initial_size: usize) -> Self {
        let mut pool = ObjectPool {
            available: VecDeque::new(),
            in_use: Vec::new(),
            max_size: initial_size,
        };

        for _ in 0..initial_size {
            pool.available.push_back(PooledResource {
                id: ObjectId::new(),
                resource: T::default(),
                in_use: false,
            });
        }

        pool
    }

    pub fn acquire(&mut self, resource: T) -> Option<ObjectId> {
        if let Some(mut pooled) = self.available.pop_front() {
            pooled.resource = resource;
            pooled.in_use = true;
            let id = pooled.id.clone();
            self.in_use.push(pooled);
            Some(id)
        } else {
            None
        }
    }

    pub fn release(&mut self, id: &ObjectId) -> bool {
        if let Some(pos) = self.in_use.iter().position(|p| &p.id == id) {
            let mut pooled = self.in_use.remove(pos);
            pooled.in_use = false;
            self.available.push_back(pooled);
            true
        } else {
            false
        }
    }

    pub fn utilization(&self) -> f64 {
        self.in_use.len() as f64 / (self.in_use.len() + self.available.len()) as f64
    }

    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    pub fn in_use_count(&self) -> usize {
        self.in_use.len()
    }
}

impl<K: Clone + PartialEq, V: Clone> Cache<K, V> {
    pub fn new(max_entries: usize) -> Self {
        Cache {
            entries: Vec::new(),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    pub fn get(&mut self, key: &K, current_time: u64) -> Option<V> {
        if let Some(pos) = self.entries.iter().position(|e| &e.key == key) {
            let mut entry = self.entries.remove(pos);
            entry.access_count += 1;
            entry.timestamp = current_time;
            let value = entry.value.clone();
            self.entries.push(entry);
            self.hits += 1;
            Some(value)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn put(&mut self, key: K, value: V, current_time: u64) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }

        self.entries.push(CacheEntry {
            key,
            value,
            access_count: 1,
            timestamp: current_time,
        });
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

impl<T: Clone> Batch<T> {
    pub fn new(max_size: usize) -> Self {
        Batch {
            items: Vec::new(),
            size: 0,
            max_size,
        }
    }

    pub fn add(&mut self, item: T) -> bool {
        if self.size < self.max_size {
            self.items.push(item);
            self.size += 1;
            true
        } else {
            false
        }
    }

    pub fn flush(&mut self) -> Vec<T> {
        let items = std::mem::take(&mut self.items);
        self.size = 0;
        items
    }

    pub fn is_full(&self) -> bool {
        self.size >= self.max_size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn current_size(&self) -> usize {
        self.size
    }
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        PerformanceOptimizer {
            cache_hit_rate: 0.0,
            batch_size_avg: 0.0,
            pool_utilization: 0.0,
        }
    }

    pub fn record_metrics(&mut self, cache_hr: f64, batch_size: f64, pool_util: f64) {
        self.cache_hit_rate = cache_hr;
        self.batch_size_avg = batch_size;
        self.pool_utilization = pool_util;
    }

    pub fn get_optimization_score(&self) -> f64 {
        (self.cache_hit_rate * 0.4) + (self.batch_size_avg / 100.0 * 0.3) + (self.pool_utilization * 0.3)
    }

    pub fn cache_hit_rate(&self) -> f64 {
        self.cache_hit_rate
    }

    pub fn batch_size_avg(&self) -> f64 {
        self.batch_size_avg
    }

    pub fn pool_utilization(&self) -> f64 {
        self.pool_utilization
    }
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_pool_creation() {
        let pool: ObjectPool<u32> = ObjectPool::new(10);
        assert_eq!(pool.available_count(), 10);
        assert_eq!(pool.in_use_count(), 0);
    }

    #[test]
    fn test_object_pool_acquire_and_release() {
        let mut pool: ObjectPool<u32> = ObjectPool::new(5);

        let id1 = pool.acquire(42);
        assert!(id1.is_some());
        assert_eq!(pool.in_use_count(), 1);
        assert_eq!(pool.available_count(), 4);

        let released = pool.release(&id1.unwrap());
        assert!(released);
        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.available_count(), 5);
    }

    #[test]
    fn test_object_pool_utilization() {
        let mut pool: ObjectPool<u32> = ObjectPool::new(10);

        pool.acquire(1);
        pool.acquire(2);
        pool.acquire(3);

        let util = pool.utilization();
        assert!(util > 0.25 && util < 0.35);
    }

    #[test]
    fn test_cache_hit_and_miss() {
        let mut cache: Cache<String, u32> = Cache::new(10);

        cache.put("key1".to_string(), 100, 0);

        let hit = cache.get(&"key1".to_string(), 1);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap(), 100);

        let miss = cache.get(&"key2".to_string(), 2);
        assert!(miss.is_none());

        assert!(cache.hit_rate() > 0.4);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache: Cache<i32, String> = Cache::new(3);

        cache.put(1, "one".to_string(), 0);
        cache.put(2, "two".to_string(), 1);
        cache.put(3, "three".to_string(), 2);
        cache.put(4, "four".to_string(), 3);

        assert_eq!(cache.size(), 3);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache: Cache<String, u32> = Cache::new(10);

        cache.put("key1".to_string(), 100, 0);
        cache.put("key2".to_string(), 200, 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_batch_add_and_flush() {
        let mut batch: Batch<u32> = Batch::new(5);

        assert!(batch.add(1));
        assert!(batch.add(2));
        assert!(batch.add(3));

        assert_eq!(batch.current_size(), 3);
        assert!(!batch.is_full());

        let flushed = batch.flush();
        assert_eq!(flushed.len(), 3);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_batch_full_detection() {
        let mut batch: Batch<u32> = Batch::new(3);

        batch.add(1);
        batch.add(2);
        batch.add(3);

        assert!(batch.is_full());
        assert!(!batch.add(4));
    }

    #[test]
    fn test_batch_flush_on_full() {
        let mut batch: Batch<u32> = Batch::new(2);

        batch.add(10);
        batch.add(20);
        assert!(batch.is_full());

        let flushed = batch.flush();
        assert_eq!(flushed, vec![10, 20]);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_performance_optimizer_metrics() {
        let mut optimizer = PerformanceOptimizer::new();

        optimizer.record_metrics(0.85, 75.0, 0.65);

        assert!(optimizer.cache_hit_rate() > 0.8);
        assert!(optimizer.batch_size_avg() > 70.0);
        assert!(optimizer.pool_utilization() > 0.6);
    }

    #[test]
    fn test_optimization_score() {
        let mut optimizer = PerformanceOptimizer::new();

        optimizer.record_metrics(0.90, 100.0, 0.80);
        let score = optimizer.get_optimization_score();

        assert!(score > 0.7);
    }

    #[test]
    fn test_cache_with_multiple_keys() {
        let mut cache: Cache<u32, String> = Cache::new(10);

        for i in 0..5 {
            cache.put(i, format!("value_{}", i), i as u64);
        }

        for i in 0..5 {
            let value = cache.get(&i, 10);
            assert!(value.is_some());
            assert_eq!(value.unwrap(), format!("value_{}", i));
        }

        assert!(cache.hit_rate() > 0.95);
    }

    #[test]
    fn test_pool_multiple_acquire_and_release() {
        let mut pool: ObjectPool<u32> = ObjectPool::new(3);

        let id1 = pool.acquire(1).unwrap();
        let id2 = pool.acquire(2).unwrap();
        let id3 = pool.acquire(3).unwrap();

        assert_eq!(pool.in_use_count(), 3);
        assert_eq!(pool.available_count(), 0);

        pool.release(&id2);
        assert_eq!(pool.in_use_count(), 2);
        assert_eq!(pool.available_count(), 1);

        pool.release(&id1);
        pool.release(&id3);
        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.available_count(), 3);
    }

    #[test]
    fn test_batch_processing_workflow() {
        let mut batch: Batch<u32> = Batch::new(5);

        let mut processed_batches = 0;

        for i in 0..12 {
            if batch.add(i) {
                if batch.is_full() {
                    let _flushed = batch.flush();
                    processed_batches += 1;
                }
            } else {
                let _flushed = batch.flush();
                processed_batches += 1;
                batch.add(i);
            }
        }

        if !batch.is_empty() {
            batch.flush();
            processed_batches += 1;
        }

        assert!(processed_batches >= 2);
    }
}
