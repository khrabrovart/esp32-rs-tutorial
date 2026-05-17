use anyhow::Result;
use core::convert::TryInto;
use embassy_executor::Spawner;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi,
};

pub const PROJECT_NAME: &str = "ch32_wifi_ap";

const WIFI_SSID: &str = "ESP32-AP";
const WIFI_PASSWORD: &str = "123esp";

pub struct State {
    _wifi: BlockingWifi<EspWifi<'static>>,
}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

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

    Ok(State { _wifi: wifi })
}

pub async fn update(_state: &mut State) -> Result<()> {
    Ok(())
}
