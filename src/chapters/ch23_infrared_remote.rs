use crate::utils::ir_rmt::IRReceiverDriver;
use anyhow::Result;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_idf_svc::hal::gpio::{Output, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::units::Hertz;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

pub const PROJECT_NAME: &str = "ch23_infrared_remote";

type ButtonFunction = Box<dyn Fn(&mut State) -> Result<()> + Send + Sync>;

struct Button {
    name: &'static str,
    function: ButtonFunction,
}

const CORRECT_CODE: &str = "1346";

static BUTTON_BY_CODE: LazyLock<HashMap<u32, Button>> = LazyLock::new(|| {
    HashMap::from([
        (
            0xFFA25D,
            Button {
                name: "ON/OFF",
                function: Box::new(|_: &mut State| Ok(())),
            },
        ),
        (
            0xFFE21D,
            Button {
                name: "MENU",
                function: Box::new(|_: &mut State| Ok(())),
            },
        ),
        (
            0xFF22DD,
            Button {
                name: "TEST",
                function: Box::new(|state: &mut State| -> Result<()> {
                    match flash_led(Arc::clone(&state.green_led_pin)) {
                        Ok(token) => state.spawner.spawn(token),
                        Err(_) => log::warn!("flash_led task pool busy"),
                    }
                    Ok(())
                }),
            },
        ),
        (
            0xFFC23D,
            Button {
                name: "BACK",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.clear();
                    Ok(())
                }),
            },
        ),
        (
            0xFFE01F,
            Button {
                name: "SKIP_BACK",
                function: Box::new(|_: &mut State| Ok(())),
            },
        ),
        (
            0xFF906F,
            Button {
                name: "SKIP_FWD",
                function: Box::new(|_: &mut State| Ok(())),
            },
        ),
        (
            0xFFA857,
            Button {
                name: "PLAY",
                function: Box::new(|state: &mut State| -> Result<()> {
                    if state.code_buffer.is_empty() {
                        return Ok(());
                    }

                    if state.code_buffer.iter().collect::<String>() == CORRECT_CODE {
                        match flash_led(Arc::clone(&state.green_led_pin)) {
                            Ok(token) => state.spawner.spawn(token),
                            Err(_) => log::warn!("flash_led task pool busy"),
                        }
                    }

                    state.code_buffer.clear();

                    Ok(())
                }),
            },
        ),
        (
            0xFF02FD,
            Button {
                name: "+",
                function: Box::new(|_: &mut State| Ok(())),
            },
        ),
        (
            0xFF9867,
            Button {
                name: "-",
                function: Box::new(|_: &mut State| Ok(())),
            },
        ),
        (
            0xFFB04F,
            Button {
                name: "C",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.pop();
                    Ok(())
                }),
            },
        ),
        (
            0xFF6897,
            Button {
                name: "0",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('0');
                    Ok(())
                }),
            },
        ),
        (
            0xFF30CF,
            Button {
                name: "1",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('1');
                    Ok(())
                }),
            },
        ),
        (
            0xFF18E7,
            Button {
                name: "2",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('2');
                    Ok(())
                }),
            },
        ),
        (
            0xFF7A85,
            Button {
                name: "3",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('3');
                    Ok(())
                }),
            },
        ),
        (
            0xFF10EF,
            Button {
                name: "4",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('4');
                    Ok(())
                }),
            },
        ),
        (
            0xFF38C7,
            Button {
                name: "5",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('5');
                    Ok(())
                }),
            },
        ),
        (
            0xFF5AA5,
            Button {
                name: "6",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('6');
                    Ok(())
                }),
            },
        ),
        (
            0xFF42BD,
            Button {
                name: "7",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('7');
                    Ok(())
                }),
            },
        ),
        (
            0xFF4AB5,
            Button {
                name: "8",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('8');
                    Ok(())
                }),
            },
        ),
        (
            0xFF52AD,
            Button {
                name: "9",
                function: Box::new(|state: &mut State| -> Result<()> {
                    state.code_buffer.push('9');
                    Ok(())
                }),
            },
        ),
    ])
});

#[embassy_executor::task(pool_size = 4)]
async fn flash_led(led: Arc<Mutex<PinDriver<'static, Output>>>) {
    log::info!("LED on");

    let _ = led.lock().unwrap().set_high();

    Timer::after(Duration::from_secs(1)).await;

    log::info!("LED off");

    let _ = led.lock().unwrap().set_low();
}

const IR_RMT_RESOLUTION_HZ: Hertz = Hertz(1_000_000);

const DATA_START_INDEX: usize = 1;
const DATA_END_INDEX: usize = 32;

pub struct State {
    green_led_pin: Arc<Mutex<PinDriver<'static, Output>>>,
    yellow_led_pin: PinDriver<'static, Output>,
    receiver: IRReceiverDriver<'static>,
    code_buffer: Vec<char>,
    spawner: Spawner,
}

pub async fn setup(peripherals: Peripherals, spawner: Spawner) -> Result<State> {
    let green_led_pin = Arc::new(Mutex::new(PinDriver::output(peripherals.pins.gpio12)?));
    let yellow_led_pin = PinDriver::output(peripherals.pins.gpio13)?;

    let receiver = IRReceiverDriver::init(peripherals.pins.gpio14)?;

    Ok(State {
        green_led_pin,
        yellow_led_pin,
        receiver,
        code_buffer: Vec::new(),
        spawner,
    })
}

pub async fn update(state: &mut State) -> Result<()> {
    receive(state).await?;

    Ok(())
}

async fn receive(state: &mut State) -> Result<()> {
    let symbols = state.receiver.receive().await?;

    if symbols.is_empty() {
        log::warn!("No symbols captured");
        return Ok(());
    }

    log::info!("Captured {} symbols", symbols.len());

    if symbols.len() < DATA_END_INDEX + 1 {
        log::warn!("Not enough symbols");
        return Ok(());
    }

    let mut button_code: u32 = 0x000000;

    for (index, symbol) in symbols[DATA_START_INDEX..=DATA_END_INDEX]
        .iter()
        .enumerate()
    {
        let low_duration = symbol.level0().ticks.duration(IR_RMT_RESOLUTION_HZ);
        let high_duration = symbol.level1().ticks.duration(IR_RMT_RESOLUTION_HZ);

        log::info!(
            "[{}] Low: {:>5} µs, High: {:>5} µs | {}",
            index + DATA_START_INDEX,
            low_duration.as_micros(),
            high_duration.as_micros(),
            if low_duration > high_duration {
                "0"
            } else {
                "1"
            }
        );

        button_code <<= 1;

        if low_duration > high_duration {
            button_code |= 0;
        } else {
            button_code |= 1;
        }
    }

    let nibbles: [String; 8] = core::array::from_fn(|i| {
        let nibble = ((button_code >> (28 - i * 4)) & 0xF) as u8;
        format!("{:04b}", nibble)
    });

    log::info!("{:06x} | {}", button_code, nibbles.join(" "));

    let button = BUTTON_BY_CODE.get(&button_code).unwrap();

    log::info!("Button: {}", button.name);

    (button.function)(state)?;

    log::info!("Buffer: {:?}", state.code_buffer);

    if !state.code_buffer.is_empty() {
        state.yellow_led_pin.set_high()?;
    } else {
        state.yellow_led_pin.set_low()?;
    }

    Ok(())
}
