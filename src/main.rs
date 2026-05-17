use esp32_tutorial::ch32_wifi_station::{PROJECT_NAME, setup, update};

use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::peripherals::Peripherals;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = start(spawner).await {
        log::error!("Critical error: {:?}", e);
    }
}

async fn start(spawner: Spawner) -> Result<()> {
    log::info!("Setting up '{}'", PROJECT_NAME);

    let peripherals = Peripherals::take().unwrap();
    let mut state = setup(peripherals, spawner).await?;

    log::info!("Starting '{}'", PROJECT_NAME);

    loop {
        update(&mut state).await?;
        Timer::after(Duration::from_millis(10)).await;
    }
}
