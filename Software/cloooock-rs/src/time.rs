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
        let intermediate: i16 = self.bpm as i16 + rhs as i16;
        let bpm: u16 = if (intermediate < 0) {
            9999
        } else if (intermediate >= 10000) {
            0
        } else {
            intermediate
        } as u16;
        BPM { bpm }
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