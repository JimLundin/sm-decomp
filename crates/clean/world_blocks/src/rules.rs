//! The rules (spec §5, `BlocksMarkedNothing`): for each family, and for each family a cell marked
//! `Nothing` is met as, what her touches and the hits ask of its cells. A family's act is a start
//! of its reaction; a `Nothing` mark's is a place held until the run; both kinds of row answer a
//! request the same way (`Row::answer`), each with its own act. Each row is a row of the
//! rules a caller may adjust (§2, "The rules' rows"); every row names its member, and
//! `Rules::reference` is the game's, keyed from the tables of those rows.

use enum_map::{Enum, EnumMap};
use types::{Family, Heading, HitBy, Look, MetAs, Motion, Picture, Script, Set, Way, by_member};

use crate::cells::{Answer, CellKind, ReactingBody};
use crate::inputs::Meeting;
use crate::reactions::Step;

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
pub struct Row<K, A: 'static> {
    /// The member of the table it is for.
    pub member: K,
    /// What her touches do.
    pub touch: TouchRule<A>,
    /// The hits that act, each with the bodies it acts on; any other hit does nothing.
    pub hits: &'static [HitRule<A>],
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
    Moving(&'static [Motion]),
    /// Every touch.
    Any,
}

/// The hits that act on a row's cells.
pub struct HitRule<A> {
    /// What reaches the cell.
    pub by: &'static [HitBy],
    /// The bodies it acts on.
    pub on: Bodies,
    /// What it does.
    pub act: A,
}

/// The bodies of reacting cells a hit acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bodies {
    /// A block and reacting air alike.
    Either,
    /// A block only, not reacting air (`BombsReveal`, `BlocksMarkedNothing` met as crumble
    /// blocks).
    BlockOnly,
}

/// What a row makes of a request of its cell.
enum Response<A> {
    /// The row's act: a start of the cell's reaction, or a place held until the run.
    Act(A),
    /// `FullPlacesLetHerThrough` (a quirk): with every place taken, a touch of a cell whose
    /// touches ask it to react (a crumble, speed or bomb block, or a cell marked `Nothing` met as
    /// crumble or bomb blocks, by any way, however she moves) lets her through without acting.
    LetsHerThrough,
    /// Nothing reacts: she meets the cell by its body.
    Ignored,
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
    /// The game's rules: the rows the entries of §5 state.
    #[must_use]
    pub fn reference() -> Self {
        Self {
            families: by_member(FAMILY_ROWS, |row| row.member),
            marked_nothing: by_member(MARKED_NOTHING_ROWS, |row| row.member),
        }
    }
}

/// The game's row for each family, one for each.
const FAMILY_ROWS: <Family as Enum>::Array<Row<Family, Start>> = [
    BOMB_BLOCKS,
    CRUMBLE_BLOCKS,
    POWER_BOMB_BLOCKS,
    SHOT_BLOCKS,
    SPEED_BLOCKS,
    SUPER_MISSILE_BLOCKS,
];

/// The game's row for each family a cell marked `Nothing` is met as, one for each.
const MARKED_NOTHING_ROWS: <MetAs as Enum>::Array<Row<MetAs, HoldUntilRun>> =
    [NOTHING_AS_BOMB, NOTHING_AS_CRUMBLE, NOTHING_AS_SHOT];

/// Every hit: any of her weapons or the grapple.
const EVERY_HIT: &[HitBy] = &[
    HitBy::Beam,
    HitBy::Bomb,
    HitBy::Grapple,
    HitBy::Missile,
    HitBy::PowerBomb,
    HitBy::SuperMissile,
];

/// A blast: a bomb's or a power bomb's.
const BLASTS: &[HitBy] = &[HitBy::Bomb, HitBy::PowerBomb];

/// `SpeedBlocks`: dashing or shinesparking.
const AT_SPEED: &[Motion] = &[Motion::Dashing, Motion::Shinesparking];

/// `BombBlocks`: dashing, shinesparking or screw attacking.
const AT_SPEED_OR_SCREWING: &[Motion] = &[
    Motion::Dashing,
    Motion::Shinesparking,
    Motion::ScrewAttacking,
];

