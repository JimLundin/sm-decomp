//! A frame (spec §7): its requests answered one at a time in the frame's order, each meeting the
//! cells as the earlier ones left them; then the run, unless the run is paused.

use clean_types::{Cell, HitBy, Motion, Point, Set, Touch};

use crate::cells::{Answer, CellKind, Inside, Mark, Reacting};
use crate::places::{Place, Places, Running};
use crate::reactions::{ReactionName, Reactions, Step};
use crate::report::Report;
use crate::room_start::Placement;
use crate::rules::{HitRule, NothingRules, NothingTouch, Rules, Start, TouchRule};
use crate::run::run;
use crate::state::State;

/// One of the frame's requests (spec §1.3). A cell is the one a stand-in stands for, already
/// resolved.
#[derive(Clone, Copy)]
pub enum Request {
    /// A cell she is inside, and the point of her that is inside it.
    Inside {
        /// The cell.
        cell: Cell,
        /// Her point inside it.
        point: Point,
    },
    /// A cell her move or look reaches, and its way; answered in the report.
    Touch(Touch),
    /// A cell, and what reaches it.
    Hit {
        /// The cell.
        cell: Cell,
        /// What reaches it.
        by: HitBy,
    },
    /// Another part's start, which takes a place its part holds.
    OtherPart,
}

/// Whether the scripts run this frame (`TheScriptsRun`), as the room's loading and the doors say.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Run {
    /// The run comes after the requests.
    Runs,
    /// Paused: nothing runs, and every wait, hold and touch stays to the next run.
    Paused,
}

/// The blocks model: the rules it follows and the imported reactions it runs.
pub struct Blocks {
    /// The rules by family.
    pub rules: Rules,
    /// The imported reactions, by name.
    pub reactions: Reactions,
}

impl Blocks {
    /// One frame (spec §1.3, §7): the room's placements if it starts this frame (`RoomStart`), the
    /// requests in the frame's order, then the run unless it is paused. `moving` is how she moves
    /// this frame. Leaves `state` as the frame leaves it, its screens the camera's to read, and
    /// returns what the frame reports.
    pub fn step(
        &self,
        state: &mut State,
        placements: &[Placement],
        requests: &[Request],
        moving: Set<Motion>,
        scripts: Run,
    ) -> Report {
        state.start_room(placements);
        let mut report = Report::default();
        for request in requests {
            match *request {
                Request::Inside { cell, point } => inside(state, cell, point),
                Request::Touch(touch) => {
                    let answer = self.touch(state, touch, moving);
                    report.answers.push(answer);
                }
                Request::Hit { cell, by } => self.hit(state, cell, by),
                Request::OtherPart => {
                    state.places.take(Place::OtherPart);
                }
            }
        }
        if scripts == Run::Runs {
            run(&self.reactions, state, &mut report);
        }
        report
    }

    /// `AnswersToHerMotion`: a touch, answered as the cell meets it; a cell that doesn't react, or
    /// a reacting one that says nothing else, meets her by its body as the touch leaves it.
    fn touch(&self, state: &mut State, touch: Touch, moving: Set<Motion>) -> Answer {
        let Some(tile) = state.tiles.get(touch.cell).cloned() else {
            return Answer::Air;
        };
        if tile.kind.reacting().is_none() {
            return tile.kind.body();
        }
        let answer = match &tile.mark {
            Mark::Other(_) => Some(Answer::Other),
            Mark::Scroll => {
                state.touch_scroll(touch.cell);
                None
            }
            Mark::Nothing(met) => touch_nothing(&self.rules.marked_nothing[*met], state),
            Mark::Reaction(name) => self.touch_reaction(state, touch, name, moving),
            Mark::Inert | Mark::Unmarked | Mark::Reach(_) => None,
        };
        answer.unwrap_or_else(|| {
            state
                .tiles
                .get(touch.cell)
                .map_or(Answer::Air, |after| after.kind.body())
        })
    }

    /// Spec §5: a touch of a block with a reaction. A family whose blocks her touches ask to react
    /// lets her through with every place taken, and otherwise starts if the touch is one that
    /// starts it. Returns an answer other than the block's body, if the touch has one.
    fn touch_reaction(
        &self,
        state: &mut State,
        touch: Touch,
        name: &ReactionName,
        moving: Set<Motion>,
    ) -> Option<Answer> {
        let reaction = self.reactions.get(name)?;
        let TouchRule::Asks { starts_on, start } = &self.rules.families[reaction.family].touch
        else {
            return None;
        };
        if let Some(answer) = full_places_let_her_through(&state.places) {
            return Some(answer);
        }
        if starts_on.starts(touch.way, &moving) {
            self.start(state, touch.cell, name, *start);
        }
        None
    }

