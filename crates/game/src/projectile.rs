//! Samus's projectiles each frame (bank $90 log, `$90:AC1C` onward).
//!
//! The cooldown runs, then each live projectile's pre-instruction, which flies a beam and
//! checks the blocks it reaches (bank $94 log, `$94:A23B`), then its instruction list (bank
//! $93 log, `$93:81E9`).

use data::level::BlockKind;

use crate::plm::crumble::{self, ShotFault};
use crate::pose::FACING_FORWARD;
use crate::state::State;
use crate::state::projectile::{
    Beam, BeamShot, Direction, ExplosionStep, ProjectileBody, ProjectileType, Projectiles, Radius,
    SHOT_SLOTS, Shot, ShotCount, SubpixelSpeed,
};
use crate::state::room::{BlockIndex, Camera, RoomScoped};
use crate::state::samus::Fixed;
use crate::state::slots::{Frames, Occupancy, Slot};

/// The pixels a block spans, as a shift.
const BLOCK_SHIFT: u32 = 4;
/// The blocks a screen spans.
const SCREEN_BLOCKS: u16 = 16;
/// A span of this many blocks past the first is too wide to check: the check gives up and
/// reports no collision (`$94:A299`).
const WIDEST_SPAN: u16 = 0x10;
/// The frames between a beam's trails (`$90:AF05`).
const TRAIL_INTERVAL: Frames = Frames(4);
/// The damage an explosion deals (`$93:8145`).
const EXPLOSION_DAMAGE: u16 = 8;
/// The bits of a speed below its whole pixels, as a shift: a speed counts 1/256ths of a
/// pixel, a position's subpixels 1/65536ths.
const SPEED_FRACTION: u32 = 8;

/// One frame of a projectile's instruction list: how long it shows and the hitbox it gives
/// the projectile (`$93:8209`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ListFrame {
    /// Frames it shows for.
    frames: Frames,
    /// The projectile's radius while it shows.
    radius: Radius,
}

/// The uncharged power beam's one frame, by direction (`$93:86DB` to `$93:872F`): its list
/// shows it and goes back to it.
const fn power_beam_frame(direction: Direction) -> ListFrame {
    const FRAMES: Frames = Frames(0x0f);
    let radius = match direction {
        Direction::UpFacingRight
        | Direction::DownFacingRight
        | Direction::DownFacingLeft
        | Direction::UpFacingLeft => Radius { x: 4, y: 4 },
        Direction::UpRight
        | Direction::Right
        | Direction::DownRight
        | Direction::DownLeft
        | Direction::Left
        | Direction::UpLeft => Radius { x: 8, y: 4 },
    };
    ListFrame {
        frames: FRAMES,
        radius,
    }
}

/// The beam explosion's frames (`$93:A007`), after which its list deletes it.
const BEAM_EXPLOSION: [ListFrame; 6] = [ListFrame {
    frames: Frames(3),
    radius: Radius { x: 0, y: 0 },
}; 6];
/// How far past the camera's edge a projectile may fly before it's deleted: 40h pixels
/// either side, the playfield 140h wide and tall (`$90:B16A`).
const MARGIN: u16 = 0x40;
/// The span from the camera's edge past which a projectile is deleted.
const REACH: u16 = 0x140;

/// The frame's cooldown (`$90:AC1C`): it counts down to zero, and a negative one is zero.
// open: frozen time sets it to 20h; the time-freezing states join with their rooms.
pub const fn cool_down(projectiles: &mut Projectiles) {
    let Frames(left) = projectiles.cooldown;
    projectiles.cooldown = Frames(if left == 0 || signed(left) < 0 {
        0
    } else {
        left.wrapping_sub(1)
    });
}

/// Why a projectile can't go on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileFault {
    /// It shot a shootable block whose reaction isn't modelled.
    Shot(ShotFault),
}

/// The projectiles' part of Samus's normal current state handler (`$90:DCDD`): the cooldown
/// runs unless she faces forward, then the projectile pass.
///
/// # Errors
///
/// A [`ProjectileFault`] when a beam reaches a block whose reaction isn't modelled.
// open: the HUD selection and the movement type's selection handler between them, which
// fire, join with firing; frozen time, which skips the pass, with its states.
pub fn current_state(state: &mut State) -> Result<(), ProjectileFault> {
    if !FACING_FORWARD.contains(&state.samus.pose) {
        cool_down(&mut state.projectiles);
    }
    handle(state)
}

