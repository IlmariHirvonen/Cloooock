#![no_std]

pub mod encoder;
pub mod display;

struct Prescaler {
    numerator: u16,
    denominator: u16,
}

// reset at the start of every bar.
// struct ClockChannel {
//     start_us: u32,
//     prescaler: Prescaler,
//     output_state: bool,
//     led_pin: u16,       // TODO: use avr-pac types
//     channel_pin: u16,   // TODO: use avr-pac types
// }

// impl ClockChannel {
//     fn new() -> Self {
//         ClockChannel {
//             start_us: 0,
//             prescaler: Prescaler {
//                 numerator: 1,
//                 denominator: 1,
//             },
//             output_state: false,
//             led_pin: u16,       // TODO: use avr-pac types
//             channel_pin: u16,   // TODO: use avr-pac types
//         }
//     }
// }
