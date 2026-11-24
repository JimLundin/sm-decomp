//! Ceres Ridley and the baby in his hold.
//!
//! In Ridley's room he runs first of the room's enemies, the baby in his hold, and the door of
//! his room after him. Fetching his room's contents places him: the HUD meets a boss, the
//! stage is calm and his room's colours are the fade's goal. The baby's colours go to the
//! screen as it glows.

use super::{Content, Fault, Rules, station};
use crate::production::ridley;
use boss_ceres_ridley::{CeresRidley, RidleyPhase, RidleyRun};
use world_doors::Palette;

impl station::State {
    /// Ridley's drawing as the frame leaves him, with the doors he draws; none outside his
    /// room. It is produced for drawing, which this loop doesn't compare.
    #[must_use]
    pub fn ridley_drawing(&self) -> Option<ridley::Drawing> {
        self.ridley
            .as_ref()
            .map(|ridley| ridley::drawing(ridley, &self.doors))
    }
}

/// One run of Ridley, the baby in his hold: its colour set goes to the screen at once. Awake,
/// a later piece runs him.
pub(super) fn run(
    ridley: CeresRidley,
    passage_under_way: bool,
    random: types::RandomSequence,
    content: &Content,
    rules: &Rules,
    palette: &mut Palette,
) -> Result<RidleyRun, Fault> {
    if ridley.phase == RidleyPhase::Awake {
        return Err(Fault::RidleyAwake);
    }
    let run = ridley.run(passage_under_way, random.number.0, &rules.ceres);
    if let Some(colours) = run.colours {
        ridley::show_baby(colours, &content.rooms.actors.ceres_ridley, palette)
            .map_err(|ridley::ColourFault| Fault::BabyColours)?;
    }
    Ok(run)
}