/// The projectile pass (`$90:AECE`), which the locked current state handler runs alone
/// (`$90:E713`): each live projectile, last slot first, runs its pre-instruction, then its
/// instruction list.
///
/// # Errors
///
/// A [`ProjectileFault`] when a beam reaches a block whose reaction isn't modelled.
// open: the wave beam, hyper beam, missiles and bombs join with the rooms that fire them;
// until then they stand still.
pub fn handle(state: &mut State) -> Result<(), ProjectileFault> {
    let State {
        projectiles, room, ..
    } = state;
    for index in (0..projectiles.slots.slots.len()).rev() {
        let Some(slot) = projectiles.slots.slots.get(index) else {
            continue;
        };
        let Occupancy::Live(body) = slot.occupancy else {
            continue;
        };
        match body.kind.shot {
            Shot::Beam(beam) if !beam.beams.contains(Beam::Wave) => {
                fly_beam(projectiles, index, room)?;
            }
            Shot::Beam(_)
            | Shot::Missile
            | Shot::SuperMissile
            | Shot::PowerBomb
            | Shot::Bomb
            | Shot::BeamExplosion { .. }
            | Shot::MissileExplosion => {}
        }
        animate(projectiles, index);
    }
    Ok(())
}

/// The instruction handler (`$93:81E9`) for slot `index`: the frame timer counts down, and
/// at zero the list shows its next frame, which sets the projectile's radius.
// open: the spritemaps and trail frames the lists also give join with drawing; the lists of
// the beams besides the uncharged power beam, and of missiles and bombs, with those shots.
fn animate(projectiles: &mut Projectiles, index: usize) {
    let Some(slot) = projectiles.slots.slots.get_mut(index) else {
        return;
    };
    let Occupancy::Live(body) = &mut slot.occupancy else {
        return;
    };
    body.frame_timer = Frames(body.frame_timer.0.wrapping_sub(1));
    if body.frame_timer != Frames(0) {
        return;
    }
    let direction = body.heading.direction;
    let next = match &mut body.kind.shot {
        Shot::Beam(BeamShot {
            beams,
            charged: false,
        }) if beams.is_empty() => Some(power_beam_frame(direction)),
        Shot::BeamExplosion { step, .. } => {
            let frame = BEAM_EXPLOSION.get(usize::from(step.0)).copied();
            *step = ExplosionStep(step.0.saturating_add(1));
            frame
        }
        Shot::Beam(_)
        | Shot::Missile
        | Shot::SuperMissile
        | Shot::PowerBomb
        | Shot::Bomb
        | Shot::MissileExplosion => return,
    };
    match next {
        Some(frame) => {
            body.frame_timer = frame.frames;
            body.radius = frame.radius;
        }
        // The list's delete (`$93:822F`).
        None => clear(projectiles, index),
    }
}

/// The pre-instruction of a beam without the wave beam (`$90:AEF3`), in slot `index`.
fn fly_beam(
    projectiles: &mut Projectiles,
    index: usize,
    room: &mut RoomScoped,
) -> Result<(), ProjectileFault> {
    let Some(slot) = projectiles.slots.slots.get_mut(index) else {
        return Ok(());
    };
    let Occupancy::Live(mut body) = slot.occupancy else {
        return Ok(());
    };
    if body.heading.deleted {
        clear(projectiles, index);
        return Ok(());
    }
    let Frames(trail) = slot.common.trail_timer;
    slot.common.trail_timer = Frames(trail.wrapping_sub(1));
    if slot.common.trail_timer == Frames(0) {
        // open: the trail itself (`$90:B657`) joins with drawing.
        slot.common.trail_timer = TRAIL_INTERVAL;
    }
    let (ax, ay) = acceleration(body.heading.direction);
    body.velocity.x = SubpixelSpeed(body.velocity.x.0.wrapping_add(ax));
    body.velocity.y = SubpixelSpeed(body.velocity.y.0.wrapping_add(ay));
    let collided = match axes(body.heading.direction) {
        Axes::Horizontal => move_horizontally(&mut body, room)?,
        Axes::Vertical => move_vertically(&mut body, room)?,
        Axes::Diagonal => move_horizontally(&mut body, room)? || move_vertically(&mut body, room)?,
    };
    if collided && let Shot::Beam(beam) = body.kind.shot {
        explode(&mut body, beam);
    }
    if let Some(slot) = projectiles.slots.slots.get_mut(index) {
        slot.occupancy = Occupancy::Live(body);
    }
    if !on_screen(&body, &room.camera) {
        clear(projectiles, index);
    }
    Ok(())
}

