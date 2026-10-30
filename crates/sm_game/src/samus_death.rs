//! Samus's death sequence: the frozen pose, the death pose, the suit flash
//! and the suit explosion, with their palettes and tile uploads.
#![forbid(unsafe_code)]
use crate::constants::buttons::{kButton_A, kButton_L, kButton_R};
use crate::menus::prelude::CoroutineRet;
use crate::player_transitions as pt;
use crate::ram::{RawRam, word};
use crate::runtime_access::RuntimeAccess;
use crate::scene::samus_body::Body;
use crate::sprite_output;
use crate::visual_host::{Content, Record, Source};
use crate::word_transfers;

/// The death sequence draws Samus through the host, which chooses between
/// the owned display and the native body tables.
pub trait Host: crate::menus::host::Host + crate::display_refresh::Host + RuntimeAccess {
    fn draw_samus(&mut self, body: Body);
}

pub fn advance_death_pose<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    pt::animation::advance_animation(&mut game.runtime(), pt::state::AnimationStep::Death);
    game.draw_samus(Body::Death);
    let expired = word!(game.g_word_7E0DE8 -= 1, reread) & 0x8000 != 0;
    if word!(game.g_word_7E0DE8) == 0 || expired {
        word!(game.game_state += 1);
    }
    0
}

// Death phases share palette selection and the native upload queue with live drawing.
pub fn begin_death<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    game.draw_samus(Body::Frozen);
    if game.has_queued_music() == 0 {
        const FRAMES: [u8; 28] = [
            5, 5, 5, 5, 1, 5, 5, 0, 1, 0, 5, 5, 5, 5, 5, 5, 5, 1, 1, 1, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        let movement = game.ram().read_byte(0xa1f);
        if game.ram().read_byte(0xa1f) == 3 {
            game.queue_sound(0, 0x32, 6);
        }
        let frame = u16::from(FRAMES[usize::from(movement)]);
        word!(
            game.samus_pose = if game.ram().read_byte(0xa1e) == 4 {
                0xd8
            } else {
                0xd7
            }
        );
        game.finish_pose_change();
        word! { game; samus_anim_frame_skip = 0; samus_anim_frame = frame; }
        let x = game.player_x();
        let x = x.wrapping_sub(word!(game.layer1_x_pos));
        game.set_player_x(x);
        let y = game.player_y();
        let y = y.wrapping_sub(word!(game.layer1_y_pos));
        game.set_player_y(y);
        word!(game.game_state += 1);
    }
    0
}

pub fn flash_death<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    if word!(game.g_word_7E0DE6) as i32 - 4i32 & 0x8000 != 0 {
        queue_death_tiles((2u16).wrapping_mul(word!(game.g_word_7E0DE6)), game);
    }
    if word!(game.g_word_7E0DE6 += 1, reread) as i32 - 60i32 & 0x8000 != 0 {
        let expired = word!(game.game_options_screen_index -= 1, reread) & 0x8000 != 0;
        if word!(game.game_options_screen_index) == 0 || expired {
            if word!(game.g_word_7E0DE4) != 0 {
                word! { game; g_word_7E0DE4 = 0; game_options_screen_index = 3; }
            } else {
                word! { game, 1; g_word_7E0DE4, game_options_screen_index }
            }
            death_palette(Some(word!(game.g_word_7E0DE4).wrapping_mul(2)), game);
        }
        game.draw_samus(Body::Death);
    } else {
        death_palette(None, game);
        queue_death_tiles(8, game);
        let delay = game.runtime().table(Source::Native, 0x9bb823).byte(0);
        word!(game.game_options_screen_index = u16::from(delay));
        word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
        advance_suit_explosion(game);
        word!(
            game.substate = (word!(game.joypad2_last) as i32 & 0xb0
                == kButton_A as i32 | kButton_L as i32 | kButton_R as i32)
                as u16
        );
        word!(game.game_state += 1);
    }
    0
}

