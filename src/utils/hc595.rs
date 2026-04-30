use anyhow::Result;
use esp_idf_svc::hal::gpio::{Output, OutputPin, PinDriver};

pub struct Hc595Driver {
    data_pin: PinDriver<'static, Output>,
    clock_pin: PinDriver<'static, Output>,
    latch_pin: PinDriver<'static, Output>,
}

pub enum ShiftOrder {
    MsbFirst,
    #[allow(dead_code)]
    LsbFirst,
}

impl Hc595Driver {
    pub fn write_byte(&mut self, value: u8, order: ShiftOrder) -> Result<()> {
        self.shift_byte(value, order)?;
        self.pulse_latch()?;

        Ok(())
    }

    fn shift_byte(&mut self, value: u8, order: ShiftOrder) -> Result<()> {
        let bits: Vec<usize> = match order {
            ShiftOrder::MsbFirst => (0..8).rev().collect(),
            ShiftOrder::LsbFirst => (0..8).collect(),
        };

        for bit_index in bits {
            let is_set = (value & (1u8 << bit_index)) != 0;

            if is_set {
                self.data_pin.set_high()?;
            } else {
                self.data_pin.set_low()?;
            }

            self.pulse_clock()?;
        }

        Ok(())
    }

    fn pulse_clock(&mut self) -> Result<()> {
        self.clock_pin.set_low()?;
        self.clock_pin.set_high()?;
        self.clock_pin.set_low()?;

        Ok(())
    }

    fn pulse_latch(&mut self) -> Result<()> {
        self.latch_pin.set_low()?;
        self.latch_pin.set_high()?;
        self.latch_pin.set_low()?;

        Ok(())
    }
}

pub fn init(
    data_pin: impl OutputPin + 'static,
    clock_pin: impl OutputPin + 'static,
    latch_pin: impl OutputPin + 'static,
) -> Result<Hc595Driver> {
    let mut data_pin = PinDriver::output(data_pin)?;
    let mut clock_pin = PinDriver::output(clock_pin)?;
    let mut latch_pin = PinDriver::output(latch_pin)?;

    data_pin.set_low()?;
    clock_pin.set_low()?;
    latch_pin.set_low()?;

    Ok(Hc595Driver {
        data_pin,
        clock_pin,
        latch_pin,
    })
}
