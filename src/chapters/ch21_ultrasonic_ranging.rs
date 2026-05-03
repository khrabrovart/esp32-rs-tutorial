use crate::utils::hd44780_i2c::HD44780I2cDriver;
use crate::utils::i2c;
use anyhow::Result;
use embassy_time::{Duration, Instant, Timer};
use esp_idf_svc::hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const CHAPTER_NAME: &str = "ch21_ultrasonic_ranging";

const LCD_I2C_ADDR: u8 = 0x27;
const BAUD_RATE_HZ: u32 = 100_000;
const BACKLIGHT_ON: bool = true;

const DISTANCE_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

const ECHO_RISING_TIMEOUT: Duration = Duration::from_millis(30);
const ECHO_PULSE_MAX: Duration = Duration::from_millis(40);

const CM_PER_US_ROUND_TRIP_HALF: f32 = 0.0343 / 2.0;

pub struct State {
    lcd: HD44780I2cDriver<'static>,
    trigger_pin: PinDriver<'static, Output>,
    echo_pin: PinDriver<'static, Input>,
    distance: f32,
    last_distance_update: Instant,
}

pub async fn setup(peripherals: Peripherals) -> Result<State> {
    let i2c_driver = i2c::init(
        peripherals.i2c0,
        peripherals.pins.gpio14,
        peripherals.pins.gpio4,
        BAUD_RATE_HZ,
    )?;

    let lcd = HD44780I2cDriver::new(i2c_driver, LCD_I2C_ADDR, BACKLIGHT_ON).await?;

    let trigger_pin = PinDriver::output(peripherals.pins.gpio12)?;
    let echo_pin = PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?;

    let state = State {
        lcd,
        trigger_pin,
        echo_pin,
        distance: 0.0,
        last_distance_update: Instant::now(),
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    update_distance(state).await?;

    update_lcd(state).await?;

    Ok(())
}

async fn update_distance(state: &mut State) -> Result<()> {
    if (Instant::now() - state.last_distance_update) < DISTANCE_UPDATE_INTERVAL {
        return Ok(());
    }

    state.distance = measure_distance(&mut state.trigger_pin, &state.echo_pin).await?;
    state.last_distance_update = Instant::now();

    log::info!("distance: {:.3}cm", state.distance);

    Ok(())
}

async fn measure_distance(
    trigger_pin: &mut PinDriver<'static, Output>,
    echo_pin: &PinDriver<'static, Input>,
) -> Result<f32> {
    trigger_pin.set_low()?;

    Timer::after(Duration::from_micros(2)).await;

    trigger_pin.set_high()?;

    Timer::after(Duration::from_micros(10)).await;

    trigger_pin.set_low()?;

    let wait_start = Instant::now();

    while echo_pin.is_low() {
        if Instant::now().saturating_duration_since(wait_start) > ECHO_RISING_TIMEOUT {
            log::error!("echo rising edge timeout");
            return Ok(0.0);
        }
    }

    let pulse_start = Instant::now();

    while echo_pin.is_high() {
        let elapsed = Instant::now().saturating_duration_since(pulse_start);

        if elapsed > ECHO_PULSE_MAX {
            log::error!("echo pulse too long");
            return Ok(0.0);
        }
    }

    let pulse_end = Instant::now();
    let pulse_us = pulse_end.saturating_duration_since(pulse_start).as_micros() as f32;

    Ok(pulse_us * CM_PER_US_ROUND_TRIP_HALF)
}

async fn update_lcd(state: &mut State) -> Result<()> {
    state.lcd.write_line(0, "Distance-meter").await?;

    state
        .lcd
        .write_line(1, &format!("{:.0}cm", state.distance))
        .await?;

    Ok(())
}
