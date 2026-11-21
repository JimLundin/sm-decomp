//! Actual S6 Drawing and final controller histories.

use super::{Caller, State};
use canonical::Canonical;

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Progress {
    Initial,
    Drawn,
    Finalized,
    Native,
}

/// Current and prior controller and drawing quantities retained at their actual uses.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct History {
    held: u16,
    pressed: u16,
    previous_held: u16,
    previous_pressed: u16,
    previous_health: u16,
    auto_jump_timer: u16,
    jump: u16,
    native_latch: u16,
    progress: Progress,
}

impl History {
    /// Admit separately decoded input, previous-frame and equipment quantities from origin.
    #[must_use]
    pub const fn new(
        current: [u16; 2],
        previous: [u16; 2],
        previous_health: u16,
        auto_jump_timer: u16,
        jump: u16,
        native_latch: u16,
    ) -> Self {
        let [held, pressed] = current;
        let [previous_held, previous_pressed] = previous;
        Self {
            held,
            pressed,
            previous_held,
            previous_pressed,
            previous_health,
            auto_jump_timer,
            jump,
            native_latch,
            progress: Progress::Initial,
        }
    }

    pub(super) fn complete_drawing(&mut self, state: &State, energy: u16) -> bool {
        let Caller::PlayerContinuation { completed, .. } = &state.caller else {
            return false;
        };
        let Some(pose) = completed.0.pose_input.as_ref() else {
            return false;
        };
        if self.progress != Progress::Initial
            || self.held != pose.dependency.held.0
            || self.pressed != pose.dependency.pressed.0
            || self.jump != pose.dependency.bindings.jump.0
            || self.held & self.jump != 0
            || energy.wrapping_sub(self.previous_health) & (1 << 15) != 0
        {
            return false;
        }
        self.auto_jump_timer = 0;
        self.previous_held = self.held;
        self.previous_pressed = self.pressed;
        self.previous_health = energy;
        self.progress = Progress::Drawn;
        true
    }

    pub(super) const fn finish_held(&mut self) -> bool {
        if !matches!(self.progress, Progress::Drawn) {
            return false;
        }
        self.previous_held = self.held;
        self.progress = Progress::Finalized;
        true
    }

    pub(super) const fn capture_nmi(&mut self, held: u16) -> bool {
        if !matches!(self.progress, Progress::Finalized) {
            return false;
        }
        self.pressed = (held ^ self.native_latch) & held;
        self.held = held;
        self.native_latch = held;
        self.progress = Progress::Native;
        true
    }

    pub(super) const fn finalized_for_native(&self, returned: bool) -> bool {
        matches!(
            (self.progress, returned),
            (Progress::Finalized, false) | (Progress::Native, true)
        )
    }

    pub(super) const fn begin_opportunity(&mut self) {
        self.progress = Progress::Initial;
    }

    /// Current held and newly pressed controller words.
    #[must_use]
    pub const fn controller(&self) -> [u16; 2] {
        [self.held, self.pressed]
    }

    /// The current held and pressed words captured by the completed native return.
    #[must_use]
    pub const fn native_controller(&self) -> Option<[u16; 2]> {
        match self.progress {
            Progress::Native => Some(self.controller()),
            Progress::Initial | Progress::Drawn | Progress::Finalized => None,
        }
    }

    /// The actual Drawing history is complete before any final history.
    #[must_use]
    pub const fn drawn(&self) -> bool {
        matches!(
            self.progress,
            Progress::Drawn | Progress::Finalized | Progress::Native
        )
    }
}