/// Which axes a beam moves along, by its direction (`$90:AF36`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axes {
    /// Horizontally only.
    Horizontal,
    /// Vertically only.
    Vertical,
    /// Horizontally, then vertically unless that collided.
    Diagonal,
}

const fn axes(direction: Direction) -> Axes {
    match direction {
        Direction::UpFacingRight
        | Direction::DownFacingRight
        | Direction::DownFacingLeft
        | Direction::UpFacingLeft => Axes::Vertical,
        Direction::UpRight | Direction::DownRight | Direction::DownLeft | Direction::UpLeft => {
            Axes::Diagonal
        }
        Direction::Right | Direction::Left => Axes::Horizontal,
    }
}

/// A beam's speeds' change a frame, in 1/256ths of a pixel, by its direction (`$90:C353`,
/// `$90:C367`): 10h away from Samus along each axis it moves on.
const fn acceleration(direction: Direction) -> (i16, i16) {
    const STEP: i16 = 0x10;
    match direction {
        Direction::UpFacingRight | Direction::UpFacingLeft => (0, -STEP),
        Direction::UpRight => (STEP, -STEP),
        Direction::Right => (STEP, 0),
        Direction::DownRight => (STEP, STEP),
        Direction::DownFacingRight | Direction::DownFacingLeft => (0, STEP),
        Direction::DownLeft => (-STEP, STEP),
        Direction::Left => (-STEP, 0),
        Direction::UpLeft => (-STEP, -STEP),
    }
}

/// `position` moved by `speed`: the speed's whole pixels and its fraction as the top byte of
/// the subpixels, carried as one 32-bit number.
fn advance(position: Fixed, speed: SubpixelSpeed) -> Fixed {
    let [pixels_high, pixels_low] = position.pixels.to_be_bytes();
    let [subpixels_high, subpixels_low] = position.subpixels.to_be_bytes();
    let whole = u32::from_be_bytes([pixels_high, pixels_low, subpixels_high, subpixels_low]);
    let moved = whole.wrapping_add_signed(i32::from(speed.0) << SPEED_FRACTION);
    let [a, b, c, d] = moved.to_be_bytes();
    Fixed {
        pixels: u16::from_be_bytes([a, b]),
        subpixels: u16::from_be_bytes([c, d]),
    }
}

/// A word read as signed, as the original's sign tests read it.
const fn signed(value: u16) -> i16 {
    i16::from_le_bytes(value.to_le_bytes())
}

/// Whether `value - other` is negative, as the original's compare and branch on minus read
/// it: not a signed compare, which overflow would flip.
const fn below(value: u16, other: u16) -> bool {
    signed(value.wrapping_sub(other)) < 0
}

/// The blocks a span from `low - radius` to `high + radius - 1` covers past the first, as
/// the original counts them from the low edge's block (`$94:A1F7`).
const fn span(centre: u16, radius: u16) -> u16 {
    let low = (centre.wrapping_sub(radius) >> BLOCK_SHIFT) << BLOCK_SHIFT;
    centre
        .wrapping_add(radius)
        .wrapping_sub(1)
        .wrapping_sub(low)
        >> BLOCK_SHIFT
}

/// The low byte of a word, as the original's 8-bit multiplier takes it.
fn low_byte(value: u16) -> u16 {
    let [low, _] = value.to_le_bytes();
    u16::from(low)
}

/// The high byte of a word: a position's screen.
fn high_byte(value: u16) -> u16 {
    let [_, high] = value.to_le_bytes();
    u16::from(high)
}

/// How the blocks a beam's leading edge reached answer it.
struct Reaction {
    /// Blocks still to check, less one (`$26`).
    left: u16,
    /// Collisions still needed to stop the beam, less one (`$28`): negative once enough hit.
    needed: u16,
}

