//! The places reactions run in (`Places`), and the run of what they hold (`TheScriptsRun`): at
//! most 40 run at once, each one started takes the highest free place, and every frame that isn't
//! paused each place from the highest down has its turn.

use std::sync::Arc;

use clean_types::{Cell, Screen, ScreenKind, Script};

use crate::cells::{Grid, ReactingBody, Tile, Tilemap, offset};
use crate::reactions::{Frames, PatternCell, Reaction, Step};
use crate::report::Report;
use crate::rules::Start;

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
///
/// Its reaction, its script and what it grows back as are set together, so that a reaction whose
/// script has a `Regrow` step always knows what it grows back as, and one whose script has none
/// never does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Running {
    /// The imported reaction it runs.
    reaction: Arc<Reaction>,
    /// The script it runs.
    script: Script,
    /// What it grows back as, kept for as long as it runs, if its script has a `Regrow` step.
    grows_back_as: Option<ReactingBody>,
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

/// A screen a scroll reaction changes, and the kind it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenChange {
    /// The screen.
    pub screen: Screen,
    /// The kind it takes.
    pub becomes: ScreenKind,
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

    /// `BlocksMarkedNothing`, `ScrollCells`: a request takes the highest free place until the next
    /// run that isn't paused; with every place taken it takes none. Returns whether it took one.
    pub fn hold_until_run(&mut self) -> bool {
        self.take(Place::HeldUntilRun)
    }

    /// `ScrollCells`: a scroll cell touched takes the highest free place until the run, and its
    /// reaction is touched; with every place taken it isn't touched.
    pub fn touch_scroll(&mut self, cell: Cell) {
        if !self.hold_until_run() {
            return;
        }
        let scroll = self.0.iter_mut().find_map(|place| match place {
            Place::Scroll(scroll) if scroll.cell == cell => Some(scroll),
            Place::Free
            | Place::Running(_)
            | Place::Scroll(_)
            | Place::OtherPart
            | Place::HeldUntilRun => None,
        });
        if let Some(scroll) = scroll {
            scroll.touched = Touched::Yes;
        }
    }

    /// `Places`, spec §5: a block's start, on `tile` at `cell`, whose body is `body`. Its reaction
    /// takes the highest free place, waiting its first wait before the step it starts from; then
    /// the cell changes at once. With every place taken it doesn't start, and the cell is left as
    /// it was.
    pub fn start(
        &mut self,
        tile: &mut Tile,
        cell: Cell,
        reaction: &Arc<Reaction>,
        start: Start,
        body: ReactingBody,
    ) {
        let running = Running::started(Arc::clone(reaction), start, cell, body);
        if self.take(Place::Running(running)) {
            start.change.apply(tile, body, reaction);
        }
    }

    /// `TheScriptsRun`: each place from the highest down: a running reaction counts down and runs
    /// its steps, a touched scroll reaction sets its screens, a hold until the run is freed.
    pub fn run(
        &mut self,
        tiles: &mut Tilemap,
        screens: &mut Grid<ScreenKind>,
        report: &mut Report,
    ) {
        for place in self.0.iter_mut().rev() {
            match place {
                Place::Free | Place::OtherPart => {}
                Place::HeldUntilRun => *place = Place::Free,
                Place::Scroll(scroll) => scroll.run(screens),
                Place::Running(running) => {
                    if running.advance(tiles, report) == Progress::Ends {
                        *place = Place::Free;
                    }
                }
            }
        }
    }
}

impl Running {
    /// A running reaction as a caller holds it, if what it grows back as is given exactly when its
    /// script has a `Regrow` step (`Regrowing`).
    #[must_use]
    pub fn new(
        reaction: Arc<Reaction>,
        script: Script,
        grows_back_as: Option<ReactingBody>,
        cell: Cell,
        next: usize,
        wait: Frames,
    ) -> Option<Self> {
        (grows_back_as.is_some() == reaction.regrows(script)).then_some(Self {
            reaction,
            script,
            grows_back_as,
            cell,
            next,
            wait,
        })
    }

    /// `Places`, `Regrowing`: a reaction started on a cell of this body: it waits its script's
    /// first wait before the step it starts from, and keeps the body to grow back as if its script
    /// has a `Regrow` step.
    fn started(reaction: Arc<Reaction>, start: Start, cell: Cell, body: ReactingBody) -> Self {
        Self {
            grows_back_as: reaction.regrows(start.script).then_some(body),
            next: start.from.index(&reaction.scripts[start.script]),
            wait: reaction.first_wait_of(start.script),
            script: start.script,
            reaction,
            cell,
        }
    }

    /// The imported reaction it runs.
    #[must_use]
    pub const fn reaction(&self) -> &Arc<Reaction> {
        &self.reaction
    }

    /// The script it runs.
    #[must_use]
    pub const fn script(&self) -> Script {
        self.script
    }

    /// What it grows back as; none for a reaction whose script has no `Regrow` step.
    #[must_use]
    pub const fn grows_back_as(&self) -> Option<ReactingBody> {
        self.grows_back_as
    }

    /// `TheScriptsRun`: its wait drops by one frame; when it runs out its steps run from its next
    /// step, until a step that waits or ends it. A script that runs out of steps ends.
    fn advance(&mut self, tiles: &mut Tilemap, report: &mut Report) -> Progress {
        if let Some(left) = self.wait.less_one() {
            self.wait = left;
            return Progress::Waits;
        }
        loop {
            let step = self.reaction.scripts[self.script].get(self.next);
            self.next = self.next.saturating_add(1);
            match step {
                Some(Step::Sound(sound)) => report.sounds.push(sound.clone()),
                Some(Step::Show { pattern, frames }) => {
                    show(pattern, self.cell, tiles, report);
                    self.wait = *frames;
                    return Progress::Waits;
                }
                Some(Step::Regrow) => {
                    self.regrow(tiles, report);
                    self.wait = Frames::ONE;
                    return Progress::Waits;
                }
                Some(Step::End) | None => return Progress::Ends,
            }
        }
    }

    /// `Regrowing`: its cell grows back as what it remembers, with its reaction's whole look;
    /// `WhatTheScreenRedraws`: the cell is written.
    fn regrow(&self, tiles: &mut Tilemap, report: &mut Report) {
        if let (Some(body), Some(tile)) = (self.grows_back_as, tiles.get_mut(self.cell)) {
            tile.regrow(body, self.reaction.whole_look);
            report.written.push(self.cell);
        }
    }
}

/// `PatternCells`: each pattern cell, at the reaction's cell plus its offset, is written as the
/// pattern says; `WhatTheScreenRedraws`: in the pattern's order.
fn show(pattern: &[PatternCell], origin: Cell, tiles: &mut Tilemap, report: &mut Report) {
    for cell in pattern {
        let Some(at) = offset(origin, cell.offset) else {
            continue;
        };
        if let Some(tile) = tiles.get_mut(at) {
            tile.show(cell);
            report.written.push(at);
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
