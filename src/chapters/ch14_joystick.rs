use crate::utils::adc;
use crate::utils::button;
use crate::utils::ledc;
use anyhow::Result;
use embassy_executor::Spawner;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH5, ADCCH6, ADCU2};
use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::ledc::{LedcDriver, Resolution};
use esp_idf_svc::hal::peripherals::Peripherals;
use std::rc::Rc;

pub const PROJECT_NAME: &str = "ch14_joystick";

const FREQUENCY: u32 = 5000;
const RESOLUTION: Resolution = Resolution::Bits8;

pub struct State {
    ledc_channel_up: LedcDriver<'static>,
    ledc_channel_down: LedcDriver<'static>,
    potentiometer_y_pin: AdcChannelDriver<'static, ADCCH5<ADCU2>, Rc<AdcDriver<'static, ADCU2>>>,
    potentiometer_x_pin: AdcChannelDriver<'static, ADCCH6<ADCU2>, Rc<AdcDriver<'static, ADCU2>>>,
    led_pin: PinDriver<'static, Output>,
    joystick_pin: PinDriver<'static, Input>,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let (_, ledc_channel_up) = ledc::init(
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
        peripherals.pins.gpio25,
        FREQUENCY,
        RESOLUTION,
    )?;

    let (_, ledc_channel_down) = ledc::init(
        peripherals.ledc.timer1,
        peripherals.ledc.channel1,
        peripherals.pins.gpio27,
        FREQUENCY,
        RESOLUTION,
    )?;

    let adc2 = adc::init(peripherals.adc2)?;

    let potentiometer_y_pin = adc2.assign(peripherals.pins.gpio12)?;
    let potentiometer_x_pin = adc2.assign(peripherals.pins.gpio14)?;

    let state = State {
        ledc_channel_up,
        ledc_channel_down,
        potentiometer_y_pin,
        potentiometer_x_pin,
        led_pin: PinDriver::output(peripherals.pins.gpio26)?,
        joystick_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    let x = adc::read_normalized(&mut state.potentiometer_x_pin)?;
    let y = adc::read_normalized(&mut state.potentiometer_y_pin)?;

    ledc::set_duty_percentage(&mut state.ledc_channel_up, x)?;
    ledc::set_duty_percentage(&mut state.ledc_channel_down, y)?;

    let z = state.joystick_pin.is_high();

    if button::check_pressed(&state.joystick_pin).await {
        state.led_pin.set_high()?;
    } else {
        state.led_pin.set_low()?;
    }

    log::info!("x: {:.2}, y: {:.2}, z: {}", x, y, if z { 1 } else { 0 });

    Ok(())
}
