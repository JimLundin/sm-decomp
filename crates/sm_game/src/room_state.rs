//! Room state rules that run on named RAM and the owned-Session view.
#![forbid(unsafe_code)]
use crate::ram::RawRam as _;
use crate::ram::{checked_word, word, words as fields};
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
    let size = usize::from(checked_word!(game.ram7F_start));
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
    word_transfers!(game, descending, 16; le(target_palettes, 223) <- rom(0x9a8000, 223));
    word_transfers!(game, descending, 16; native(0xc3de) <- native(0xc1de));
    word_transfers!(game, descending, 16; native(0xc31e) <- rom(0x9afc00, 15));
}

pub fn advance_ceres_elevator<H: Host + ?Sized>(game: &mut H) {
    let mut v1: i16 = 0;
    if (word!(game.ceres_status) & 0x8000) != 0 {
        if (112i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0
            && (144i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0
            && word!(game.samus_y_pos) as i32 - 128i32 & 0x8000 != 0
            && word!(game.samus_y_pos) as i32 - 75i32 & 0x8000 == 0
            && word!(game.samus_y_speed) == 0
            && word!(game.samus_y_subspeed) == 0
            && word!(game.game_state) as i32 == 8
        {
            game.player_command(2);
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            word!(game.game_state = 32);
        }
        let countdown = word!(@read game, 0x7e3);
        word!(@write game, 0x7e3, countdown.wrapping_sub(1));
        if (u16::from_ne_bytes([game.ram().read_byte(0x7e3), game.ram().read_byte(0x7e4)]) as i16)
            < 0
        {
            let v0: i32 = ((6u16).wrapping_mul(u16::from_ne_bytes([
                game.ram().read_byte(0x7e1),
                game.ram().read_byte(0x7e2),
            ]))) as i32
                >> 1;
            let step = crate::ceres_elevator::step(game, v0 as isize);
            word_transfers!(game, ascending, 1; native(0x7e3) <- value(step.frames));
            word!(game.reg_M7B = step.mode7_b);
            word!(game.reg_M7C = -(word!(game.reg_M7B) as i32) as u16);
            word!(game.reg_M7A = step.mode7_a);
            word!(game.reg_M7D = word!(game.reg_M7A));
            if (u16::from_ne_bytes([game.ram().read_byte(0x7e1), game.ram().read_byte(0x7e2)])
                as i16)
                < 0
            {
                v1 = ((u16::from_ne_bytes([
                    game.ram().read_byte(0x7e1),
                    game.ram().read_byte(0x7e2),
                ])) as i16)
                    .wrapping_sub(1);
                if u16::from_ne_bytes([game.ram().read_byte(0x7e1), game.ram().read_byte(0x7e2)])
                    == 0x8001
                {
                    v1 = 0;
                }
            } else {
                v1 = ((u16::from_ne_bytes([
                    game.ram().read_byte(0x7e1),
                    game.ram().read_byte(0x7e2),
                ])) as i16)
                    .wrapping_add(1);
                if u16::from_ne_bytes([game.ram().read_byte(0x7e1), game.ram().read_byte(0x7e2)])
                    == 67
                {
                    v1 = -32700i16;
                }
            }
            word_transfers!(game, ascending, 1; native(0x7e1) <- value(v1 as u16));
        }
    }
}

pub fn advance_elevatube<H: SessionView>(h: &mut H) {
    use crate::player_collision::operations::{self, Axis, DisplacementRequest};
    h.write_word_le(128, fields::samus_x_pos);
    h.write_word_le(0, fields::samus_x_subpos);
    let amount = i32::from(h.native_word(0x7e5) as i16) << 8;
    let position = u32::from_ne_bytes([h.byte(0x7e1), h.byte(0x7e2), h.byte(0x7e3), h.byte(0x7e4)]);
    let whole = (position.wrapping_add(amount as u32) >> 16) as u16;
    h.write_slice_le(0x7e1, (i32::from(h.word(0x7e1)) + amount) as u16);
    h.write_slice_native(0x7e3, whole);
    operations::displace(
        h,
        DisplacementRequest::TerrainOnly {
            axis: Axis::Y,
            amount,
        },
    );
    if ((i32::from(h.native_word(0x7e7)) + i32::from(h.native_word(0x7e5)) + 3616) as u16) < 0x1c41
    {
        h.write_slice_le(0x7e5, h.word(0x7e5).wrapping_add(h.native_word(0x7e7)));
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
                .wrapping_add(h.word(fields::bg1_y_offset))
                .wrapping_add(h.word(fields::layer1_y_pos)),
            fields::reg_BG1VOFS,
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
            h.word(fields::reg_BG1VOFS).wrapping_add(shift),
            fields::reg_BG1VOFS,
        );
        h.write_word_le(shift.wrapping_sub(48), fields::reg_BG2VOFS);
    } else if h.resolve_actor(body, Native).variables[2] == 34
        && h.resolve_actor(body, Native).variables[3] != 0
    {
        let timer = h.edit_actor(body, Native, |actor| {
            actor.variables[3] = actor.variables[3].wrapping_sub(1);
            actor.variables[3]
        });
        let x = h.word(fields::layer1_x_pos);
        h.write_word_le(
            if timer & 1 != 0 {
                x.wrapping_sub(4)
            } else {
                x.wrapping_add(4)
            },
            fields::layer1_x_pos,
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
            [fields::reg_BG1HOFS, fields::reg_BG2HOFS],
        ),
        (
            [-2, -2, 0, 2, 2, 2, 0, -2],
            [fields::reg_BG1VOFS, fields::reg_BG2VOFS],
        ),
    ] {
        let shift = table[index] as u16;
        for register in registers {
            h.write_word_le(h.word(register).wrapping_add(shift), register);
        }
    }
}
