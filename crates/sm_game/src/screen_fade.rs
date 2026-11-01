//! Screen fade transitions over the presentation's owned state.
#![forbid(unsafe_code)]
pub use crate::visual_state::{CinematicFade, ScreenFade};

/// Admission to the current presentation owner. Compatibility implementations
/// import/export only at their execution boundary when a Session cannot resume.
pub trait Owner {
    fn with_screen_fade<R>(&mut self, action: impl FnOnce(&mut ScreenFade) -> R) -> R;
    fn cinematic_screen_fade(&mut self, fade: CinematicFade) -> bool;
    fn tick_screen_fade(&mut self) -> bool {
        self.with_screen_fade(|screen| screen.tick())
    }
    fn set_screen_fade(&mut self, delay: u16, counter: u16) {
        self.with_screen_fade(|screen| *screen = ScreenFade { delay, counter });
    }
    fn set_screen_fade_counter(&mut self, counter: u16) {
        self.with_screen_fade(|screen| screen.counter = counter);
    }
}

pub fn store<H: Owner + ?Sized>(host: &mut H, delay: u16, counter: u16) {
    host.set_screen_fade(delay, counter);
}

impl ScreenFade {
    /// Whether the ordinary brightness fade advances this frame.
    pub fn tick(&mut self) -> bool {
        let (counter, due) = countdown(self.counter, || self.delay);
        self.counter = counter;
        due
    }

    /// The cinematic's signed low-byte countdown precedes brightness sampling.
    pub fn cinematic_ready(&mut self, fade: CinematicFade) -> bool {
        if matches!(fade, CinematicFade::SlowIn | CinematicFade::SlowOut) {
            let (delay, ready) = cinematic_countdown(self.delay);
            self.delay = delay;
            return ready;
        }
        true
    }

    /// None preserves fade-out's return without a brightness write or reset.
    pub fn cinematic_brightness(&self, fade: CinematicFade, brightness: u8) -> Option<(u8, bool)> {
        let step = if matches!(fade, CinematicFade::SlowIn | CinematicFade::SlowOut) {
            1
        } else {
            self.counter.to_le_bytes()[0]
        };
        cinematic_brightness(fade, brightness, step)
    }

    /// The low-byte stores run after the actual brightness write.
    pub fn cinematic_finish(&mut self, fade: CinematicFade, complete: bool) {
        if matches!(fade, CinematicFade::SlowIn | CinematicFade::SlowOut) {
            let delay = if complete {
                0
            } else {
                self.counter.to_le_bytes()[0]
            };
            self.delay = with_low_byte(self.delay, delay);
            if complete {
                self.counter = with_low_byte(self.counter, 0);
            }
        }
    }
}

/// Select a reload only once the signed word countdown expires.
pub fn countdown(counter: u16, delay: impl FnOnce() -> u16) -> (u16, bool) {
    let remaining = counter.wrapping_sub(1);
    let due = i16::from_ne_bytes(remaining.to_ne_bytes()) < 0;
    (if due { delay() } else { remaining }, due)
}

/// The signed low-byte countdown, preserving the word's high byte.
pub fn cinematic_countdown(delay: u16) -> (u16, bool) {
    let low = delay.to_le_bytes()[0].wrapping_sub(1);
    (with_low_byte(delay, low), i8::from_ne_bytes([low]) <= 0)
}

/// Compute the next physical brightness from the already sampled step.
pub fn cinematic_brightness(fade: CinematicFade, brightness: u8, step: u8) -> Option<(u8, bool)> {
    let increasing = matches!(fade, CinematicFade::FastIn | CinematicFade::SlowIn);
    if !increasing && brightness & 15 == 0 {
        return None;
    }
    Some(if increasing {
        let next = brightness.wrapping_add(step) & 31;
        (next.min(15), next >= 15)
    } else {
        let next = (brightness & 15).wrapping_sub(step);
        let complete = i8::from_ne_bytes([next]) <= 0;
        (if complete { 0x80 } else { next }, complete)
    })
}

fn with_low_byte(word: u16, value: u8) -> u16 {
    (word & 0xff00) | u16::from(value)
}
