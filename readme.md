# FIFO Cache with TTL

A simple, thread-safe, FIFO (First In, First Out) cache with TTL (Time To Live) support for Rust.

## Features

- **FIFO eviction policy**: Oldest entries are removed when capacity is reached
- **TTL support**: Entries automatically expire after a specified duration
- **Zero dependencies**: Uses only standard library types
- **Memory efficient**: Minimal overhead per cache entry
- Can be wrapped in `Arc<RwLock<>>` for concurrent access
- TTL and capacity can be modified after cache creation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
fifo-cache = "0.1.0"
```

## Example

```rust
use fifo_cache::FifoCache;
use std::time::Duration;

let mut cache = FifoCache::new(1000, Duration::from_secs(300));
cache.insert("key", "value");

if let Some(value) = cache.get(&"key") {
  println!("Found: {}", value);
}
```

## Testing

To run the tests, use the following command:
```bash
cargo test
```

### Testing for Windows 7

In `Cargo.lock`, replace `version = 4` with `version = 3`. Then:

```bash
rustup install 1.77
cargo +1.77 test
```


## Running an example

```bash
cargo run --example basic_usage
```
