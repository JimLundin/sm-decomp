//! Room state rules that run on named RAM and the owned-Session view.
pub mod fx;
use crate::enemy_projectiles::host::Effects;
use crate::menu_state::SaveSelection;
use crate::ram::RawRam as _;
use crate::room_mechanisms::creation::spawn_hardcoded;
use crate::room_view::EffectWord;
use crate::runtime_access::SessionView;
use crate::session::HookSlot;
use crate::session_access::SessionAccess as _;

/// The level data ($7F:0000..$7F:9601) as the host holds it: its size word,
/// then the room terrain's cells (`Terrain::cells`), each cell's level word
/// ($7F:0002 + 2 * cell) and behaviour byte ($7F:6402 + cell), while a Session
/// is current, RAM otherwise. The intro and the credits decompress their
/// tilemaps over it.
pub trait LevelData {
    /// The level data's size in bytes ($7F:0000), little-endian.
    fn level_data_size(&self) -> u16;
    /// Cell `cell`'s level word, little-endian.
    fn level_word(&self, cell: usize) -> u16;
    /// Cell `cell`'s behaviour byte.
    fn level_behavior(&self, cell: usize) -> u8;
}

/// Named and raw work RAM, the loaded cartridge and its records, and the
/// Samus pose command.
pub trait Host:
    crate::display::preparation::Access
    + crate::ram::CompatibilityRam
    + crate::ram::WorkRamAccess
    + crate::word_transfers::LoadedRom
    + crate::records::Records
    + crate::menus::host::MenuRuntime
    + LevelData
{
}
impl<T> Host for T where
    T: crate::display::preparation::Access
        + crate::ram::CompatibilityRam
        + crate::ram::WorkRamAccess
        + crate::word_transfers::LoadedRom
        + crate::records::Records
        + crate::menus::host::MenuRuntime
        + LevelData
        + ?Sized
{
}

pub fn copy_level_planes<H: Host + ?Sized>(game: &mut H) {
    let size = usize::from(game.level_data_size());
    for (source, destination, count) in [
        (0x10002 + size + size / 2, 0x19602, size),
        (0x10002 + size, 0x16402, size / 2),
    ] {
        // Valid room arrays can overlap. Preserve their pre-copy contents;
        // malformed extents retain the checked forward loop's write prefix.
        if source < destination + count
            && destination < source + count
            && source + count <= 0x20000
            && destination + count <= 0x20000
        {
            crate::probe::hit(if destination == 0x19602 {
                "room.level.layer2_overlap"
            } else {
                "room.level.bts_overlap"
            });
            game.ram().copy_within(source..source + count, destination);
        } else {
            for index in 0..count {
                game.ram()
                    .write_byte(destination + index, game.ram().read_byte(source + index));
            }
        }
    }
}

/// Load the sprite palettes into the target palettes: colours 208..224 from
/// bank $9A, colours 224..240 from the current palette, and colours 128..144
/// (the beam's) from $9A:FC00.
pub fn load_sprite_palettes<H: Host + ?Sized>(game: &mut H) {
    use sm_data::cartridge::lorom_offset;
    let rom = game.loaded_rom_image();
    // The ROM words load in native order, as the original's word copies did.
    let rom_word = |address: u32, index: usize| {
        let base = usize::try_from(lorom_offset(address)).expect("LoROM offset");
        rom.table_word(base, isize::try_from(index).expect("palette index"))
    };
    let mut runtime = game.runtime();
    let visual = runtime.session_mut().presentation.visual_mut();
    // Each copy runs from its last word down, as the original's did.
    for index in (208..224).rev() {
        visual.targets[index] = rom_word(0x9a8000, index);
    }
    for index in (224..240).rev() {
        visual.targets[index] = visual.colors[index];
    }
    // The beam palette's bytes copy unchanged: its words keep their RAM byte order.
    for index in (0..16).rev() {
        let word = rom_word(0x9afc00, index);
        visual.targets[128 + index] = u16::from_le_bytes(word.to_ne_bytes());
    }
}

