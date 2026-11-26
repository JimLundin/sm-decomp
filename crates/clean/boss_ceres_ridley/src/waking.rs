//! What Ridley's waking reads and counts: his content, the colour sets his eye and his body
//! fade in through, and the waits he counts down as he readies his roar and roars.

use types::Colour;

use crate::getaway::GetawayContent;

/// How many colours of his palette his eye's colour sets cover: colours 12 to 14.
const EYE_COLOURS: usize = 3;
/// How many colours of his palettes his body's colour sets cover: colours 1 to 11.
const BODY_COLOURS: usize = 11;

/// One of his eye's colour sets: the screen shows it at once on colours 12 to 14 of the palette
/// he is drawn in, the doors'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyeColours(pub [Colour; EYE_COLOURS]);

/// One of his body's colour sets: the screen shows it at once on colours 1 to 11 of the palette
/// he is drawn in and of his second palette, which none of his figures in this part uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyColours(pub [Colour; BODY_COLOURS]);

/// A fade's colour sets, in the order it shows them: at least one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FadeSets<T> {
    /// The set it shows first.
    first: T,
    /// The sets after it, in order.
    rest: Vec<T>,
}

impl<T: Copy> FadeSets<T> {
    /// These sets in this order, or none for no set.
    #[must_use]
    pub fn new(sets: Vec<T>) -> Option<Self> {
        let mut sets = sets.into_iter();
        let first = sets.next()?;
        Some(Self {
            first,
            rest: sets.collect(),
        })
    }

    /// How many sets it has.
    #[must_use]
    pub const fn count(&self) -> usize {
        self.rest.len().saturating_add(1)
    }

    /// The set at this place, 0 for the first; none past the last.
    #[must_use]
    pub fn get(&self, place: u16) -> Option<T> {
        place
            .checked_sub(1)
            .map_or(Some(self.first), |after_first| {
                self.rest.get(usize::from(after_first)).copied()
            })
    }

    /// The last set.
    #[must_use]
    pub fn last(&self) -> T {
        self.rest.last().copied().unwrap_or(self.first)
    }
}

/// His content, imported: the eye's steps and the body's colour sets.
///
/// Each of the eye's steps names one of the eye's colour sets; here a step holds the set it
/// names, so the steps are the sets in the order the eye shows them, a set repeated where steps
/// name the same one. The game's eye has 64 steps and his body 16 sets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RidleyContent {
    /// `EyeFadesIn`: the eye's steps, one shown each run.
    pub eye_steps: FadeSets<EyeColours>,
    /// `BodyFadesIn`: the body's colour sets, one shown every `p` runs.
    pub body_sets: FadeSets<BodyColours>,
    /// `GetawayBackdrop`: the two walls and the picture's colour sets.
    pub getaway: GetawayContent,
}

/// A count of runs left, 0 to `LONGEST`: a wait he counts down a run at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemainingTurns<const LONGEST: u8>(u8);

impl<const LONGEST: u8> RemainingTurns<LONGEST> {
    /// The wait at its longest, as it starts.
    pub const FULL: Self = Self(LONGEST);

    /// A wait of this many runs, or none past the longest.
    #[must_use]
    pub const fn new(runs: u8) -> Option<Self> {
        if runs <= LONGEST {
            Some(Self(runs))
        } else {
            None
        }
    }

    /// How many runs it is.
    #[must_use]
    pub const fn runs(self) -> u8 {
        self.0
    }

    /// The wait a run leaves, one less; none at 0, when it is over.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            Some(runs) => Some(Self(runs)),
            None => None,
        }
    }
}

/// `BodyFadesIn`, `ReadyToRoar`: tangible, he waits 4 runs before he roars.
const READY_RUNS: u8 = 4;
/// `ReadyToRoar`, `Roars`: the roar's wait, 252 runs, before he takes off.
const ROAR_RUNS: u8 = 252;

/// His wait while `ReadyToRoar`: 0 to 4 runs.
pub type ReadyWait = RemainingTurns<READY_RUNS>;
/// His wait while `Roaring`: 0 to 252 runs.
pub type RoarWait = RemainingTurns<ROAR_RUNS>;
