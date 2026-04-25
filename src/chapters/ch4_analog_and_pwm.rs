use crate::utils::buttons;
use anyhow::Result;
use esp_idf_svc::hal::gpio::{Input, PinDriver, Pull};
use esp_idf_svc::hal::ledc::config::{Resolution, TimerConfig};
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::FromValueType;

pub const CHAPTER_NAME: &str = "ch4_analog_and_pwm";

const RESOLUTION: Resolution = Resolution::Bits8;
const FREQUENCY: u32 = 25;
const DIM_STEP: u32 = 10;

pub struct State {
    ledc_channel: LedcDriver<'static>,
    btn_pin: PinDriver<'static, Input>,

    button_pressed: bool,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let timer_config = TimerConfig::default()
        .frequency(FREQUENCY.kHz().into())
        .resolution(RESOLUTION);

    let ledc_timer = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;

    let ledc_channel = LedcDriver::new(
        peripherals.ledc.channel0,
        &ledc_timer,
        peripherals.pins.gpio4,
    )?;

    let state = State {
        ledc_channel,
        btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        button_pressed: false,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
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