/// Start earthquake `kind` with a timer that never runs out.
fn start_earthquake<H: ScriptHost + ?Sized>(kind: u16, game: &mut H) {
    crate::gameplay_context::set_effect(game, EffectWord::EarthquakeKind, kind);
    crate::gameplay_context::set_effect(game, EffectWord::EarthquakeTimer, u16::MAX);
}

/// What room setup and door ASM need: the menus' runtime and the owned Session.
pub trait ScriptHost: crate::menus::host::Host {}
impl<T: crate::menus::host::Host + ?Sized> ScriptHost for T {}

/// Execute room setup ASM ($8F:E88F): the room state's setup routine
/// `script`. Was `RunRoomSetupCode`.
pub fn execute_setup_code<H: ScriptHost + ?Sized>(script: u16, game: &mut H) {
    use crate::constants::bank_87::*;
    use crate::scrolling_sky;
    match script {
        // Clear Crateria mainstreet escape passage if critters escaped and shake screen.
        0x9194 => {
            spawn_hardcoded(0x3d, 0xb, 0xbb30, game);
            start_earthquake(24, game);
        }
        // Make old Tourian escape shaft fake wall explode; spawn critters escape block.
        0x91a9 => spawn_hardcoded(0x10, 0x87, 0xb964, game),
        0x91b2 => spawn_hardcoded(0xf, 0xa, 0xb9ed, game),
        // Scrolling sky land, shake screen; scrolling sky land; ocean.
        0x91bd => {
            start_earthquake(6, game);
            scrolling_sky::initialize_land(game);
        }
        0x91c9 => scrolling_sky::initialize_land(game),
        0x91ce => scrolling_sky::initialize_ocean(game),
        // Run statue unlocking animations.
        0x91d7 => {
            for animation in [
                ADDR_K_ANIMTILES_TOURIAN_STATUE_KRAID,
                ADDR_K_ANIMTILES_TOURIAN_STATUE_PHANTOON,
                ADDR_K_ANIMTILES_TOURIAN_STATUE_DRAYGON,
                ADDR_K_ANIMTILES_TOURIAN_STATUE_RIDLEY,
            ] {
                crate::animated_tiles::execution::leased::spawn(animation as u16, game);
            }
        }
        // Spawn pre-Phantoon room enemy projectile.
        0xc8c8 => {
            use crate::constants::bank_86::ADDR_K_EPROJ_PRE_PHANTOM_ROOM;
            crate::enemy_projectiles::lifecycle::spawn_room_leased(
                ADDR_K_EPROJ_PRE_PHANTOM_ROOM as u16,
                0xc8c8,
                game,
            );
        }
        // Shaktool's room.
        0xc8d3 => spawn_hardcoded(0, 0, 0xb8eb, game),
        // Set pausing code for Draygon: native-order addresses, then the bank.
        0xc8dd => {
            let mut runtime = game.runtime();
            let room = &mut runtime.session_mut().room;
            for (hook, address) in [(HookSlot::Pause, 0xc8f6u16), (HookSlot::Unpause, 0xc8fc)] {
                room.set_hook_address(hook, address.to_ne_bytes());
                room.set_hook_bank(hook, 0x8f);
            }
        }
        // Set collected map.
        0xc90a => {
            let area = usize::from(crate::gameplay_context::navigation(game).area);
            let at = 0xd908 + area;
            let collected =
                u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) | 1;
            // The area's map station byte; the word store's high byte keeps the
            // next area's.
            crate::runtime_access::with_session(game, |session| {
                session.gameplay_flags.map_stations[area] = collected.to_ne_bytes()[0];
            });
            crate::menus::state::with(game, |_, selection: &mut SaveSelection| {
                selection.has_area_map = collected;
            });
        }
        // Set Zebes timebomb event and set light horizontal room shaking.
        0xc91f => {
            crate::progress::leased::mark_event(0xe, game);
            start_earthquake(18, game);
        }
        // Light horizontal shaking; rising acid and medium horizontal shaking.
        0xc933 | 0xc953 => {
            let kind = if script == 0xc953 {
                spawn_hardcoded(0x10, 0x10, 0xb968, game);
                21
            } else {
                18
            };
            crate::gameplay_context::set_effect(game, EffectWord::EarthquakeKind, kind);
            {
                let mut runtime = game.runtime();
                let room = &mut runtime.session_mut().room;
                room.set_main_native(1, kind);
                room.set_main_native(0, 0);
            }
            crate::gameplay_context::set_effect(game, EffectWord::EarthquakeTimer, u16::MAX);
        }
        // Set medium horizontal room shaking.
        0xc946 => {
            start_earthquake(21, game);
        }
        // Ceres door to solid blocks, Ceres haze, BG1/2 tiles base and haze.
        0xc96e => {
            spawn_hardcoded(0xf, 0x26, 0xba48, game);
            crate::room_visuals::initialize_room_haze(game);
        }
        0xc976 => crate::room_visuals::initialize_room_haze(game),
        0xc97b => {
            game.ram().write_byte(0x5d, 102);
            crate::room_visuals::initialize_room_haze(game);
            crate::runtime_access::with_session(game, |session| {
                session.room.ceres_hdma_table = 9;
            });
        }
        0x91bb | 0x91bc | 0x91d3 | 0x91d4 | 0x91d5 | 0x91d6 | 0x91f4 | 0x91f5 | 0x91f6 | 0x91f7
        | 0xc8c7 | 0xc8d0 | 0xc8d1 | 0xc8d2 | 0xc8dc | 0xc91e => {}
        _ => {
            game.rtl_unreachable();
        }
    }
}

