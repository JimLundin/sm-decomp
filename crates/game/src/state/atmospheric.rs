//! The atmospheric graphics: the splashes, bubbles and dust around Samus, four slots the
//! Samus drawing handler animates and draws before her body (bank `$90` log, `$90:8A4C`).

use canonical::Canonical;

/// Atmospheric graphic slots (`$0AD4` to `$0AF3`, four words per field).
pub const ATMOSPHERIC_SLOTS: usize = 4;

/// The slot the landing and wall-jump graphics use (`$91:FA76`, `$91:F166`).
pub const LANDING_SLOT: usize = 3;

/// What an atmospheric graphic shows: its type byte (`$0AED`).
///
/// Type 2 has no sprite table (`$90:8BFF` gives it none) and nothing spawns it, so it has no
/// variant.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtmosphericKind {
    /// Type 1: footstep splashes in shallow liquid (`$90:8112`).
    FootstepSplash,
    /// Type 3: the splash of diving into liquid (`$90:80F8`).
    DivingSplash,
    /// Type 4: lava or acid surface damage (`$90:826A`).
    LavaSurfaceDamage,
    /// Type 5: air bubbles under water (`$90:8159`).
    Bubbles,
    /// Type 6: dust from landing or a wall jump (`$91:F186`, `$91:FA96`).
    LandingDust,
    /// Type 7: dust from speed boosting (`$90:EED4`).
    SpeedBoostDust,
}

/// A graphic's animation frame, from 0 (`$0AEC`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtmosphericFrame(pub u8);

/// A graphic's animation timer word (`$0AD4`): frames left in its frame, or, with the top
/// bit set, a hold before it first shows.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtmosphericTimer(pub u16);

/// What one frame's count-down tells the handler (`$90:8A65`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerStep {
    /// The frame runs on: draw it.
    Draw,
    /// The frame ran out: start the next one.
    Advance,
    /// The hold ran out: start this frame's timer and draw it.
    Release,
    /// Still held, or a timer that wrapped below zero: neither drawn nor advanced.
    Wait,
}

impl AtmosphericTimer {
    /// The hold bit: a timer counting down from it shows the graphic only once it reaches it.
    const HOLD: u16 = 1 << 15;

    /// Count a frame down and say what it means.
    pub const fn count_down(&mut self) -> TimerStep {
        self.0 = self.0.wrapping_sub(1);
        let left = i16::from_le_bytes(self.0.to_le_bytes());
        if left == 0 {
            TimerStep::Advance
        } else if left > 0 {
            TimerStep::Draw
        } else if self.0 == Self::HOLD {
            TimerStep::Release
        } else {
            TimerStep::Wait
        }
    }
}

/// One live atmospheric graphic.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtmosphericGraphic {
    /// What it shows.
    pub kind: AtmosphericKind,
    /// Its animation frame.
    pub frame: AtmosphericFrame,
    /// Its animation timer.
    pub timer: AtmosphericTimer,
    /// Where it is in the room, in pixels (`$0ADC`).
    pub x: u16,
    /// Where it is in the room, in pixels (`$0AE4`).
    pub y: u16,
}

/// The atmospheric graphic slots; `None` where the frame and type word is zero.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Atmospherics {
    /// The slots, drawn from the last to the first.
    pub slots: [Option<AtmosphericGraphic>; ATMOSPHERIC_SLOTS],
}
