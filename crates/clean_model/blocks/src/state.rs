//! What the model remembers from frame to frame (spec §1.2): the cells, the places and the
//! screens, and the two entry points that change them: the room's start and the frame's step.

use clean_types::{Cell, HitBy, Motion, Point, ScreenKind, Set, Touch};

use crate::cells::{Answer, CellKind, Grid, Inside, Mark, ReactingBody, Tile, Tilemap};
use crate::inputs::{OutsideTheRoom, Placement, Request, Run};
use crate::places::{Place, Places, Room, ScrollReaction, Touched};
use crate::report::Report;
use crate::rules::{HoldUntilRun, Response, Rules};

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
    ///
    /// # Errors
    /// `OutsideTheRoom`, at the first placement whose cell is past the room's edge, which the
    /// room's loading rules out (R2): the room's start stops there, the placements before it
    /// placed.
    pub fn start_room(&mut self, placements: &[Placement]) -> Result<(), OutsideTheRoom> {
        for placement in placements {
            let Some(place) = self.places.highest_free() else {
                continue;
            };
            let (cell, kind, mark) = match placement {
                Placement::OtherPart => {
                    *place = Place::OtherPart;
                    continue;
                }
                Placement::Scroll { cell, changes } => {
                    *place = Place::Scroll(ScrollReaction {
                        cell: *cell,
                        changes: changes.clone(),
                        touched: Touched::No,
                    });
                    (*cell, CellKind::Reacting(ReactingBody::Air), Mark::Scroll)
                }
                Placement::StandIn { cell, way, reach } => {
                    (*cell, CellKind::StandIn(*way), Mark::Reach(*reach))
                }
            };
            let tile = entering(&mut self.tiles, cell)?;
            tile.kind = kind;
            tile.mark = mark;
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
        for request in requests {
            match *request {
                Request::Inside { cell, point } => self.inside(cell, point)?,
                Request::Touch(touch) => report.answers.push(self.touch(rules, touch, moving)?),
                Request::Hit { cell, by } => self.hit(rules, cell, by)?,
                Request::OtherPart => {
                    self.places.take(Place::OtherPart);
                }
            }
        }
        if run == Run::Runs {
            let mut room = Room {
                tiles: &mut self.tiles,
                report: &mut report,
            };
            self.places.run(&mut room, &mut self.screens);
        }
        Ok(report)
    }

    /// `AnswersToHerMotion`: her touch of a cell, answered as the cell meets it. A cell that
    /// doesn't react meets her by its body; a reacting cell's mark gives its row of the rules,
    /// whose response to the touch is acted on; a cell marked `Other` is its part's to answer.
    fn touch(
        &mut self,
        rules: &Rules,
        touch: Touch,
        moving: Set<Motion>,
    ) -> Result<Answer, OutsideTheRoom> {
        let Self { tiles, places, .. } = self;
        let tile = entering(tiles, touch.cell)?;
        let CellKind::Reacting(body) = tile.kind else {
            return Ok(tile.kind.body());
        };
        let full = places.full();
        Ok(match tile.mark.clone() {
            Mark::Reaction(reaction) => {
                let row = &rules.families[reaction.family];
                let response = row.touch.respond(touch.way, moving, full);
                answer(tile, response, |tile, start| {
                    places.start(tile, touch.cell, &reaction, start, body);
                    tile.kind.body()
                })
            }
            Mark::Nothing(met_as) => {
                let row = &rules.marked_nothing[met_as];
                let response = row.touch.respond(touch.way, moving, full);
                answer(tile, response, |_, HoldUntilRun| {
                    places.hold_until_run();
                    Answer::Air
                })
            }
            Mark::Scroll => {
                places.touch_scroll(touch.cell);
                tile.kind.body()
            }
            Mark::Other(_) => Answer::Other,
            Mark::Inert | Mark::Unmarked | Mark::Reach(_) => tile.kind.body(),
        })
    }

    /// A hit of a cell by `by`. A reacting cell's mark gives its row of the rules, and the act of
    /// the row's hit, if one acts on the cell, is done: a start of its reaction, or a place held
    /// until the run for a cell marked `Nothing`, or marked `Other` and met as crumble blocks.
    fn hit(&mut self, rules: &Rules, cell: Cell, by: HitBy) -> Result<(), OutsideTheRoom> {
        let Self { tiles, places, .. } = self;
        let tile = entering(tiles, cell)?;
        let CellKind::Reacting(body) = tile.kind else {
            return Ok(());
        };
        match tile.mark.clone() {
            Mark::Reaction(reaction) => {
                if let Some(start) = rules.families[reaction.family].hit(by, body) {
                    places.start(tile, cell, &reaction, start, body);
                }
            }
            Mark::Nothing(met_as) | Mark::Other(Some(met_as)) => {
                if rules.marked_nothing[met_as].hit(by, body).is_some() {
                    places.hold_until_run();
                }
            }
            Mark::Other(None) | Mark::Scroll | Mark::Inert | Mark::Unmarked | Mark::Reach(_) => {}
        }
        Ok(())
    }

    /// `ScrollCells`, `BlocksMarkedNothing`: a cell she is inside. Reacting air whose mark holds a
    /// place when she is inside it holds one at each point; a scroll cell is touched at her centre
    /// only; a cell marked `Other` is its part's.
    fn inside(&mut self, cell: Cell, point: Point) -> Result<(), OutsideTheRoom> {
        let tile = entering(&mut self.tiles, cell)?;
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

/// The cell a request or a placement names, as it enters the model: the one place its cell is
/// checked against the room.
fn entering(tiles: &mut Tilemap, cell: Cell) -> Result<&mut Tile, OutsideTheRoom> {
    tiles.get_mut(cell).ok_or(OutsideTheRoom(cell))
}

/// `AnswersToHerMotion`, `FullPlacesLetHerThrough`: a row's response to her touch of `tile`,
/// acted on. The row's act is `act`, which answers as she meets the cell after it; with every
/// place taken she passes; a touch that asks nothing meets the cell by its body.
fn answer<A>(
    tile: &mut Tile,
    response: Response<A>,
    act: impl FnOnce(&mut Tile, A) -> Answer,
) -> Answer {
    match response {
        Response::Act(done) => act(tile, done),
        Response::LetsHerThrough => Answer::Air,
        Response::Ignored => tile.kind.body(),
    }
}
