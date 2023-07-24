#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_device::atmega328p::TC1;
use avr_device::{
    atmega328p::tc1::tccr1b::CS1_A, interrupt::Mutex,
};

use arduino_hal::{
    adc::{self},
    port::{mode::Output, Pin},
    prelude::*,
};
use cloooock_rs::cv_output::ClockChannel;
use cloooock_rs::state_machine::{DeviceState, self, ButtonPressed};
use cloooock_rs::time::TICK_RATE;
use cloooock_rs::time::TicksPerBar;
use core::cell::RefCell;
use core::mem;
use ufmt::{uWrite, uwriteln};

use cloooock_rs::cv_output::{ClockOutput, Prescaler};
use cloooock_rs::display::Display;
use cloooock_rs::{display::Displayable, encoder::Encoder};
use cloooock_rs::time::BPM;
use panic_halt as _;

const NUM_CHANNELS: u8 = 4;
const MIN_BPM: u16 = 30;
const MAX_BPM: u16 = 9999;

struct ClockChannels {
    bar_ticks: TicksPerBar,
    channels: [ClockChannel; 4],
}

// global mutable state
static TICKS: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));
static MASTER_BPM: Mutex<RefCell<BPM>> = Mutex::new(RefCell::new(BPM::new(120)));
// output devices
static mut CLOCK_CHANNELS: mem::MaybeUninit<ClockChannels> = mem::MaybeUninit::uninit();


