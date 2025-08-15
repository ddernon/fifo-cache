# FIFO Cache with TTL

[![crates.io](https://img.shields.io/crates/v/fifo-cache.svg)](https://crates.io/crates/fifo-cache)
[![license](https://img.shields.io/crates/l/fifo-cache?logo=open%20source%20initiative&logoColor=%23fff)](https://framagit.org/dder/fifo-cache/blob/master/license.txt)
![minimum suppported rust version](https://img.shields.io/crates/msrv/fifo-cache?logo=rust)
[![docs.rs](https://img.shields.io/docsrs/fifo-cache?logo=docs.rs)](https://docs.rs/fifo-cache)
[![pipeline status](https://framagit.org/dder/fifo-cache/badges/master/pipeline.svg)](https://framagit.org/dder/fifo-cache/pipelines)

A minimalist, thread-safe, FIFO (First In, First Out) cache with TTL (Time To Live) support for Rust.

## Features

- FIFO eviction policy: Oldest entries are removed when capacity is reached
- TTL support: Entries automatically expire after a specified duration
- Zero dependencies: Uses only standard library types
- Thread-safe: Can be wrapped in `Arc<RwLock<>>` (or `Arc<Mutex<>>`) for concurrent access
- TTL and capacity can be modified after cache creation

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
fifo-cache = "0.1"
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


## Windows 7 compatibility
The minimum required Rust version is 1.59. While this is unlikely to change in the foreseeable future,
the main objective is to remain at or below Rust 1.77, so as to preserve
[Windows 7 compatibility](https://blog.rust-lang.org/2024/02/26/Windows-7/).

To test with Rust 1.77:
- Change `version = 4` to `version = 3` in `Cargo.lock`.
- Install the 1.77 target: `rustup install 1.77.0-x86_64-pc-windows-gnu`.
- Then run clippy and the tests as follows:
```
cargo +1.77 clippy
cargo +1.77 test
```


## Running an example

```bash
cargo run --example basic_usage
```
