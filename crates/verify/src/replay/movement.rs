//! Once-only movement admission and preceding-cut causal neighbour observations.

use super::{compare::Observation, data_failure, index::ObservationId, result::Outcome};
use crate::source::{self, WramWord};
use core::num::Wrapping;
use data::assets::samus::map;
use data::level::BlockKind;
use data::level::slope::Slopes;
use data::pose::animation::{AnimationDelays, AnimationFrame, RunPaces};
use data::pose::{PoseId, Poses, Shot, ShotDirection};
use data::rom::Rom;
use game::production::world::{Boundary, MovementWorld};
use game::production::{MovementContent, PoseContent};
use samus_movement::{
    Bounces, Carry, Choices, Chosen, Dashing, FrameBefore, JumpHeld, Knockback, Phase, Push,
    PushFrames, Run, Samus, Seat, Vertical,
};
use types::{Action, Buttons, Fixed, Item, Parity, Pose, Position, Radius, Set, Side, Turn, ZERO};

/// The movement types a spin carried on counts as spins: spin jumping (3) and wall jumping (14h).
const SPIN_MOVEMENT_TYPES: [u8; 2] = [0x03, 0x14];

fn word(wram: &[u8], source: WramWord) -> Result<u16, Outcome> {
    source
        .read(wram)
        .map_err(|_| data_failure(None, source.name))
}
fn fixed(wram: &[u8], pixel: WramWord, sub: WramWord) -> Result<Fixed, Outcome> {
    Ok(types::fixed(Wrapping(word(wram, pixel)?), word(wram, sub)?))
}
fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}

pub struct Content {
    pub movement: MovementContent,
    pub clips: game::samus_appearance_bridge::Clips,
    /// Her clips as production plays them, for her clip's look ahead.
    appearance: game::production::AppearanceContent,
    slopes: Slopes,
    rom: Rom,
}

impl Content {
    pub fn load(path: &std::path::Path) -> Result<Self, Outcome> {
        let image = std::fs::read(path).map_err(|_| data_failure(None, "player ROM file"))?;
        let rom = Rom::new(image).map_err(|_| data_failure(None, "player ROM pin"))?;
        let poses = Poses::load(rom.lorom()).map_err(|_| data_failure(None, "pose content"))?;
        let delays = AnimationDelays::load(rom.lorom(), &poses)
            .map_err(|_| data_failure(None, "animation clip content"))?;
        let paces = RunPaces::load(rom.lorom())
            .map_err(|_| data_failure(None, "animation pace content"))?;
        let clips = game::samus_appearance_bridge::Clips::import(&delays, &paces)
            .map_err(|_| data_failure(None, "semantic animation content"))?;
        let entries = map::MAP
            .iter()
            .map(|entry| {
                let definition = poses
                    .get(entry.id)
                    .ok_or_else(|| data_failure(None, "selectable pose content"))?;
                Ok(PoseContent {
                    pose: entry.key,
                    radius: Radius {
                        x: Wrapping(5),
                        y: Wrapping(u16::from(definition.y_radius)),
                    },
                    cannon_heading: heading(definition.shot),
                })
            })
            .collect::<Result<Vec<_>, Outcome>>()?;
        let movement = MovementContent::new(entries)
            .map_err(|_| data_failure(None, "movement content relation"))?;
        let slopes = Slopes::load(rom.lorom()).map_err(|_| data_failure(None, "slope content"))?;
        let appearance = super::appearance::production_content(&rom, &clips)?;
        Ok(Self {
            movement,
            clips,
            appearance,
            slopes,
            rom,
        })
    }
}

