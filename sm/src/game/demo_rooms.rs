//! Attract-mode playback, room selection, placement and delayed setup.
#![forbid(unsafe_code)]
use crate::game::imported::Imported;
use crate::game::{gameplay_flag_runtime, menu_tilemap::native_record};
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rom::{RomImage, lorom_offset};
use crate::types::EnemyData;
use sm_data::demo_rooms::{
    DemoEntry, DemoScene, DemoScenes, DemoSets, SCENES_ADDRESS, SCENES_BYTES, SETS_ADDRESS,
    SETS_BYTES,
};

pub(crate) type ImportedSets = Imported<DemoSets>;
pub(crate) type ImportedScenes = Imported<DemoScenes>;

pub(crate) fn import_sets(image: &RomImage) -> Option<ImportedSets> {
    Imported::import(image, SETS_ADDRESS, SETS_BYTES, DemoSets::import)
}

pub(crate) fn import_scenes(image: &RomImage) -> Option<ImportedScenes> {
    Imported::import(image, SCENES_ADDRESS, SCENES_BYTES, DemoScenes::import)
}

/// The set's first scene address in the loaded image. The table serves it
/// while that image is the imported allocation; otherwise, and past the four
/// sets, the original indexed load runs with its failures.
fn set_start(game: &Game, set: u16) -> u16 {
    let rom = game.loaded_rom.as_ref().expect("ROM must be loaded");
    Imported::from(&game.demo_sets, rom)
        .and_then(|sets| sets.start(usize::from(set)))
        .unwrap_or_else(|| rom.table_word(lorom_offset(SETS_ADDRESS) as usize, set as isize))
}

/// The scene or end marker at a bank $82 address in the native image, while it
/// is the imported allocation. Every imported entry lies above $8000, so its
/// original fetch has no invalid-window diagnostic; callers keep that fetch
/// for any other address, image or a missing image.
fn entry(game: &Game, address: u16) -> Option<DemoEntry> {
    let native = game.capture_native_rom_binding()?;
    Imported::from(&game.demo_scenes, &native).and_then(|scenes| scenes.entry(address))
}

fn scene(game: &Game, address: u16) -> Option<DemoScene> {
    match entry(game, address)? {
        DemoEntry::Scene(scene) => Some(scene),
        DemoEntry::End => None,
    }
}

pub(crate) fn reset(game: &mut Game) -> u8 {
    word! { game, 0; UNUSED_word_7E0DF8, UNUSED_word_7E0DFA, UNUSED_word_7E0DFC }
    word! {game; cinematic_function = ((0x8b9b68i32) as u16); demo_set = 0; }
    if word!(game.num_demo_sets) == 4 {
        word!(game.demo_set = 3);
    }
    word!(game.game_state += 1);
    0
}

pub(crate) fn play(game: &mut Game, gameplay: impl FnOnce(&mut Game) -> u8) -> u8 {
    crate::verify_probe::hit("coroutine.demo");
    let resume = game.ram().read_byte(0x614);
    'done: {
        if resume > 2 {
            break 'done;
        }
        'finish_demo: {
            if resume != 1 {
                let yielded: u8 = gameplay(game);
                if yielded != 0 {
                    game.ram().write_byte(0x614, 2);
                    return yielded;
                }
                if word!(game.joypad1_newkeys) != 0 {
                    word!(game.substate = 1);
                    break 'finish_demo;
                }
                word!(game.demo_timer -= 1);
                if !(word!(game.demo_timer) == 0 || (word!(game.demo_timer) & 0x8000) != 0) {
                    break 'done;
                }
                word! {game; substate = 0; my_counter = 90; }
            }
            do_until! { word!(game.my_counter -= 1, reread) == 0;
                let yielded: u8 = crate::game::display_refresh::wait(true, game);
                if yielded != 0 {
                    game.ram().write_byte(0x614, 1);
                    return yielded;
                }
                if word!(game.joypad1_newkeys) != 0 {
                    word!(game.substate = 1);
                    break;
                }
            }
        }
        word!(game.game_state += 1);
        word!(game.debug_disable_sounds = 0);
        game.ram().write_byte(0x51, 0x80);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
    }
    game.ram().write_byte(0x614, 0);
    0
}

