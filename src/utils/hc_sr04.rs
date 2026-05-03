use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::hal::delay::TickType;
use esp_idf_svc::hal::gpio::InputPin;
use esp_idf_svc::hal::rmt::config::{ReceiveConfig, RxChannelConfig};
use esp_idf_svc::hal::rmt::{RxChannelDriver, Symbol};
use esp_idf_svc::hal::units::Hertz;

pub const HC_SR04_RMT_RESOLUTION_HZ: Hertz = Hertz(1_000_000);

pub const ECHO_SIGNAL_RANGE_MIN: Duration = Duration::from_micros(10);
pub const ECHO_SIGNAL_RANGE_MAX: Duration = Duration::from_millis(40);
pub const RECEIVE_WAIT_TIMEOUT: Duration = Duration::from_millis(50);

pub fn init<'d>(pin: impl InputPin + 'd) -> Result<RxChannelDriver<'d>> {
    let config = RxChannelConfig {
        resolution: HC_SR04_RMT_RESOLUTION_HZ,
        ..Default::default()
    };

    let rx = RxChannelDriver::new(pin, &config)?;

    Ok(rx)
}

pub fn receive<'d>(driver: &mut RxChannelDriver<'d>, buf: &mut [Symbol]) -> Result<Vec<Symbol>> {
    let recv = ReceiveConfig {
        signal_range_min: ECHO_SIGNAL_RANGE_MIN,
        signal_range_max: ECHO_SIGNAL_RANGE_MAX,
        timeout: Some(TickType::from(RECEIVE_WAIT_TIMEOUT).into()),
        ..Default::default()
    };

    let n = driver.receive(buf, &recv)?;

    Ok(buf[..n].to_vec())
}
