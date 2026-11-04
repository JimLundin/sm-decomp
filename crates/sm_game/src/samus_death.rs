//! Samus's death sequence: the frozen pose, the death pose, the suit flash
//! and the suit explosion, with their palettes and tile uploads.
use crate::constants::buttons::{K_BUTTON_A, K_BUTTON_L, K_BUTTON_R};
use crate::menu_state::MenuScreens;
use crate::menus::prelude::CoroutineRet;
use crate::player_transitions as pt;
use crate::scene::samus_body::Body;
use crate::sprite_output;
use crate::visual_host::{Record, Source};

/// The lease the death states run on: the frame's first lease, kept open
/// through the state.
pub trait Lease:
    crate::runtime_access::PlayerLease
    + crate::samus_drawing::Authority
    + crate::menus::state::Access
    + crate::gameplay_context::Access
    + crate::sequence_state::Access
    + crate::session_owners::AudioHost
    + crate::effects::SoundEffects
    + crate::player_position::PlayerFacts
    + crate::records::Records
    + crate::display_refresh::DisplayOwner
    + crate::display::transfer::TilemapQueue
    + crate::screen_fade::Owner
    + crate::sprite_output::MapHost
{
}
impl<
    T: crate::runtime_access::PlayerLease
        + crate::samus_drawing::Authority
        + crate::menus::state::Access
        + crate::gameplay_context::Access
        + crate::sequence_state::Access
        + crate::session_owners::AudioHost
        + crate::effects::SoundEffects
        + crate::player_position::PlayerFacts
        + crate::records::Records
        + crate::display_refresh::DisplayOwner
        + crate::display::transfer::TilemapQueue
        + crate::screen_fade::Owner
        + crate::sprite_output::MapHost
        + ?Sized,
> Lease for T
{
}

pub fn advance_death_pose<H: Lease>(game: &mut H) -> CoroutineRet {
    pt::animation::advance_animation(game, pt::state::AnimationStep::Death);
    crate::samus_drawing::draw_body(game, Body::Death);
    let expired = crate::menus::state::with(game, |_, screens: &mut MenuScreens| {
        let timer = &mut screens.death.flash_timer;
        *timer = timer.wrapping_sub(1);
        *timer == 0 || *timer & 0x8000 != 0
    });
    if expired {
        crate::gameplay_word!(game.GAME_STATE += 1);
    }
    0
}

// Death phases share palette selection and the native upload queue with live drawing.
pub fn begin_death<H: Lease>(game: &mut H) -> CoroutineRet {
    crate::samus_drawing::draw_body(game, Body::Frozen);
    if crate::music_requests::has_queued_music(game) == 0 {
        const FRAMES: [u8; 28] = [
            5, 5, 5, 5, 1, 5, 5, 0, 1, 0, 5, 5, 5, 5, 5, 5, 5, 1, 1, 1, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        let movement = game.movement_type();
        if game.movement_type() == 3 {
            crate::effects::SoundEffects::queue_sound(game, 0, 0x32, 6);
        }
        let frame = u16::from(FRAMES[usize::from(movement)]);
        let pose = if game.facing() == 4 { 0xd8 } else { 0xd7 };
        // The death pose, its animation start and Samus's screen position are
        // Session-owned player state: set them on the lease that finishes the
        // pose change.
        game.session_mut().player.pose = pose;
        crate::player_transitions::pose::finish_pose_change(game);
        let player = &mut game.session_mut().player;
        player.transitions.animation_skip =
            crate::player_transitions::state::AnimationSkip::from_raw(0);
        player.animation_clock.frame = frame;
        let layer1 = game.session().room.camera.layer1;
        let x = game.player_x().wrapping_sub(layer1[0]);
        game.set_player_x(x);
        let y = game.player_y().wrapping_sub(layer1[1]);
        game.set_player_y(y);
        crate::gameplay_word!(game.GAME_STATE += 1);
    }
    0
}

pub fn flash_death<H: Lease>(game: &mut H) -> CoroutineRet {
    crate::menus::state::with(game, |game, screens: &mut MenuScreens| {
        if screens.death.counter.wrapping_sub(4) & 0x8000 != 0 {
            queue_death_tiles(2u16.wrapping_mul(screens.death.counter), game);
        }
        screens.death.counter = screens.death.counter.wrapping_add(1);
        if screens.death.counter.wrapping_sub(60) & 0x8000 != 0 {
            let timer = &mut screens.options_menu.screen_index;
            *timer = timer.wrapping_sub(1);
            if *timer == 0 || *timer & 0x8000 != 0 {
                let (index, delay) = if screens.death.index != 0 {
                    (0, 3)
                } else {
                    (1, 1)
                };
                screens.death.index = index;
                screens.options_menu.screen_index = delay;
                death_palette(Some(index.wrapping_mul(2)), game);
            }
            crate::samus_drawing::draw_body(game, Body::Death);
        } else {
            death_palette(None, game);
            queue_death_tiles(8, game);
            let delay = game.table(Source::Native, 0x9bb823).byte(0);
            screens.options_menu.screen_index = u16::from(delay);
            screens.death.index = 0;
            screens.death.counter = 0;
            advance_suit_explosion(game, screens);
            let stage = u16::from(
                crate::gameplay_word!(game.JOYPAD2_LAST) as i32 & 0xb0
                    == K_BUTTON_A as i32 | K_BUTTON_L as i32 | K_BUTTON_R as i32,
            );
            crate::sequence_state::with_words(
                game,
                |_, words: &mut crate::session_words::SessionWords| words.substate = stage,
            );
            crate::gameplay_word!(game.GAME_STATE += 1);
        }
        0
    })
}

fn death_palette<H: Lease>(frame: Option<u16>, game: &mut H) {
    let pointer = crate::gameplay_tables::death_palette(
        game,
        frame.is_some(),
        (crate::session_words::current(game).samus_suit_palette_index >> 1) as isize,
    )
    .wrapping_add(frame.unwrap_or(0));
    let address = game.legacy_native_record(0x9b, pointer).native_word(0);
    let colors = game.legacy_native_record(0x9b, address).bytes::<32>(0);
    store_colors(game, 192, colors);
    // The second source is resolved only after the first complete palette copy.
    let address = match frame {
        Some(frame) => crate::gameplay_tables::death_frame_palette(game, (frame >> 1) as isize),
        None => 0xa120,
    };
    let colors = game.legacy_native_record(0x9b, address).bytes::<32>(0);
    store_colors(game, 240, colors);
}

/// Store a 16-colour palette's bytes from ROM at palette colour `first`, then
/// refresh the lease's observations as the projected word stores did.
fn store_colors(
    lease: &mut (impl crate::session_access::SessionAccess + ?Sized),
    first: usize,
    bytes: [u8; 32],
) {
    let colors = &mut lease.session_mut().presentation.visual_mut().colors[first..first + 16];
    for (color, pair) in colors.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *color = u16::from_le_bytes(*pair);
    }
    lease.words_stored();
}

