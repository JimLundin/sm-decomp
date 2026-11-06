//! Drawing Samus: her top and bottom halves' spritemaps for her pose and animation frame,
//! placed from her position on screen, into the sprite buffer.
//!
//! Bank `$90` log (<https://patrickjohnston.org/bank/90>), draw Samus `$90:85E2`: unless she
//! flickers this frame, the top half's spritemap is drawn from the position calculate
//! Samus spritemap position `$90:8C1F` gives, then, if her movement type's handler
//! (`$90:864E`) says so, the bottom half's from the same position. Bank `$A0` log, `$A0:884D`
//! draws her with enemy layer 3, after the enemies of layers 0 to 2.

use data::pose::animation::AnimationFrame;
use data::pose::spritemap::{Half, PositionOffsets};
use data::pose::{MovementType, PoseId};
use data::spritemap::SpriteAttributes;

use crate::pose::Content;
use crate::state::State;
use crate::state::samus::{DrawingHandler, Samus};
use crate::state::video::NmiBatch;

/// The facing-forward poses, power suit and the other suits.
const FACING_FORWARD: PoseId = PoseId(0x00);
const FACING_FORWARD_SUITED: PoseId = PoseId(0x9B);
/// The first landing pose; landing runs to pose $A7.
const FIRST_LANDING: u16 = 0xA4;
/// The first crouching, standing, morphing or unmorphing transition pose; they run to $40.
const FIRST_TRANSITION: u16 = 0x35;

/// The chest sprite the power suit's facing-forward pose adds before her bottom half
/// (`$90:868D`): tile $21, palette 4, priority 3.
const CHEST: SpriteAttributes = SpriteAttributes::from_word(0x3821);
/// Where the chest sprite sits from her position: 7 pixels left, 17 up.
const CHEST_OFFSET: (u16, u16) = (7, 17);

/// A frame number the original compares as a signed word.
const fn signed(frame: AnimationFrame) -> i16 {
    i16::from_le_bytes(frame.0.to_le_bytes())
}

/// Whether she is hidden this frame: invincible and flickering on an odd frame.
///
/// `$90:85E2` draws her whenever her knockback timer runs, she isn't invincible or her
/// special palette timer runs; otherwise only on even frames of the frame counter.
// open: the knockback timer and the special palette timer join with damage and the speed
// booster; until then both are zero, as in slice 0's rows.
const fn flickers_off(samus: &Samus, odd_frame: bool) -> bool {
    samus.invincibility.0 != 0 && odd_frame
}

