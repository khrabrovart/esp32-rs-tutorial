use crate::utils::buttons;
use anyhow::Result;
use esp_idf_svc::hal::adc::attenuation::DB_12;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH6, ADCU1};
use esp_idf_svc::hal::gpio::{Input, PinDriver, Pull};
use esp_idf_svc::hal::ledc::config::{Resolution, TimerConfig};
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver, LowSpeed};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;

pub const CHAPTER_NAME: &str = "ch4_analog_and_pwm";

const RESOLUTION: Resolution = Resolution::Bits10;
const MIN_FREQUENCY: u32 = 10;
const MAX_FREQUENCY: u32 = 500;
const DIM_STEP: u32 = 10;

pub struct State {
    ledc_timer: LedcTimerDriver<'static, LowSpeed>,
    ledc_channel: LedcDriver<'static>,
    btn_pin: PinDriver<'static, Input>,
    adc_pin: AdcChannelDriver<'static, ADCCH6<ADCU1>, AdcDriver<'static, ADCU1>>,

    current_frequency: u32,
    button_pressed: bool,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let timer_config = TimerConfig::default()
        .frequency(MIN_FREQUENCY.Hz())
        .resolution(RESOLUTION);

    let ledc_timer = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;

    let ledc_channel = LedcDriver::new(
        peripherals.ledc.channel0,
        &ledc_timer,
        peripherals.pins.gpio4,
    )?;

    let adc_config = AdcChannelConfig {
        attenuation: DB_12,
        ..Default::default()
    };
    let adc = AdcDriver::new(peripherals.adc1)?;
    let adc_pin = AdcChannelDriver::new(adc, peripherals.pins.gpio34, &adc_config)?;

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

    if buttons::button_pressed(&state.btn_pin).await {
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

fn light_up(state: &mut State) -> Result<()> {
    let max_duty = state.ledc_channel.get_max_duty();

    log::info!("Lighting up LED with duty {}", max_duty);

    state.ledc_channel.set_duty(max_duty)?;

    Ok(())
}

fn update_frequency(state: &mut State) -> Result<()> {
    let adc_value = state.adc_pin.read_raw()?;

    let frequency =
        MIN_FREQUENCY + (adc_value as f32 / 4095.0 * (MAX_FREQUENCY - MIN_FREQUENCY) as f32) as u32;

    if state.current_frequency == frequency {
        return Ok(());
    }

    state.ledc_timer.set_frequency(frequency.Hz())?;
    state.current_frequency = frequency;

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
