//! The rules (spec §5, `BlocksMarkedNothing`): for each family, and for each family a cell marked
//! `Nothing` is met as, what her touches and the hits ask of its cells. A family's act is a start
//! of its reaction; a `Nothing` mark's is a place held until the run. `Rules::reference` is the
//! game's.

use clean_types::{Family, Heading, HitBy, Look, Members, MetAs, Motion, Script, Set, Way};
use clean_types::{by_member, set_of};
use enum_map::EnumMap;

use crate::cells::{CellKind, ReactingBody, Tile};
use crate::reactions::{Reaction, Step};

/// The rules the model reads.
pub struct Rules {
    /// What a reaction of each family does, by its family.
    pub families: EnumMap<Family, Row<Family, Start>>,
    /// What a cell marked `Nothing` does (or one marked `Other` that a blast meets as crumble
    /// blocks), by the family whose blocks it is met as.
    pub marked_nothing: EnumMap<MetAs, Row<MetAs, HoldUntilRun>>,
}

/// One row of the rules: what her touches and the hits ask of the cells it is for, each answered
/// by the row's act, `A`.
pub struct Row<K, A> {
    /// The member of the table it is for.
    pub member: K,
    /// What her touches do.
    pub touch: TouchRule<A>,
    /// The hits that act, each with the bodies it acts on; any other hit does nothing.
    pub hits: Vec<HitRule<A>>,
}

/// What her touches do to a row's cells.
pub enum TouchRule<A> {
    /// Her touches don't ask it to react: she meets it by its body.
    ByBody,
    /// Her touches ask it to react, so with every place taken it lets her through
    /// (`FullPlacesLetHerThrough`); a touch `when` matches acts with `act`.
    Reacts {
        /// Which touches act.
        when: Starter,
        /// What such a touch does.
        act: A,
    },
}

/// Which touches act on a cell whose touches ask it to react.
pub enum Starter {
    /// A touch that reaches it this way, however she moves.
    Way(Way),
    /// A touch by any way while she moves in one of these ways.
    Moving(Set<Motion>),
    /// Every touch.
    Any,
}

/// The hits that act on a row's cells.
pub struct HitRule<A> {
    /// What reaches the cell.
    pub by: Set<HitBy>,
    /// The bodies it acts on: a block, reacting air or either.
    pub on: Set<ReactingBody>,
    /// What it does.
    pub act: A,
}

/// What a row makes of a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response<A> {
    /// The row's act: a start of the cell's reaction, or a place held until the run.
    Act(A),
    /// `FullPlacesLetHerThrough` (a quirk): with every place taken, a touch of a cell whose
    /// touches ask it to react (a crumble, speed or bomb block, or a cell marked `Nothing` met as
    /// crumble or bomb blocks, by any way, however she moves) lets her through without acting.
    LetsHerThrough,
    /// Nothing reacts.
    Ignored,
}

/// What meets a reacting cell: her touch, or a hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meets {
    /// Her move or look reaches it this way.
    Touch(Way),
    /// This reaches it.
    Hit(HitBy),
}

/// `BlocksMarkedNothing`: a cell marked `Nothing` reacts to nothing, but a request that would
/// ask a block of its family to react takes the highest free place until this frame's run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoldUntilRun;

/// How a block's start begins its reaction and changes its cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Start {
    /// The script that runs.
    pub script: Script,
    /// The step it runs from.
    pub from: FirstStep,
    /// How the cell changes at once.
    pub change: Change,
}

/// The step a start runs its script from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstStep {
    /// Its first step.
    First,
    /// Past its leading `Sound` step, if it has one (`BombBlocks`).
    PastLeadingSound,
}

/// How a start changes its cell at once (spec §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// It stops reacting and keeps its body: a block becomes `Solid`, reacting air `Air`; with
    /// the whole look if its reaction has one, and its own look otherwise.
    StopsReacting,
    /// It becomes `Air`, with the whole look.
    BecomesAir,
    /// It is left as it is until the script's show.
    LeftAsItIs,
}

/// `CrumbleBlocks`, `ShotBlocks`, `SuperMissileBlocks`, `PowerBombBlocks`, `BombBlocks` (a power
/// bomb's blast reaching one): the `broken` script from its first step; the cell stops reacting.
const BREAKS: Start = Start {
    script: Script::Broken,
    from: FirstStep::First,
    change: Change::StopsReacting,
};

