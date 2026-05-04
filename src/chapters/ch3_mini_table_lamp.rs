use crate::utils::button;
use anyhow::Result;
use esp_idf_svc::hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const CHAPTER_NAME: &str = "ch3_mini_table_lamp";

pub struct State {
    led_pin: PinDriver<'static, Output>,
    btn_pin: PinDriver<'static, Input>,

    led_on: bool,
    button_pressed: bool,
}

pub async fn setup(peripherals: Peripherals) -> Result<State> {
    let state = State {
        led_pin: PinDriver::output(peripherals.pins.gpio4)?,
        btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        led_on: false,
        button_pressed: false,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    if button::check_pressed(&state.btn_pin).await {
        if state.button_pressed {
            return Ok(());
        }

        state.button_pressed = true;

        toggle_led(state)?;
    } else {
        state.button_pressed = false;
    }

    Ok(())
}

fn toggle_led(state: &mut State) -> Result<()> {
    state.led_on = !state.led_on;

    log::info!("Turning LED {}", if state.led_on { "on" } else { "off" });

    if state.led_on {
        state.led_pin.set_high()?;
    } else {
        state.led_pin.set_low()?;
    }

    Ok(())
}