fn queue_death_tiles<H: Lease>(frame: u16, game: &mut H) {
    const SOURCES: [u16; 5] = [0x8400, 0x8800, 0x8c00, 0x9000, 0x8000];
    const DESTINATIONS: [u16; 5] = [0x6200, 0x6400, 0x6600, 0x6800, 0x6000];
    crate::display_refresh::queue_native_transfer(
        game.queue_tail(),
        1024,
        |_| SOURCES[usize::from(frame >> 1)],
        0x9b,
        |_| DESTINATIONS[usize::from(frame >> 1)],
        game,
    );
}

fn whiten_death_palette<H: Lease>(game: &mut H, screens: &mut MenuScreens) {
    if crate::session_words::current(game).substate == 0 && screens.death.index != 0 {
        const SHADES: [u16; 22] = [
            0x421, 0xc63, 0x14a5, 0x1ce7, 0x2529, 0x2d6b, 0x35ad, 0x4210, 0x4a52, 0x4e73, 0x5294,
            0x56b5, 0x5ad6, 0x5ef7, 0x6318, 0x6739, 0x6b5a, 0x6f7b, 0x739c, 0x77bd, 0x7bde, 0x7fff,
        ];
        let shade = SHADES[usize::from(screens.death.counter)];
        // Every colour but Samus's palette ($C180..$C19F) and the last two
        // sprite palettes ($C1E0..$C1FF).
        let shade = crate::word_transfers::stored_native(shade);
        let (first, samus_and_sprites) = game
            .session_mut()
            .presentation
            .visual_mut()
            .colors
            .split_at_mut(192);
        for color in first.iter_mut().chain(&mut samus_and_sprites[16..48]) {
            *color = shade;
        }
        game.words_stored();
        if screens.death.counter.wrapping_sub(20) & 0x8000 != 0 {
            screens.death.counter = screens.death.counter.wrapping_add(1);
        }
    }
}

pub fn explode_suit<H: Lease>(game: &mut H) -> CoroutineRet {
    let exploded = crate::menus::state::with(game, |game, screens: &mut MenuScreens| {
        whiten_death_palette(game, screens);
        advance_suit_explosion(game, screens)
    });
    if exploded {
        crate::screen_fade::store(game, 1, 1);
        crate::gameplay_word!(game.GAME_STATE += 1);
    }
    0
}

fn advance_suit_explosion<H: Lease>(game: &mut H, screens: &mut MenuScreens) -> bool {
    let timer = &mut screens.options_menu.screen_index;
    *timer = timer.wrapping_sub(1);
    if *timer == 0 || *timer & 0x8000 != 0 {
        screens.death.index = screens.death.index.wrapping_add(1);
        if screens.death.index.wrapping_sub(9) & 0x8000 == 0 {
            screens.death.counter = 21;
            whiten_death_palette(game, screens);
            crate::sequence_state::with_words(
                game,
                |_, words: &mut crate::session_words::SessionWords| words.substate = 0,
            );
            return true;
        }
        let recolor = crate::session_words::current(game).substate == 0
            || screens.death.index.wrapping_sub(2) & 0x8000 != 0;
        let index = 2 * usize::from(screens.death.index);
        // The delay and frame bytes are one table record.
        let record = game.table(Source::Native, 0x9bb823);
        screens.options_menu.screen_index = u16::from(record.byte(index));
        if recolor {
            let frame = record.byte(index + 1);
            death_palette(Some(2u16.wrapping_mul(u16::from(frame))), game);
        }
    }
    let base = if game.facing() == 4 { 2085 } else { 2076 };
    sprite_output::samus(
        screens.death.index.wrapping_add(base),
        (game.player_x(), game.player_y()),
        game,
        &mut sprite_output::Native,
    );
    false
}
