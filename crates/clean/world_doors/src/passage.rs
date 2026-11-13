//! The passage's state: what a caller holds from one frame to the next, saves and restores, and
//! builds at any value the spec allows to start play part-way.

use types::{Fixed, Heading, Pair, Px};

use crate::colours::FadeTime;

/// The doors' state: play, or a passage under way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doors {
    /// No passage: play runs, and a touch of a door to a room starts one (`TheTouch`).
    Play,
    /// A passage under way: play doesn't run.
    Passing(Passage),
}

/// A passage from one room to the next: its phase, the arrival once taken and the slide's frames,
/// and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Passage {
    /// The phase, with what it carries.
    pub phase: Phase,
    /// The doorway's arrival, held from the frame `Squaring` takes it; none before.
    ///
    /// A passage built past `Squaring` without one has nowhere to send the view: what the arrival
    /// would move stays, and its slide counts as ended.
    pub arrival: Option<Arrival>,
    /// The slide frames run so far: 0 until the slide starts, then up to the slide's length.
    pub slide_frames: u16,
}

impl Passage {
    /// `TheTouch`: the passage on the frame after the touch, quieting the room.
    pub const START: Self = Self {
        phase: Phase::Quieting { asked: false },
        arrival: None,
        slide_frames: 0,
    };
}

/// The phase of a passage, with what each carries. Listed alphabetically; `PassageTime` gives
/// their course.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// She is placed in the next room and the view slides in while the room's tiles and then
    /// its contents are fetched.
    Arriving {
        /// Whether she has been placed.
        placed: bool,
        /// What is still being fetched.
        fetching: Fetching,
    },
    /// The dark screen before the view sets off: the frames of work since the view was squared.
    BeforeTheSlide(WorkFrames<LAST_BEFORE_THE_SLIDE>),
    /// The new room's colours coming up, at the fade's step; play goes on after it.
    Brightening(FadeTime),
    /// The room's colours going dark, at the fade's step.
    Darkening(FadeTime),
    /// The frames of work since the slide ended.
    Landing(WorkFrames<LAST_LANDING>),
    /// The dark new room waiting for its track.
    Music {
        /// Whether the track has started.
        started: bool,
    },
    /// The room falling silent.
    Quieting {
        /// Whether the silence has been asked for.
        asked: bool,
    },
    /// In the dark, the doorway's arrival taken and the view moved onto a screen's edge.
    Squaring,
}

/// What `Arriving` is still fetching. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fetching {
    /// The room's contents.
    Contents,
    /// Nothing: the contents are fetched, and the frame of work is held until the slide ends.
    Done,
    /// The tileset's tiles and colours.
    Tiles,
}

/// `BeforeTheSlide(n)`: `n` runs 0 to 2.
pub const LAST_BEFORE_THE_SLIDE: u8 = 2;
/// `Landing(n)`: `n` runs 0 to 1.
pub const LAST_LANDING: u8 = 1;

/// A count of frames of work in a phase, from 0 to `LAST`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkFrames<const LAST: u8>(u8);

impl<const LAST: u8> WorkFrames<LAST> {
    /// The phase's first frame of work, 0.
    pub const FIRST: Self = Self(0);

    /// The count `n`, if it is 0 to `LAST`.
    #[must_use]
    pub const fn new(n: u8) -> Option<Self> {
        if n <= LAST { Some(Self(n)) } else { None }
    }

    /// The count.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The next frame of work's count, or none after the last.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        self.0.checked_add(1).and_then(Self::new)
    }
}

/// A doorway's arrival, from the world's records: where the view and she go in the next room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arrival {
    /// The way she leaves by: the way the view and she move.
    pub way: Heading,
    /// The view's place in the next room, in px: a whole number of screens on each axis, until
    /// `SettingOff` takes it 32 px lower for a way `Up`.
    pub view: Pair<Px>,
    /// Her speed in the slide, `P.S` a frame, if the doorway sets it; with none she moves at the
    /// passage's own (`Carrying`).
    pub speed: Option<Fixed>,
}
