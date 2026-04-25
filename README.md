# ESP32 Rust Tutorial

A small ESP32 tutorial project.

## What it does

This project currently includes:

- `ch1_blink_led` - blinks LED on `GPIO4`
- `ch2_button_and_led` - turns LED on `GPIO4` on/off from a button on `GPIO13` (currently used by `src/main.rs`)

Core libraries:

- [esp-idf-svc](https://github.com/esp-rs/esp-idf-svc)
- [embassy](https://github.com/embassy-rs/embassy)
- [anyhow](https://github.com/dtolnay/anyhow)

## Prerequisites

- ESP Rust toolchain (for `rust-toolchain.toml`, `channel = "esp"`), e.g. via [espup](https://github.com/esp-rs/espup)
- ESP-IDF build environment (configured through `build.rs` / `embuild`)
- `espflash` available in `PATH` (used as Cargo runner in `.cargo/config.toml`)
- ESP32 board connected over USB

## Run with script

From the project root:

```bash
./scripts/flash.sh
```

If needed:

```bash
chmod +x ./scripts/flash.sh
./scripts/flash.sh
```

The script runs:

1. `cargo clippy-check` (alias: `clippy --all-targets -- -D warnings`)
2. `cargo run` (builds, flashes, and starts monitor via `espflash flash --monitor`)

## Manual commands

```bash
cargo clippy-check
cargo build
cargo run
```

## Switching chapters

To change the active lesson, update `src/main.rs` to call the desired module:

```rust
if let Err(e) = esp32_tutorial::ch1_blink_led::run(peripherals).await {
    log::error!("Critical error: {:?}", e);
}
```

or:

```rust
if let Err(e) = esp32_tutorial::ch2_button_and_led::run(peripherals).await {
    log::error!("Critical error: {:?}", e);
}
```

## Release build

```bash
cargo build --release
cargo run --release
```
