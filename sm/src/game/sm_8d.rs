use crate::game::music::{QueueMusic_Delayed8};
pub use crate::game::constants::bank_8d::*;
pub use crate::game::prelude::*;
use crate::game::sm_80::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::ram_map::words as ram;
use palette_ram::{native_word, read, write, write_native, SlotWord::*};

// Transitional checked access: RAM stays authoritative, with no decoded state
// or borrow retained across callbacks. Byte indices may alias adjacent fields.
mod palette_ram {
    #![forbid(unsafe_code)]
    use crate::{game_state::Game, ram_map::word_arrays};

    pub(super) fn native_word(address: usize, game: &Game) -> u16 {
        u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]])
    }
    pub(super) fn read(address: usize, game: &Game) -> u16 {
        u16::from_le(native_word(address, game))
    }
    pub(super) fn write_native(address: usize, value: u16, game: &mut Game) {
        game.g_ram[address..address + 2].copy_from_slice(&value.to_ne_bytes());
    }
    pub(super) fn write(address: usize, value: u16, game: &mut Game) {
        write_native(address, value.to_le(), game);
    }

    #[derive(Clone, Copy)]
    #[repr(usize)]
    pub(super) enum SlotWord {
        Id = word_arrays::palettefx_ids,
        Color = word_arrays::palettefx_color_indexes,
        PreInstruction = word_arrays::palettefx_pre_instr,
        Instruction = word_arrays::palettefx_instr_list_ptrs,
        InstructionTimer = word_arrays::palettefx_instr_timers,
        Timer = word_arrays::palettefx_timers,
    }
    impl SlotWord {
        pub(super) fn address(self, index: u16) -> usize {
            self as usize + usize::from(index & !1)
        }
        pub(super) fn read(self, index: u16, game: &Game) -> u16 {
            read(self.address(index), game)
        }
        pub(super) fn write(self, index: u16, value: u16, game: &mut Game) {
            write(self.address(index), value, game);
        }
        pub(super) fn decrement(self, index: u16, game: &mut Game) -> u16 {
            let previous = self.read(index, game);
            self.write(index, previous.wrapping_sub(1), game);
            previous
        }
    }
}

const palette_script: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x8d>;

#[forbid(unsafe_code)]
pub fn EnablePaletteFx(game: &mut Game) {
    write(
        ram::flag_for_palette_fx_objects,
        read(ram::flag_for_palette_fx_objects, game) | 0x8000,
        game,
    );
}
#[forbid(unsafe_code)]
pub fn DisablePaletteFx(game: &mut Game) {
    write(
        ram::flag_for_palette_fx_objects,
        read(ram::flag_for_palette_fx_objects, game) & !0x8000,
        game,
    );
}
#[forbid(unsafe_code)]
pub fn ClearPaletteFXObjects(game: &mut Game) {
    for slot in (0..8).rev() {
        Id.write(2 * slot, 0, game);
    }
}
pub unsafe fn SpawnPalfxObject(definition: u16, game: &mut Game) {
    let Some(slot) = (0..8).rev().map(|i| 2 * i).find(|&i| Id.read(i, game) == 0) else {
        return;
    };
    Id.write(slot, definition, game);
    Color.write(slot, 0, game);
    PreInstruction.write(slot, 0xc526, game);
    Instruction.write(slot, palette_script(definition, game).word(2), game);
    InstructionTimer.write(slot, 1, game);
    Timer.write(slot, 0, game);
    let definition = palette_script(definition, game);
    CallPalfxFunc(0x8d0000 | u32::from(definition.word(0)), 0xffff, slot, game);
}
pub unsafe fn PaletteFxHandler(game: &mut Game) {
    if read(ram::flag_for_palette_fx_objects, game) & 0x8000 != 0 {
        let mut slot = 14i32;
        while slot >= 0 {
            write(ram::palettefx_index, slot as u16, game);
            if Id.read(slot as u16, game) != 0 {
                PalFx_ProcessOne(slot as u16, game);
                slot = i32::from(read(ram::palettefx_index, game));
            }
            slot -= 2;
        }
    }
}
pub unsafe fn PalFx_ProcessOne(k: u16, game: &mut Game) {
    CallPalFxPreInstr(0x8d0000 | u32::from(PreInstruction.read(k, game)), k, game);
    let mut slot = read(ram::palettefx_index, game);
    if InstructionTimer.decrement(slot, game) != 1 {
        return;
    }
    let mut cursor = Instruction.read(slot, game);
    let duration = loop {
        let script = palette_script(cursor, game);
        let instruction = script.word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        let next = CallPalInstr(
            0x8d0000 | u32::from(instruction),
            slot,
            cursor.wrapping_add(2),
            game,
        );
        slot = next.k;
        cursor = next.j;
        if cursor == 0 {
            return;
        }
    };
    InstructionTimer.write(slot, duration, game);
    let mut color = Color.read(slot, game);
    // Instructions now receive a color byte index in k, rather than a slot index.
    loop {
        let instruction = loop {
            let value = palette_script(cursor, game).word(2);
            if value & 0x8000 != 0 {
                break value;
            }
            write(
                crate::ram_map::word_arrays::palette_buffer + usize::from(color & !1),
                value,
                game,
            );
            color = color.wrapping_add(2);
            cursor = cursor.wrapping_add(2);
        };
        let next = CallPalInstr(0x8d0000 | u32::from(instruction), color, cursor, game);
        color = next.k;
        cursor = next.j;
        if cursor == 0 {
            return;
        }
    }
}

