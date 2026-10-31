//! Room state rules that run on named RAM and the owned-Session view.
use crate::enemy_projectiles::host::Effects;
use crate::ram::RawRam as _;
use crate::ram::{checked_word, words as fields};
use crate::runtime_access::SessionView;
use crate::word_transfers;

/// Named and raw work RAM, the loaded cartridge and its records, and the
/// Samus pose command.
pub trait Host:
    crate::ram::CompatibilityRam
    + crate::ram::WorkRamAccess
    + crate::word_transfers::LoadedRom
    + crate::records::Records
    + crate::menus::host::MenuRuntime
{
}
impl<T> Host for T where
    T: crate::ram::CompatibilityRam
        + crate::ram::WorkRamAccess
        + crate::word_transfers::LoadedRom
        + crate::records::Records
        + crate::menus::host::MenuRuntime
        + ?Sized
{
}

pub fn copy_level_planes<H: Host + ?Sized>(game: &mut H) {
    let size = usize::from(checked_word!(game.RAM7_F_START));
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

pub fn load_sprite_palettes<H: Host + ?Sized>(game: &mut H) {
    word_transfers!(game, descending, 16; le(TARGET_PALETTES, 223) <- rom(0x9a8000, 223));
    word_transfers!(game, descending, 16; native(0xc3de) <- native(0xc1de));
    word_transfers!(game, descending, 16; native(0xc31e) <- rom(0x9afc00, 15));
}

/// What the room main code needs beyond the owned-Session view: the Samus
/// pose command, the imported Ceres rotation table, and the sprite object,
/// sky, projectile and cutscene effects of the compatibility runtime.
pub trait MainCodeHost: SessionView {
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
            h.write_word_le(
                h.word(fields::EARTHQUAKE_TIMER) | 0x8000,
                fields::EARTHQUAKE_TIMER,
            );
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
            if h.word(fields::CERES_STATUS) != 0 {
                let room = &mut h.session_mut().room;
                room.main_variables[0] = room.main_variables[0].wrapping_sub(1);
                if (room.main_native(0) as i16) < 0 {
                    room.main_variables[0] = 8;
                    let definition = if (h.word(fields::RANDOM_NUMBER) & 0x8000) != 0 {
                        ADDR_STRU_869742
                    } else {
                        ADDR_STRU_869734
                    } as u16;
                    let x = [
                        0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x110,
                        0x130, 0x150, 0x170, 0x190,
                    ][usize::from(h.word(fields::RANDOM_NUMBER) & 15)];
                    h.spawn_room_projectile(definition, x);
                }
            }
        }
        // Main ASM - handle Ceres Ridley getaway cutscene.
        0xe571 => {
            if (h.word(fields::CERES_STATUS) & 1) != 0 {
                h.handle_ceres_getaway();
            }
        }
        // Main ASM - shake screen switching between light horizontal and
        // medium diagonal.
        0xe57c => {
            if let Some(kind) = select_earthquake(18, 0x200, 23, h) {
                h.write_word_le(kind, fields::EARTHQUAKE_TYPE);
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
            h.write_word_le(h.session().room.main_native(1), fields::EARTHQUAKE_TYPE);
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
    if h.word(fields::TIME_IS_FROZEN_FLAG) != 0
        || (h.word(fields::NMI_FRAME_COUNTER_WORD) & mask) != 0
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
        let row = u16::from((y >> 4) as u8) * u16::from(h.word(fields::ROOM_WIDTH_IN_BLOCKS) as u8);
        entry_x = row.wrapping_add(x >> 4).wrapping_mul(2);
        if (h.native_word(0x10002 + usize::from(entry_x)) & 0x3ff) == 255 {
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
    if (h.word(fields::CERES_STATUS) & 0x8000) != 0 {
        if (112i16).wrapping_sub(h.session().player.x as i16) < 0
            && (144i16).wrapping_sub(h.session().player.x as i16) >= 0
            && (h.session().player.y as i32 - 128i32) & 0x8000 != 0
            && (h.session().player.y as i32 - 75i32) & 0x8000 == 0
            && h.word(fields::SAMUS_Y_SPEED) == 0
            && h.word(fields::SAMUS_Y_SUBSPEED) == 0
            && h.word(fields::GAME_STATE) as i32 == 8
        {
            h.command_player(2);
            h.session_mut().presentation.visual_mut().screen_fade = Default::default();
            h.write_word_le(32, fields::GAME_STATE);
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
            h.write_word_le(step.mode7_b, fields::REG_M7B);
            h.write_word_le(-(h.word(fields::REG_M7B) as i32) as u16, fields::REG_M7C);
            h.write_word_le(step.mode7_a, fields::REG_M7A);
            h.write_word_le(h.word(fields::REG_M7A), fields::REG_M7D);
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
        h.write_word_le(
            shift
                .wrapping_add(h.session().room.camera.bg1_offset[1])
                .wrapping_add(h.session().room.camera.layer1[1]),
            fields::REG_BG1VOFS,
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
        h.write_word_le(
            h.word(fields::REG_BG1VOFS).wrapping_add(shift),
            fields::REG_BG1VOFS,
        );
        h.write_word_le(shift.wrapping_sub(48), fields::REG_BG2VOFS);
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
            [fields::REG_BG1HOFS, fields::REG_BG2HOFS],
        ),
        (
            [-2, -2, 0, 2, 2, 2, 0, -2],
            [fields::REG_BG1VOFS, fields::REG_BG2VOFS],
        ),
    ] {
        let shift = table[index] as u16;
        for register in registers {
            h.write_word_le(h.word(register).wrapping_add(shift), register);
        }
    }
}