/// `BombBlocks`: her touch while dashing, shinesparking or screw attacking bursts it; a bomb's
/// blast breaks it past its sound, a power bomb's from its first step.
const BOMB_BLOCKS: Row<Family, Start> = Row {
    member: Family::Bomb,
    touch: TouchRule::Reacts {
        when: Starter::Moving(AT_SPEED_OR_SCREWING),
        act: BURSTS,
    },
    hits: &[
        HitRule {
            by: &[HitBy::Bomb],
            on: Bodies::Either,
            act: BREAKS_QUIETLY,
        },
        HitRule {
            by: &[HitBy::PowerBomb],
            on: Bodies::Either,
            act: BREAKS,
        },
    ],
};

/// `CrumbleBlocks`: a move down breaks it; `BombsReveal`: a bomb's blast on one that is a block
/// to her reveals it.
const CRUMBLE_BLOCKS: Row<Family, Start> = Row {
    member: Family::Crumble,
    touch: TouchRule::Reacts {
        when: Starter::Way(Way::Move(Heading::Down)),
        act: BREAKS,
    },
    hits: &[HitRule {
        by: &[HitBy::Bomb],
        on: Bodies::BlockOnly,
        act: REVEALS,
    }],
};

/// `PowerBombBlocks`: a power bomb's blast breaks it, a bomb's reveals it.
const POWER_BOMB_BLOCKS: Row<Family, Start> = Row {
    member: Family::PowerBomb,
    touch: TouchRule::ByBody,
    hits: &[
        HitRule {
            by: &[HitBy::PowerBomb],
            on: Bodies::Either,
            act: BREAKS,
        },
        HitRule {
            by: &[HitBy::Bomb],
            on: Bodies::Either,
            act: REVEALS,
        },
    ],
};

/// `ShotBlocks`: any weapon or the grapple breaks it.
const SHOT_BLOCKS: Row<Family, Start> = Row {
    member: Family::Shot,
    touch: TouchRule::ByBody,
    hits: &[HitRule {
        by: EVERY_HIT,
        on: Bodies::Either,
        act: BREAKS,
    }],
};

/// `SpeedBlocks`: her touch while dashing or shinesparking shatters it; `BombsReveal`: a bomb's
/// blast on one that is a block to her reveals it.
const SPEED_BLOCKS: Row<Family, Start> = Row {
    member: Family::Speed,
    touch: TouchRule::Reacts {
        when: Starter::Moving(AT_SPEED),
        act: SHATTERS,
    },
    hits: &[HitRule {
        by: &[HitBy::Bomb],
        on: Bodies::BlockOnly,
        act: REVEALS,
    }],
};

/// `SuperMissileBlocks`: a super missile breaks it, a bomb's blast reveals it.
const SUPER_MISSILE_BLOCKS: Row<Family, Start> = Row {
    member: Family::SuperMissile,
    touch: TouchRule::ByBody,
    hits: &[
        HitRule {
            by: &[HitBy::SuperMissile],
            on: Bodies::Either,
            act: BREAKS,
        },
        HitRule {
            by: &[HitBy::Bomb],
            on: Bodies::Either,
            act: REVEALS,
        },
    ],
};

/// `BlocksMarkedNothing`, met as bomb blocks: every touch and every hit holds a place.
const NOTHING_AS_BOMB: Row<MetAs, HoldUntilRun> = Row {
    member: MetAs::Bomb,
    touch: TouchRule::Reacts {
        when: Starter::Any,
        act: HoldUntilRun,
    },
    hits: &[HitRule {
        by: EVERY_HIT,
        on: Bodies::Either,
        act: HoldUntilRun,
    }],
};

/// `BlocksMarkedNothing`, met as crumble blocks: every touch holds a place; a blast holds one on
/// a block, and nothing on reacting air.
const NOTHING_AS_CRUMBLE: Row<MetAs, HoldUntilRun> = Row {
    member: MetAs::Crumble,
    touch: TouchRule::Reacts {
        when: Starter::Any,
        act: HoldUntilRun,
    },
    hits: &[HitRule {
        by: BLASTS,
        on: Bodies::BlockOnly,
        act: HoldUntilRun,
    }],
};