pub(crate) fn advance_set(game: &mut Game) {
    let base = set_start(game, word!(game.demo_set));
    let address = base.wrapping_add(word!(game.demo_scene).wrapping_mul(18));
    let ended = match entry(game, address) {
        Some(entry) => entry == DemoEntry::End,
        None => native_record(0x820000 | u32::from(address), game).word(0) == 0xffff,
    };
    if ended {
        word!(game.substate = 0);
        let next = word!(game.demo_set).wrapping_add(1);
        word! { game;
            demo_set = if next >= word!(game.num_demo_sets) { 0 } else { next };
            demo_scene = 0;
        }
    } else {
        word!(game.substate = 0x8000);
    }
}

pub(crate) fn load(game: &mut Game) {
    word!(game.door_def_ptr = 0);
    // Loading reads the scene before the set table; the resumed hook does the reverse.
    let offset = 18 * i32::from(word!(game.demo_scene));
    let base = set_start(game, word!(game.demo_set));
    let address = (offset + i32::from(base)) as u16;
    let room = scene(game, address).unwrap_or_else(|| {
        let record = native_record(0x820000 | u32::from(address), game);
        DemoScene {
            room: record.word(0),
            door: record.word(2),
            door_slot: record.word(4),
            layer1_x: record.word(6),
            layer1_y: record.word(8),
            samus_y_offset: record.word(10),
            samus_x_offset: record.word(12),
            timer: record.word(14),
            hook: record.word(16),
        }
    });
    word! { game;
        room_ptr = room.room;
        door_def_ptr = room.door;
        layer1_x_pos = room.layer1_x;
        bg1_x_offset = word!(game.layer1_x_pos);
        layer1_y_pos = room.layer1_y;
        bg1_y_offset = word!(game.layer1_y_pos);
        samus_y_pos = word!(game.layer1_y_pos).wrapping_add(room.samus_y_offset);
        samus_prev_y_pos = word!(game.samus_y_pos);
        samus_x_pos = room.samus_x_offset.wrapping_add(word!(game.layer1_x_pos)).wrapping_add(128);
        samus_prev_x_pos = word!(game.samus_x_pos);
        demo_timer = room.timer;
    }
    let value = native_record(0x8f0000 | u32::from(word!(game.room_ptr)), game).byte(1);
    game.ram().write_byte(0x79f, value);
    word! { game, 0; reg_BG1HOFS, reg_BG1VOFS }
    word!(game.demo_scene += 1);
    word_transfers!(game, ascending, 4;
        native(0xd830) <- value(u16::MAX),
        native(0xd870) <- value(u16::MAX),
        le(UNUSED_word_7ED8F0, 0) <- value(u16::MAX),
        native(0xd908) <- value(u16::MAX),
        native(0xd8f8) <- value(u16::MAX),
        native(0xd900) <- value(u16::MAX),
        native(0xd8b0) <- value(0),
        native(0xd820) <- value(0),
        native(0xd828) <- value(0),
    );
    word_transfers!(game, ascending, 28;
        native(0xd838) <- value(u16::MAX),
        native(0xd878) <- value(u16::MAX),
        native(0xd8b8) <- value(0),
    );
    word_transfers!(game, ascending, 768; le(explored_map_tiles_saved, 0) <- value(0));
    word! { game, 0;
        samus_max_reserve_health, samus_reserve_health, reserve_health_mode,
        loading_game_state, debug_disable_minimap,
    }
}

pub(crate) fn finish_setup(game: &mut Game) {
    let base = set_start(game, word!(game.demo_set));
    let address = (i32::from(base) + 18 * (i32::from(word!(game.demo_scene)) - 1)) as u16;
    let hook = scene(game, address).map_or_else(
        || native_record(0x820000 | u32::from(address), game).word(16),
        |scene| scene.hook,
    );
    run_room_hook(0x820000 | u32::from(hook), game);
}

pub(crate) fn run_room_hook(address: u32, game: &mut Game) {
    match address {
        0x82891a => game.ram().write_byte(0xcd20 + 33, 0),
        0x828924 => {}
        0x828925 => game.ram().copy_in(0x59, &74u16.to_ne_bytes()),
        0x82892b => slot!(game, EnemyData[0].ai_preinstr = 60),
        0x828932 => gameplay_flag_runtime::set_boss_byte_at(1, 1, game),
        _ => {
            crate::game::animated_tiles::native::unreachable(game);
        }
    }
}