/// Execute door ASM ($8F:E8A3): the door's routine `script`, or the scroll
/// updates its pointer names. Was `RunDoorSetupCode`.
pub fn execute_door_code<H: ScriptHost + ?Sized>(script: u16, game: &mut H) {
    use crate::constants::bank_87::*;
    match script {
        // Start Wrecked Ship treadmill - west / east entrance.
        0xb971 | 0xe1d8 => {
            let (animation, mechanism) = if script == 0xb971 {
                (ADDR_K_ANIMTILES_WRECKED_SHIP_TRADMILL_RIGHT, 0xb64b)
            } else {
                (ADDR_K_ANIMTILES_WRECKED_SHIP_TRADMILL_LEFT, 0xb64f)
            };
            crate::animated_tiles::execution::leased::spawn(animation as u16, game);
            spawn_hardcoded(4, 9, mechanism, game);
        }
        // Set up elevatube from south / north.
        0xe26c | 0xe291 => {
            {
                let mut runtime = game.runtime();
                let room = &mut runtime.session_mut().room;
                if script == 0xe26c {
                    room.set_main_native(2, (-256i16) as u16);
                    room.set_main_native(1, 2496);
                    room.set_main_native(3, (-32i16) as u16);
                } else {
                    // The original strings write little-endian bytes on every host.
                    room.set_main_native(2, 256);
                    room.main_variables[1] = 64;
                    room.main_variables[3] = 32;
                }
            }
            game.player_command(0);
            spawn_hardcoded(1, 0, 0xb8f9, game);
        }
        // Reset elevatube on north / south exit.
        0xe301 | 0xe309 => {
            if script == 0xe309 {
                // Scroll cells 0 and 1, in native order.
                let mut runtime = game.runtime();
                runtime
                    .session_mut()
                    .room
                    .set_scroll_pair(0, 514u16.to_ne_bytes());
            }
            game.player_command(1);
        }
        // To Ceres elevator shaft.
        0xe4e0 => {
            game.ram().write_byte(0x56, 7);
            game.apply(
                "ceres elevator shaft",
                crate::display::video::Command::Control(
                    crate::display::video::Control::BackgroundMode(7),
                ),
            );
            {
                crate::frame_display_word!(game.AffineWords0Pair = 256);
                crate::frame_display_word!(game.AffineWords3Pair = 256);
            }
            {
                crate::frame_display_word!(game.AffineWords1Pair = 0);
                crate::frame_display_word!(game.AffineWords2Pair = 0);
            }
            {
                crate::frame_display_word!(game.AffineWords4Pair = 128);
                crate::frame_display_word!(game.AffineWords5Pair = 1008);
            }
            let mut runtime = game.runtime();
            let room = &mut runtime.session_mut().room;
            room.mode7_irq = 1;
            room.main_variables[0] = 34;
            room.main_variables[1] = 60;
        }
        // From Ceres elevator shaft.
        0xe513 => {
            game.ram().write_byte(0x56, 9);
            crate::runtime_access::with_session(game, |session| session.room.mode7_irq = 0);
        }
        _ => {
            if !crate::room_scrolls::apply(0x8f0000 | u32::from(script), &mut game.runtime()) {
                game.rtl_unreachable();
            }
        }
    }
}

