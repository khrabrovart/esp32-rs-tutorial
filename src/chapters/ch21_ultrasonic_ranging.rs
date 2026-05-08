use crate::utils::hc_sr04_rmt::HCSR04Driver;
use crate::utils::hd44780_i2c::HD44780I2cDriver;
use crate::utils::i2c;
use anyhow::Result;
use embassy_time::{Duration, Instant};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch21_ultrasonic_ranging";

const LCD_I2C_ADDR: u8 = 0x27;
const BAUD_RATE_HZ: u32 = 100_000;
const BACKLIGHT_ON: bool = true;

const DISTANCE_UPDATE_INTERVAL: Duration = Duration::from_millis(200);

pub struct State {
    lcd: HD44780I2cDriver<'static>,
    hc_sr04: HCSR04Driver<'static>,
    distance_cm: Option<f32>,
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
    let hc_sr04 = HCSR04Driver::init(peripherals.pins.gpio12, peripherals.pins.gpio13)?;

    let state = State {
        lcd,
        hc_sr04,
        distance_cm: None,
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

    let distance_mm = match state.hc_sr04.measure_mm().await {
        Ok(mm) if mm > 0.0 => Some(mm),
        Ok(_) => {
            log::error!("could not parse distance measurement");
            None
        }
        Err(e) => {
            log::error!("distance measurement error: {e:?}");
            None
        }
    };

    state.last_distance_update = Instant::now();
    state.distance_cm = distance_mm.map(|mm| mm / 10.0);

    if let Some(distance_cm) = state.distance_cm {
        log::info!("distance: {:.1}cm", distance_cm);
    }

    Ok(())
}

async fn update_lcd(state: &mut State) -> Result<()> {
    state.lcd.write_line(0, "Distance-meter").await?;

    let distance_text = state
        .distance_cm
        .map_or("--- cm".to_string(), |cm| format!("{:>3.0} cm", cm));

    state.lcd.write_line(1, &distance_text).await?;

    Ok(())
}