/// The spritemaps' origin on screen: her position less the camera's, moved up by the
/// pose's graphics offset or, for some poses and frames, by their own tables.
///
/// `$90:8C1F` dispatches on the movement type (`$90:8C5C`): standing `$90:8CC3`, the
/// transitions `$90:8D3C`, shinespark and drained `$90:8D98`, the others `$90:8C94`.
// open: the rotating Ceres elevator room's projected position (`$90:8C1F` with `$093F`
// negative) joins with Ceres.
fn origin(
    samus: &Samus,
    camera: (u16, u16),
    movement: MovementType,
    y_offset: i8,
    offsets: &PositionOffsets,
) -> (u16, u16) {
    let pose = samus.pose;
    let frame = samus.animation.frame;
    let at = usize::from(frame.0);
    // Each case's amount added to her Y.
    let usual = i16::from(y_offset).wrapping_neg();
    let down = match movement {
        MovementType::Standing if pose == FACING_FORWARD || pose == FACING_FORWARD_SUITED => {
            if signed(frame) >= 2 {
                -1
            } else {
                usual
            }
        }
        MovementType::Standing if (FIRST_LANDING..FIRST_LANDING + 4).contains(&pose.0) => {
            let row = usize::from(pose.0.wrapping_sub(FIRST_LANDING)) * 4;
            offsets
                .landing
                .get(row.wrapping_add(at))
                .map_or(usual, |&up| i16::from(up).wrapping_neg())
        }
        MovementType::CrouchingTransition
            if (FIRST_TRANSITION..=FIRST_TRANSITION + 0xB).contains(&pose.0) =>
        {
            let row = usize::from(pose.0.wrapping_sub(FIRST_TRANSITION)) * 2;
            offsets
                .transition
                .get(row.wrapping_add(at))
                .map_or(usual, |&down| i16::from(down))
        }
        MovementType::Shinespark if matches!(pose.0, 0xE8 | 0xE9) => offsets
            .drained
            .get(at)
            .map_or(usual, |&down| i16::from(down)),
        MovementType::Shinespark if matches!(pose.0, 0xEA | 0xEB) && signed(frame) >= 5 => -3,
        MovementType::Standing
        | MovementType::Running
        | MovementType::NormalJumping
        | MovementType::SpinJumping
        | MovementType::MorphBallOnGround
        | MovementType::Crouching
        | MovementType::Falling
        | MovementType::Unused07
        | MovementType::MorphBallFalling
        | MovementType::Unused09
        | MovementType::KnockbackOrCrystalFlashEnding
        | MovementType::Unused0B
        | MovementType::Unused0C
        | MovementType::Unused0D
        | MovementType::TurningAroundOnGround
        | MovementType::CrouchingTransition
        | MovementType::Moonwalking
        | MovementType::SpringBallOnGround
        | MovementType::SpringBallInAir
        | MovementType::SpringBallFalling
        | MovementType::WallJumping
        | MovementType::RanIntoWall
        | MovementType::Grappling
        | MovementType::TurningAroundJumping
        | MovementType::TurningAroundFalling
        | MovementType::DamageBoost
        | MovementType::GrabbedByDraygon
        | MovementType::Shinespark => usual,
    };
    let (camera_x, camera_y) = camera;
    let x = samus.position.x.pixels.wrapping_sub(camera_x);
    let y = samus
        .position
        .y
        .pixels
        .wrapping_add_signed(down)
        .wrapping_sub(camera_y);
    (x, y)
}

/// What her movement type's bottom-half handler does (`$90:864E`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BottomHalf {
    /// The bottom half is drawn.
    Drawn,
    /// The chest sprite, then the bottom half (`$90:868D`, the power suit facing forward).
    WithChest,
    /// No bottom half: the spin, the morph ball and some frames of other poses.
    Hidden,
}

/// The bottom-half handler of her movement type for her pose and frame.
fn bottom_half(movement: MovementType, pose: PoseId, frame: AnimationFrame) -> BottomHalf {
    let shown = |drawn: bool| {
        if drawn {
            BottomHalf::Drawn
        } else {
            BottomHalf::Hidden
        }
    };
    let frame_number = signed(frame);
    match movement {
        MovementType::Standing if pose == FACING_FORWARD => BottomHalf::WithChest,
        MovementType::Standing
        | MovementType::Running
        | MovementType::NormalJumping
        | MovementType::Crouching
        | MovementType::Falling
        | MovementType::Unused0B
        | MovementType::Unused0C
        | MovementType::TurningAroundOnGround
        | MovementType::Moonwalking
        | MovementType::RanIntoWall
        | MovementType::Grappling
        | MovementType::TurningAroundJumping
        | MovementType::TurningAroundFalling
        | MovementType::GrabbedByDraygon => BottomHalf::Drawn,
        MovementType::MorphBallOnGround
        | MovementType::Unused07
        | MovementType::MorphBallFalling
        | MovementType::Unused09
        | MovementType::SpringBallOnGround
        | MovementType::SpringBallInAir
        | MovementType::SpringBallFalling => BottomHalf::Hidden,
        // `$90:86C6`: the spin's first frame, its wind-down, and the poses that don't spin.
        MovementType::SpinJumping => shown(
            matches!(pose.0, 0x81 | 0x82 | 0x1B | 0x1C) || frame_number == 0 || frame_number >= 11,
        ),
        // `$90:86EE`.
        MovementType::KnockbackOrCrystalFlashEnding => {
            shown(!matches!(pose.0, 0xD7 | 0xD8) || frame_number >= 3)
        }
        // `$90:874C`.
        MovementType::Unused0D => shown(!matches!(pose.0, 0x65 | 0x66) || frame_number < 1),
        // `$90:870C`: of the transitions, only crouching and standing keep a bottom half.
        MovementType::CrouchingTransition => shown(if pose.0 >= 0xF1 {
            true
        } else if pose.0 >= 0xDB {
            frame_number == if pose.0 < 0xDD { 0 } else { 2 }
        } else {
            matches!(pose.0, 0x35 | 0x36 | 0x3B | 0x3C)
        }),
        // `$90:8768`: not while she spins off the wall.
        MovementType::WallJumping => shown(!(3..13).contains(&frame_number)),
        // `$90:877C`.
        MovementType::DamageBoost => shown(!(2..9).contains(&frame_number)),
        // `$90:8790`.
        MovementType::Shinespark => shown(if pose.0 < 0xCF {
            !matches!(pose.0, 0xCB | 0xCC)
        } else {
            !matches!(pose.0, 0xE8 | 0xE9) || frame_number >= 2
        }),
    }
}