/// `BombBlocks`: a bomb's blast starts the `broken` script past its leading sound.
const BREAKS_QUIETLY: Start = Start {
    script: Script::Broken,
    from: FirstStep::PastLeadingSound,
    change: Change::StopsReacting,
};

/// `SpeedBlocks`: the `broken` script; the cell is air at once.
const SHATTERS: Start = Start {
    script: Script::Broken,
    from: FirstStep::First,
    change: Change::BecomesAir,
};

/// `BombBlocks`: her touch starts the `touched` script; the cell is air at once.
const BURSTS: Start = Start {
    script: Script::Touched,
    from: FirstStep::First,
    change: Change::BecomesAir,
};

/// `BombsReveal`, `SuperMissileBlocks`, `PowerBombBlocks`: the `revealed` script, the cell left
/// as it is.
const REVEALS: Start = Start {
    script: Script::Revealed,
    from: FirstStep::First,
    change: Change::LeftAsItIs,
};

impl Rules {
    /// The game's rules.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            families: by_member(
                [
                    bomb_blocks(),
                    crumble_blocks(),
                    power_bomb_blocks(),
                    shot_blocks(),
                    speed_blocks(),
                    super_missile_blocks(),
                ],
                |row| row.member,
            ),
            marked_nothing: by_member(
                [nothing_as_bomb(), nothing_as_crumble(), nothing_as_shot()],
                |row| row.member,
            ),
        }
    }
}

/// Every hit: any of her weapons or the grapple.
const EVERY_HIT: &[HitBy] = &[
    HitBy::Beam,
    HitBy::Bomb,
    HitBy::Grapple,
    HitBy::Missile,
    HitBy::PowerBomb,
    HitBy::SuperMissile,
];

impl<A> HitRule<A> {
    /// The hits `by` act on a block and on reacting air alike.
    fn on_either(by: &[HitBy], act: A) -> Self {
        Self {
            by: set_of(by.iter().copied()),
            on: set_of([ReactingBody::Air, ReactingBody::Block]),
            act,
        }
    }

    /// The hits `by` act on a block only, not on reacting air.
    fn on_block(by: &[HitBy], act: A) -> Self {
        Self {
            by: set_of(by.iter().copied()),
            on: set_of([ReactingBody::Block]),
            act,
        }
    }
}

/// `BombBlocks`: her touch while dashing, shinesparking or screw attacking bursts it; a bomb's
/// blast breaks it past its sound, a power bomb's from its first step.
fn bomb_blocks() -> Row<Family, Start> {
    Row {
        member: Family::Bomb,
        touch: TouchRule::Reacts {
            when: Starter::Moving(set_of([
                Motion::Dashing,
                Motion::Shinesparking,
                Motion::ScrewAttacking,
            ])),
            act: BURSTS,
        },
        hits: vec![
            HitRule::on_either(&[HitBy::Bomb], BREAKS_QUIETLY),
            HitRule::on_either(&[HitBy::PowerBomb], BREAKS),
        ],
    }
}

/// `CrumbleBlocks`: a move down breaks it; `BombsReveal`: a bomb's blast on one that is a block
/// to her reveals it.
fn crumble_blocks() -> Row<Family, Start> {
    Row {
        member: Family::Crumble,
        touch: TouchRule::Reacts {
            when: Starter::Way(Way::Move(Heading::Down)),
            act: BREAKS,
        },
        hits: vec![HitRule::on_block(&[HitBy::Bomb], REVEALS)],
    }
}

/// `PowerBombBlocks`: a power bomb's blast breaks it, a bomb's reveals it.
fn power_bomb_blocks() -> Row<Family, Start> {
    Row {
        member: Family::PowerBomb,
        touch: TouchRule::ByBody,
        hits: vec![
            HitRule::on_either(&[HitBy::PowerBomb], BREAKS),
            HitRule::on_either(&[HitBy::Bomb], REVEALS),
        ],
    }
}

/// `ShotBlocks`: any weapon or the grapple breaks it.
fn shot_blocks() -> Row<Family, Start> {
    Row {
        member: Family::Shot,
        touch: TouchRule::ByBody,
        hits: vec![HitRule::on_either(EVERY_HIT, BREAKS)],
    }
}

