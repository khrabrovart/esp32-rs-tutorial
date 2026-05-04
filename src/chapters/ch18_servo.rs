use crate::utils::button;
use crate::utils::ledc;
use anyhow::Result;
use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::ledc::LedcDriver;
use esp_idf_svc::hal::ledc::Resolution;
use esp_idf_svc::hal::peripherals::Peripherals;

pub const CHAPTER_NAME: &str = "ch18_servo";

const FREQUENCY: u32 = 50;
const RESOLUTION: Resolution = Resolution::Bits13;

const SERVO_DUTY_CHANGE: f32 = 0.01;
const SERVO_DUTY_MIN: f32 = 0.025;
const SERVO_DUTY_MAX: f32 = 0.125;

pub struct State {
    red_btn_pin: PinDriver<'static, Input>,
    green_btn_pin: PinDriver<'static, Input>,
    servo_ledc_channel: LedcDriver<'static>,
    servo_duty: f32,
}

pub async fn setup(peripherals: Peripherals) -> Result<State> {
    let (_, ledc_channel) = ledc::init(
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
        peripherals.pins.gpio5,
        FREQUENCY,
        RESOLUTION,
    )?;

    let state = State {
        red_btn_pin: PinDriver::input(peripherals.pins.gpio12, Pull::Floating)?,
        green_btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        servo_ledc_channel: ledc_channel,
        servo_duty: SERVO_DUTY_MIN,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    update_duty(state).await?;

    ledc::set_duty_percentage(&mut state.servo_ledc_channel, state.servo_duty)?;

    Ok(())
}

async fn update_duty(state: &mut State) -> Result<()> {
    let delta = if button::check_pressed(&state.red_btn_pin).await {
        SERVO_DUTY_CHANGE
    } else if button::check_pressed(&state.green_btn_pin).await {
        -SERVO_DUTY_CHANGE
    } else {
        0.0
    };

    state.servo_duty = (state.servo_duty + delta).clamp(SERVO_DUTY_MIN, SERVO_DUTY_MAX);

    let duty = state.servo_ledc_channel.get_duty();
    let max_duty = state.servo_ledc_channel.get_max_duty();

    log::info!(
        "duty: {:.3}/{:.3}, {:.1}ms",
        duty,
        max_duty,
        duty as f32 / max_duty as f32 * 1000.0 / FREQUENCY as f32
    );

    Ok(())
}
