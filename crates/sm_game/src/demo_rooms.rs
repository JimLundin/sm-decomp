//! Attract-mode playback, room selection, placement and delayed setup.
use crate::imported::Imported;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::{EnemyData, do_until};
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::session_slot;
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::demo_rooms::{
    DemoEntry, DemoScene, DemoScenes, DemoSets, SCENES_ADDRESS, SCENES_BYTES, SETS_ADDRESS,
    SETS_BYTES,
};

pub type ImportedSets = Imported<DemoSets>;
pub type ImportedScenes = Imported<DemoScenes>;

pub fn import_sets(image: &RomImage) -> Option<ImportedSets> {
    Imported::import(image, SETS_ADDRESS, SETS_BYTES, DemoSets::import)
}

pub fn import_scenes(image: &RomImage) -> Option<ImportedScenes> {
    Imported::import(image, SCENES_ADDRESS, SCENES_BYTES, DemoScenes::import)
}

/// The set's first scene address in the loaded image. The table serves it
/// while that image is the imported allocation; otherwise, and past the four
/// sets, the original indexed load runs with its failures.
fn set_start<H: crate::menus::host::Host + ?Sized>(game: &H, set: u16) -> u16 {
    let rom = game.loaded_image();
    Imported::from(game.demo_sets(), rom)
        .and_then(|sets| sets.start(usize::from(set)))
        .unwrap_or_else(|| rom.table_word(lorom_offset(SETS_ADDRESS) as usize, set as isize))
}

/// The scene or end marker at a bank $82 address in the native image, while it
/// is the imported allocation. Every imported entry lies above $8000, so its
/// original fetch has no invalid-window diagnostic; callers keep that fetch
/// for any other address, image or a missing image.
fn entry<H: crate::menus::host::Host + ?Sized>(game: &H, address: u16) -> Option<DemoEntry> {
    let native = game.native_image_binding()?;
    Imported::from(game.demo_scenes(), &native).and_then(|scenes| scenes.entry(address))
}

fn scene<H: crate::menus::host::Host + ?Sized>(game: &H, address: u16) -> Option<DemoScene> {
    match entry(game, address)? {
        DemoEntry::Scene(scene) => Some(scene),
        DemoEntry::End => None,
    }
}

pub fn reset<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> u8 {
    word! { game, 0; UNUSED_WORD_7E0DF8, UNUSED_WORD_7E0DFA, UNUSED_WORD_7E0DFC }
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = 0x9b68
    });
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.demo_set = 0);
    if crate::session_cohort::load::<crate::flow::Flow, _>(game).num_demo_sets == 4 {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.demo_set = 3);
    }
    word!(game.GAME_STATE += 1);
    0
}

pub fn play<H: crate::menus::host::Host + ?Sized>(
    game: &mut H,
    gameplay: impl FnOnce(&mut H) -> u8,
) -> u8 {
    crate::probe::hit("coroutine.demo");
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
                if word!(game.JOYPAD1_NEWKEYS) != 0 {
                    word!(game.SUBSTATE = 1);
                    break 'finish_demo;
                }
                {
                    let value = crate::session_cohort::load::<crate::flow::Flow, _>(game)
                        .demo_timer
                        .wrapping_sub(1);
                    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                        flow.demo_timer = value
                    })
                };
                if !(crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_timer == 0
                    || (crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_timer
                        & 0x8000)
                        != 0)
                {
                    break 'done;
                }
                word! {game; SUBSTATE = 0; MY_COUNTER = 90; }
            }
            do_until! { word!(game.MY_COUNTER -= 1, reread) == 0;
                let yielded: u8 = crate::display_refresh::wait(true, game);
                if yielded != 0 {
                    game.ram().write_byte(0x614, 1);
                    return yielded;
                }
                if word!(game.JOYPAD1_NEWKEYS) != 0 {
                    word!(game.SUBSTATE = 1);
                    break;
                }
            }
        }
        word!(game.GAME_STATE += 1);
        word!(game.DEBUG_DISABLE_SOUNDS = 0);
        game.ram().write_byte(0x51, 0x80);
        crate::screen_fade::store(game, 1, 1);
    }
    game.ram().write_byte(0x614, 0);
    0
}

