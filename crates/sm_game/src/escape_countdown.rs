//! Owned escape countdown; no content, display, RAM or native allocation dependencies.
use crate::scene::state::WordOrder;

/// Every hidden countdown byte is supplied at start or selected-image adoption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub centiseconds: u8,
    pub seconds: u8,
    pub minutes: u8,
    pub x: u16,
    pub y: u16,
    pub native_order: WordOrder,
    pub initialized: bool,
}
/// The existing timing.timer_status remains its single status owner.
pub trait Access {
    fn countdown_apply<T>(&mut self, action: impl FnOnce(&mut State, &mut u16) -> T) -> T;
    fn countdown_byte(&self, offset: usize) -> u8;
    fn countdown_native_word(&self, offset: usize) -> u16;
}
// Only compatibility initialization may construct this uninitialized value.
impl Default for State {
    fn default() -> Self {
        Self {
            centiseconds: 0,
            seconds: 0,
            minutes: 0,
            x: 0,
            y: 0,
            native_order: if cfg!(target_endian = "little") {
                WordOrder::Little
            } else {
                WordOrder::Big
            },
            initialized: false,
        }
    }
}
impl State {
    pub fn initialized(digits: [u8; 3], position: [u16; 2], native_order: WordOrder) -> Self {
        Self {
            centiseconds: digits[0],
            seconds: digits[1],
            minutes: digits[2],
            x: position[0],
            y: position[1],
            native_order,
            initialized: true,
        }
    }
    pub fn byte(&self, offset: usize) -> u8 {
        let [xl, xh] = self.x.to_le_bytes();
        let [yl, yh] = self.y.to_le_bytes();
        [
            self.centiseconds,
            self.seconds,
            self.minutes,
            xl,
            xh,
            yl,
            yh,
        ][offset]
    }
    pub fn native_word(&self, offset: usize) -> u16 {
        self.native_order
            .decode([self.byte(offset), self.byte(offset + 1)])
    }
    fn set_byte(&mut self, offset: usize, value: u8) {
        match offset {
            0 => self.centiseconds = value,
            1 => self.seconds = value,
            2 => self.minutes = value,
            3 => self.x = (self.x & 0xff00) | u16::from(value),
            4 => self.x = (self.x & 0xff) | (u16::from(value) << 8),
            5 => self.y = (self.y & 0xff00) | u16::from(value),
            6 => self.y = (self.y & 0xff) | (u16::from(value) << 8),
            _ => unreachable!(),
        }
    }
    fn set_digits(&mut self, minutes: u16) {
        // Native words overlap at seconds; keep their original publication order.
        let [first, second] = self.native_order.encode(0);
        self.set_byte(0, first);
        self.set_byte(1, second);
        let [first, second] = self.native_order.encode(minutes);
        self.set_byte(1, first);
        self.set_byte(2, second);
    }
    pub fn clear(&mut self, status: &mut u16) {
        self.x = 0x8000;
        self.y = 0x8000;
        self.set_digits(0);
        *status = 0;
    }
    fn start(&mut self, status: &mut u16, minutes: u16) -> bool {
        self.clear(status);
        self.set_digits(minutes);
        *status = 0x8003;
        false
    }
    /// Commit the captured state's updates before any live frame query.
    /// The caller releases this borrow and ticks only when this returns true.
    pub fn prepare(&mut self, status: &mut u16, state: u8) -> bool {
        let states: [fn(&mut Self, &mut u16) -> bool; 7] = [
            |_, _| false,
            |timer, status| timer.start(status, 0x100),
            |timer, status| timer.start(status, 0x300),
            |timer, status| timer.delay(status, 0x10, false),
            |timer, status| timer.delay(status, 0x60, true),
            Self::move_into_place,
            |_, _| true,
        ];
        states[usize::from(state)](self, status)
    }
    fn delay(&mut self, status: &mut u16, limit: u8, count_down: bool) -> bool {
        self.set_byte(3, self.x.to_le_bytes()[0].wrapping_add(1));
        if self.x.to_le_bytes()[0] >= limit {
            if count_down {
                self.set_byte(3, 0);
            }
            *status = (*status & 0xff00) | u16::from(status.to_le_bytes()[0].wrapping_add(1));
        }
        count_down
    }
    fn move_into_place(&mut self, status: &mut u16) -> bool {
        let x = self.x.wrapping_add(224);
        let x_ready = x >= 0xdc00;
        self.x = if x_ready { 0xdc00 } else { x };
        let y = self.y.wrapping_sub(193);
        let y_ready = y < 0x3000;
        self.y = if y_ready { 0x3000 } else { y };
        if x_ready && y_ready {
            *status = status.wrapping_add(1);
        }
        true
    }
    pub fn tick(&mut self, frame: u16) -> u8 {
        const CENTISECOND_DECREMENTS: [u8; 128] = [
            1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2,
            2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2,
            2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1,
            2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1,
            2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2,
        ];
        let amount = CENTISECOND_DECREMENTS[usize::from(frame & 0x7f)];
        if decrement_decimal(&mut self.centiseconds, amount)
            && decrement_decimal(&mut self.seconds, 1)
        {
            if decrement_decimal(&mut self.minutes, 1) {
                self.centiseconds = 0;
                self.seconds = 0;
                self.minutes = 0;
            } else {
                self.seconds = 0x59;
            }
        }
        u8::from((self.centiseconds | self.seconds | self.minutes) == 0)
    }
}
fn decrement_decimal(byte: &mut u8, value: u8) -> bool {
    let mut result = (i32::from(*byte) & 0xf) + (!i32::from(value) & 0xf) + 1;
    if result < 0x10 {
        result = (result - 0x6) & (if result - 0x6 < 0 { 0xf } else { 0x1f });
    }
    result += (i32::from(*byte) & 0xf0) + (!i32::from(value) & 0xf0);
    if result < 0x100 {
        result -= 0x60;
    }
    *byte = result.to_le_bytes()[0];
    result < 0x100
}
