use anyhow::Result;
use core::convert::TryInto;
use embassy_executor::Spawner;
use esp_idf_svc::eventloop::{EspSubscription, EspSystemEventLoop, System};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AccessPointConfiguration, ClientConfiguration};
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, Configuration, EspWifi, WifiEvent};

pub const PROJECT_NAME: &str = "ch32_wifi_ap_station";

const AP_WIFI_SSID: &str = "ESP32-AP";
const AP_WIFI_PASSWORD: &str = "123esp123";

const STA_WIFI_SSID: &str = "DIGIFIBRA-13AF";
const STA_WIFI_PASSWORD: &str = "**************";

pub struct State {
    _wifi: BlockingWifi<EspWifi<'static>>,
    _wifi_events: EspSubscription<'static, System>,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let wifi_events = sys_loop.subscribe::<WifiEvent, _>(|event| match event {
        WifiEvent::StaConnected(sta) => log::info!("STA connected: {sta:?}"),
        WifiEvent::StaDisconnected(sta) => log::info!("STA disconnected: {sta:?}"),
        WifiEvent::ApStaConnected(sta) => log::info!("Device connected: {sta:?}"),
        WifiEvent::ApStaDisconnected(sta) => log::info!("Device disconnected: {sta:?}"),
        _ => {}
    })?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let wifi_configuration = Configuration::Mixed(
        ClientConfiguration {
            ssid: STA_WIFI_SSID.try_into()?,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: STA_WIFI_PASSWORD.try_into()?,
            channel: None,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: AP_WIFI_SSID.try_into()?,
            ssid_hidden: false,
            auth_method: AuthMethod::WPA2Personal,
            password: AP_WIFI_PASSWORD.try_into()?,
            ..Default::default()
        },
    );

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;

    let ap_ip = wifi.wifi().ap_netif().get_ip_info()?;
    let sta_ip = wifi.wifi().sta_netif().get_ip_info()?;

    log::info!(
        "AP '{AP_WIFI_SSID}' up: ip={}, gateway={}, netmask={}",
        ap_ip.ip,
        ap_ip.subnet.gateway,
        ap_ip.subnet.mask
    );
    log::info!(
        "STA connected to '{STA_WIFI_SSID}': ip={}, gateway={}, netmask={}",
        sta_ip.ip,
        sta_ip.subnet.gateway,
        sta_ip.subnet.mask
    );

    Ok(State {
        _wifi: wifi,
        _wifi_events: wifi_events,
    })
}

pub async fn update(_state: &mut State) -> Result<()> {
    Ok(())
}
