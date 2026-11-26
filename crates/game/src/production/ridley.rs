//! Ceres Ridley in the played frame (boss-ceres-ridley parts 3 and 4a): what his placing hands
//! the HUD, his content in the model's terms, and the baby's colours and his own on the screen.
//!
//! He runs first of the room's enemies, on each frame they run; the door of his room runs
//! after him. Fetching his room's contents places him.

use boss_ceres_ridley::{
    BabyColours, BodyColours, Boss, CeresDoor, CeresRidley, EyeColours, FadeSets, FlightTail,
    RidleyContent, RidleyDrawing, TAIL_SEGMENTS,
};
use data::assets::actors::{BabyShades, CeresRidleyColours, SpriteRow};
use data::assets::art::{Colour, ROW_COLOURS};
use world_doors::Palette;

/// The screen's first sprite row: the sprite rows follow the eight background rows.
const SPRITE_ROW: usize = 8;
/// The palette he is drawn in, the doors' in his room.
const HIS_ROW: SpriteRow = SpriteRow::CERES_DOORS;
/// The first of his palette's colours his eye's sets cover.
const EYE_FIRST: usize = 12;
/// The first colour of a sprite row that shows: colour 0 is transparent.
const FIRST_SHOWN: usize = 1;

/// A colour of his content or the baby's has more than 15 bits, or one of his fades has no
/// colour set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColourFault;

/// The boss his placing hands the HUD as encountered, in the HUD's terms.
#[must_use]
pub const fn encountered(boss: Boss) -> interface_hud::Boss {
    match boss {
        Boss::CeresRidley => interface_hud::Boss::RidleyInCeres,
    }
}

/// His content as the model reads it: the eye's steps and the body's colour sets, imported.
///
/// # Errors
/// [`ColourFault`] for a colour past 15 bits, or a fade with no set.
pub fn content(colours: &CeresRidleyColours) -> Result<RidleyContent, ColourFault> {
    Ok(RidleyContent {
        eye_steps: fade(&colours.eye_steps, EyeColours)?,
        body_sets: fade(&colours.body_sets, BodyColours)?,
    })
}

/// A fade's sets, each colour checked, in the model's terms.
fn fade<const N: usize, T: Copy>(
    sets: &[[Colour; N]],
    set: fn([types::Colour; N]) -> T,
) -> Result<FadeSets<T>, ColourFault> {
    let sets = sets
        .iter()
        .map(|shades| checked(shades).map(set))
        .collect::<Result<Vec<_>, _>>()?;
    FadeSets::new(sets).ok_or(ColourFault)
}

/// These content colours as the screen's, each within 15 bits.
fn checked<const N: usize>(shades: &[Colour; N]) -> Result<[types::Colour; N], ColourFault> {
    let mut colours = [types::Colour::BLACK; N];
    for (colour, shade) in colours.iter_mut().zip(shades) {
        *colour = types::Colour::new(shade.0).ok_or(ColourFault)?;
    }
    Ok(colours)
}

/// The screen shows `colours` at once in sprite row `row`, from its colour `from` on.
fn show(palette: &mut Palette, row: SpriteRow, from: usize, colours: &[types::Colour]) {
    let mut rows = palette.screen.as_chunks_mut::<ROW_COLOURS>().0.iter_mut();
    if let Some(row) = rows.nth(SPRITE_ROW.saturating_add(usize::from(row.get()))) {
        for (colour, &shown) in row.iter_mut().skip(from).zip(colours) {
            *colour = shown;
        }
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
    show(palette, SpriteRow::BABY, FIRST_SHOWN, &checked(shades)?);
    Ok(())
}

/// `EyeFadesIn`: the screen shows the eye's set at once on colours 12 to 14 of his palette.
pub fn show_eye(colours: EyeColours, palette: &mut Palette) {
    show(palette, HIS_ROW, EYE_FIRST, &colours.0);
}

/// `BodyFadesIn`: the screen shows the body's set at once on colours 1 to 11 of his palette
/// and of his second.
pub fn show_body(colours: BodyColours, palette: &mut Palette) {
    for row in [HIS_ROW, SpriteRow::RIDLEY_SECOND] {
        show(palette, row, FIRST_SHOWN, &colours.0);
    }
}

/// What Ridley's drawing draws: him and the baby, and the door of his room while it is drawn
/// with him (`DrawnWithRidley`). The model reports the two apart; drawing reads them together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// His look and the baby's, each at its position.
    pub ridley: RidleyDrawing,
    /// The seven in-flight tail places, from base to tip.
    pub flight_tail: Option<[types::Pair<types::Px>; TAIL_SEGMENTS]>,
    /// The doors he draws, closed.
    pub doors: Vec<CeresDoor>,
}

/// His drawing with the room's doors as they stand.
#[must_use]
pub fn drawing(ridley: &CeresRidley, doors: &[CeresDoor]) -> Drawing {
    Drawing {
        ridley: ridley.drawing(),
        flight_tail: ridley.flight_tail.as_ref().map(FlightTail::places),
        doors: doors
            .iter()
            .filter(|door| door.phase.drawn_with_ridley())
            .copied()
            .collect(),
    }
}
