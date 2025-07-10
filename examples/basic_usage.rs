use fifo_cache::FifoCache;
use std::time::Duration;

fn main() {
  // Create a cache with capacity 1000 and 5-minute TTL
  let mut cache = FifoCache::new(1000, Duration::from_secs(300));
  
  // Insert some values
  cache.insert("user:123", "John Doe");
  cache.insert("user:456", "Jane Smith");
  cache.insert("config:timeout", "30");
  
  // Retrieve values
  if let Some(name) = cache.get(&"user:123") {
    println!("Found user: {}", name);
  }
  
  // Cache will automatically evict oldest entries when full
  // and expire entries after 5 minutes
  
  println!("Cache size: {}/{}", cache.len(), cache.max_size());
  
  // Manual cleanup of expired entries
  cache.cleanup_expired();
}
