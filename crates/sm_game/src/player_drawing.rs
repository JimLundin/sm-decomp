//! Samus's frame drawing on a session lease: the cannon tiles, body, echoes,
//! grapple and projectiles in the original layer order.
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

/// The actual frame protocol, independent of the legacy RAM-shaped sinks.
/// Each sink returns before the next source/selector query. Cannon output and
/// upload publication must bind supplied semantic content on prepared hosts.
pub trait FrameHost: crate::visual_host::Host {
    fn owned_frame_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R;
    fn refresh_player_observations(&mut self);
    fn advance_atmosphere(&mut self);
    fn draw_body(&mut self, mode: Body);
    fn draw_echoes(&mut self);
    fn draw_crash_echoes(&mut self, slot: u16);
    fn cannon_tile(&mut self, x: u16, y: u16, body_y: u16, character: u16);
    fn queue_cannon_tiles(&mut self, tiles: Tiles);
    fn draw_flare(&mut self);
    fn draw_grapple_flare(&mut self);
    fn restore_grapple_start_positions(&mut self);
    fn update_grapple_tiles(&mut self);
    fn draw_grapple_rope(&mut self);
    fn draw_projectiles(&mut self);
    fn finish_player_frame(&mut self);
    fn invalid_draw_mode(&mut self);
}
impl<H: Host> FrameHost for H {
    fn owned_frame_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        crate::lease::Lease::owned_player_call(self, action)
    }
    fn refresh_player_observations(&mut self) {
        crate::lease::Lease::refresh_player_observations(self)
    }
    fn advance_atmosphere(&mut self) {
        crate::player_transitions::draw_atmospheric(self);
    }
    fn draw_body(&mut self, mode: Body) {
        samus_drawing::draw_body(self, mode);
    }
    fn draw_echoes(&mut self) {
        samus_drawing::draw_echoes(self);
    }
    fn draw_crash_echoes(&mut self, slot: u16) {
        samus_drawing::draw_shinespark_crash_echoes(slot, self);
    }
    fn cannon_tile(&mut self, x: u16, y: u16, body_y: u16, character: u16) {
        cannon_tile(self, x, y, body_y, character);
    }
    fn queue_cannon_tiles(&mut self, tiles: Tiles) {
        queue_cannon_tiles(self, tiles);
    }
    fn draw_flare(&mut self) {
        Host::draw_flare(self);
    }
    fn draw_grapple_flare(&mut self) {
        Host::draw_grapple_flare(self);
    }
    fn restore_grapple_start_positions(&mut self) {
        Host::restore_grapple_start_positions(self);
    }
    fn update_grapple_tiles(&mut self) {
        Host::update_grapple_tiles(self);
    }
    fn draw_grapple_rope(&mut self) {
        Host::draw_grapple_rope(self);
    }
    fn draw_projectiles(&mut self) {
        Host::draw_projectiles(self);
    }
    fn finish_player_frame(&mut self) {
        Host::finish_player_frame(self);
    }
    fn invalid_draw_mode(&mut self) {
        crate::lease::Lease::native_unreachable(self);
    }
}
fn frame_cannon_tile<H: FrameHost>(h: &mut H, x: u16, y: u16, body_y: u16, character: u16) {
    h.cannon_tile(x, y, body_y, character);
}
fn frame_queue_cannon_tiles<H: FrameHost>(h: &mut H, tiles: Tiles) {
    h.queue_cannon_tiles(tiles);
}

fn draw_player_with_cannon<H: FrameHost>(runtime: &mut H) {
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    runtime.advance_atmosphere();
    if mode != 0 && mode != 2 {
        cannon::draw(runtime, frame_cannon_tile, frame_queue_cannon_tiles);
    }
    runtime.owned_frame_call(|runtime| runtime.draw_body(Body::Normal));
    if mode == 2 {
        cannon::draw(runtime, frame_cannon_tile, frame_queue_cannon_tiles);
    }
}

/// Queue the active beam's tiles and write its palette.
pub fn update_beam_tiles_and_palette<H: FrameHost>(h: &mut H) {
    cannon::update_beam(h, frame_queue_cannon_tiles);
}

/// Draw Samus's frame for the current draw handler, then the projectiles.
pub fn draw_player_frame<H: FrameHost>(h: &mut H) {
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
                h.owned_frame_call(draw_player_with_cannon);
                h.owned_frame_call(|runtime| runtime.draw_echoes());
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.restore_grapple_start_positions();
                }
                h.draw_grapple_flare();
                h.owned_frame_call(draw_player_with_cannon);
                h.update_grapple_tiles();
                if h.session().player.grapple_length != 0 {
                    h.draw_grapple_rope();
                }
            }
        }
        DrawMode::ShinesparkEnd => {
            h.owned_frame_call(|runtime| runtime.draw_body(Body::Normal));
            for index in [2, 0] {
                h.owned_frame_call(|runtime| runtime.draw_crash_echoes(index));
            }
        }
        DrawMode::Frozen => {
            h.owned_frame_call(|runtime| runtime.draw_body(Body::Frozen));
        }
        DrawMode::Elevator if h.inputs().frame & 1 == 0 => {
            h.owned_frame_call(|runtime| runtime.draw_body(Body::Frozen));
        }
        DrawMode::Disabled | DrawMode::Empty | DrawMode::Elevator => {}
        DrawMode::Unknown(_) => h.invalid_draw_mode(),
    }
    h.draw_projectiles();
    h.finish_player_frame();
}
