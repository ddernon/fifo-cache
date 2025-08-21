
use fifo_cache::FifoCache;
#[cfg(feature = "ttl")]
use std::thread;
#[cfg(feature = "ttl")]
use std::time::Duration;

#[test]
fn test_basic_operations() {
  let mut cache = FifoCache::new(
    3,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert("a", 1);
  cache.insert("b", 2);
  cache.insert("c", 3);
  
  assert_eq!(cache.get(&"a"), Some(&1));
  assert_eq!(cache.get(&"b"), Some(&2));
  assert_eq!(cache.get(&"c"), Some(&3));
  assert_eq!(cache.len(), 3);
}

#[test]
fn test_fifo_eviction() {
  let mut cache = FifoCache::new(
    2,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert("a", 1);
  cache.insert("b", 2);
  cache.insert("c", 3); // Should evict "a"
  
  assert_eq!(cache.get(&"a"), None);
  assert_eq!(cache.get(&"b"), Some(&2));
  assert_eq!(cache.get(&"c"), Some(&3));
  assert_eq!(cache.len(), 2);
}

#[test]
fn test_reduce_max_size_and_prune() {
  let mut cache = FifoCache::new(
    3,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert("a", 1);
  cache.insert("b", 2);
  cache.insert("c", 3);
  
  assert_eq!(cache.get(&"a"), Some(&1));
  assert_eq!(cache.len(), 3);

  cache.set_max_size(2, true); // Should prune
  
  assert_eq!(cache.get(&"a"), None);
  assert_eq!(cache.get(&"b"), Some(&2));
  assert_eq!(cache.get(&"c"), Some(&3));
  assert_eq!(cache.len(), 2);
}

#[test]
fn test_reduce_max_size_no_prune() {
  let mut cache = FifoCache::new(
    3,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert("a", 1);
  cache.insert("b", 2);
  cache.insert("c", 3);
  
  cache.set_max_size(2, false); // Should NOT prune
  
  assert_eq!(cache.get(&"a"), Some(&1));
  assert_eq!(cache.get(&"b"), Some(&2));
  assert_eq!(cache.get(&"c"), Some(&3));
  assert_eq!(cache.len(), 3);

  cache.insert("c", 4); // Update existing key => should still not prune
  assert_eq!(cache.get(&"a"), Some(&1));
  assert_eq!(cache.get(&"b"), Some(&2));
  assert_eq!(cache.get(&"c"), Some(&4));
  assert_eq!(cache.len(), 3);

  cache.insert("d", 5); // New key => should prune
  assert_eq!(cache.get(&"a"), None);
  assert_eq!(cache.get(&"b"), None);
  assert_eq!(cache.get(&"c"), Some(&4));
  assert_eq!(cache.get(&"d"), Some(&5));
  assert_eq!(cache.len(), 2);
}

#[cfg(feature = "ttl")]
#[test]
fn test_ttl_expiration() {
  let mut cache = FifoCache::new(10, Duration::from_millis(100));
  
  cache.insert("key", "value");
  assert_eq!(cache.get(&"key"), Some(&"value"));
  
  thread::sleep(Duration::from_millis(150));
  assert_eq!(cache.get(&"key"), None);
}

#[test]
fn test_update_existing() {
  let mut cache = FifoCache::new(
    10,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert("key", "value1");
  cache.insert("key", "value2");
  
  assert_eq!(cache.get(&"key"), Some(&"value2"));
  assert_eq!(cache.len(), 1);
}

#[test]
fn test_remove() {
  let mut cache = FifoCache::new(
    10,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert("key", "value");
  assert_eq!(cache.remove(&"key"), Some("value"));
  assert_eq!(cache.get(&"key"), None);
  assert_eq!(cache.len(), 0);
}

#[cfg(feature = "ttl")]
#[test]
fn test_cleanup_expired() {
  let mut cache = FifoCache::new(10, Duration::from_millis(100));
  
  cache.insert("key1", "value1");
  cache.insert("key2", "value2");
  
  thread::sleep(Duration::from_millis(150));
  cache.cleanup_expired();
  
  assert_eq!(cache.len(), 0);
}

#[test]
fn test_lazy() {
  let mut cache: FifoCache<String, String> = FifoCache::new(
    10,
    #[cfg(feature = "ttl")]
    Duration::from_secs(60)
  );
  
  cache.insert_lazy("key1", "value1");
  cache.insert_lazy("key2", "value2");
  assert_eq!(cache.get("key1"), Some(&String::from("value1")));
  assert_eq!(cache.remove("key2"), Some(String::from("value2")));
  assert_eq!(cache.get("key2"), None);
}
