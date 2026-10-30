//! Samus's frame drawing on the session Runtime: the cannon tiles, body, echoes,
//! grapple and projectiles in the original layer order.
use crate::game::player_projectiles;
use crate::game::session_runtime::Runtime;
use crate::game::visual_host::{self, Source};
use crate::game_state::Game;

#[forbid(unsafe_code)]
fn queue_cannon_tiles(runtime: &mut Runtime<'_>, tiles: player_projectiles::cannon::Tiles) {
    use player_projectiles::cannon::Tiles;
    use visual_host::Content;
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

#[forbid(unsafe_code)]
fn cannon_tile(runtime: &mut Runtime<'_>, x: u16, y: u16, body_y: u16, character: u16) {
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

fn draw_player_with_cannon(runtime: &mut Runtime<'_>) {
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    crate::game::player_transitions::draw_atmospheric(runtime);
    if mode != 0 && mode != 2 {
        player_projectiles::cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
    runtime.owned_player_call(|runtime| {
        sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Normal)
    });
    if mode == 2 {
        player_projectiles::cannon::draw(runtime, cannon_tile, queue_cannon_tiles);
    }
}

pub(crate) fn update_beam_tiles_and_palette(game: &mut Game) {
    player_projectiles::cannon::update_beam(&mut Runtime::new(game), queue_cannon_tiles);
}

pub(crate) fn draw_player_frame(game: &mut Game) {
    Runtime::new(game).owned_player_call(draw_player_frame_owned);
}

pub(crate) fn draw_player_frame_owned(h: &mut Runtime<'_>) {
    use crate::game::player_transitions::state::DrawMode;
    player_projectiles::cannon::update(h);
    h.refresh_player_observations();
    match h.session().player.transitions.draw_handler {
        DrawMode::Normal | DrawMode::Grapple => {
            let mode = h.session().player.transitions.draw_handler;
            if mode == DrawMode::Normal {
                h.foreign(player_projectiles::flare);
            }
            let grapple = h.session().player.grapple;
            if mode == DrawMode::Normal || (grapple.wrapping_add(0x37aa) as i16) >= 0 {
                h.owned_player_call(draw_player_with_cannon);
                h.owned_player_call(|runtime| sm_game::samus_drawing::draw_echoes(runtime));
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.foreign(crate::game::grapple_swing::restore_start_positions);
                }
                h.foreign(player_projectiles::grapple_flare);
                h.owned_player_call(draw_player_with_cannon);
                h.foreign(crate::game::grapple_swing::update_tiles);
                if h.session().player.grapple_length != 0 {
                    h.foreign(crate::game::grapple_swing::draw_rope);
                }
            }
        }
        DrawMode::ShinesparkEnd => {
            h.owned_player_call(|runtime| {
                sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Normal)
            });
            for index in [2, 0] {
                h.owned_player_call(|runtime| {
                    sm_game::samus_drawing::draw_shinespark_crash_echoes(index, runtime)
                });
            }
        }
        DrawMode::Frozen => {
            h.owned_player_call(|runtime| {
                sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Frozen)
            });
        }
        DrawMode::Elevator if h.inputs().frame & 1 == 0 => {
            h.owned_player_call(|runtime| {
                sm_game::samus_drawing::draw_body(runtime, sm_game::scene::samus_body::Body::Frozen)
            });
        }
        DrawMode::Disabled | DrawMode::Empty | DrawMode::Elevator => {}
        DrawMode::Unknown(_) => {
            h.foreign(|game| crate::rtl::Unreachable(game));
        }
    }
    h.foreign(player_projectiles::draw_projectiles);
    h.owned_call(crate::game::session_runtime::finish_player_frame);
}
