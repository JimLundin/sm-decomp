//! The places running reactions occupy (`Places`): at most 40 run at once, and each one started
//! takes the highest free place.

use clean_types::{Cell, Screen, ScreenKind, Script};

use crate::cells::{Grid, Reacting};
use crate::reactions::{Frames, ReactionName};

/// How many places there are: 40, numbered 0 to 39 (`Places`).
pub const PLACE_COUNT: usize = 40;

/// The places, by number: `places.0[n]` is place `n`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Places(pub [Place; PLACE_COUNT]);

/// What a place holds (spec §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// Nothing: a start may take it.
    Free,
    /// A running reaction.
    Running(Running),
    /// A scroll cell's reaction, placed by the room (`ScrollCells`).
    Scroll(ScrollReaction),
    /// Another part's reaction, which its part answers.
    OtherPart,
    /// A hold until the next run that isn't paused: it only takes its place.
    HeldUntilRun,
}

/// A running reaction (spec §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Running {
    /// The imported reaction it runs.
    pub reaction: ReactionName,
    /// The script it runs.
    pub script: Script,
    /// Its cell.
    pub cell: Cell,
    /// The index of its next step in its script, from 0.
    pub next: usize,
    /// The frames it still waits.
    pub wait: Frames,
    /// What it grows back as; none for one with no `Regrow` step still ahead.
    pub grows_back_as: Option<Reacting>,
}

/// A scroll cell's reaction (`ScrollCells`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScrollReaction {
    /// Its cell.
    pub cell: Cell,
    /// The screens it changes, in order.
    pub changes: Vec<ScreenChange>,
    /// Whether a touch waits for the run.
    pub touched: Touched,
}

/// Whether a scroll reaction's touch waits for the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touched {
    /// No touch waits.
    No,
    /// Touched: the next run that isn't paused sets its screens.
    Yes,
}

/// A screen a scroll reaction changes, and the kind it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenChange {
    /// The screen.
    pub screen: Screen,
    /// The kind it takes.
    pub becomes: ScreenKind,
}

impl Default for Places {
    /// Every place free.
    fn default() -> Self {
        Self(core::array::from_fn(|_| Place::Free))
    }
}

impl Places {
    /// `Places`: puts `occupant` in the highest free place; with every place taken it takes none.
    /// Returns whether it took one.
    pub fn take(&mut self, occupant: Place) -> bool {
        let Some(place) = self.0.iter_mut().rev().find(|place| **place == Place::Free) else {
            return false;
        };
        *place = occupant;
        true
    }

    /// Whether every place is taken.
    #[must_use]
    pub fn full(&self) -> bool {
        !self.0.contains(&Place::Free)
    }

    /// The scroll reaction on `cell`, if one holds a place.
    pub fn scroll_at(&mut self, cell: Cell) -> Option<&mut ScrollReaction> {
        self.0.iter_mut().find_map(|place| match place {
            Place::Scroll(scroll) if scroll.cell == cell => Some(scroll),
            Place::Free
            | Place::Running(_)
            | Place::Scroll(_)
            | Place::OtherPart
            | Place::HeldUntilRun => None,
        })
    }
}

impl ScrollReaction {
    /// `ScrollCells`: in a run, a touched scroll reaction sets each of its screens' kinds in the
    /// order listed, and is no longer touched.
    pub fn run(&mut self, screens: &mut Grid<ScreenKind>) {
        if self.touched == Touched::No {
            return;
        }
        for change in &self.changes {
            if let Some(screen) = screens.get_mut(change.screen) {
                *screen = change.becomes;
            }
        }
        self.touched = Touched::No;
    }
}
