//! Minimal BLE GATT peripheral using `esp-idf-svc` Bluedroid only (no extra BLE crates).
//! Test from iOS with LightBlue or nRF Connect: subscribe to the notify characteristic and
//! write arbitrary bytes to the writable characteristic.

use std::sync::{Arc, Mutex};

use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant};
use enumset::enum_set;
use esp_idf_svc::bt::ble::gap::{AdvConfiguration, BleGapEvent, EspBleGap};
use esp_idf_svc::bt::ble::gatt::server::{ConnectionId, EspGatts, GattsEvent};
use esp_idf_svc::bt::ble::gatt::{
    AutoResponse, GattCharacteristic, GattDescriptor, GattId, GattInterface, GattServiceId,
    GattStatus, Handle, Permission, Property,
};
use esp_idf_svc::bt::{Ble, BtDriver, BtStatus, BtUuid};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

pub const PROJECT_NAME: &str = "ch27_bluetooth";

const APP_ID: u16 = 0;
const NUM_SERVICE_HANDLES: u16 = 8;
const DEVICE_NAME: &str = "ESP32-RS-BLE";

const SERVICE_UUID: u128 = 0xad91_b201_7347_4047_9e17_3bed82d75f9d;
const NOTIFY_CHAR_UUID: u128 = 0x503d_e214_8682_46c4_828f_d59144da41be;
const WRITE_CHAR_UUID: u128 = 0xb6fc_cb50_87be_44f3_ae22_f85485ea42c4;
const CCCD_UUID: u16 = 0x2902;

const NOTIFY_INTERVAL: Duration = Duration::from_secs(1);

struct ConnectionInfo {
    conn_id: ConnectionId,
    subscribed: bool,
}

#[derive(Default)]
struct BleShared {
    gatt_if: Option<GattInterface>,
    service_handle: Option<Handle>,
    notify_handle: Option<Handle>,
    notify_cccd_handle: Option<Handle>,
    write_handle: Option<Handle>,
    conn: Option<ConnectionInfo>,
}

type BleDriver = Arc<BtDriver<'static, Ble>>;
type BleGap = Arc<EspBleGap<'static, Ble, BleDriver>>;
type BleGatts = Arc<EspGatts<'static, Ble, BleDriver>>;

pub struct State {
    gap: BleGap,
    gatts: BleGatts,
    shared: Arc<Mutex<BleShared>>,
    counter: u32,
    last_notify: Instant,
}

fn check_gatt_status(status: GattStatus) -> Result<(), esp_idf_svc::sys::EspError> {
    if !matches!(status, GattStatus::Ok) {
        log::warn!("GATT status: {status:?}");
        return Err(esp_idf_svc::sys::EspError::from_infallible::<
            { esp_idf_svc::sys::ESP_FAIL },
        >());
    }
    Ok(())
}

fn adv_configuration() -> AdvConfiguration<'static> {
    AdvConfiguration {
        include_name: true,
        flag: 2,
        service_uuid: Some(BtUuid::uuid128(SERVICE_UUID)),
        ..Default::default()
    }
}

fn restart_advertising(
    gap: &EspBleGap<'static, Ble, BleDriver>,
) -> Result<(), esp_idf_svc::sys::EspError> {
    gap.set_device_name(DEVICE_NAME)?;
    gap.set_adv_conf(&adv_configuration())
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let nvs = EspDefaultNvsPartition::take()?;
    let bt = Arc::new(BtDriver::new(peripherals.modem, Some(nvs))?);
    let gap = Arc::new(EspBleGap::new(bt.clone())?);
    let gatts = Arc::new(EspGatts::new(bt)?);

    let shared = Arc::new(Mutex::new(BleShared::default()));

    let gap_cb = gap.clone();
    gap.subscribe(move |event| {
        if let BleGapEvent::AdvertisingConfigured(status) = event {
            if status == BtStatus::Success {
                if let Err(e) = gap_cb.start_advertising() {
                    log::warn!("start_advertising: {e:?}");
                }
            } else {
                log::warn!("AdvertisingConfigured: {status:?}");
            }
        }
    })?;

    let gap_ev = gap.clone();
    let gatts_ev = gatts.clone();
    let shared_ev = shared.clone();
    gatts.subscribe(move |(gatt_if, event)| {
        if let Err(e) = on_gatts_event(gatt_if, event, &gap_ev, &gatts_ev, &shared_ev) {
            log::warn!("GATTS handler: {e:?}");
        }
    })?;

    gatts.register_app(APP_ID)?;

    log::info!("BLE GATT app registered; building service…");

    Ok(State {
        gap,
        gatts,
        shared,
        counter: 0,
        last_notify: Instant::now(),
    })
}

