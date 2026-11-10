//! What the model remembers from frame to frame (spec §1.2): the cells, the places and the
//! screens, and the two entry points that change them: the room's start and the frame's step.

use std::sync::Arc;

use clean_types::{Cell, Motion, Point, ScreenKind, Set};

use crate::cells::{Answer, CellKind, Grid, Inside, Mark, ReactingBody, Tile, Tilemap};
use crate::inputs::{OutsideTheRoom, Placement, Request, Run};
use crate::places::{Place, Places, ScrollReaction, Touched};
use crate::reactions::Reaction;
use crate::report::Report;
use crate::rules::{HoldUntilRun, Meets, Response, Rules, Start};

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
    /// `RoomStart`: when a room starts, each placement takes a place in the room's order, as any
    /// start does (`Places`). A scroll cell's reaction keeps its place and its cell becomes
    /// reacting air marked `Scroll`, keeping its look; a stand-in's cell becomes the stand-in,
    /// marked with its reach, and its place is free again at once, so it needs only that one is
    /// free; another part's holds its place. With every place taken a placement doesn't start, and
    /// its cell is left as it was. A placement past the room's edge is refused, and nothing
    /// changes.
    ///
    /// # Errors
    /// `OutsideTheRoom`, with the first such placement's cell.
    pub fn start_room(&mut self, placements: &[Placement]) -> Result<(), OutsideTheRoom> {
        self.refuse_outside(placements.iter().filter_map(Placement::cell))?;
        for placement in placements {
            let (cell, kind, mark) = match placement {
                Placement::Scroll { cell, changes } => {
                    let reaction = ScrollReaction {
                        cell: *cell,
                        changes: changes.clone(),
                        touched: Touched::No,
                    };
                    if !self.places.take(Place::Scroll(reaction)) {
                        continue;
                    }
                    (cell, CellKind::Reacting(ReactingBody::Air), Mark::Scroll)
                }
                Placement::StandIn { cell, way, reach } => {
                    if self.places.full() {
                        continue;
                    }
                    (cell, CellKind::StandIn(*way), Mark::Reach(*reach))
                }
                Placement::OtherPart => {
                    self.places.take(Place::OtherPart);
                    continue;
                }
            };
            if let Some(tile) = self.tiles.get_mut(*cell) {
                tile.kind = kind;
                tile.mark = mark;
            }
        }
        Ok(())
    }

    /// One frame (spec §7): the requests answered one at a time in the frame's order, each meeting
    /// the cells as the earlier ones left them; then the run, unless `run` says it is paused.
    /// `moving` is how she moves this frame. Leaves the state as the frame leaves it, its screens
    /// the camera's to read, and returns what the frame reports. A request past the room's edge is
    /// refused, and nothing changes.
    ///
    /// # Errors
    /// `OutsideTheRoom`, with the first such request's cell.
    pub fn step(
        &mut self,
        rules: &Rules,
        requests: &[Request],
        moving: Set<Motion>,
        run: Run,
    ) -> Result<Report, OutsideTheRoom> {
        self.refuse_outside(requests.iter().filter_map(Request::cell))?;
        let mut report = Report::default();
        for request in requests {
            match *request {
                Request::Inside { cell, point } => self.inside(cell, point)?,
                Request::Touch(touch) => {
                    let answer = self.meet(rules, touch.cell, Meets::Touch(touch.way), moving)?;
                    report.answers.push(answer);
                }
                Request::Hit { cell, by } => {
                    self.meet(rules, cell, Meets::Hit(by), moving)?;
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

    /// The first of `cells` past the room's edge, refused.
    fn refuse_outside(&self, mut cells: impl Iterator<Item = Cell>) -> Result<(), OutsideTheRoom> {
        cells
            .find(|&cell| self.tiles.get(cell).is_none())
            .map_or(Ok(()), |cell| Err(OutsideTheRoom(cell)))
    }

    /// A touch or a hit of a cell, answered as the cell meets it (`AnswersToHerMotion`; a hit's
    /// answer is no one's). A cell that doesn't react meets her by its body; a reacting cell's
    /// mark gives its row of the rules, and the row's response acts on it.
    fn meet(
        &mut self,
        rules: &Rules,
        cell: Cell,
        meets: Meets,
        moving: Set<Motion>,
    ) -> Result<Answer, OutsideTheRoom> {
        let Self { tiles, places, .. } = self;
        let tile = tiles.get_mut(cell).ok_or(OutsideTheRoom(cell))?;
        let CellKind::Reacting(body) = tile.kind else {
            return Ok(tile.kind.body());
        };
        let full = places.full();
        let lets_her_through = match tile.mark.clone() {
            Mark::Other(_) if matches!(meets, Meets::Touch(_)) => return Ok(Answer::Other),
            Mark::Reaction(reaction) => {
                let response = rules.families[reaction.family].respond(meets, body, moving, full);
                react(places, tile, cell, body, &reaction, response)
            }
            Mark::Nothing(met_as) | Mark::Other(Some(met_as)) => hold(
                places,
                rules.marked_nothing[met_as].respond(meets, body, moving, full),
            ),
            Mark::Scroll => {
                if matches!(meets, Meets::Touch(_)) {
                    places.touch_scroll(cell);
                }
                false
            }
            Mark::Other(None) | Mark::Inert | Mark::Unmarked | Mark::Reach(_) => false,
        };
        Ok(if lets_her_through {
            Answer::Air
        } else {
            tile.kind.body()
        })
    }

    /// `ScrollCells`, `BlocksMarkedNothing`: a cell she is inside. Reacting air whose mark holds a
    /// place when she is inside it holds one at each point; a scroll cell is touched at her centre
    /// only; a cell marked `Other` is its part's.
    fn inside(&mut self, cell: Cell, point: Point) -> Result<(), OutsideTheRoom> {
        let tile = self.tiles.get(cell).ok_or(OutsideTheRoom(cell))?;
        let CellKind::Reacting(body) = tile.kind else {
            return Ok(());
        };
        if matches!(tile.mark, Mark::Other(_)) {
            return Ok(());
        }
        if body == ReactingBody::Air && tile.inside == Inside::HoldsPlace {
            self.places.hold_until_run();
        } else if tile.mark == Mark::Scroll && point == Point::Centre {
            self.places.touch_scroll(cell);
        }
        Ok(())
    }
}

/// A family's response to a request of its block, `tile` at `cell` with this body: a start of
/// its reaction, which happens only with a place free (`Places`). Returns whether she passes the
/// block whatever its body.
fn react(
    places: &mut Places,
    tile: &mut Tile,
    cell: Cell,
    body: ReactingBody,
    reaction: &Arc<Reaction>,
    response: Response<Start>,
) -> bool {
    match response {
        Response::Act(start) => {
            places.start(tile, cell, reaction, start, body);
            false
        }
        Response::LetsHerThrough => true,
        Response::Ignored => false,
    }
}

/// `BlocksMarkedNothing`: a `Nothing` row's response: a hold until the run, if a place is free.
/// A touch that holds a place, or would with one free, lets her through. Returns whether she
/// passes the cell whatever its body.
fn hold(places: &mut Places, response: Response<HoldUntilRun>) -> bool {
    match response {
        Response::Act(HoldUntilRun) => {
            places.hold_until_run();
            true
        }
        Response::LetsHerThrough => true,
        Response::Ignored => false,
    }
}