const fn heading(shot: Shot) -> Option<types::FiringDirection> {
    use types::FiringDirection as F;
    match shot {
        Shot::Withheld(_) => None,
        Shot::Fires(direction) => Some(match direction {
            ShotDirection::Left => F::Left,
            ShotDirection::Right => F::Right,
            ShotDirection::DownLeft => F::DownLeft,
            ShotDirection::DownRight => F::DownRight,
            ShotDirection::UpLeft => F::UpLeft,
            ShotDirection::UpRight => F::UpRight,
            ShotDirection::DownFacingLeft => F::DownFacingLeft,
            ShotDirection::DownFacingRight => F::DownFacingRight,
            ShotDirection::UpFacingLeft => F::UpFacingLeft,
            ShotDirection::UpFacingRight => F::UpFacingRight,
        }),
    }
}

/// The only record-to-selected-state conversion. Subsequent cuts decode neighbours only.
pub fn seed(wram: &[u8]) -> Result<Samus, Outcome> {
    let pose = map::key_of(PoseId(word(wram, source::SAMUS_POSE)?))
        .ok_or_else(|| unsupported("movement seed pose"))?;
    let (carry, way) = match word(wram, source::SAMUS_X_ACCEL_MODE)? {
        0 => (Carry::FollowsPad, Side::of(pose.facing)),
        1 => (Carry::CarriesOn, Side::of(pose.facing.back())),
        2 => (Carry::CarriesOn, Side::of(pose.facing)),
        _ => return Err(data_failure(None, "movement speed mode")),
    };
    let magnitude = fixed(wram, source::SAMUS_Y_SPEED, source::SAMUS_Y_SUBSPEED)?;
    let airborne = matches!(
        pose.action,
        Action::Jumping | Action::Spinning | Action::Falling | Action::WallJumping
    );
    let turning = matches!(pose.action, Action::Turning(Turn::Jump | Turn::Fall));
    let (phase, speed) = match word(wram, source::SAMUS_Y_DIR)? {
        0 if airborne || (turning && magnitude != ZERO) => (Phase::Moonfall, -magnitude),
        0 => (Phase::Ground, -magnitude),
        1 => (Phase::Rising, -magnitude),
        2 if airborne || turning => (Phase::Falling, magnitude),
        2 if magnitude == ZERO => (Phase::GroundFalling, ZERO),
        2 => (Phase::Ground, magnitude),
        _ => return Err(data_failure(None, "movement vertical phase")),
    };
    let [bounce, _] = word(wram, source::SAMUS_BOUNCE_STATE)?.to_le_bytes();
    let bounces = match bounce {
        0 => Bounces::None,
        1 => Bounces::Once,
        2 => Bounces::Twice,
        _ => return Err(data_failure(None, "movement bounce count")),
    };
    // `$91:EBF8` shifts the pose history only when a choice is made: the old pose's movement
    // type is what her latest choice recorded, the old-old one what the choice before it did.
    let chosen = |source: source::WramByte| -> Result<Chosen, Outcome> {
        let movement_type = source
            .read(wram)
            .map_err(|_| data_failure(None, source.name))?;
        Ok(if SPIN_MOVEMENT_TYPES.contains(&movement_type) {
            Chosen::Spin
        } else {
            Chosen::Other
        })
    };
    let choices = Choices {
        latest: chosen(source::SAMUS_PREV_MOVEMENT_TYPE2)?,
        before: chosen(source::SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE)?,
    };
    let knockback = knockback(wram)?;
    Ok(Samus {
        position: Position {
            x: fixed(wram, source::SAMUS_X_POS, source::SAMUS_X_SUBPOS)?,
            y: fixed(wram, source::SAMUS_Y_POS, source::SAMUS_Y_SUBPOS)?,
        },
        run: Run {
            speed: fixed(
                wram,
                source::SAMUS_X_BASE_SPEED,
                source::SAMUS_X_BASE_SUBSPEED,
            )?,
            way,
            carry,
            dash: fixed(
                wram,
                source::SAMUS_X_EXTRA_RUN_SPEED,
                source::SAMUS_X_EXTRA_RUN_SUBSPEED,
            )?,
            dashing: if word(wram, source::SAMUS_HAS_MOMENTUM_FLAG)? == 0 {
                Dashing::Off
            } else {
                Dashing::On
            },
        },
        vertical: Vertical {
            speed,
            phase,
            bounces,
        },
        pose,
        choices,
        seat: match word(wram, source::SAMUS_SLOPE_ADJUSTED)? {
            0 => Seat::Unseated,
            1 => Seat::Seated,
            _ => return Err(data_failure(None, "movement slope seat")),
        },
        jump_held: JumpHeld {
            frames: Wrapping(word(wram, source::SAMUS_JUMP_HELD_FRAMES)?),
            before: if word(wram, source::SAMUS_POSE_INPUT_HANDLER)? == 0xE926 {
                FrameBefore::LandedOrEnded
            } else {
                FrameBefore::Other
            },
        },
        knockback,
    })
}

