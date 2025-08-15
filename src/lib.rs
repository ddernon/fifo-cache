//! A minimalist FIFO (First In, First Out) cache with TTL (Time To Live) support.
//!
//! This crate provides a memory-efficient cache that evicts the oldest entries
//! when it reaches capacity, and automatically expires entries after a specified
//! time duration.
//!
//! # Examples
//!
//! ```
//! use fifo_cache::FifoCache;
//! use std::time::Duration;
//!
//! let mut cache = FifoCache::new(100, Duration::from_secs(60));
//! cache.insert("key1", "value1");
//! 
//! if let Some(value) = cache.get(&"key1") {
//!     println!("Found: {}", value);
//! }
//! ```

use std::borrow::Borrow;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// A cache entry that stores a value along with its expiration time.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
  value: V,
  expires_at: Instant,
}

/// A FIFO cache with TTL support.
///
/// This cache maintains insertion order and evicts the oldest entries when
/// the maximum size is reached. Entries also expire after the specified TTL.
#[derive(Debug)]
pub struct FifoCache<K, V> {
  map: HashMap<K, CacheEntry<V>>,
  order: VecDeque<K>,
  max_size: usize,
  default_ttl: Duration,
}

impl<K, V> FifoCache<K, V>
where
  K: Clone + Eq + std::hash::Hash,
  V: Clone,
{
  /// Creates a new FIFO cache with the specified maximum size and default TTL.
  ///
  /// # Arguments
  ///
  /// * `max_size` - Maximum number of entries the cache can hold
  /// * `default_ttl` - Default time-to-live for cache entries
  ///
  /// # Examples
  ///
  /// ```
  /// use fifo_cache::FifoCache;
  /// use std::time::Duration;
  ///
  /// let cache: FifoCache<String, i32> = 
  ///     FifoCache::new(1000, Duration::from_secs(300));
  /// ```
  pub fn new(max_size: usize, default_ttl: Duration) -> Self {
    Self {
      map: HashMap::with_capacity(max_size),
      order: VecDeque::with_capacity(max_size),
      max_size,
      default_ttl,
    }
  }

  /// Retrieves a value from the cache if it exists and hasn't expired.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to look up
  ///
  /// # Returns
  ///
  /// `Some(&V)` if the key exists and hasn't expired, `None` otherwise.
  ///
  /// # Example
  ///
  /// ```
  /// use fifo_cache::FifoCache;
  /// use std::time::Duration;
  ///
  /// let mut cache = FifoCache::new(100, Duration::from_secs(60));
  /// cache.insert("my_key", "my_value");
  /// let value = cache.get(&"my_key");
  /// assert_eq!(value, Some(&"my_value"));
  /// ```
  pub fn get<Q>(&self, key: &Q) -> Option<&V> 
  where 
    K: Borrow<Q>,
    Q: ?Sized + std::hash::Hash + Eq,
  {
    let now = Instant::now();
    self.map.get(key)
      .filter(|entry| entry.expires_at > now)
      .map(|entry| &entry.value)
  }

  /// Inserts a key-value pair into the cache.
  ///
  /// If the key already exists, its value is updated and TTL is refreshed.
  /// If the cache is at capacity, the oldest entry is evicted.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to insert
  /// * `value` - The value to associate with the key
  pub fn insert(&mut self, key: K, value: V) {
    let expires_at = Instant::now() + self.default_ttl;
    
    if self.map.contains_key(&key) {
      // Entry exists, just update it
      self.map.insert(key, CacheEntry { value, expires_at });
    } else {
      // Entry doesn't exist, make room and insert it
      self.prune_plus_one();
      self.order.push_back(key.clone());
      self.map.insert(key, CacheEntry { value, expires_at });
    }
  }

  /// Inserts a key-value pair into the cache using types that can be converted into the key and value types.
  ///
  /// This is a convenience wrapper around [`insert`](Self::insert) that accepts any types implementing
  /// `Into<K>` and `Into<V>`. Note that using only `insert_lazy` prevents type inference, so you'll 
  /// need to explicitly specify the cache types:
  ///
  /// ```
  /// use fifo_cache::FifoCache;
  /// use std::time::Duration;
  /// 
  /// // With insert - types are inferred
  /// let mut cache = FifoCache::new(100, Duration::from_secs(60));
  /// cache.insert("key", "value");  // FifoCache<&str, &str>
  ///
  /// // With insert_lazy - types must be specified  
  /// let mut cache: FifoCache<String, String> = FifoCache::new(100, Duration::from_secs(60));
  /// cache.insert_lazy("key", "value");  // &str -> String conversion
  /// ```
  pub fn insert_lazy<Kinto: Into<K>, Vinto: Into<V>>(&mut self, key: Kinto, value: Vinto) {
    self.insert(key.into(), value.into())
  }

  /// Removes a key from the cache.
  ///
  /// # Arguments
  ///
  /// * `key` - The key to remove
  ///
  /// # Returns
  ///
  /// `Some(V)` if the key existed, `None` otherwise.
  pub fn remove<Q>(&mut self, key: &Q) -> Option<V> 
  where
    K: Borrow<Q>,
    Q: ?Sized + std::hash::Hash + Eq,
  {
    if let Some(entry) = self.map.remove(key) {
      self.order.retain(|k| k.borrow() != key);
      Some(entry.value)
    } else {
      None
    }
  }

  /// Returns the current number of entries in the cache.
  pub fn len(&self) -> usize {
    self.map.len()
  }

  /// Returns `true` if the cache is empty.
  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  /// Removes all expired entries from the cache.
  ///
  /// This is performed automatically during normal operations, but can be
  /// called manually to free memory immediately.
  pub fn cleanup_expired(&mut self) {
    let now = Instant::now();
    self.order.retain(|key| {
      if let Some(entry) = self.map.get(key) {
        if entry.expires_at <= now {
          self.map.remove(key);
          false
        } else {
          true
        }
      } else {
        false
      }
    });
  }

  /// Clears all entries from the cache.
  pub fn clear(&mut self) {
    self.map.clear();
    self.order.clear();
  }

  /// Returns the maximum capacity of the cache.
  pub fn max_size(&self) -> usize {
    self.max_size
  }

  /// Sets the maximum capacity of the cache.
  /// 
  /// # Arguments
  ///
  /// * `max_size` - The new maximum number of entries the cache can hold
  /// * `prune` - Whether or not to immediately prune the excess number of entries, in case the update causes
  ///   the cache to be above capacity
  pub fn set_max_size(&mut self, max_size: usize, prune: bool) {
    self.max_size = max_size;
    if prune {
      self.prune_exact();
    }
  }

  /// Returns the default TTL for cache entries.
  pub fn default_ttl(&self) -> Duration {
    self.default_ttl
  }

  /// Sets the default TTL for cache entries.
  /// Note that this will only affect entries that get inserted or updated after the change.
  /// Existing entries will keep their TTL until they expire.
  /// 
  /// # Arguments
  ///
  /// * `default_ttl` - The new default time-to-live for cache entries
  pub fn set_default_ttl(&mut self, default_ttl: Duration) {
    self.default_ttl = default_ttl;
  }

  // Evicts oldest entries if at capacity
  fn prune_exact(&mut self) {
    while self.order.len() > self.max_size {
      if let Some(old_key) = self.order.pop_front() {
        self.map.remove(&old_key);
      }
    }
  }

  // Evicts oldest entries if at capacity, and makes room for a new one
  fn prune_plus_one(&mut self) {
    while self.order.len() >= self.max_size {
      if let Some(old_key) = self.order.pop_front() {
        self.map.remove(&old_key);
      }
    }
  }
}

impl<K, V> Default for FifoCache<K, V>
where
  K: Clone + Eq + std::hash::Hash,
  V: Clone,
{
  /// Creates a cache with capacity 1000 and TTL of 5 minutes.
  /// This is *extremely* arbitrary and you most likely want to use your own, use-case-adjusted settings rather than this default.
  fn default() -> Self {
    Self::new(1000, Duration::from_secs(300))
  }
}
