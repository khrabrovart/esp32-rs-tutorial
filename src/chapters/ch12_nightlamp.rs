use crate::utils::adc;
use crate::utils::ledc;
use crate::utils::math;
use anyhow::Result;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{ADCCH6, ADCU1, ADCU2};
use esp_idf_svc::hal::ledc::{LedcDriver, Resolution};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const CHAPTER_NAME: &str = "ch12_nightlamp";

const FREQUENCY: u32 = 5000;
const RESOLUTION: Resolution = Resolution::Bits8;
const MIN_LIGHT_LEVEL: f32 = 0.8;

pub struct State {
    ledc_channel: LedcDriver<'static>,
    potentiometer_pin: AdcChannelDriver<'static, ADCCH6<ADCU1>, AdcDriver<'static, ADCU1>>,
    photoresistor_pin: AdcChannelDriver<'static, ADCCH6<ADCU2>, AdcDriver<'static, ADCU2>>,
}

pub fn setup(peripherals: Peripherals) -> Result<State> {
    let potentiometer_pin = adc::init(peripherals.adc1, peripherals.pins.gpio34)?;
    let photoresistor_pin = adc::init(peripherals.adc2, peripherals.pins.gpio14)?;

    let (_, ledc_channel) = ledc::init(
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
        peripherals.pins.gpio4,
        FREQUENCY,
        RESOLUTION,
    )?;

    let state = State {
        ledc_channel,
        potentiometer_pin,
        photoresistor_pin,
    };

    Ok(state)
}

pub async fn update(state: &mut State) -> Result<()> {
    let light_level = adc::read_normalized(&mut state.photoresistor_pin)?;
    let light_level = math::remap_clamped(light_level, MIN_LIGHT_LEVEL, 1.0, 0.0, 1.0);
    let potentiometer_level = adc::read_normalized(&mut state.potentiometer_pin)?;
    let brightness = light_level * potentiometer_level;

    ledc::set_duty_percentage(&mut state.ledc_channel, brightness)?;

    Ok(())
}