/// Her knockback: the push time left (`18AA`), the side a hit pushes her to (`0A54`) and the
/// push under way (`0A52`, 1 to 3 up, 4 or 5 down).
fn knockback(wram: &[u8]) -> Result<Knockback, Outcome> {
    let time_left = u8::try_from(word(wram, source::SAMUS_KNOCKBACK_TIMER)?)
        .ok()
        .and_then(PushFrames::new)
        .ok_or_else(|| data_failure(None, "movement push time"))?;
    let side = match word(wram, source::SAMUS_KNOCKBACK_X_DIRECTION)? {
        0 => Side::Left,
        1 => Side::Right,
        _ => return Err(data_failure(None, "movement push side")),
    };
    let push = match word(wram, source::SAMUS_KNOCKBACK_DIRECTION)? {
        0 => None,
        1..=3 => Some(Push::Up),
        4 | 5 => Some(Push::Down),
        _ => return Err(data_failure(None, "movement push under way")),
    };
    Ok(Knockback {
        time_left,
        side,
        push,
    })
}

pub struct Input {
    pub sprite: types::Sprite,
    pub cinematic: bool,
    pub pad: Buttons,
    pub pressed: Buttons,
    pub items: Set<Item>,
    pub ahead: samus_appearance::Ahead,
    pub elevator: bool,
    pub parity: Parity,
    pub world: MovementWorld,
}

impl Input {
    pub(super) const fn stimuli(&self) -> game::production::MovementInputs {
        game::production::MovementInputs {
            sprite_before_movement: self.sprite,
            cinematic_active: self.cinematic,
            pad: self.pad,
            pressed: self.pressed,
            items: self.items,
            spin_turned_far: self.ahead.turned_far,
            elevator_moving: self.elevator,
            parity: self.parity,
            animation_ends: self.ahead.end,
        }
    }
}

pub(super) fn advance_failure(
    cut: u64,
    initial_pose: Pose,
    missing_seed: &str,
    fault: game::production::ProductionFault,
) -> Outcome {
    use game::production::{ProductionFault, Unsupported};
    if fault == ProductionFault::Unsupported(Unsupported::PoseContent(initial_pose)) {
        return data_failure(Some(cut), missing_seed);
    }
    super::production_failure(cut, fault)
}

pub(super) fn pad(value: u16) -> Buttons {
    use types::button as B;
    [
        (0x8000, B::RUN),
        (0x0080, B::JUMP),
        (0x0040, B::SHOOT),
        (0x0200, B::LEFT),
        (0x0100, B::RIGHT),
        (0x0800, B::UP),
        (0x0400, B::DOWN),
        (0x0010, B::AIM_UP),
        (0x0020, B::AIM_DOWN),
    ]
    .into_iter()
    .filter(|(bit, _)| value & bit != 0)
    .fold(Buttons::default(), |out, (_, button)| {
        Buttons(out.0 | button.0)
    })
}

pub(super) fn items(value: u16) -> Set<Item> {
    let mut items = Set::default();
    for (bit, item) in [
        (0x0001, Item::VariaSuit),
        (0x0002, Item::SpringBall),
        (0x0004, Item::MorphBall),
        (0x0008, Item::ScrewAttack),
        (0x0020, Item::GravitySuit),
        (0x0100, Item::HiJumpBoots),
        (0x0200, Item::SpaceJump),
        (0x2000, Item::SpeedBooster),
    ] {
        items[item] = value & bit != 0;
    }
    items
}

