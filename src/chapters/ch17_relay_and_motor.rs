use crate::utils::button;
use anyhow::Result;
use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::peripherals::Peripherals;

pub const CHAPTER_NAME: &str = "ch17_relay_and_motor";

pub struct State {
    btn_pin: PinDriver<'static, Input>,
    relay_pin: PinDriver<'static, Output>,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let state = State {
        btn_pin: PinDriver::input(peripherals.pins.gpio14, Pull::Floating)?,
        relay_pin: PinDriver::output(peripherals.pins.gpio13)?,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    if button::check_pressed(&state.btn_pin).await {
        state.relay_pin.set_high()?;
        log::info!("relay on");
    } else {
        state.relay_pin.set_low()?;
        log::info!("relay off");
    }

    Ok(())
}
