use crate::utils::adc;
use crate::utils::button;
use crate::utils::ledc;
use anyhow::Result;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH6, ADCU1};
use esp_idf_svc::hal::gpio::{Input, PinDriver, Pull};
use esp_idf_svc::hal::ledc::config::Resolution;
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver, LowSpeed};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;
use std::rc::Rc;

pub const PROJECT_NAME: &str = "ch4_analog_and_pwm";

const RESOLUTION: Resolution = Resolution::Bits10;
const MIN_FREQUENCY: u32 = 10;
const MAX_FREQUENCY: u32 = 500;
const DIM_STEP: u32 = 10;

pub struct State {
    ledc_timer: LedcTimerDriver<'static, LowSpeed>,
    ledc_channel: LedcDriver<'static>,
    btn_pin: PinDriver<'static, Input>,
    adc_pin: AdcChannelDriver<'static, ADCCH6<ADCU1>, Rc<AdcDriver<'static, ADCU1>>>,

    current_frequency: u32,
    button_pressed: bool,
}

pub async fn setup(peripherals: Peripherals) -> Result<State> {
    let (ledc_timer, ledc_channel) = ledc::init(
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
        peripherals.pins.gpio4,
        MIN_FREQUENCY,
        RESOLUTION,
    )?;

    let adc1 = adc::init(peripherals.adc1)?;

    let adc_pin = adc1.assign(peripherals.pins.gpio34)?;

    let state = State {
        ledc_timer,
        ledc_channel,
        btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        adc_pin,
        current_frequency: MIN_FREQUENCY,
        button_pressed: false,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    update_frequency(state)?;
    update_led(state)?;

    if button::check_pressed(&state.btn_pin).await {
        if state.button_pressed {
            return Ok(());
        }

        state.button_pressed = true;

        light_up(state)?;
    } else {
        state.button_pressed = false;
    }

    Ok(())
}

fn update_frequency(state: &mut State) -> Result<()> {
    let frequency = adc::read_mapped(
        &mut state.adc_pin,
        MIN_FREQUENCY as f32,
        MAX_FREQUENCY as f32,
    )? as u32;

    if state.current_frequency == frequency {
        return Ok(());
    }

    state.ledc_timer.set_frequency(frequency.Hz())?;
    state.current_frequency = frequency;

    Ok(())
}

fn light_up(state: &mut State) -> Result<()> {
    let max_duty = state.ledc_channel.get_max_duty();

    log::info!("Lighting up LED with duty {}", max_duty);

    state.ledc_channel.set_duty(max_duty)?;

    Ok(())
}

fn update_led(state: &mut State) -> Result<()> {
    let duty = state.ledc_channel.get_duty();

    if duty == 0 {
        return Ok(());
    }

    let new_duty = duty.saturating_sub(DIM_STEP);

    log::info!("Dimming LED with duty {}", new_duty);

    state.ledc_channel.set_duty(new_duty)?;

    Ok(())
}
