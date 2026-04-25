# ESP32 Rust Tutorial

A set of ESP32 Rust tutorials.

## What it does

This project includes:

- `ch1_blink_led` — blinks the LED on `GPIO4`
- `ch2_button_and_led` — holds the LED on `GPIO4` on while a button on `GPIO13` is pressed
- `ch2_2_mini_table_lamp` — toggles the LED on `GPIO4` on each press of the button on `GPIO13` (edge-style behavior)

`src/main.rs` currently runs `ch2_2_mini_table_lamp`.

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

Each chapter exposes `CHAPTER_NAME`, `setup`, and `update`. `main.rs` takes peripherals once, builds state with `setup`, then loops calling `update` (with a short delay between iterations).

To run another lesson, change the glob import in `src/main.rs` to the module you want:

```rust
use esp32_tutorial::ch1_blink_led::*;
```

or:

```rust
use esp32_tutorial::ch2_button_and_led::*;
```

or:

```rust
use esp32_tutorial::ch2_2_mini_table_lamp::*;
```

`start()` in `main.rs` already uses `CHAPTER_NAME`, `setup`, and `update` from that import, so no other code changes are required when switching.

## Release build

```bash
cargo build --release
cargo run --release
```
