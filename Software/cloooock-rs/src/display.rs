use arduino_hal::{
    delay_us,
    port::{mode::Output, Pin},
};
use embedded_hal::digital::v2::{OutputPin, PinState};

const ZERO: u8 = 0b011000000;
const ONE: u8 = 0b11111001;
const TWO: u8 = 0b10100100;
const THREE: u8 = 0b10110000;
const FOUR: u8 = 0b10011001;
const FIVE: u8 = 0b10010010;
const SIX: u8 = 0b10000010;
const SEVEN: u8 = 0b11111000;
const EIGHT: u8 = 0b10000000;
const NINE: u8 = 0b10010000;
const DIGITS: [u8; 10] = [ZERO, ONE, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE];

fn shift_out(byte: u8, sck_pin: &mut Pin<Output>, data_pin: &mut Pin<Output>) {
    fn cycle(data: bool, sck_pin: &mut Pin<Output>, data_pin: &mut Pin<Output>) {
        sck_pin.set_low();
        data_pin.set_state(PinState::from(data));
        delay_us(1);
        sck_pin.set_high();
    }

    for i in 0..8 {
        let bit = (byte >> (7 - i)) & 1;
        cycle(bit != 0, sck_pin, data_pin)
    }
}

pub trait Displayable {
    fn display_digit(&self, index: u8) -> u8;
}

impl Displayable for u16 {
    fn display_digit(&self, index: u8) -> u8 {
        let thousands = ((*self / 1000) % 10) as u8;
        let hundreds: u8 = ((*self / 100) % 10) as u8;
        let tens: u8 = ((*self / 10) % 10) as u8;
        let ones: u8 = (*self % 10) as u8;

        match index {
            0 => thousands,
            1 => hundreds,
            2 => tens,
            3 => ones,
            _ => unreachable!(),
        }
    }
}

pub struct Display {
    clk_pin: Pin<Output>,
    data_pin: Pin<Output>,
    latch_pin: Pin<Output>,
    index: u8,
}

impl Display {
    pub fn new(clk_pin: Pin<Output>, data_pin: Pin<Output>, latch_pin: Pin<Output>) -> Self {
        Display {
            clk_pin,
            data_pin,
            latch_pin,
            index: 0,
        }
    }

    pub fn update(&mut self, display_value: impl Displayable) {
        let display_byte: u8 = 0b0000_0001;
        self.latch_pin.set_low();
        shift_out(
            display_byte << self.index,
            &mut self.clk_pin,
            &mut self.data_pin,
        );
        let value = display_value.display_digit(self.index);
        shift_out(
            DIGITS[value as usize],
            &mut self.clk_pin,
            &mut self.data_pin,
        );
        self.latch_pin.set_high();
        self.index += 1;
        if self.index > 3 {
            self.index = 0;
        }
    }

    pub fn debug(&mut self) {
        let display_byte: u8 = 0b0000_0001;
        self.latch_pin.set_low();
        shift_out(display_byte, &mut self.clk_pin, &mut self.data_pin);
        shift_out(
            DIGITS[self.index as usize],
            &mut self.clk_pin,
            &mut self.data_pin,
        );
        self.latch_pin.set_high();
        self.index += 1;
        if self.index > 1 {
            self.index = 0;
        }
    }
}
