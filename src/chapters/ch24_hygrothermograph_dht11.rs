use crate::utils::dht11;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_idf_svc::hal::gpio::{InputOutput, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch24_hygrothermograph_dht11";

const READ_INTERVAL: Duration = Duration::from_secs(2);
const STARTUP_SETTLE: Duration = Duration::from_secs(2);

pub struct State {
    dht_pin: PinDriver<'static, InputOutput>,
    last_update: Instant,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let dht_pin = PinDriver::input_output_od(peripherals.pins.gpio13, Pull::Up)?;

    Timer::after(STARTUP_SETTLE).await;

    Ok(State {
        dht_pin,
        last_update: Instant::now(),
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    if state.last_update.elapsed() < READ_INTERVAL {
        return Ok(());
    }

    match dht11::measure(&mut state.dht_pin).await {
        Ok((humidity, temperature)) => {
            log::info!(
                "Humidity: {:.1}%, Temperature: {:.1}°C",
                humidity,
                temperature
            );
        }
        Err(e) => {
            log::warn!("Measurement failed: {e:?}");
        }
    }

    state.last_update = Instant::now();

    Ok(())
}
