use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::hal::gpio::InputPin;
use esp_idf_svc::hal::rmt::config::{MemoryAccess, ReceiveConfig, RxChannelConfig};
use esp_idf_svc::hal::rmt::{RxChannelDriver, Symbol};
use esp_idf_svc::hal::units::Hertz;

const RF_RMT_RESOLUTION_HZ: Hertz = Hertz(1_000_000);

const SIGNAL_RANGE_MIN: Duration = Duration::from_nanos(3000);
const SIGNAL_RANGE_MAX: Duration = Duration::from_micros(65_000);

const SYMBOL_BUFFER_LENGTH: usize = 128;

pub struct RFReceiverDriver<'d> {
    rx: RxChannelDriver<'d>,
}

impl<'d> RFReceiverDriver<'d> {
    pub fn init(data_pin: impl InputPin + 'd) -> Result<Self> {
        let config = RxChannelConfig {
            resolution: RF_RMT_RESOLUTION_HZ,
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
            timeout: None,
            ..Default::default()
        };

        let mut buffer = [Symbol::default(); SYMBOL_BUFFER_LENGTH];

        let n = self.rx.receive_async(&mut buffer, &config).await?;

        Ok(buffer[..n].to_vec())
    }
}
