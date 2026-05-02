use anyhow::Result;
use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::{config::Config, I2c, I2cDriver};
use esp_idf_svc::hal::units::Hertz;

pub fn init<'d>(
    i2c: impl I2c + 'd,
    sda: impl InputPin + OutputPin + 'd,
    scl: impl InputPin + OutputPin + 'd,
    baudrate_hz: u32,
) -> Result<I2cDriver<'d>> {
    let config = Config::new().baudrate(Hertz(baudrate_hz));
    Ok(I2cDriver::new(i2c, sda, scl, &config)?)
}

pub fn send(i2c: &mut I2cDriver<'_>, addr: u8, data: &[u8]) -> Result<()> {
    i2c.write(addr, data, BLOCK)?;
    Ok(())
}
