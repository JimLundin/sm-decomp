//! The rules by family: what starts a reaction and how its start changes its cell (spec §5), and
//! what a cell marked `Nothing` holds (`BlocksMarkedNothing`). `Rules::REFERENCE` is the game's.

use clean_types::{Family, Heading, HitBy, Look, Members, MetAs, Motion, Script, Set, Table, Way};

use crate::cells::{CellKind, Reacting, Tile};
use crate::reactions::Step;

/// The rules the model reads.
pub struct Rules {
    /// Each family's starts.
    pub families: Table<Family, FamilyRules>,
    /// What a cell marked `Nothing` holds, by the family it is met as.
    pub marked_nothing: Table<MetAs, NothingRules>,
}

/// How a family's blocks meet her touches and her hits.
pub struct FamilyRules {
    /// What her touches do.
    pub touch: TouchRule,
    /// What each hit does.
    pub hits: Table<HitBy, HitRule>,
}

/// What her touches do to a family's blocks.
pub enum TouchRule {
    /// Her touches don't ask it to react: she meets it by its body.
    Ignored,
    /// Her touches ask it to react, so with every place taken it lets her through
    /// (`FullPlacesLetHerThrough`); a touch that `starts_on` says starts it with `start`.
    Asks {
        /// Which touches start it.
        starts_on: Starter,
        /// How such a touch starts it.
        start: Start,
    },
}

/// Which touches start a block.
pub enum Starter {
    /// A touch that reaches it this way, however she moves.
    Way(Way),
    /// A touch by any way while she moves in one of these ways.
    Moving(Set<Motion>),
}

/// What a hit does to a family's blocks.
#[derive(Clone, Copy)]
pub enum HitRule {
    /// Nothing starts.
    Ignored,
    /// It starts this way.
    Starts(Start),
    /// It starts this way if the block is a block to her (`Block`), and not if it is air to her.
    StartsIfBlock(Start),
}

/// How a block's start begins its reaction and changes its cell.
#[derive(Clone, Copy)]
pub struct Start {
    /// The script that runs.
    pub script: Script,
    /// The step it runs from.
    pub from: FirstStep,
    /// How the cell changes at once.
    pub change: Change,
}

/// The step a start runs its script from.
#[derive(Clone, Copy)]
pub enum FirstStep {
    /// Its first step.
    First,
    /// Past its leading `Sound` step, if it has one (`BombBlocks`).
    PastLeadingSound,
}

/// How a start changes its cell at once (spec §5).
#[derive(Clone, Copy)]
pub enum Change {
    /// It becomes `Solid` (`Air` if it was air to her), with the whole look if its reaction has
    /// one and its own look otherwise.
    BecomesSolid,
    /// It becomes `Air`, with the whole look.
    BecomesAir,
    /// It is left as it is until the script's show.
    Unchanged,
}

/// `BlocksMarkedNothing`: what a cell marked `Nothing` (or a cell marked `Other` that a blast
/// meets as crumble blocks) holds, by the family whose blocks it is met as.
pub struct NothingRules {
    /// What her touch of it does.
    pub touch: NothingTouch,
    /// The hits that hold a place when it is a block to her.
    pub block_hits: Set<HitBy>,
    /// The hits that hold a place when it is air to her.
    pub air_hits: Set<HitBy>,
}

/// What her touch of a cell marked `Nothing` does.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NothingTouch {
    /// It lets her through, by any way, and holds a place until the run.
    HoldsAndPasses,
    /// She meets it by its body, and it takes no place.
    ByBody,
}

/// `CrumbleBlocks`, `ShotBlocks`, `SuperMissileBlocks`, `PowerBombBlocks`, `BombBlocks` (a bomb's
/// blast reaching one): the `broken` script from its first step; the cell turns solid.
const BREAKS: Start = Start {
    script: Script::Broken,
    from: FirstStep::First,
    change: Change::BecomesSolid,
};

/// `BombBlocks`: a bomb's blast starts the `broken` script past its leading sound.
const BREAKS_QUIETLY: Start = Start {
    script: Script::Broken,
    from: FirstStep::PastLeadingSound,
    change: Change::BecomesSolid,
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
    change: Change::Unchanged,
};

// The charts below list their entries in their key's order. A table of hits: beam, bomb's blast,
// grapple, missile, power bomb's blast, super missile (`HitBy`). A set of motions: dashing,
// shinesparking, screw attacking (`Motion`).

/// The hit starts nothing.
const NO: HitRule = HitRule::Ignored;
/// The hit breaks the block.
const BREAK: HitRule = HitRule::Starts(BREAKS);
/// The hit reveals the block.
const REVEAL: HitRule = HitRule::Starts(REVEALS);
/// The hit reveals the block if it is a block to her.
const REVEAL_BLOCK: HitRule = HitRule::StartsIfBlock(REVEALS);

/// `SpeedBlocks`: dashing or shinesparking.
const AT_SPEED: Set<Motion> = Table::from_array([true, true, false]);
/// `BombBlocks`: dashing, shinesparking or screw attacking.
const AT_SPEED_OR_SCREW: Set<Motion> = Table::from_array([true, true, true]);

