//! The screen's colours and the colours a fade heads for: their groups, seed and compared fields.

use crate::replay::compare::Observation;
use crate::replay::data_failure;
use crate::replay::result::Outcome;
use world_doors::{ColourGroup, Palette};

/// The screen's colours as they stand, and the colours a fade heads for.
const SCREEN_COLOURS: usize = 0xc000;
const GOAL_COLOURS: usize = 0xc200;

/// The colours as the record holds them.
pub(super) fn palette(wram: &[u8]) -> Result<Palette, Outcome> {
    Ok(Palette {
        screen: colours(wram, SCREEN_COLOURS)?,
        goal: colours(wram, GOAL_COLOURS)?,
    })
}

/// The doors' colour groups, by colour index: which colours the passage lights, which belong
/// to the HUD's icons, the escape timer and the visor, and the rest.
pub(super) fn groups() -> [ColourGroup; world_doors::COLOURS] {
    std::array::from_fn(|index| match index {
        9 | 10 | 13 | 14 | 17..=19 | 29 => ColourGroup::Lit,
        20..=23 | 28 => ColourGroup::Icon,
        209 | 210 | 212 | 221 => ColourGroup::Timer,
        196 => ColourGroup::Visor,
        _ => ColourGroup::Other,
    })
}

/// The 256 colours at `at`, as the record holds them.
fn colours(wram: &[u8], at: usize) -> Result<[types::Colour; world_doors::COLOURS], Outcome> {
    let bad = || data_failure(None, "played colours observation");
    let words = wram
        .get(at..)
        .and_then(|bank| bank.as_chunks::<2>().0.get(..world_doors::COLOURS))
        .ok_or_else(bad)?;
    let mut colours = [types::Colour::BLACK; world_doors::COLOURS];
    for (colour, word) in colours.iter_mut().zip(words) {
        *colour = types::Colour::new(u16::from_le_bytes(*word)).ok_or_else(bad)?;
    }
    Ok(colours)
}

/// The colours, compared only at a cut whose frame of work finished: a lag cut finds a fade's
/// step part-done.
pub(super) fn observations(palette: &Palette) -> [Observation; 2] {
    let bytes = |colours: &[types::Colour]| {
        colours
            .iter()
            .flat_map(|colour| colour.bits().to_le_bytes())
            .collect()
    };
    [
        Observation {
            field: "doors.colours.screen".into(),
            bytes: bytes(&palette.screen),
        },
        Observation {
            field: "doors.colours.goal".into(),
            bytes: bytes(&palette.goal),
        },
    ]
}
