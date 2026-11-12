//! Reactions and their scripts: the imported assets (spec §2). What a script shows, for how long
//! and with which sound is game content; the model only runs it.

use enum_map::EnumMap;
use types::{Family, Look, Pair, Picture, Script};

/// The most frames a wait or a show may last: 32,767 (spec §1, §2).
pub const MOST_FRAMES: u16 = 0x7FFF;

/// A count of frames, 1 to 32,767: a reaction's wait, a show's length, a first wait. No frame ends
/// with a wait of 0, so none is held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frames(u16);

impl Frames {
    /// One frame: what a `Regrow` step waits, and a `revealed` script's first wait.
    pub const ONE: Self = Self(1);

    /// This many frames, if the spec allows it (1 to 32,767).
    #[must_use]
    pub const fn new(count: u16) -> Option<Self> {
        if count != 0 && count <= MOST_FRAMES {
            Some(Self(count))
        } else {
            None
        }
    }

    /// How many frames.
    #[must_use]
    pub const fn count(self) -> u16 {
        self.0
    }

    /// `TheScriptsRun`: a frame's countdown: the frames still to wait, or none when the wait has
    /// run out.
    #[must_use]
    pub const fn less_one(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

/// The name of an imported reaction, which a reacting cell's mark gives.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReactionName(pub String);

/// A reaction: what a reacting cell does, as the game's content gives it (spec §2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reaction {
    /// Its name, which the marks that name it give.
    pub name: ReactionName,
    /// Which rules start it: the only part of a reaction the code decides on.
    pub family: Family,
    /// The picture it shows when whole, never flipped, which its start gives the cell and growing
    /// back restores; none for the shot blocks that keep their own look.
    pub whole_look: Option<Picture>,
    /// The frames from a start of its `broken` or `touched` script to its first step.
    pub first_wait: Frames,
    /// Its scripts; one it doesn't have is empty.
    pub scripts: EnumMap<Script, Vec<Step>>,
}

impl Reaction {
    /// The frames from a start of `script` to its first step: a `revealed` script's start always
    /// waits 1, the others the reaction's first wait (spec §2).
    #[must_use]
    pub const fn first_wait_of(&self, script: Script) -> Frames {
        match script {
            Script::Revealed => Frames::ONE,
            Script::Broken | Script::Touched => self.first_wait,
        }
    }

    /// `Regrowing`: whether `script` has a `Regrow` step, so that a reaction running it keeps
    /// what it grows back as for as long as it runs.
    #[must_use]
    pub fn regrows(&self, script: Script) -> bool {
        self.scripts[script].contains(&Step::Regrow)
    }
}

/// A script's step (spec §2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// Asks for a sound and goes straight on to the next step.
    Sound(SoundRequest),
    /// The pattern's cells take their kinds and looks; the reaction waits `frames`.
    Show {
        /// The cells it changes, in order.
        pattern: Vec<PatternCell>,
        /// How long the reaction then waits, 1 to 32,767.
        frames: Frames,
    },
    /// The cell grows back as it was, with the whole look; the reaction waits one frame.
    Regrow,
    /// The reaction ends and its place is free.
    End,
}

/// A sound a script asks for (`Sounds`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundRequest {
    /// The sound, by the importer's name for it.
    pub sound: String,
    /// Its queue limit, a count, which the audio domain reads.
    pub queue_limit: u16,
}

/// One cell of a show's pattern (spec §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatternCell {
    /// Where it is from the reaction's cell; `(0, 0)` is the cell itself.
    pub offset: Pair<i16>,
    /// What the cell becomes.
    pub becomes: Becomes,
    /// The look it takes.
    pub look: Look,
}

/// What a pattern cell becomes (spec §2, `PatternCells`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Becomes {
    /// Air.
    Air,
    /// A reacting block, `Block`, whatever it was (reacting air included), its mark unchanged.
    Block,
    /// A stand-in along its row, reaching as its mark says.
    StandInAcross,
    /// A stand-in along its column, reaching as its mark says.
    StandInDown,
}
