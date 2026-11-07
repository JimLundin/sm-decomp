//! The file's play time as a harness domain, the mechanism's first proof: today's
//! [`GameTime::tick`] is the model, the clock's four words the State and the declared fields.

use core::fmt;

use game::state::GameTime;
use oracle_record::FrameRecord;

use super::{Domain, Field, Interval};
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramWord};

/// The game state the main loop runs gameplay in.
const MAIN_GAMEPLAY: u16 = 8;

/// The main loop's game state, as far as the clock cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Main gameplay, where the clock counts.
    Gameplay,
    /// Any other game state, by its code.
    Other(u16),
}

impl From<u16> for Mode {
    fn from(code: u16) -> Self {
        match code {
            MAIN_GAMEPLAY => Self::Gameplay,
            code => Self::Other(code),
        }
    }
}

/// A record the clock can't be read from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreadable(DecodeError);

impl fmt::Display for Unreadable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// The clock domain.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock;

fn word(record: &FrameRecord, word: WramWord) -> Result<u16, Unreadable> {
    word.read(&record.wram).map_err(Unreadable)
}

fn time(record: &FrameRecord) -> Result<GameTime, Unreadable> {
    Ok(GameTime {
        hours: word(record, source::GAME_TIME_HOURS)?,
        minutes: word(record, source::GAME_TIME_MINUTES)?,
        seconds: word(record, source::GAME_TIME_SECONDS)?,
        frames: word(record, source::GAME_TIME_FRAMES)?,
    })
}

impl Domain for Clock {
    const NAME: &'static str = "game time";
    type State = GameTime;
    type Inputs = Mode;
    type World = ();
    type Fields = GameTime;
    type Skip = Unreadable;
    const FIELDS: &'static [Field<GameTime>] = &[
        domain_field!("hours", hours),
        domain_field!("minutes", minutes),
        domain_field!("seconds", seconds),
        domain_field!("frames", frames),
    ];

    fn decode(
        &self,
        from: &FrameRecord,
        _interval: Interval,
    ) -> Result<(GameTime, Mode), Unreadable> {
        Ok((time(from)?, Mode::from(word(from, source::GAME_STATE)?)))
    }

    fn world(&self, _from: &FrameRecord, _next: &FrameRecord) -> Result<(), Unreadable> {
        Ok(())
    }

    fn step(&self, state: &mut GameTime, mode: &Mode, (): &mut ()) {
        match mode {
            Mode::Gameplay => state.tick(),
            Mode::Other(_) => {}
        }
    }

    fn fields(&self, state: &GameTime) -> GameTime {
        *state
    }

    fn expected(&self, next: &FrameRecord) -> Result<GameTime, Unreadable> {
        time(next)
    }
}
