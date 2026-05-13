use crate::utils::i2c;
use anyhow::Result;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::i2c::I2cDriver;

const REGISTER_SELECT: u8 = 1 << 0;
const ENABLE: u8 = 1 << 2;
const BACKLIGHT_ON: u8 = 1 << 3;
const ROW_WIDTH: usize = 16;
const ROW_0: u8 = 0x80;
const ROW_1: u8 = 0xC0;

pub struct HD44780I2cDriver<'d> {
    i2c: I2cDriver<'d>,
    address: u8,
    backlight_on: bool,
}

impl<'d> HD44780I2cDriver<'d> {
    pub async fn new(i2c: I2cDriver<'d>, address: u8, backlight_on: bool) -> Result<Self> {
        let mut driver = Self {
            i2c,
            address,
            backlight_on,
        };

        driver.init().await?;

        Ok(driver)
    }

    pub async fn write_line(&mut self, line: u8, text: &str) -> Result<()> {
        let ddram = match line {
            0 => ROW_0,
            1 => ROW_1,
            _ => {
                log::error!("Line must be 0 or 1");
                return Ok(());
            }
        };

        self.write_byte(ddram, false).await?;

        let bytes = text.as_bytes();

        for &b in bytes.iter().take(ROW_WIDTH) {
            self.write_byte(b, true).await?;
        }

        for _ in bytes.len()..ROW_WIDTH {
            self.write_byte(b' ', true).await?;
        }

        Ok(())
    }

    async fn init(&mut self) -> Result<()> {
        Timer::after(Duration::from_millis(50)).await;

        for _ in 0..3 {
            self.write_nibble(0x03, false).await?;
            Timer::after(Duration::from_millis(5)).await;
        }

        self.write_nibble(0x02, false).await?;

        Timer::after(Duration::from_millis(1)).await;

        self.write_byte(0x28, false).await?;
        self.write_byte(0x0C, false).await?;
        self.write_byte(0x06, false).await?;
        self.write_byte(0x01, false).await?;

        Timer::after(Duration::from_millis(2)).await;

        Ok(())
    }

    async fn write_byte(&mut self, byte: u8, is_data: bool) -> Result<()> {
        self.write_nibble((byte >> 4) & 0x0F, is_data).await?;
        self.write_nibble(byte & 0x0F, is_data).await?;

        Ok(())
    }

    async fn write_nibble(&mut self, nibble: u8, is_data: bool) -> Result<()> {
        let base = self.expander_base(nibble, is_data);
        self.pulse_enable(base).await
    }

    fn expander_base(&self, nibble: u8, is_data: bool) -> u8 {
        let mut byte = (nibble & 0x0F) << 4;

        if is_data {
            byte |= REGISTER_SELECT;
        }

        if self.backlight_on {
            byte |= BACKLIGHT_ON;
        }

        byte
    }

    async fn pulse_enable(&mut self, byte: u8) -> Result<()> {
        let byte_with_enable_low = byte;
        let byte_with_enable_high = byte | ENABLE;

        i2c::write(&mut self.i2c, self.address, &[byte_with_enable_high])?;

        Timer::after(Duration::from_micros(50)).await;

        i2c::write(&mut self.i2c, self.address, &[byte_with_enable_low])?;

        Timer::after(Duration::from_micros(100)).await;

        Ok(())
    }
}