/// The blocks from byte index `start`, `step` bytes apart, checked in turn (`$94:A1B5`,
/// `$94:A1D6`): whether enough collided to stop the beam.
fn check_blocks(
    room: &mut RoomScoped,
    start: u16,
    step: u16,
    count: u16,
) -> Result<bool, ProjectileFault> {
    let mut reaction = Reaction {
        left: count,
        needed: count,
    };
    let mut at = start;
    let size = u16::try_from(room.blocks.len().saturating_mul(2)).unwrap_or(u16::MAX);
    loop {
        // A block past the level data counts as neither.
        if at < size {
            let block = BlockIndex(at >> 1);
            let kind = room
                .blocks
                .get(usize::from(block.0))
                .map(|cell| cell.kind)
                .unwrap_or_default();
            if react(room, block, kind)? {
                reaction.needed = reaction.needed.wrapping_sub(1);
            }
        }
        at = at.wrapping_add(step);
        reaction.left = reaction.left.wrapping_sub(1);
        if signed(reaction.left) < 0 {
            break;
        }
    }
    Ok(signed(reaction.needed) < 0)
}

/// Whether block `block` of `kind` stops a beam (`$94:A175`, `$94:A195`).
// open: slopes (`$94:A147`), shootable and bombable air (`$94:9E55`, `$94:9FD6`), the
// extensions (`$94:9411`, `$94:9447`) and bombable blocks (`$94:9FF4`) join with the rooms
// whose beams reach them; until then they let the beam through.
fn react(
    room: &mut RoomScoped,
    block: BlockIndex,
    kind: BlockKind,
) -> Result<bool, ProjectileFault> {
    Ok(match kind {
        BlockKind::Air(_)
        | BlockKind::SpikeAir(_)
        | BlockKind::SpecialAir(_)
        | BlockKind::UnusedAir(_)
        | BlockKind::Slope(_)
        | BlockKind::ShootableAir(_)
        | BlockKind::HorizontalExtension(_)
        | BlockKind::BombableAir(_)
        | BlockKind::VerticalExtension(_)
        | BlockKind::Bombable(_) => false,
        BlockKind::Solid(_)
        | BlockKind::Door(_)
        | BlockKind::Spike(_)
        | BlockKind::Special(_)
        | BlockKind::Grapple(_) => true,
        BlockKind::Shootable(_) => {
            crumble::shoot_block(room, block).map_err(ProjectileFault::Shot)?;
            true
        }
    })
}

/// The room's size in screens across and down (`$07A9`, `$07AB`).
fn screens(room: &RoomScoped) -> (u16, u16) {
    let width = room.width.0;
    let rows = u16::try_from(room.blocks.len())
        .unwrap_or(u16::MAX)
        .checked_div(width)
        .unwrap_or(0);
    (width / SCREEN_BLOCKS, rows / SCREEN_BLOCKS)
}

/// Whether a boundary's screen is short of the room's `screens`.
fn inside_room(boundary: u16, screens: u16) -> bool {
    below(high_byte(boundary), screens)
}

/// Move a beam horizontally and check the blocks its leading edge reached (`$94:A23B`):
/// whether it collided.
fn move_horizontally(
    body: &mut ProjectileBody,
    room: &mut RoomScoped,
) -> Result<bool, ProjectileFault> {
    let count = span(body.position.y.pixels, body.radius.y);
    let row = low_byte(body.position.y.pixels.wrapping_sub(body.radius.y) >> BLOCK_SHIFT);
    let row_start = row.wrapping_mul(low_byte(room.width.0));
    body.position.x = advance(body.position.x, body.velocity.x);
    let x = body.position.x.pixels;
    let boundary = if body.velocity.x.0 < 0 {
        x.wrapping_sub(body.radius.x)
    } else {
        x.wrapping_add(body.radius.x).wrapping_sub(1)
    };
    let start = row_start
        .wrapping_add(boundary >> BLOCK_SHIFT)
        .wrapping_shl(1);
    let (across, _) = screens(room);
    if !below(count, WIDEST_SPAN) || !inside_room(boundary, across) {
        return Ok(false);
    }
    let step = room.width.0.wrapping_mul(2);
    check_blocks(room, start, step, count)
}

