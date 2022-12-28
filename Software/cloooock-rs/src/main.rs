#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_device::atmega328p::TC1;
use avr_device::atmega328p::TC2;
use avr_device::{
    atmega328p::tc1::tccr1b::CS1_A, atmega328p::tc2::tccr2b::CS2_A, interrupt::Mutex,
};

use arduino_hal::{
    adc::{self},
    port::{mode::Output, Pin},
    prelude::*,
};
use cloooock_rs::cv_output::ClockChannel;
use cloooock_rs::time::TICK_RATE;
use cloooock_rs::time::TicksPerBar;
use core::array;
use core::borrow::BorrowMut;
use core::cell::Cell;
use core::cell::RefCell;
use core::mem;
use ufmt::{uWrite, uwriteln};

use cloooock_rs::cv_output::{ClockOutput, Prescaler};
use cloooock_rs::display::Display;
use cloooock_rs::{display::Displayable, encoder::Encoder};
use cloooock_rs::time::BPM;
use panic_halt as _;

const NUM_CHANNELS: u8 = 4;

struct ClockChannels {
    channels: [ClockChannel; 4],
}

// global mutable state
static TICKS: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));
static MASTER_BPM: Mutex<RefCell<BPM>> = Mutex::new(RefCell::new(BPM::new(120)));
// output devices
static mut DISPLAY: mem::MaybeUninit<Display> = mem::MaybeUninit::uninit();
static mut CLOCK_CHANNELS: mem::MaybeUninit<ClockChannels> = mem::MaybeUninit::uninit();


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    // Pinout
    // Purpose  Arduino Pin     AVR Pin
    // Led 1    A3              PC3
    // Led 2    A2              PC2
    // Led 3    A1              PC1
    // Led 4    A0              PC0
    let led_0 = pins.a0.into_output().downgrade();
    let led_1 = pins.a1.into_output().downgrade();
    let led_2 = pins.a2.into_output().downgrade();
    let led_3 = pins.a3.into_output().downgrade();

    let output_0 = pins.d7.into_output().downgrade();
    let output_1 = pins.d10.into_output().downgrade();
    let output_2 = pins.d8.into_output().downgrade();
    let output_3 = pins.d9.into_output().downgrade();

    // pinout
    // let encoder_button = pins.d2.into_input().downgrade();
    let encoder_dt_channel = &adc::channel::ADC6.into_channel();
    let encoder_clk_channel = &adc::channel::ADC7.into_channel();
    let display_latch_pin = pins.d4.into_output().downgrade();
    let display_clk_pin = pins.d5.into_output().downgrade();
    let display_data_pin = pins.d6.into_output().downgrade();
    let mut display = Display::new(display_clk_pin, display_data_pin, display_latch_pin);

    let mut encoder = Encoder::new(adc, encoder_clk_channel, encoder_dt_channel);

    unsafe {
        // SAFETY: Interrupts are not enabled at this point so we can safely write the global
        // variable here.  A memory barrier afterwards ensures the compiler won't reorder this
        // after any operation that enables interrupts.

        CLOCK_CHANNELS = mem::MaybeUninit::new(ClockChannels {
            channels: [
                ClockChannel::new(led_0, output_0),
                ClockChannel::new(led_1, output_1),
                ClockChannel::new(led_2, output_2),
                ClockChannel::new(led_3, output_3)
            ]
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }

    // timers
    let tmr1: TC1 = dp.TC1;
    rig_timer1(&tmr1, &mut serial);
     //let tmr2: TC2 = dp.TC2;
     //rig_timer2(&tmr2, &mut serial);

    ufmt::uwriteln!(&mut serial, "Start enable interrupts").void_unwrap();
    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe {
        // SAFETY: Not inside a critical section and any non-atomic operations have been completed
        // at this point.
        avr_device::interrupt::enable();
    }
    ufmt::uwriteln!(&mut serial, "Done enable interrupts").void_unwrap();

    loop {
        // ufmt::uwriteln!(&mut serial, "{}!\r", enc_value).void_unwrap();
        // unsafe {ufmt::uwriteln!(&mut serial, "{}\r", TIMER_2_VALUE).void_unwrap();}
        if let Some(change) = encoder.poll() {
            // ufmt::uwriteln!(&mut serial, "{}\r", change).void_unwrap();

            avr_device::interrupt::free(|cs| {
                let mut bpm_ref = MASTER_BPM.borrow(cs).borrow_mut();
                *bpm_ref = *bpm_ref + change;
            });
        }

        avr_device::interrupt::free(|cs| {
            let bpm_ref = MASTER_BPM.borrow(cs).borrow();
            display.update(*bpm_ref);
        });

        avr_device::interrupt::free(|cs| {
            let bpm_ref = MASTER_BPM.borrow(cs).borrow();
            let mut ticks_ref = TICKS.borrow(cs).borrow_mut();
            let state = unsafe { &mut *CLOCK_CHANNELS.as_mut_ptr() };
            let bar_ticks = TicksPerBar::from(BPM::from(bpm_ref.bpm*4));
            if *ticks_ref >= bar_ticks.ticks {
                *ticks_ref = 0;
                for channel in state.channels.iter_mut() {
                    channel.update(*ticks_ref);
                }
            }
        });
    }
}

pub const fn calc_overflow(clock_hz: u32, target_hz: u32, prescale: u32) -> u32 {
    /*
    https://github.com/Rahix/avr-hal/issues/75
    reversing the formula F = 16 MHz / (256 * (1 + 15624)) = 4 Hz
     */
    clock_hz / target_hz / prescale - 1
}

fn rig_timer1<W: uWrite<Error = void::Void>>(tmr1: &TC1, serial: &mut W) {
    /*
     https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf
     section 15.11
    */
    use arduino_hal::clock::Clock;

    const ARDUINO_UNO_CLOCK_FREQUENCY_HZ: u32 = arduino_hal::DefaultClock::FREQ;
    const CLOCK_SOURCE: CS1_A = CS1_A::PRESCALE_8;
    let clock_divisor: u32 = match CLOCK_SOURCE {
        CS1_A::DIRECT => 1,
        CS1_A::PRESCALE_8 => 8,
        CS1_A::PRESCALE_64 => 64,
        CS1_A::PRESCALE_256 => 256,
        CS1_A::PRESCALE_1024 => 1024,
        CS1_A::NO_CLOCK | CS1_A::EXT_FALLING | CS1_A::EXT_RISING => {
            uwriteln!(serial, "uhoh, code tried to set the clock source to something other than a static prescaler {}", CLOCK_SOURCE as usize)
                .void_unwrap();
            1
        }
    };

    let ticks = calc_overflow(ARDUINO_UNO_CLOCK_FREQUENCY_HZ, TICK_RATE, clock_divisor) as u16;
    ufmt::uwriteln!(
        serial,
        "configuring timer output compare register = {}\r",
        ticks
    )
    .void_unwrap();

    tmr1.tccr1a.write(|w| w.wgm1().bits(0b00));
    tmr1.tccr1b.write(|w| {
        w.cs1()
            //.prescale_256()
            .variant(CLOCK_SOURCE)
            .wgm1()
            .bits(0b01)
    });
    tmr1.ocr1a.write(|w| unsafe { w.bits(ticks) });
    tmr1.timsk1.write(|w| w.ocie1a().set_bit()); //enable this specific interrupt
}

fn rig_timer2<W: uWrite<Error = void::Void>>(tmr2: &TC2, serial: &mut W) {
    /*
     https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf
     section 15.11
    */
    const CLOCK_SOURCE: CS2_A = CS2_A::PRESCALE_256;
    let clock_divisor: u32 = match CLOCK_SOURCE {
        CS2_A::DIRECT => 1,
        CS2_A::PRESCALE_8 => 8,
        CS2_A::PRESCALE_32 => 32,
        CS2_A::PRESCALE_64 => 64,
        CS2_A::PRESCALE_128 => 128,
        CS2_A::PRESCALE_256 => 256,
        CS2_A::PRESCALE_1024 => 1024,
        CS2_A::NO_CLOCK => {
            uwriteln!(serial, "uhoh, code tried to set the clock source to something other than a static prescaler {}", CLOCK_SOURCE as usize)
                .void_unwrap();
            1
        }
    };

    let ticks = calc_overflow(16_000_000, 50, clock_divisor) as u8;
    ufmt::uwriteln!(
        serial,
        "configuring timer output compare register = {}\r",
        ticks
    )
    .void_unwrap();

    tmr2.tccr2a.write(|w| w.wgm2().bits(0b00));
    tmr2.tccr2b.write(|w| {
        w.cs2()
            .variant(CLOCK_SOURCE)
            .wgm22()
            .bit(true)
    });
    tmr2.ocr2a.write(|w| unsafe { w.bits(ticks) });
    tmr2.timsk2.write(|w| w.ocie2a().set_bit()); //enable this specific interrupt
}

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    avr_device::interrupt::free(|cs| {
        // Interrupts are disabled here
        let mut ticks_ref = TICKS.borrow(cs).borrow_mut();
        *ticks_ref += 1;
    });
}
