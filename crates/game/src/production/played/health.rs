//! Her health's frame of play: her energy, the warning and her protection.
//!
//! Health plays after the room's enemies, reading the impact the hitter offers, judged on
//! her protection as the frame starts.

use super::{Content, Rules, station::Contact};
use samus_health::Impact;

impl super::State {
    /// Health's frame of play, with the frame's hit, if any.
    pub(super) const fn health_frame(
        &self,
        hit: Option<Contact>,
        content: &Content,
        rules: &Rules,
    ) -> samus_health::PlayedFrame {
        self.health
            .0
            .play_frame(impact(hit, content), &rules.health)
    }

    /// The whole energy the frame's hit leaves her, before the frame's loss: what her hurt
    /// colours compare with the energy the frame starts with. Health's own frame tells it with
    /// no loss pending.
    pub(super) const fn energy_the_hit_leaves(
        &self,
        hit: Option<Contact>,
        content: &Content,
        rules: &Rules,
    ) -> u16 {
        samus_health::State {
            pending_loss: samus_health::EnergyLoss::ZERO,
            ..self.health.0
        }
        .play_frame(impact(hit, content), &rules.health)
        .state
        .energy
        .whole
        .get()
    }
}

const fn impact(hit: Option<Contact>, content: &Content) -> Impact {
    match hit {
        Some(Contact::Tail(_)) => Impact::EligibleHit {
            worth: content.tail_worth,
        },
        Some(Contact::Steam(_)) => Impact::EligibleHit {
            worth: samus_health::WholeEnergy::ZERO,
        },
        None => samus_health::Impact::Absent,
    }
}
