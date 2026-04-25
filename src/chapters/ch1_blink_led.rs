use anyhow::Result;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const CHAPTER_NAME: &str = "ch1_blink_led";

pub struct State {
    led_pin: PinDriver<'static, Output>,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let state = State {
        led_pin: PinDriver::output(peripherals.pins.gpio4)?,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    state.led_pin.set_high()?;
    Timer::after(Duration::from_millis(500)).await;

    state.led_pin.set_low()?;
    Timer::after(Duration::from_millis(500)).await;

    Ok(())
}
