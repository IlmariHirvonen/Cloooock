#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_device::{atmega328p::tc1::tccr1b::CS1_A, interrupt::Mutex};
use avr_device::atmega328p::TC1;
use arduino_hal::{
    adc::{
        self,
    },
    port::{mode::Output, Pin},
    prelude::*,
};
use ufmt::{uWrite, uwriteln};
use core::cell::Cell;
use core::mem;

use cloooock_rs::{encoder::Encoder, display::Displayable};
use cloooock_rs::display::Display;
use panic_halt as _;


const NUM_CHANNELS: u8 = 4;

struct InterruptState {
    display: Display
}

// global mutable state
static mut BPM: Mutex<Cell<u16>> = Mutex::new(Cell::new(120));
static mut INTERRUPT_STATE: mem::MaybeUninit<InterruptState> = mem::MaybeUninit::uninit();

// Pinout
//
// Purpose  Arduino Pin     AVR Pin
// Led 1    A3              PC3
// Led 2    A2              PC2
// Led 3    A1              PC1
// Led 4    A0              PC0

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut leds: [Pin<Output>; 4] = [
        pins.a0.into_output().downgrade(),
        pins.a1.into_output().downgrade(),
        pins.a2.into_output().downgrade(),
        pins.a3.into_output().downgrade(),
        ];
        
    // pinout
    // let encoder_button = pins.d2.into_input().downgrade();
    let encoder_dt_channel = &adc::channel::ADC6.into_channel();
    let encoder_clk_channel = &adc::channel::ADC7.into_channel();
    let mut display_latch_pin = pins.d4.into_output().downgrade();
    let mut display_clk_pin = pins.d5.into_output().downgrade();
    let mut display_data_pin = pins.d6.into_output().downgrade();

    let mut encoder = Encoder::new(adc, encoder_clk_channel, encoder_dt_channel);
    let mut display = Display::new(
        display_clk_pin,
        display_data_pin,
        display_latch_pin,
    );

    unsafe {
        // SAFETY: Interrupts are not enabled at this point so we can safely write the global
        // variable here.  A memory barrier afterwards ensures the compiler won't reorder this
        // after any operation that enables interrupts.
        INTERRUPT_STATE = mem::MaybeUninit::new(InterruptState {
            display
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }

    let tmr1: TC1 = dp.TC1;
    ufmt::uwriteln!(&mut serial, "Start rig_timer").void_unwrap();
    rig_timer(&tmr1, &mut serial);
    ufmt::uwriteln!(&mut serial, "Done rig_timer").void_unwrap();

    ufmt::uwriteln!(&mut serial, "Start enable interrupts").void_unwrap();
    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe {
        // SAFETY: Not inside a critical section and any non-atomic operations have been completed
        // at this point.
        avr_device::interrupt::enable();
    }
    ufmt::uwriteln!(&mut serial, "Done enable interrupts").void_unwrap();
    
    loop {
        // for led in leds.iter_mut() {
        //     led.set_high();
        // }
        // arduino_hal::delay_ms(1000);
        // for led in leds.iter_mut() {
        //     led.set_low();
        // }
        // arduino_hal::delay_ms(1000);

        // encoder.debug(&mut serial);

        // ufmt::uwriteln!(&mut serial, "{}!\r", enc_value).void_unwrap();
        if let Some(change) = encoder.poll() {
            ufmt::uwriteln!(&mut serial, "{}\r", change).void_unwrap();

            avr_device::interrupt::free(|cs| {
                unsafe {
                    let cell = BPM.borrow(cs);
                    let mut bpm = cell.take();
                    bpm = (bpm as i16 + change as i16) as u16;
                    cell.set(bpm);
                }
            })
        }
    }


}


pub const fn calc_overflow(clock_hz: u32, target_hz: u32, prescale: u32) -> u32 {
    /*
    https://github.com/Rahix/avr-hal/issues/75
    reversing the formula F = 16 MHz / (256 * (1 + 15624)) = 4 Hz
     */
    clock_hz / target_hz / prescale - 1
}

pub fn rig_timer<W: uWrite<Error = void::Void>>(tmr1: &TC1, serial: &mut W) {
    /*
     https://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf
     section 15.11
    */
    use arduino_hal::clock::Clock;

    const ARDUINO_UNO_CLOCK_FREQUENCY_HZ: u32 = arduino_hal::DefaultClock::FREQ;
    const CLOCK_SOURCE: CS1_A = CS1_A::PRESCALE_64;
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

    let ticks = calc_overflow(ARDUINO_UNO_CLOCK_FREQUENCY_HZ, 300, clock_divisor) as u16;
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

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    let state = unsafe {
        // SAFETY: Interrupts will only be enabled after global state init
        &mut *INTERRUPT_STATE.as_mut_ptr()
    };

    avr_device::interrupt::free(|cs| {
        unsafe {
            let cell = BPM.borrow(cs);
            let bpm = cell.take();
            state.display.update(bpm);
            cell.set(bpm);
        }
    })

}