fn on_gatts_event(
    gatt_if: GattInterface,
    event: GattsEvent<'_>,
    gap: &EspBleGap<'static, Ble, BleDriver>,
    gatts: &EspGatts<'static, Ble, BleDriver>,
    shared: &Mutex<BleShared>,
) -> Result<(), esp_idf_svc::sys::EspError> {
    match event {
        GattsEvent::ServiceRegistered { status, app_id } => {
            check_gatt_status(status)?;
            if app_id != APP_ID {
                return Ok(());
            }
            {
                let mut s = shared.lock().unwrap();
                s.gatt_if = Some(gatt_if);
            }
            restart_advertising(gap)?;
            gatts.create_service(
                gatt_if,
                &GattServiceId {
                    id: GattId {
                        uuid: BtUuid::uuid128(SERVICE_UUID),
                        inst_id: 0,
                    },
                    is_primary: true,
                },
                NUM_SERVICE_HANDLES,
            )?;
        }
        GattsEvent::ServiceCreated {
            status,
            service_handle,
            ..
        } => {
            check_gatt_status(status)?;
            {
                let mut s = shared.lock().unwrap();
                s.service_handle = Some(service_handle);
            }
            gatts.start_service(service_handle)?;
            gatts.add_characteristic(
                service_handle,
                &GattCharacteristic {
                    uuid: BtUuid::uuid128(NOTIFY_CHAR_UUID),
                    permissions: enum_set!(Permission::Read),
                    properties: enum_set!(Property::Read | Property::Notify),
                    max_len: 64,
                    auto_rsp: AutoResponse::ByGatt,
                },
                b"0",
            )?;
        }
        GattsEvent::CharacteristicAdded {
            status,
            attr_handle,
            service_handle,
            char_uuid,
        } => {
            check_gatt_status(status)?;
            let mut s = shared.lock().unwrap();
            if s.service_handle != Some(service_handle) {
                return Ok(());
            }
            if char_uuid == BtUuid::uuid128(NOTIFY_CHAR_UUID) {
                s.notify_handle = Some(attr_handle);
                drop(s);
                gatts.add_descriptor(
                    service_handle,
                    &GattDescriptor {
                        uuid: BtUuid::uuid16(CCCD_UUID),
                        permissions: enum_set!(Permission::Read | Permission::Write),
                    },
                )?;
            } else if char_uuid == BtUuid::uuid128(WRITE_CHAR_UUID) {
                s.write_handle = Some(attr_handle);
                log::info!("GATT table ready (notify + write)");
            }
        }
        GattsEvent::DescriptorAdded {
            status,
            attr_handle,
            service_handle,
            descr_uuid,
        } => {
            check_gatt_status(status)?;
            if descr_uuid != BtUuid::uuid16(CCCD_UUID) {
                return Ok(());
            }
            let mut s = shared.lock().unwrap();
            if s.service_handle != Some(service_handle) {
                return Ok(());
            }
            s.notify_cccd_handle = Some(attr_handle);
            drop(s);
            gatts.add_characteristic(
                service_handle,
                &GattCharacteristic {
                    uuid: BtUuid::uuid128(WRITE_CHAR_UUID),
                    permissions: enum_set!(Permission::Write),
                    properties: enum_set!(Property::Write),
                    max_len: 200,
                    auto_rsp: AutoResponse::ByGatt,
                },
                &[],
            )?;
        }
        GattsEvent::PeerConnected { conn_id, addr, .. } => {
            log::info!("BLE connected: {addr}");
            let mut s = shared.lock().unwrap();
            s.conn = Some(ConnectionInfo {
                conn_id,
                subscribed: false,
            });
            drop(s);
            let _ = restart_advertising(gap);
        }
        GattsEvent::PeerDisconnected { conn_id, addr, .. } => {
            log::info!("BLE disconnected: {addr} (conn_id={conn_id})");
            let mut s = shared.lock().unwrap();
            if s.conn.as_ref().is_some_and(|c| c.conn_id == conn_id) {
                s.conn = None;
            }
            drop(s);
            let _ = restart_advertising(gap);
        }
        GattsEvent::Write {
            conn_id,
            handle,
            value,
            is_prep,
            ..
        } => {
            if is_prep {
                return Ok(());
            }
            let s = shared.lock().unwrap();
            let Some(cccd) = s.notify_cccd_handle else {
                return Ok(());
            };
            let Some(write_h) = s.write_handle else {
                return Ok(());
            };
            if handle == cccd {
                if value.len() >= 2 {
                    let flags = u16::from_le_bytes([value[0], value[1]]);
                    let sub = (flags & 0x0001) != 0;
                    drop(s);
                    let mut s = shared.lock().unwrap();
                    if let Some(c) = s.conn.as_mut() {
                        if c.conn_id == conn_id {
                            c.subscribed = sub;
                            log::info!("Notify subscription: {sub} (CCCD=0x{flags:04x})");
                        }
                    }
                }
            } else if handle == write_h {
                log::info!(
                    "BLE write ({} bytes): {:?} utf8={:?}",
                    value.len(),
                    value,
                    core::str::from_utf8(value)
                );
            }
        }
        _ => {}
    }
    Ok(())
}

pub async fn update(state: &mut State) -> Result<()> {
    if state.last_notify.elapsed() < NOTIFY_INTERVAL {
        return Ok(());
    }
    state.last_notify = Instant::now();
    state.counter = state.counter.wrapping_add(1);

    let snapshot = {
        let s = state.shared.lock().unwrap();
        match (s.gatt_if, s.notify_handle, s.conn.as_ref()) {
            (Some(gif), Some(h), Some(c)) if c.subscribed => Some((gif, c.conn_id, h)),
            _ => None,
        }
    };

    if let Some((gatt_if, conn_id, handle)) = snapshot {
        let msg = format!("Counter: {}", state.counter);
        if let Err(e) = state.gatts.notify(gatt_if, conn_id, handle, msg.as_bytes()) {
            log::warn!("notify failed: {e:?}");
        }
    }

    Ok(())
}
