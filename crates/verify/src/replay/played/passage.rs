//! Whether a door passage is under way: as the record's game state says, and compared.

use super::word;
use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::source;

/// The game's states for play, the fade in as a new game begins, which plays as play does, and
/// the three stretches of a door passage.
const PLAY: u16 = 8;
const FADING_IN: u16 = 7;
const TOUCHED: u16 = 9;
const LOADING: u16 = 10;
const ARRIVING: u16 = 11;

/// Whether the record's game state is a door passage's; play is not.
pub(super) fn passing(wram: &[u8]) -> Result<bool, Outcome> {
    match word(wram, source::GAME_STATE)? {
        PLAY | FADING_IN => Ok(false),
        TOUCHED | LOADING | ARRIVING => Ok(true),
        _ => Err(Outcome::Unsupported {
            cut: None,
            dependency: "played game state outside play and door passages".into(),
        }),
    }
}

/// Whether a door passage is under way, compared as one byte.
pub(super) fn observation(passing: bool) -> Observation {
    Observation {
        field: "doors.passing".into(),
        bytes: vec![u8::from(passing)],
    }
}
