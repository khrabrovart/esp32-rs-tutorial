use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS;
use esp32_nimble::utilities::mutex::Mutex;
use esp32_nimble::{BLEAdvertisementData, BLECharacteristic, BLEDevice, NimbleProperties, uuid128};
use std::sync::Arc;

pub const PROJECT_NAME: &str = "ch27_ble_server";

pub struct State {
    notifying_characteristic: Arc<Mutex<BLECharacteristic>>,
    counter: u32,
    last_update: Instant,
}

pub async fn setup(_peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();
    let ble_server = ble_device.get_server();

    ble_server.on_connect(|server, desc| {
        log::info!("Client connected: {:?}", desc);

        server
            .update_conn_params(desc.conn_handle(), 24, 48, 0, 60)
            .unwrap();

        if server.connected_count() < (CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
            log::info!("Multi-connect support: start advertising");
            ble_advertising.lock().start().unwrap();
        }
    });

    ble_server.on_disconnect(|_desc, reason| {
        log::info!("Client disconnected ({:?})", reason);
    });

    let ble_service = ble_server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

    let static_characteristic = ble_service.lock().create_characteristic(
        uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
        NimbleProperties::READ,
    );

    static_characteristic
        .lock()
        .set_value("Hello, world!".as_bytes());

    let notifying_characteristic = ble_service.lock().create_characteristic(
        uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );

    notifying_characteristic.lock().set_value(b"Initial value.");

    let writable_characteristic = ble_service.lock().create_characteristic(
        uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::WRITE,
    );

    writable_characteristic
        .lock()
        .on_read(move |_, _| {
            log::info!("Read from writable characteristic.");
        })
        .on_write(|args| {
            log::info!(
                "Wrote to writable characteristic: {:?} -> {:?}",
                args.current_data(),
                args.recv_data()
            );
        });

    ble_advertising.lock().set_data(
        BLEAdvertisementData::new()
            .name("ESP32-GATT-Server")
            .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa")),
    )?;

    ble_advertising.lock().start()?;

    ble_server.ble_gatts_show_local();

    Ok(State {
        notifying_characteristic,
        counter: 0,
        last_update: Instant::now(),
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    if state.last_update.elapsed() < Duration::from_secs(1) {
        return Ok(());
    }

    state
        .notifying_characteristic
        .lock()
        .set_value(format!("Counter: {}", state.counter).as_bytes())
        .notify();

    state.counter += 1;
    state.last_update = Instant::now();

    Ok(())
}
