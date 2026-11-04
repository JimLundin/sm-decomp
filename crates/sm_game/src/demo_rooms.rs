//! Attract-mode playback, room selection, placement and delayed setup.
use crate::actors::access::{AccessMode, ActorKey};
use crate::display::preparation::Field;
use crate::imported::Imported;
use crate::menus::menu_tilemap::native_record;
use crate::menus::prelude::do_until;
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

/// The imported demo tables.
pub trait Tables {
    fn demo_sets(&self) -> &Option<ImportedSets>;
    fn demo_scenes(&self) -> &Option<ImportedScenes>;
}
impl<T: crate::menus::host::Host + ?Sized> Tables for T {
    fn demo_sets(&self) -> &Option<ImportedSets> {
        crate::menus::host::Host::demo_sets(self)
    }
    fn demo_scenes(&self) -> &Option<ImportedScenes> {
        crate::menus::host::Host::demo_scenes(self)
    }
}

/// What the demo tables' reads need: the tables and the loaded and native
/// images.
pub trait TableReads:
    Tables + crate::cartridge::LoadedImage + crate::cartridge::NativeImage
{
}
impl<T: Tables + crate::cartridge::LoadedImage + crate::cartridge::NativeImage + ?Sized> TableReads
    for T
{
}

/// The set's first scene address in the loaded image. The table serves it
/// while that image is the imported allocation; otherwise, and past the four
/// sets, the original indexed load runs with its failures.
fn set_start<H: TableReads + ?Sized>(game: &H, set: u16) -> u16 {
    let rom = game.loaded_image();
    Imported::from(game.demo_sets(), rom)
        .and_then(|sets| sets.start(usize::from(set)))
        .unwrap_or_else(|| rom.table_word(lorom_offset(SETS_ADDRESS) as usize, set as isize))
}

/// The scene or end marker at a bank $82 address in the native image, while it
/// is the imported allocation. Every imported entry lies above $8000, so its
/// original fetch has no invalid-window diagnostic; callers keep that fetch
/// for any other address, image or a missing image.
fn entry<H: TableReads + ?Sized>(game: &H, address: u16) -> Option<DemoEntry> {
    let native = game.native_image_binding()?;
    Imported::from(game.demo_scenes(), &native).and_then(|scenes| scenes.entry(address))
}

fn scene<H: TableReads + ?Sized>(game: &H, address: u16) -> Option<DemoScene> {
    match entry(game, address)? {
        DemoEntry::Scene(scene) => Some(scene),
        DemoEntry::End => None,
    }
}

/// The demo reset state's frame, on the frame's lease.
pub fn reset<H: crate::gameplay_frame::PassLease + ?Sized>(game: &mut H) -> u8 {
    let session = game.session_mut();
    session.player.turn_request = 0;
    session.projectiles.preparation.aim_history = 0;
    session.player.spare_0dfc = 0;
    game.words_stored();
    crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = 0x9b68
    });
    crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| flow.demo_set = 0);
    if crate::sequence_state::Access::sequence_flow(&*game).num_demo_sets == 4 {
        crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| flow.demo_set = 3);
    }
    crate::gameplay_word!(game.GAME_STATE += 1);
    0
}

