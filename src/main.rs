use esp32_tutorial::proj_rxb85_rf_receiver::{PROJECT_NAME, setup, update};

use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::peripherals::Peripherals;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    if let Err(e) = start().await {
        log::error!("Critical error: {:?}", e);
    }
}

async fn start() -> Result<()> {
    log::info!("Setting up '{}'", PROJECT_NAME);

    let peripherals = Peripherals::take().unwrap();
    let mut state = setup(peripherals).await?;

    log::info!("Starting '{}'", PROJECT_NAME);

    loop {
        update(&mut state).await?;
        Timer::after(Duration::from_millis(10)).await;
    }
}
