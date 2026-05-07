use anyhow::Result;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_svc::hal::peripherals::Peripherals;

pub const PROJECT_NAME: &str = "ch22_matrix_keypad";

const KEY_LABELS: [[char; 4]; 4] = [
    ['1', '2', '3', 'A'],
    ['4', '5', '6', 'B'],
    ['7', '8', '9', 'C'],
    ['*', '0', '#', 'D'],
];

pub struct State {
    row_pins: [PinDriver<'static, Output>; 4],
    col_pins: [PinDriver<'static, Input>; 4],
    pressed_key: Option<char>,
}

pub async fn setup(peripherals: Peripherals) -> Result<State> {
    let mut row_pins = [
        PinDriver::output(peripherals.pins.gpio22)?,
        PinDriver::output(peripherals.pins.gpio21)?,
        PinDriver::output(peripherals.pins.gpio25)?,
        PinDriver::output(peripherals.pins.gpio26)?,
    ];

    let col_pins = [
        PinDriver::input(peripherals.pins.gpio27, Pull::Up)?,
        PinDriver::input(peripherals.pins.gpio14, Pull::Up)?,
        PinDriver::input(peripherals.pins.gpio12, Pull::Up)?,
        PinDriver::input(peripherals.pins.gpio13, Pull::Up)?,
    ];

    set_all_rows_high(&mut row_pins)?;

    Ok(State {
        row_pins,
        col_pins,
        pressed_key: None,
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    let current_key = state.pressed_key;
    state.pressed_key = get_pressed_key(state).await?;

    if let Some(key) = state.pressed_key
        && current_key.is_none()
    {
        log::info!("Pressed key: '{}'", key);
    }

    Ok(())
}

async fn get_pressed_key(state: &mut State) -> Result<Option<char>> {
    for (row_index, _) in KEY_LABELS.iter().enumerate().take(state.row_pins.len()) {
        set_all_rows_high(&mut state.row_pins)?;
        state.row_pins[row_index].set_low()?;

        Timer::after(Duration::from_micros(50)).await;

        for (col_index, col) in state.col_pins.iter().enumerate() {
            if col.is_low() {
                let key = KEY_LABELS[row_index][col_index];
                return Ok(Some(key));
            }
        }
    }

    set_all_rows_high(&mut state.row_pins)?;

    Ok(None)
}

fn set_all_rows_high(rows: &mut [PinDriver<'static, Output>]) -> Result<()> {
    for row in rows {
        row.set_high()?;
    }

    Ok(())
}