/// What the room main code needs beyond the owned-Session view: the Samus
/// pose command, the imported Ceres rotation table, and the sprite object,
/// sky, projectile and cutscene effects of the compatibility runtime.
pub trait MainCodeHost: SessionView + crate::display::preparation::Access {
    fn command_player(&mut self, command: u16);
    fn ceres_rotation_step(&mut self, word: isize) -> sm_data::ceres_elevator::RotationStep;
    /// Create sprite object `kind` at (`x`, `y`) ($B4:BC26).
    fn create_explosion(&mut self, x: u16, y: u16, kind: u16);
    /// Update the scrolling sky land or ocean tilemap ($88:AF8D, $88:AF99).
    fn queue_sky_rows(&mut self, ocean: bool);
    /// Spawn enemy projectile `definition` with room graphics ($86:8027).
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16);
    /// Handle Ceres Ridley getaway cutscene ($A6:A35A, via $8F:E571).
    fn handle_ceres_getaway(&mut self);
}

/// Execute room main ASM ($8F:E8BD): the room's main ASM routine `script`
/// for this frame. Was `RunRoomMainCode`.
pub fn execute_main_code<H: MainCodeHost>(script: u16, h: &mut H) {
    use crate::constants::bank_86::{ADDR_STRU_869734, ADDR_STRU_869742};
    match script {
        // Main ASM - scrolling sky land / ocean.
        0xc116 => h.queue_sky_rows(false),
        0xc11b => h.queue_sky_rows(true),
        // Main ASM - scrolling sky land, Zebes timebomb set; and the shake
        // screen and generate random explosions it continues into.
        0xc120 | 0xc124 => {
            if script == 0xc120 {
                h.queue_sky_rows(false);
            }
            random_explosion(1, true, h);
            h.session_mut().feedback.earthquake_timer |= 0x8000;
            h.words_stored();
        }
        // Main ASM - scroll screen right in Dachora room.
        0xc1e6 => {
            if h.byte(0xcd2b) == 2
                && h.session().room.camera.layer1[1] < 0x500
                && h.session().room.camera.layer1[0] < 0x380
            {
                let x = h.session().room.camera.layer1[0] + 3;
                crate::camera::set_layer1(h, 0, x);
            }
        }
        0xe2b6 => advance_elevatube(h),
        0xe51f => advance_ceres_elevator(h),
        0xe524 => {}
        // Main ASM - spawn Ceres pre elevator hall falling debris.
        0xe525 => {
            if h.session().room.ceres_status != 0 {
                let room = &mut h.session_mut().room;
                room.main_variables[0] = room.main_variables[0].wrapping_sub(1);
                if (room.main_native(0) as i16) < 0 {
                    room.main_variables[0] = 8;
                    let definition = if (h.session().timing.random_number & 0x8000) != 0 {
                        ADDR_STRU_869742
                    } else {
                        ADDR_STRU_869734
                    } as u16;
                    let x = [
                        0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x110,
                        0x130, 0x150, 0x170, 0x190,
                    ][usize::from(h.session().timing.random_number & 15)];
                    h.spawn_room_projectile(definition, x);
                }
            }
        }
        // Main ASM - handle Ceres Ridley getaway cutscene.
        0xe571 => {
            if (h.session().room.ceres_status & 1) != 0 {
                h.handle_ceres_getaway();
            }
        }
        // Main ASM - shake screen switching between light horizontal and
        // medium diagonal.
        0xe57c => {
            if let Some(kind) = select_earthquake(18, 0x200, 23, h) {
                h.session_mut().feedback.earthquake_kind = kind;
                h.words_stored();
            }
            random_explosion(3, false, h);
        }
        // Main ASM - generate random explosion on every fourth frame.
        0xe5a0 => random_explosion(3, false, h),
        // Main ASM - shake screen switching between medium horizontal and
        // strong diagonal.
        0xe5a4 => {
            if let Some(kind) = select_earthquake(21, 0x180, 26, h) {
                h.session_mut().room.set_main_native(1, kind);
            }
            let kind = h.session().room.main_native(1);
            h.session_mut().feedback.earthquake_kind = kind;
            h.words_stored();
            random_explosion(3, false, h);
        }
        0xe8cd => shake_crocomire(h),
        0xe950 => shake_ridley(h),
        _ => Effects::unreachable(h),
    }
}