/// Any of her weapons or the grapple.
const EVERY_HIT: Set<HitBy> = Table::from_array([true, true, true, true, true, true]);
/// No hit.
const NO_HIT: Set<HitBy> = Table::from_array([false, false, false, false, false, false]);
/// A bomb's or a power bomb's blast.
const BLASTS: Set<HitBy> = Table::from_array([false, true, false, false, true, false]);

/// `BombBlocks`: her touch at speed or in a screw attack; a bomb's blast past the sound, a power
/// bomb's from the first step.
const BOMB_BLOCKS: FamilyRules = FamilyRules {
    touch: TouchRule::Asks {
        starts_on: Starter::Moving(AT_SPEED_OR_SCREW),
        start: BURSTS,
    },
    hits: Table::from_array([NO, HitRule::Starts(BREAKS_QUIETLY), NO, NO, BREAK, NO]),
};

/// `CrumbleBlocks`: a move down; `BombsReveal`: a bomb's blast on one that is a block to her.
const CRUMBLE_BLOCKS: FamilyRules = FamilyRules {
    touch: TouchRule::Asks {
        starts_on: Starter::Way(Way::Move(Heading::Down)),
        start: BREAKS,
    },
    hits: Table::from_array([NO, REVEAL_BLOCK, NO, NO, NO, NO]),
};

/// `PowerBombBlocks`: a power bomb's blast breaks it, a bomb's reveals it.
const POWER_BOMB_BLOCKS: FamilyRules = FamilyRules {
    touch: TouchRule::Ignored,
    hits: Table::from_array([NO, REVEAL, NO, NO, BREAK, NO]),
};

/// `ShotBlocks`: any weapon or the grapple breaks it.
const SHOT_BLOCKS: FamilyRules = FamilyRules {
    touch: TouchRule::Ignored,
    hits: Table::from_array([BREAK, BREAK, BREAK, BREAK, BREAK, BREAK]),
};

/// `SpeedBlocks`: her touch at speed, by any way; `BombsReveal`: a bomb's blast on one that is a
/// block to her.
const SPEED_BLOCKS: FamilyRules = FamilyRules {
    touch: TouchRule::Asks {
        starts_on: Starter::Moving(AT_SPEED),
        start: SHATTERS,
    },
    hits: Table::from_array([NO, REVEAL_BLOCK, NO, NO, NO, NO]),
};

/// `SuperMissileBlocks`: a super missile breaks it, a bomb's blast reveals it.
const SUPER_MISSILE_BLOCKS: FamilyRules = FamilyRules {
    touch: TouchRule::Ignored,
    hits: Table::from_array([NO, REVEAL, NO, NO, NO, BREAK]),
};

/// `BlocksMarkedNothing`, met as bomb blocks: her touch passes and holds; any hit holds.
const NOTHING_AS_BOMB: NothingRules = NothingRules {
    touch: NothingTouch::HoldsAndPasses,
    block_hits: EVERY_HIT,
    air_hits: EVERY_HIT,
};

/// Met as crumble blocks: her touch passes and holds; a blast holds on a block, nothing on air.
const NOTHING_AS_CRUMBLE: NothingRules = NothingRules {
    touch: NothingTouch::HoldsAndPasses,
    block_hits: BLASTS,
    air_hits: NO_HIT,
};

/// Met as shot blocks: she meets it by its body; any hit holds.
const NOTHING_AS_SHOT: NothingRules = NothingRules {
    touch: NothingTouch::ByBody,
    block_hits: EVERY_HIT,
    air_hits: EVERY_HIT,
};

impl Rules {
    /// The game's rules: each family's row in `Family`'s order, each met-as row in `MetAs`'s.
    pub const REFERENCE: Self = Self {
        families: Table::from_array([
            BOMB_BLOCKS,
            CRUMBLE_BLOCKS,
            POWER_BOMB_BLOCKS,
            SHOT_BLOCKS,
            SPEED_BLOCKS,
            SUPER_MISSILE_BLOCKS,
        ]),
        marked_nothing: Table::from_array([NOTHING_AS_BOMB, NOTHING_AS_CRUMBLE, NOTHING_AS_SHOT]),
    };
}

impl Starter {
    /// Whether a touch this way, as she moves, starts the block.
    #[must_use]
    pub fn starts(&self, way: Way, moving: &Set<Motion>) -> bool {
        match self {
            Self::Way(starting) => *starting == way,
            Self::Moving(motions) => motions.intersects(moving),
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
    /// The start's change to its cell: the whole look if the reaction has one, the cell's own
    /// otherwise.
    pub fn apply(self, tile: &mut Tile, whole_look: Option<Look>) {
        tile.kind = match self {
            Self::BecomesSolid if tile.kind == CellKind::ReactingAir => CellKind::Air,
            Self::BecomesSolid => CellKind::Solid,
            Self::BecomesAir => CellKind::Air,
            Self::Unchanged => return,
        };
        if let Some(look) = whole_look {
            tile.look = look;
        }
    }
}

impl NothingRules {
    /// Whether a hit `by` this holds a place, on a cell with this body.
    #[must_use]
    pub fn hit_holds(&self, by: HitBy, body: Reacting) -> bool {
        match body {
            Reacting::Block => self.block_hits[by],
            Reacting::Air => self.air_hits[by],
        }
    }
}