/// The demo's play loop on the frame's lease: the pass, the demo timer and
/// the closing wait.
pub fn play<H: crate::gameplay_frame::PassLease + ?Sized>(
    game: &mut H,
    gameplay: impl FnOnce(&mut H) -> u8,
) -> u8 {
    crate::probe::hit("coroutine.demo");
    let resume = game.demo_phase();
    'done: {
        if resume > 2 {
            break 'done;
        }
        'finish_demo: {
            if resume != 1 {
                let yielded: u8 = gameplay(game);
                if yielded != 0 {
                    game.demo_set_phase(2);
                    return yielded;
                }
                if crate::gameplay_word!(game.JOYPAD1_NEWKEYS) != 0 {
                    crate::sequence_word!(game.SUBSTATE = 1);
                    break 'finish_demo;
                }
                {
                    let value = crate::sequence_state::Access::sequence_flow(&*game)
                        .demo_timer
                        .wrapping_sub(1);
                    crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
                        flow.demo_timer = value
                    })
                };
                if !(crate::sequence_state::Access::sequence_flow(&*game).demo_timer == 0
                    || (crate::sequence_state::Access::sequence_flow(&*game).demo_timer & 0x8000)
                        != 0)
                {
                    break 'done;
                }
                crate::sequence_word!(game.SUBSTATE = 0);
                game.session_mut().messages.delay = 90;
                game.words_stored();
            }
            do_until! { count_down(game) == 0;
                let yielded: u8 = crate::display_refresh::wait(true, game);
                if yielded != 0 {
                    game.demo_set_phase(1);
                    return yielded;
                }
                if crate::gameplay_word!(game.JOYPAD1_NEWKEYS) != 0 {
                    crate::sequence_word!(game.SUBSTATE = 1);
                    break;
                }
            }
        }
        crate::gameplay_word!(game.GAME_STATE += 1);
        crate::sequence_word!(game.DEBUG_DISABLE_SOUNDS = 0);
        crate::frame_display_byte!(game, Field::Brightness => 0x80);
        crate::screen_fade::store(game, 1, 1);
    }
    game.demo_set_phase(0);
    0
}

/// Step to the demo's next scene or set, on the frame's lease.
pub fn advance_set<H: crate::gameplay_frame::PassLease + ?Sized>(game: &mut H) {
    let base = set_start(
        game,
        crate::sequence_state::Access::sequence_flow(&*game).demo_set,
    );
    let address = base.wrapping_add(
        crate::sequence_state::Access::sequence_flow(&*game)
            .demo_scene
            .wrapping_mul(18),
    );
    let ended = match entry(game, address) {
        Some(entry) => entry == DemoEntry::End,
        None => native_record(0x820000 | u32::from(address), game).word(0) == 0xffff,
    };
    if ended {
        crate::sequence_word!(game.SUBSTATE = 0);
        let next = crate::sequence_state::Access::sequence_flow(&*game)
            .demo_set
            .wrapping_add(1);
        {
            let value =
                if next >= crate::sequence_state::Access::sequence_flow(&*game).num_demo_sets {
                    0
                } else {
                    next
                };
            crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
                flow.demo_set = value
            })
        };
        crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
            flow.demo_scene = 0
        });
    } else {
        crate::sequence_word!(game.SUBSTATE = 0x8000);
    }
}

