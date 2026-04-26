use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::rmt::config::{MemoryAccess, TransmitConfig, TxChannelConfig};
use esp_idf_svc::hal::rmt::encoder::CopyEncoder;
use esp_idf_svc::hal::rmt::PinState;
use esp_idf_svc::hal::rmt::Symbol;
use esp_idf_svc::hal::rmt::TxChannelDriver;
use esp_idf_svc::hal::units::Hertz;
use std::vec::Vec;

pub const WS2812_RMT_RESOLUTION_HZ: Hertz = Hertz(10_000_000);

pub const WS2812_T0H: Duration = Duration::from_nanos(350);
pub const WS2812_T0L: Duration = Duration::from_nanos(800);
pub const WS2812_T1H: Duration = Duration::from_nanos(700);
pub const WS2812_T1L: Duration = Duration::from_nanos(600);
pub const WS2812_TRESET: Duration = Duration::from_micros(281);

#[derive(Clone, Copy)]
pub struct StripItem {
    g: u8,
    r: u8,
    b: u8,
}

impl StripItem {
    pub fn new(g: u8, r: u8, b: u8) -> Self {
        Self { g, r, b }
    }
}

pub fn write_strip(tx: &mut TxChannelDriver<'_>, buffer: &[StripItem]) -> Result<()> {
    let mut symbols: Vec<Symbol> = Vec::new();

    for item in buffer {
        push_byte(&mut symbols, item.g)?;
        push_byte(&mut symbols, item.r)?;
        push_byte(&mut symbols, item.b)?;
    }

    symbols.push(Symbol::new_with(
        WS2812_RMT_RESOLUTION_HZ,
        PinState::Low,
        WS2812_TRESET,
        PinState::High,
        Duration::from_nanos(1),
    )?);

    let encoder = CopyEncoder::new()?;
    let mut queue = tx.queue(core::iter::once(encoder));

    queue.push(&symbols, &TransmitConfig::default())?;

    Ok(())
}

pub fn init<'d, P>(pin: P) -> Result<TxChannelDriver<'d>>
where
    P: OutputPin + 'd,
{
    let config = TxChannelConfig {
        resolution: WS2812_RMT_RESOLUTION_HZ,
        memory_access: MemoryAccess::Indirect {
            memory_block_symbols: 64,
        },
        transaction_queue_depth: 4,
        ..Default::default()
    };

    let tx = TxChannelDriver::new(pin, &config)?;

    Ok(tx)
}

fn push_byte(symbols: &mut Vec<Symbol>, byte: u8) -> Result<()> {
    for i in 0..8 {
        let is_one = (byte & (0x80u8.wrapping_shr(i as u32))) != 0;

        let (th, tl) = if is_one {
            (WS2812_T1H, WS2812_T1L)
        } else {
            (WS2812_T0H, WS2812_T0L)
        };

        symbols.push(Symbol::new_with(
            WS2812_RMT_RESOLUTION_HZ,
            PinState::High,
            th,
            PinState::Low,
            tl,
        )?);
    }

    Ok(())
}
