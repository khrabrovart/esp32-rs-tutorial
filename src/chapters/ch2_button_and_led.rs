use anyhow::Result;
use embassy_executor::Spawner;
use esp_idf_svc::hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch2_button_and_led";

pub struct State {
    led_pin: PinDriver<'static, Output>,
    btn_pin: PinDriver<'static, Input>,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let state = State {
        led_pin: PinDriver::output(peripherals.pins.gpio4)?,
        btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    if button_pressed(state) {
        state.led_pin.set_high()?;
    } else {
        state.led_pin.set_low()?;
    }

    Ok(())
}

fn button_pressed(state: &State) -> bool {
    !state.btn_pin.is_high()
}