fn death_palette<H: Host + ?Sized>(frame: Option<u16>, game: &mut H) {
    let table = if frame.is_some() { 0x9bb6d2 } else { 0x9bb5c8 };
    let pointer = crate::cartridge::rom_table_word!(
        game,
        table,
        (word!(game.samus_suit_palette_index) >> 1) as isize
    )
    .wrapping_add(frame.unwrap_or(0));
    let address = game
        .runtime()
        .legacy_native_record(0x9b, pointer)
        .native_word(0);
    let colors = game
        .runtime()
        .legacy_native_record(0x9b, address)
        .bytes::<32>(0);
    game.ram().copy_in(0xc180, &colors);
    // The second source is resolved only after the first complete palette copy.
    let address = match frame {
        Some(frame) => crate::cartridge::rom_table_word!(game, 0x9bb80f, (frame >> 1) as isize),
        None => 0xa120,
    };
    let colors = game
        .runtime()
        .legacy_native_record(0x9b, address)
        .bytes::<32>(0);
    game.ram().copy_in(0xc1e0, &colors);
}

fn queue_death_tiles<H: Host + ?Sized>(frame: u16, game: &mut H) {
    const SOURCES: [u16; 5] = [0x8400, 0x8800, 0x8c00, 0x9000, 0x8000];
    const DESTINATIONS: [u16; 5] = [0x6200, 0x6400, 0x6600, 0x6800, 0x6000];
    crate::display_refresh::queue_native_transfer(
        word!(game.vram_write_queue_tail),
        1024,
        |_| SOURCES[usize::from(frame >> 1)],
        0x9b,
        |_| DESTINATIONS[usize::from(frame >> 1)],
        game,
    );
}

fn whiten_death_palette<H: Host + ?Sized>(game: &mut H) {
    if word!(game.substate) == 0 && word!(game.g_word_7E0DE4) != 0 {
        const SHADES: [u16; 22] = [
            0x421, 0xc63, 0x14a5, 0x1ce7, 0x2529, 0x2d6b, 0x35ad, 0x4210, 0x4a52, 0x4e73, 0x5294,
            0x56b5, 0x5ad6, 0x5ef7, 0x6318, 0x6739, 0x6b5a, 0x6f7b, 0x739c, 0x77bd, 0x7bde, 0x7fff,
        ];
        let shade = SHADES[usize::from(word!(game.g_word_7E0DE6))];
        word_transfers!(game, ascending, 192; native(0xc000) <- value(shade));
        word_transfers!(game, ascending, 32; native(0xc1a0) <- value(shade));
        if word!(game.g_word_7E0DE6) as i32 - 20i32 & 0x8000 != 0 {
            word!(game.g_word_7E0DE6 += 1);
        }
    }
}

pub fn explode_suit<H: Host + ?Sized>(game: &mut H) -> CoroutineRet {
    whiten_death_palette(game);
    if advance_suit_explosion(game) {
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    0
}

fn advance_suit_explosion<H: Host + ?Sized>(game: &mut H) -> bool {
    let expired = word!(game.game_options_screen_index -= 1, reread) & 0x8000 != 0;
    if word!(game.game_options_screen_index) == 0 || expired {
        if word!(game.g_word_7E0DE4 += 1, reread) as i32 - 9i32 & 0x8000 == 0 {
            word!(game.g_word_7E0DE6 = 21);
            whiten_death_palette(game);
            word!(game.substate = 0);
            return true;
        }
        let recolor =
            word!(game.substate) == 0 || word!(game.g_word_7E0DE4) as i32 - 2i32 & 0x8000 != 0;
        // Each read captures the native table afresh; the frame index is a RAM
        // read, so taking it first leaves the captured bytes unchanged.
        let index = 2 * usize::from(word!(game.g_word_7E0DE4));
        let delay = game.runtime().table(Source::Native, 0x9bb823).byte(index);
        word!(game.game_options_screen_index = u16::from(delay));
        if recolor {
            let index = 2 * usize::from(word!(game.g_word_7E0DE4)) + 1;
            let frame = game.runtime().table(Source::Native, 0x9bb823).byte(index);
            death_palette(Some(2u16.wrapping_mul(u16::from(frame))), game);
        }
    }
    let sprite = (if game.ram().read_byte(0xa1e) == 4 {
        word!(game.g_word_7E0DE4) as i32 + 2085
    } else {
        word!(game.g_word_7E0DE4) as i32 + 2076
    }) as u16;
    sprite_output::samus(
        sprite,
        (game.player_x(), game.player_y()),
        game,
        &mut sprite_output::Native,
    );
    false
}
