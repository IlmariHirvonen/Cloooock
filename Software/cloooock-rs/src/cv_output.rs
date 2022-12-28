use arduino_hal::port::{mode::Output, Pin};

use crate::time::{TicksPerBar, BPM, TICK_RATE};

pub struct Prescaler {
    numerator: u16,
    denominator: u16,
}
impl Prescaler {
    pub fn new(numerator: u16, denominator: u16) -> Self {
        Prescaler {
            numerator,
            denominator,
        }
    }
}

pub struct ClockOutput {
    state: bool,
    led_pin: Pin<Output>,
    output_pin: Pin<Output>,
}

impl ClockOutput {
    pub fn new(led_pin: Pin<Output>, output_pin: Pin<Output>) -> Self {
        ClockOutput {
            state: false,
            led_pin,
            output_pin,
        }
    }

    pub fn set_high(&mut self) {
        self.state = true;
        self.led_pin.set_high();
        self.output_pin.set_high();
    }

    pub fn set_low(&mut self) {
        self.state = false;
        self.led_pin.set_low();
        self.output_pin.set_low();
    }

    pub fn toggle(&mut self) {
        self.state = !self.state;
        match self.state {
            true => self.set_high(),
            false => self.set_low(),
        }
    }

    pub fn set_numerator() {
        todo!();
    }
    pub fn set_denominator() {
        todo!();
    }
}

// reset at the start of every bar.
pub struct ClockChannel {
    prescaler: Prescaler,
    threshold:u32,
    output: ClockOutput,
}

impl ClockChannel {
    pub fn new(led_pin: Pin<Output>, output_pin: Pin<Output>,prescaler:Prescaler, ticks_per_bar: u32) -> Self {
        ClockChannel {
            threshold: ticks_per_bar/prescaler.denominator as u32 *prescaler.numerator as u32,
            prescaler,
            output: ClockOutput::new(led_pin, output_pin),
        }
    }

    pub fn calculate_threshold(&mut self, bar_ticks: u32){
        self.threshold = bar_ticks/self.prescaler.denominator as u32 *self.prescaler.numerator as u32;

    }

    pub fn update(&mut self, ticks: u32) {
        if ticks % self.threshold == 0 {
             self.output.toggle()
        }
        //self.output.toggle()
    }
}
