//! A minimalist FIFO (First In, First Out) cache with TTL (Time To Live) support.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "ttl")] {
//! use fifo_cache::FifoCache;
//! use std::time::Duration;
//!
//! let mut cache = FifoCache::new(100, Duration::from_secs(60));
//! cache.insert("key1", "value1");
//! 
//! if let Some(value) = cache.get(&"key1") {
//!   println!("Found: {}", value);
//! }
//! # }
//! ```

use std::borrow::Borrow;
use std::collections::{hash_map, HashMap, VecDeque};
#[cfg(feature = "ttl")]
use std::time::{Duration, Instant};

/// A cache entry that stores a value along with its expiration time.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
  value: V,
  #[cfg(feature = "ttl")]
  expires_at: Instant,
}

/// A FIFO cache with TTL support.
///
/// This cache maintains insertion order and evicts the oldest entries when
/// the maximum size is reached. Entries also expire after the specified TTL.
/// 
/// Note that:
/// - reinserting an existing entry will not move it back to the front
/// - the maximum capacity may *very briefly* be exceeded by 1
#[derive(Debug)]
pub struct FifoCache<K, V> {
  map: HashMap<K, CacheEntry<V>>,
  order: VecDeque<K>,
  max_size: usize,
  #[cfg(feature = "ttl")]
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
  pub fn new(
    max_size: usize,
    #[cfg(feature = "ttl")]
    default_ttl: Duration
  ) -> Self {
    Self {
      map: HashMap::with_capacity(max_size + 1),
      order: VecDeque::with_capacity(max_size + 1),
      max_size,
      #[cfg(feature = "ttl")]
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
  /// With TTL enabled:
  /// ```
  /// # #[cfg(feature = "ttl")] {
  /// use fifo_cache::FifoCache;
  /// use std::time::Duration;
  ///
  /// let mut cache = FifoCache::new(100, Duration::from_secs(60));
  /// cache.insert("my_key", "my_value");
  /// let value = cache.get(&"my_key");
  /// assert_eq!(value, Some(&"my_value"));
  /// # }
  /// ```
  /// 
  /// Without TTL:
  /// ```
  /// # #[cfg(not(feature = "ttl"))] {
  /// use fifo_cache::FifoCache;
  ///
  /// let mut cache = FifoCache::new(100);
  /// cache.insert("my_key", "my_value");
  /// assert_eq!(cache.get(&"my_key"), Some(&"my_value"));
  /// # }
  /// ```
  pub fn get<Q>(&self, key: &Q) -> Option<&V> 
  where 
    K: Borrow<Q>,
    Q: ?Sized + std::hash::Hash + Eq,
  {
    #[cfg(feature = "ttl")] {
      let now = Instant::now();
      self.map
        .get(key)
        .filter(|entry| entry.expires_at > now)
        .map(|entry| &entry.value)
    }

    #[cfg(not(feature = "ttl"))] {
      self.map.get(key).map(|entry| &entry.value)
    }
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
    #[cfg(feature = "ttl")] {
      let expires_at = Instant::now() + self.default_ttl;
      
      match self.map.entry(key.clone()) {
        hash_map::Entry::Occupied(mut entry) => {
          // Entry exists, just update it
          entry.insert(CacheEntry { value, expires_at });
        }
        hash_map::Entry::Vacant(entry) => {
          // Entry doesn't exist, insert it then prune
          entry.insert(CacheEntry { value, expires_at });
          self.order.push_back(key);
          self.prune();
        }
      }
    }

    #[cfg(not(feature = "ttl"))] {
      match self.map.entry(key.clone()) {
        hash_map::Entry::Occupied(mut entry) => {
          entry.insert(CacheEntry { value });
        }
        hash_map::Entry::Vacant(entry) => {
          entry.insert(CacheEntry { value });
          self.order.push_back(key);
          self.prune();
        }
      }
    }
  }

  /// Inserts a key-value pair into the cache using types that can be converted into the key and value types.
  ///
  /// This is a convenience wrapper around [`insert`](Self::insert) that accepts any types implementing
  /// `Into<K>` and `Into<V>`. Note that using only `insert_lazy` prevents type inference, so you'll 
  /// need to explicitly specify the cache types:
  ///
  /// ```
  /// # #[cfg(feature = "ttl")] {
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
  /// # }
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

  #[cfg(feature = "ttl")]
  /// Removes all expired entries from the cache.
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
      self.prune();
    }
  }

  #[cfg(feature = "ttl")]
  /// Returns the default TTL for cache entries.
  pub fn default_ttl(&self) -> Duration {
    self.default_ttl
  }

  #[cfg(feature = "ttl")]
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
  fn prune(&mut self) {
    while self.order.len() > self.max_size {
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
    Self::new(
      1000,
      #[cfg(feature = "ttl")]
      Duration::from_secs(300)
    )
  }
}