/// Move a beam vertically and check the blocks its leading edge reached (`$94:A2CA`):
/// whether it collided.
fn move_vertically(
    body: &mut ProjectileBody,
    room: &mut RoomScoped,
) -> Result<bool, ProjectileFault> {
    let count = span(body.position.x.pixels, body.radius.x);
    body.position.y = advance(body.position.y, body.velocity.y);
    let y = body.position.y.pixels;
    let boundary = if body.velocity.y.0 < 0 {
        y.wrapping_sub(body.radius.y)
    } else {
        y.wrapping_add(body.radius.y).wrapping_sub(1)
    };
    let row_start = low_byte(boundary >> BLOCK_SHIFT).wrapping_mul(low_byte(room.width.0));
    let column = body.position.x.pixels.wrapping_sub(body.radius.x) >> BLOCK_SHIFT;
    let start = row_start.wrapping_add(column).wrapping_shl(1);
    let (_, down) = screens(room);
    if !below(count, WIDEST_SPAN) || !inside_room(boundary, down) {
        return Ok(false);
    }
    check_blocks(room, start, 2, count)
}

/// Kill beam `beam` that collided (`$90:AE06`): it steps to its leading edge and turns into
/// its explosion (`$93:80CF`), whose list shows its first frame as the instruction handler
/// runs this frame.
// open: missiles' explosions and the cooldown cap join with missiles, the hit sound with the
// sound domain.
fn explode(body: &mut ProjectileBody, beam: BeamShot) {
    let (dx, dy) = edge(body.heading.direction);
    let step = |position: Fixed, sign: i8, radius: u16| Fixed {
        pixels: match sign {
            1 => position.pixels.wrapping_add(radius),
            -1 => position.pixels.wrapping_sub(radius),
            _ => position.pixels,
        },
        ..position
    };
    body.position.x = step(body.position.x, dx, body.radius.x);
    body.position.y = step(body.position.y, dy, body.radius.y);
    body.kind = ProjectileType {
        shot: Shot::BeamExplosion {
            beam,
            step: ExplosionStep::default(),
        },
        ..body.kind
    };
    body.frame_timer = Frames(1);
    body.damage.0 = EXPLOSION_DAMAGE;
}

/// Which way a beam steps to its leading edge as it dies, by its direction (`$90:AE3A`).
const fn edge(direction: Direction) -> (i8, i8) {
    match direction {
        Direction::UpFacingRight | Direction::UpFacingLeft => (0, -1),
        Direction::UpRight => (1, -1),
        Direction::Right => (1, 0),
        Direction::DownRight => (1, 1),
        Direction::DownFacingRight | Direction::DownFacingLeft => (0, 1),
        Direction::DownLeft => (-1, 1),
        Direction::Left => (-1, 0),
        Direction::UpLeft => (-1, -1),
    }
}

/// Whether a projectile is near enough the playfield to keep (`$90:B16A`): within 40h pixels
/// of it on both axes, by the sign of each difference as the original tests it.
fn on_screen(body: &ProjectileBody, camera: &Camera) -> bool {
    let near = |position: u16, edge: u16| {
        let offset = position.wrapping_sub(edge);
        !below(offset, MARGIN.wrapping_neg()) && below(offset, REACH)
    };
    near(body.position.x.pixels, camera.x.pixels) && near(body.position.y.pixels, camera.y.pixels)
}

/// Clear slot `index` (`$90:ADB7`): it frees, keeping only its trail timer, and the count
/// of its kind falls, never below zero.
// open: the bomb counter (`$0CD2`) joins with bombs; a bomb slot's clear leaves the shot
// count alone.
fn clear(projectiles: &mut Projectiles, index: usize) {
    let Some(slot) = projectiles.slots.slots.get_mut(index) else {
        return;
    };
    *slot = Slot {
        occupancy: Occupancy::Free,
        ..*slot
    };
    if index < SHOT_SLOTS {
        let ShotCount(count) = projectiles.count;
        let fewer = count.wrapping_sub(1);
        projectiles.count = ShotCount(if signed(fewer) < 0 { 0 } else { fewer });
    }
}

#[cfg(test)]
mod tests;