/// Generate random explosion on even frames on random non-blank tile
/// ($8F:C131, `on_tile`) or on every fourth frame ($8F:C183), then at
/// ([$12], [$14]) ($8F:C1A9).
fn random_explosion<H: MainCodeHost>(mask: u16, on_tile: bool, h: &mut H) {
    if h.session().timing.time_frozen != 0
        || (crate::frame_display_word!(h.NmiFrameWordPair) & mask) != 0
    {
        return;
    }
    let random = h.next_random();
    let layer1 = h.session().room.camera.layer1;
    let x = layer1[0].wrapping_add(random as u8 as u16);
    let y = layer1[1].wrapping_add(u16::from(random.to_ne_bytes()[1]));
    // X on entry to $8F:C1A9: 0 from the room main ASM dispatch, or the
    // block index the tile check ($8F:C131) left.
    let mut entry_x = 0;
    if on_tile {
        let row = u16::from((y >> 4) as u8) * u16::from(h.session().room.width as u8);
        entry_x = row.wrapping_add(x >> 4).wrapping_mul(2);
        let block = match h
            .session()
            .room
            .terrain
            .cells()
            .get(usize::from(entry_x / 2))
        {
            // The native-order word the RAM bytes hold.
            Some(cell) => u16::from_ne_bytes(cell.level.to_le_bytes()),
            // LEGACY(original-overflow): the random tile's block index past the
            // 3200h level-data cells reads $7F:0002 + index anyway.
            None => crate::legacy_overflow::ram_native_word(h, 0x10002 + usize::from(entry_x)),
        };
        if (block & 0x3ff) == 255 {
            return;
        }
    }
    let random = h.next_random() & 15;
    if random < 8 {
        let sound = [0x24, 0, 0, 0x25, 0, 0, 0, 0][usize::from(random)];
        if sound != 0 {
            Effects::queue_sound(h, 2, 6, sound);
        }
    }
    // $8F:C1B5 loads X only for a random value below 8; otherwise the
    // sprite object ID comes from the X it entered with.
    let index = if random < 8 { random } else { entry_x & 7 };
    let kind = [3, 3, 9, 12, 12, 18, 18, 21][usize::from(index)];
    h.create_explosion(x, y, kind);
}

/// The diagonal earthquake's countdown: when it runs out, the `ending`
/// earthquake type; when a random number falls below `threshold`, a 2Ah
/// frame `starting` earthquake.
fn select_earthquake<H: MainCodeHost>(
    ending: u16,
    threshold: u16,
    starting: u16,
    h: &mut H,
) -> Option<u16> {
    let timer = h.session().room.main_native(0);
    if timer != 0 {
        let remaining = timer.wrapping_sub(1);
        h.session_mut().room.set_main_native(0, remaining);
        (remaining == 0).then_some(ending)
    } else if h.next_random() < threshold {
        h.session_mut().room.main_variables[0] = 42;
        Some(starting)
    } else {
        None
    }
}

