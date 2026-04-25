use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp32_tutorial::ch4_analog_and_pwm::*;
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
    log::info!("Setting up chapter '{}'", CHAPTER_NAME);

    let peripherals = Peripherals::take().unwrap();
    let mut state = setup(peripherals)?;

    log::info!("Starting chapter '{}'", CHAPTER_NAME);

    loop {
        update(&mut state).await?;
        Timer::after(Duration::from_millis(10)).await;
    }
}
