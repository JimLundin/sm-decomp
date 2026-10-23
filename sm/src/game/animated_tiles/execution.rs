//! Safe animation execution with exact command identities and live callback boundaries.
#![forbid(unsafe_code)]
use super::{
    Owner,
    compatibility::SlotWord::{self, *},
};
use crate::game::constants::bank_86::{
    addr_kEproj_TourianStatueEyeGlow, addr_kEproj_TourianStatueSoul,
};
use crate::game::sm_80::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::{music::QueueMusic_Delayed8, palette_effects, progress_flags::*};
use crate::ram_map::{word_arrays as arrays, words as ram};
use crate::rom::RomRecord;
fn record(owner: Owner<'_>, address: u16) -> (Owner<'_>, RomRecord) {
    if address & 0x8000 == 0 {
        return owner.foreign(|g| super::native::low_record(address, g));
    }
    let image = owner.rom();
    let base = crate::rtl::lorom_offset(0x870000 | u32::from(address)) as usize;
    (owner, RomRecord { image, base })
}
fn decrement(owner: &mut Owner<'_>, field: SlotWord, index: u16) -> u16 {
    let previous = owner.read_slot(field, index);
    owner.write_slot(field, index, previous.wrapping_sub(1));
    previous
}
pub(super) fn spawn(mut owner: Owner<'_>, definition: u16) -> Owner<'_> {
    let Some(slot) = (0..6).rev().find(|&i| owner.state.slots[i].id == 0) else {
        return owner;
    };
    owner.state.slots[slot].id = definition;
    owner.state.slots[slot].counter = 0;
    let data;
    (owner, data) = record(owner, definition);
    owner.state.slots[slot].program = data.word(0);
    owner.state.slots[slot].source = 0;
    owner.state.slots[slot].size = data.word(2);
    owner.state.slots[slot].vram = data.word(4);
    owner.state.slots[slot].delay = 1;
    crate::verify_probe::hit("animated_tiles.spawn.initialized");
    owner
}
pub(super) fn advance(mut owner: Owner<'_>) -> Owner<'_> {
    if owner.state.flags & 0x8000 != 0 {
        let mut index = 10i32;
        while index >= 0 {
            owner.state.selected = index as u16;
            if owner.read_slot(Id, index as u16) != 0 {
                owner = process(owner);
                index = i32::from(owner.state.selected);
            }
            index -= 2;
        }
    }
    owner
}
fn goto(owner: Owner<'_>, cursor: u16) -> (Owner<'_>, u16) {
    let (owner, data) = record(owner, cursor);
    (owner, data.word(0))
}
fn goto_relative(mut owner: Owner<'_>, cursor: u16) -> (Owner<'_>, u16) {
    owner.state.last_instruction = cursor;
    let (owner, data) = record(owner, cursor);
    (owner, cursor.wrapping_add(data.byte(0) as i8 as u16))
}
pub(super) fn command(
    mut owner: Owner<'_>,
    address: u32,
    index: u16,
    cursor: u16,
) -> (Owner<'_>, u16) {
    crate::verify_probe::hit("animated_tiles.command.dispatched");
    let next = match address {
        8880306 => {
            owner.write_slot(Id, index, 0);
            0
        }
        8880311 => return goto(owner, cursor),
        8880316 => return goto_relative(owner, cursor),
        8880340 | 8880348 => {
            let relative = address == 8880348;
            if decrement(&mut owner, Counter, index) == 1 {
                cursor.wrapping_add(if relative { 1 } else { 2 })
            } else if relative {
                return goto_relative(owner, cursor);
            } else {
                return goto(owner, cursor);
            }
        }
        8880355 => {
            let data;
            (owner, data) = record(owner, cursor);
            owner.write_counter_byte(index, data.byte(0));
            cursor.wrapping_add(1)
        }
        8880368 | 8880380 | 8880392 | 8880404 => {
            let data;
            (owner, data) = record(owner, cursor);
            let sound = u16::from(data.byte(0));
            (owner, ()) = owner.foreign(|g| match address {
                8880368 => QueueMusic_Delayed8(sound, g),
                8880380 => QueueSfx1_Max6(sound, g),
                8880392 => QueueSfx2_Max6(sound, g),
                _ => QueueSfx3_Max6(sound, g),
            });
            cursor.wrapping_add(1)
        }
        8880416 | 8880447 | 8880899 => {
            let data;
            (owner, data) = record(owner, cursor);
            let (width, taken) = match address {
                8880416 => {
                    let mask = data.word(0) as u8 as u16;
                    let taken;
                    (owner, taken) = owner.foreign(|g| CheckBossBitForCurArea(mask, g) & 1 != 0);
                    (1, taken)
                }
                8880447 => {
                    let event = data.word(0);
                    let taken;
                    (owner, taken) = owner.foreign(|g| CheckEventHappened(event, g) & 1 != 0);
                    (2, taken)
                }
                _ => (
                    2,
                    data.byte(0) & owner.byte(0xd828 + usize::from(data.byte(1))) != 0,
                ),
            };
            let next = cursor.wrapping_add(width);
            if taken {
                return goto(owner, next);
            }
            next.wrapping_add(2)
        }
        8880435 | 8880464 => {
            let data;
            (owner, data) = record(owner, cursor);
            if address == 8880435 {
                let mask = u16::from(data.byte(0));
                (owner, ()) = owner.foreign(|g| SetBossBitForCurArea(mask, g));
                cursor.wrapping_add(1)
            } else {
                let event = data.word(0);
                (owner, ()) = owner.foreign(|g| SetEventHappened(event, g));
                cursor.wrapping_add(2)
            }
        }
        8880570 => {
            let alive;
            (owner, alive) = owner.foreign(|g| CheckBossBitForCurArea(1, g) & 1 == 0);
            if alive {
                owner.write_slot(Delay, index, 1);
                0
            } else {
                cursor
            }
        }
        8880928 | 8880943 => {
            let data;
            (owner, data) = record(owner, cursor);
            let projectile = if address == 8880928 {
                addr_kEproj_TourianStatueEyeGlow
            } else {
                addr_kEproj_TourianStatueSoul
            };
            let parameter = data.word(0);
            (owner, ()) =
                owner.foreign(|g| super::native::projectile(projectile as u16, parameter, g));
            cursor.wrapping_add(2)
        }
        8880958 => {
            if owner.read_le(ram::tourian_entrance_statue_animstate) & 0x8000 == 0 {
                cursor.wrapping_add(2)
            } else {
                return goto(owner, cursor);
            }
        }
        8880969 | 8880978 => {
            let previous = owner.read_le(ram::tourian_entrance_statue_animstate);
            let data;
            (owner, data) = record(owner, cursor);
            let mask = data.word(0);
            owner.write_word_le(
                if address == 8880969 {
                    previous | mask
                } else {
                    previous & !mask
                },
                ram::tourian_entrance_statue_animstate,
            );
            cursor.wrapping_add(2)
        }
        8880987 => {
            let data;
            (owner, data) = record(owner, cursor);
            let color = usize::from(data.word(0) & !1);
            for offset in [0, 2, 4] {
                owner.write_word_le(0, arrays::palette_buffer + color + offset);
            }
            cursor.wrapping_add(2)
        }
        8881010 => {
            let data;
            (owner, data) = record(owner, cursor);
            let definition = data.word(0);
            (owner, ()) = owner.foreign(|g| palette_effects::spawn(definition, g));
            cursor.wrapping_add(2)
        }
        8881023 => {
            let data;
            (owner, data) = record(owner, cursor);
            let mut color = data.word(0);
            for value in [
                0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
            ] {
                owner.write_word_le(value, arrays::target_palettes + usize::from(color & !1));
                color = color.wrapping_add(2);
            }
            cursor.wrapping_add(2)
        }
        _ => return owner.foreign(super::native::unreachable),
    };
    (owner, next)
}
pub(super) fn process(mut owner: Owner<'_>) -> Owner<'_> {
    let index = owner.state.selected;
    if decrement(&mut owner, Delay, index) != 1 {
        return owner;
    }
    let mut cursor = owner.read_slot(Program, index);
    let duration = loop {
        let data;
        (owner, data) = record(owner, cursor);
        let instruction = data.word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        owner.state.last_instruction = instruction;
        (owner, cursor) = command(
            owner,
            0x870000 | u32::from(instruction),
            index,
            cursor.wrapping_add(2),
        );
        if cursor == 0 {
            return owner;
        }
    };
    owner.write_slot(Delay, index, duration);
    let data;
    (owner, data) = record(owner, cursor);
    owner.write_slot(Source, index, data.word(2));
    owner.write_slot(Program, index, cursor.wrapping_add(4));
    crate::verify_probe::hit("animated_tiles.frame.requested");
    owner
}
