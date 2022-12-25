use core::prelude::*;
use arduino_hal::{
    adc::{self, Channel},
    clock::MHz16,
    hal::usart::Usart0,
    prelude::*,
    Adc, Usart,
};

pub struct Encoder<'a> {
    adc: Adc,
    clk_channel: &'a Channel,
    dt_channel: &'a Channel,
    last_clk: bool,
}

impl<'a> Encoder<'a> {
    fn analog_read(adc: &mut Adc, channel: &adc::Channel) -> u16 {
        adc.read_blocking(channel)
    }

    pub fn new(adc: Adc, clk_channel: &'a Channel, dt_channel: &'a Channel) -> Self {
        Encoder {
            adc,
            clk_channel,
            dt_channel,
            last_clk: false,
        }
    }

    pub fn debug(&mut self, serial: &mut Usart0<MHz16>) {
        let clk = Self::analog_read(&mut self.adc, self.clk_channel);
        let dt = Self::analog_read(&mut self.adc, self.dt_channel);
        ufmt::uwriteln!(serial, "{} {}\r", clk, dt).void_unwrap();
    }

    pub fn poll(&mut self) -> Option<i8> {
        fn to_bool(x: u16) -> bool {
            x > (1024u16 / 2)
        }
        let current_clk = Self::analog_read(&mut self.adc, self.clk_channel);
        let current_dt = Self::analog_read(&mut self.adc, self.dt_channel);
        let current_clk = to_bool(current_clk);
        let current_dt = to_bool(current_dt);

        // on falling edge
        let ret_val = if !current_clk && self.last_clk {
            match current_dt {
                true => Some(-1),
                false => Some(1),
            }
        } else {
            None
        };
        self.last_clk = current_clk;
        ret_val
    }
}