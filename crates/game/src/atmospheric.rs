//! The atmospheric graphics: their animation and drawing each frame, and the pose
//! initialisations that start them.
//!
//! Bank `$90` log, handle atmospheric effects `$90:8A4C`: from the last slot to the first,
//! each live graphic counts its timer down, moves on to its next frame when it runs out and
//! ends after its last; a graphic still in a frame moves as its kind does and is drawn,
//! either as one sprite (`$90:8AC5`) or as one of Samus's spritemaps (`$90:8B74`).

use data::pose::MovementType;
use data::pose::spritemap::{SamusSpritemaps, SpritemapIndex};
use data::spritemap::SpriteAttributes;

use crate::pose::Content;
use crate::state::State;
use crate::state::atmospheric::{
    AtmosphericFrame, AtmosphericGraphic, AtmosphericKind, AtmosphericTimer, Atmospherics,
    LANDING_SLOT, TimerStep,
};
use crate::state::samus::{Facing, PoseRecord, Samus};
use crate::state::video::NmiBatch;

/// How a kind looks.
#[derive(Clone, Copy, Debug)]
enum Look {
    /// One sprite, its tile and attributes by frame (`$90:8BFF`).
    Sprite([SpriteAttributes; 4]),
    /// The Samus spritemap whose index is the first plus the frame.
    SamusSpritemaps(SpritemapIndex),
}

/// How a kind moves each frame it is drawn.
#[derive(Clone, Copy, Debug)]
enum Motion {
    /// It stays where it started.
    Still,
    /// A pixel up (`$90:8B57`).
    Rises,
    /// A pixel up and a pixel outward: slots 0 and 1 rightward, 2 and 3 leftward
    /// (`$90:8B2E`).
    Spreads,
    /// On the liquid's surface (`$90:8B16`).
    // open: the FX Y position isn't in the State yet (it joins with FX); nothing spawns the
    // splash until then, so the graphic keeps the Y it has.
    OnLiquidSurface,
}

/// A kind's behaviour: its frame timers (`$90:8B93`, as many as its frames, `$90:8BEF`), its
/// look and its motion.
#[derive(Clone, Copy, Debug)]
struct Behaviour {
    timers: &'static [u16],
    look: Look,
    motion: Motion,
}

/// The splash and dust sprites: four tiles from `first_tile`, palette 5, priority 2
/// (`$90:8C0F`, `$90:8C17`).
const fn sprites(first_tile: u16) -> [SpriteAttributes; 4] {
    [
        sprite(first_tile),
        sprite(first_tile.wrapping_add(1)),
        sprite(first_tile.wrapping_add(2)),
        sprite(first_tile.wrapping_add(3)),
    ]
}

/// One splash or dust sprite.
const fn sprite(tile: u16) -> SpriteAttributes {
    SpriteAttributes {
        tile,
        palette: 5,
        priority: 2,
        flip_x: false,
        flip_y: false,
    }
}

const SPLASH_SPRITES: [SpriteAttributes; 4] = sprites(0x2C);
const DUST_SPRITES: [SpriteAttributes; 4] = sprites(0x48);
/// The diving splash's first Samus spritemap (`$90:8B25`).
const DIVING_SPLASH_SPRITEMAPS: SpritemapIndex = SpritemapIndex(0x18F);
/// The bubbles' first Samus spritemap (`$90:8B6D`).
const BUBBLE_SPRITEMAPS: SpritemapIndex = SpritemapIndex(0x186);
const SPLASH_TIMERS: [u16; 4] = [3, 3, 3, 3];
const DUST_TIMERS: [u16; 4] = [3, 4, 5, 6];

/// Each kind's behaviour (`$90:8AB5`'s handlers and their tables).
const fn behaviour(kind: AtmosphericKind) -> Behaviour {
    match kind {
        AtmosphericKind::FootstepSplash => Behaviour {
            timers: &SPLASH_TIMERS,
            look: Look::Sprite(SPLASH_SPRITES),
            motion: Motion::Still,
        },
        AtmosphericKind::DivingSplash => Behaviour {
            timers: &[2, 2, 3, 3, 3, 5, 5, 6, 7],
            look: Look::SamusSpritemaps(DIVING_SPLASH_SPRITEMAPS),
            motion: Motion::OnLiquidSurface,
        },
        AtmosphericKind::LavaSurfaceDamage => Behaviour {
            timers: &[2, 2, 2, 2],
            look: Look::Sprite(DUST_SPRITES),
            motion: Motion::Spreads,
        },
        AtmosphericKind::Bubbles => Behaviour {
            timers: &[5; 8],
            look: Look::SamusSpritemaps(BUBBLE_SPRITEMAPS),
            motion: Motion::Still,
        },
        AtmosphericKind::LandingDust | AtmosphericKind::SpeedBoostDust => Behaviour {
            timers: &DUST_TIMERS,
            look: Look::Sprite(DUST_SPRITES),
            motion: Motion::Rises,
        },
    }
}

/// The slots that spread leftward (`$90:8B2F` tests the slot's word index for 4).
const FIRST_LEFTWARD_SLOT: usize = 2;

/// How far up and left of its position a single sprite's corner sits (`$90:8AD7`).
const SPRITE_CORNER: u16 = 4;

