use crate::utils::adc;
use crate::utils::button;
use crate::utils::ws2812::{self, StripItem};
use anyhow::Result;
use embassy_time::{Duration, Instant};
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH6, ADCU1};
use esp_idf_svc::hal::gpio::{Input, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::rmt::TxChannelDriver;

pub const CHAPTER_NAME: &str = "ch6_led_pixel";

const NUM_LEDS: usize = 8;
const ACTIVE_BRIGHTNESS: u8 = 32;
const MIN_STEP_MS: u64 = 5;
const MAX_STEP_MS: u64 = 1000;

pub struct State {
    ws2812_pin: TxChannelDriver<'static>,
    btn_pin: PinDriver<'static, Input>,
    adc_pin: AdcChannelDriver<'static, ADCCH6<ADCU1>, AdcDriver<'static, ADCU1>>,

    active_index: usize,
    last_advance: Instant,
    step_interval: Duration,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let ws2812_pin = ws2812::init(peripherals.pins.gpio4)?;

    let adc_pin = adc::init(peripherals.adc1, peripherals.pins.gpio34)?;

    let mut state = State {
        ws2812_pin,
        btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        adc_pin,
        active_index: 0,
        last_advance: Instant::now(),
        step_interval: Duration::from_millis(MIN_STEP_MS),
    };

    update_strip(&mut state)?;

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    update_step_interval(state)?;

    if button::button_pressed(&state.btn_pin).await {
        return Ok(());
    }

    let should_advance =
        Instant::now().saturating_duration_since(state.last_advance) >= state.step_interval;

    if should_advance {
        advance_led(state)?;
    }

    Ok(())
}

fn update_step_interval(state: &mut State) -> Result<()> {
    let ms = adc::read_mapped(&mut state.adc_pin, MAX_STEP_MS as f32, MIN_STEP_MS as f32)? as u64;
    state.step_interval = Duration::from_millis(ms);

    Ok(())
}

fn advance_led(state: &mut State) -> Result<()> {
    state.active_index = (state.active_index + 1) % NUM_LEDS;
    state.last_advance = Instant::now();

    update_strip(state)
}

fn update_strip(state: &mut State) -> Result<()> {
    let mut buffer: Vec<StripItem> = Vec::new();

    for i in 0..NUM_LEDS {
        if i == state.active_index {
            buffer.push(StripItem::new(ACTIVE_BRIGHTNESS, 0, 0));
        } else {
            buffer.push(StripItem::new(0, 0, 0));
        }
    }

    ws2812::write_strip(&mut state.ws2812_pin, &buffer)
}