/// Draw Samus into the sprite buffer through her drawing handler, for the Draw gameplay
/// phase (`$90:EB4B`).
// open: the arm cannon (`$90:C663`, drawn only once it opens; `$90:C5C4` before the
// handler), the charge flare (`$90:BAFC`), the speed echoes (`$90:87BD`), the grapple
// beam's flare, tiles and beam (`$9B:C036`, `$9B:BFA5`, `$94:AFBA`) and the shinespark
// crash's echoes (`$90:88BA`) join with those lanes; slice 0's power beam keeps the cannon
// closed and those handlers draw her body alone.
pub fn draw(state: &mut State, content: &Content) {
    let odd_frame = state.clock.counters.completed.0 & 1 != 0;
    match state.samus.drawing {
        DrawingHandler::Default
        | DrawingHandler::FiringGrapple
        | DrawingHandler::ShinesparkCrashEchoCircle => {
            if !flickers_off(&state.samus, odd_frame) {
                draw_body(state, content);
            }
        }
        // `$90:EC14`: on odd frames of the frame counter the ride leaves her out.
        DrawingHandler::UsingElevator if odd_frame => {}
        // `$90:EC1D`, draw inanimate Samus `$90:8A00`: `$90:85E2` without the flicker.
        DrawingHandler::UsingElevator | DrawingHandler::Inanimate => draw_body(state, content),
        DrawingHandler::Hidden => {}
    }
}

/// Draw Samus's body: her top half, then her bottom half if her movement type's handler
/// says so (`$90:85E2` past the flicker, `$90:8A00`).
// open: the half indices the original keeps (`$0AC8`, `$0ACA`, which `$90:8A00` leaves
// alone) and the body position (`$0B04`, `$0B06`) join the State when a reader needs them;
// the tile transfers (`$92:8000`) are C6b.
fn draw_body(state: &mut State, content: &Content) {
    let samus = &state.samus;
    let Some(definition) = content.poses.get(samus.pose) else {
        return;
    };
    let maps = &content.samus_spritemaps;
    let camera = (state.room.camera.x.pixels, state.room.camera.y.pixels);
    let at = origin(
        samus,
        camera,
        definition.movement,
        definition.y_offset,
        &maps.offsets,
    );
    let frame = samus.animation.frame;
    let batch = &mut state.video.batch;
    let draw_half = |batch: &mut NmiBatch, half| {
        if let Some(map) = maps
            .index(half, samus.pose, frame)
            .and_then(|index| maps.spritemap(index))
        {
            batch.add_spritemap(map, at);
        }
    };
    draw_half(batch, Half::Top);
    match bottom_half(definition.movement, samus.pose, frame) {
        BottomHalf::Hidden => {}
        BottomHalf::Drawn => draw_half(batch, Half::Bottom),
        BottomHalf::WithChest => {
            let (camera_x, camera_y) = camera;
            let (left, up) = CHEST_OFFSET;
            let x = samus
                .position
                .x
                .pixels
                .wrapping_sub(left)
                .wrapping_sub(camera_x);
            let y = samus
                .position
                .y
                .pixels
                .wrapping_sub(up)
                .wrapping_sub(camera_y);
            let [y_low, _] = y.to_le_bytes();
            batch.add_low_sprite(x, y_low, CHEST);
            draw_half(batch, Half::Bottom);
        }
    }
}

#[cfg(test)]
mod tests;
