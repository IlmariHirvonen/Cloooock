use arduino_hal::port::{mode::Output, Pin};

//use crate::time::{TicksPerBar, BPM, TICK_RATE};

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
    pub fn set_numerator(&mut self, numerator: u16) {
        self.numerator = numerator;
    }

    pub fn set_denominator(&mut self, denominator: u16) {
        self.denominator = denominator;
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
}

// reset at the start of every bar.
pub struct ClockChannel {
    prescaler: Prescaler,
    threshold: u32,
    threshold_interval: u32,
    output: ClockOutput,
    previous_ticks: u32,
}

impl ClockChannel {
    pub fn new(
        led_pin: Pin<Output>,
        output_pin: Pin<Output>,
        prescaler: Prescaler,
        ticks_per_bar: u32,
    ) -> Self {
        ClockChannel {
            threshold_interval: ticks_per_bar / prescaler.denominator as u32
                * prescaler.numerator as u32,
            threshold: ticks_per_bar / prescaler.denominator as u32 * prescaler.numerator as u32,
            prescaler,
            output: ClockOutput::new(led_pin, output_pin),
            previous_ticks: 0,
        }
    }

    pub fn calculate_threshold(&mut self, bar_ticks: u32) {
        self.threshold_interval =
            bar_ticks / self.prescaler.denominator as u32 * self.prescaler.numerator as u32;

        // if removed stops rapid pulses but it takes time for all channels to catch up and sync
        // self.reset_threshold();
    }
    pub fn reset_threshold(&mut self) {
        self.threshold = self.threshold_interval;
        self.previous_ticks = 0;
        self.output.set_high();
        //self.output.toggle();
    }

    pub fn update(&mut self, ticks: u32) {
        if self.previous_ticks > ticks {
            self.reset_threshold()
        } else if ticks >= self.threshold {
            self.output.toggle();
            self.threshold = self.threshold + self.threshold_interval;
        }
        self.previous_ticks = ticks;
        //self.output.toggle()
    }

    pub fn set_led(&mut self, state: bool) {
        if state {
            self.output.led_pin.set_high();
        } else {
            self.output.led_pin.set_low();
        }
    }

    pub fn set_numerator(&mut self, numerator: u16) {
        self.prescaler.numerator = numerator;
    }
    pub fn update_denominator(&mut self, change: i8, bar_ticks: u32) {
        if change < 0 {
            if self.prescaler.denominator > 1 {
                self.prescaler.denominator -= 1;
            } else {
                self.prescaler.denominator = 128;
            }
        } else if change > 0 {
            if self.prescaler.denominator < 128 {
                self.prescaler.denominator += 1;
            } else {
                self.prescaler.denominator = 1;
            }
        }
        self.calculate_threshold(bar_ticks);
    }
    pub fn get_denominator(&self) -> u16 {
        self.prescaler.denominator
    }
}