/// Animate and draw the atmospheric graphics (`$90:8A4C`), for the drawing handlers that
/// draw them before Samus.
pub fn handle(state: &mut State, content: &Content) {
    let camera = (state.room.camera.x.pixels, state.room.camera.y.pixels);
    let batch = &mut state.video.batch;
    let slots = state.atmospherics.slots.iter_mut().enumerate().rev();
    for (slot, place) in slots {
        let Some(graphic) = place else {
            continue;
        };
        let behaviour = behaviour(graphic.kind);
        let timer = |frame: AtmosphericFrame| {
            AtmosphericTimer(
                behaviour
                    .timers
                    .get(usize::from(frame.0))
                    .copied()
                    .unwrap_or_default(),
            )
        };
        match graphic.timer.count_down() {
            TimerStep::Wait => continue,
            TimerStep::Draw => {}
            TimerStep::Release => graphic.timer = timer(graphic.frame),
            TimerStep::Advance => {
                // The finished frame's entry times the next one.
                graphic.timer = timer(graphic.frame);
                graphic.frame = AtmosphericFrame(graphic.frame.0.wrapping_add(1));
                if usize::from(graphic.frame.0) >= behaviour.timers.len() {
                    *place = None;
                    continue;
                }
            }
        }
        step(graphic, slot, behaviour.motion);
        draw(
            batch,
            *graphic,
            behaviour.look,
            camera,
            &content.samus_spritemaps,
        );
    }
}

/// Move a graphic as its kind does.
const fn step(graphic: &mut AtmosphericGraphic, slot: usize, motion: Motion) {
    match motion {
        Motion::Still | Motion::OnLiquidSurface => {}
        Motion::Rises => graphic.y = graphic.y.wrapping_sub(1),
        Motion::Spreads => {
            graphic.x = if slot < FIRST_LEFTWARD_SLOT {
                graphic.x.wrapping_add(1)
            } else {
                graphic.x.wrapping_sub(1)
            };
            graphic.y = graphic.y.wrapping_sub(1);
        }
    }
}

/// Draw a graphic's frame.
fn draw(
    batch: &mut NmiBatch,
    graphic: AtmosphericGraphic,
    look: Look,
    (camera_x, camera_y): (u16, u16),
    maps: &SamusSpritemaps,
) {
    let x = graphic.x.wrapping_sub(camera_x);
    let y = graphic.y.wrapping_sub(camera_y);
    let frame = usize::from(graphic.frame.0);
    match look {
        // `$90:8AC5`: the sprite is dropped unless its corner is on the 256-pixel screen both
        // ways; it takes the low table's four bytes only.
        Look::Sprite(sprites) => {
            let corner = |at: u16| u8::try_from(at.wrapping_sub(SPRITE_CORNER)).ok();
            if let (Some(x), Some(y), Some(&attributes)) =
                (corner(x), corner(y), sprites.get(frame))
            {
                batch.add_low_sprite(u16::from(x), y, attributes);
            }
        }
        // `$90:8B74`: dropped unless its Y is on the screen; the X may be anywhere.
        Look::SamusSpritemaps(first) => {
            let index = SpritemapIndex(first.0.wrapping_add(u16::from(graphic.frame.0)));
            if let (Ok(y), Some(map)) = (u8::try_from(y), maps.spritemap(index)) {
                batch.add_spritemap(map, (x, u16::from(y)));
            }
        }
    }
}

/// How far to the side of her position the wall-jump dust starts (`$91:FAB6`).
const WALL_JUMP_DUST_OFFSET: u16 = 6;
/// The wall-jump dust's first frame's timer (`$91:FA9C`).
const WALL_JUMP_DUST_TIMER: AtmosphericTimer = AtmosphericTimer(3);

/// The atmospheric graphics the pose initialisation (`$91:F433`) starts for a new pose.
///
/// Called after the pose change with the pose history it found: a history that now records
/// a different pose means the change initialised the pose it took.
// open: `pose::change` doesn't yet say when it initialised a pose, so this reads it from
// the history it records; the landing graphics (`$91:F0A3`) join with that report.
pub fn initialise_pose(
    atmospherics: &mut Atmospherics,
    before: PoseRecord,
    samus: &Samus,
    content: &Content,
) {
    if samus.previous.pose == before.pose {
        return;
    }
    let Some(definition) = content.poses.get(samus.pose) else {
        return;
    };
    if definition.movement == MovementType::WallJumping {
        wall_jump_dust(atmospherics, samus, definition.y_radius);
    }
}

/// "Initialise Samus pose - wall jumping" (`$91:FA76`): dust at her feet, on the side away
/// from the way she faces, in the landing slot.
// open: the dust is skipped when her feet are in liquid with liquid physics (`$91:FA7A`);
// that joins with FX, and slice 0's rooms have no liquid.
fn wall_jump_dust(atmospherics: &mut Atmospherics, samus: &Samus, y_radius: u8) {
    let at = samus.motion.position.x.pixel();
    let x = match samus.facing {
        Facing::Right => at.wrapping_sub(WALL_JUMP_DUST_OFFSET),
        Facing::Left | Facing::Forward | Facing::Unknown(_) => {
            at.wrapping_add(WALL_JUMP_DUST_OFFSET)
        }
    };
    // `$90:EC3E`: her bottom boundary, the last pixel row she covers.
    let y = samus
        .motion
        .position
        .y
        .pixel()
        .wrapping_add(u16::from(y_radius))
        .wrapping_sub(1);
    if let Some(slot) = atmospherics.slots.get_mut(LANDING_SLOT) {
        *slot = Some(AtmosphericGraphic {
            kind: AtmosphericKind::LandingDust,
            frame: AtmosphericFrame(0),
            timer: WALL_JUMP_DUST_TIMER,
            x,
            y,
        });
    }
}

#[cfg(test)]
mod tests;
