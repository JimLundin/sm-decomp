//! Her health's frame of play: her energy, the warning and her protection.
//!
//! Health plays after the room's enemies, reading the frame's hit of Ridley's tail, the one his
//! run reports: his tail's impact (`TailHit`), judged on her protection as the frame starts.

use super::Rules;
use types::Hit;

impl super::State {
    /// Health's frame of play, with the frame's hit, if any.
    pub(super) const fn health_frame(
        &self,
        hit: Option<Hit>,
        rules: &Rules,
    ) -> samus_health::PlayedFrame {
        self.health.0.play_frame(impact(hit), &rules.health)
    }

    /// The whole energy the frame's hit leaves her, before the frame's loss: what her hurt
    /// colours compare with the energy the frame starts with. Health's own frame tells it with
    /// no loss pending.
    pub(super) const fn energy_the_hit_leaves(&self, hit: Option<Hit>, rules: &Rules) -> u16 {
        samus_health::State {
            pending_loss: samus_health::EnergyLoss::ZERO,
            ..self.health.0
        }
        .play_frame(impact(hit), &rules.health)
        .state
        .energy
        .whole
        .get()
    }
}

/// The frame's hit as Health reads it: Ridley's tail is the one hitter in this loop.
const fn impact(hit: Option<Hit>) -> samus_health::TailImpact {
    match hit {
        Some(_) => samus_health::TailImpact::EligibleCeresTail,
        None => samus_health::TailImpact::Absent,
    }
}
