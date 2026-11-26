//! Restorable state of the self-destruct warning. Save it with the drawing owner's current
//! room picture. The clock, room actors, countdown and passage are restored by their owners.

use core::num;

use serde::{Deserialize, Serialize};

/// Spec: `WarningGlow`. Number of ordered heading colour looks.
pub const WARNING_GLOW_LOOKS: usize = 16;
const HEADING_RUNS: u8 = 128;
const SUBTITLE_RUNS: u8 = 32;

/// Spec: `WarningHeading`. Runs left in the heading hold.
pub type HeadingHold = WarningHold<HEADING_RUNS>;
/// Spec: `WarningHeading`. Runs left before the Japanese subtitle appears.
pub type SubtitleHold = WarningHold<SUBTITLE_RUNS>;

/// Spec: `WarningHeading`. A positive hold with a phase-specific maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct WarningHold<const MAX: u8>(num::NonZeroU8);

impl<const MAX: u8> WarningHold<MAX> {
    /// Restore a hold in `1..=MAX`.
    #[must_use]
    pub const fn new(left: u8) -> Option<Self> {
        if left > MAX {
            return None;
        }
        match num::NonZeroU8::new(left) {
            Some(left) => Some(Self(left)),
            None => None,
        }
    }

    /// Ordinary runs left.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0.get()
    }

    /// Spend this run; none when the hold ends.
    #[must_use]
    pub const fn spend(self) -> Option<Self> {
        Self::new(self.get().saturating_sub(1))
    }
}

impl HeadingHold {
    /// Start the full heading hold.
    pub const FULL: Self = Self(num::NonZeroU8::new(HEADING_RUNS).unwrap());
}

impl SubtitleHold {
    /// Start the full subtitle hold.
    pub const FULL: Self = Self(num::NonZeroU8::new(SUBTITLE_RUNS).unwrap());
}

impl<const MAX: u8> TryFrom<u8> for WarningHold<MAX> {
    type Error = &'static str;

    fn try_from(left: u8) -> Result<Self, Self::Error> {
        Self::new(left).ok_or("warning hold outside phase range")
    }
}

impl<const MAX: u8> From<WarningHold<MAX>> for u8 {
    fn from(hold: WarningHold<MAX>) -> Self {
        hold.get()
    }
}

/// Spec: `WarningArt`. Number of art runs already spent. Restore against the same schedule:
/// the count must be less than its length.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarningArtProgress(num::NonZeroUsize);

impl WarningArtProgress {
    /// Restore progress after at least one run but before the final scheduled run.
    #[must_use]
    pub const fn new(loaded: usize, schedule_len: usize) -> Option<Self> {
        if loaded >= schedule_len {
            return None;
        }
        match num::NonZeroUsize::new(loaded) {
            Some(loaded) => Some(Self(loaded)),
            None => None,
        }
    }

    /// The number of completed art runs.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

/// Spec: `WarningTyping`. How many ordinary runs must pass before consuming another cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypingWait {
    /// Consume a cell on the next run.
    Ready,
    /// Wait one run.
    OneRun,
    /// Wait two runs.
    TwoRuns,
}

impl TypingWait {
    /// Spend a waiting run, or return none if ready to consume a cell now.
    #[must_use]
    pub const fn spend(self) -> Option<Self> {
        match self {
            Self::Ready => None,
            Self::OneRun => Some(Self::Ready),
            Self::TwoRuns => Some(Self::OneRun),
        }
    }
}

/// Spec: `WarningTyping`. The stroke beat changes only when a letter is printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeBeat {
    /// Print the next letter without a sound.
    Silent,
    /// Request a stroke with the next printed letter.
    Audible,
}

impl StrokeBeat {
    /// Pick the beat after printing a letter.
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::Silent => Self::Audible,
            Self::Audible => Self::Silent,
        }
    }
}

/// Spec: `WarningTyping`. Restore against the same English text: consumed cells range from
/// zero through the text length, including blanks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarningTypingProgress {
    /// Number of cells already consumed, including blanks.
    pub consumed: usize,
    /// Ordinary runs to wait before consuming another cell.
    pub wait: TypingWait,
    /// Whether to request a stroke on the next letter.
    pub beat: StrokeBeat,
}

impl WarningTypingProgress {
    /// Ready to print the first cell, silently.
    pub const START: Self = Self {
        consumed: 0,
        wait: TypingWait::Ready,
        beat: StrokeBeat::Silent,
    };
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningTyping`, `WarningEnds`.
/// The warning resumes in exactly this phase and at its associated progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningPhase {
    /// The flyby has handed back the room; art loading has not started.
    WarningReady,
    /// The first batch has loaded, with at least one batch remaining.
    SettingArt(WarningArtProgress),
    /// The heading is displayed; English typing has not started.
    Heading(HeadingHold),
    /// English typing is active while the Japanese subtitle is pending.
    SubtitleWait {
        /// Ordinary runs left before displaying the subtitle.
        left: SubtitleHold,
        /// Current English typing progress.
        typing: WarningTypingProgress,
    },
    /// Only English typing remains.
    Typing(WarningTypingProgress),
    /// The warning finished; the next ordinary run activates the escape.
    EscapeReady,
    /// Escape activation has already been published.
    Escape,
}

/// Spec: `WarningGlow`. The current look, from 0 to 15.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct WarningGlow(u8);

impl WarningGlow {
    /// Look zero, the initial look in a fresh room.
    pub const FIRST: Self = Self(0);

    /// Restore the current look by its zero-based index.
    #[must_use]
    pub fn new(look: u8) -> Option<Self> {
        if usize::from(look) < WARNING_GLOW_LOOKS {
            Some(Self(look))
        } else {
            None
        }
    }

    /// The current look's zero-based index.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Advance one look, wrapping back to the first after the last.
    #[must_use]
    pub fn advance(self) -> Self {
        Self::new(self.0.saturating_add(1)).unwrap_or(Self::FIRST)
    }
}

impl TryFrom<u8> for WarningGlow {
    type Error = &'static str;

    fn try_from(look: u8) -> Result<Self, Self::Error> {
        Self::new(look).ok_or("warning glow must be 0 through 15")
    }
}

impl From<WarningGlow> for u8 {
    fn from(look: WarningGlow) -> Self {
        look.get()
    }
}

/// Spec: `WarningArt`, `WarningHeading`, `WarningTyping`, `WarningGlow`, `WarningEnds`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarningState {
    /// The warning's phase and any progress carried by that phase.
    pub phase: WarningPhase,
    /// The last heading glow look selected.
    pub glow: WarningGlow,
}

impl WarningState {
    /// Spec: `WarningArt`. Start when the flyby hands back the room. Art loading will begin on
    /// the following ordinary run.
    #[must_use]
    pub const fn ready() -> Self {
        Self {
            phase: WarningPhase::WarningReady,
            glow: WarningGlow::FIRST,
        }
    }
}
