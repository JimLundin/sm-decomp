//! Exact numeric identities and live execution; encoded access stays owner-private.
#![forbid(unsafe_code)]
use super::{
    Owner,
    state::Column::{self, *},
};
use crate::game::music::QueueMusic_Delayed8;
use crate::game::sm_80::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::ram_map::words as ram;
use crate::rom::RomRecord;

fn field(owner: &Owner<'_>, column: Column, index: u16) -> u16 {
    owner.read_le(column.address(index))
}
fn put(owner: &mut Owner<'_>, column: Column, index: u16, value: u16) {
    owner.write_slice_le(column.address(index), value);
}
fn decrement(owner: &mut Owner<'_>, column: Column, index: u16) -> u16 {
    let old = field(owner, column, index);
    put(owner, column, index, old.wrapping_sub(1));
    old
}
fn miniboss_dead(owner: &Owner<'_>) -> bool {
    owner.read_native(0xd828 + usize::from(owner.read_le(ram::area_index))) & 2 != 0
}
/// High-window records use the current owned image. Low windows preserve the
/// exact old diagnostic and g_fail side effect, after publishing pending state.
fn record(owner: Owner<'_>, address: u16) -> (Owner<'_>, RomRecord) {
    if address & 0x8000 == 0 {
        return owner.foreign(|g| super::diagnostics::low_record(address, g));
    }
    let image = owner.rom();
    let base = crate::rtl::lorom_offset(0x8d0000 | u32::from(address)) as usize;
    (owner, RomRecord { image, base })
}
pub(super) fn spawn(mut owner: Owner<'_>, definition: u16) -> Owner<'_> {
    let Some(slot) = (0..8)
        .rev()
        .map(|i| 2 * i)
        .find(|&i| field(&owner, Id, i) == 0)
    else {
        return owner;
    };
    crate::verify_probe::hit("palette.spawn.allocated");
    put(&mut owner, Id, slot, definition);
    put(&mut owner, Color, slot, 0);
    put(&mut owner, PreAction, slot, 0xc526);
    let data;
    (owner, data) = record(owner, definition);
    put(&mut owner, Program, slot, data.word(2));
    put(&mut owner, Delay, slot, 1);
    put(&mut owner, Counter, slot, 0);
    let data;
    (owner, data) = record(owner, definition);
    setup(owner, 0x8d0000 | u32::from(data.word(0)), slot)
}
pub(super) fn advance(mut owner: Owner<'_>) -> Owner<'_> {
    if owner.read_le(ram::flag_for_palette_fx_objects) & 0x8000 != 0 {
        let mut slot = 14i32;
        while slot >= 0 {
            owner.write_slice_le(ram::palettefx_index, slot as u16);
            if field(&owner, Id, slot as u16) != 0 {
                owner = process(owner, slot as u16);
                slot = i32::from(owner.read_le(ram::palettefx_index));
            }
            slot -= 2;
        }
    }
    owner
}
pub(super) fn process(mut owner: Owner<'_>, k: u16) -> Owner<'_> {
    let before = 0x8d0000 | u32::from(field(&owner, PreAction, k));
    owner = pre(owner, before, k);
    let mut slot = owner.read_le(ram::palettefx_index);
    if decrement(&mut owner, Delay, slot) != 1 {
        return owner;
    }
    let mut cursor = field(&owner, Program, slot);
    let duration = loop {
        let data;
        (owner, data) = record(owner, cursor);
        let instruction = data.word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        (owner, slot, cursor) = command(
            owner,
            0x8d0000 | u32::from(instruction),
            slot,
            cursor.wrapping_add(2),
        );
        if cursor == 0 {
            return owner;
        }
    };
    put(&mut owner, Delay, slot, duration);
    crate::verify_probe::hit("palette.phase.colors");
    let mut color = field(&owner, Color, slot);
    loop {
        let instruction = loop {
            let data;
            (owner, data) = record(owner, cursor);
            let value = data.word(2);
            if value & 0x8000 != 0 {
                break value;
            }
            // This is a byte displacement, not a bounded palette index. Apply each
            // write immediately: later script commands and other owners can observe it.
            owner.write_slice_le(0xc000 + usize::from(color & !1), value);
            crate::verify_probe::hit("palette.color.write");
            color = color.wrapping_add(2);
            cursor = cursor.wrapping_add(2);
        };
        (owner, color, cursor) = command(owner, 0x8d0000 | u32::from(instruction), color, cursor);
        if cursor == 0 {
            return owner;
        }
    }
}
pub(super) fn pre(mut owner: Owner<'_>, ea: u32, k: u16) -> Owner<'_> {
    match ea {
        9291046 => {}
        9298443 => {
            if owner.read_le(ram::cinematic_function) == 0xb35f {
                put(&mut owner, Program, k, 0xe192);
                put(&mut owner, Delay, k, 1);
            }
        }
        9298656 => {
            if owner.read_native(0xf78 + core::mem::offset_of!(crate::types::EnemyData, health))
                == 0
            {
                put(&mut owner, Id, k, 0);
            }
        }
        9298809 => {
            if owner.read_le(ram::equipped_items) & 0x21 == 0 {
                let low = u32::from(owner.read_native(ram::samus_periodic_subdamage));
                let high = u32::from(owner.read_native(ram::samus_periodic_damage));
                let damage = (low | (high << 16)).wrapping_add(0x4000);
                owner.write_slice_native(ram::samus_periodic_subdamage, damage as u16);
                owner.write_slice_native(ram::samus_periodic_damage, (damage >> 16) as u16);
                if owner.read_le(ram::nmi_frame_counter_word) & 7 == 0
                    && owner.read_le(ram::samus_health) > 0x46
                {
                    owner = owner.foreign(|g| QueueSfx3_Max6(0x2d, g)).0;
                }
            }
            if owner.read_le(ram::samus_in_heat_palfx_index)
                != owner.read_le(ram::samus_in_heat_palettefx_prev_index)
            {
                owner.write_slice_le(
                    ram::samus_in_heat_palettefx_prev_index,
                    owner.read_le(ram::samus_in_heat_palfx_index),
                );
                let offset = owner
                    .read_le(ram::samus_in_heat_palfx_index)
                    .wrapping_mul(2);
                put(&mut owner, Delay, k, 1);
                let table = if owner.read_le(ram::equipped_items) & 0x20 != 0 {
                    0xe3e0
                } else if owner.read_le(ram::equipped_items) & 1 != 0 {
                    0xe400
                } else {
                    0xe420
                };
                let data;
                (owner, data) = record(owner, table);
                put(&mut owner, Program, k, data.word(usize::from(offset)));
            }
        }
        9301081 | 9301380 => {
            if owner.read_le(ram::samus_y_pos) < 0x380 {
                put(&mut owner, Delay, k, 1);
                put(
                    &mut owner,
                    Program,
                    k,
                    if ea == 9301081 { 0xeb43 } else { 0xec76 },
                );
            }
        }
        9301701 => {
            if miniboss_dead(&owner) {
                put(&mut owner, Id, k, 0);
            }
        }
        9303585 => {
            if owner.read_native(ram::flag_for_palette_fx_objects + usize::from(k)) != 0 {
                put(&mut owner, Id, k, 0);
            }
        }
        _ => {
            owner = owner.foreign(super::diagnostics::unreachable).0;
        }
    }
    owner
}
pub(super) fn command(mut owner: Owner<'_>, ea: u32, k: u16, j: u16) -> (Owner<'_>, u16, u16) {
    crate::verify_probe::hit("palette.command");
    let (k, j) = match ea {
        9291157 => {
            owner.write_slice_native(
                Program.address(owner.read_le(ram::palettefx_index)),
                j.wrapping_add(4),
            );
            crate::verify_probe::hit("palette.continuation.save");
            (0, 0)
        }
        9291161 => (k.wrapping_add(4), j.wrapping_add(2)),
        9291170 => (k.wrapping_add(6), j.wrapping_add(2)),
        9291179 => (k.wrapping_add(8), j.wrapping_add(2)),
        9291188 => (k.wrapping_add(16), j.wrapping_add(2)),
        9291197 => (k.wrapping_add(18), j.wrapping_add(2)),
        9291206 => (k.wrapping_add(30), j.wrapping_add(2)),
        9291215 => {
            put(&mut owner, Id, k, 0);
            (k, 0)
        }
        9291220 => {
            let data;
            (owner, data) = record(owner, j);
            put(&mut owner, PreAction, k, data.word(0));
            (k, j.wrapping_add(2))
        }
        9291229 => {
            put(&mut owner, PreAction, k, 0xc5e3);
            (k, j)
        }
        9291294 => {
            let data;
            (owner, data) = record(owner, j);
            (k, data.word(0))
        }
        9291321 => {
            let cursor = if decrement(&mut owner, Counter, k) == 1 {
                j.wrapping_add(2)
            } else {
                let data;
                (owner, data) = record(owner, j);
                data.word(0)
            };
            (k, cursor)
        }
        9291336 => {
            let data;
            (owner, data) = record(owner, j);
            owner.set_byte(0x1edd + usize::from(k), data.byte(0));
            (k, j.wrapping_add(1))
        }
        9291349 => {
            let data;
            (owner, data) = record(owner, j);
            put(&mut owner, Color, k, data.word(0));
            (k, j.wrapping_add(2))
        }
        9291358 => {
            let data;
            (owner, data) = record(owner, j);
            owner = owner
                .foreign(|g| QueueMusic_Delayed8(u16::from(data.byte(0)), g))
                .0;
            (k, j.wrapping_add(1))
        }
        9291370 | 9291379 | 9291388 => {
            let data;
            (owner, data) = record(owner, j);
            let sound = data.word(0);
            owner = owner
                .foreign(|g| match ea {
                    9291370 => QueueSfx1_Max6(sound, g),
                    9291379 => QueueSfx2_Max6(sound, g),
                    _ => QueueSfx3_Max6(sound, g),
                })
                .0;
            (k, j.wrapping_add(1))
        }
        9302470 => {
            let data;
            (owner, data) = record(owner, j);
            owner.write_slice_le(ram::samus_in_heat_palfx_index, u16::from(data.byte(0)));
            (k, j.wrapping_add(1))
        }
        _ => {
            owner = owner.foreign(super::diagnostics::unreachable).0;
            (0, 0)
        }
    };
    (owner, k, j)
}
pub(super) fn setup(mut owner: Owner<'_>, ea: u32, slot: u16) -> Owner<'_> {
    crate::verify_probe::hit("palette.setup");
    match ea {
        9291397 => {}
        9298436 => put(&mut owner, PreAction, slot, 0xe20b),
        9299008 => {
            let script = if owner.read_le(ram::equipped_items) & 0x20 != 0 {
                0xe8b6
            } else if owner.read_le(ram::equipped_items) & 1 != 0 {
                0xe68a
            } else {
                0xe45e
            };
            put(&mut owner, Program, slot, script);
        }
        9303856 => {
            if miniboss_dead(&owner) {
                put(&mut owner, Id, slot, 0);
            }
        }
        _ => {
            owner = owner.foreign(super::diagnostics::unreachable).0;
        }
    }
    owner
}