pub fn advance_ceres_elevator<H: MainCodeHost + ?Sized>(h: &mut H) {
    if (h.session().room.ceres_status & 0x8000) != 0 {
        if (112i16).wrapping_sub(h.session().player.x as i16) < 0
            && (144i16).wrapping_sub(h.session().player.x as i16) >= 0
            && (h.session().player.y as i32 - 128i32) & 0x8000 != 0
            && (h.session().player.y as i32 - 75i32) & 0x8000 == 0
            && h.session().player.contact.y_speed.whole == 0
            && h.session().player.contact.y_speed.fraction == 0
            && h.session().game_loop.mode == 8
        {
            h.command_player(2);
            h.session_mut().presentation.visual_mut().screen_fade = Default::default();
            h.session_mut().game_loop.mode = 32;
            h.words_stored();
        }
        // Variable 1 counts frames to the next rotation step; variable 0 is
        // the signed step index, negative while rotating back.
        let room = &mut h.session_mut().room;
        room.main_variables[1] = room.main_variables[1].wrapping_sub(1);
        if (room.main_native(1) as i16) < 0 {
            // Each rotation step is three words.
            let step_word = 6u16.wrapping_mul(room.main_native(0)) >> 1;
            let step = h.ceres_rotation_step(step_word as isize);
            h.session_mut().room.set_main_native(1, step.frames);
            // The Mode 7 matrix: B, C = -B, A, then D = A.
            crate::frame_display_word!(h.AffineWords1Pair = step.mode7_b);
            let b = crate::frame_display_word!(h.AffineWords1Pair);
            crate::frame_display_word!(h.AffineWords2Pair = b.wrapping_neg());
            crate::frame_display_word!(h.AffineWords0Pair = step.mode7_a);
            let a = crate::frame_display_word!(h.AffineWords0Pair);
            crate::frame_display_word!(h.AffineWords3Pair = a);
            let index = h.session().room.main_native(0);
            let next = if (index as i16) < 0 {
                if index == 0x8001 {
                    0
                } else {
                    (index as i16).wrapping_sub(1)
                }
            } else if index == 67 {
                -32700i16
            } else {
                (index as i16).wrapping_add(1)
            };
            h.session_mut().room.set_main_native(0, next as u16);
        }
    }
}

pub fn advance_elevatube<H: SessionView>(h: &mut H) {
    use crate::player_collision::operations::{self, Axis, DisplacementRequest};
    h.set_player_x(128);
    h.set_player_x_fraction(0);
    // Variables 0 and 1 are the 16.16 position, 2 the velocity and 3 the
    // acceleration, as the original loads them.
    let room = &h.session().room;
    let amount = i32::from(room.main_native(2) as i16) << 8;
    let [low, high] = [room.main_variables[0], room.main_variables[1]].map(u16::to_le_bytes);
    let position = u32::from_ne_bytes([low[0], low[1], high[0], high[1]]);
    let whole = (position.wrapping_add(amount as u32) >> 16) as u16;
    let room = &mut h.session_mut().room;
    room.main_variables[0] = (i32::from(room.main_variables[0]) + amount) as u16;
    room.set_main_native(1, whole);
    operations::displace(
        h,
        DisplacementRequest::TerrainOnly {
            axis: Axis::Y,
            amount,
        },
    );
    let room = &mut h.session_mut().room;
    if ((i32::from(room.main_native(3)) + i32::from(room.main_native(2)) + 3616) as u16) < 0x1c41 {
        room.main_variables[2] = room.main_variables[2].wrapping_add(room.main_native(3));
    }
}

