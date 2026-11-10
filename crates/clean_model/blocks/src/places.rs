//! The places reactions run in (`Places`), and the run of what they hold (`TheScriptsRun`): at
//! most 40 run at once, each one started takes the highest free place, and every frame that isn't
//! paused each place from the highest down has its turn.

use std::sync::Arc;

use clean_types::{Cell, ScreenKind, Script};

use crate::cells::{Grid, ReactingBody, Tile, Tilemap, offset};
use crate::inputs::ScreenChange;
use crate::reactions::{Frames, PatternCell, Reaction, Step};
use crate::report::Report;
use crate::rules::Start;

/// The room's cells as the run writes them, travelling with the frame's report, which records
/// each cell written (`WhatTheScreenRedraws`) and each sound asked for (`Sounds`).
struct Room<'a> {
    /// The room's cells.
    tiles: &'a mut Tilemap,
    /// The frame's report.
    report: &'a mut Report,
}

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
    pub reaction: Arc<Reaction>,
    /// The script it runs.
    pub script: Script,
    /// What it grows back as (`Regrowing`): what its cell was when it started, kept for as long
    /// as it runs, for a reaction whose script has a `Regrow` step; none for a script without one.
    pub grows_back_as: Option<ReactingBody>,
    /// Its cell.
    pub cell: Cell,
    /// The index of its next step in its script, from 0.
    pub next: usize,
    /// The frames it still waits.
    pub wait: Frames,
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

/// Whether a running reaction goes on after its turn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Progress {
    /// It waits for a later frame.
    Waits,
    /// It ended: its place is free.
    Ends,
}

impl Default for Places {
    /// Every place free.
    fn default() -> Self {
        Self(core::array::from_fn(|_| Place::Free))
    }
}

impl Places {
    /// `Places`: `occupant` takes the highest-numbered free place; with every place taken it takes
    /// none. Returns whether it took one.
    pub(crate) fn take(&mut self, occupant: Place) -> bool {
        let highest_free = self.0.iter_mut().rev().find(|place| **place == Place::Free);
        highest_free.map(|place| *place = occupant).is_some()
    }

    /// Whether every place is taken.
    pub(crate) fn full(&self) -> bool {
        !self.0.contains(&Place::Free)
    }

    /// `BlocksMarkedNothing`, `ScrollCells`: a request takes the highest free place until the next
    /// run that isn't paused; with every place taken it takes none. Returns whether it took one.
    pub(crate) fn hold_until_run(&mut self) -> bool {
        self.take(Place::HeldUntilRun)
    }

    /// `ScrollCells`: the scroll cell at `cell` touched takes the highest free place until the
    /// run, and its reaction is touched; with every place taken it isn't touched.
    pub(crate) fn touch_scroll(&mut self, cell: Cell) {
        if !self.hold_until_run() {
            return;
        }
        for place in &mut self.0 {
            if let Place::Scroll(scroll) = place
                && scroll.cell == cell
            {
                scroll.touched = Touched::Yes;
            }
        }
    }

    /// `TheScriptsRun`: each place from the highest down: a running reaction counts down and runs
    /// its steps, writing the room's cells and the frame's report; a touched scroll reaction sets
    /// its screens; a hold until the run is freed.
    pub(crate) fn run(
        &mut self,
        tiles: &mut Tilemap,
        screens: &mut Grid<ScreenKind>,
        report: &mut Report,
    ) {
        let mut room = Room { tiles, report };
        for place in self.0.iter_mut().rev() {
            match place {
                Place::Free | Place::OtherPart => {}
                Place::HeldUntilRun => *place = Place::Free,
                Place::Scroll(scroll) => scroll.run(screens),
                Place::Running(running) => {
                    if running.advance(&mut room) == Progress::Ends {
                        *place = Place::Free;
                    }
                }
            }
        }
    }
}

impl Running {
    /// `Places`, `Regrowing`: a reaction started on `cell`, of this body: it waits its script's
    /// first wait before the step it starts from, and keeps the body to grow back as if its script
    /// has a `Regrow` step.
    pub(crate) fn started(
        reaction: Arc<Reaction>,
        start: Start,
        cell: Cell,
        body: ReactingBody,
    ) -> Self {
        Self {
            grows_back_as: reaction.regrows(start.script).then_some(body),
            next: start.from.index(&reaction.scripts[start.script]),
            wait: reaction.first_wait_of(start.script),
            script: start.script,
            reaction,
            cell,
        }
    }

    /// `TheScriptsRun`: its wait drops by one frame; when it runs out its steps run from its next
    /// step, until a step that waits or ends it. A script that runs out of steps ends.
    fn advance(&mut self, room: &mut Room) -> Progress {
        if let Some(left) = self.wait.less_one() {
            self.wait = left;
            return Progress::Waits;
        }
        loop {
            let step = self.reaction.scripts[self.script].get(self.next);
            self.next = self.next.saturating_add(1);
            match step {
                Some(Step::Sound(sound)) => room.report.sounds.push(sound.clone()),
                Some(Step::Show { pattern, frames }) => {
                    room.show(pattern, self.cell);
                    self.wait = *frames;
                    return Progress::Waits;
                }
                Some(Step::Regrow) => {
                    self.regrow(room);
                    self.wait = Frames::ONE;
                    return Progress::Waits;
                }
                Some(Step::End) | None => return Progress::Ends,
            }
        }
    }

    /// `Regrowing`: its cell grows back as the reacting kind it remembers, with its reaction's
    /// whole look. A reaction that remembers none has nothing to grow back, and writes nothing.
    fn regrow(&self, room: &mut Room) {
        if let Some(body) = self.grows_back_as {
            room.write(self.cell, |tile| {
                tile.regrow(body, self.reaction.whole_look);
            });
        }
    }
}

impl Room<'_> {
    /// `WhatTheScreenRedraws`: a step writes the cell at `at` with `write`, and it is reported
    /// written, whether or not its look changed. A cell past the room's edge, which no reaction's
    /// pattern reaches (the room's loading refuses such a room, R2), is neither.
    fn write(&mut self, at: Cell, write: impl FnOnce(&mut Tile)) {
        if let Some(tile) = self.tiles.get_mut(at) {
            write(tile);
            self.report.written.push(at);
        }
    }

    /// `PatternCells`: a show writes each of its pattern's cells, at `origin` plus its offset, as
    /// the pattern says, in the pattern's order.
    fn show(&mut self, pattern: &[PatternCell], origin: Cell) {
        for cell in pattern {
            if let Some(at) = offset(origin, cell.offset) {
                self.write(at, |tile| tile.show(cell));
            }
        }
    }
}

impl ScrollReaction {
    /// `ScrollCells`: in a run, a touched scroll reaction sets each of its screens' kinds in the
    /// order listed, and is no longer touched.
    fn run(&mut self, screens: &mut Grid<ScreenKind>) {
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
