use arduino_hal::port::{mode::Output, Pin};

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
    counter: u16,
    prescaler: Prescaler,
    output: ClockOutput,
    time_between_hit:u32,

}

impl ClockChannel {
    pub fn new(led_pin: Pin<Output>, output_pin: Pin<Output>) -> Self {
        ClockChannel {
            counter: 0,
            prescaler: Prescaler {
                numerator: 1,
                denominator: 1,
            },
            output: ClockOutput::new(led_pin, output_pin),
            time_between_hit:1000,

        }
    }

    pub fn update(&mut self, timer_us: u32, new_time_between_hits:u32) {
        if new_time_between_hits != self.time_between_hit {
            self.time_between_hit = new_time_between_hits;

        }
        //let time_between_hit = 10;
        if timer_us % self.time_between_hit == 0 {
            self.output.toggle()
        }
    }
}
