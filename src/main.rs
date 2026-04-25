use embassy_executor::Spawner;
use esp_idf_svc::hal::peripherals::Peripherals;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    if let Err(e) = esp32_tutorial::ch2_button_and_led::run(peripherals).await {
        log::error!("Critical error: {:?}", e);
    }
}
