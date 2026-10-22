//! Shared sprite-effect pool and animation; RAM remains its compatibility state.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::sm_81::DrawSpritemapWithBaseTile;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays};
use crate::rtl::rom_bank_record;

#[forbid(unsafe_code)]
fn native_slot(game: &Game, base: usize) -> usize {
    base + usize::from(word!(game.sprite_object_index) >> 1) * 2
}
#[forbid(unsafe_code)]
fn write_native(game: &mut Game, base: usize, value: u16) {
    let address = native_slot(game, base);
    game.g_ram[address..address + 2].copy_from_slice(&value.to_ne_bytes());
}

pub(crate) unsafe fn create_sprite(
    x: u16,
    y: u16,
    selector: u16,
    palette: u16,
    game: &mut Game,
) -> u16 {
    let Some(slot) = (0..32)
        .rev()
        .find(|&slot| word!(game.sprite_instr_list_ptrs[slot]) == 0)
    else {
        return 0xffff;
    };
    word!(game.sprite_palettes[slot] = 0);
    word!(game.sprite_x_subpos[slot] = 0);
    word!(game.sprite_y_subpos[slot] = 0);
    word!(game.sprite_disable_flag[slot] = 0);
    word!(game.sprite_x_pos[slot] = x);
    word!(game.sprite_y_pos[slot] = y);
    word!(game.sprite_palettes[slot] = palette);
    let instruction = crate::rtl::rom_table_word!(game, 0xb4bda8, selector as isize);
    word!(game.sprite_instr_list_ptrs[slot] = instruction);
    let timer = unsafe { rom_bank_record::<0xb4>(instruction, game) }.word(0);
    word!(game.sprite_instr_timer[slot] = timer);
    slot as u16 * 2
}

unsafe fn instruction(opcode: u16, game: &mut Game) {
    match u32::from(opcode) | 0xb40000 {
        11844848 => {
            let slot = word!(game.sprite_object_index) >> 1;
            word!(game.sprite_instr_list_ptrs[slot] += 0xfffc);
            word!(game.sprite_instr_timer[slot] = 0x7fff);
        }
        11844871 => write_native(game, arrays::sprite_instr_list_ptrs, 0),
        11844882 => {
            let address = native_slot(game, arrays::sprite_instr_list_ptrs);
            let current = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]);
            let target = unsafe { rom_bank_record::<0xb4>(current, game) }.word(2);
            // Diagnostics can change the live slot and ROM allocation between writes.
            write_native(game, arrays::sprite_instr_list_ptrs, target);
            let timer = unsafe { rom_bank_record::<0xb4>(target, game) }.word(0);
            write_native(game, arrays::sprite_instr_timer, timer);
        }
        _ => unsafe {
            crate::rtl::Unreachable(game);
        },
    }
}

pub(crate) unsafe fn update_sprites(game: &mut Game) {
    if word!(game.debug_time_frozen_for_enemies) | word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    word!(game.sprite_object_index = 62);
    loop {
        let slot = word!(game.sprite_object_index) >> 1;
        if word!(game.sprite_instr_list_ptrs[slot]) != 0
            && word!(game.sprite_disable_flag[slot]) & 1 == 0
        {
            let mut timer = word!(game.sprite_instr_timer[slot]);
            if timer & 0x8000 == 0 {
                word!(game.sprite_instr_timer[slot] = timer.wrapping_sub(1));
                if timer == 1 {
                    let next = word!(game.sprite_instr_list_ptrs[slot]).wrapping_add(4);
                    word!(game.sprite_instr_list_ptrs[slot] = next);
                    timer = unsafe { rom_bank_record::<0xb4>(next, game) }.word(0);
                    if timer & 0x8000 == 0 {
                        write_native(game, arrays::sprite_instr_timer, timer);
                    }
                }
            }
            if timer & 0x8000 != 0 {
                unsafe {
                    instruction(timer, game);
                }
            }
        }
        word!(game.sprite_object_index += 0xfffe);
        if word!(game.sprite_object_index) & 0x8000 != 0 {
            break;
        }
    }
}

pub(crate) unsafe fn draw_sprites(game: &mut Game) {
    for slot in (0..32).rev() {
        if word!(game.sprite_instr_list_ptrs[slot]) == 0 {
            continue;
        }
        let x = word!(game.sprite_x_pos[slot]).wrapping_sub(word!(game.layer1_x_pos));
        // Keep the original signed/wrapping culling predicates, including far offsets.
        if (x as i16).wrapping_add(16) < 0 || (i32::from(x) - 272) & 0x8000 == 0 {
            continue;
        }
        let y =
            (word!(game.sprite_y_pos[slot]) as i16).wrapping_sub(word!(game.layer1_y_pos) as i16);
        if y < 0 || (i32::from(y) - 272) & 0x8000 == 0 {
            continue;
        }
        let palette = word!(game.sprite_palettes[slot]) & 0xe00;
        let tile = word!(game.sprite_palettes[slot]) & 0x1ff;
        let record =
            unsafe { rom_bank_record::<0xb4>(word!(game.sprite_instr_list_ptrs[slot]), game) };
        unsafe {
            DrawSpritemapWithBaseTile(0xb4, record.word(2), x, y as u16, palette, tile, game);
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn clear_sprites(game: &mut Game) {
    // Clears the wider 1,022-byte block, deliberately preserving its first word.
    word_transfers!(game, descending, 511; le(sprite_instr_list_ptrs, 511) <- value(0));
}
