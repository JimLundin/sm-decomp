//! Samus's frame drawing on a session lease: the cannon tiles, body, echoes,
//! grapple and projectiles in the original layer order.
#![forbid(unsafe_code)]
use crate::player_projectiles::cannon::{self, Tiles};
use crate::player_transitions::state::DrawMode;
use crate::runtime_access::SessionRam;
use crate::samus_drawing;
use crate::scene::samus_body::Body;
use crate::visual_host::Source;

/// The session lease the frame draws on; the effects below leave the lease,
/// export the Session to the compatibility host, and resume.
pub trait Host:
    SessionRam + crate::visual_host::Host + samus_drawing::Authority + crate::lease::Lease
{
    /// The charge flare and grapple flare, on the compatibility host.
    fn draw_flare(&mut self);
    fn draw_grapple_flare(&mut self);
    /// The grapple's start positions, tiles and rope, on the compatibility host.
    fn restore_grapple_start_positions(&mut self);
    fn update_grapple_tiles(&mut self);
    fn draw_grapple_rope(&mut self);
    /// The player projectiles, on the compatibility host.
    fn draw_projectiles(&mut self);
    /// Publish the finished player frame.
    fn finish_player_frame(&mut self);
}

fn queue_cannon_tiles<H: Host>(runtime: &mut H, tiles: Tiles) {
    let (size, destination) = match tiles {
        Tiles::Beam(_) => (256, 0x6300),
        Tiles::Cannon(_) => (32, 0x61f0),
    };
    let saved = runtime.word(0x330);
    runtime.write_slice_native(0xd0 + usize::from(saved), size);
    // Size publication precedes source acquisition and every source read.
    let source = match tiles {
        Tiles::Beam(index) => runtime
            .table(Source::Loaded, 0x90c3b1)
            .word(usize::from(index) * 2),
        Tiles::Cannon(pointer) => runtime.legacy_native_record(0x90, pointer).word(0),
    };
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(2)), source);
    runtime.set_byte(0xd0 + usize::from(saved.wrapping_add(4)), 0x9a);
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(5)), destination);
    runtime.write_word_le(saved.wrapping_add(7), 0x330);
}

fn cannon_tile<H: Host>(runtime: &mut H, x: u16, y: u16, body_y: u16, character: u16) {
    let saved = runtime.word(0x590);
    let x = x
        .wrapping_add(runtime.session().player.x)
        .wrapping_sub(runtime.word(0x911));
    if x < 256 {
        let entry = 0x370 + usize::from(runtime.word(0x590));
        runtime.set_byte(entry, x as u8);
        let y = y
            .wrapping_add(runtime.session().player.y)
            .wrapping_sub(body_y)
            .wrapping_sub(runtime.word(0x915));
        if y < 256 {
            runtime.set_byte(entry + 1, y as u8);
            runtime.write_slice_native(entry + 2, character);
            runtime.write_word_le(saved.wrapping_add(4), 0x590);
        }
    }
}

fn draw_player_with_cannon<H: Host>(runtime: &mut H) {
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    crate::player_transitions::draw_atmospheric(runtime);
    if mode != 0 && mode != 2 {
        cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
    runtime.owned_player_call(|runtime| samus_drawing::draw_body(runtime, Body::Normal));
    if mode == 2 {
        cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
}

/// Queue the active beam's tiles and write its palette.
pub fn update_beam_tiles_and_palette<H: Host>(h: &mut H) {
    cannon::update_beam(h, queue_cannon_tiles);
}

/// Draw Samus's frame for the current draw handler, then the projectiles.
pub fn draw_player_frame<H: Host>(h: &mut H) {
    cannon::update(h);
    h.refresh_player_observations();
    match h.session().player.transitions.draw_handler {
        DrawMode::Normal | DrawMode::Grapple => {
            let mode = h.session().player.transitions.draw_handler;
            if mode == DrawMode::Normal {
                h.draw_flare();
            }
            let grapple = h.session().player.grapple;
            if mode == DrawMode::Normal || (grapple.wrapping_add(0x37aa) as i16) >= 0 {
                h.owned_player_call(draw_player_with_cannon);
                h.owned_player_call(|runtime| samus_drawing::draw_echoes(runtime));
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.restore_grapple_start_positions();
                }
                h.draw_grapple_flare();
                h.owned_player_call(draw_player_with_cannon);
                h.update_grapple_tiles();
                if h.session().player.grapple_length != 0 {
                    h.draw_grapple_rope();
                }
            }
        }
        DrawMode::ShinesparkEnd => {
            h.owned_player_call(|runtime| samus_drawing::draw_body(runtime, Body::Normal));
            for index in [2, 0] {
                h.owned_player_call(|runtime| {
                    samus_drawing::draw_shinespark_crash_echoes(index, runtime)
                });
            }
        }
        DrawMode::Frozen => {
            h.owned_player_call(|runtime| samus_drawing::draw_body(runtime, Body::Frozen));
        }
        DrawMode::Elevator if h.inputs().frame & 1 == 0 => {
            h.owned_player_call(|runtime| samus_drawing::draw_body(runtime, Body::Frozen));
        }
        DrawMode::Disabled | DrawMode::Empty | DrawMode::Elevator => {}
        DrawMode::Unknown(_) => h.native_unreachable(),
    }
    h.draw_projectiles();
    h.finish_player_frame();
}
