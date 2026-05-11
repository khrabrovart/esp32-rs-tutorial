use crate::utils::button;
use crate::utils::hd44780_i2c::HD44780I2cDriver;
use crate::utils::i2c;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::Duration;
use embassy_time::Instant;
use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch20_lcd1602";

const LCD_I2C_ADDR: u8 = 0x27;
const BAUD_RATE_HZ: u32 = 100_000;
const BACKLIGHT_ON: bool = true;

const COUNT_UPDATE_INTERVAL: Duration = Duration::from_millis(200);

pub struct State {
    red_btn_pin: PinDriver<'static, Input>,
    green_btn_pin: PinDriver<'static, Input>,
    lcd: HD44780I2cDriver<'static>,
    count: u32,
    last_count_update: Instant,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let i2c_driver = i2c::init(
        peripherals.i2c0,
        peripherals.pins.gpio14,
        peripherals.pins.gpio4,
        BAUD_RATE_HZ,
    )?;

    let mut lcd = HD44780I2cDriver::new(i2c_driver, LCD_I2C_ADDR, BACKLIGHT_ON).await?;
    lcd.write_line(0, "Hello, World!").await?;

    let state = State {
        red_btn_pin: PinDriver::input(peripherals.pins.gpio12, Pull::Floating)?,
        green_btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        lcd,
        count: 0,
        last_count_update: Instant::now(),
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    let count_updated = update_count(state).await?;

    if !count_updated {
        return Ok(());
    }

    update_lcd(state).await?;

    Ok(())
}

async fn update_count(state: &mut State) -> Result<bool> {
    if (Instant::now() - state.last_count_update) < COUNT_UPDATE_INTERVAL {
        return Ok(false);
    }

    let mut count_updated = false;

    if button::check_pressed(&state.red_btn_pin).await {
        state.count += 1;
        count_updated = true;
    }

    if button::check_pressed(&state.green_btn_pin).await {
        state.count -= 1;
        count_updated = true;
    }

    if count_updated {
        state.last_count_update = Instant::now();
    }

    log::info!("count: {}, updated: {}", state.count, count_updated);

    Ok(count_updated)
}

async fn update_lcd(state: &mut State) -> Result<()> {
    state.lcd.write_line(0, "Counting numbers").await?;

    state
        .lcd
        .write_line(1, &format!("Count: {}", state.count))
        .await?;

    Ok(())
}
