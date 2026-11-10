//! What the model remembers from frame to frame (spec §1.2): the cells, the places and the
//! screens, and the two entry points that change them: the room's start and the frame's step.

use std::sync::Arc;

use clean_types::{Cell, Motion, Point, ScreenKind, Set};

use crate::cells::{Answer, CellKind, Grid, Inside, Mark, ReactingBody, Tile, Tilemap};
use crate::inputs::{Meeting, OutsideTheRoom, Placement, Request, Run};
use crate::places::{Place, Places, Running, ScrollReaction, Touched};
use crate::report::Report;
use crate::rules::{HoldUntilRun, Rules};

/// The blocks' state: everything a later frame needs, and nothing else. A caller builds each part
/// at any value the spec allows and reads each part back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The room's cells: their kinds, marks and looks.
    pub tiles: Tilemap,
    /// The places, each free or with what it holds.
    pub places: Places,
    /// The room's screens, which the camera reads.
    pub screens: Grid<ScreenKind>,
}

impl State {
    /// `RoomStart`: when a room starts, each placement takes the highest free place in the room's
    /// order, as any start does (`Places`), and with every place taken it doesn't start and its
    /// cell is left as it was. A scroll cell's reaction keeps its place, and its cell becomes
    /// reacting air marked `Scroll`, keeping its look; a stand-in's cell becomes the stand-in,
    /// marked with its reach, and its place is free again at once; another part's holds its place.
    /// A cell marked anew holds no place when she is inside it, whatever it held before.
    ///
    /// # Errors
    /// `OutsideTheRoom`, at the first placement whose cell is past the room's edge, which the
    /// room's loading rules out (R2): the room's start stops there, that placement taking no
    /// place and the placements before it placed.
    pub fn start_room(
        &mut self,
        placements: impl IntoIterator<Item = Placement>,
    ) -> Result<(), OutsideTheRoom> {
        for placement in placements {
            let (cell, kind, mark, occupant) = match placement {
                Placement::OtherPart => {
                    self.places.take(Place::OtherPart);
                    continue;
                }
                Placement::Scroll { cell, changes } => (
                    cell,
                    CellKind::Reacting(ReactingBody::Air),
                    Mark::Scroll,
                    Place::Scroll(ScrollReaction {
                        cell,
                        changes,
                        touched: Touched::No,
                    }),
                ),
                // Its place is free again at once.
                Placement::StandIn { cell, way, reach } => (
                    cell,
                    CellKind::StandIn(way),
                    Mark::Reach(reach),
                    Place::Free,
                ),
            };
            let tile = entering(&mut self.tiles, cell)?;
            if self.places.take(occupant) {
                tile.kind = kind;
                tile.mark = mark;
                tile.inside = Inside::HoldsNoPlace;
            }
        }
        Ok(())
    }

    /// One frame (spec §7): the requests answered one at a time in the frame's order, each meeting
    /// the cells as the earlier ones left them; then the run, unless `run` says it is paused.
    /// `moving` is how she moves this frame. Leaves the state as the frame leaves it, its screens
    /// the camera's to read, and returns what the frame reports.
    ///
    /// # Errors
    /// `OutsideTheRoom`, at the first request whose cell is past the room's edge, which the spec
    /// makes the caller's to rule out (§1.3, §8): the frame stops there, the requests before it
    /// answered and the run not run.
    pub fn step(
        &mut self,
        rules: &Rules,
        requests: &[Request],
        moving: Set<Motion>,
        run: Run,
    ) -> Result<Report, OutsideTheRoom> {
        let mut report = Report::default();
        for &request in requests {
            match request {
                Request::Meets { cell, meeting } => {
                    let answer = self.meet(rules, cell, meeting, moving)?;
                    if let Meeting::Touch(_) = meeting {
                        report.answers.push(answer);
                    }
                }
                Request::OtherPart => {
                    self.places.take(Place::OtherPart);
                }
            }
        }
        if run == Run::Runs {
            self.places
                .run(&mut self.tiles, &mut self.screens, &mut report);
        }
        Ok(report)
    }

    /// Spec §5, `AnswersToHerMotion`: a request of `cell`, met as `meeting`, as she moves; returns
    /// how she meets the cell after it, which the report keeps for a touch. A cell that doesn't
    /// react meets her by its body. Of a reacting cell, in this order:
    /// - `BlocksMarkedNothing`: she inside reacting air whose mark holds a place when she is
    ///   inside it, whatever else it says, holds one until the run at each point;
    /// - `ScrollCells`: a scroll cell is touched by any touch and by her centre inside it;
    /// - a touch of a cell marked `Other` is its part's to answer;
    /// - a reaction's cell is met by its family's row, whose act starts its reaction (`Places`):
    ///   it takes the highest free place, and then the cell changes at once; with every place
    ///   taken it doesn't start, and the cell is left as it was;
    /// - a cell marked `Nothing`, or one marked `Other` and met as crumble blocks, is met by the
    ///   row for the cells marked `Nothing` of the family it is met as (`Mark::met_as`), whose act
    ///   holds a place until the run;
    /// - any other request meets it by its body.
    fn meet(
        &mut self,
        rules: &Rules,
        cell: Cell,
        meeting: Meeting,
        moving: Set<Motion>,
    ) -> Result<Answer, OutsideTheRoom> {
        let Self { tiles, places, .. } = self;
        let Tile {
            kind,
            mark,
            look,
            inside,
        } = entering(tiles, cell)?;
        let CellKind::Reacting(body) = *kind else {
            return Ok(kind.answer());
        };
        let full = places.full();
        Ok(match (meeting, &*mark) {
            (Meeting::Inside(_), _)
                if body == ReactingBody::Air && *inside == Inside::HoldsPlace =>
            {
                places.hold_until_run();
                kind.answer()
            }
            (Meeting::Inside(Point::Centre) | Meeting::Touch(_), Mark::Scroll) => {
                places.touch_scroll(cell);
                kind.answer()
            }
            (Meeting::Touch(_), Mark::Other(_)) => Answer::Other,
            (_, Mark::Reaction(reaction)) => {
                let row = &rules.families[reaction.family];
                row.answer(meeting, moving, full, body, |start| {
                    let running = Running::started(Arc::clone(reaction), start, cell, body);
                    if places.take(Place::Running(running)) {
                        start.change.apply(kind, look, body, reaction.whole_look);
                    }
                    kind.answer()
                })
            }
            (_, other) => other.met_as().map_or_else(
                || kind.answer(),
                |met_as| {
                    let row = &rules.marked_nothing[met_as];
                    row.answer(meeting, moving, full, body, |HoldUntilRun| {
                        places.hold_until_run();
                        Answer::Air
                    })
                },
            ),
        })
    }
}

/// The cell a request or a placement names, as it enters the model: the one place its cell is
/// checked against the room.
fn entering(tiles: &mut Tilemap, cell: Cell) -> Result<&mut Tile, OutsideTheRoom> {
    tiles.get_mut(cell).ok_or(OutsideTheRoom(cell))
}