/// `BlocksMarkedNothing`, met as shot blocks: she meets it by its body; every hit holds a place.
const NOTHING_AS_SHOT: Row<MetAs, HoldUntilRun> = Row {
    member: MetAs::Shot,
    touch: TouchRule::ByBody,
    hits: &[HitRule {
        by: EVERY_HIT,
        on: Bodies::Either,
        act: HoldUntilRun,
    }],
};

impl<K, A: Copy> Row<K, A> {
    /// `AnswersToHerMotion`, `FullPlacesLetHerThrough`: the row's answer to a request of one of
    /// its cells, whose body is `body`, as she moves, with every place taken or not. A request
    /// that asks for the row's act has it done by `act`, which answers as she meets the cell after
    /// it; with every place taken, a touch that asks the cell to react lets her through; any other
    /// request meets the cell by its body. Her being inside a cell asks nothing of its row.
    pub(crate) fn answer(
        &self,
        meeting: Meeting,
        moving: Set<Motion>,
        places_full: bool,
        body: ReactingBody,
        act: impl FnOnce(A) -> Answer,
    ) -> Answer {
        let response = match meeting {
            Meeting::Touch(way) => self.touch.respond(way, moving, places_full),
            Meeting::Hit(by) => self.hit(by, body).map_or(Response::Ignored, Response::Act),
            Meeting::Inside(_) => Response::Ignored,
        };
        match response {
            Response::Act(done) => act(done),
            Response::LetsHerThrough => Answer::Air,
            Response::Ignored => CellKind::Reacting(body).answer(),
        }
    }

    /// The act of a hit by `by` on a cell of this body, if one of the row's hits acts on it.
    fn hit(&self, by: HitBy, body: ReactingBody) -> Option<A> {
        self.hits
            .iter()
            .find(|hit| hit.by.contains(&by) && hit.on.includes(body))
            .map(|hit| hit.act)
    }
}

impl<A: Copy> TouchRule<A> {
    /// What a touch this way makes of the cell, as she moves, with every place taken or not.
    fn respond(&self, way: Way, moving: Set<Motion>, places_full: bool) -> Response<A> {
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
            Self::Moving(motions) => motions.iter().any(|&motion| moving[motion]),
            Self::Any => true,
        }
    }
}

impl Bodies {
    /// Whether these bodies include `body`, so that a hit acts on a cell of it.
    #[must_use]
    pub const fn includes(self, body: ReactingBody) -> bool {
        matches!(
            (self, body),
            (Self::Either, _) | (Self::BlockOnly, ReactingBody::Block)
        )
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
    /// The start's change to its cell's kind and look, the cell's body being `body`: it takes
    /// its reaction's whole look if it has one, and keeps its own otherwise.
    pub(crate) const fn apply(
        self,
        kind: &mut CellKind,
        look: &mut Look,
        body: ReactingBody,
        whole_look: Option<Picture>,
    ) {
        *kind = match self {
            Self::StopsReacting => body.plain(),
            Self::BecomesAir => CellKind::Air,
            Self::LeftAsItIs => return,
        };
        if let Some(picture) = whole_look {
            *look = Look::unflipped(picture);
        }
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use super::*;

    /// Each member of `K` is named by exactly one of `rows`; a failure names the member.
    fn names_every_member_once<K: Enum + Debug + PartialEq, A>(table: &str, rows: &[Row<K, A>]) {
        for (member, ()) in EnumMap::<K, ()>::from_fn(|_| ()) {
            let count = rows.iter().filter(|row| row.member == member).count();
            assert_eq!(count, 1, "{table}: {member:?} is named by {count} rows");
        }
    }

    #[test]
    fn each_rule_table_names_every_member_of_its_enum_once() {
        names_every_member_once("the families' rows", &FAMILY_ROWS);
        names_every_member_once("the rows of the cells marked Nothing", &MARKED_NOTHING_ROWS);
    }
}
