use anyhow::Result;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;

struct State {
    led_pin: PinDriver<'static, Output>,
}

pub async fn run(peripherals: Peripherals) -> Result<()> {
    let mut state = setup(peripherals)?;

    loop {
        update(&mut state).await?;
        Timer::after(Duration::from_millis(10)).await;
    }
}

fn setup(peripherals: Peripherals) -> Result<State> {
    let state = State {
        led_pin: PinDriver::output(peripherals.pins.gpio4)?,
    };

    Ok(state)
}

async fn update(state: &mut State) -> Result<()> {
    state.led_pin.set_high()?;
    Timer::after(Duration::from_millis(500)).await;

    state.led_pin.set_low()?;
    Timer::after(Duration::from_millis(500)).await;

    Ok(())
}
