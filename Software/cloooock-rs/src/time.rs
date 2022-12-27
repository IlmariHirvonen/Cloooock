use core::{ops::Add, fmt::Display};

use crate::display::Displayable;

const DEFAULT_BPM: u16 = 120;

#[derive(Copy, Clone)]
pub struct BPM {
    pub bpm: u16,
}

impl BPM {
    pub const fn new(bpm: u16) -> Self {
        BPM { bpm }
    }
}

impl From<u16> for BPM {
    fn from(value: u16) -> Self {
        assert!(value <= 9999);
        BPM { bpm: value }
    }
}

impl Add<i8> for BPM {
    type Output = Self;

    fn add(self, rhs: i8) -> Self::Output {
        BPM { bpm: (self.bpm as i16 + rhs as i16) as u16 % 10000 % 5536 }
    }
}

impl Default for BPM {
    /// Default to [DEFAULT_BPM] beats per minute.
    fn default() -> Self {
        Self { bpm: DEFAULT_BPM }
    }
}

impl Displayable for BPM {
    fn display_digit(&self, index: u8) -> u8 {
        self.bpm.display_digit(index)
    }
}