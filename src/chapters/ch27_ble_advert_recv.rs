use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp32_nimble::{BLEAddress, BLEAddressType, BLEDevice, BLEScan};

pub const PROJECT_NAME: &str = "ch27_ble_advert_recv";

const SENDER_ADDR: &str = "60:3E:5F:67:1C:A5";

const MFG_COMPANY_ID: u16 = 0xFFFF;

pub struct State {
    scores: [u8; 2],
    seq: u16,
}

#[embassy_executor::task]
async fn ble_scan_loop() {
    let mut state = State {
        scores: [0, 0],
        seq: 0,
    };

    loop {
        log::info!("Starting BLE scan");

        let ble = BLEDevice::take();
        let mut scan = BLEScan::new();
        scan.active_scan(false).interval(100).window(99);

        if let Err(e) = scan
            .start(ble, 30_000, |device, data| {
                log::info!("Received from {}: {:?}", device.addr(), data);

                if !addr_matches_sender(device.addr()) {
                    return None;
                }

                let mfg = data.manufacture_data()?;

                log::info!("Manufacturer data for {}: {:?}", device.addr(), mfg);

                if mfg.company_identifier != MFG_COMPANY_ID {
                    return None;
                }

                let payload = mfg.payload;

                log::info!("Payload: {:?}", payload);

                if payload.len() < 4 {
                    return None;
                }

                let team = payload[0] as usize;
                let action = payload[1] as i8;
                let seq = u16::from_le_bytes([payload[2], payload[3]]);
                let addr = device.addr();

                log::info!(
                    "Received from {}: team={}, action={}, seq={}",
                    addr,
                    team,
                    action,
                    seq
                );

                if state.seq > seq {
                    log::warn!(
                        "Sequence number is less than the previous one (received: {}, state: {})",
                        seq,
                        state.seq
                    );
                    return None;
                }

                state.scores[team] = match action {
                    1 => state.scores[team].saturating_add(1),
                    -1 => state.scores[team].saturating_sub(1),
                    0 => 0,
                    _ => return None,
                };

                state.seq = seq;

                Some(())
            })
            .await
        {
            log::warn!("BLE scan session: {}", e);
        }

        log::info!("Scores: {:?}", state.scores);
        Timer::after(Duration::from_millis(10)).await;
    }
}

fn addr_matches_sender(addr: BLEAddress) -> bool {
    BLEAddress::from_str(SENDER_ADDR, BLEAddressType::Public)
        .is_some_and(|expected| expected == addr)
}

pub async fn setup(_peripherals: Peripherals, spawner: Spawner) -> Result<State> {
    let token = ble_scan_loop().map_err(|_| anyhow::anyhow!("BLE scan task already running"))?;
    spawner.spawn(token);

    Ok(State {
        scores: [0, 0],
        seq: 0,
    })
}

pub async fn update(_state: &mut State) -> Result<()> {
    Ok(())
}
