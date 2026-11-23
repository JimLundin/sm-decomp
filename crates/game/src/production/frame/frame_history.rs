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

pub(super) const fn newly_pressed(held: u16, previous: u16) -> u16 {
    (held ^ previous) & held
}

pub use crate::production::health_tail::{EnergyChange, EnergyRead};

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct ControllerRepeat {
    initial_delay: u16,
    timer: u16,
}

impl ControllerRepeat {
    fn after_sample(self, held: u16, previous: u16) -> Option<(Self, bool)> {
        if held == 0 || held != previous {
            return Some((
                Self {
                    timer: self.initial_delay,
                    ..self
                },
                false,
            ));
        }
        let timer = self.timer.wrapping_sub(1);
        (timer != 0).then_some((Self { timer, ..self }, true))
    }
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
    controller_repeat: Option<ControllerRepeat>,
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
            controller_repeat: None,
            progress: Progress::Initial,
        }
    }

    pub(super) fn complete_drawing(&mut self, state: &State, health: &samus_health::State) -> bool {
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
        {
            return false;
        }
        self.auto_jump_timer = 0;
        self.previous_held = self.held;
        self.previous_pressed = self.pressed;
        EnergyRead::read_and_commit(health, &mut self.previous_health);
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

    /// Admit the bounded repeat timer and initial delay from the same original input image.
    pub const fn admit_controller_repeat(&mut self, initial_delay: u16, timer: u16) {
        self.controller_repeat = Some(ControllerRepeat {
            initial_delay,
            timer,
        });
    }

    pub(super) fn controller_tail_repeats(&self, held: u16) -> Option<bool> {
        self.controller_repeat?
            .after_sample(held, self.native_latch)
            .map(|(_, repeats)| repeats)
    }

    pub(super) fn capture_nmi(&mut self, held: u16) -> bool {
        if !matches!(self.progress, Progress::Finalized) {
            return false;
        }
        if let Some(repeat) = self.controller_repeat {
            let Some((next, _)) = repeat.after_sample(held, self.native_latch) else {
                return false;
            };
            self.controller_repeat = Some(next);
        }
        self.pressed = newly_pressed(held, self.native_latch);
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

    /// Whole energy retained by the last completed Drawing.
    #[must_use]
    pub const fn previous_health(&self) -> u16 {
        self.previous_health
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

#[cfg(test)]
mod tests {
    use super::{EnergyChange, EnergyRead};
    use samus_health::{EnergyAmount, EnergyLoss, ProtectionFrames, State, Warning, WholeEnergy};

    #[test]
    fn drawing_history_commits_the_second_live_energy_read() {
        let mut history = 53;
        let mut health = State {
            energy: EnergyAmount {
                whole: WholeEnergy::new(41).unwrap(),
                fraction: 0,
            },
            pending_loss: EnergyLoss::ZERO,
            warning: Warning::Off,
            protection: ProtectionFrames::ZERO,
        };
        assert_eq!(
            EnergyRead::compare(&health, history),
            EnergyRead {
                current: 41,
                previous: 53,
                change: EnergyChange::Decreased
            },
        );
        health.energy.whole = WholeEnergy::new(38).unwrap();
        assert_eq!(EnergyRead::read_and_commit(&health, &mut history), 38);
        assert_eq!(history, 38);
        assert_eq!(
            EnergyRead::compare(&health, history).change,
            EnergyChange::Held
        );
    }
}
