use anyhow::Result;
use core::borrow::Borrow;
use esp_idf_svc::hal::adc::attenuation::DB_12;
use esp_idf_svc::hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_svc::hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_svc::hal::adc::{Adc, AdcChannel, AdcUnit};
use esp_idf_svc::hal::gpio::ADCPin;
use std::rc::Rc;

const MAX_VALUE: f32 = 4095.0;

pub struct AdcDriverWrapper<'d, U> {
    driver: Rc<AdcDriver<'d, U>>,
    config: AdcChannelConfig,
}

impl<'d, U> AdcDriverWrapper<'d, U>
where
    U: AdcUnit + 'd,
{
    pub fn assign<C>(
        &self,
        pin: impl ADCPin<AdcChannel = C> + 'd,
    ) -> Result<AdcChannelDriver<'d, C, Rc<AdcDriver<'d, U>>>>
    where
        C: AdcChannel<AdcUnit = U> + 'd,
    {
        let driver = Rc::clone(&self.driver);
        Ok(AdcChannelDriver::new(driver, pin, &self.config)?)
    }
}

pub fn init<'d, A, U>(adc: A) -> Result<AdcDriverWrapper<'d, U>>
where
    A: Adc<AdcUnit = U> + 'd,
    U: AdcUnit + 'd,
{
    let adc = AdcDriver::new(adc)?;
    let adc_cfg = AdcChannelConfig {
        attenuation: DB_12,
        ..Default::default()
    };

    Ok(AdcDriverWrapper {
        driver: Rc::new(adc),
        config: adc_cfg,
    })
}

pub fn read_mapped<'d, C, M>(
    pin: &mut AdcChannelDriver<'d, C, M>,
    from: f32,
    to: f32,
) -> Result<f32>
where
    C: AdcChannel + 'd,
    M: Borrow<AdcDriver<'d, C::AdcUnit>> + 'd,
{
    let raw = pin.read_raw()? as f32;
    let result = from + (raw / MAX_VALUE * (to - from));
    Ok(result)
}

pub fn read_normalized<'d, C, M>(pin: &mut AdcChannelDriver<'d, C, M>) -> Result<f32>
where
    C: AdcChannel + 'd,
    M: Borrow<AdcDriver<'d, C::AdcUnit>> + 'd,
{
    let raw = pin.read_raw()? as f32;
    let result = raw / MAX_VALUE;
    Ok(result)
}
