use crate::game::sm_81::DrawSpritemapWithBaseTile;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::{RomPtr, Unreachable};

#[inline]
unsafe fn RomPtr_B4(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xb40000u32) | ((addr) as u32), game)
}
pub const fnSpriteObject_Instr_RepeatLast: u32 = 11844848;
pub const fnSpriteObject_Instr_Terminate: u32 = 11844871;
pub const fnSpriteObject_Instr_Goto: u32 = 11844882;
pub unsafe fn CreateSpriteAtPos(
    x_r18: u16,
    y_r20: u16,
    ilist_r22: u16,
    pal_r24: u16,
    game: &mut Game,
) -> u16 {
    let mut v0: i32 = 62i32;
    while ram_word!(game, sprite_instr_list_ptrs[(v0 >> 1i32)]) != 0 {
        v0 -= 2i32;
        if (v0 & 0x8000i32) != 0i32 {
            return 0xffffu16;
        }
    }
    let v1: i32 = v0 >> 1i32;
    set_ram_word!(game.sprite_palettes[v1] = 0u16);
    set_ram_word!(game.sprite_x_subpos[v1] = 0u16);
    set_ram_word!(game.sprite_y_subpos[v1] = 0u16);
    set_ram_word!(game.sprite_disable_flag[v1] = 0u16);
    set_ram_word!(game.sprite_x_pos[v1] = x_r18);
    set_ram_word!(game.sprite_y_pos[v1] = y_r20);
    set_ram_word!(game.sprite_palettes[v1] = pal_r24);
    let v2: u16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xb4bda8)) as *const u8 as *mut u16)
        .offset(ilist_r22 as isize);
    set_ram_word!(game.sprite_instr_list_ptrs[v1] = v2);
    set_ram_word!(game.sprite_instr_timer[v1] = *(RomPtr_B4(v2, game) as *mut u16));
    v0 as u16
}
pub unsafe fn CallSpriteObjectInstr(ea: u32, game: &mut Game) {
    match ea {
        11844848 => SpriteObject_Instr_RepeatLast(game),
        11844871 => SpriteObject_Instr_Terminate(game),
        11844882 => SpriteObject_Instr_Goto(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn HandleSpriteObjects(game: &mut Game) {
    let mut v1: u16 = 0;
    if ram_word!(game, debug_time_frozen_for_enemies) as i32
        | ram_word!(game, time_is_frozen_flag) as i32
        != 0
    {
        return;
    }
    set_ram_word!(game.sprite_object_index = 62u16);
    let mut current_block_9: u64;
    loop {
        let mut v0: i32 = 0;
        v0 = ram_word!(game, sprite_object_index) as i32 >> 1i32;
        if !(ram_word!(game, sprite_instr_list_ptrs[v0]) == 0
            || (ram_word!(game, sprite_disable_flag[v0]) as i32 & 1i32) != 0i32)
        {
            v1 = ram_word!(game, sprite_instr_timer[v0]);
            if v1 as i32 & 0x8000i32 != 0 {
                current_block_9 = 8895462745870634114;
            } else {
                set_ram_word!(game.sprite_instr_timer[v0] = (v1).wrapping_sub(1u16));
                if v1 == 1u16 {
                    let v3: u16 = (ram_word!(game, sprite_instr_list_ptrs[v0])).wrapping_add(4u16);
                    set_ram_word!(game.sprite_instr_list_ptrs[v0] = v3);
                    v1 = *(RomPtr_B4(v3, game) as *mut u16);
                    if v1 as i32 & 0x8000i32 != 0 {
                        current_block_9 = 8895462745870634114;
                    } else {
                        *((&raw mut game.g_ram as *mut u8).offset(0xeff8) as *mut u16).offset(
                            (ram_word!(game, sprite_object_index) as i32 >> 1i32) as isize,
                        ) = v1;
                        current_block_9 = 14916268686031723178;
                    }
                } else {
                    current_block_9 = 14916268686031723178;
                }
            }
            match current_block_9 {
                14916268686031723178 => {}
                _ => {
                    CallSpriteObjectInstr(((v1) as u32) | (0xb40000u32), game);
                }
            }
        }
        if (update_ram_word!(game.sprite_object_index, wrapping_sub(2u16), reread) as i32
            & 0x8000i32
            != 0)
        {
            break;
        }
    }
}
pub unsafe fn SpriteObject_Instr_RepeatLast(game: &mut Game) {
    let v2: i32 = ram_word!(game, sprite_object_index) as i32 >> 1i32;
    update_ram_word!(game.sprite_instr_list_ptrs[v2], wrapping_sub(4u16));
    set_ram_word!(game.sprite_instr_timer[v2] = 0x7fffu16);
}
pub unsafe fn SpriteObject_Instr_Terminate(game: &mut Game) {
    *((&raw mut game.g_ram as *mut u8).offset(0xef78) as *mut u16)
        .offset((ram_word!(game, sprite_object_index) as i32 >> 1i32) as isize) = 0u16;
}
pub unsafe fn SpriteObject_Instr_Goto(game: &mut Game) {
    let v2: u16 = *(RomPtr_B4(
        *((&raw mut game.g_ram as *mut u8).offset(0xef78) as *mut u16)
            .offset((ram_word!(game, sprite_object_index) as i32 >> 1i32) as isize),
        game,
    )
    .offset(2) as *mut u16);
    *((&raw mut game.g_ram as *mut u8).offset(0xef78) as *mut u16)
        .offset((ram_word!(game, sprite_object_index) as i32 >> 1i32) as isize) = v2;
    *((&raw mut game.g_ram as *mut u8).offset(0xeff8) as *mut u16)
        .offset((ram_word!(game, sprite_object_index) as i32 >> 1i32) as isize) =
        *(RomPtr_B4(v2, game) as *mut u16);
}
pub unsafe fn DrawSpriteObjects(game: &mut Game) {
    let mut i: i32 = 62i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        if ram_word!(game, sprite_instr_list_ptrs[v1]) != 0 {
            let x: u16 =
                (ram_word!(game, sprite_x_pos[v1])).wrapping_sub(ram_word!(game, layer1_x_pos));
            if ((x) as i16).wrapping_add(16i16) >= 0i16 && x as i32 - 272i32 & 0x8000i32 != 0 {
                let y: i16 = ((ram_word!(game, sprite_y_pos[v1])) as i16)
                    .wrapping_sub((ram_word!(game, layer1_y_pos)) as i16);
                if y >= 0i16 && y as i32 - 272i32 & 0x8000i32 != 0 {
                    let r3: u16 = (ram_word!(game, sprite_palettes[v1])) & (0xe00u16);
                    let r0: u16 = (ram_word!(game, sprite_palettes[v1])) & (0x1ffu16);
                    let v3: *const u8 =
                        RomPtr_B4(ram_word!(game, sprite_instr_list_ptrs[v1]), game);
                    DrawSpritemapWithBaseTile(
                        0xb4u8,
                        *(v3.offset(2) as *mut u16),
                        x,
                        y as u16,
                        r3,
                        r0,
                        game,
                    );
                }
            }
        }
        i -= 2i32;
    }
}
pub unsafe fn ClearSpriteObjects(game: &mut Game) {
    let mut v0: i32 = 1022i32;
    loop {
        set_ram_word!(game.sprite_instr_list_ptrs[(v0 >> 1i32)] = 0u16);
        v0 -= 2i32;
        if (v0 == 0) {
            break;
        }
    }
}
