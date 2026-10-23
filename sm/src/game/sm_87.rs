pub use crate::game::constants::bank_86::*;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::game::progress_flags::{
    CheckBossBitForCurArea, CheckEventHappened, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_80::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::ram_map::{checked_word, word_arrays};

const animtiles_record: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x87>;

#[forbid(unsafe_code)]
pub fn EnableAnimtiles(game: &mut Game) {
    checked_word!(game.animtiles_enable_flag = checked_word!(game.animtiles_enable_flag) | 0x8000);
}
#[forbid(unsafe_code)]
pub fn DisableAnimtiles(game: &mut Game) {
    checked_word!(game.animtiles_enable_flag = checked_word!(game.animtiles_enable_flag) & !0x8000);
}
#[forbid(unsafe_code)]
pub fn ClearAnimtiles(game: &mut Game) {
    for slot in (0..6).rev() {
        checked_word!(game.animtiles_ids[slot] = 0);
    }
}

// RAM remains authoritative. Byte places retain broad/odd aliases without a
// whole-RAM borrow or a second independently mutable state owner.
#[forbid(unsafe_code)]
fn decrement(address: usize, game: &mut Game) -> u16 {
    let previous = checked_word!(game, address;);
    checked_word!(game, address; = previous.wrapping_sub(1));
    previous
}

pub unsafe fn SpawnAnimtiles(definition: u16, game: &mut Game) {
    unsafe {
        let Some(slot) = (0..6)
            .rev()
            .find(|&i| checked_word!(game.animtiles_ids[i]) == 0)
        else {
            return;
        };
        checked_word!(game.animtiles_ids[slot] = definition);
        checked_word!(game.animtiles_timers[slot] = 0);
        let record = animtiles_record(definition, game);
        checked_word!(game.animtiles_instr_list_ptrs[slot] = record.word(0));
        checked_word!(game.animtiles_src_ptr[slot] = 0);
        checked_word!(game.animtiles_sizes[slot] = record.word(2));
        checked_word!(game.animtiles_vram_ptr[slot] = record.word(4));
        checked_word!(game.animtiles_instr_timers[slot] = 1);
    }
}
pub unsafe fn AnimtilesHandler(game: &mut Game) {
    unsafe {
        if checked_word!(game.animtiles_enable_flag) & 0x8000 != 0 {
            let mut index = 10i32;
            while index >= 0 {
                checked_word!(game.animtiles_object_index = index as u16);
                if checked_word!(game.animtiles_ids[index >> 1]) != 0 {
                    ProcessAnimtilesObject(game);
                    index = i32::from(checked_word!(game.animtiles_object_index));
                }
                index -= 2;
            }
        }
    }
}

unsafe fn goto(cursor: u16, game: &mut Game) -> u16 {
    unsafe { animtiles_record(cursor, game).word(0) }
}
unsafe fn goto_relative(cursor: u16, game: &mut Game) -> u16 {
    unsafe {
        checked_word!(game.animtiles_instruction = cursor);
        cursor.wrapping_add(animtiles_record(cursor, game).byte(0) as i8 as u16)
    }
}

pub unsafe fn CallAnimtilesInstr(address: u32, index: u16, cursor: u16, game: &mut Game) -> u16 {
    unsafe {
        match address {
            8880306 => {
                // Delete.
                checked_word!(game.animtiles_ids[index >> 1] = 0);
                0
            }
            8880311 => goto(cursor, game),
            8880316 => goto_relative(cursor, game),
            8880340 | 8880348 => {
                // Count down before choosing a word or relative jump.
                let relative = address == 8880348;
                let previous = decrement(
                    word_arrays::animtiles_timers + usize::from(index & !1),
                    game,
                );
                if previous == 1 {
                    cursor.wrapping_add(if relative { 1 } else { 2 })
                } else if relative {
                    goto_relative(cursor, game)
                } else {
                    goto(cursor, game)
                }
            }
            8880355 => {
                // The timer byte deliberately uses the unrounded index.
                game.g_ram[word_arrays::animtiles_timers + usize::from(index)] =
                    animtiles_record(cursor, game).byte(0);
                cursor.wrapping_add(1)
            }
            8880368 | 8880380 | 8880392 | 8880404 => {
                let record = animtiles_record(cursor, game);
                let sound = u16::from(record.byte(0));
                match address {
                    8880368 => QueueMusic_Delayed8(sound, game),
                    8880380 => QueueSfx1_Max6(sound, game),
                    8880392 => QueueSfx2_Max6(sound, game),
                    _ => QueueSfx3_Max6(sound, game),
                }
                cursor.wrapping_add(1)
            }
            8880416 | 8880447 | 8880899 => {
                // Boss/event branches retain operand widths.
                let record = animtiles_record(cursor, game);
                let (width, taken) = match address {
                    8880416 => (
                        1,
                        CheckBossBitForCurArea(record.word(0) as u8 as u16, game) & 1 != 0,
                    ),
                    8880447 => (2, CheckEventHappened(record.word(0), game) & 1 != 0),
                    _ => (
                        2,
                        record.byte(0) & game.g_ram[0xd828 + usize::from(record.byte(1))] != 0,
                    ),
                };
                let next = cursor.wrapping_add(width);
                if taken {
                    goto(next, game)
                } else {
                    next.wrapping_add(2)
                }
            }
            8880435 => {
                let record = animtiles_record(cursor, game);
                SetBossBitForCurArea(u16::from(record.byte(0)), game);
                cursor.wrapping_add(1)
            }
            8880464 => {
                let record = animtiles_record(cursor, game);
                SetEventHappened(record.word(0), game);
                cursor.wrapping_add(2)
            }
            8880570 => {
                // Wait until the area's boss dies.
                if CheckBossBitForCurArea(1, game) & 1 == 0 {
                    checked_word!(game.animtiles_instr_timers[index >> 1] = 1);
                    0
                } else {
                    cursor
                }
            }
            8880928 | 8880943 => {
                let record = animtiles_record(cursor, game);
                let projectile = if address == 8880928 {
                    addr_kEproj_TourianStatueEyeGlow
                } else {
                    addr_kEproj_TourianStatueSoul
                };
                SpawnEprojWithRoomGfx(projectile as u16, record.word(0), game);
                cursor.wrapping_add(2)
            }
            8880958 => {
                if checked_word!(game.tourian_entrance_statue_animstate) & 0x8000 == 0 {
                    cursor.wrapping_add(2)
                } else {
                    goto(cursor, game)
                }
            }
            8880969 | 8880978 => {
                let previous = checked_word!(game.tourian_entrance_statue_animstate);
                let record = animtiles_record(cursor, game);
                let mask = record.word(0);
                checked_word!(
                    game.tourian_entrance_statue_animstate = if address == 8880969 {
                        previous | mask
                    } else {
                        previous & !mask
                    }
                );
                cursor.wrapping_add(2)
            }
            8880987 => {
                // Clear three linear color words, without cursor wrapping.
                let color = usize::from(animtiles_record(cursor, game).word(0) & !1);
                for offset in [0, 2, 4] {
                    checked_word!(game, word_arrays::palette_buffer + color + offset; = 0);
                }
                cursor.wrapping_add(2)
            }
            8881010 => {
                let record = animtiles_record(cursor, game);
                SpawnPalfxObject(record.word(0), game);
                cursor.wrapping_add(2)
            }
            8881023 => {
                // Target-palette indices, unlike the clear above, wrap.
                let mut color = animtiles_record(cursor, game).word(0);
                for value in [
                    0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
                ] {
                    checked_word!(game.target_palettes[color >> 1] = value);
                    color = color.wrapping_add(2);
                }
                cursor.wrapping_add(2)
            }
            _ => Unreachable(game) as u16,
        }
    }
}

pub unsafe fn ProcessAnimtilesObject(game: &mut Game) {
    unsafe {
        let index = checked_word!(game.animtiles_object_index);
        let slot = index >> 1;
        if decrement(
            word_arrays::animtiles_instr_timers + usize::from(index & !1),
            game,
        ) != 1
        {
            return;
        }
        let mut cursor = checked_word!(game.animtiles_instr_list_ptrs[slot]);
        let duration = loop {
            let record = animtiles_record(cursor, game);
            let instruction = record.word(0);
            if instruction & 0x8000 == 0 {
                break instruction;
            }
            checked_word!(game.animtiles_instruction = instruction);
            cursor = CallAnimtilesInstr(
                0x870000 | u32::from(instruction),
                index,
                cursor.wrapping_add(2),
                game,
            );
            if cursor == 0 {
                return;
            }
        };
        // Callbacks may change the live handler index; this update retains its slot.
        checked_word!(game.animtiles_instr_timers[slot] = duration);
        checked_word!(game.animtiles_src_ptr[slot] = animtiles_record(cursor, game).word(2));
        checked_word!(game.animtiles_instr_list_ptrs[slot] = cursor.wrapping_add(4));
    }
}
