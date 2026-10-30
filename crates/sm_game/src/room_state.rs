//! Room state rules that run on named RAM and the owned-Session view.
#![forbid(unsafe_code)]
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
/// pose command and the imported Ceres rotation table.
pub trait MainCodeHost: SessionView {
    fn command_player(&mut self, command: u16);
    fn ceres_rotation_step(&mut self, word: isize) -> sm_data::ceres_elevator::RotationStep;
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
            h.write_word_le(0, fields::SCREEN_FADE_DELAY);
            h.write_word_le(0, fields::SCREEN_FADE_COUNTER);
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