#[forbid(unsafe_code)]
fn miniboss_dead(game: &Game) -> bool {
    native_word(0xd828 + usize::from(read(ram::area_index, game)), game) & 2 != 0
}

pub unsafe fn CallPalFxPreInstr(ea: u32, k: u16, game: &mut Game) {
    match ea {
        9291046 => (),
        9298443 => {
            // Intro-page palette transition.
            if read(ram::cinematic_function, game) == 0xb35f {
                Instruction.write(k, addr_off_8DE192 as u16, game);
                InstructionTimer.write(k, 1, game);
            }
        }
        9298656 => {
            // Retire the effect when enemy zero dies.
            if native_word(0xf78 + core::mem::offset_of!(EnemyData, health), game) == 0 {
                Id.write(k, 0, game);
            }
        }
        9298809 => {
            // Heat damage and suit-dependent script selection.
            if read(ram::equipped_items, game) & 0x21 == 0 {
                let low = u32::from(native_word(ram::samus_periodic_subdamage, game));
                let high = u32::from(native_word(ram::samus_periodic_damage, game));
                let damage = (low | (high << 16)).wrapping_add(0x4000);
                write_native(ram::samus_periodic_subdamage, damage as u16, game);
                write_native(ram::samus_periodic_damage, (damage >> 16) as u16, game);
                if read(ram::nmi_frame_counter_word, game) & 7 == 0
                    && read(ram::samus_health, game) > 0x46
                {
                    QueueSfx3_Max6(0x2d, game);
                }
            }
            if read(ram::samus_in_heat_palfx_index, game)
                != read(ram::samus_in_heat_palettefx_prev_index, game)
            {
                write(
                    ram::samus_in_heat_palettefx_prev_index,
                    read(ram::samus_in_heat_palfx_index, game),
                    game,
                );
                let offset = read(ram::samus_in_heat_palfx_index, game).wrapping_mul(2);
                InstructionTimer.write(k, 1, game);
                let table = if read(ram::equipped_items, game) & 0x20 != 0 {
                    addr_off_8DE3E0
                } else if read(ram::equipped_items, game) & 1 != 0 {
                    addr_off_8DE400
                } else {
                    addr_off_8DE420
                };
                Instruction.write(
                    k,
                    palette_script(table as u16, game).word(usize::from(offset)),
                    game,
                );
            }
        }
        9301081 | 9301380 => {
            if read(ram::samus_y_pos, game) < 0x380 {
                InstructionTimer.write(k, 1, game);
                Instruction.write(
                    k,
                    if ea == 9301081 {
                        addr_word_8DEB43
                    } else {
                        addr_word_8DEC76
                    } as u16,
                    game,
                );
            }
        }
        9301701 => {
            if miniboss_dead(game) {
                Id.write(k, 0, game);
            }
        }
        9303585 => {
            // This flag lookup uses the unrounded byte index.
            if native_word(ram::flag_for_palette_fx_objects + usize::from(k), game) != 0 {
                Id.write(k, 0, game);
            }
        }
        _ => {
            Unreachable(game);
        }
    }
}

