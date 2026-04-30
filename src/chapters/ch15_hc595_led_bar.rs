use crate::utils::adc;
use crate::utils::hc595;
use crate::utils::hc595::Hc595Driver;
use crate::utils::hc595::ShiftOrder;
use crate::utils::math;
use anyhow::Result;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH8, ADCCH9, ADCU2};
use esp_idf_svc::hal::peripherals::Peripherals;
use std::rc::Rc;

pub const CHAPTER_NAME: &str = "ch15_hc595_led_bar";

const MAX_BAR_LEVEL: f32 = 8.0;

pub struct State {
    potentiometer_pin: AdcChannelDriver<'static, ADCCH9<ADCU2>, Rc<AdcDriver<'static, ADCU2>>>,
    photoresistor_pin: AdcChannelDriver<'static, ADCCH8<ADCU2>, Rc<AdcDriver<'static, ADCU2>>>,
    hc595: Hc595Driver,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let adc2 = adc::init(peripherals.adc2)?;

    let potentiometer_pin = adc2.assign(peripherals.pins.gpio26)?;
    let photoresistor_pin = adc2.assign(peripherals.pins.gpio25)?;

    let hc595 = hc595::init(
        peripherals.pins.gpio14,
        peripherals.pins.gpio13,
        peripherals.pins.gpio12,
    )?;

    let state = State {
        potentiometer_pin,
        photoresistor_pin,
        hc595,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    let photoresistor_level = adc::read_normalized(&mut state.photoresistor_pin)?;
    let potentiometer_level = adc::read_normalized(&mut state.potentiometer_pin)?;

    let light_level = math::remap_clamped(
        photoresistor_level,
        1.0 - potentiometer_level,
        1.0,
        0.0,
        1.0,
    );

    let active_led_count = (MAX_BAR_LEVEL * light_level)
        .round()
        .clamp(0.0, MAX_BAR_LEVEL) as u8;

    let data = ((1u16 << active_led_count) - 1) as u8;

    state.hc595.write_byte(data, ShiftOrder::MsbFirst)?;

    log::info!(
        "potentiometer: {:.2}, photoresistor: {:.2}, light: {:.2}, bar: 0x{:08b}",
        potentiometer_level,
        photoresistor_level,
        light_level,
        data
    );

    Ok(())
}
