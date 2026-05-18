use anyhow::Result;
use core::convert::TryInto;
use embassy_executor::Spawner;
use esp_idf_svc::eventloop::{EspSubscription, EspSystemEventLoop, System};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::AccessPointConfiguration;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, Configuration, EspWifi, WifiEvent};

pub const PROJECT_NAME: &str = "ch32_wifi_ap";

const WIFI_SSID: &str = "ESP32-AP";
const WIFI_PASSWORD: &str = "123esp123";

pub struct State {
    _wifi: BlockingWifi<EspWifi<'static>>,
    _wifi_events: EspSubscription<'static, System>,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let wifi_events = sys_loop.subscribe::<WifiEvent, _>(|event| match event {
        WifiEvent::ApStaConnected(sta) => log::info!("Device connected: {sta:?}"),
        WifiEvent::ApStaDisconnected(sta) => log::info!("Device disconnected: {sta:?}"),
        _ => {}
    })?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let wifi_configuration = Configuration::AccessPoint(AccessPointConfiguration {
        ssid: WIFI_SSID.try_into()?,
        ssid_hidden: false,
        auth_method: AuthMethod::WPA2Personal,
        password: WIFI_PASSWORD.try_into()?,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().ap_netif().get_ip_info()?;

    log::info!(
        "AP '{WIFI_SSID}' up: ip={}, gateway={}, netmask={}",
        ip_info.ip,
        ip_info.subnet.gateway,
        ip_info.subnet.mask
    );

    Ok(State {
        _wifi: wifi,
        _wifi_events: wifi_events,
    })
}

pub async fn update(_state: &mut State) -> Result<()> {
    Ok(())
}