pub(super) struct Borrowed {
    pub held: Buttons,
    pub pressed: Buttons,
    pub items: Set<Item>,
    pub cinematic: bool,
    pub elevator: bool,
    pub parity: Parity,
    pub world: MovementWorld,
}

/// Decode unselected current stimuli without reading selected pose, playhead or Ahead.
pub(super) fn borrowed(
    current: &[u8],
    geometry: &[u8],
    content: &Content,
) -> Result<Borrowed, Outcome> {
    Ok(Borrowed {
        held: pad(word(current, source::JOYPAD1_LASTKEYS)?),
        pressed: pad(word(current, source::JOYPAD1_NEWKEYS)?),
        items: items(word(current, source::EQUIPPED_ITEMS)?),
        cinematic: word(current, source::CINEMATIC_ACTIVE)? != 0,
        elevator: word(current, source::ELEVATOR_STATUS)? != 0,
        parity: if word(current, source::NMI_FRAME_COUNTER_WORD)? & 1 == 0 {
            Parity::Even
        } else {
            Parity::Odd
        },
        world: world(geometry, content)?,
    })
}

pub fn input(wram: &[u8], actual_pose: Pose, content: &Content) -> Result<Input, Outcome> {
    input_at(wram, wram, actual_pose, content)
}

/// Frame and geometry can have separately qualified consumption opportunities.
pub(super) fn input_at(
    wram: &[u8],
    world_wram: &[u8],
    actual_pose: Pose,
    content: &Content,
) -> Result<Input, Outcome> {
    let raw_pose = PoseId(word(wram, source::SAMUS_POSE)?);
    let observed_figure = content
        .clips
        .of(raw_pose)
        .ok_or_else(|| unsupported("appearance figure observation"))?;
    let imported = content
        .clips
        .poses()
        .iter()
        .find(|imported| {
            imported.figure.pose == actual_pose
                && imported.figure.sprite == observed_figure.figure.sprite
        })
        .ok_or_else(|| unsupported("actual movement pose animation content"))?;
    let step = imported
        .step_of(AnimationFrame(word(wram, source::SAMUS_ANIM_FRAME)?))
        .ok_or_else(|| data_failure(None, "appearance observed playhead"))?;
    let timer = u8::try_from(word(wram, source::SAMUS_ANIM_FRAME_TIMER)?)
        .map_err(|_| data_failure(None, "appearance observed frame count"))?;
    let [laps, stage] = word(wram, source::SPEED_BOOSTER_LEVEL)?.to_le_bytes();
    let stage = types::BoostStage::ALL
        .get(usize::from(stage))
        .copied()
        .ok_or_else(|| data_failure(None, "appearance observed boost stage"))?;
    // A movement case borrows the read-ahead, which asks her State for her step and the time
    // left (spec §1.1). It borrows her animation alone; the rest of her appearance State is no
    // input of the case, so it stands at her initialization's, which the query doesn't read.
    let appearance = samus_appearance::AppearanceState {
        animation: samus_appearance::Animation {
            playhead: samus_appearance::Playhead {
                step,
                time_left: samus_appearance::Frames(timer),
            },
            stage,
            laps_left: samus_appearance::Laps::new(laps).map_or(
                samus_appearance::LapsLeft::WithoutBooster,
                samus_appearance::LapsLeft::WithBooster,
            ),
        },
        ..samus_appearance::AppearanceState::INITIAL
    };
    let ahead = samus_appearance::Ahead::of(&appearance, imported.figure, &content.appearance)
        .map_err(|_| unsupported("actual movement pose animation content"))?;
    Ok(Input {
        sprite: observed_figure.figure.sprite,
        cinematic: word(wram, source::CINEMATIC_ACTIVE)? != 0,
        pad: pad(word(wram, source::JOYPAD1_LASTKEYS)?),
        pressed: pad(word(wram, source::JOYPAD1_NEWKEYS)?),
        items: items(word(wram, source::EQUIPPED_ITEMS)?),
        ahead,
        elevator: word(wram, source::ELEVATOR_STATUS)? != 0,
        parity: if word(wram, source::NMI_FRAME_COUNTER_WORD)? & 1 == 0 {
            Parity::Even
        } else {
            Parity::Odd
        },
        world: world(world_wram, content)?,
    })
}