/// `SpeedBlocks`: her touch while dashing or shinesparking shatters it; `BombsReveal`: a bomb's
/// blast on one that is a block to her reveals it.
fn speed_blocks() -> Row<Family, Start> {
    Row {
        member: Family::Speed,
        touch: TouchRule::Reacts {
            when: Starter::Moving(set_of([Motion::Dashing, Motion::Shinesparking])),
            act: SHATTERS,
        },
        hits: vec![HitRule::on_block(&[HitBy::Bomb], REVEALS)],
    }
}

/// `SuperMissileBlocks`: a super missile breaks it, a bomb's blast reveals it.
fn super_missile_blocks() -> Row<Family, Start> {
    Row {
        member: Family::SuperMissile,
        touch: TouchRule::ByBody,
        hits: vec![
            HitRule::on_either(&[HitBy::SuperMissile], BREAKS),
            HitRule::on_either(&[HitBy::Bomb], REVEALS),
        ],
    }
}

/// `BlocksMarkedNothing`, met as bomb blocks: every touch and every hit holds a place.
fn nothing_as_bomb() -> Row<MetAs, HoldUntilRun> {
    Row {
        member: MetAs::Bomb,
        touch: TouchRule::Reacts {
            when: Starter::Any,
            act: HoldUntilRun,
        },
        hits: vec![HitRule::on_either(EVERY_HIT, HoldUntilRun)],
    }
}

/// Met as crumble blocks: every touch holds a place; a bomb's or a power bomb's blast holds one
/// on a block, and nothing on reacting air.
fn nothing_as_crumble() -> Row<MetAs, HoldUntilRun> {
    Row {
        member: MetAs::Crumble,
        touch: TouchRule::Reacts {
            when: Starter::Any,
            act: HoldUntilRun,
        },
        hits: vec![HitRule::on_block(
            &[HitBy::Bomb, HitBy::PowerBomb],
            HoldUntilRun,
        )],
    }
}

/// Met as shot blocks: she meets it by its body; every hit holds a place.
fn nothing_as_shot() -> Row<MetAs, HoldUntilRun> {
    Row {
        member: MetAs::Shot,
        touch: TouchRule::ByBody,
        hits: vec![HitRule::on_either(EVERY_HIT, HoldUntilRun)],
    }
}

impl<K, A: Copy> Row<K, A> {
    /// What this row makes of `meets` on a cell of this body, as she moves, with every place
    /// taken or not.
    #[must_use]
    pub fn respond(
        &self,
        meets: Meets,
        body: ReactingBody,
        moving: Set<Motion>,
        places_full: bool,
    ) -> Response<A> {
        match meets {
            Meets::Touch(way) => self.touch.respond(way, moving, places_full),
            Meets::Hit(by) => self
                .hits
                .iter()
                .find(|hit| hit.by[by] && hit.on[body])
                .map_or(Response::Ignored, |hit| Response::Act(hit.act)),
        }
    }
}

impl<A: Copy> TouchRule<A> {
    /// What a touch this way makes of the cell, as she moves, with every place taken or not.
    #[must_use]
    pub fn respond(&self, way: Way, moving: Set<Motion>, places_full: bool) -> Response<A> {
        match self {
            Self::Reacts { .. } if places_full => Response::LetsHerThrough,
            Self::Reacts { when, act } if when.starts(way, moving) => Response::Act(*act),
            Self::ByBody | Self::Reacts { .. } => Response::Ignored,
        }
    }
}

impl Starter {
    /// Whether a touch this way, as she moves, acts.
    #[must_use]
    pub fn starts(&self, way: Way, moving: Set<Motion>) -> bool {
        match self {
            Self::Way(starting) => *starting == way,
            Self::Moving(motions) => motions.intersects(&moving),
            Self::Any => true,
        }
    }
}

impl FirstStep {
    /// The index of the step a start runs `steps` from.
    #[must_use]
    pub const fn index(self, steps: &[Step]) -> usize {
        match (self, steps.first()) {
            (Self::PastLeadingSound, Some(Step::Sound(_))) => 1,
            (Self::First | Self::PastLeadingSound, _) => 0,
        }
    }
}

impl Change {
    /// The start's change to its cell, whose body is `body`, with its reaction's whole look if it
    /// has one, the cell's own otherwise.
    pub const fn apply(self, tile: &mut Tile, body: ReactingBody, reaction: &Reaction) {
        tile.kind = match self {
            Self::StopsReacting => body.plain(),
            Self::BecomesAir => CellKind::Air,
            Self::LeftAsItIs => return,
        };
        if let Some(picture) = reaction.whole_look {
            tile.look = Look::unflipped(picture);
        }
    }
}
