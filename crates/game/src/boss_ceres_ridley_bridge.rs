//! The Ceres doors (boss-ceres-ridley, part 1) in the engine: an enemy slot holds the model's
//! door as is, the main enemy routine runs it, and the drawing shows its figure.
//!
//! A door's slot keeps its common words (position, box, properties, frame counter) as every
//! enemy's; its body is the model's [`CeresDoor`]. Each frame the enemies run, the routine runs
//! the doors in slot order through [`run`], which reads what each door reads as it runs, steps
//! the model and writes what the rest of the engine reads back into the slot: whether it is
//! drawn and whether it is solid, which the next frame's selection reads (spec §4: she meets a
//! door as it was when the frame began). The old instruction processing never runs for a door.
//! The drawing shows the door's figure through [`spritemap`], the bridge's table from figures
//! to the spritemaps the content loads, until AF3 imports the figures.
//!
//! The shakes and the opening sound the doors ask for have no consumer yet: the engine holds no
//! screen shake and no sound queue, so they are dropped here and compared in the harness only
//! (docs/LEGACY.md, "The Ceres doors' shakes and sound").

use boss_ceres_ridley::{
    CeresDoor, DoorKind, FigureLook, Inputs, Phase, Quarter, Rules, Stage, Wait,
};
use canonical::{Canonical, DecodeError, Difference, Reader, SchemaHasher};
use core::num::Wrapping;
use data::enemy::program::{CERES_DOOR_SPRITEMAPS, EnemyPrograms, SpritemapId};
use types::{Pair, Px, Side};

use crate::state::room::{CeresStage, EnemyCommon, EnemyExtraProperty, EnemyProperty};
use crate::state::slots::{Flag, FlagSet};

/// A Ceres door's body: the model's door as is (MODEL §1 decision 21).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Door(pub CeresDoor);

/// What the station gives the doors in one frame, threaded from door to door in slot order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Station {
    /// The station's stage.
    pub stage: Stage,
    /// The frame's random number, as the main loop stepped it before the enemies run.
    pub random: u16,
    /// The frames the screen's shake has left as the next door runs: a shake an earlier door
    /// started this frame counts, so at most one door starts one.
    pub shake_frames_left: u16,
}

impl Station {
    /// The station as the frame's first door reads it. The engine holds no screen shake, so the
    /// frame starts with none running (docs/LEGACY.md, "The Ceres doors' shakes and sound").
    #[must_use]
    pub const fn new(stage: CeresStage, random: u16) -> Self {
        Self {
            stage: self::stage(stage),
            random,
            shake_frames_left: 0,
        }
    }
}

/// The model's stage for the engine's.
#[must_use]
pub const fn stage(stage: CeresStage) -> Stage {
    match stage {
        CeresStage::Calm => Stage::Calm,
        CeresStage::Getaway => Stage::Getaway,
        CeresStage::Escape => Stage::Escape,
    }
}

/// One run of a door, on a frame the enemies run it.
///
/// The model steps with `her` position as it stands when the door runs (before her move this
/// frame) and the station; the slot's properties then say what the door's phase shows and
/// blocks, for the drawing and the next frame's selection. A shake the run starts is threaded
/// on to the next door.
pub fn run(common: &mut EnemyCommon, door: &mut Door, her: Pair<Px>, station: &mut Station) {
    let inputs = Inputs {
        her,
        stage: station.stage,
        random: station.random,
        shake_frames_left: station.shake_frames_left,
    };
    let figure_taken = door.0.phase.counted_down().is_none();
    let ran = door.0.run(&inputs, &Rules::default());
    if let Some(shake) = ran.shake {
        station.shake_frames_left = shake.length;
    }
    door.0 = ran.door;
    let phase = ran.door.phase;
    set(
        &mut common.properties,
        EnemyProperty::Invisible,
        !phase.shown(),
    );
    set(
        &mut common.properties,
        EnemyProperty::Intangible,
        !phase.solid(),
    );
    // The original's "new figure" bit, which only extended spritemaps read: a run whose wait
    // ran out takes a figure, any other run doesn't.
    set(
        &mut common.extra_properties,
        EnemyExtraProperty::GraphicUpdated,
        figure_taken,
    );
}

/// Inserts `member` into `flags` when `on`, takes it out otherwise.
fn set<E: Flag>(flags: &mut FlagSet<E>, member: E, on: bool) {
    if on {
        flags.insert(member);
    } else {
        flags.remove(member);
    }
}

/// The spritemap that draws the door's figure, from the content: none while it holds none.
#[must_use]
pub fn spritemap(door: Door, programs: &EnemyPrograms) -> Option<SpritemapId> {
    let figure = door.0.figure()?;
    let look = match figure.look {
        FigureLook::Closed => 0,
        FigureLook::Ajar1 => 1,
        FigureLook::Ajar2 => 2,
        FigureLook::Ajar3 => 3,
        FigureLook::Open => 4,
    };
    let facing = match figure.facing {
        Side::Right => 0,
        Side::Left => CERES_DOOR_SPRITEMAPS.len() / 2,
    };
    let spritemap = CERES_DOOR_SPRITEMAPS.get(facing.wrapping_add(look))?;
    programs.spritemap_id(*spritemap)
}

