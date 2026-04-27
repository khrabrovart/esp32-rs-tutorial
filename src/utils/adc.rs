use anyhow::Result;
use esp_idf_svc::hal::adc::attenuation::DB_12;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{Adc, AdcChannel};
use esp_idf_svc::hal::gpio::ADCPin;

const MAX_VALUE: f32 = 4095.0;

pub fn init<'d, C, A>(
    peripheral: A,
    pin: impl ADCPin<AdcChannel = C> + 'd,
) -> Result<AdcChannelDriver<'d, C, AdcDriver<'d, C::AdcUnit>>>
where
    C: AdcChannel + 'd,
    A: Adc<AdcUnit = C::AdcUnit> + 'd,
{
    let config = AdcChannelConfig {
        attenuation: DB_12,
        ..Default::default()
    };

    let adc = AdcDriver::new(peripheral)?;
    let adc_pin = AdcChannelDriver::new(adc, pin, &config)?;

    Ok(adc_pin)
}

pub fn read_mapped<'d, C>(
    pin: &mut AdcChannelDriver<'d, C, AdcDriver<'d, C::AdcUnit>>,
    from: f32,
    to: f32,
) -> Result<f32>
where
    C: AdcChannel + 'd,
{
    let raw = pin.read_raw()? as f32;
    let result = from + (raw / MAX_VALUE * (to - from));
    Ok(result)
}

pub fn read_normalized<'d, C>(
    pin: &mut AdcChannelDriver<'d, C, AdcDriver<'d, C::AdcUnit>>,
) -> Result<f32>
where
    C: AdcChannel + 'd,
{
    let raw = pin.read_raw()? as f32;
    let result = raw / MAX_VALUE;
    Ok(result)
}