fn world(wram: &[u8], content: &Content) -> Result<MovementWorld, Outcome> {
    level_world(wram, content, enemy_roster(wram)?)
}

pub(super) fn roster_slots(wram: &[u8]) -> Result<Vec<u16>, Outcome> {
    let mut slots = Vec::new();
    for occurrence in 0..=32 {
        let index = source::INTERACTIVE_ENEMY_INDICES
            .word(occurrence)
            .and_then(|field| field.read(wram))
            .map_err(|_| data_failure(None, "prior-cut interactive roster observation"))?;
        if index == u16::MAX {
            return Ok(slots);
        }
        if occurrence == 32 || !index.is_multiple_of(64) || index >= 2048 {
            return Err(data_failure(
                None,
                "prior-cut interactive roster slot or termination",
            ));
        }
        slots.push(index / 64);
    }
    Err(data_failure(
        None,
        "prior-cut interactive roster termination",
    ))
}

fn enemy_roster(wram: &[u8]) -> Result<Vec<types::Hitbox>, Outcome> {
    let mut boxes = Vec::new();
    for slot in roster_slots(wram)? {
        let read = |field| {
            source::ENEMY_DATA
                .word(field, slot)
                .and_then(|source| source.read(wram))
                .map_err(|_| data_failure(None, "enemy geometry observation"))
        };
        if read(source::ENEMY_SPECIES)? == 0 {
            return Err(data_failure(
                None,
                "prior-cut interactive roster occupied slot",
            ));
        }
        let observed_solid = (read(source::ENEMY_PROPERTIES)? & 0x8000) != 0;
        let observed_frozen = read(source::ENEMY_FROZEN_TIMER)? != 0;
        if observed_solid || observed_frozen {
            boxes.push(types::Hitbox {
                centre: types::Pair {
                    x: Wrapping(read(source::ENEMY_X_POS)?),
                    y: Wrapping(read(source::ENEMY_Y_POS)?),
                },
                radius: Radius {
                    x: Wrapping(read(source::ENEMY_X_RADIUS)?),
                    y: Wrapping(read(source::ENEMY_Y_RADIUS)?),
                },
            });
        }
    }
    Ok(boxes)
}

pub(super) fn level_world(
    wram: &[u8],
    content: &Content,
    enemies: Vec<types::Hitbox>,
) -> Result<MovementWorld, Outcome> {
    let (width, cells, permissions) = level_blocks(wram, content)?;
    MovementWorld::new(width, cells, permissions, enemies, Boundary::Air)
        .map_err(|_| data_failure(None, "admitted movement world relation"))
}

/// Decode current immutable room blocks without inventing an actor provider.
pub(super) fn level_blocks(
    wram: &[u8],
    content: &Content,
) -> Result<(u16, Vec<types::Block>, Vec<bool>), Outcome> {
    let width = word(wram, source::ROOM_WIDTH_IN_BLOCKS)?;
    let height = word(wram, source::ROOM_HEIGHT_IN_BLOCKS)?;
    if width > 255 {
        return Err(unsupported("movement wrapped level addressing"));
    }
    let count = usize::from(width)
        .checked_mul(usize::from(height))
        .ok_or_else(|| data_failure(None, "level observation size"))?;
    let byte_count = count
        .checked_mul(2)
        .ok_or_else(|| data_failure(None, "level observation bytes"))?;
    let words = source::LEVEL_DATA
        .read(wram)
        .ok()
        .and_then(|bytes| bytes.get(..byte_count))
        .ok_or_else(|| data_failure(None, "level observation"))?;
    let bts = source::BTS
        .read(wram)
        .ok()
        .and_then(|bytes| bytes.get(..count))
        .ok_or_else(|| data_failure(None, "level BTS observation"))?;
    let room = data::room::decode(
        content.rom.lorom(),
        data::room::RoomKey::from_room_pointer(word(wram, source::ROOM_PTR)?),
    )
    .map_err(|_| data_failure(None, "room door content"))?;
    let kinds: Vec<BlockKind> = words
        .as_chunks::<2>()
        .0
        .iter()
        .zip(bts)
        .map(|(bytes, bts)| data::level::Block::new(u16::from_le_bytes(*bytes), *bts).kind)
        .collect();
    let leads = |place| {
        matches!(
            room.doors.doors.get(place),
            Some(data::room::Door::ToRoom(_))
        )
    };
    let (cells, permissions) =
        game::production::room_loading::blocks(&kinds, width, &content.slopes, leads)
            .map_err(|_| data_failure(None, "slope shape"))?;
    Ok((width, cells, permissions))
}

