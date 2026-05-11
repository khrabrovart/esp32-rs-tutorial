use crate::utils::adc;
use crate::utils::hc595;
use crate::utils::hc595::Hc595Driver;
use crate::utils::hc595::ShiftOrder;
use crate::utils::thermo;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::Timer;
use embassy_time::{Duration, Instant};
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH8, ADCU2};
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

pub const PROJECT_NAME: &str = "ch16_4_digit_7_segment";

const TEMPERATURE_UPDATE_INTERVAL: Duration = Duration::from_millis(200);

// Bit 7..0 = A, B, C, D, E, F, G, DP
// Common-anode: 1 = segment off, 0 = segment on
static SEGMENT_MASKS: LazyLock<HashMap<char, u8>> = LazyLock::new(|| {
    HashMap::from([
        (' ', 0b1111_1111),
        ('0', 0b0000_0011),
        ('1', 0b1001_1111),
        ('2', 0b0010_0101),
        ('3', 0b0000_1101),
        ('4', 0b1001_1001),
        ('5', 0b0100_1001),
        ('6', 0b0100_0001),
        ('7', 0b0001_1111),
        ('8', 0b0000_0001),
        ('9', 0b0000_1001),
        ('C', 0b0110_0011),
    ])
});

const DECIMAL_POINT_MASK: u8 = 0b1111_1110;

pub struct State {
    thermistor_pin: AdcChannelDriver<'static, ADCCH8<ADCU2>, Rc<AdcDriver<'static, ADCU2>>>,
    hc595: Hc595Driver,
    digit_pins: [PinDriver<'static, Output>; 4],

    temperature: f32,
    last_temperature_update: Instant,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let adc2 = adc::init(peripherals.adc2)?;

    let thermistor_pin = adc2.assign(peripherals.pins.gpio25)?;

    let hc595 = hc595::init(
        peripherals.pins.gpio14,
        peripherals.pins.gpio13,
        peripherals.pins.gpio12,
    )?;

    let digit_pins = [
        PinDriver::output(peripherals.pins.gpio5)?,
        PinDriver::output(peripherals.pins.gpio4)?,
        PinDriver::output(peripherals.pins.gpio26)?,
        PinDriver::output(peripherals.pins.gpio27)?,
    ];

    let state = State {
        thermistor_pin,
        hc595,
        digit_pins,
        temperature: 0.0,
        last_temperature_update: Instant::now(),
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    measure_temperature(state)?;

    let mut chars = format!("{:.1}C", state.temperature)
        .chars()
        .collect::<Vec<char>>();

    while chars.len() - 1 < state.digit_pins.len() {
        chars.insert(0, ' ');
    }

    update_display(state, get_display_mask(chars.as_slice())).await?;

    Ok(())
}

fn measure_temperature(state: &mut State) -> Result<()> {
    if (Instant::now() - state.last_temperature_update) < TEMPERATURE_UPDATE_INTERVAL {
        return Ok(());
    }

    let thermistor_level = adc::read_normalized(&mut state.thermistor_pin)?;

    state.temperature = thermo::ntc_to_celsius(thermistor_level, 10_000.0, 10_000.0, 25.0, 3950.0);
    state.last_temperature_update = Instant::now();

    log::info!("temperature: {:.2}°C", state.temperature);

    Ok(())
}

async fn update_display(state: &mut State, mask: Vec<u8>) -> Result<()> {
    if mask.len() != state.digit_pins.len() {
        log::error!("mask length does not match digit pins length");
        return Ok(());
    }

    for (digit_index, mask) in mask.iter().enumerate() {
        set_active_digit(digit_index, &mut state.digit_pins)?;

        state.hc595.write_byte(*mask, ShiftOrder::MsbFirst)?;

        Timer::after(Duration::from_millis(2)).await;

        state.hc595.write_byte(0xFF, ShiftOrder::MsbFirst)?;
    }

    Ok(())
}

fn get_display_mask(text: &[char]) -> Vec<u8> {
    let mut display_masks = Vec::new();

    for char in text.iter() {
        if *char == '.' {
            *display_masks.last_mut().unwrap() &= DECIMAL_POINT_MASK;
            continue;
        }

        let mask = SEGMENT_MASKS.get(char).copied().unwrap();
        display_masks.push(mask);
    }

    display_masks
}

fn set_active_digit(
    digit_index: usize,
    digit_pins: &mut [PinDriver<'static, Output>],
) -> Result<()> {
    for digit_pin in digit_pins.iter_mut() {
        digit_pin.set_low()?;
    }

    digit_pins[digit_index].set_high()?;

    Ok(())
}
