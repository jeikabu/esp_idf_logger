[log](https://crates.io/crates/log) implementation for platforms that use ESP-IDF (like ESP32).

`Cargo.toml`:
```toml
[dependencies]
esp_idf_logger = "0.1"
log = "0.4"
```

Usage:
```rust
esp_idf_logger::init().unwrap();
log::info!("Log stuff={} and={}", 1, "hi");
```