pub unsafe fn CallPalInstr(ea: u32, k: u16, j: u16, game: &mut Game) -> PairU16 {
    let (k, j) = match ea {
        9291157 => {
            // Save the continuation for the live active slot.
            write_native(
                Instruction.address(read(ram::palettefx_index, game)),
                j.wrapping_add(4),
                game,
            );
            (0, 0)
        }
        9291161 => (k.wrapping_add(4), j.wrapping_add(2)),
        9291170 => (k.wrapping_add(6), j.wrapping_add(2)),
        9291179 => (k.wrapping_add(8), j.wrapping_add(2)),
        9291188 => (k.wrapping_add(16), j.wrapping_add(2)),
        9291197 => (k.wrapping_add(18), j.wrapping_add(2)),
        9291206 => (k.wrapping_add(30), j.wrapping_add(2)),
        9291215 => {
            Id.write(k, 0, game);
            (k, 0)
        }
        9291220 => {
            PreInstruction.write(k, palette_script(j, game).word(0), game);
            (k, j.wrapping_add(2))
        }
        9291229 => {
            PreInstruction.write(k, addr_locret_8DC5E3 as u16, game);
            (k, j)
        }
        9291294 => (k, palette_script(j, game).word(0)),
        9291321 => {
            // A completed countdown skips its unread jump operand.
            let cursor = if Timer.decrement(k, game) == 1 {
                j.wrapping_add(2)
            } else {
                palette_script(j, game).word(0)
            };
            (k, cursor)
        }
        9291336 => {
            // This instruction addresses a byte at k, without rounding odd k down.
            game.g_ram[crate::ram_map::word_arrays::palettefx_timers + usize::from(k)] =
                palette_script(j, game).byte(0);
            (k, j.wrapping_add(1))
        }
        9291349 => {
            Color.write(k, palette_script(j, game).word(0), game);
            (k, j.wrapping_add(2))
        }
        9291358 => {
            QueueMusic_Delayed8(u16::from(palette_script(j, game).byte(0)), game);
            (k, j.wrapping_add(1))
        }
        9291370 | 9291379 | 9291388 => {
            // Sound operands are words although the cursor advances by one byte.
            let script = palette_script(j, game);
            let sound = script.word(0);
            match ea {
                9291370 => QueueSfx1_Max6(sound, game),
                9291379 => QueueSfx2_Max6(sound, game),
                _ => QueueSfx3_Max6(sound, game),
            }
            (k, j.wrapping_add(1))
        }
        9302470 => {
            // Select the next heat-palette variant.
            write(
                ram::samus_in_heat_palfx_index,
                u16::from(palette_script(j, game).byte(0)),
                game,
            );
            (k, j.wrapping_add(1))
        }
        _ => {
            Unreachable(game);
            (0, 0)
        }
    };
    PairU16 { k, j }
}

pub unsafe fn CallPalfxFunc(ea: u32, _k: u16, slot: u16, game: &mut Game) {
    match ea {
        9291397 => (),
        9298436 => PreInstruction.write(slot, 0xe20b, game),
        9299008 => {
            let script = if read(ram::equipped_items, game) & 0x20 != 0 {
                addr_off_8DE8B6
            } else if read(ram::equipped_items, game) & 1 != 0 {
                addr_off_8DE68A
            } else {
                addr_kPalfxInstrList_E45E
            };
            Instruction.write(slot, script as u16, game);
        }
        9303856 => {
            if miniboss_dead(game) {
                Id.write(slot, 0, game);
            }
        }
        _ => {
            Unreachable(game);
        }
    }
}
