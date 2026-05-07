# ESP32 Rust Tutorial

Hands-on examples for programming an ESP32 in Rust. Each “chapter” in `src/chapters/` is a self-contained sample: `PROJECT_NAME`, an async `setup`, and an `update` loop, wired from `src/main.rs` (Embassy executor, `esp-idf-svc` HAL). Shared helpers live under `src/utils/`.

Stack in short: [esp-idf-svc](https://github.com/esp-rs/esp-idf-svc) (ESP-IDF from Rust), [Embassy](https://github.com/embassy-rs/embassy) for async timing/tasks, and [anyhow](https://github.com/dtolnay/anyhow) for error handling. Build is driven by `embuild` / `build.rs` as usual for `esp-idf-sys`.

## Prerequisites

- **Rust**: ESP target toolchain matching `rust-toolchain.toml` (e.g. install with [espup](https://github.com/esp-rs/espup)).
- **ESP-IDF**: provided/fetched by the build via embuild; no separate manual install required for a typical `espup` flow.
- **espflash** on `PATH` (used as the Cargo runner in `.cargo/config.toml`).
- An **ESP32** (this project targets ESP32) connected over USB.

## Build, flash, and monitor

From the repository root:

```bash
./scripts/flash.sh
```

If the script is not executable:

```bash
chmod +x ./scripts/flash.sh
./scripts/flash.sh
```

The script runs `cargo clippy-check` (see `.cargo/config.toml` for the alias), then `cargo run`, which builds, flashes, and opens the serial monitor.

Equivalent manual steps:

```bash
cargo clippy-check
cargo build
cargo run
```

Release:

```bash
cargo build --release
cargo run --release
```

## Switching the active chapter

Point `src/main.rs` at the module you want, for example:

```rust
use esp32_tutorial::ch6_led_pixel::*;
```

Available modules are the `pub mod` entries in `src/chapters/mod.rs` (e.g. `ch1_blink_led`, `ch2_button_and_led`, …). `start()` already uses `PROJECT_NAME`, `setup`, and `update` from that import, so you only need to change the `use` line.
