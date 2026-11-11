//! Drawing Samus: her clip's frame for her pose and animation frame (`data::assets::samus`),
//! placed from her position on screen, into the sprite buffer.
//!
//! Bank `$90` log (<https://patrickjohnston.org/bank/90>), draw Samus `$90:85E2`: unless she
//! flickers this frame, her halves are drawn. Where they sit and which a frame draws are the
//! clip's, baked by the importer from the original's rules (docs/architecture/assets.md).
//! Bank `$A0` log, `$A0:884D` draws her with enemy layer 3, after the enemies of layers 0 to 2.

use clean_types::Pair;
use data::assets::samus::Frame;
use data::assets::samus::import::map;

use crate::intent::{SpriteDraw, on_screen};
use crate::pose::Content;
use crate::state::State;
use crate::state::samus::{DrawingHandler, Samus};

/// Whether she is hidden this frame: invincible and flickering on an odd frame.
///
/// `$90:85E2` draws her whenever her knockback timer runs, she isn't invincible or her
/// special palette timer runs; otherwise only on even frames of the frame counter.
// open: the knockback timer and the special palette timer join with damage and the speed
// booster; until then both are zero, as in slice 0's rows.
const fn flickers_off(samus: &Samus, odd_frame: bool) -> bool {
    samus.invincibility.0 != 0 && odd_frame
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
        // `$90:EB55` and `$90:EB86`: the atmospheric graphics go first.
        DrawingHandler::Default | DrawingHandler::FiringGrapple => {
            crate::atmospheric::handle(state, content);
            if !flickers_off(&state.samus, odd_frame) {
                draw_body(state, content);
            }
            queue_tiles(state, content);
        }
        DrawingHandler::ShinesparkCrashEchoCircle => {
            if !flickers_off(&state.samus, odd_frame) {
                draw_body(state, content);
            }
            queue_tiles(state, content);
        }
        // `$90:EC14`: on odd frames of the frame counter the ride leaves her out.
        DrawingHandler::UsingElevator if odd_frame => {}
        // `$90:EC1D`, draw inanimate Samus `$90:8A00`: `$90:85E2` without the flicker.
        DrawingHandler::UsingElevator | DrawingHandler::Inanimate => {
            draw_body(state, content);
            queue_tiles(state, content);
        }
        DrawingHandler::Hidden => {}
    }
}

/// Bank `$92` log, set Samus tiles definitions for current animation `$92:8000`, which
/// `$90:85E2` (flickering or not) and `$90:8A00` end with: her pose and frame's top half
/// tiles, and her bottom half's when the frame has them, queue for the NMI's upload
/// (`$80:9376`).
///
/// A frame past the pose's decoded frames queues nothing; the original would read the bytes
/// after its list, and no animation reaches one.
fn queue_tiles(state: &mut State, content: &Content) {
    let samus = &state.samus;
    let Some(frame) = content.samus_tiles.frame(samus.pose, samus.animation.frame) else {
        return;
    };
    let queued = &mut state.video.batch.samus_tiles;
    queued.top = Some(frame.top);
    if let Some(bottom) = frame.bottom {
        queued.bottom = Some(bottom);
    }
}

/// The frame of her clip she is drawn in: her pose's clip in the dress its id shows, or a
/// pose outside her domain's frames; `None` past them.
fn clip_frame<'a>(samus: &Samus, content: &'a Content) -> Option<&'a Frame> {
    let assets = &content.samus_assets;
    let frames = match map::shown(samus.pose) {
        Some((pose, dress)) => &assets.clip(pose, dress)?.frames,
        None => &assets.unkeyed(samus.pose)?.frames,
    };
    frames.get(usize::from(samus.animation.frame.0))
}

/// Draw Samus's body into the sprite buffer.
fn draw_body(state: &mut State, content: &Content) {
    let body = body(state, content);
    state.video.batch.draw_sprites(&body);
}

/// Samus's body from her clip's frame: its top half, the chest sprite where it has one, then
/// its bottom half, placed from her position on screen moved by the frame's origin
/// (`$90:85E2` past the flicker, `$90:8A00`). Nothing past her clip's frames.
// open: the half indices the original keeps (`$0AC8`, `$0ACA`, which `$90:8A00` leaves
// alone) and the body position (`$0B04`, `$0B06`) join the State when a reader needs them.
fn body(state: &State, content: &Content) -> SpriteDraw {
    let mut body = SpriteDraw::default();
    let Some(frame) = clip_frame(&state.samus, content) else {
        return body;
    };
    let assets = &content.samus_assets;
    let position = state.samus.motion.position;
    let at = Pair {
        x: on_screen(position.x.pixel(), state.room.camera.x.pixels),
        y: on_screen(position.y.pixel(), state.room.camera.y.pixels),
    };
    let origin = Pair {
        y: at.y.wrapping_add(i32::from(frame.origin_y)),
        ..at
    };
    if let Some(top) = frame.top.and_then(|id| assets.sheet.sprite(id)) {
        body.place(top, origin);
    }
    if frame.chest {
        let chest = assets.chest;
        let corner = Pair {
            x: at.x.wrapping_sub(i32::from(chest.left)),
            y: at.y.wrapping_sub(i32::from(chest.up)),
        };
        body.lone(corner, chest.attributes);
    }
    if let Some(bottom) = frame.bottom.and_then(|id| assets.sheet.sprite(id)) {
        body.place(bottom, origin);
    }
    body
}

#[cfg(test)]
mod tests;