pub fn shake_crocomire<H: SessionView + ?Sized>(h: &mut H) {
    use crate::actors::access::{AccessMode::Native, ActorKey};
    let body = ActorKey::from_legacy(0);
    let legs = ActorKey::from_legacy(64);
    if h.resolve_actor(body, Native).properties & 0x200 != 0 {
        return;
    }
    if h.resolve_actor(body, Native).variables[0] == 64 {
        let shift = h.resolve_actor(legs, Native).variables[3];
        h.set_scroll_word(
            crate::display_scroll::Word::Bg1V,
            shift
                .wrapping_add(h.session().room.camera.bg1_offset[1])
                .wrapping_add(h.session().room.camera.layer1[1]),
        );
    } else if h.resolve_actor(body, Native).variables[1] & 0x400 != 0 {
        let timer = h.edit_actor(legs, Native, |actor| {
            actor.variables[3] = actor.variables[3].wrapping_sub(1);
            actor.variables[3]
        });
        let shift = if (i32::from(timer) + 7) & 0x8000 != 0 {
            (i32::from(timer) - 2 * (i32::from(timer) + 7)) as u16
        } else {
            timer
        };
        h.set_scroll_word(
            crate::display_scroll::Word::Bg1V,
            h.scroll_word(crate::display_scroll::Word::Bg1V)
                .wrapping_add(shift),
        );
        h.set_scroll_word(crate::display_scroll::Word::Bg2V, shift.wrapping_sub(48));
    } else if h.resolve_actor(body, Native).variables[2] == 34
        && h.resolve_actor(body, Native).variables[3] != 0
    {
        let timer = h.edit_actor(body, Native, |actor| {
            actor.variables[3] = actor.variables[3].wrapping_sub(1);
            actor.variables[3]
        });
        let x = h.session().room.camera.layer1[0];
        crate::camera::set_layer1(
            h,
            0,
            if timer & 1 != 0 {
                x.wrapping_sub(4)
            } else {
                x.wrapping_add(4)
            },
        );
    }
}

pub fn shake_ridley<H: SessionView + ?Sized>(h: &mut H) {
    use crate::actors::access::{AccessMode::Native, ActorKey};
    let actor = ActorKey::from_legacy(256);
    let timer = h.resolve_actor(actor, Native).variables[0];
    if timer == 0 {
        return;
    }
    let index = usize::from(timer - 1);
    h.edit_actor(actor, Native, |actor| actor.variables[0] = timer - 1);
    for (table, registers) in [
        (
            [0, 2, 2, 2, 0, -2, -2, -2],
            [
                crate::display_scroll::Word::Bg1H,
                crate::display_scroll::Word::Bg2H,
            ],
        ),
        (
            [-2, -2, 0, 2, 2, 2, 0, -2],
            [
                crate::display_scroll::Word::Bg1V,
                crate::display_scroll::Word::Bg2V,
            ],
        ),
    ] {
        let shift = table[index] as u16;
        for register in registers {
            h.set_scroll_word(register, h.scroll_word(register).wrapping_add(shift));
        }
    }
}

/// What loading the room's enemy graphics needs: named RAM, immediate DMA,
/// and the original's RomPtr fetches with their observations.
pub trait EnemyGraphicsHost:
    crate::ram::CompatibilityRam + crate::display::transfer::RoomDma
{
    fn rom_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
}

/// Load enemy GFX to VRAM ($82:DFD1): each of the room's enemy tilesets
/// (bank $B4) to its fixed VRAM slot, or packed from $7000 when the size's
/// top bit is clear.
/// `tilesets` is the room state's enemy tileset list (`RoomEntry::enemy_tilesets_key`).
pub fn load_enemy_graphics<H: EnemyGraphicsHost + ?Sized>(tilesets: u16, game: &mut H) {
    use crate::actors::content_records::EnemyDef;
    use core::mem::offset_of;
    // Enemy tileset entries ($B4): the enemy definition, then its VRAM slot.
    const ENEMY_DEF: usize = 0;
    const VRAM_DESTINATION: usize = 2;
    let mut cursor = i32::from(tilesets);
    if cursor == 0 {
        return;
    }
    let mut next_tile = 0x7000u16;
    loop {
        // Reacquire each bank address: this list wraps, unlike initial staging.
        let enemy = game.rom_record(0xb4, cursor as u16).word(ENEMY_DEF);
        if enemy == 0xffff {
            return;
        }
        let definition = game.rom_record(0xa0, enemy);
        let size = definition.word(offset_of!(EnemyDef, tile_data_size));
        let destination = if (size & 0x8000) != 0 {
            let tileset = game.rom_record(0xb4, cursor as u16);
            ((tileset.word(VRAM_DESTINATION) & 0xf000) >> 4) | 0x6000
        } else {
            let destination = next_tile;
            next_tile = next_tile.wrapping_add(size >> 1);
            destination
        };
        let source = definition.load24(offset_of!(EnemyDef, tile_data));
        crate::display::transfer::copy_to_vram_now(destination, source, size & 0x7fff, None, game);
        cursor += 4;
    }
}
