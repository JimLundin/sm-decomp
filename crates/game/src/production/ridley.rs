//! Ceres Ridley as she enters his room (boss-ceres-ridley part 3), in the played frame: what his
//! placing hands the HUD, and the baby's colours on the screen.
//!
//! He runs first of the room's enemies, on each frame they run; the door of his room runs
//! after him. Fetching his room's contents places him.

use boss_ceres_ridley::{BabyColours, Boss, CeresDoor, CeresRidley, RidleyDrawing};
use data::assets::actors::{BabyShades, CeresRidleyColours, SpriteRow};
use data::assets::art::ROW_COLOURS;
use world_doors::Palette;

/// The screen's first sprite row: the sprite rows follow the eight background rows.
const SPRITE_ROW: usize = 8;

/// A colour of the baby's content has more than 15 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColourFault;

/// The boss his placing hands the HUD as encountered, in the HUD's terms.
#[must_use]
pub const fn encountered(boss: Boss) -> interface_hud::Boss {
    match boss {
        Boss::CeresRidley => interface_hud::Boss::RidleyInCeres,
    }
}

/// `BabyCries`: the screen shows the baby's colour set in its palette at once, colours 1 to
/// 15; the colours the fade heads for are left as they are.
///
/// # Errors
/// [`ColourFault`] for a content colour past 15 bits, leaving the screen as it was.
pub fn show_baby(
    colours: BabyColours,
    content: &CeresRidleyColours,
    palette: &mut Palette,
) -> Result<(), ColourFault> {
    let shades: &BabyShades = match colours {
        BabyColours::Glow1 => &content.glow1,
        BabyColours::Glow2 => &content.glow2,
        BabyColours::Glow3 => &content.glow3,
        BabyColours::Rest => &content.rest,
    };
    let mut screen = palette.screen;
    let mut rows = screen.as_chunks_mut::<ROW_COLOURS>().0.iter_mut();
    if let Some(row) = rows.nth(SPRITE_ROW + usize::from(SpriteRow::BABY.get())) {
        // Colour 0 of a sprite row is transparent; the set is colours 1 to 15.
        for (colour, shade) in row.iter_mut().skip(1).zip(shades) {
            *colour = types::Colour::new(shade.0).ok_or(ColourFault)?;
        }
    }
    palette.screen = screen;
    Ok(())
}

/// What Ridley's drawing draws: him and the baby, and the door of his room while it is drawn
/// with him (`DrawnWithRidley`). The model reports the two apart; drawing reads them together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// His look and the baby's, each at its position.
    pub ridley: RidleyDrawing,
    /// The doors he draws, closed.
    pub doors: Vec<CeresDoor>,
}

/// His drawing with the room's doors as they stand.
#[must_use]
pub fn drawing(ridley: &CeresRidley, doors: &[CeresDoor]) -> Drawing {
    Drawing {
        ridley: ridley.drawing(),
        doors: doors
            .iter()
            .filter(|door| door.phase.drawn_with_ridley())
            .copied()
            .collect(),
    }
}