/// Load the demo's room on the frame's lease.
pub fn load<H: crate::gameplay_frame::PassLease + ?Sized>(game: &mut H) {
    game.session_mut().set_exit_key(0);
    // Loading reads the scene before the set table; the resumed hook does the reverse.
    let offset = 18 * i32::from(crate::sequence_state::Access::sequence_flow(&*game).demo_scene);
    let base = set_start(
        game,
        crate::sequence_state::Access::sequence_flow(&*game).demo_set,
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
        // The room and door keys, the camera and Samus's position are Session
        // state.
        use crate::camera::{set_bg1_offset, set_layer1};
        let runtime = &mut *game;
        runtime.session_mut().set_room_key(room.room);
        runtime.session_mut().set_exit_key(room.door);
        set_layer1(runtime, 0, room.layer1_x);
        let x = runtime.session().room.camera.layer1[0];
        set_bg1_offset(runtime, 0, x);
        set_layer1(runtime, 1, room.layer1_y);
        let y = runtime.session().room.camera.layer1[1];
        runtime.session_mut().room.camera.bg1_offset[1] = y;
        let layer1 = runtime.session().room.camera.layer1;
        runtime.set_player_y(layer1[1].wrapping_add(room.samus_y_offset));
        let y = runtime.player_y();
        runtime.set_previous_player_y(y);
        runtime.set_player_x(
            room.samus_x_offset
                .wrapping_add(layer1[0])
                .wrapping_add(128),
        );
        let x = runtime.player_x();
        runtime.set_previous_player_x(x);
    }
    crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
        flow.demo_timer = room.timer
    });
    // The room header's area byte.
    let room_key = game.session().room_key();
    let value = native_record(0x8f0000 | u32::from(room_key), game).byte(1);
    // The original stores only the area's low byte.
    let area = &mut game.session_mut().room.area;
    *area = *area & 0xff00 | u16::from(value);
    {
        crate::display_scroll::write(game, crate::display_scroll::Word::Bg1H, 0);
        crate::display_scroll::write(game, crate::display_scroll::Word::Bg1V, 0);
    }
    {
        let value = crate::sequence_state::Access::sequence_flow(&*game)
            .demo_scene
            .wrapping_add(1);
        crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
            flow.demo_scene = value
        })
    };
    // A demo's flags: every room object, item, station and map station used,
    // no door opened, no event or boss.
    {
        let flags = &mut game.session_mut().gameplay_flags;
        for flag in flags
            .used_room_objects
            .iter_mut()
            .chain(&mut flags.collected_locations)
            .chain(&mut flags.map_stations)
            .chain(&mut flags.used_stations)
        {
            *flag = 0xff;
        }
        for flag in flags
            .opened_doors
            .iter_mut()
            .chain(&mut flags.events)
            .chain(&mut flags.bosses_by_area)
        {
            *flag = 0;
        }
        flags.unused = [0xff; 8];
    }
    game.words_stored();
    // A demo's explored maps: the first six areas' clear.
    crate::menus::save_slots::clear_saved_maps(game, 0..768);
    crate::gameplay_context::with_inventory(
        game,
        |_, inventory: &mut crate::inventory::Inventory| {
            inventory.reserve_capacity = 0;
            inventory.reserve_health = 0;
            inventory.reserve_mode = 0;
        },
    );
    crate::menus::state::with(
        game,
        |_, selection: &mut crate::menu_state::SaveSelection| {
            selection.loading_game_state = 0;
        },
    );
    game.session_mut().minimap.disabled = 0;
    game.words_stored();
}

/// Count the demo's closing delay down a frame; the frames left.
fn count_down<H: crate::gameplay_frame::PassLease + ?Sized>(game: &mut H) -> u16 {
    let delay = &mut game.session_mut().messages.delay;
    *delay = delay.wrapping_sub(1);
    let left = *delay;
    game.words_stored();
    left
}

/// Run the demo room's setup hook on the frame's lease.
pub fn finish_setup<H: crate::gameplay_frame::PassLease + ?Sized>(game: &mut H) {
    let base = set_start(
        game,
        crate::sequence_state::Access::sequence_flow(&*game).demo_set,
    );
    let address = (i32::from(base)
        + 18 * (i32::from(crate::sequence_state::Access::sequence_flow(&*game).demo_scene) - 1))
        as u16;
    let hook = scene(game, address).map_or_else(
        || native_record(0x820000 | u32::from(address), game).word(16),
        |scene| scene.hook,
    );
    run_room_hook(0x820000 | u32::from(hook), game);
}

/// Run the demo room hook at `address` on a lease.
pub fn run_room_hook<H: crate::gameplay_frame::PassLease + ?Sized>(address: u32, game: &mut H) {
    match address {
        0x82891a => {
            game.session_mut().room.set_scroll(33, 0);
            game.words_stored();
        }
        0x828924 => {}
        0x828925 => {
            let [low, high] = 74u16.to_ne_bytes();
            crate::frame_display_byte!(game, Field::Screens1 => low);
            crate::frame_display_byte!(game, Field::Screens2 => high);
        }
        0x82892b => {
            let key = ActorKey::from_legacy(0);
            game.edit_actor(key, AccessMode::Native, |actor| actor.pre_instruction = 60);
        }
        0x828932 => crate::progress::set_boss_byte_at(game, 1, 1),
        _ => {
            game.rtl_unreachable();
        }
    }
}
