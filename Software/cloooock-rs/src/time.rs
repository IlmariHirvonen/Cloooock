use core::ops::Add;

use crate::display::Displayable;

const DEFAULT_BPM: u16 = 120;
const MAX_BPM: u16 = 10_000;

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
        let bpm: u16 = if intermediate < 0 {
            0
        } else if intermediate > MAX_BPM as i16 {
            MAX_BPM
        } else {
            intermediate as u16
        };
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

pub const TICK_RATE: u32 = 10_000;

// Prescaler    Counter Resolution [us]     Counter Overflow [s]
//---------------------------------------------------------------------
// 1            0.0625                      0.0040959375
// 8            0.5                         0.0327675
// 64           4.0                         0.26214
// 256          16.0                        1.04856
// 1024         64.0                        4.19424

pub struct TicksPerBar {
    pub ticks: u32,
}

impl From<BPM> for TicksPerBar {
    fn from(bpm: BPM) -> Self {
        TicksPerBar {
            ticks: (240 / 2 * TICK_RATE) / bpm.bpm as u32,
        }
    }
}