#[arduino_hal::entry]
fn main() -> ! {
    let mut state = DeviceState::Running;
    let mut selected_channel: i8 = 0;
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

    let pause_button = pins.a4.into_floating_input().downgrade();
    let mut pause_button_was_pressed = false;
    let mut pause_button_previous_state = false;

    let output_0 = pins.d7.into_output().downgrade();
    let output_1 = pins.d10.into_output().downgrade();
    let output_2 = pins.d8.into_output().downgrade();
    let output_3 = pins.d9.into_output().downgrade();
    // pinout
    // let encoder_button = pins.d2.into_input().downgrade();
    let encoder_dt_channel = &adc::channel::ADC6.into_channel();
    let encoder_clk_channel = &adc::channel::ADC7.into_channel();
    let encoder_button = pins.d2.into_floating_input().downgrade();
    let mut encoder_button_was_pressed = false;
    let mut encoder_button_previous_state = false;


    let display_latch_pin = pins.d4.into_output().downgrade();
    let display_clk_pin = pins.d5.into_output().downgrade();
    let display_data_pin = pins.d6.into_output().downgrade();
    let mut display = Display::new(display_clk_pin, display_data_pin, display_latch_pin);

    let mut encoder = Encoder::new(adc, encoder_clk_channel, encoder_dt_channel, &encoder_button);

    unsafe {
        // SAFETY: Interrupts are not enabled at this point so we can safely write the global
        // variable here.  A memory barrier afterwards ensures the compiler won't reorder this
        // after any operation that enables interrupts.
        avr_device::interrupt::free(|cs| {
            let bpm_ref = MASTER_BPM.borrow(cs).borrow();
            let ticks_per_bar = TicksPerBar::from(*bpm_ref).ticks;
            CLOCK_CHANNELS = mem::MaybeUninit::new(ClockChannels {
                bar_ticks: TicksPerBar::from(*bpm_ref),
                channels: [
                    ClockChannel::new(led_0, output_0,Prescaler::new(1, 2),ticks_per_bar), 
                    ClockChannel::new(led_1, output_1,Prescaler::new(1, 4),ticks_per_bar),
                    ClockChannel::new(led_2, output_2,Prescaler::new(1, 6),ticks_per_bar),
                    ClockChannel::new(led_3, output_3,Prescaler::new(1, 8),ticks_per_bar)
                ]
            });
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        });
    }

    // timers
    let tmr1: TC1 = dp.TC1;
    rig_timer1(&tmr1, &mut serial);

    ufmt::uwriteln!(&mut serial, "Start enable interrupts").void_unwrap();
    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe {
        // SAFETY: Not inside a critical section and any non-atomic operations have been completed
        // at this point.
        avr_device::interrupt::enable();
    }
    ufmt::uwriteln!(&mut serial, "Done enable interrupts").void_unwrap();

    loop {

        if pause_button_previous_state && pause_button.is_high() {
            pause_button_was_pressed = true;
        } else {
            pause_button_was_pressed = false;
        }
        pause_button_previous_state = pause_button.is_low();

        if encoder_button_previous_state && encoder_button.is_high() {
            encoder_button_was_pressed = true;
        } else {
            encoder_button_was_pressed = false;
        }
        encoder_button_previous_state = encoder_button.is_low();



        match state {
            DeviceState::Running => {
                if let Some(change) = encoder.poll() {
                    avr_device::interrupt::free(|cs| {
                        let mut bpm_ref = MASTER_BPM.borrow(cs).borrow_mut();
                        if !((bpm_ref.bpm <= MIN_BPM && change < 0) || (bpm_ref.bpm >= MAX_BPM && change > 0 )) {
                            *bpm_ref = *bpm_ref + change;
                        }
        
                        unsafe {
                            (&mut *CLOCK_CHANNELS.as_mut_ptr()).bar_ticks = TicksPerBar::from(*bpm_ref);
                        }
                        let state = unsafe { &mut *CLOCK_CHANNELS.as_mut_ptr() };
                        let ticks_per_bar = TicksPerBar::from(*bpm_ref).ticks;
                        for channel in state.channels.iter_mut() {
                            channel.calculate_threshold(ticks_per_bar);
                        }
                    });
                }
                avr_device::interrupt::free(|cs| {
                    let bpm_ref = MASTER_BPM.borrow(cs).borrow();
                    let mut ticks_ref = TICKS.borrow(cs).borrow_mut();
                    let state = unsafe { &mut *CLOCK_CHANNELS.as_mut_ptr() };
                    if *ticks_ref >= state.bar_ticks.ticks {
                        *ticks_ref = 0;
                        for channel in state.channels.iter_mut() {
                            channel.reset_threshold();
                        }
                    }
                    for channel in state.channels.iter_mut() {
                        channel.update(*ticks_ref);
                    }
                });
                avr_device::interrupt::free(|cs| {
                    let bpm_ref = MASTER_BPM.borrow(cs).borrow();
                    display.update(*bpm_ref);
                });
                if pause_button_was_pressed {
                    //ufmt::uwriteln!(&mut serial, "Pausing").void_unwrap();
                    state = state.transition(ButtonPressed::PauseButton);
                }
                if encoder_button_was_pressed {
                    let clock_channels = unsafe { &mut *CLOCK_CHANNELS.as_mut_ptr() };
                    for channel in clock_channels.channels.iter_mut() {
                        channel.set_led(false);
                    }
                    clock_channels.channels[selected_channel as usize].set_led(true);
                    state = state.transition(ButtonPressed::EncoderButton);
                }
            }

            DeviceState::Paused => {
                if pause_button_was_pressed {
                    state = state.transition(ButtonPressed::PauseButton);
                }
                avr_device::interrupt::free(|cs| {
                    let bpm_ref = MASTER_BPM.borrow(cs).borrow();
                    display.update(*bpm_ref);
                });
            }

            DeviceState::SelectingChannel => {
                let clock_channels = unsafe { &mut *CLOCK_CHANNELS.as_mut_ptr() };
                if let Some(change) = encoder.poll() {
                    selected_channel += change;
                    if selected_channel > 3 {
                        selected_channel = 0;
                    } else if selected_channel < 0 {
                        selected_channel = 3;
                    }
                    for channel in clock_channels.channels.iter_mut() {
                        channel.set_led(false);
                    }
                    clock_channels.channels[selected_channel as usize].set_led(true); 
                }
                if encoder_button_was_pressed {
                    
                    for channel in clock_channels.channels.iter_mut() {
                        channel.set_led(false);
                    }
                    clock_channels.channels[selected_channel as usize].set_led(true);
                    state = state.transition(ButtonPressed::EncoderButton);
                }
                if pause_button_was_pressed {
                    for channel in clock_channels.channels.iter_mut() {
                        channel.reset_threshold();
                    }
                    state = state.transition(ButtonPressed::PauseButton);
                }
                avr_device::interrupt::free(|cs| {
                    let bpm_ref = MASTER_BPM.borrow(cs).borrow();
                    display.update(*bpm_ref);
                });
                
            }
            DeviceState::SettingDivisionState => {
                let clock_channels = unsafe { &mut *CLOCK_CHANNELS.as_mut_ptr() };  
                if let Some(change) = encoder.poll() {
                    clock_channels.channels[selected_channel as usize].set_led(true);
                    clock_channels.channels[selected_channel as usize].update_denominator(change, clock_channels.bar_ticks.ticks);
                }
                display.update(clock_channels.channels[selected_channel as usize].get_denominator());
                if encoder_button_was_pressed {
                    for channel in clock_channels.channels.iter_mut() {
                        channel.set_led(false);
                    }
                    clock_channels.channels[selected_channel as usize].set_led(true);
                    state = state.transition(ButtonPressed::EncoderButton);
                }
                if pause_button_was_pressed {
                    for channel in clock_channels.channels.iter_mut() {
                        channel.reset_threshold();
                    }
                    state = state.transition(ButtonPressed::PauseButton);
                }
            }
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

#[avr_device::interrupt(atmega328p)]
fn TIMER1_COMPA() {
    avr_device::interrupt::free(|cs| {
        // Interrupts are disabled here
        let mut ticks_ref = TICKS.borrow(cs).borrow_mut();
        *ticks_ref += 1;
    });
}
