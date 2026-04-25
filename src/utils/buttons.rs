use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Input, PinDriver};

pub async fn button_pressed(btn_pin: &PinDriver<'static, Input>) -> bool {
    for _ in 0..2 {
        if btn_pin.is_low() {
            return false;
        }

        Timer::after(Duration::from_millis(20)).await;
    }

    true
}