pub fn advance_set<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let base = set_start(
        game,
        crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_set,
    );
    let address = base.wrapping_add(
        crate::session_cohort::load::<crate::flow::Flow, _>(game)
            .demo_scene
            .wrapping_mul(18),
    );
    let ended = match entry(game, address) {
        Some(entry) => entry == DemoEntry::End,
        None => native_record(0x820000 | u32::from(address), game).word(0) == 0xffff,
    };
    if ended {
        word!(game.SUBSTATE = 0);
        let next = crate::session_cohort::load::<crate::flow::Flow, _>(game)
            .demo_set
            .wrapping_add(1);
        {
            let value = if next
                >= crate::session_cohort::load::<crate::flow::Flow, _>(game).num_demo_sets
            {
                0
            } else {
                next
            };
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.demo_set = value
            })
        };
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.demo_scene = 0);
    } else {
        word!(game.SUBSTATE = 0x8000);
    }
}

pub fn load<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    use crate::session_access::SessionAccess;
    game.runtime().session_mut().set_exit_key(0);
    // Loading reads the scene before the set table; the resumed hook does the reverse.
    let offset =
        18 * i32::from(crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_scene);
    let base = set_start(
        game,
        crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_set,
    );
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
    {
        let mut runtime = game.runtime();
        runtime.session_mut().set_room_key(room.room);
        runtime.session_mut().set_exit_key(room.door);
    }
    let layer1 = {
        use crate::camera::{set_bg1_offset, set_layer1};
        let mut runtime = game.runtime();
        set_layer1(&mut runtime, 0, room.layer1_x);
        let x = runtime.session().room.camera.layer1[0];
        set_bg1_offset(&mut runtime, 0, x);
        set_layer1(&mut runtime, 1, room.layer1_y);
        let y = runtime.session().room.camera.layer1[1];
        runtime.session_mut().room.camera.bg1_offset[1] = y;
        runtime.session().room.camera.layer1
    };
    game.set_player_y(layer1[1].wrapping_add(room.samus_y_offset));
    game.set_previous_player_y(game.player_y());
    game.set_player_x(
        room.samus_x_offset
            .wrapping_add(layer1[0])
            .wrapping_add(128),
    );
    game.set_previous_player_x(game.player_x());
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.demo_timer = room.timer
    });
    let room_key = game.runtime().session().room_key();
    let value = native_record(0x8f0000 | u32::from(room_key), game).byte(1);
    // The original stores only the area's low byte.
    let mut runtime = game.runtime();
    let area = &mut runtime.session_mut().room.area;
    *area = *area & 0xff00 | u16::from(value);
    drop(runtime);
    word! { game, 0; REG_BG1HOFS, REG_BG1VOFS }
    {
        let value = crate::session_cohort::load::<crate::flow::Flow, _>(game)
            .demo_scene
            .wrapping_add(1);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.demo_scene = value
        })
    };
    crate::word_transfers!(game, ascending, 4;
        native(0xd830) <- value(u16::MAX),
        native(0xd870) <- value(u16::MAX),
        le(UNUSED_WORD_7ED8F0, 0) <- value(u16::MAX),
        native(0xd908) <- value(u16::MAX),
        native(0xd8f8) <- value(u16::MAX),
        native(0xd900) <- value(u16::MAX),
        native(0xd8b0) <- value(0),
        native(0xd820) <- value(0),
        native(0xd828) <- value(0),
    );
    crate::word_transfers!(game, ascending, 28;
        native(0xd838) <- value(u16::MAX),
        native(0xd878) <- value(u16::MAX),
        native(0xd8b8) <- value(0),
    );
    crate::word_transfers!(game, ascending, 768; le(EXPLORED_MAP_TILES_SAVED, 0) <- value(0));
    word! { game, 0;
        SAMUS_MAX_RESERVE_HEALTH, SAMUS_RESERVE_HEALTH, RESERVE_HEALTH_MODE,
        LOADING_GAME_STATE, DEBUG_DISABLE_MINIMAP,
    }
}

pub fn finish_setup<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let base = set_start(
        game,
        crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_set,
    );
    let address = (i32::from(base)
        + 18 * (i32::from(crate::session_cohort::load::<crate::flow::Flow, _>(game).demo_scene)
            - 1)) as u16;
    let hook = scene(game, address).map_or_else(
        || native_record(0x820000 | u32::from(address), game).word(16),
        |scene| scene.hook,
    );
    run_room_hook(0x820000 | u32::from(hook), game);
}

pub fn run_room_hook<H: crate::menus::host::Host + ?Sized>(address: u32, game: &mut H) {
    match address {
        0x82891a => game.ram().write_byte(0xcd20 + 33, 0),
        0x828924 => {}
        0x828925 => game.ram().copy_in(0x59, &74u16.to_ne_bytes()),
        0x82892b => session_slot!(game, EnemyData[0].ai_preinstr = 60),
        0x828932 => crate::progress::leased::set_boss_byte_at(1, 1, game),
        _ => {
            game.unreachable_native();
        }
    }
}
