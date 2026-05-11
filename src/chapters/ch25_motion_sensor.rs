use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch25_motion_sensor";

const STARTUP_SETTLE: Duration = Duration::from_secs(60);

pub struct State {
    led_pin: PinDriver<'static, Output>,
    motion_sensor_pin: PinDriver<'static, Input>,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let led_pin = PinDriver::output(peripherals.pins.gpio14)?;
    let motion_sensor_pin = PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?;

    Timer::after(STARTUP_SETTLE).await;

    Ok(State {
        led_pin,
        motion_sensor_pin,
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    if state.motion_sensor_pin.is_high() {
        state.led_pin.set_high()?;
    } else {
        state.led_pin.set_low()?;
    }

    Ok(())
}
