use anyhow::Result;
use core::time::Duration;
use embassy_time::{Duration as EmbassyDuration, Timer};
use esp_idf_svc::hal::gpio::{InputPin, Output, OutputPin, PinDriver};
use esp_idf_svc::hal::rmt::config::{ReceiveConfig, RxChannelConfig};
use esp_idf_svc::hal::rmt::{PinState, RxChannelDriver, Symbol};
use esp_idf_svc::hal::units::Hertz;

const HC_SR04_RMT_RESOLUTION_HZ: Hertz = Hertz(1_000_000);

const TRIGGER_PRE_PULSE_LOW_US: u64 = 2;
const TRIGGER_PULSE_HIGH_US: u64 = 10;

const ECHO_SIGNAL_RANGE_MIN: Duration = Duration::from_micros(2);
const ECHO_SIGNAL_RANGE_MAX: Duration = Duration::from_millis(40);

const SYMBOL_BUFFER_LENGTH: usize = 64;

const MM_PER_MICROSECOND_ECHO: f32 = (0.0343 / 2.0) * 10.0;

pub struct HCSR04Driver<'d> {
    trigger_pin: PinDriver<'d, Output>,
    echo_rx: RxChannelDriver<'d>,
}

impl<'d> HCSR04Driver<'d> {
    pub fn init(trigger_pin: impl OutputPin + 'd, echo_pin: impl InputPin + 'd) -> Result<Self> {
        let trigger_pin = PinDriver::output(trigger_pin)?;

        let config = RxChannelConfig {
            resolution: HC_SR04_RMT_RESOLUTION_HZ,
            ..Default::default()
        };

        let echo_rx = RxChannelDriver::new(echo_pin, &config)?;

        Ok(Self {
            trigger_pin,
            echo_rx,
        })
    }

    pub async fn measure_mm(&mut self) -> Result<f32> {
        self.trigger().await?;
        let symbols = self.receive().await?;
        let pulse_us = Self::decode(&symbols).unwrap_or(0.0);

        Ok(pulse_us * MM_PER_MICROSECOND_ECHO)
    }

    async fn trigger(&mut self) -> Result<()> {
        self.trigger_pin.set_low()?;

        Timer::after(EmbassyDuration::from_micros(TRIGGER_PRE_PULSE_LOW_US)).await;

        self.trigger_pin.set_high()?;

        Timer::after(EmbassyDuration::from_micros(TRIGGER_PULSE_HIGH_US)).await;

        self.trigger_pin.set_low()?;

        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<Symbol>> {
        let recv = ReceiveConfig {
            signal_range_min: ECHO_SIGNAL_RANGE_MIN,
            signal_range_max: ECHO_SIGNAL_RANGE_MAX,
            timeout: None,
            ..Default::default()
        };

        let mut buffer = [Symbol::default(); SYMBOL_BUFFER_LENGTH];
        let n = self.echo_rx.receive_async(&mut buffer, &recv).await?;

        Ok(buffer[..n].to_vec())
    }

    fn decode(symbols: &[Symbol]) -> Option<f32> {
        let mut in_pulse = false;
        let mut echo = Duration::ZERO;
        let resolution = HC_SR04_RMT_RESOLUTION_HZ;

        for symbol in symbols {
            for pulse in [symbol.level0(), symbol.level1()] {
                let pulse_duration = pulse.ticks.duration(resolution);

                if pulse_duration.is_zero() {
                    continue;
                }

                if !in_pulse {
                    if pulse.pin_state == PinState::High {
                        in_pulse = true;
                        echo = pulse_duration;
                    }
                } else if pulse.pin_state == PinState::High {
                    echo += pulse_duration;
                } else {
                    return Some(echo.as_micros() as f32);
                }
            }
        }

        if in_pulse {
            Some(echo.as_micros() as f32)
        } else {
            None
        }
    }
}
