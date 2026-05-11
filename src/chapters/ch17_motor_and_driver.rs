use crate::utils::button;
use crate::utils::ledc;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::Duration;
use embassy_time::Instant;
use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::ledc::LedcDriver;
use esp_idf_svc::hal::ledc::Resolution;
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch17_motor_and_driver";

const FREQUENCY: u32 = 1000;
const RESOLUTION: Resolution = Resolution::Bits10;

const MOTOR_SPEED_UPDATE_INTERVAL: Duration = Duration::from_millis(50);
const MOTOR_SPEED_INCREMENT: f32 = 0.025;
const MOTOR_SPEED_DECREMENT: f32 = -0.015;

pub struct State {
    btn_pin: PinDriver<'static, Input>,
    switch_pin: PinDriver<'static, Input>,
    motor_in_1_pin: PinDriver<'static, Output>,
    motor_in_2_pin: PinDriver<'static, Output>,
    motor_ledc_channel: LedcDriver<'static>,
    motor_speed: f32,
    last_motor_speed_update: Instant,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let (_, ledc_channel) = ledc::init(
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
        peripherals.pins.gpio14,
        FREQUENCY,
        RESOLUTION,
    )?;

    let state = State {
        btn_pin: PinDriver::input(peripherals.pins.gpio19, Pull::Floating)?,
        switch_pin: PinDriver::input(peripherals.pins.gpio18, Pull::Floating)?,
        motor_in_1_pin: PinDriver::output(peripherals.pins.gpio12)?,
        motor_in_2_pin: PinDriver::output(peripherals.pins.gpio13)?,
        motor_ledc_channel: ledc_channel,
        motor_speed: 0.0,
        last_motor_speed_update: Instant::now(),
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    update_motor_speed(state).await?;
    update_motor_direction(state).await?;

    ledc::set_duty_percentage(&mut state.motor_ledc_channel, state.motor_speed)?;

    Ok(())
}

async fn update_motor_speed(state: &mut State) -> Result<()> {
    if (Instant::now() - state.last_motor_speed_update) < MOTOR_SPEED_UPDATE_INTERVAL {
        return Ok(());
    }

    let delta = if button::check_pressed(&state.btn_pin).await {
        MOTOR_SPEED_INCREMENT
    } else {
        MOTOR_SPEED_DECREMENT
    };

    state.motor_speed = (state.motor_speed + delta).clamp(0.0, 1.0);
    state.last_motor_speed_update = Instant::now();

    Ok(())
}

async fn update_motor_direction(state: &mut State) -> Result<()> {
    if button::check_pressed(&state.switch_pin).await {
        state.motor_in_1_pin.set_high()?;
        state.motor_in_2_pin.set_low()?;
    } else {
        state.motor_in_1_pin.set_low()?;
        state.motor_in_2_pin.set_high()?;
    }

    Ok(())
}
