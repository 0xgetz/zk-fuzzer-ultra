// Intelligent caching system for circuit analysis results
// Provides TTL-based expiration and LRU eviction policies

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Cache entry with metadata for TTL tracking
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    inserted_at: Instant,
    last_accessed: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            inserted_at: now,
            last_accessed: now,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }

    fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// Eviction policy for the cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvictionPolicy {
    /// Least Recently Used - evicts the least recently accessed item
    #[default]
    LRU,
    /// First In First Out - evicts the oldest item
    FIFO,
    /// Least Frequently Used - evicts the item with fewest accesses
    LFU,
}

/// Configuration for the cache system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,
    /// Default TTL for cache entries
    pub default_ttl_seconds: u64,
    /// Eviction policy to use
    pub eviction_policy: EvictionPolicy,
    /// Whether to enable cache statistics tracking
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            default_ttl_seconds: 3600, // 1 hour
            eviction_policy: EvictionPolicy::LRU,
            enable_stats: true,
        }
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub expirations: u64,
    pub total_entries: usize,
    pub hit_rate: f64,
}

impl CacheStats {
    fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

/// Intelligent caching system for circuit analysis results
/// 
/// Features:
/// - TTL-based expiration for time-sensitive data
/// - Configurable eviction policies (LRU, FIFO, LFU)
/// - Thread-safe with read-write locks
/// - Statistics tracking for cache performance monitoring
/// - Automatic cleanup of expired entries
#[derive(Debug)]
pub struct AnalysisCache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    config: CacheConfig,
    stats: CacheStats,
    access_order: VecDeque<K>,
    access_count: HashMap<K, u64>,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync>
    AnalysisCache<K, V>
{
    /// Create a new cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            stats: CacheStats::default(),
            access_order: VecDeque::new(),
            access_count: HashMap::new(),
        }
    }

    /// Create a new cache with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Insert a value into the cache with the default TTL
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_with_ttl(key, value, Duration::from_secs(self.config.default_ttl_seconds));
    }

    /// Insert a value into the cache with a custom TTL
    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) {
        // If key already exists, update it
        if self.entries.contains_key(&key) {
            self.entries.insert(key.clone(), CacheEntry::new(value, ttl));
            return;
        }

        // Evict if at capacity
        while self.entries.len() >= self.config.max_entries {
            self.evict_one();
        }

        // Insert new entry
        let entry = CacheEntry::new(value, ttl);
        self.entries.insert(key.clone(), entry);
        self.access_order.push_back(key);
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        // Check if entry exists and is not expired
        if let Some(entry) = self.entries.get_mut(key) {
            if entry.is_expired() {
                // Remove expired entry
                self.entries.remove(key);
                self.stats.expirations += 1;
                self.stats.misses += 1;
                return None;
            }

            // Update access metadata
            entry.touch();
            *self.access_count.entry(key.clone()).or_insert(0) += 1;

            self.stats.hits += 1;
            return Some(entry.value.clone());
        }

        self.stats.misses += 1;
        None
    }

    /// Check if a key exists in the cache (without updating access time)
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.get(key).map_or(false, |e| !e.is_expired())
    }

    /// Remove a specific key from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.entries.remove(key) {
            self.access_count.remove(key);
            // Remove from access order (inefficient but simple)
            self.access_order.retain(|k| k != key);
            Some(entry.value)
        } else {
            None
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.access_count.clear();
    }

    /// Remove all expired entries
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_keys: Vec<K> = self
            .entries
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.remove(&key);
        }
        count
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get mutable reference to stats for updating hit rate
    pub fn stats_mut(&mut self) -> &mut CacheStats {
        self.stats.total_entries = self.entries.len();
        self.stats.calculate_hit_rate();
        &mut self.stats
    }

    /// Get the current configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Evict one entry based on the configured eviction policy
    fn evict_one(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        let key_to_evict = match self.config.eviction_policy {
            EvictionPolicy::LRU => self.find_lru_key(),
            EvictionPolicy::FIFO => self.find_fifo_key(),
            EvictionPolicy::LFU => self.find_lfu_key(),
        };

        if let Some(key) = key_to_evict {
            self.entries.remove(&key);
            self.access_count.remove(&key);
            self.access_order.retain(|k| k != &key);
            self.stats.evictions += 1;
        }
    }

    fn find_lru_key(&self) -> Option<K> {
        self.entries
            .iter()
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(k, _)| k.clone())
    }

    fn find_fifo_key(&self) -> Option<K> {
        self.access_order.front().cloned()
    }

    fn find_lfu_key(&self) -> Option<K> {
        self.access_count
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(k, _)| k.clone())
    }
}

/// Thread-safe wrapper for AnalysisCache
#[derive(Debug)]
pub struct ThreadSafeCache<K, V> {
    inner: Arc<RwLock<AnalysisCache<K, V>>>,
}

impl<K: Eq + Hash + Clone + Send + Sync + 'static, V: Clone + Send + Sync>
    ThreadSafeCache<K, V>
{
    /// Create a new thread-safe cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AnalysisCache::new(config))),
        }
    }

    /// Create a new thread-safe cache with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut cache) = self.inner.write() {
            cache.insert(key, value);
        }
    }

    /// Insert with custom TTL
    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        if let Ok(mut cache) = self.inner.write() {
            cache.insert_with_ttl(key, value, ttl);
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.write().ok().and_then(|mut cache| cache.get(key))
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner
            .read()
            .map(|cache| cache.contains_key(key))
            .unwrap_or(false)
    }

    /// Remove a key
    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.write().ok().and_then(|mut cache| cache.remove(key))
    }

    /// Clear the cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.inner.write() {
            cache.clear();
        }
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&self) -> usize {
        self.inner.write().map(|mut cache| cache.cleanup_expired()).unwrap_or(0)
    }

    /// Get cache length
    pub fn len(&self) -> usize {
        self.inner.read().map(|cache| cache.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().map(|cache| cache.is_empty()).unwrap_or(true)
    }

    /// Get statistics
    pub fn stats(&self) -> Option<CacheStats> {
        self.inner.read().ok().map(|cache| cache.stats().clone())
    }
}

impl<K, V> Clone for ThreadSafeCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_get() {
        let mut cache = AnalysisCache::with_defaults();
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));
    }

    #[test]
    fn test_ttl_expiration() {
        let mut cache = AnalysisCache::new(CacheConfig {
            default_ttl_seconds: 0,
            ..Default::default()
        });
        cache.insert("key1", "value1");
        // Should be expired immediately with 0 TTL
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(cache.get(&"key1"), None);
    }

    #[test]
    fn test_eviction_policy_lru() {
        let config = CacheConfig {
            max_entries: 3,
            eviction_policy: EvictionPolicy::LRU,
            ..Default::default()
        };
        let mut cache = AnalysisCache::new(config);
        
        cache.insert(1, "a");
        cache.insert(2, "b");
        cache.insert(3, "c");
        
        // Access key 1 to make it recently used
        cache.get(&1);
        
        // Insert 4th item, should evict key 2 (LRU)
        cache.insert(4, "d");
        
        assert!(cache.contains_key(&1));
        assert!(!cache.contains_key(&2));
        assert!(cache.contains_key(&3));
        assert!(cache.contains_key(&4));
    }

    #[test]
    fn test_thread_safe_cache() {
        let cache = ThreadSafeCache::with_defaults();
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = AnalysisCache::with_defaults();
        cache.insert("key1", "value1");
        
        cache.get(&"key1"); // hit
        cache.get(&"key2"); // miss
        
        let stats = cache.stats_mut();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.5).abs() < 0.01);
    }
}