    /// Spec §5, `BlocksMarkedNothing`: a hit of a reacting cell.
    fn hit(&self, state: &mut State, cell: Cell, by: HitBy) {
        let Some(tile) = state.tiles.get(cell).cloned() else {
            return;
        };
        let Some(body) = tile.kind.reacting() else {
            return;
        };
        match &tile.mark {
            Mark::Reaction(name) => self.hit_reaction(state, cell, name, by, body),
            Mark::Nothing(met) | Mark::Other(Some(met)) => {
                if self.rules.marked_nothing[*met].hit_holds(by, body) {
                    state.hold_until_run();
                }
            }
            Mark::Other(None) | Mark::Scroll | Mark::Inert | Mark::Unmarked | Mark::Reach(_) => {}
        }
    }

    /// Spec §5: a hit of a block with a reaction starts it as its family's chart says.
    fn hit_reaction(
        &self,
        state: &mut State,
        cell: Cell,
        name: &ReactionName,
        by: HitBy,
        body: Reacting,
    ) {
        let Some(reaction) = self.reactions.get(name) else {
            return;
        };
        match self.rules.families[reaction.family].hits[by] {
            HitRule::Ignored => {}
            HitRule::Starts(start) => self.start(state, cell, name, start),
            HitRule::StartsIfBlock(start) => {
                if body == Reacting::Block {
                    self.start(state, cell, name, start);
                }
            }
        }
    }

    /// `Places`, spec §5: a block's start. Its reaction takes the highest free place, waiting its
    /// first wait before the step it starts from, and remembering what the cell was if a `Regrow`
    /// step is ahead; then the cell changes at once. With every place taken it doesn't start, and
    /// the cell is left as it was.
    fn start(&self, state: &mut State, cell: Cell, name: &ReactionName, start: Start) {
        let Some(reaction) = self.reactions.get(name) else {
            return;
        };
        let Some(tile) = state.tiles.get_mut(cell) else {
            return;
        };
        let Some(was) = tile.kind.reacting() else {
            return;
        };
        let steps = &reaction.scripts[start.script];
        let next = start.from.index(steps);
        let regrows = steps
            .get(next..)
            .is_some_and(|ahead| ahead.contains(&Step::Regrow));
        let running = Running {
            reaction: name.clone(),
            script: start.script,
            cell,
            next,
            wait: reaction.first_wait_of(start.script),
            grows_back_as: regrows.then_some(was),
        };
        if state.places.take(Place::Running(running)) {
            start.change.apply(tile, reaction.whole_look);
        }
    }
}

/// `FullPlacesLetHerThrough` (a quirk): with every place taken, a touch of a block whose touches
/// ask it to react (a crumble, speed or bomb block, by any way, at any speed) lets her through
/// without starting it, where it would otherwise stop her. Returns `Air` when it does.
fn full_places_let_her_through(places: &Places) -> Option<Answer> {
    places.full().then_some(Answer::Air)
}

/// `BlocksMarkedNothing`: a touch of a cell marked `Nothing`. Met as crumble or bomb blocks it lets
/// her through and holds a place until the run, if one is free; met as shot blocks she meets it by
/// its body. Returns an answer other than its body, if the touch has one.
fn touch_nothing(rules: &NothingRules, state: &mut State) -> Option<Answer> {
    match rules.touch {
        NothingTouch::HoldsAndPasses => {
            state.hold_until_run();
            Some(Answer::Air)
        }
        NothingTouch::ByBody => None,
    }
}

/// `ScrollCells`, `BlocksMarkedNothing`: a cell she is inside. Reacting air whose mark holds a place
/// when she is inside it holds one at each point; a scroll cell is touched at her centre only; a
/// cell marked `Other` is its part's.
fn inside(state: &mut State, cell: Cell, point: Point) {
    let Some(tile) = state.tiles.get(cell) else {
        return;
    };
    if tile.kind.reacting().is_none() || matches!(tile.mark, Mark::Other(_)) {
        return;
    }
    let holds = tile.kind == CellKind::ReactingAir && tile.inside == Inside::HoldsPlace;
    let scroll = tile.mark == Mark::Scroll;
    if holds {
        state.hold_until_run();
    } else if scroll && point == Point::Centre {
        state.touch_scroll(cell);
    }
}
