use anyhow::{Result, anyhow};
use embassy_time::{Duration, Instant, Timer};
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{InputOutput, Level, PinDriver};

const IDLE_BEFORE_START_MS: u64 = 2;
const START_LOW_MS: u64 = 20;

const ACK_WAIT_TIMEOUT: Duration = Duration::from_millis(3);
const BIT_SAMPLE_TIMEOUT: Duration = Duration::from_millis(2);

const BIT_THRESHOLD_US: u64 = 40;

pub async fn measure(pin: &mut PinDriver<'_, InputOutput>) -> Result<(f32, f32)> {
    handshake(pin).await?;

    let mut bits = [0u8; 40];

    for bit in bits.iter_mut() {
        wait_for_level(pin, Level::High, BIT_SAMPLE_TIMEOUT)?;

        let t0 = Instant::now();

        wait_for_level(pin, Level::Low, BIT_SAMPLE_TIMEOUT)?;

        let high_us = t0.elapsed().as_micros();

        *bit = u8::from(high_us > BIT_THRESHOLD_US);
    }

    let bytes = bits_to_bytes(&bits);

    if !checksum_ok(bytes) {
        return Err(anyhow!("checksum mismatch: {:?}", bytes));
    }

    Ok((
        f32::from(bytes[0]) + f32::from(bytes[1]) / 10.0,
        f32::from(bytes[2]) + f32::from(bytes[3]) / 10.0,
    ))
}

fn wait_for_level(pin: &PinDriver<'_, InputOutput>, level: Level, timeout: Duration) -> Result<()> {
    let start = Instant::now();

    while pin.get_level() != level {
        if start.elapsed() > timeout {
            return Err(anyhow!("timeout waiting for {:?}", level));
        }

        Ets::delay_us(1);
    }
    Ok(())
}

async fn handshake(pin: &mut PinDriver<'_, InputOutput>) -> Result<()> {
    pin.set_high()?;

    Timer::after(Duration::from_millis(IDLE_BEFORE_START_MS)).await;

    pin.set_low()?;

    Timer::after(Duration::from_millis(START_LOW_MS)).await;

    pin.set_high()?;

    wait_for_level(pin, Level::Low, ACK_WAIT_TIMEOUT)?;
    wait_for_level(pin, Level::High, ACK_WAIT_TIMEOUT)?;
    wait_for_level(pin, Level::Low, ACK_WAIT_TIMEOUT)?;

    Ok(())
}

fn bits_to_bytes(bits: &[u8; 40]) -> [u8; 5] {
    let mut out = [0u8; 5];

    for (i, bit) in bits.iter().enumerate() {
        let byte_index = i / 8;
        let bit_position = 7 - (i % 8);
        out[byte_index] |= bit << bit_position;
    }

    out
}

fn checksum_ok(bytes: [u8; 5]) -> bool {
    let sum: u16 = bytes[..4].iter().map(|&x| u16::from(x)).sum();
    sum & 0xff == u16::from(bytes[4])
}