/// The room's door cells, each with its place in the room's list of doors.
pub(super) fn door_cells(wram: &[u8]) -> Result<Vec<(types::Cell, usize)>, Outcome> {
    let width = word(wram, source::ROOM_WIDTH_IN_BLOCKS)?;
    let count = usize::from(width)
        .checked_mul(usize::from(word(wram, source::ROOM_HEIGHT_IN_BLOCKS)?))
        .ok_or_else(|| data_failure(None, "level observation size"))?;
    let words = source::LEVEL_DATA
        .read(wram)
        .ok()
        .and_then(|bytes| bytes.get(..count.saturating_mul(2)))
        .ok_or_else(|| data_failure(None, "level observation"))?;
    let bts = source::BTS
        .read(wram)
        .ok()
        .and_then(|bytes| bytes.get(..count))
        .ok_or_else(|| data_failure(None, "level BTS observation"))?;
    let mut doors = Vec::new();
    for (at, (bytes, bts)) in (0_u16..).zip(words.as_chunks::<2>().0.iter().zip(bts)) {
        if let BlockKind::Door(door) =
            data::level::Block::new(u16::from_le_bytes(*bytes), *bts).kind
        {
            let cell = types::Cell {
                x: at.checked_rem(width).unwrap_or(at),
                y: at.checked_div(width).unwrap_or(0),
            };
            doors.push((cell, door.place()));
        }
    }
    Ok(doors)
}

pub fn expected(
    wram: &[u8],
    selected: &[ObservationId],
    row: &str,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let travels = crate::decode::directional_travel(wram)
        .map_err(|_| data_failure(None, "directional travel observation"))?;
    selected
        .iter()
        .map(|field| expected_field(wram, *field, &travels, row, cut))
        .collect()
}

fn travel_quantity(
    travels: &types::DirectionalTravels,
    direction: types::Direction,
) -> (&'static str, i64) {
    let name = match direction {
        types::Direction::Left => "movement.travel.Left",
        types::Direction::Right => "movement.travel.Right",
        types::Direction::Up => "movement.travel.Up",
        types::Direction::Down => "movement.travel.Down",
    };
    (name, i64::from(travels[direction].0.0))
}

fn expected_field(
    wram: &[u8],
    field: ObservationId,
    travels: &types::DirectionalTravels,
    row: &str,
    cut: u64,
) -> Result<Observation, Outcome> {
    use ObservationId as O;
    use types::Direction;

    for (id, name, whole, fraction) in [
        (
            O::PositionX,
            "movement.position.x",
            source::SAMUS_X_POS,
            source::SAMUS_X_SUBPOS,
        ),
        (
            O::PositionY,
            "movement.position.y",
            source::SAMUS_Y_POS,
            source::SAMUS_Y_SUBPOS,
        ),
        (
            O::VerticalSpeed,
            "movement.vertical_speed",
            source::SAMUS_Y_SPEED,
            source::SAMUS_Y_SUBSPEED,
        ),
    ] {
        if field == id {
            return Ok(Observation::quantity(
                name,
                i64::from(fixed(wram, whole, fraction)?.0),
            ));
        }
    }
    for (id, direction) in [
        (O::TravelLeft, Direction::Left),
        (O::TravelRight, Direction::Right),
        (O::TravelUp, Direction::Up),
        (O::TravelDown, Direction::Down),
    ] {
        if field == id {
            let (name, value) = travel_quantity(travels, direction);
            return Ok(Observation::quantity(name, value));
        }
    }
    if matches!(field, O::WallTest | O::Reached) {
        return reached_observation(field, row, cut);
    }
    Err(data_failure(None, "movement observation scope"))
}

