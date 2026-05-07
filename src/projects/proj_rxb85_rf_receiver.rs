use crate::utils::button;
use crate::utils::radio::RFReceiverDriver;
use anyhow::Result;
use esp_idf_svc::hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::rmt::{PinState, Symbol};
use esp_idf_svc::hal::units::Hertz;

pub const PROJECT_NAME: &str = "proj_rxb85_rf_receiver";

const RF_RMT_RESOLUTION_HZ: Hertz = Hertz(1_000_000);

pub struct State {
    led_pin: PinDriver<'static, Output>,
    btn_pin: PinDriver<'static, Input>,
    radio: RFReceiverDriver<'static>,
}

pub async fn setup(peripherals: Peripherals) -> Result<State> {
    let led_pin = PinDriver::output(peripherals.pins.gpio18)?;
    let btn_pin = PinDriver::input(peripherals.pins.gpio4, Pull::Floating)?;
    let radio = RFReceiverDriver::init(peripherals.pins.gpio14)?;

    Ok(State {
        led_pin,
        btn_pin,
        radio,
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    if button::check_pressed(&state.btn_pin).await {
        log::info!("Activating RF receiver");

        state.led_pin.set_high()?;

        let symbols = state.radio.receive().await?;

        state.led_pin.set_low()?;

        log_symbols(&symbols);
    }

    Ok(())
}

fn log_symbols(symbols: &[Symbol]) {
    if symbols.is_empty() {
        log::info!("No symbols received");
        return;
    }

    log::info!("Captured {} symbols", symbols.len());

    for (sym_idx, symbol) in symbols.iter().enumerate() {
        for pulse in [symbol.level0(), symbol.level1()] {
            let pulse_duration = pulse.ticks.duration(RF_RMT_RESOLUTION_HZ);

            if pulse_duration.is_zero() {
                continue;
            }

            let level = match pulse.pin_state {
                PinState::High => "HIGH",
                PinState::Low => "LOW",
            };

            let duration_us = pulse_duration.as_micros();

            log::info!(
                "  [{}] {:>4} | {:>5} µs {}",
                sym_idx,
                level,
                duration_us,
                if duration_us > 1000 { "<--->" } else { "" }
            );
        }
    }
}
