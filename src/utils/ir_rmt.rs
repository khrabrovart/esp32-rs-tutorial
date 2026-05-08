use anyhow::Result;
use core::time::Duration;
use embassy_time::{Duration as EmbassyDuration, Timer};
use esp_idf_svc::hal::delay::TickType;
use esp_idf_svc::hal::gpio::InputPin;
use esp_idf_svc::hal::rmt::config::{MemoryAccess, ReceiveConfig, RxChannelConfig};
use esp_idf_svc::hal::rmt::{RxChannelDriver, Symbol};
use esp_idf_svc::hal::units::Hertz;
use esp_idf_svc::sys::ESP_ERR_TIMEOUT;

const IR_RMT_RESOLUTION_HZ: Hertz = Hertz(1_000_000);

const SIGNAL_RANGE_MIN: Duration = Duration::from_nanos(3000);
const SIGNAL_RANGE_MAX: Duration = Duration::from_millis(12);

const SYMBOL_BUFFER_LENGTH: usize = 64;

pub struct IRReceiverDriver<'d> {
    rx: RxChannelDriver<'d>,
}

impl<'d> IRReceiverDriver<'d> {
    pub fn init(data_pin: impl InputPin + 'd) -> Result<Self> {
        let config = RxChannelConfig {
            resolution: IR_RMT_RESOLUTION_HZ,
            memory_access: MemoryAccess::Indirect {
                memory_block_symbols: SYMBOL_BUFFER_LENGTH,
            },
            ..Default::default()
        };

        let rx = RxChannelDriver::new(data_pin, &config)?;

        Ok(Self { rx })
    }

    pub async fn receive(&mut self) -> Result<Vec<Symbol>> {
        let config = ReceiveConfig {
            signal_range_min: SIGNAL_RANGE_MIN,
            signal_range_max: SIGNAL_RANGE_MAX,
            timeout: Some(TickType::new_millis(10).ticks()),
            ..Default::default()
        };

        let mut buffer = [Symbol::default(); SYMBOL_BUFFER_LENGTH];

        loop {
            match self.rx.receive(&mut buffer, &config) {
                Ok(n) => return Ok(buffer[..n].to_vec()),
                Err(e) if e.code() == ESP_ERR_TIMEOUT as i32 => {
                    Timer::after(EmbassyDuration::from_ticks(0)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}
