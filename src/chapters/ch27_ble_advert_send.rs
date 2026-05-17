use crate::utils::button;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Input, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp32_nimble::enums::ConnMode;
use esp32_nimble::{BLEAdvertisementData, BLEDevice};

pub const PROJECT_NAME: &str = "ch27_ble_advert_send";

const MFG_COMPANY_ID: u16 = 0xFFFF;

pub struct State {
    btn_pin: PinDriver<'static, Input>,
    button_pressed: bool,
    seq: u16,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    Ok(State {
        btn_pin: PinDriver::input(peripherals.pins.gpio13, Pull::Floating)?,
        button_pressed: false,
        seq: 0,
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    if button::check_pressed(&state.btn_pin).await {
        if state.button_pressed {
            return Ok(());
        }

        advertise(1, -1, state.seq).await?;

        state.button_pressed = true;
        state.seq += 1;
    } else {
        state.button_pressed = false;
    }

    Ok(())
}

async fn advertise(team: u8, action: i8, seq: u16) -> Result<()> {
    let ble = BLEDevice::take();

    match ble.get_addr() {
        Ok(mac) => log::info!("MAC address: {mac}"),
        Err(e) => log::warn!("MAC address unavailable: {e:?}"),
    }

    let mut adv = ble.get_advertising().lock();

    adv.advertisement_type(ConnMode::Non).scan_response(false);

    let mut data = BLEAdvertisementData::new();

    data.name("ESP32-SCORE");
    data.manufacturer_data(&create_payload(team, action, seq));

    adv.set_data(&mut data)?;

    log::info!("Starting advertising...");

    adv.start()?;

    Timer::after(Duration::from_millis(300)).await;

    log::info!("Stopping advertising...");

    adv.stop()?;

    Ok(())
}

fn create_payload(team: u8, action: i8, seq: u16) -> [u8; 6] {
    let mut out = [0u8; 6];

    out[0..2].copy_from_slice(&MFG_COMPANY_ID.to_le_bytes());
    out[2] = team;
    out[3] = action as u8;
    out[4..6].copy_from_slice(&seq.to_le_bytes());

    out
}