/// A door's kind and facing from its population parameter: 0 a door facing right, 1 a door
/// facing left, 4 the shaft's door; the station's other parameters are later pieces of B8.
#[must_use]
pub const fn kind(parameter: u16) -> Option<(DoorKind, Side)> {
    match parameter {
        0 => Some((DoorKind::Door, Side::Right)),
        1 => Some((DoorKind::Door, Side::Left)),
        4 => Some((DoorKind::ShaftDoor, Side::Left)),
        _ => None,
    }
}

/// The door a room's start places in a slot with `common`'s parameter and position, `Placed`,
/// with the colour set the doors' palette takes; none for a parameter no door of this piece has.
#[must_use]
pub fn place(common: &EnemyCommon, stage: CeresStage) -> Option<boss_ceres_ridley::Placement> {
    let (kind, facing) = kind(common.parameter_1)?;
    let position = Pair {
        x: Wrapping(common.position.x.pixels),
        y: Wrapping(common.position.y.pixels),
    };
    Some(CeresDoor::place(kind, facing, position, self::stage(stage)))
}

/// A door's canonical bytes: its kind, facing and position, then its phase as a tag, the
/// quarter of an opening or closing and its wait (0 where the phase carries none).
impl Canonical for Door {
    const SCHEMA: u64 = SchemaHasher::new()
        .name("Door")
        .name("kind")
        .schema(u8::SCHEMA)
        .name("facing")
        .schema(u8::SCHEMA)
        .name("position")
        .schema(u16::SCHEMA)
        .schema(u16::SCHEMA)
        .name("phase")
        .schema(u8::SCHEMA)
        .name("quarter")
        .schema(u8::SCHEMA)
        .name("wait")
        .schema(u16::SCHEMA)
        .finish();

    fn encode(&self, out: &mut Vec<u8>) {
        let door = self.0;
        let kind: u8 = match door.kind {
            DoorKind::Door => 0,
            DoorKind::ShaftDoor => 1,
        };
        let facing: u8 = match door.facing {
            Side::Left => 0,
            Side::Right => 1,
        };
        let (phase, quarter, wait) = phase_parts(door.phase);
        kind.encode(out);
        facing.encode(out);
        door.position.x.0.encode(out);
        door.position.y.0.encode(out);
        phase.encode(out);
        quarter.encode(out);
        wait.encode(out);
    }

    fn decode(input: &mut Reader<'_>) -> Result<Self, DecodeError> {
        let unknown = |tag: u8| DecodeError::UnknownVariant {
            type_name: "Door",
            tag,
        };
        let kind = match u8::decode(input)? {
            0 => DoorKind::Door,
            1 => DoorKind::ShaftDoor,
            tag => return Err(unknown(tag)),
        };
        let facing = match u8::decode(input)? {
            0 => Side::Left,
            1 => Side::Right,
            tag => return Err(unknown(tag)),
        };
        let position = Pair {
            x: Wrapping(u16::decode(input)?),
            y: Wrapping(u16::decode(input)?),
        };
        let tag = u8::decode(input)?;
        let quarter = u8::decode(input)?;
        let wait = u16::decode(input)?;
        let phase = phase_of(tag, quarter, wait).ok_or_else(|| unknown(tag))?;
        Ok(Self(CeresDoor {
            kind,
            facing,
            position,
            phase,
        }))
    }

    fn diff(&self, other: &Self) -> Option<Difference> {
        (self != other).then(|| Difference::here(self, other))
    }
}

/// A phase's tag, its quarter's place and its wait in frames, 0 for those it doesn't carry.
const fn phase_parts(phase: Phase) -> (u8, u8, u16) {
    match phase {
        Phase::Closed { wait } => (0, 0, wait.frames()),
        Phase::Closing { quarter, wait } => (1, quarter_place(quarter), wait.frames()),
        Phase::Open { wait } => (2, 0, wait.frames()),
        Phase::Opening { quarter, wait } => (3, quarter_place(quarter), wait.frames()),
        Phase::Placed => (4, 0, 0),
        Phase::Starting { wait } => (5, 0, wait.frames()),
        Phase::Wall => (6, 0, 0),
    }
}

/// [`phase_parts`]'s inverse; none for parts no phase has, so each phase has one encoding: a
/// phase without a quarter refuses any but 0, one without a wait any but 0.
const fn phase_of(tag: u8, place: u8, wait: u16) -> Option<Phase> {
    let Some(quarter) = quarter_at(place) else {
        return None;
    };
    let no_quarter = place == 0;
    let wait = Wait::new(wait);
    Some(match (tag, wait) {
        (0, Some(wait)) if no_quarter => Phase::Closed { wait },
        (1, Some(wait)) => Phase::Closing { quarter, wait },
        (2, Some(wait)) if no_quarter => Phase::Open { wait },
        (3, Some(wait)) => Phase::Opening { quarter, wait },
        (4, None) if no_quarter => Phase::Placed,
        (5, Some(wait)) if no_quarter => Phase::Starting { wait },
        (6, None) if no_quarter => Phase::Wall,
        _ => return None,
    })
}

/// A quarter's place, from 0.
const fn quarter_place(quarter: Quarter) -> u8 {
    match quarter {
        Quarter::First => 0,
        Quarter::Second => 1,
        Quarter::Third => 2,
        Quarter::Fourth => 3,
    }
}

/// [`quarter_place`]'s inverse.
const fn quarter_at(place: u8) -> Option<Quarter> {
    match place {
        0 => Some(Quarter::First),
        1 => Some(Quarter::Second),
        2 => Some(Quarter::Third),
        3 => Some(Quarter::Fourth),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
