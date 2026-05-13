use anyhow::Result;
use embassy_executor::Spawner;
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch27_bluetooth";

pub struct State {}

pub async fn setup(peripherals: Peripherals, _spawner: Spawner) -> Result<State> {
    Ok(State {})
}

pub async fn update(state: &mut State) -> Result<()> {
    Ok(())
}
