use anyhow::Result;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::ledc::config::TimerConfig;
use esp_idf_svc::hal::ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, Resolution};
use esp_idf_svc::hal::units::FromValueType;

pub fn init<T, C>(
    timer: T,
    channel: C,
    pin: impl OutputPin + 'static,
    frequency_hz: u32,
    resolution: Resolution,
) -> Result<(LedcTimerDriver<'static, T::SpeedMode>, LedcDriver<'static>)>
where
    T: LedcTimer + 'static,
    C: LedcChannel<SpeedMode = T::SpeedMode> + 'static,
{
    let timer_config = TimerConfig::default()
        .frequency(frequency_hz.Hz())
        .resolution(resolution);

    let ledc_timer = LedcTimerDriver::new(timer, &timer_config)?;
    let ledc_channel = LedcDriver::new(channel, &ledc_timer, pin)?;

    Ok((ledc_timer, ledc_channel))
}