fn reached_observation(field: ObservationId, row: &str, cut: u64) -> Result<Observation, Outcome> {
    if row != "item-morph-ball" || cut != 817 {
        return Err(data_failure(
            Some(cut),
            "named reached-cell witness row and cut",
        ));
    }
    if field == ObservationId::WallTest {
        Ok(Observation::quantity("movement.wall_test", 0))
    } else {
        Ok(Observation {
            field: "movement.reached".to_owned(),
            bytes: reached_w15(),
        })
    }
}

/// Observer's named W15 ordered reached-cell witness, in pixel coordinates.
fn reached_w15() -> Vec<u8> {
    encode_reached(&[
        ([1120, 640], 1),
        ([1120, 656], 1),
        ([1104, 640], 0),
        ([1104, 656], 0),
        ([1120, 656], 4),
        ([1136, 656], 4),
        ([1120, 624], 0),
        ([1136, 624], 0),
        ([1120, 672], 0),
        ([1136, 672], 0),
    ])
}

fn encode_reached(reached: &[([u16; 2], u8)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for ([x, y], way) in reached {
        bytes.extend_from_slice(&x.to_le_bytes());
        bytes.extend_from_slice(&y.to_le_bytes());
        bytes.push(*way);
    }
    bytes
}

pub fn reached(outcome: &samus_movement::Outcome) -> Observation {
    let reached: Vec<_> = outcome
        .reached
        .iter()
        .map(|touch| {
            let way = match touch.way {
                types::Way::Look => 0,
                types::Way::Move(types::Heading::Left) => 1,
                types::Way::Move(types::Heading::Right) => 2,
                types::Way::Move(types::Heading::Up) => 3,
                types::Way::Move(types::Heading::Down) => 4,
            };
            (
                [touch.cell.x.wrapping_mul(16), touch.cell.y.wrapping_mul(16)],
                way,
            )
        })
        .collect();
    Observation {
        field: "movement.reached".to_owned(),
        bytes: encode_reached(&reached),
    }
}

#[cfg(test)]
mod tests {
    use super::roster_slots;
    use crate::replay::result::Outcome;

    fn capture(indices: &[u16]) -> Vec<u8> {
        let mut wram = vec![0; 0x20000];
        for (offset, index) in indices.iter().enumerate() {
            let at = 0x17EC_usize.saturating_add(offset.saturating_mul(2));
            wram[at..at.saturating_add(2)].copy_from_slice(&index.to_le_bytes());
        }
        wram
    }

    #[test]
    fn prior_roster_preserves_occurrences_and_stops_before_unrelated_words() {
        assert_eq!(
            roster_slots(&capture(&[192, 64, 192, u16::MAX, 5])).unwrap(),
            [3, 1, 3]
        );
        assert_eq!(
            roster_slots(&capture(&[u16::MAX])).unwrap(),
            Vec::<u16>::new()
        );
    }

    #[test]
    fn malformed_or_unterminated_roster_is_a_data_failure() {
        for indices in [&[63, u16::MAX][..], &[2048, u16::MAX][..], &[0; 33][..]] {
            assert!(matches!(
                roster_slots(&capture(indices)),
                Err(Outcome::DataFailure { .. })
            ));
        }
        assert!(matches!(
            roster_slots(&[]),
            Err(Outcome::DataFailure { .. })
        ));
    }
}
