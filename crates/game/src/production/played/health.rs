//! Her health's frame of play: her energy, the warning and her protection.
//!
//! Health plays first. No Ridley's tail is in this loop, so no tail hit lands, and the energy
//! her animation reads, the energy a tail hit leaves before the frame's loss, is the energy the
//! frame starts with.

use super::Rules;

impl super::State {
    /// Health's frame of play, with no tail hit.
    pub(super) const fn health_frame(&self, rules: &Rules) -> samus_health::PlayedFrame {
        self.health
            .0
            .play_frame(samus_health::TailImpact::Absent, &rules.health)
    }
}
