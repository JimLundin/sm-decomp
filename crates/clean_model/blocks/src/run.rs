//! The run (`TheScriptsRun`): every frame that isn't paused, after the requests, each place from
//! the highest down has its turn. The one interpreter of the scripts' steps lives here.

use clean_types::Cell;

use crate::cells::{CellKind, Reacting, Tilemap, offset};
use crate::places::{Place, Running};
use crate::reactions::{Becomes, Frames, PatternCell, Reaction, Reactions, Step};
use crate::report::Report;
use crate::state::State;

/// Whether a running reaction goes on after its turn.
#[derive(PartialEq, Eq)]
enum Progress {
    /// It waits for a later frame.
    Waits,
    /// It ended: its place is free.
    Ends,
}

/// `TheScriptsRun`: each place from the highest down: a running reaction counts down and runs
/// its steps, a touched scroll reaction sets its screens, a hold until the run is freed.
pub fn run(reactions: &Reactions, state: &mut State, report: &mut Report) {
    let State {
        tiles,
        places,
        screens,
    } = state;
    for place in places.0.iter_mut().rev() {
        match place {
            Place::Free | Place::OtherPart => {}
            Place::HeldUntilRun => *place = Place::Free,
            Place::Scroll(scroll) => scroll.run(screens),
            Place::Running(running) => {
                if advance(reactions, running, tiles, report) == Progress::Ends {
                    *place = Place::Free;
                }
            }
        }
    }
}

/// `TheScriptsRun`: its wait drops by one frame; when it runs out its steps run from its next
/// step, until a step that waits or ends it.
fn advance(
    reactions: &Reactions,
    running: &mut Running,
    tiles: &mut Tilemap,
    report: &mut Report,
) -> Progress {
    running.wait = running.wait.less_one();
    if running.wait.count() != 0 {
        return Progress::Waits;
    }
    let Some(reaction) = reactions.get(&running.reaction) else {
        return Progress::Ends;
    };
    loop {
        let step = reaction.scripts[running.script].get(running.next);
        running.next = running.next.saturating_add(1);
        match step {
            Some(Step::Sound(sound)) => report.sounds.push(sound.clone()),
            Some(Step::Show { pattern, frames }) => {
                show(pattern, running.cell, tiles, report);
                running.wait = *frames;
                return Progress::Waits;
            }
            Some(Step::Regrow) => {
                regrow(reaction, running, tiles, report);
                running.wait = Frames::ONE;
                return Progress::Waits;
            }
            Some(Step::End) | None => return Progress::Ends,
        }
    }
}

/// `PatternCells`: each pattern cell, at the reaction's cell plus its offset, takes the kind the
/// pattern says and its look, keeping its mark; `WhatTheScreenRedraws`: in the pattern's order.
fn show(pattern: &[PatternCell], origin: Cell, tiles: &mut Tilemap, report: &mut Report) {
    for cell in pattern {
        let Some(at) = offset(origin, cell.offset) else {
            continue;
        };
        let Some(tile) = tiles.get_mut(at) else {
            continue;
        };
        tile.kind = becomes(cell.becomes);
        tile.look = cell.look;
        report.redrawn.push(at);
    }
}

/// `PatternCells`: the kind a pattern cell becomes. `Itself` is its reacting kind as a block.
const fn becomes(becomes: Becomes) -> CellKind {
    match becomes {
        Becomes::Air => CellKind::Air,
        Becomes::Itself => CellKind::Block,
        Becomes::StandInAcross => CellKind::StandInAcross,
        Becomes::StandInDown => CellKind::StandInDown,
    }
}

/// `Regrowing` (a quirk): the cell grows back as the reacting kind it was when the reaction
/// started, with its reaction's whole look unflipped, not the look it had before its start, so a
/// hidden block shows itself once it has grown back. A reaction that remembers nothing grows back
/// as a block.
fn regrow(reaction: &Reaction, running: &mut Running, tiles: &mut Tilemap, report: &mut Report) {
    let Some(tile) = tiles.get_mut(running.cell) else {
        return;
    };
    tile.kind = running
        .grows_back_as
        .take()
        .unwrap_or(Reacting::Block)
        .kind();
    if let Some(look) = reaction.whole_look {
        tile.look = look.unflipped();
    }
    report.redrawn.push(running.cell);
}
