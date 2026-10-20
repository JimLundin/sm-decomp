pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::sm_80::{
    AddGrappleToHudTilemap, AddMissilesToHudTilemap, AddPowerBombsToHudTilemap,
    AddSuperMissilesToHudTilemap, AddXrayToHudTilemap, CheckBossBitForCurArea, CheckEventHappened,
    PrepareBitAccess, QueueMusic_Delayed8, QueueSfx1_Max1, QueueSfx1_Max15, QueueSfx1_Max3,
    QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max1, QueueSfx2_Max15, QueueSfx2_Max3,
    QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max1, QueueSfx3_Max15, QueueSfx3_Max3,
    QueueSfx3_Max6, QueueSfx3_Max9, SetEventHappened,
};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game::sm_85::{DisplayMessageBox, DisplayMessageBox_Async, DisplayMessageBox_Poll};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_90::{CallSomeSamusCode, UpdateBeamTilesAndPalette};
use crate::game::sm_91::{
    GravitySuitPickup, LoadBlockToXrayTilemap, MakeSamusFaceForward, VariaSuitPickup,
};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::abs16;
use crate::rtl::rom_ptr;
use crate::types::TileDrawOrder;
use ::core::ffi::{c_char, c_long, c_void};
use ::core::ptr::{null, null_mut};

use crate::c_library::{__assert_fail, printf};
use crate::rtl::{Load24, Mult8x8, RomPtr, SnesDivide, SnesModulus, Unreachable};
pub use crate::types::{CoroutineRet, Func_Y_V, LongPtr, VoidP, VramWriteEntry};
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct PlmHeader_Size4 {
        pub func_ptr, instr_list_ptr: VoidP;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct XraySpecialCasing {
        pub x_block, y_block: u8;
        pub level_data_block: u16;
    }
}
pub use crate::types::Buttons;
pub use crate::types::Consts_84;
pub use crate::types::Consts_86;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{Consts_60, RoomDefRoomstate, TileTable, TileTables};
pub use crate::types::{EnemyData, RoomPlmEntry, SamusPose};
pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    kCoroutineNone = 0,
}
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    crate::native_assertion! {
        '_c2rust_label: addr < 0x2000;
        b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
    };
    ram_ptr!(game; addr as i32 as isize)
}
const RomPtr_84: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x84>;
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
#[inline]
unsafe fn get_RoomDefRoomstate(a: u16, game: &mut Game) -> *mut RoomDefRoomstate {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomDefRoomstate
}
#[inline]
unsafe fn get_RoomPlmEntry(a: u16, game: &mut Game) -> *mut RoomPlmEntry {
    RomPtr((0x8f0000u32) | ((a) as u32), game) as *mut RoomPlmEntry
}
#[inline]
unsafe fn get_PlmHeader_Size4(a: u16, game: &mut Game) -> *mut PlmHeader_Size4 {
    RomPtr((0x840000u32) | ((a) as u32), game) as *mut PlmHeader_Size4
}
pub unsafe fn SetGoldenTorizoPalette(mut a: u16, game: &mut Game) {
    let mut v1: i16 = ((byte!(a; 1)) as i16) & (0x78i16);
    if (v1 as u16 & 0x40) != 0 {
        v1 = 56;
    }
    let mut v2: u16 = (4 * v1 as i32 | 0x1e) as u16;
    for i in (0i32..=30).rev().step_by(2) {
        let v4: i32 = v2 as i32 >> 1;
        let v5: i32 = i >> 1;
        word! { game;
            palette_buffer[(v5 + 160)] = crate::rtl::rom_table_word!(game, 0x848132, v4 as isize);
            palette_buffer[(v5 + 144)] = crate::rtl::rom_table_word!(game, 0x848032, v4 as isize);
        }
        v2 = (v2).wrapping_sub(2);
    }
}
pub unsafe fn LoadRoomPlmGfx(game: &mut Game) {
    word!(game.plm_item_gfx_index = 0);
    for v0 in (0i32..8).step_by(2) {
        if word!(game.plm_item_gfx_ptrs[(v0 >> 1)]) != 0 {
            PlmInstr_LoadItemPlmGfx(
                RomPtr_84(word!(game.plm_item_gfx_ptrs[(v0 >> 1)]), game),
                v0 as u16,
                game,
            );
        }
    }
}
pub unsafe fn ClearSoundsWhenGoingThroughDoor(game: &mut Game) {
    CallSomeSamusCode(0x1d, game);
}
pub unsafe fn PlaySpinJumpSoundIfSpinJumping(game: &mut Game) {
    CallSomeSamusCode(0x1c, game);
}
pub unsafe fn CalculatePlmBlockCoords(k: u16, game: &mut Game) {
    let block_index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
    let block_index = u16::from_le_bytes([
        game.g_ram[block_index_addr],
        game.g_ram[block_index_addr + 1],
    ]) >> 1;
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let y = SnesDivide(block_index, room_width);
    game.g_ram[0x1c2b..0x1c2d].copy_from_slice(&y.to_le_bytes()); // plm_y_block
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let x = SnesModulus(block_index, room_width);
    game.g_ram[0x1c29..0x1c2b].copy_from_slice(&x.to_le_bytes()); // plm_x_block
}
pub fn WriteLevelDataBlockTypeAndBts(k: u16, a: u16, game: &mut Game) {
    // The level-data byte uses the exact offset, while BTS rounds down to a word.
    let block_high = 0x10003 + usize::from(k);
    game.g_ram[block_high] = (a >> 8) as u8 | (game.g_ram[block_high] & 0x0f);
    game.g_ram[0x16402 + usize::from(k >> 1)] = a as u8;
}
pub unsafe fn WriteRowOfLevelDataBlockAndBTS(
    k: u16,
    arg0: u16,
    arg1: u16,
    arg2: u16,
    game: &mut Game,
) {
    let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
    let v8: u16 = (v2 >> 1);
    let mut i: i32 = 0;
    while i < arg2 as i32 {
        *(ram_ptr!(game, u16; 0x10002)).offset((v2 as i32 + i * 2 >> 1) as isize) = arg0;
        *ram_ptr!(game; 0x16402, (v8 as i32 + i) as isize) = arg1 as u8;
        i += 1;
    }
}
pub unsafe fn LoadXrayBlocks(game: &mut Game) {
    let mut i: i32 = 78;
    while i >= 0 {
        let v1: i32 = i >> 1;
        if word!(game.plm_header_ptr[v1]) as i32 >= 0x84df89 & 0xffff {
            let k: u16 = i as u16;
            let v2: u16 = *ram_ptr!(game; 0xd870, PrepareBitAccess(word!(game.plm_room_arguments[v1]), game) as isize)
                as u16;
            i = k as i32;
            if (word!(game.bitmask) & v2) == 0 {
                CalculatePlmBlockCoords(k, game);
                let v3 = RomPtr_84(
                    crate::rtl::rom_table_word!(
                        game,
                        0x84839d,
                        (*(ram_ptr!(game, u16; 0xdf0c)).offset((k >> 1) as isize) >> 1) as isize,
                    ),
                    game,
                );
                LoadBlockToXrayTilemap(
                    (*(v3.offset(2) as *mut u16)) & (0xfffu16),
                    word!(game.plm_x_block),
                    word!(game.plm_y_block),
                    game,
                );
                i = k as i32;
            }
        }
        i -= 2;
    }
    let RS = get_RoomDefRoomstate(word!(game.roomdefroomstate_ptr), game);
    if (*RS).xray_special_casing_ptr != 0 {
        let mut p = RomPtr_8F((*RS).xray_special_casing_ptr, game) as *const XraySpecialCasing;
        while (*p).x_block != 0 || (*p).y_block != 0 {
            LoadBlockToXrayTilemap(
                (*p).level_data_block,
                (*p).x_block as u16,
                (*p).y_block as u16,
                game,
            );
            p = p.offset(1);
        }
    }
}
pub fn EnablePLMs(game: &mut Game) {
    // plm_flag: enable bit 15 while preserving the other flag bits.
    let flags = u16::from_le_bytes([game.g_ram[0x1c23], game.g_ram[0x1c24]]);
    game.g_ram[0x1c23..0x1c25].copy_from_slice(&(flags | 0x8000).to_le_bytes());
}
pub unsafe fn DisablePLMs(game: &mut Game) {
    word!(game.plm_flag, |fresh22| (fresh22 & !(0x8000)));
}
pub fn ClearPLMs(game: &mut Game) {
    for slot in (0..40).rev() {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        game.g_ram[header_addr..header_addr + 2].copy_from_slice(&0u16.to_le_bytes());
    }
    game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&0u16.to_le_bytes()); // plm_item_gfx_index
}
pub unsafe fn SpawnHardcodedPlm(p: SpawnHardcodedPlmArgs, game: &mut Game) {
    spawn_hardcoded_plm(p, core::ptr::null_mut(), game);
}
// Nested setup spawns retain the calling collision's displacement.
unsafe fn spawn_hardcoded_plm(
    mut p: SpawnHardcodedPlmArgs,
    displacement: *mut i32,
    game: &mut Game,
) {
    let pp = &raw mut p;
    let mut v1: u16 = 78;
    while word!(game.plm_header_ptr[(v1 >> 1)]) != 0 {
        v1 = (v1).wrapping_sub(2);
        if (v1 & 0x8000) != 0 {
            return;
        }
    }
    let prod: u16 = Mult8x8((*pp).block_y, word!(game.room_width_in_blocks) as u8);
    let index = usize::from(v1 >> 1) * 2;
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_index = (2 * (u32::from(prod) + u32::from((*pp).block_x))) as u16;
    write_word(game, 0x1c87 + index, block_index); // plm_block_indices
    let v3 = (*pp).header;
    write_word(game, 0x1c37 + index, v3); // plm_header_ptr
    write_word(game, 0x1dc7 + index, 0); // plm_room_arguments
    write_word(game, 0xdf0c + index, 0); // plm_variables
    write_word(game, 0x1cd7 + index, 0x8469); // plm_pre_instrs
    let PH = get_PlmHeader_Size4(v3, game);
    let instruction = (*PH).instr_list_ptr;
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0); // plm_timers
    write_word(game, 0x1c27, v1); // plm_id
    CallPlmHeaderFunc(
        (((*PH).func_ptr) as u32) | (0x840000u32),
        v1,
        displacement,
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}
pub unsafe fn SpawnRoomPLM(k: u16, game: &mut Game) {
    let v1 = match (0..40).rev().find(|&slot| {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        u16::from_le_bytes([game.g_ram[header_addr], game.g_ram[header_addr + 1]]) == 0
    }) {
        Some(slot) => (slot * 2) as u16,
        None => return,
    };
    let entry = if k == 0x12 {
        game.g_ram.as_ptr().add(0x12)
    } else {
        get_RoomPlmEntry(k, game).cast::<u8>()
    };
    let read_entry_word =
        |offset: usize| u16::from_le_bytes([*entry.add(offset), *entry.add(offset + 1)]);
    let y_block = u16::from(*entry.add(3));
    let room_width = u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]) as u8;
    let row_start = y_block * u16::from(room_width);
    let x_block = u16::from(*entry.add(2));
    let index = usize::from(v1 >> 1) * 2;
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let block_index = row_start.wrapping_add(x_block).wrapping_mul(2);
    write_word(game, 0x1c87 + index, block_index); // plm_block_indices
    let argument = read_entry_word(4);
    write_word(game, 0x1dc7 + index, argument); // plm_room_arguments
    let pp = read_entry_word(0);
    let header = read_entry_word(0);
    write_word(game, 0x1c37 + index, header); // plm_header_ptr
    write_word(game, 0xdf0c + index, 0); // plm_variables
    write_word(game, 0x1cd7 + index, 0x84e6); // plm_pre_instrs
    let header_bytes = get_PlmHeader_Size4(pp, game).cast::<u8>();
    let read_header_word = |offset: usize| {
        u16::from_le_bytes([*header_bytes.add(offset), *header_bytes.add(offset + 1)])
    };
    let instruction = read_header_word(2);
    write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
    write_word(game, 0xde1c + index, 1); // plm_instruction_timer
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0); // plm_timers
    write_word(game, 0x1c27, v1); // plm_id
    CallPlmHeaderFunc(
        0x840000 | u32::from(read_header_word(0)),
        v1,
        core::ptr::null_mut(),
        addr_kDefaultPlmDrawInstruction as u16,
        game,
    );
}
/// Prepare a PLM slot at the current collision block and invoke its setup.
///
/// # Safety
/// The selected setup callback requires the legacy game runtime to be initialized.
pub unsafe fn SpawnPLM(header: u16, game: &mut Game) -> u8 {
    SpawnPLMWithCollision(header, core::ptr::null_mut(), game)
}
/// Prepare a PLM while retaining the caller's active collision displacement.
///
/// # Safety
/// Setup requires initialized runtime state. A non-null `displacement` must
/// remain live and writable through setup and any nested PLM spawns.
pub unsafe fn SpawnPLMWithCollision(header: u16, displacement: *mut i32, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    let write_word = |game: &mut Game, offset: usize, value: u16| {
        game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    // Allocate from the last slot down, retaining the original full-table result.
    let Some(slot) = (0..40)
        .rev()
        .find(|&slot| read_word(game, 0x1c37 + slot * 2) == 0)
    else {
        return 0;
    };
    let index = slot * 2;
    write_word(game, 0x1c87 + index, game.cur_block_index().wrapping_mul(2));
    write_word(game, 0x1c37 + index, header);
    write_word(game, 0x1cd7 + index, 0x853d);
    let instruction = game.read_rom_word(0x840000 | u32::from(header), 2);
    write_word(game, 0x1d27 + index, instruction);
    write_word(game, 0xde1c + index, 1);
    write_word(game, 0xde6c + index, addr_kDefaultPlmDrawInstruction as u16);
    write_word(game, 0x1d77 + index, 0);
    write_word(game, 0x1dc7 + index, 0);
    write_word(game, 0xdf0c + index, 0);
    write_word(game, 0x1c27, index as u16);
    // Read setup after preparing the slot, at the original callback boundary.
    let setup = game.read_rom_word(0x840000 | u32::from(header), 0);
    CallPlmHeaderFunc(
        0x840000 | u32::from(setup),
        index as u16,
        displacement,
        0,
        game,
    )
}
pub unsafe fn PlmHandler_Async(game: &mut Game) -> CoroutineRet {
    crate::verify_probe::hit("coroutine.plm");
    let resume_state: *mut u8 = ram_ptr!(game; 0x613);
    let resume = *resume_state;

    if resume <= 1 {
        let mut base = null::<u8>();
        let mut v5 = null::<u8>();
        let mut message_pending = resume == 1;
        if resume == 0 {
            if (word!(game.plm_flag) & 0x8000) == 0 {
                return kCoroutineNone as CoroutineRet;
            }
            word! {game; plm_draw_tilemap_index = 0; plm_id = 78; }
        }
        loop {
            if !message_pending {
                if (word!(game.plm_id) & 0x8000) != 0 {
                    word!(game.plm_id = 0);
                    break;
                }
                if *(ram_ptr!(game, u16; 0x1c37)).offset((word!(game.plm_id) >> 1) as isize) == 0 {
                    word!(game.plm_id, wrapping_sub(2));
                    continue;
                }
                CallPlmPreInstr(
                    ((*(ram_ptr!(game, u16; 0x1cd7)).offset((word!(game.plm_id) >> 1) as isize))
                        as u32)
                        | (0x840000u32),
                    word!(game.plm_id),
                    game,
                );

                if word!(
                    game.plm_instruction_timer[(word!(game.plm_id) >> 1)],
                    wrapping_sub(1),
                    reread
                ) != 0
                {
                    word!(game.plm_id, wrapping_sub(2));
                    continue;
                }
                base = RomBankBase(0x84, game);
                v5 = null::<u8>();
                v5 = base.offset(
                    *(ram_ptr!(game, u16; 0x1d27)).offset((word!(game.plm_id) >> 1) as isize) as i32
                        as isize,
                );
            }
            loop {
                if message_pending {
                    crate::verify_probe::hit("coroutine.plm.message");
                    let yielded: u8 =
                        DisplayMessageBox_Async(word!(game.queued_message_box_index), game);
                    if yielded != 0 {
                        *resume_state = 1;
                        return yielded;
                    }
                    word!(game.queued_message_box_index = 0);
                    base = RomBankBase(0x84, game);
                    v5 = base.offset(
                        *(ram_ptr!(game, u16; 0x1d27)).offset((word!(game.plm_id) >> 1) as isize)
                            as i32 as isize,
                    );

                    message_pending = false;
                }
                if (*(v5 as *mut u16) & 0x8000) == 0 {
                    let mut v7: i32 = word!(game.plm_id) as i32 >> 1;
                    word! { game;
                        plm_instruction_timer[v7] = *(v5 as *mut u16);
                        plm_instruction_draw_ptr[v7] = *(v5.offset(2) as *mut u16);
                        plm_instr_list_ptrs[v7] = v5.offset(4).offset_from(base) as c_long as u16;
                    }
                    ProcessPlmDrawInstruction(word!(game.plm_id), game);
                    CalculatePlmBlockCoords(word!(game.plm_id), game);
                    DrawPlm(word!(game.plm_id), game);

                    break;
                }
                v5 = CallPlmInstr(
                    ((*(v5 as *mut u16)) as u32) | (0x840000u32),
                    v5.offset(2),
                    word!(game.plm_id),
                    game,
                );

                if v5.is_null() {
                    break;
                }
                if (v5 as usize) < 0x10000i32 as usize {
                    v5 = base.offset(v5 as usize as isize);
                }
                if word!(game.queued_message_box_index) != 0 {
                    *(ram_ptr!(game, u16; 0x1d27)).offset((word!(game.plm_id) >> 1) as isize) =
                        v5.offset_from(base) as c_long as u16;
                    message_pending = true;
                }
            }
            word!(game.plm_id, wrapping_sub(2));
        }
    }
    *resume_state = 0;
    0 as CoroutineRet
}
#[inline]
unsafe fn RomPtr_84orRAM(addr: u16, game: &mut Game) -> *mut u8 {
    if addr & 0x8000 != 0 {
        RomPtr((0x840000u32) | ((addr) as u32), game) as *mut u8
    } else {
        crate::native_assertion! {
            '_c2rust_label: addr < 0x2000;
            b"addr < 0x2000\0", b"src/sm_84.c\0", 265, b"uint8 *RomPtr_84orRAM(uint16_t)\0"
        };
        RomPtr_RAM(addr, game)
    }
}
pub unsafe fn ProcessPlmDrawInstruction(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1) * 2;
    let draw_instruction =
        u16::from_le_bytes([game.g_ram[0xde6c + slot], game.g_ram[0xde6d + slot]]);
    let mut words = RomPtr_84orRAM(draw_instruction, game) as *const u16;
    let mut destination =
        u16::from_le_bytes([game.g_ram[0x1c87 + slot], game.g_ram[0x1c88 + slot]]);
    let origin = destination;
    loop {
        let command = words.read_unaligned();
        words = words.add(1);
        let stride = if command & 0x8000 != 0 {
            usize::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]])) * 2
        } else {
            2
        };
        for block in 0..usize::from(command as u8) {
            let word = words.read_unaligned();
            words = words.add(1);
            // The original word indexing rounds odd byte destinations down.
            let offset = 0x10002 + ((usize::from(destination) + block * stride) >> 1) * 2;
            game.g_ram[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
        }
        let displacement = words.read_unaligned();
        if displacement == 0 {
            break;
        }
        let x = i32::from(displacement as u8 as i8);
        let y = i32::from((displacement >> 8) as u8 as i8);
        let width = i32::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]));
        destination = (i32::from(origin) + 2 * (x + y * width)) as u16;
        words = words.add(1);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallPlmInstrFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9557220 => VariaSuitPickup; 9557434 => GravitySuitPickup;
    }
}
pub unsafe fn PlmInstr_Goto(plmp: *const u8, _k: u16) -> *const u8 {
    let target = (plmp as *const u16).read_unaligned();
    usize::from(target) as *const u8
}
unsafe fn PartiallyQueueVramForSingleScreenPlm(
    a: u16,
    k: u16,
    r20: u16,
    r28: u16,
    game: &mut Game,
) {
    let ent = gVramWriteEntry(k as i32, game);
    (*ent).vram_dst = (r28) | (a);
    let size: u16 = (4u16).wrapping_mul(r20);
    let base: VoidP = (word!(game.plm_draw_tilemap_index)).wrapping_add(0xc6c8);
    (*ent).size = size;
    (*ent.offset(1)).size = size;
    (*ent).src = LongPtr {
        addr: base,
        bank: 126,
    };
    (*ent.offset(1)).src = LongPtr {
        addr: ((base).wrapping_add(size)),
        bank: 126,
    };
}
unsafe fn CalculatePlmDrawTilemapVramDst(
    k: u16,
    r9: u16,
    r12: u16,
    r20: u16,
    r24: u16,
    r26: u16,
    r28: u16,
    game: &mut Game,
) {
    let mut a: u16 = 0;
    let prod: u16 = ((r26 as i32 & 0xf) * 0x40) as u16;
    let v1: u16 = (r24) & (0x1fu16);
    if v1 >= 0x10 {
        a = (prod as i32 + r12 as i32 + 2 * v1 as i32) as u16;
        if (word!(game.bg1_x_offset) & 0x100) != 0 {
            a = (a).wrapping_sub(1024);
        }
    } else {
        a = (prod as i32 + r9 as i32 + 2 * v1 as i32) as u16;
        if (word!(game.bg1_x_offset) & 0x100) != 0 {
            a = (a).wrapping_add(1024);
        }
    }
    PartiallyQueueVramForSingleScreenPlm(a, k, r20, r28, game);
}
pub unsafe fn DrawPlm(mut k: u16, game: &mut Game) {
    let mut v35: u16 = 0;
    let mut r22: u16 = 0;
    let mut a: u16 = 0;
    let mut r24: u16 = 0;
    let mut r20: u16 = 0;
    let mut r18: u16 = 0;
    let r9: u16 = addr_unk_605000 as u16;
    let r12: u16 = addr_unk_6053E0 as u16;
    let mut v1: u16 = word!(game.plm_instruction_draw_ptr[(k >> 1)]);
    let mut x: u16 = word!(game.plm_x_block);
    let mut y: u16 = word!(game.plm_y_block);
    let mut r3: u16 = 0;
    let mut r0 = null_mut::<u16>();
    let mut r6 = null_mut::<u16>();
    '_LABEL_2: loop {
        let mut r26: u16 = (word!(game.layer1_y_pos) >> 4);
        if (word!(game.layer1_y_pos) as i32 >> 4) + 15 - y as i32 & 0x8000 != 0 {
            return;
        }
        let mut v2: i16 = *(RomPtr_84orRAM(v1, game) as *mut u16) as i16;
        if v2 < 0 {
            r20 = ((v2) as u16) & (0x7fffu16);
            if !(word!(game.layer1_x_pos) as i32 >> 4 == x as i32
                || (((word!(game.layer1_x_pos) as i32 >> 4) - x as i32) as i16) < 0)
            {
                break;
            }
            let mut v20: i16 = ((word!(game.layer1_x_pos) as i32 >> 4) + 17) as i16;
            if !(v20 as i32 != x as i32 && (v20).wrapping_sub((x) as i16) >= 0) {
                break;
            }
            r24 = x;
            r18 = 0;
            r22 = (r26).wrapping_add(16);
            if ((r26) as i16).wrapping_sub((y) as i16) < 0 {
                r26 = y;
            } else {
                r18 = (r26).wrapping_sub(y);
                if y as i32 + r20 as i32 - r26 as i32 & 0x8000 != 0 {
                    return;
                }
                let v21: bool = r20 == r18;
                let v22: bool = ((r20) as i16).wrapping_sub((r18) as i16) < 0;
                r20 = (r20).wrapping_sub(r18);
                if v22 {
                    Unreachable(game);
                }
                if v21 {
                    return;
                }
            }
            r22 = (r20 as i32 + r26 as i32 - r22 as i32) as u16;
            if !((r22 & 0x8000) != 0 || {
                let mut v23: bool = ((r20) as i16).wrapping_sub((r22) as i16) < 0;
                r20 = (r20).wrapping_sub(r22);
                r20 != 0 && !v23
            }) {
                break;
            }
            v35 = k;
            let mut v24: u16 = word!(game.vram_write_queue_tail);
            if !(((word!(game.vram_write_queue_tail)) as i16).wrapping_sub(240) < 0
                && (((512u16).wrapping_sub(word!(game.plm_draw_tilemap_index))) as i32 >> 3)
                    - r20 as i32
                    & 0x8000
                    == 0)
            {
                break;
            }
            CalculatePlmDrawTilemapVramDst(
                word!(game.vram_write_queue_tail),
                r9,
                r12,
                r20,
                r24,
                r26,
                0x8000,
                game,
            );
            let mut v25 = gVramWriteEntry(v24 as i32, game);
            (*v25.offset(1)).vram_dst = ((*v25).vram_dst).wrapping_add(1);
            r0 = ram_ptr!(game, u16; (*v25).src.addr as i32 as isize);
            r6 = ram_ptr!(game, u16; (*v25.offset(1)).src.addr as i32 as isize);
            word!(game.vram_write_queue_tail = (v24).wrapping_add(14));
            r18 = (r18).wrapping_mul(2);
            r3 = (r18 as i32 + v1 as i32 + 2) as u16;
            let mut v26: u16 = 0;
            do_until! { r20 == 0;
                x = *(RomPtr_84orRAM(r3, game) as *mut u16);
                let mut v29: u16 = (v26).wrapping_add(2);
                TileTables::draw_block(
                    (&raw const game.g_ram).cast::<u8>().add(0xa000).cast(),
                    x,
                    [
                        r0.add((v26 >> 1) as usize),
                        r6.add((v26 >> 1) as usize),
                        r0.add((v29 >> 1) as usize),
                        r6.add((v29 >> 1) as usize),
                    ],
                    TileDrawOrder::Rows,
                );
                v26 = (v29).wrapping_add(2);
                r3 = r3.wrapping_add(1);
                r3 = r3.wrapping_add(1);
                word!(game.plm_draw_tilemap_index, wrapping_add(8));
                if word!(game.plm_draw_tilemap_index) as i32 - 512i32 & 0x8000 == 0 {
                    break '_LABEL_2;
                }
                r20 = r20.wrapping_sub(1);
            }
        } else {
            r20 = ((v2) as u16) & (0x7fffu16);
            if y as i32 - r26 as i32 & 0x8000 != 0 {
                return;
            }
            r26 = y;
            r18 = 0;
            r24 = x;
            r22 = ((((word!(game.layer1_x_pos)).wrapping_add(15)) as i32 >> 4) - 1i32) as u16;
            if ((r22) as i16).wrapping_sub((x) as i16) >= 0 && r22 != x {
                r18 = (r22).wrapping_sub(x);
                if x as i32 + r20 as i32 == r22 as i32
                    || ((x as i32 + r20 as i32 - r22 as i32) as i16) < 0
                {
                    return;
                }
                r20 = (r20).wrapping_sub(r18);
                r24 = r22;
            }
            r22 = (r22).wrapping_add(17);
            if (r22 as i32 - x as i32 & 0x8000 != 0) {
                break;
            }
            r22 = (r20 as i32 + r24 as i32 - 1i32 - r22 as i32) as u16;
            if !((r22 & 0x8000) != 0 || {
                r20 = (r20).wrapping_sub(r22);
                r20 != 0
            }) {
                break;
            }
            v35 = k;
            let v3: u16 = word!(game.vram_write_queue_tail);
            if !(((word!(game.vram_write_queue_tail)) as i16).wrapping_sub(480) < 0
                && (((512u16).wrapping_sub(word!(game.plm_draw_tilemap_index))) as i32 >> 3)
                    - r20 as i32
                    & 0x8000
                    == 0)
            {
                break;
            }
            let prod: u16 = ((r26 as i32 & 0xf) * 0x40) as u16;
            let v4: u16 = (r24) & (0x1fu16);
            if v4 >= 0x10 {
                let mut v8: i16 = (r12 as i32 + 2 * v4 as i32) as i16;
                a = (prod).wrapping_add((v8) as u16);
                if (word!(game.bg1_x_offset) & 0x100) != 0 {
                    a = (a).wrapping_sub(1024);
                }
            } else {
                let mut v5: i16 = (r9 as i32 + 2 * v4 as i32) as i16;
                let RegWord: u16 = prod;
                a = (RegWord).wrapping_add((v5) as u16);
                if (word!(game.bg1_x_offset) & 0x100) != 0 {
                    a = (a).wrapping_add(1024);
                }
            }
            x = (2u16).wrapping_mul(r20);
            let R34: u16 = (a) & (0x1fu16);
            if (2 * r20 as i32 + (a as i32 & 0x1f) - 1i32 & 0xffe0) != 0 {
                if ((v3) as i16).wrapping_sub(228) >= 0 || (32i16).wrapping_sub((R34) as i16) < 0 {
                    return;
                }
                let v9: u16 = (2 * (32 - R34 as i32)) as u16;
                let mut v10 = gVramWriteEntry(v3 as i32, game);
                (*v10).size = v9;
                (*v10.offset(2)).size = v9;
                (*v10).vram_dst = a;
                (*v10.offset(1)).vram_dst = (a & 0xffe0 ^ 0x400);
                (*v10.offset(3)).vram_dst = ((*v10.offset(1)).vram_dst).wrapping_add(32);
                (*v10.offset(2)).vram_dst = ((*v10).vram_dst).wrapping_add(32);
                x = (4u16).wrapping_mul(r20);
                let v11: u16 = (4 * r20 as i32 - (*v10).size as i32) as u16;
                (*v10.offset(1)).size = v11;
                (*v10.offset(3)).size = v11;
                let mut v12: VoidP = (word!(game.plm_draw_tilemap_index)).wrapping_sub(14648);
                (*v10).src.addr = (word!(game.plm_draw_tilemap_index)).wrapping_sub(14648);
                r0 = ram_ptr!(game, u16; v12 as i32 as isize);
                let mut v13: VoidP = ((*v10).size).wrapping_add(v12);
                (*v10.offset(1)).src.addr = v13;
                let v14: u16 = ((*v10.offset(1)).size).wrapping_add(v13);
                (*v10.offset(2)).src.addr = v14;
                r6 = ram_ptr!(game, u16; v14 as i32 as isize);
                (*v10.offset(3)).src.addr = ((*v10.offset(2)).size).wrapping_add(v14);
                (*v10).src.bank = 126;
                (*v10.offset(1)).src.bank = 126;
                (*v10.offset(2)).src.bank = 126;
                (*v10.offset(3)).src.bank = 126;
                word!(game.vram_write_queue_tail = (v3).wrapping_add(28));
            } else {
                PartiallyQueueVramForSingleScreenPlm(a, v3, r20, 0, game);
                let mut v15 = gVramWriteEntry(v3 as i32, game);
                (*v15.offset(1)).vram_dst = ((*v15).vram_dst).wrapping_add(32);
                r0 = ram_ptr!(game, u16; (*v15).src.addr as i32 as isize);
                r6 = ram_ptr!(game, u16; (*v15.offset(1)).src.addr as i32 as isize);
                word!(game.vram_write_queue_tail = (v3).wrapping_add(14));
            }
            r18 = (r18).wrapping_mul(2);
            r3 = (r18 as i32 + v1 as i32 + 2) as u16;
            let mut v16: u16 = 0;
            do_until! { r20 == 0;
                x = *(RomPtr_84orRAM(r3, game) as *mut u16);
                let mut v19: u16 = (v16).wrapping_add(2);
                TileTables::draw_block(
                    (&raw const game.g_ram).cast::<u8>().add(0xa000).cast(),
                    x,
                    [
                        r0.add((v16 >> 1) as usize),
                        r6.add((v16 >> 1) as usize),
                        r0.add((v19 >> 1) as usize),
                        r6.add((v19 >> 1) as usize),
                    ],
                    TileDrawOrder::Columns,
                );
                v16 = (v19).wrapping_add(2);
                r3 = (r3).wrapping_add(2);
                word!(game.plm_draw_tilemap_index, wrapping_add(8));
                if word!(game.plm_draw_tilemap_index) as i32 - 512i32 & 0x8000 == 0 {
                    break '_LABEL_2;
                }
                r20 = r20.wrapping_sub(1);
            }
        }
        k = v35;
        let mut addr: u16 = r3;
        if (r22 & 0x8000) == 0 {
            addr = (r3 as i32 + 2 * r22 as i32) as u16;
        }
        let mut v31: i16 = *(RomPtr_84orRAM(addr, game) as *mut u16) as i16;
        if v31 != 0 {
            x = (word!(game.plm_x_block)).wrapping_add((v31 as i8) as u16);
            let v32: u16 = (addr).wrapping_add(1);
            y = (word!(game.plm_y_block)).wrapping_add((*RomPtr_84orRAM(v32, game) as i8) as u16);
            v1 = (v32).wrapping_add(1);
        } else {
            return;
        }
    }
}
macro_rules! find_plm_at_block {
    ($found:ident, $plm_offset:ident, $block:ident, $game:ident) => {
        let $found: bool;
        let mut $plm_offset: u16 = 78;
        loop {
            if ($block == word!($game.plm_block_indices[($plm_offset >> 1)])) {
                $found = true;
                break;
            }
            $plm_offset = ($plm_offset).wrapping_sub(2);
            if ($plm_offset & 0x8000) != 0 {
                $found = false;
                break;
            }
        }
    };
}

pub unsafe fn ActivateStationIfSamusCannonLinedUp(a: u16, j: u16, game: &mut Game) -> u8 {
    find_plm_at_block!(found_plm, v2, a, game);
    if found_plm {
        CalculatePlmBlockCoords(word!(game.plm_id), game);
        if ((16u16).wrapping_mul(word!(game.plm_y_block))) | 0xb == word!(game.samus_y_pos) {
            let v3: i32 = v2 as i32 >> 1;
            word!(game.plm_instr_list_ptrs[v3] = word!(game.plm_instruction_list_link_reg[v3]));
            word!(game.plm_instruction_timer[v3] = 1);
            CallSomeSamusCode(6, game);
            return 1;
        }
    }
    word!(game.plm_header_ptr[(j >> 1)] = 0);
    1
}
pub fn DeletePlm(j: u16, game: &mut Game) {
    let header_offset = 0x1c37 + usize::from(j & !1);
    game.g_ram[header_offset..header_offset + 2].fill(0);
}
fn setup_extension_block(j: u16, block_type_and_bts: u16, game: &mut Game) -> u8 {
    let slot_offset = 0x1c87 + usize::from(j & !1);
    let block_index = u16::from_le_bytes([game.g_ram[slot_offset], game.g_ram[slot_offset + 1]]);
    WriteLevelDataBlockTypeAndBts(block_index, block_type_and_bts, game);
    DeletePlm(j, game);
    0
}
fn advance_plm_instruction(word_index: usize, bytes: u16, game: &mut Game) {
    let offset = 0x1d27 + word_index * 2;
    let instruction = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    game.g_ram[offset..offset + 2].copy_from_slice(&instruction.wrapping_add(bytes).to_le_bytes());
}
pub fn SkipDebugDrawInstructionsForScrollPlms(j: u16, game: &mut Game) {
    advance_plm_instruction(usize::from(j >> 1), 4, game);
}
fn setup_scroll_block(j: u16, block_type_and_bts: u16, game: &mut Game) -> u8 {
    let slot_index = usize::from(j & !1);
    let block_offset = 0x1c87 + slot_index;
    let block_index = u16::from_le_bytes([game.g_ram[block_offset], game.g_ram[block_offset + 1]]);
    WriteLevelDataBlockTypeAndBts(block_index, block_type_and_bts, game);
    let trigger_offset = 0x1e17 + slot_index;
    game.g_ram[trigger_offset..trigger_offset + 2].fill(0);
    // Read the live instruction after the level-data/BTS and trigger writes.
    SkipDebugDrawInstructionsForScrollPlms(j, game);
    0
}
pub fn PlmSetup_ReturnCarryClear(_j: u16) -> u8 {
    0
}
pub fn PlmSetup_ReturnCarrySet(_j: u16) -> u8 {
    1
}
fn update_level_block_bits(word_index: usize, mask: u16, bits: u16, game: &mut Game) {
    let offset = 0x10002 + word_index * 2;
    let block = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    game.g_ram[offset..offset + 2].copy_from_slice(&((block & mask) | bits).to_le_bytes());
}
fn retain_plm_block_bits(j: u16, mask: u16, game: &mut Game) {
    // Both indices are byte offsets; the original word indexing rounds them down.
    let slot_offset = 0x1c87 + usize::from(j & !1);
    let block_index = u16::from_le_bytes([game.g_ram[slot_offset], game.g_ram[slot_offset + 1]]);
    update_level_block_bits(usize::from(block_index >> 1), mask, 0, game);
}
pub unsafe fn UNUSED_sub_84B3E3(j: u16) -> u8 {
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/sm_84.c\0", 1370, b"uint8 UNUSED_sub_84B3E3(uint16)\0"
    };
    0
}
static g_word_84B48B: [u16; 2] = [0x200, 0x200];
static g_word_84B48F: [u16; 2] = [0x120, 0x100];
static g_word_84B493: [u16; 2] = [0x280, 0x380];
pub fn PlmSetup_QuicksandSurface_0(j: u16, game: &mut Game) {
    game.g_ram[0xb2c..0xb2e].fill(0);
    game.g_ram[0xb2e..0xb30].fill(0);
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    // Write the middle word of the extra 16.16 Y displacement.
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48F[usize::from(j >> 1)].to_le_bytes());
}
pub fn PlmSetup_QuicksandSurface_1(j: u16, game: &mut Game) {
    let physics_index = usize::from(j >> 1);
    let y_speed_middle = u16::from_le_bytes([game.g_ram[0xb2d], game.g_ram[0xb2e]]);
    if g_word_84B493[physics_index] < y_speed_middle {
        game.g_ram[0xb2c..0xb2e].fill(0);
        game.g_ram[0xb2e..0xb30].fill(0);
        game.g_ram[0xb2d..0xb2f].copy_from_slice(&g_word_84B493[physics_index].to_le_bytes());
    }
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48B[physics_index].to_le_bytes());
}
pub fn PlmSetup_QuicksandSurface_2(j: u16, game: &mut Game) {
    game.g_ram[0xb5a..0xb5c].fill(0);
    game.g_ram[0xb5c..0xb5e].fill(0);
    game.g_ram[0xb5b..0xb5d].copy_from_slice(&g_word_84B48B[usize::from(j >> 1)].to_le_bytes());
    game.g_ram[0xaf4..0xaf6].fill(0);
}
/// Apply the collision-side quicksand reaction to the active displacement.
///
/// # Safety
/// A non-null `displacement` must point to a live, writable i32 for the
/// duration of this call, as passed through collision PLM setup.
pub unsafe fn PlmSetup_QuicksandSurfaceB(_j: u16, displacement: *mut i32, game: &mut Game) -> u8 {
    let read_word = |game: &Game, offset: usize| {
        u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
    };
    if read_word(game, 0xb02) & 2 == 0 {
        return 0;
    }
    if displacement.is_null() {
        printf(b"ERROR: PlmSetup_QuicksandSurfaceB is broken!!\n\0" as *const u8 as *const c_char);
        Unreachable(game);
        return 0;
    }
    match read_word(game, 0xb36) & 3 {
        0 | 3 => {
            if read_word(game, 0xb02) & 0x0f == 3 {
                if read_word(game, 0xa6e) == 1 {
                    *displacement = 0;
                    return 1;
                }
                // Compare the unsigned middle word; retain both outside bytes.
                if (*displacement >> 8) as u16 > 0x30 {
                    *displacement = ((*displacement as u32 & 0xff0000ff) | 0x3000) as i32;
                }
                let in_quicksand = read_word(game, 0x1e71).wrapping_add(1);
                game.g_ram[0x1e71..0x1e73].copy_from_slice(&in_quicksand.to_le_bytes());
            }
        }
        2 => {
            if read_word(game, 0xa6e) == 1 {
                *displacement = 0;
                return 1;
            }
            let in_quicksand = read_word(game, 0x1e71).wrapping_add(1);
            game.g_ram[0x1e71..0x1e73].copy_from_slice(&in_quicksand.to_le_bytes());
        }
        _ => {}
    }
    0
}
pub use self::PlmSetup_ReturnCarryClear as PlmSetup_B73B_B73F_SandFalls;
pub unsafe fn WakePlmIfSamusIsBelowAndRightOfTarget(
    k: u16,
    x_r18: u16,
    y_r20: u16,
    game: &mut Game,
) -> u8 {
    if x_r18 < word!(game.samus_x_pos) && y_r20 < word!(game.samus_y_pos) {
        let v2: i32 = k as i32 >> 1;
        word!(game.plm_instr_list_ptrs[v2], wrapping_add(2));
        word!(game.plm_instruction_timer[v2] = 1);
        return false_0 as u8;
    }
    true_0 as u8
}
pub fn PlmSetup_B974(j: u16) -> u8 {
    0
}
crate::forward_calls! { pub unsafe fn (j: u16, game: &mut Game) -> u8;
    sub_84C63F => sub_84C64C(
        j,
        (word!(game.plm_block_indices[(j >> 1)])).wrapping_add(2),
        game,
    );
    sub_84C647 => sub_84C64C(
        j,
        (word!(game.plm_block_indices[(j >> 1)])).wrapping_sub(2),
        game,
    );
}
pub unsafe fn sub_84C64C(j: u16, a: u16, game: &mut Game) -> u8 {
    find_plm_at_block!(found_plm, v2, a, game);
    if found_plm {
        let mut v3: i32 = v2 as i32 >> 1;
        if word!(game.plm_timers[v3]) == 0 {
            word!(game.plm_timers[v3], wrapping_add(1));
        }
    }
    word!(game.plm_header_ptr[(j >> 1)] = 0);
    1
}
pub unsafe fn SetBts0x10FiveStepsDown(j: u16, game: &mut Game) {
    let v1: u16 = SetBtsTo0x10AdvanceRow((word!(game.plm_block_indices[(j >> 1)]) >> 1), game);
    let v2: u16 = SetBtsTo0x10AdvanceRow(v1, game);
    let v3: u16 = SetBtsTo0x10AdvanceRow(v2, game);
    let v4: u16 = SetBtsTo0x10AdvanceRow(v3, game);
    SetBtsTo0x10AdvanceRow(v4, game);
}
pub unsafe fn SetBtsTo0x10AdvanceRow(k: u16, game: &mut Game) -> u16 {
    *(ram_ptr!(game, u16; 0x16402, k as isize)) =
        ((*ram_ptr!(game; 0x16402, (k as i32 + 1) as isize) as i32) << 8 | 0x10) as u16;
    (word!(game.room_width_in_blocks)).wrapping_add(k)
}
pub unsafe fn SetBts0x10FiveStepsUp(j: u16, game: &mut Game) {
    let v1: u16 = SetBtsTo0x10AdvanceRowUp((word!(game.plm_block_indices[(j >> 1)]) >> 1), game);
    let v2: u16 = SetBtsTo0x10AdvanceRowUp(v1, game);
    let v3: u16 = SetBtsTo0x10AdvanceRowUp(v2, game);
    let v4: u16 = SetBtsTo0x10AdvanceRowUp(v3, game);
    SetBtsTo0x10AdvanceRowUp(v4, game);
}
pub unsafe fn SetBtsTo0x10AdvanceRowUp(k: u16, game: &mut Game) -> u16 {
    *(ram_ptr!(game, u16; 0x16402, k as isize)) =
        ((*ram_ptr!(game; 0x16402, (k as i32 + 1) as isize) as i32) << 8 | 0x10) as u16;
    (k).wrapping_sub(word!(game.room_width_in_blocks))
}
pub unsafe fn PlmSetup_C82A_DownwardsClosedGate(j: u16, setup_a: u16, game: &mut Game) -> u8 {
    SpawnEprojWithRoomGfx(0xe659, setup_a, game);
    SetBts0x10FiveStepsDown(j, game);
    0
}
macro_rules! install_optional_gate_block {
    ($game:ident, $block_table:literal, $plm_index:ident, $neighbor_operation:ident) => {
        let v2: u16 = crate::rtl::rom_table_word!(
            $game,
            $block_table,
            (word!($game.plm_room_arguments[$plm_index]) >> 1) as isize
        );
        if v2 != 0 {
            WriteLevelDataBlockTypeAndBts(
                (word!($game.plm_block_indices[$plm_index])).$neighbor_operation(2),
                v2,
                $game,
            );
        }
    };
}

pub fn PlmSetup_D0D8_SetVFlag(j: u16) -> u8 {
    0x41
}
pub fn PlmSetup_D0D8_ClearVflag(j: u16) -> u8 {
    1
}
pub unsafe fn PlmSetup_D6DA_LowerNorfairChozoHandTrigger(
    j: u16,
    displacement: *mut i32,
    game: &mut Game,
) -> u8 {
    if (word!(game.collected_items) & 0x200) != 0
        && (word!(game.samus_collision_direction) & 0xf) == 3
        && (word!(game.samus_pose) as i32 == kPose_1D_FaceR_Morphball_Ground as i32
            || word!(game.samus_pose) as i32 == kPose_79_FaceR_Springball_Ground as i32
            || word!(game.samus_pose) as i32 == kPose_7A_FaceL_Springball_Ground as i32)
    {
        SetEventHappened(0xc, game);
        (*(ram_ptr!(game, EnemyData; 0xf78))).parameter_1 = 1;
        let v2: i32 = word!(game.plm_block_indices[(j >> 1)]) as i32 >> 1;
        word!(game.level_data[v2], |fresh3| (fresh3) & (0xfffu16));
        CallSomeSamusCode(0, game);
        spawn_hardcoded_plm(
            SpawnHardcodedPlmArgs::new(0xc, 0x1d, 0xd113 as VoidP),
            displacement,
            game,
        );
    }
    word!(game.plm_header_ptr[(j >> 1)] = 0);
    1
}
pub unsafe fn SpawnMotherBrainGlassShatteringShard(a: u16, game: &mut Game) {
    SpawnEprojWithRoomGfx(0xcefc, a, game);
}
pub unsafe fn PlmSetup_D6F2_WreckedShipChozoHandTrigger(
    j: u16,
    displacement: *mut i32,
    game: &mut Game,
) -> u8 {
    if CheckBossBitForCurArea(1, game) & 1 != 0
        && (word!(game.samus_collision_direction) & 0xf) == 3
        && (word!(game.samus_pose) as i32 == kPose_1D_FaceR_Morphball_Ground as i32
            || word!(game.samus_pose) as i32 == kPose_79_FaceR_Springball_Ground as i32
            || word!(game.samus_pose) as i32 == kPose_7A_FaceL_Springball_Ground as i32)
    {
        (*(ram_ptr!(game, EnemyData; 0xf78))).parameter_1 = 1;
        *(ram_ptr!(game, u16; 0xcd20, 7)) = 514;
        *(ram_ptr!(game, u16; 0xcd20, 13)) = 257;
        let v1: i32 = word!(game.plm_block_indices[(j >> 1)]) as i32 >> 1;
        word!(game.level_data[v1], |fresh2| (fresh2) & (0xfffu16));
        CallSomeSamusCode(0, game);
        spawn_hardcoded_plm(
            SpawnHardcodedPlmArgs::new(0x17, 0x1d, 0xd6f8 as VoidP),
            displacement,
            game,
        );
    }
    word!(game.plm_header_ptr[(j >> 1)] = 0);
    1
}
pub use self::PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused as PlmSetup_D70C_NoobTube;
pub unsafe fn sub_84D7EF(k: u16, game: &mut Game) {
    let v1: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + k as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v1, 0xd0ff, game);
    let v2: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v1 as i32) as u16;
    WriteLevelDataBlockTypeAndBts(v2, 0xd0fe, game);
    WriteLevelDataBlockTypeAndBts(
        (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16,
        0xd0fd,
        game,
    );
}
pub unsafe fn SetPlmVarPtr(k: u16, a: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; word!(game.plm_variable[(k >> 1)]) as isize)) = a;
}
pub use self::PlmInstr_DamageDraygonTurret as PlmInstr_DamageDraygonTurret2;
pub use self::PlmSetup_DraygonCannonFacingRight as PlmSetup_DraygonCannonFacingLeft;
pub unsafe fn sub_84EE5F(j: u16, a: u16, game: &mut Game) -> u8 {
    word!(game.plm_variables[(j >> 1)] = a);
    sub_84EE64(j, game)
}
pub unsafe fn sub_84EE89(j: u16, a: u16, game: &mut Game) -> u8 {
    word!(game.plm_variables[(j >> 1)] = a);
    sub_84EE8E(j, game)
}
crate::game::callback_dispatch! {
    /// Dispatch setup with the active collision displacement, or null outside collision.
/// `setup_a` preserves the spawner accumulator for setup routines that consume it.
///
/// # Safety
/// Legacy setup callbacks require initialized runtime state. A non-null
/// `displacement` must remain live and writable throughout the setup call.
    pub unsafe fn CallPlmHeaderFunc(
    ea: u32,
    j: u16,
    displacement: *mut i32,
    setup_a: u16,
    game: &mut Game,
) -> u8 => ea;
    call (j, game); default Unreachable(game) as u8;
    @functions [pub unsafe fn (j: u16, game: &mut Game) -> u8];
    [8694568] PlmSetup_CrumbleBotwoonWall [] {
        word!(game.plm_instruction_timer[(j >> 1)] = 64);
        0
    };
    [8695882] PlmSetup_SetrupWreckedShipEntrance [] {
        let mut v1: u16 = word!(game.plm_block_indices[(j >> 1)]);
        for _ in 0i16..56i16 {
            word!(game.level_data[(v1 >> 1)] = 255);
            v1 = (v1).wrapping_add(2);
        }
        0
    };
    [8696028] PlmSetup_BTS_Brinstar_0x80_Floorplant [] {
        if (word!(game.samus_y_radius) as i32 + word!(game.samus_y_pos) as i32 - 1i32 & 0xf) == 15 {
            let v1: i32 = word!(game.plm_block_indices[(j >> 1)]) as i32 >> 1;
            word!(game.level_data[v1], |fresh20| (fresh20) & (0x8fffu16));
            let v2: i32 = j as i32 >> 1;
            word!(game.plm_variable[v2] = word!(game.samus_x_pos));
            word!(game.plm_variables[v2] = (word!(game.samus_y_pos)).wrapping_sub(1));
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8696083] PlmSetup_BTS_Brinstar_0x81_Ceilingplant [] {
        if (word!(game.samus_y_pos) as i32 - word!(game.samus_y_radius) as i32 & 0xf) != 0 {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        } else {
            let v1: i32 = word!(game.plm_block_indices[(j >> 1)]) as i32 >> 1;
            word!(game.level_data[v1], |fresh19| (fresh19) & (0x8fffu16));
            let v2: i32 = j as i32 >> 1;
            word!(game.plm_variable[v2] = word!(game.samus_x_pos));
            word!(game.plm_variables[v2] = (word!(game.samus_y_pos)).wrapping_add(1));
        }
        0
    };
    [8696264] PlmSetup_Bts47_MapStationRightAccess [] {
        if (word!(game.samus_collision_direction) & 0xf) == 0
            && word!(game.samus_pose) as i32 == kPose_8A_FaceL_Ranintowall as i32
            && (*ram_ptr!(game; 0xa1e) & 4) != 0
        {
            return ActivateStationIfSamusCannonLinedUp(
                (word!(game.plm_block_indices[(j >> 1)])).wrapping_sub(2),
                j,
                game,
            );
        }
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        1
    };
    [8696304] PlmSetup_Bts4_MapStationLeftAccess [] {
        if (word!(game.samus_collision_direction) & 0xf) == 1
            && word!(game.samus_pose) as i32 == kPose_89_FaceR_Ranintowall as i32
            && (*ram_ptr!(game; 0xa1e) & 8) != 0
        {
            return ActivateStationIfSamusCannonLinedUp(
                (word!(game.plm_block_indices[(j >> 1)])).wrapping_add(4),
                j,
                game,
            );
        }
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        1
    };
    [8696429] PlmSetup_B6E3_EnergyStationRightAccess [] {
        if (word!(game.samus_collision_direction) & 0xf) != 0
            || word!(game.samus_pose) as i32 != kPose_8A_FaceL_Ranintowall as i32
            || (*ram_ptr!(game; 0xa1e) & 4) == 0
            || word!(game.samus_health) == word!(game.samus_max_health)
        {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
            1
        } else {
            ActivateStationIfSamusCannonLinedUp(
                (word!(game.plm_block_indices[(j >> 1)])).wrapping_sub(2),
                j,
                game,
            )
        }
    };
    [8696477] PlmSetup_B6E7_EnergyStationLeftAccess [] {
        if (word!(game.samus_collision_direction) & 0xf) == 1
            && word!(game.samus_pose) as i32 == kPose_89_FaceR_Ranintowall as i32
            && (*ram_ptr!(game; 0xa1e) & 8) != 0
            && word!(game.samus_health) != word!(game.samus_max_health)
        {
            return ActivateStationIfSamusCannonLinedUp(
                (word!(game.plm_block_indices[(j >> 1)])).wrapping_add(2),
                j,
                game,
            );
        }
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        1
    };
    [8696528] PlmSetup_B6EF_MissileStationRightAccess [] {
        if (word!(game.samus_collision_direction) & 0xf) != 0
            || word!(game.samus_pose) as i32 != kPose_8A_FaceL_Ranintowall as i32
            || (*ram_ptr!(game; 0xa1e) & 4) == 0
            || word!(game.samus_missiles) == word!(game.samus_max_missiles)
        {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
            return 1;
        }
        ActivateStationIfSamusCannonLinedUp(
            (word!(game.plm_block_indices[(j >> 1)])).wrapping_sub(2),
            j,
            game,
        )
    };
    [8696576] PlmSetup_B6F3_MissileStationLeftAccess [] {
        if (word!(game.samus_collision_direction) & 0xf) == 1
            && word!(game.samus_pose) as i32 == kPose_89_FaceR_Ranintowall as i32
            && (*ram_ptr!(game; 0xa1e) & 8) != 0
            && word!(game.samus_missiles) != word!(game.samus_max_missiles)
        {
            ActivateStationIfSamusCannonLinedUp(
                (word!(game.plm_block_indices[(j >> 1)])).wrapping_add(2),
                j,
                game,
            )
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
            1
        }
    };
    [8697169] PlmSetup_ClearShitroidInvisibleWall [] {
        let mut v1: u16 = word!(game.plm_block_indices[(j >> 1)]);
        for _ in 0i16..10i16 {
            word!(game.level_data[(v1 >> 1)], |fresh10| (fresh10) & (0xfffu16));
            v1 = (v1 as i32 + word!(game.room_width_in_blocks) as i32 * 2) as u16;
        }
        0
    };
    [8697199] PlmSetup_B767_ClearShitroidInvisibleWall [] {
        let mut v1: u16 = word!(game.plm_block_indices[(j >> 1)]);
        for _ in 0i16..10i16 {
            word!(
                game.level_data[(v1 >> 1)] = (word!(game.level_data[(v1 >> 1)]) & 0xfff | 0x8000)
            );
            v1 = (v1 as i32 + word!(game.room_width_in_blocks) as i32 * 2) as u16;
        }
        0
    };
    [8697232] PlmSetup_B76B_SaveStationTrigger [] {
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        if read_word(game, 0x592) == 0 // power_bomb_explosion_status
        && (game.samus_pose() == kPose_01_FaceR_Normal as u16
            || game.samus_pose() == kPose_02_FaceL_Normal as u16)
        && read_word(game, 0x1e75) == 0 // save_station_lockout_flag
        && game.samus_collision_direction() & 0xf == 3
        {
            CalculatePlmBlockCoords(j, game);
            let samus_block = game.samus_x_pos().wrapping_sub(8) >> 4;
            let station_block = read_word(game, 0x1c29); // plm_x_block
            if samus_block == station_block {
                let trigger_offset = usize::from(j >> 1) * 2;
                let block_index = read_word(game, 0x1c87 + trigger_offset);
                // Remove the temporary trigger before finding the station's PLM.
                game.g_ram[0x1c87 + trigger_offset..0x1c89 + trigger_offset]
                    .copy_from_slice(&0u16.to_le_bytes());
                game.g_ram[0x1c37 + trigger_offset..0x1c39 + trigger_offset]
                    .copy_from_slice(&0u16.to_le_bytes());
                for slot in (0..40).rev() {
                    let slot_offset = slot * 2;
                    if read_word(game, 0x1c87 + slot_offset) == block_index {
                        advance_plm_instruction(slot, 1, game);
                        advance_plm_instruction(slot, 1, game);
                        game.g_ram[0xde1c + slot_offset..0xde1e + slot_offset]
                            .copy_from_slice(&1u16.to_le_bytes());
                        break;
                    }
                }
            }
        }
        1
    };
    [8697326] PlmSetup_B76F_SaveStation [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0xb04d, game);
        0
    };
    [8697336] PlmSetup_MotherBrainRoomEscapeDoor [] {
        let v1: u16 = word!(game.plm_block_indices[(j >> 1)]);
        WriteLevelDataBlockTypeAndBts(v1, 0x9001, game);
        let v2: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v1 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v2, 0xd0ff, game);
        let v3: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v3, 0xd0ff, game);
        WriteLevelDataBlockTypeAndBts(
            (word!(game.room_width_in_blocks) as i32 * 2 + v3 as i32) as u16,
            0xd0ff,
            game,
        );
        0
    };
    [8697795] PlmSetup_B7EB_EnableSoundsIn32Frames [] {
        let mut v1: u16 = if word!(game.area_index) == 6 { 32 } else { 240 };
        let v2: i32 = j as i32 >> 1;
        word! {game; plm_timers[v2] = v1; plm_pre_instrs[v2] = ((0x84b7ddi32) as u16); }
        0
    };
    [8698012] PlmSetup_SpeedBoosterEscape [] {
        if CheckEventHappened(0x15, game) != 0 {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8698076] PlmSetup_ShaktoolsRoom [] {
        *(ram_ptr!(game, u16; 0xcd20)) = 1;
        *(ram_ptr!(game, u16; 0xcd20, 2)) = 0;
        0
    };
    [8698232] PlmSetup_B9C1_CrittersEscapeBlock [] {
        if *(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize) != 0 {
            let v1: i32 = j as i32 >> 1;
            let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
            let v3: u16 = (word!(game.level_data[v2]) & 0xf000 | 0x9f);
            word! {game; plm_variable[v1] = v3; level_data[v2] = (v3) & (0x8fffu16); }
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8698309] PlmSetup_B9ED_CrittersEscapeBlock [] {
        let v1: u16 = word!(game.plm_block_indices[(j >> 1)]);
        WriteLevelDataBlockTypeAndBts(v1, 0xc04f, game);
        let v2: u16 = (v1 as i32 + word!(game.room_width_in_blocks) as i32 * 2) as u16;
        WriteLevelDataBlockTypeAndBts(v2, 0xd0ff, game);
        WriteLevelDataBlockTypeAndBts(
            (v2 as i32 + word!(game.room_width_in_blocks) as i32 * 2) as u16,
            0xd0ff,
            game,
        );
        0
    };
    [8698353] sub_84B9F1 [] {
        let v1: u16 = word!(game.plm_block_indices[(j >> 1)]);
        update_level_block_bits((v1 >> 1) as usize, 0xfff, 0x8000, game);
        let v2: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v1 as i32) as u16;
        update_level_block_bits((v2 >> 1) as usize, 0xfff, 0x8000, game);
        let v3: u16 = (word!(game.room_width_in_blocks) as i32
            + word!(game.room_width_in_blocks) as i32
            + v2 as i32) as u16;
        update_level_block_bits((v3 >> 1) as usize, 0xfff, 0x8000, game);
        let v4: i32 = (word!(game.room_width_in_blocks) as i32 * 2 + v3 as i32) as u16 as i32 >> 1;
        update_level_block_bits(v4 as usize, 0xfff, 0x8000, game);
        0
    };
    [8698633] PlmSetup_BB30_CrateriaMainstreetEscape [] {
        if CheckEventHappened(0xf, game) == 0 {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8701261] PlmSetup_C806_LeftGreenGateTrigger [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xfff)
            == 512
        {
            return sub_84C63F(j, game);
        }
        QueueSfx2_Max6(0x57, game);
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        0
    };
    [8701292] PlmSetup_C80A_RightGreenGateTrigger [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xfff)
            == 512
        {
            return sub_84C647(j, game);
        }
        QueueSfx2_Max6(0x57, game);
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        0
    };
    [8701323] PlmSetup_C80E_LeftRedGateTrigger [] {
        let mut v1: i16 = ((*(ram_ptr!(game, u16; 0xc18))
            .offset((word!(game.projectile_index) >> 1) as isize))
            as i16)
            & (0xfffi16);
        if v1 == 256 || v1 == 512 {
            return sub_84C63F(j, game);
        }
        QueueSfx2_Max6(0x57, game);
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        0
    };
    [8701359] PlmSetup_C812_RightRedGateTrigger [] {
        let mut v1: i16 = ((*(ram_ptr!(game, u16; 0xc18))
            .offset((word!(game.projectile_index) >> 1) as isize))
            as i16)
            & (0xfffi16);
        if v1 == 256 || v1 == 512 {
            return sub_84C647(j, game);
        }
        QueueSfx2_Max6(0x57, game);
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        0
    };
    [8701425] PlmSetup_C822_RightYellowGateTrigger [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xfff)
            != 768
        {
            return sub_84C647(j, game);
        }
        QueueSfx2_Max6(0x57, game);
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        0
    };
    [8701395] PlmSetup_C81E_LeftYellowGateTrigger [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xfff)
            == 768
        {
            return sub_84C63F(j, game);
        }
        QueueSfx2_Max6(0x57, game);
        let v1: i32 = j as i32 >> 1;
        word!(game.plm_header_ptr[v1] = 0);
        PlmSetup_C822_RightYellowGateTrigger(j, game)
    };
    [8701479] PlmSetup_C81A_RightBlueGateTrigger [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xfff)
            != 768
        {
            return sub_84C647(j, game);
        }
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        0
    };
    [8701456] PlmSetup_C816_LeftBlueGateTrigger [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xfff)
            != 768
        {
            return sub_84C63F(j, game);
        }
        word!(game.plm_header_ptr[(j >> 1)] = 0);
        PlmSetup_C81A_RightBlueGateTrigger(j, game)
    };
    [8701643] PlmSetup_C832_UpwardsClosedGate [] {
        SpawnEprojWithRoomGfx(0xe675, 0, game);
        SetBts0x10FiveStepsUp(j, game);
        0
    };
    [8701656] PlmSetup_C826_DownwardsOpenGate [] {
        SetBts0x10FiveStepsDown(j, game);
        0
    };
    [8701660] PlmSetup_C82E_UpwardsOpenGate [] {
        SetBts0x10FiveStepsUp(j, game);
        0
    };
    [8701883] PlmSetup_Door_Blue [] {
        if word!(game.projectile_index) & 0x8000 == 0
            && (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
                & 0xf00)
                == 768
        {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        } else {
            let index = usize::from(j >> 1) * 2;
            let v1 = u16::from_le_bytes([game.g_ram[0x1c87 + index], game.g_ram[0x1c88 + index]]);
            word!(
                game.level_data[(v1 >> 1)] = (word!(game.level_data[(v1 >> 1)]) & 0xfff | 0x8000)
            );
        }
        0
    };
    [8703426] PlmSetup_D028_D02C_Unused [] {
        if word!(game.samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
            || word!(game.samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
        {
            let v1: i32 = j as i32 >> 1;
            let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
            let v3: u16 = word!(game.level_data[v2]);
            word! {game; plm_variable[v1] = v3; level_data[v2] = (v3) & (0xfffu16); }
            0
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
            1
        }
    };
    [8703466] PlmSetup_RespawningSpeedBoostBlock [] {
        if (word!(game.speed_boost_counter) & 0xf00) == 1024
            || word!(game.samus_pose) as i32 == kPose_C9_FaceR_Shinespark_Horiz as i32
            || word!(game.samus_pose) as i32 == kPose_CA_FaceL_Shinespark_Horiz as i32
            || word!(game.samus_pose) as i32 == kPose_CB_FaceR_Shinespark_Vert as i32
            || word!(game.samus_pose) as i32 == kPose_CC_FaceL_Shinespark_Vert as i32
            || word!(game.samus_pose) as i32 == kPose_CD_FaceR_Shinespark_Diag as i32
            || word!(game.samus_pose) as i32 == kPose_CE_FaceL_Shinespark_Diag as i32
        {
            let v1: i32 = j as i32 >> 1;
            let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
            let v3: u16 = (word!(game.level_data[v2]) & 0xf000 | 0xb6);
            word! {game; plm_variable[v1] = v3; level_data[v2] = (v3) & (0xfffu16); }
            0
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
            1
        }
    };
    [8703543] PlmSetup_RespawningCrumbleBlock [] {
        if (word!(game.samus_collision_direction) & 0xf) == 3 {
            let v1: i32 = j as i32 >> 1;
            let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
            let v3: u16 = (word!(game.level_data[v2]) & 0xf000 | 0xbc);
            word! {game; plm_variable[v1] = v3; level_data[v2] = (v3) & (0x8fffu16);
                plm_instruction_timer[v1] = 4; }
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        1
    };
    [8703619] PlmSetup_RespawningBombBlock [] {
        if (word!(game.speed_boost_counter) & 0xf00) == 1024
            || word!(game.samus_pose) as i32 == kPose_81_FaceR_Screwattack as i32
            || word!(game.samus_pose) as i32 == kPose_82_FaceL_Screwattack as i32
            || word!(game.samus_pose) as i32 == kPose_C9_FaceR_Shinespark_Horiz as i32
            || word!(game.samus_pose) as i32 == kPose_CA_FaceL_Shinespark_Horiz as i32
            || word!(game.samus_pose) as i32 == kPose_CB_FaceR_Shinespark_Vert as i32
            || word!(game.samus_pose) as i32 == kPose_CC_FaceL_Shinespark_Vert as i32
            || word!(game.samus_pose) as i32 == kPose_CD_FaceR_Shinespark_Diag as i32
            || word!(game.samus_pose) as i32 == kPose_CE_FaceL_Shinespark_Diag as i32
        {
            let v2: i32 = j as i32 >> 1;
            let v3: i32 = word!(game.plm_block_indices[v2]) as i32 >> 1;
            let v4: u16 = (word!(game.level_data[v3]) & 0xf000 | 0x58);
            word! {game; plm_variable[v2] = v4; level_data[v3] = (v4) & (0xfffu16); }
            0
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
            1
        }
    };
    [8703706] PlmSetup_RespawningBombBlock2 [] {
        let mut v1: i16 = ((*(ram_ptr!(game, u16; 0xc18))
            .offset((word!(game.projectile_index) >> 1) as isize))
            as i16)
            & (0xf00i16);
        if v1 == 1280 {
            let v5: i32 = j as i32 >> 1;
            advance_plm_instruction(v5 as usize, 3, game);
            let v6: i32 = word!(game.plm_block_indices[v5]) as i32 >> 1;
            let v7: u16 = (word!(game.level_data[v6]) & 0xf000 | 0x58);
            word! {game; plm_variable[v5] = v7; level_data[v6] = (v7) & (0x8fffu16); }
        } else if v1 == 768 {
            let v2: i32 = j as i32 >> 1;
            let v3: i32 = word!(game.plm_block_indices[v2]) as i32 >> 1;
            let v4: u16 = (word!(game.level_data[v3]) & 0xf000 | 0x58);
            word! {game; plm_variable[v2] = v4; level_data[v3] = (v4) & (0x8fffu16); }
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8703847] PlmSetup_D08C_SuperMissileBlockRespawning [] {
        let mut v1: i16 = ((*(ram_ptr!(game, u16; 0xc18))
            .offset((word!(game.projectile_index) >> 1) as isize))
            as i16)
            & (0xf00i16);
        if v1 == 1280 {
            word!(game.plm_instr_list_ptrs[(j >> 1)] = addr_kPlmInstrList_C922 as u16);
        } else if v1 == 512 {
            let v2: i32 = j as i32 >> 1;
            let v3: i32 = word!(game.plm_block_indices[v2]) as i32 >> 1;
            let v4: u16 = (word!(game.level_data[v3]) & 0xf000 | 0x9f);
            word! {game; plm_variable[v2] = v4; level_data[v3] = (v4) & (0x8fffu16); }
        } else {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8703904] PlmSetup_D08C_CrumbleBlock [] {
        if (*(ram_ptr!(game, u16; 0xc18)).offset((word!(game.projectile_index) >> 1) as isize)
            & 0xf00)
            != 1280
        {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8703957] PlmSetup_D0E8_GiveSamusDamage [] {
        word!(game.samus_periodic_damage, wrapping_add(1));
        0x41
    };
    [8704264] PlmSetup_D113_LowerNorfairChozoRoomPlug [] {
        let a: u16 = 0;
        *(ram_ptr!(game, u16; 0x10002))
            .offset((word!(game.plm_block_indices[(j >> 1)]) >> 1) as isize) = (a) & (0xfffu16);
        0
    };
    [8704279] PlmSetup_D127 [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0xc000, game);
        0
    };
    [8704299] PlmSetup_D138 [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0xe000, game);
        0
    };
    [8705542] PlmSetup_DeletePlmIfAreaTorizoDead [] {
        if CheckBossBitForCurArea(4, game) & 1 != 0 {
            word!(game.plm_header_ptr[(j >> 1)] = 0);
        }
        0
    };
    [8705558] PlmSetup_MakeBllockChozoHandTrigger [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0xb080, game);
        0
    };
    [8705663] PlmSetup_D700_MakePlmAirBlock_Unused [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0x44, game);
        0
    };
    [8705673] PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0x8044, game);
        0
    };
    [8706700] PlmSetup_EyeDoorEye [] {
        let idx: i32 = PrepareBitAccess(word!(game.plm_room_arguments[(j >> 1)]), game) as i32;
        if (word!(game.bitmask) & *ram_ptr!(game; 0xd8b0, idx as isize) as u16) == 0 {
            let v1: i32 = j as i32 >> 1;
            WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[v1]), 0xc044, game);
            WriteLevelDataBlockTypeAndBts(
                (2 * word!(game.room_width_in_blocks) as i32
                    + word!(game.plm_block_indices[v1]) as i32) as u16,
                0xd0ff,
                game,
            );
        }
        0
    };
    [8706745] PlmSetup_EyeDoor [] {
        let idx: i32 = PrepareBitAccess(word!(game.plm_room_arguments[(j >> 1)]), game) as i32;
        if (word!(game.bitmask) & *ram_ptr!(game; 0xd8b0, idx as isize) as u16) == 0 {
            WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0xa000, game);
        }
        0
    };
    [8706846] PlmSetup_SetMetroidRequiredClearState [] {
        word!(
            game.plm_pre_instrs[(j >> 1)] = crate::rtl::rom_table_word!(
                game,
                0x84db28,
                (word!(game.plm_room_arguments[(j >> 1)]) >> 1) as isize,
            )
        );
        0
    };
    [8711780] sub_84EE64 [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0x45, game);
        word!(game.global_number_of_items_loaded_ctr, wrapping_add(1));
        0
    };
    @functions [pub fn (j: u16, game: &mut Game) -> u8];
    [8696723] PlmSetup_B6FF_ScrollBlockTouch [] {
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let block_offset = 0x1c87 + usize::from(j & !1);
        let block_index = read_word(game, block_offset);
        write_word(game, block_offset, 0);
        // Search after clearing our block index, including our own slot as before.
        let Some(slot) = (0..40)
            .rev()
            .find(|&slot| read_word(game, 0x1c87 + slot * 2) == block_index)
        else {
            // The original handler hangs when no matching scroll PLM exists.
            loop {}
        };
        let index = slot * 2;
        if read_word(game, 0x1e17 + index) & 0x8000 == 0 {
            write_word(game, 0x1e17 + index, 0x8000);
            // Retain both separate instruction writes before setting its timer.
            advance_plm_instruction(slot, 1, game);
            advance_plm_instruction(slot, 1, game);
            write_word(game, 0xde1c + index, 1);
        }
        0
    };
    [8696769] PlmSetup_DeactivatePlm [] {
        retain_plm_block_bits(j, 0x8fff, game);
        0
    };
    [8696788] PlmSetup_D094_EnemyBreakableBlock [] {
        // Turn the spike into air, retaining its tile and flip bits.
        retain_plm_block_bits(j, 0x0fff, game);
        0
    };
    [8701873] PlmSetup_Door_Colored [] {
        let index = usize::from(j >> 1) * 2;
        let block_index =
            u16::from_le_bytes([game.g_ram[0x1c87 + index], game.g_ram[0x1c88 + index]]);
        WriteLevelDataBlockTypeAndBts(block_index, 0xc044, game);
        0
    };
    [8703790] PlmSetup_RespawningPowerBombBlock [] {
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let projectile_index = usize::from(read_word(game, 0xdde) >> 1) * 2;
        let projectile_type = read_word(game, 0xc18 + projectile_index) & 0x0f00;
        let index = usize::from(j >> 1) * 2;
        match projectile_type {
            0x0500 => write_word(game, 0x1d27 + index, addr_kPlmInstrList_C91C as u16),
            0x0300 => {
                let block_index = usize::from(read_word(game, 0x1c87 + index) >> 1) * 2;
                let block = read_word(game, 0x10002 + block_index);
                let revealed = (block & 0xf000) | 0x0057;
                write_word(game, 0x1e17 + index, revealed);
                // Keep the original reveal mask and variable-before-level write order.
                write_word(game, 0x10002 + block_index, revealed & 0x8fff);
            }
            _ => write_word(game, 0x1c37 + index, 0),
        }
        0
    };
    @functions [pub fn (_j: u16, game: &mut Game) -> u8];
    [8696811] PlmSetup_B70F_IcePhysics [] {
        let y_radius = u16::from_le_bytes([game.g_ram[0xb00], game.g_ram[0xb01]]);
        let y_pos = u16::from_le_bytes([game.g_ram[0xafa], game.g_ram[0xafb]]);
        let bottom = y_pos.wrapping_add(y_radius).wrapping_sub(1);
        if bottom & 0x0f == 7 || bottom & 0x0f == 15 {
            game.g_ram[0xb4c..0xb4e].copy_from_slice(&16u16.to_le_bytes());
        }
        0
    };
    [8696840] PlmSetup_QuicksandSurface [] {
        // Clear momentum, boost and extra running speed in the original order.
        for offset in [0xb3c, 0xb3e, 0xb40, 0xb44, 0xb42] {
            game.g_ram[offset..offset + 2].fill(0);
        }
        game.g_ram[0xb49] &= 0x7f;
        game.g_ram[0xb46..0xb48].fill(0);
        let equipped_items = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
        let physics_index = if equipped_items & 0x20 != 0 { 2 } else { 0 };
        let reaction_point = u16::from_le_bytes([game.g_ram[0x1e73], game.g_ram[0x1e74]]);
        if reaction_point == 0 {
            let y_direction = u16::from_le_bytes([game.g_ram[0xb36], game.g_ram[0xb37]]);
            match y_direction & 3 {
                0 | 3 => PlmSetup_QuicksandSurface_0(physics_index, game),
                1 => PlmSetup_QuicksandSurface_1(physics_index, game),
                2 => PlmSetup_QuicksandSurface_2(physics_index, game),
                _ => unreachable!(),
            }
        }
        0
    };
    [8696983] PlmSetup_B71F_SubmergingQuicksand [] {
        game.g_ram[0xaf4..0xaf6].fill(0);
        game.g_ram[0xb5a..0xb5c].copy_from_slice(&0x2000u16.to_le_bytes());
        game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
        0
    };
    [8697000] PlmSetup_B723_SandfallsSlow [] {
        game.g_ram[0xb5a..0xb5c].copy_from_slice(&0x4000u16.to_le_bytes());
        game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
        0
    };
    [8697014] PlmSetup_B727_SandFallsFast [] {
        game.g_ram[0xb5a..0xb5c].copy_from_slice(&0xc000u16.to_le_bytes());
        game.g_ram[0xb5c..0xb5e].copy_from_slice(&1u16.to_le_bytes());
        0
    };
    [8697153] PlmSetup_B737_SubmergingQuicksand [] {
        game.g_ram[0xb2c..0xb2e].fill(0);
        game.g_ram[0xb2e..0xb30].fill(0);
        game.g_ram[0xb32..0xb34].fill(0);
        game.g_ram[0xb34..0xb36].fill(0);
        0
    };
    @functions [pub unsafe fn (j: u16, game: &mut Game) -> u8];
    @prefix { let v1: i32 = j as i32 >> 1; };
    [8696203] PlmSetup_B6D3_MapStation [] {
        let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
        word!(game.level_data[v2] = (word!(game.level_data[v2]) & 0xfff | 0x8000));
        if *ram_ptr!(game; 0xd908, word!(game.area_index) as isize) != 0 {
            word!(game.plm_instr_list_ptrs[v1] = addr_word_84AD76 as u16);
        } else {
            WriteLevelDataBlockTypeAndBts(
                (word!(game.plm_block_indices[v1])).wrapping_add(2),
                0xb047,
                game,
            );
            WriteLevelDataBlockTypeAndBts(
                (word!(game.plm_block_indices[v1])).wrapping_sub(4),
                0xb048,
                game,
            );
        }
        0
    };
    [8696349] PlmSetup_PlmB6DF_EnergyStation [] {
        let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
        word!(game.level_data[v2] = (word!(game.level_data[v2]) & 0xfff | 0x8000));
        WriteLevelDataBlockTypeAndBts(
            (word!(game.plm_block_indices[v1])).wrapping_add(2),
            0xb049,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (word!(game.plm_block_indices[v1])).wrapping_sub(2),
            ((0x84b04ai32) as u16),
            game,
        );
        0
    };
    [8696389] PlmSetup_PlmB6EB_EnergyStation [] {
        let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
        word!(game.level_data[v2] = (word!(game.level_data[v2]) & 0xfff | 0x8000));
        WriteLevelDataBlockTypeAndBts(
            (word!(game.plm_block_indices[v1])).wrapping_add(2),
            0xb04b,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (word!(game.plm_block_indices[v1])).wrapping_sub(2),
            0xb04c,
            game,
        );
        0
    };
    [8701664] PlmSetup_C836_DownwardsGateShootblock [] {
        word!(
            game.plm_instr_list_ptrs[v1] = crate::rtl::rom_table_word!(
                game,
                0x84c70a,
                (word!(game.plm_room_arguments[v1]) >> 1) as isize,
            )
        );
        install_optional_gate_block!(game, 0x84c71a, v1, wrapping_sub);
        install_optional_gate_block!(game, 0x84c72a, v1, wrapping_add);
        0
    };
    [8701754] PlmSetup_C73A_UpwardsGateShootblock [] {
        word!(
            game.plm_instr_list_ptrs[v1] = crate::rtl::rom_table_word!(
                game,
                0x84c764,
                (word!(game.plm_room_arguments[v1]) >> 1) as isize,
            )
        );
        install_optional_gate_block!(game, 0x84c774, v1, wrapping_sub);
        install_optional_gate_block!(game, 0x84c784, v1, wrapping_add);
        0
    };
    [8701844] PlmSetup_C794_GreyDoor [] {
        word!(
            game.plm_variable[v1] = ((((*((ram_ptr!(game, u16; 0x1dc7)).offset(v1 as isize) as *mut u8)
                .offset(1,)) as u16)
                & (0x7cu16))
                >> 1)
        );
        word!(game.plm_room_arguments[v1], |fresh6| (fresh6 & !(0x7c00)));
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[v1]), 0xc044, game);
        0
    };
    [8701922] PlmSetup_Door_Strange [] {
        let v2: u16 = word!(game.plm_block_indices[v1]);
        word!(game.plm_block_indices[v1] = 0);
        let mut v3: u16 = 78;
        while v2 != word!(game.plm_block_indices[(v3 >> 1)]) {
            v3 = (v3).wrapping_sub(2);
            if (v3 & 0x8000) != 0 {
                return 0;
            }
        }
        word!(
            game.plm_timers[(v3 >> 1)] = (*(ram_ptr!(game, u16; 0xc18))
                .offset((word!(game.projectile_index) as i16 >> 1) as isize,)
                & 0x1fff
                | 0x8000)
        );
        0
    };
    [8703595] PlmSetup_RespawningShotBlock [] {
        let v2: i32 = word!(game.plm_block_indices[v1]) as i32 >> 1;
        let v3: u16 = (word!(game.level_data[v2]) & 0xf000 | 0x52);
        word! {game; plm_variable[v1] = v3; level_data[v2] = (v3) & (0x8fffu16); }
        0
    };
    [8703925] PlmSetup_D0DC_BreakableGrappleBlock [] {
        let v2: u16 = word!(game.plm_block_indices[v1]);
        word!(game.plm_variable[v1] = word!(game.level_data[(v2 >> 1)]));
        *(ram_ptr!(game, u16; 0x16402, (v2 >> 1) as isize)) =
            ((*ram_ptr!(game; 0x16402, ((v2 as i32 >> 1) + 1) as isize) as i32) << 8) as u16;
        0x41
    };
    [8705526] PlmSetup_MotherBrainGlass [] {
        word!(game.plm_room_arguments[v1] = 0);
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[v1]), 0x8044, game);
        0
    };
    [8705683] PlmSetup_D708_LowerNorfairChozoBlockUnused [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[v1]), 0x8044, game);
        WriteLevelDataBlockTypeAndBts(
            (word!(game.plm_block_indices[v1])).wrapping_add(2),
            0x50ff,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (word!(game.room_width_in_blocks) as i32
                + word!(game.room_width_in_blocks) as i32
                + word!(game.plm_block_indices[v1]) as i32) as u16,
            0xd0ff,
            game,
        );
        WriteLevelDataBlockTypeAndBts(
            (word!(game.room_width_in_blocks) as i32
                + word!(game.room_width_in_blocks) as i32
                + word!(game.plm_block_indices[v1]) as i32
                + 2) as u16,
            0xd0ff,
            game,
        );
        0
    };
    [8707732] PlmSetup_DraygonCannonFacingRight [] {
        word!(game.plm_variable[v1] = word!(game.plm_room_arguments[v1]));
        word!(game.plm_room_arguments[v1] = 0);
        let v2: u16 = word!(game.plm_block_indices[v1]);
        WriteLevelDataBlockTypeAndBts(v2, 0xc044, game);
        WriteLevelDataBlockTypeAndBts(
            (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16,
            0xd0ff,
            game,
        );
        0
    };
    [8707769] PlmSetup_DraygonCannonFacingDiagonalRight [] {
        word!(game.plm_variable[v1] = word!(game.plm_room_arguments[v1]));
        word!(game.plm_room_arguments[v1] = 0);
        let v2: u16 = word!(game.plm_block_indices[v1]);
        WriteLevelDataBlockTypeAndBts(v2, 0xc044, game);
        WriteLevelDataBlockTypeAndBts((v2).wrapping_add(2), 0x50ff, game);
        let v3: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v3, 0xd0ff, game);
        WriteLevelDataBlockTypeAndBts((v3).wrapping_add(2), 0xd0ff, game);
        0
    };
    [8707861] PlmSetup_DraygonCannonFacingDiagonalLeft [] {
        word!(game.plm_variable[v1] = word!(game.plm_room_arguments[v1]));
        word!(game.plm_room_arguments[v1] = 0);
        let v2: u16 = word!(game.plm_block_indices[v1]);
        WriteLevelDataBlockTypeAndBts(v2, 0xc044, game);
        WriteLevelDataBlockTypeAndBts((v2).wrapping_sub(2), 0x5001, game);
        let v3: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v3, 0xd0ff, game);
        WriteLevelDataBlockTypeAndBts((v3).wrapping_sub(2), 0xd0ff, game);
        0
    };
    [8707916] PlmSetup_DraygonCannon [] {
        word!(game.plm_variable[v1] = word!(game.plm_room_arguments[v1]));
        word!(game.plm_room_arguments[v1] = 3);
        0
    };
    [8711822] sub_84EE8E [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[v1]), 0xc045, game);
        word!(
            game.plm_variable[v1] = *(ram_ptr!(game, u16; 0x10002))
                .offset((word!(game.plm_block_indices[v1]) >> 1) as isize,)
        );
        word!(game.global_number_of_items_loaded_ctr, wrapping_add(1));
        0
    };
    @functions [pub unsafe fn (v0: u16, game: &mut Game) -> u8];
    [8711851] sub_84EEAB [] {
        if word!(game.time_is_frozen_flag) != 0 {
            word!(game.plm_header_ptr[(v0 >> 1)] = 0);
        } else {
            let v1: i32 = v0 as i32 >> 1;
            let v2: u16 = word!(game.plm_block_indices[v1]);
            word!(game.plm_block_indices[v1] = 0);
            let mut i: i32 = 78;
            while i >= 0 {
                if v2 == word!(game.plm_block_indices[(i >> 1)]) {
                    break;
                }
                i -= 2;
            }
            word!(game.plm_timers[(i >> 1)] = 255);
        }
        0
    };
    @forward [pub fn (j: u16, game: &mut Game) -> u8];
    [8696634] PlmSetup_B638_Rightwards_Extension => setup_extension_block(j, 0x50ff, game);
    [8696645] PlmSetup_B63F_Leftwards_Extension => setup_extension_block(j, 0x5001, game);
    [8696656] PlmSetup_B643_Downwards_Extension => setup_extension_block(j, 0xd0ff, game);
    [8696667] PlmSetup_B647_Upwards_Extension => setup_extension_block(j, 0xd001, game);
    @forward [pub fn (j: u16, game: &mut Game) -> u8];
    [8696689] PlmSetup_B703_ScrollPLM => setup_scroll_block(j, 0x3046, game);
    [8696706] PlmSetup_B707_SolidScrollPLM => setup_scroll_block(j, 0xb046, game);
    @forward [pub unsafe fn (j: u16, game: &mut Game) -> u8];
    [8711757] sub_84EE4D => sub_84EE5F(j, 8, game);
    [8711762] sub_84EE52 => sub_84EE5F(j, 0xa, game);
    [8711767] sub_84EE57 => sub_84EE5F(j, 0xc, game);
    [8711772] sub_84EE5C => sub_84EE5F(j, 0xe, game);
    @forward [pub unsafe fn (j: u16, game: &mut Game) -> u8];
    [8711799] sub_84EE77 => sub_84EE89(j, 8, game);
    [8711804] sub_84EE7C => sub_84EE89(j, 0xa, game);
    [8711809] sub_84EE81 => sub_84EE89(j, 0xc, game);
    [8711814] sub_84EE86 => sub_84EE89(j, 0xe, game);
    @external {
        8694567 | 8696783 | 8698095 | 8698618 | 8705782 | 8705783 | 8704398 => [0];
        8696784 => [PlmSetup_ReturnCarryClear(j)]; 8696786 => [PlmSetup_ReturnCarrySet(j)];
        8696803 => [UNUSED_sub_84B3E3(j)];
        8697028 => [PlmSetup_QuicksandSurfaceB(j, displacement, game)];
        8697167 => [PlmSetup_B73B_B73F_SandFalls(j)];
        8698220 => [PlmSetup_B974(j)];
        8701630 => [PlmSetup_C82A_DownwardsClosedGate(j, setup_a, game)];
        8703949 => [PlmSetup_D0D8_SetVFlag(j)];
        8703953 => [PlmSetup_D0D8_ClearVflag(j)];
        8704399 => [PlmSetup_D6DA_LowerNorfairChozoHandTrigger(j, displacement, game)];
        8705568 => [PlmSetup_D6F2_WreckedShipChozoHandTrigger(j, displacement, game)];
        8705740 => PlmSetup_D70C_NoobTube;
        8707824 => PlmSetup_DraygonCannonFacingLeft;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPlmPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [8697839] PlmPreInstr_WakeAndLavaIfBoosterCollected [] {
        if (word!(game.collected_items) & 0x2000) != 0 {
            if (word!(game.fx_target_y_pos) & 0x8000) != 0 {
                word!(game.plm_header_ptr[(k >> 1)] = 0);
            } else {
                word!(game.fx_y_vel = -128i32 as u16);
                let v1: i32 = k as i32 >> 1;
                word!(game.plm_instruction_timer[v1] = 1);
                word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
                word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
                word!(game.plm_timers[v1] = 0);
            }
        } else {
            word!(game.fx_target_y_pos = -1i32 as u16);
            word! { game, 0;
                fx_y_vel, fx_timer, earthquake_timer, plm_header_ptr[(k as i32 >> 1)],
            }
        };
    };
    [8697898] PlmPreInstr_WakePLMAndStartFxMotionSamusFarLeft [] {
        if word!(game.samus_x_pos) <= 0xae0 {
            word!(game.fx_timer = 1);
            let v1: i32 = k as i32 >> 1;
            word!(game.plm_instruction_timer[v1] = 1);
            word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            word!(game.plm_timers[v1] = 0);
        }
    };
    [8697926] PlmPreInstr_AdvanceLavaSamusMovesLeft [] {
        static g_word_84B876: [u16; 10] = [
            0x72b, 0x1bf, 0xff50, 0x50a, 0x167, 0xff20, 0x244, 0x100, 0xff20, 0x8000,
        ];
        let v1: i32 = k as i32 >> 1;
        let v2: u16 = word!(game.plm_timers[v1]);
        let v3: i32 = v2 as i32 >> 1;
        let v4: u16 = g_word_84B876[v3 as usize];
        if (v4 & 0x8000) != 0 {
            SetEventHappened(0x15, game);
        } else if v4 >= word!(game.samus_x_pos) {
            if g_word_84B876[(v3 + 1) as usize] < word!(game.fx_base_y_pos) {
                word!(game.fx_base_y_pos = g_word_84B876[(v3 + 1) as usize]);
            }
            word!(game.fx_y_vel = g_word_84B876[(v3 + 2) as usize]);
            word!(game.plm_timers[v1] = (v2).wrapping_add(6));
        }
    };
    [8698032] PlmPreInstr_ShaktoolsRoom [] {
        if word!(game.power_bomb_explosion_status) != 0 {
            *(ram_ptr!(game, u16; 0xcd20)) = 257;
            *(ram_ptr!(game, u16; 0xcd20, 2)) = 257;
        }
        if word!(game.samus_x_pos) > 0x348 {
            SetEventHappened(0xd, game);
            word!(game.plm_header_ptr[(k >> 1)] = 0);
        }
    };
    [8698151] PlmPreInstr_OldTourianEscapeShaftEscape [] {
        if WakePlmIfSamusIsBelowAndRightOfTarget(k, 240, 2080, game) == 0 {
            SpawnEprojWithRoomGfx(0xb4b1, 0, game);
        }
    };
    [8698184] PlmPreInstr_EscapeRoomBeforeOldTourianEscapeShaft [] {
        if WakePlmIfSamusIsBelowAndRightOfTarget(k, 240, 1344, game) == 0 {
            word! {game; fx_y_vel = -104i32 as u16; fx_timer = 16; }
        }
    };
    [8698731] PlmPreInstr_WakePlmIfTriggeredOrSamusBelowPlm [] {
        CalculatePlmBlockCoords(k, game);
        if word!(game.samus_x_pos) as i32 >> 4 == word!(game.plm_x_block) as i32
            && (((word!(game.samus_y_pos) as i32 >> 4) - word!(game.plm_y_block) as i32) as u16) < 5
            || word!(game.plm_timers[(k >> 1)]) != 0
        {
            let v1: i32 = k as i32 >> 1;
            advance_plm_instruction(v1 as usize, 1, game);
            advance_plm_instruction(v1 as usize, 1, game);
            word!(game.plm_instruction_timer[v1] = 1);
            word!(game.plm_pre_instrs[v1] = ((0x84bba3i32) as u16));
        }
    };
    [8698788] PlmPreInstr_WakePlmIfTriggeredOrSamusAbovePlm [] {
        CalculatePlmBlockCoords(k, game);
        if word!(game.samus_x_pos) as i32 >> 4 == word!(game.plm_x_block) as i32
            && (((word!(game.samus_y_pos) as i32 >> 4) - word!(game.plm_y_block) as i32) as u16)
                >= 0xfffc
            || word!(game.plm_timers[(k >> 1)]) != 0
        {
            let v1: i32 = k as i32 >> 1;
            word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            word!(game.plm_instruction_timer[v1] = 1);
            word!(game.plm_pre_instrs[v1] = addr_locret_84BBDC as u16);
        }
    };
    [8699216] PlmPreInstr_GoToLinkInstrIfShotWithAnyMissile [] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let shot = read_word(game, 0x1d77 + index); // plm_timers
        if shot != 0 {
            match shot & 0x0f00 {
                0x0200 => write_word(game, 0xdf0c + index, 119), // plm_variables
                0x0100 => {}
                _ => {
                    QueueSfx2_Max6(0x57, game);
                    write_word(game, 0x1d77 + index, 0);
                    return;
                }
            }
            write_word(game, 0x1d77 + index, 0);
            let link = read_word(game, 0xdebc + index); // plm_instruction_list_link_reg
            write_word(game, 0x1d27 + index, link); // plm_instr_list_ptrs
            write_word(game, 0xde1c + index, 1); // plm_instruction_timer
            return;
        }
        write_word(game, 0x1d77 + index, 0);
    };
    [8699348] PlmPreInstr_GotoLinkIfBoss1Dead [] {
        if CheckBossBitForCurArea(1, game) & 1 != 0 {
            PlmPreInstr_GoToLinkInstruction(k, game);
        } else {
            PlmPreInstr_PlayDudSound(k, game);
        };
    };
    [8699363] PlmPreInstr_GotoLinkIfMiniBossDead [] {
        if CheckBossBitForCurArea(2, game) & 1 != 0 {
            PlmPreInstr_GoToLinkInstruction(k, game);
        } else {
            PlmPreInstr_PlayDudSound(k, game);
        };
    };
    [8699378] PlmPreInstr_GotoLinkIfTorizoDead [] {
        if CheckBossBitForCurArea(4, game) & 1 != 0 {
            PlmPreInstr_GoToLinkInstruction(k, game);
        } else {
            PlmPreInstr_PlayDudSound(k, game);
        };
    };
    [8699393] PlmPreInstr_GotoLinkIfEnemyDeathQuotaOk [] {
        if word!(game.num_enemies_killed_in_room) < word!(game.num_enemy_deaths_left_to_clear) {
            PlmPreInstr_PlayDudSound(k, game);
        } else {
            SetEventHappened(0, game);
            PlmPreInstr_GoToLinkInstruction(k, game);
        };
    };
    [8699423] PlmPreInstr_GotoLinkIfTourianStatueFinishedProcessing [] {
        if (word!(game.tourian_entrance_statue_finished) & 0x8000) == 0 {
            PlmPreInstr_PlayDudSound(k, game);
        } else {
            PlmPreInstr_GoToLinkInstruction(k, game);
        };
    };
    [8699440] PlmPreInstr_GotoLinkIfCrittersEscaped [] {
        if CheckEventHappened(0xf, game) & 1 != 0 {
            PlmPreInstr_GoToLinkInstruction(k, game);
        } else {
            PlmPreInstr_PlayDudSound(k, game);
        };
    };
    [8704348] PlmPreInstr_DeletePlmAndSpawnTriggerIfBlockDestroyed [] {
        let prod: u16 = (8u16).wrapping_mul((word!(game.room_width_in_blocks) as u8) as u16);
        let v1: u16 = (2 * (prod as i32 + 4)) as u16;
        if word!(game.level_data[(v1 >> 1)]) == 255 {
            WriteLevelDataBlockTypeAndBts(v1, 0xb083, game);
            *(ram_ptr!(game, u16; 0x1c37)).offset((word!(game.plm_id) >> 1) as isize) = 0;
        }
    };
    [8704827] PlmPreInstr_WakePlmIfSamusHasBombs [] {
        if (word!(game.collected_items) & 0x1000) != 0 {
            let v1: i32 = k as i32 >> 1;
            word!(game.plm_instruction_timer[v1] = 1);
            advance_plm_instruction(v1 as usize, 2, game);
            word!(game.plm_pre_instrs[v1] = ((0x84d356i32) as u16));
        }
    };
    [8705215] PlmPreInstr_WakeOnKeyPress [] {
        if (word!(game.joypad1_newkeys)
            & (kButton_B as i32
                | kButton_Y as i32
                | kButton_Left as i32
                | kButton_Right as i32
                | kButton_A as i32
                | kButton_X as i32) as u16)
            != 0
        {
            let v2: i32 = k as i32 >> 1;
            word!(game.plm_instruction_timer[v2] = 1);
            word!(game.plm_instr_list_ptrs[v2], wrapping_add(1));
            word!(game.plm_instr_list_ptrs[v2], wrapping_add(1));
        }
    };
    [8706782] PlmPreInstr_SetMetroidsClearState_Ev0x10 [] {
        if word!(game.num_enemies_killed_in_room) >= word!(game.num_enemy_deaths_left_to_clear) {
            SetEventHappened(0x10, game);
        }
    };
    [8706798] PlmPreInstr_SetMetroidsClearState_Ev0x11 [] {
        if word!(game.num_enemies_killed_in_room) >= word!(game.num_enemy_deaths_left_to_clear) {
            SetEventHappened(0x11, game);
        }
    };
    [8706814] PlmPreInstr_SetMetroidsClearState_Ev0x12 [] {
        if word!(game.num_enemies_killed_in_room) >= word!(game.num_enemy_deaths_left_to_clear) {
            SetEventHappened(0x12, game);
        }
    };
    [8706830] PlmPreInstr_SetMetroidsClearState_Ev0x13 [] {
        if word!(game.num_enemies_killed_in_room) >= word!(game.num_enemy_deaths_left_to_clear) {
            SetEventHappened(0x13, game);
        }
    };
    [8706916] PlmPreInstr_GotoLinkIfShotWithSuperMissile [] {
        let mut v1: i16 = ((word!(game.plm_timers[(k >> 1)])) as i16) & (0xf00i16);
        if v1 == 512 {
            word!(game.plm_room_arguments[(k >> 1)] = 119);
        } else if v1 != 256 {
            return;
        }
        let v2: i32 = k as i32 >> 1;
        word!(game.plm_timers[v2] = 0);
        word!(game.plm_instr_list_ptrs[v2] = word!(game.plm_instruction_list_link_reg[v2]));
        word!(game.plm_instruction_timer[v2] = 1);
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let v2: i32 = k as i32 >> 1; };
    [8697821] PlmPreInstr_B7EB_DecTimerEnableSoundsDeletePlm [] {
        crate::update_place! { reference fresh84 [fresh85] = *(ram_ptr!(game, u16; 0x1d77)).offset(v2 as isize) => (*fresh84).wrapping_sub(1); }
        if fresh85 == 1 {
            word! { game, 0; debug_disable_sounds, plm_header_ptr[v2] }
        }
    };
    [8699151] PlmPreInstr_GoToLinkInstrIfShot [] {
        if word!(game.plm_timers[v2]) != 0 {
            word!(game.plm_timers[v2] = 0);
            word!(game.plm_instr_list_ptrs[v2] = word!(game.plm_instruction_list_link_reg[v2]));
            word!(game.plm_instruction_timer[v2] = 1);
        }
    };
    [8699174] PlmPreInstr_GoToLinkInstrIfShotWithPowerBomb [] {
        let v3: u16 = word!(game.plm_timers[v2]);
        if v3 != 0 {
            if (v3 & 0xf00) == 768 {
                word!(game.plm_timers[v2] = 0);
                word!(game.plm_instr_list_ptrs[v2] = word!(game.plm_instruction_list_link_reg[v2]));
                word!(game.plm_instruction_timer[v2] = 1);
                return;
            }
            QueueSfx2_Max6(0x57, game);
        }
        word!(game.plm_timers[v2] = 0);
    };
    [8699272] PlmPreInstr_GoToLinkInstrIfShotWithSuperMissile [] {
        let v3: u16 = word!(game.plm_timers[v2]);
        if v3 != 0 {
            if (v3 & 0xf00) == 512 {
                word!(game.plm_timers[v2] = 0);
                word!(game.plm_instr_list_ptrs[v2] = word!(game.plm_instruction_list_link_reg[v2]));
                word!(game.plm_instruction_timer[v2] = 1);
                return;
            }
            QueueSfx2_Max6(0x57, game);
        }
        word!(game.plm_timers[v2] = 0);
    };
    [8699314] PlmPreInstr_GoToLinkInstruction [] {
        word!(game.plm_timers[v2] = 0);
        word!(game.plm_instr_list_ptrs[v2] = word!(game.plm_instruction_list_link_reg[v2]));
        word!(game.plm_instruction_timer[v2] = 1);
    };
    [8699420] PlmPreInstr_PlayDudSound [] {
        if word!(game.plm_timers[v2]) != 0 {
            QueueSfx2_Max6(0x57, game);
        }
        word!(game.plm_timers[v2] = 0);
    };
    [8705875] PlmPreInstr_WakePlmIfRoomArgumentDoorIsSet [] {
        let idx: i32 = PrepareBitAccess(word!(game.plm_room_arguments[v2]), game) as i32;
        if (word!(game.bitmask) & *ram_ptr!(game; 0xd8b0, idx as isize) as u16) != 0 {
            let v3: i32 = k as i32 >> 1;
            word!(game.plm_pre_instrs[v3] = addr_locret_84D779 as u16);
            word!(game.plm_instr_list_ptrs[v3] = word!(game.plm_instruction_list_link_reg[v3]));
            word!(game.plm_instruction_timer[v3] = 1);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let v1: i32 = k as i32 >> 1; };
    [8694921] PlmPreInstr_PositionSamusAndInvincible [] {
        word!(game.samus_x_pos = word!(game.plm_variable[v1]));
        word!(game.samus_y_pos = word!(game.plm_variables[v1]));
        word!(game.samus_invincibility_timer, |fresh69| (fresh69)
            | (0x10u16));
    };
    [8698706] PlmPreInstr_WakePlmIfTriggered [] {
        if word!(game.plm_timers[v1]) != 0 {
            word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            word!(game.plm_instr_list_ptrs[v1], wrapping_add(1));
            word!(game.plm_instruction_timer[v1] = 1);
            word!(game.plm_pre_instrs[v1] = addr_locret_84BB6A as u16);
        }
    };
    [8704486] PlmPreInstr_IncrementRoomArgIfShotBySuperMissile [] {
        let v2: u16 = word!(game.plm_timers[v1]);
        if (v2 & 0xf00) == 512 || (v2 & 0xf00) == 256 {
            word!(game.plm_room_arguments[v1], wrapping_add(1));
        }
        word!(game.plm_timers[v1] = 0);
    };
    [8707977] PlmPreInstr_GotoLinkIfTriggered [] {
        if byte!(@pointer (ram_ptr!(game, u16; 0x1d77)).offset(v1 as isize); 0) == 255 {
            word!(game.plm_pre_instrs[v1] = ((0x84dfa8i32) as u16));
            word!(game.plm_instr_list_ptrs[v1] = word!(game.plm_instruction_list_link_reg[v1]));
            word!(game.plm_instruction_timer[v1] = 1);
        }
    };
    [8708070] PlmPreInstr_WakeIfTriggered [] {
        let v2: u16 = word!(game.plm_timers[v1]);
        if v2 != 768 && (v2 as u8) == 255 {
            word!(game.plm_timers[v1] = 0);
            advance_plm_instruction(v1 as usize, 1, game);
            advance_plm_instruction(v1 as usize, 1, game);
            word!(game.plm_instruction_timer[v1] = 1);
        }
    };
    @external {
        8684649 => [()]; 8708008 => [()]; 8685264 => [()]; 8684774 => [()]; 8684861 => [()];
        8686246 => [()]; 8698787 => [()]; 8704854 => [()]; 8698618 => [()];
        8696028 => [{
                    PlmSetup_BTS_Brinstar_0x80_Floorplant(k, game);
                }];
        8696083 => [{
                    PlmSetup_BTS_Brinstar_0x81_Ceilingplant(k, game);
                }];
        8698730 => [()];
        8706773 | 8706774 | 8706775 | 8706776 | 8706777 | 8706778 | 8706779 | 8706780 | 8706781
                | 8686304 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPlmInstr(ea: u32, j: *const u8, k: u16, game: &mut Game) -> *const u8 => ea;
    call (j, k, game); default {
            Unreachable(game);
            null::<u8>()
        };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8685236] PlmInstr_Sleep [] {
        let instruction = (plmp.offset_from(RomBankBase(0x84, game)) as u16).wrapping_sub(2);
        let offset = 0x1d27 + usize::from(k >> 1) * 2; // plm_instruction_list_ptrs
        game.g_ram[offset..offset + 2].copy_from_slice(&instruction.to_le_bytes());
        null()
    };
    [8685249] PlmInstr_PreInstr [] {
        let pre_instruction = (plmp as *const u16).read_unaligned();
        let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
        game.g_ram[offset..offset + 2].copy_from_slice(&pre_instruction.to_le_bytes());
        plmp.add(2)
    };
    [8685323] PlmInstr_CallFunction [] {
        CallPlmInstrFunc(Load24(plmp as *mut LongPtr), game);
        plmp.offset(3)
    };
    [8685390] PlmInstr_SetTimer [] {
        game.g_ram[0x1d77 + usize::from(k)] = *plmp; // Only one byte of plm_timers.
        plmp.add(1)
    };
    [8686194] PlmInstr_GotoIfDoorBitSet [] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
        if argument & 0x8000 != 0 {
            return plmp.add(2);
        }
        let bit_index = usize::from(PrepareBitAccess(argument, game));
        let mask = read_word(game, 0x5e7); // bitmask
        if mask & u16::from(game.g_ram[0xd8b0 + bit_index]) != 0 {
            return usize::from((plmp as *const u16).read_unaligned()) as *const u8;
        }
        plmp.add(2)
    };
    [8686913] PlmInstr_GotoIfSamusNear [] {
        CalculatePlmBlockCoords(k, game);
        let v2: u8 =
            abs16(((word!(game.samus_x_pos) as i32 >> 4) - word!(game.plm_x_block) as i32) as u16)
                as u8;
        let v4: u8 =
            abs16(((word!(game.samus_y_pos) as i32 >> 4) - word!(game.plm_y_block) as i32) as u16)
                as u8;
        if v2 <= *plmp && v4 <= *plmp.offset(1) {
            *(plmp.offset(2) as *mut u16) as usize as *const u8
        } else {
            plmp.offset(4)
        }
    };
    [8694528] PlmInstr_MovePlmDownOneBlock [] {
        word!(game.plm_block_indices[(k >> 1)], |fresh39| (fresh39 as i32
            + 2 * word!(game.room_width_in_blocks) as i32)
            as u16);
        plmp
    };
    [8694742] PlmInstr_ABD6 [] {
        word!(game.plm_block_indices[(k >> 1)], wrapping_add(2));
        plmp
    };
    [8695487] PlmInstr_GotoIfMissilesFull [] {
        if word!(game.samus_max_missiles) != word!(game.samus_missiles) {
            return plmp.offset(2);
        }
        CallSomeSamusCode(1, game);
        *(plmp as *mut u16) as usize as *const u8
    };
    [8698661] PlmInstr_MovePlmRight4Blocks [] {
        word!(game.plm_block_indices[(k >> 1)], wrapping_add(8));
        plmp
    };
    [8699455] PlmInstr_SetGreyDoorPreInstr [] {
        word!(
            game.plm_pre_instrs[(k >> 1)] = *(rom_ptr!(game, u16; 0x84be4b))
                .offset((word!(game.plm_variable[(k >> 1)]) >> 1) as isize,)
        );
        plmp
    };
    [8703379] PlmInstr_SetBtsTo1 [] {
        *ram_ptr!(game; 0x16402, (word!(game.plm_block_indices[(k >> 1)]) >> 1) as isize) = 1;
        plmp
    };
    [8704761] PlmInstr_GotoIfRoomArgLess [] {
        if word!(game.plm_room_arguments[(k >> 1)]) >= *(plmp as *mut u16) {
            plmp.offset(4)
        } else {
            *(plmp.offset(2) as *mut u16) as usize as *const u8
        }
    };
    [8705317] PlmInstr_EnableWaterPhysics [] {
        word!(game.fx_liquid_options, |fresh34| (fresh34 & !(4)));
        plmp
    };
    [8705334] PlmInstr_DiagonalEarthquake [] {
        word! {game; earthquake_type = 11; earthquake_timer = 64; }
        plmp
    };
    [8705951] PlmInstr_SpawnTwoEyeDoorSmoke [] {
        SpawnEprojWithRoomGfx(0xe517, 0x30a, game);
        SpawnEprojWithRoomGfx(0xe517, 0x30a, game);
        plmp
    };
    [8706958] PlmInstr_DamageDraygonTurret [] {
        SetPlmVarPtr(k, 1, game);
        let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
        WriteLevelDataBlockTypeAndBts(v2, 0xa003, game);
        WriteLevelDataBlockTypeAndBts(
            (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16,
            0xa003,
            game,
        );
        plmp
    };
    @functions [pub fn (_plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8685244] PlmInstr_Delete [] {
        let offset = 0x1c37 + usize::from(k >> 1) * 2; // plm_header_ptr
        game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
        null()
    };
    [8686138] PlmInstr_Return [] {
        let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
        let link = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
        usize::from(link) as *const u8
    };
    @functions [pub fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8685258] PlmInstr_ClearPreInstr [] {
        let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
        game.g_ram[offset..offset + 2].copy_from_slice(&0x86d0u16.to_le_bytes());
        plmp
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8685375] PlmInstr_DecrementAndBranchNonzero [] {
        crate::update_place! { reference fresh65 [fresh66] = *(ram_ptr!(game, u16; 0x1d77)).offset((k >> 1) as isize) => (*fresh65).wrapping_sub(1); }
        if fresh66 == 1 {
            plmp.offset(2)
        } else {
            PlmInstr_Goto(plmp, k)
        }
    };
    [8686075] PlmInstr_CollectAmmoPowerBombTank [] {
        let t: u16 = *(plmp as *mut u16);
        word!(game.samus_max_power_bombs, wrapping_add(t));
        word!(game.samus_power_bombs, wrapping_add(t));
        AddPowerBombsToHudTilemap(game);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(4, game);
        plmp.offset(2)
    };
    [8686557] PlmInstr_ClearMusicQueueAndQueueTrack [] {
        word_transfers!(game, descending, 8;
            le(music_queue_track, 7) <- value(0),
            le(music_queue_delay, 7) <- value(0),
        );
        word!(game.music_queue_read_pos = word!(game.music_queue_write_pos));
        word! { game, 0; music_timer, music_entry }
        QueueMusic_Delayed8(*plmp as u16, game);
        plmp.offset(1)
    };
    [8694617] PlmInstr_MovePlmDownOneBlock_0 [] {
        word!(game.plm_block_indices[(k >> 1)], |fresh38| (fresh38 as i32
            + 2 * word!(game.room_width_in_blocks) as i32)
            as u16);
        plmp
    };
    [8698479] PlmInstr_JumpIfSamusHasNoBombs [] {
        if (word!(game.collected_items) & 0x1000) != 0 {
            plmp.offset(2)
        } else {
            *(plmp as *mut u16) as usize as *const u8
        }
    };
    [8704779] PlmInstr_SpawnFourMotherBrainGlass [] {
        QueueSfx3_Max15(0x2e, game);
        SpawnMotherBrainGlassShatteringShard(*(plmp as *mut u16), game);
        SpawnMotherBrainGlassShatteringShard(*(plmp.offset(2) as *mut u16), game);
        SpawnMotherBrainGlassShatteringShard(*(plmp.offset(4) as *mut u16), game);
        SpawnMotherBrainGlassShatteringShard(*(plmp.offset(6) as *mut u16), game);
        plmp.offset(8)
    };
    [8704855] PlmInstr_SpawnTorizoStatueBreaking [] {
        SpawnEprojWithRoomGfx(0xa993, *(plmp as *mut u16), game);
        plmp.offset(2)
    };
    [8705347] PlmInstr_Spawn10shardsAnd6n00bs [] {
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 2, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 4, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 6, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 8, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0xa, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0xc, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0xe, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0x10, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeShards as u16, 0x12, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 2, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 4, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 6, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 8, game);
        SpawnEprojWithRoomGfx(addr_kEproj_N00bTubeReleasedAirBubbles as u16, 0xa, game);
        plmp
    };
    @functions [pub unsafe fn (mut plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8685412] PlmInstr_LoadItemPlmGfx [] {
        let gfx_index = u16::from_le_bytes([game.g_ram[0x1c2d], game.g_ram[0x1c2e]]);
        let slot = 0xdf0c + usize::from(k >> 1) * 2; // plm_variables
        let current_index = u16::from_le_bytes([game.g_ram[0x1c2d], game.g_ram[0x1c2e]]);
        game.g_ram[slot..slot + 2].copy_from_slice(&current_index.to_le_bytes());
        let next_index = gfx_index.wrapping_add(2) & 6;
        game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&next_index.to_le_bytes());
        let table_offset = usize::from(gfx_index >> 1) * 2;
        let vram_destination = game.read_rom_word(0x8487cd, table_offset);
        let tile_offset = usize::from(game.read_rom_word(0x8487d5, table_offset));
        let mut tile_number = game.read_rom_word(0x8487dd, table_offset);
        let instruction = plmp.offset_from(RomBankBase(0x84, game)) as u16;
        let pointer_offset = 0x1c2f + usize::from(gfx_index >> 1) * 2; // plm_item_gfx_ptrs
        game.g_ram[pointer_offset..pointer_offset + 2].copy_from_slice(&instruction.to_le_bytes());
        let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
        game.g_ram[entry..entry + 2].copy_from_slice(&256u16.to_le_bytes());
        let source = (plmp as *const u16).read_unaligned();
        game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
        game.g_ram[entry + 4] = 0x89;
        game.g_ram[entry + 5..entry + 7].copy_from_slice(&vram_destination.to_le_bytes());
        game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
        plmp = plmp.add(2);
        for tile in 0..8 {
            let attributes = u16::from(*plmp) << 10;
            let word = tile_number.wrapping_add(attributes);
            let destination = 0xa000 + tile_offset + tile * 2; // tile_table.tables[0].top_left
            game.g_ram[destination..destination + 2].copy_from_slice(&word.to_le_bytes());
            tile_number = tile_number.wrapping_add(1);
            plmp = plmp.add(1);
        }
        plmp
    };
    @functions [pub unsafe fn (plmp: *const u8, _k: u16, game: &mut Game) -> *const u8];
    [8685541] PlmInstr_CopyFromRamToVram [] {
        let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
        let size = (plmp as *const u16).read_unaligned();
        game.g_ram[entry..entry + 2].copy_from_slice(&size.to_le_bytes());
        let source = (plmp.add(2) as *const u16).read_unaligned();
        game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
        game.g_ram[entry + 4] = *plmp.add(4);
        let destination = (plmp.add(5) as *const u16).read_unaligned();
        game.g_ram[entry + 5..entry + 7].copy_from_slice(&destination.to_le_bytes());
        game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
        plmp.add(7)
    };
    [8686735] PlmInstr_ActivateMapStation [] {
        // map_station_byte_array; word OR with 00FF preserves the neighboring byte.
        let station = 0xd908 + usize::from(game.area_index());
        game.g_ram[station] = 0xff;
        DisplayMessageBox(0x14, game);
        game.set_has_area_map(1);
        plmp
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8685582] PlmInstr_GotoIfBossBitSet [] {
        if CheckBossBitForCurArea(*plmp as u16, game) != 0 {
            PlmInstr_Goto(plmp.offset(1), k)
        } else {
            plmp.offset(3)
        }
    };
    [8685613] PlmInstr_GotoIfEventSet [] {
        if CheckEventHappened(*(plmp as *mut u16), game) != 0 {
            PlmInstr_Goto(plmp.offset(2), k)
        } else {
            plmp.offset(4)
        }
    };
    [8685630] PlmInstr_SetEvent [plmp.offset(2)] { SetEventHappened(*(plmp as *mut u16), game); };
    [8685640] PlmInstr_GotoIfChozoSet [] {
        let v3: u16 = word!(game.plm_room_arguments[(k >> 1)]);
        let idx: i32 = PrepareBitAccess(v3, game) as i32;
        if v3 & 0x8000 == 0
            && (word!(game.bitmask) & *ram_ptr!(game; 0xd830, idx as isize) as u16) != 0
        {
            PlmInstr_Goto(plmp, k)
        } else {
            plmp.offset(2)
        }
    };
    [8685669] PlmInstr_SetRoomChozoBit [plmp] {
        let v3: u16 = word!(game.plm_room_arguments[(k >> 1)]);
        if v3 & 0x8000 == 0 {
            let idx: i32 = PrepareBitAccess(v3, game) as i32;
            crate::update_place! { reference fresh64 = *ram_ptr!(game; 0xd830, idx as isize) => (*fresh64) | ((word!(game.bitmask)) as u8); }
        }
    };
    [8685692] PlmInstr_GotoIfItemBitSet [plmp.offset(2)] {
        let index = usize::from(k >> 1) * 2;
        let v2 = u16::from_le_bytes([game.g_ram[0x1dc7 + index], game.g_ram[0x1dc8 + index]]);
        let byte_index = usize::from(PrepareBitAccess(v2, game));
        if v2 & 0x8000 == 0 {
            let bitmask = u16::from_le_bytes([game.g_ram[0x5e7], game.g_ram[0x5e8]]);
            if bitmask & u16::from(game.g_ram[0xd870 + byte_index]) != 0 {
                return PlmInstr_Goto(plmp, k);
            }
        }
    };
    [8685721] PlmInstr_SetItemBit [plmp] {
        let index = usize::from(k >> 1) * 2;
        let v2 = u16::from_le_bytes([game.g_ram[0x1dc7 + index], game.g_ram[0x1dc8 + index]]);
        if v2 & 0x8000 == 0 {
            let byte_index = usize::from(PrepareBitAccess(v2, game));
            let bitmask = u16::from_le_bytes([game.g_ram[0x5e7], game.g_ram[0x5e8]]);
            game.g_ram[0xd870 + byte_index] |= bitmask as u8;
        }
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    _k: u16,
    game: &mut Game,
) -> *const u8];
    [8685744] PlmInstr_PickupBeamAndShowMessage [plmp.add(3)] {
        let beam = u16::from_le_bytes([*plmp, *plmp.add(1)]);
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        game.set_collected_beams(read_word(game, 0x9a8) | beam);
        game.set_equipped_beams(read_word(game, 0x9a6) | beam);
        // Spazer and Plasma are mutually exclusive; retain both ordered masks.
        game.set_equipped_beams(read_word(game, 0x9a6) & !((beam << 1) & 8));
        game.set_equipped_beams(read_word(game, 0x9a6) & !((beam >> 1) & 4));
        UpdateBeamTilesAndPalette(game);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        let message = u16::from(*plmp.add(2));
        DisplayMessageBox(message, game);
    };
    [8685811] PlmInstr_PickupEquipmentAndShowMessage [plmp.add(3)] {
        let equipment = (plmp as *const u16).read_unaligned();
        let equipped = u16::from_le_bytes([game.g_ram[0x9a2], game.g_ram[0x9a3]]);
        game.set_equipped_items(equipped | equipment);
        game.set_collected_items(game.collected_items() | equipment);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        let message = u16::from(*plmp.add(2));
        DisplayMessageBox(message, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8685850] PlmInstr_PickupEquipmentAddGrappleShowMessage [plmp.offset(2)] {
        let t: u16 = *(plmp as *mut u16);
        word!(game.equipped_items, |fresh55| (fresh55) | (t));
        word!(game.collected_items, |fresh56| (fresh56) | (t));
        AddGrappleToHudTilemap(game);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(5, game);
    };
    [8685889] PlmInstr_PickupEquipmentAddXrayShowMessage [plmp.offset(2)] {
        let t: u16 = *(plmp as *mut u16);
        word!(game.equipped_items, |fresh53| (fresh53) | (t));
        word!(game.collected_items, |fresh54| (fresh54) | (t));
        AddXrayToHudTilemap(game);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(6, game);
    };
    [8685928] PlmInstr_CollectHealthEnergyTank [plmp.add(2)] {
        let amount = (plmp as *const u16).read_unaligned();
        game.set_samus_max_health(game.samus_max_health().wrapping_add(amount));
        game.set_samus_health(game.samus_max_health());
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(1, game);
    };
    [8685958] PlmInstr_CollectHealthReserveTank [plmp.offset(2)] {
        word!(
            game.samus_max_reserve_health,
            wrapping_add(*(plmp as *mut u16))
        );
        if word!(game.reserve_health_mode) == 0 {
            word!(game.reserve_health_mode, wrapping_add(1));
        }
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(0x19, game);
    };
    [8685993] PlmInstr_CollectAmmoMissileTank [plmp.offset(2)] {
        let t = *(plmp as *const u16);
        game.set_samus_max_missiles(game.samus_max_missiles().wrapping_add(t));
        game.set_samus_missiles(game.samus_missiles().wrapping_add(t));
        AddMissilesToHudTilemap(game);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(2, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    _k: u16,
    game: &mut Game,
) -> *const u8];
    [8686034] PlmInstr_CollectAmmoSuperMissileTank [] {
        let amount = u16::from_le_bytes([*plmp, *plmp.add(1)]);
        game.set_samus_max_super_missiles(game.samus_max_super_missiles().wrapping_add(amount));
        game.set_samus_super_missiles(game.samus_super_missiles().wrapping_add(amount));
        AddSuperMissilesToHudTilemap(game);
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        DisplayMessageBox(3, game);
        plmp.add(2)
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8686116] PlmInstr_SetLinkReg [plmp.add(2)] {
        let link = (plmp as *const u16).read_unaligned();
        let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
        game.g_ram[offset..offset + 2].copy_from_slice(&link.to_le_bytes());
    };
    [8686126] PlmInstr_Call [usize::from(target) as *const u8] {
        let return_address = (plmp.offset_from(RomBankBase(0x84, game)) as u16).wrapping_add(2);
        let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
        game.g_ram[offset..offset + 2].copy_from_slice(&return_address.to_le_bytes());
        let target = (plmp as *const u16).read_unaligned();
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8686225] PlmInstr_IncrementDoorHitCounterAndJGE [PlmInstr_Goto(plmp.add(1), k)] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let hits = read_word(game, 0xdf0c + index).wrapping_add(1); // plm_variables
        write_word(game, 0xdf0c + index, hits);
        if (hits as u8) < *plmp {
            return plmp.add(3);
        }
        let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
        if argument & 0x8000 == 0 {
            let bit_index = usize::from(PrepareBitAccess(argument, game));
            let mask = read_word(game, 0x5e7); // bitmask
            let door = 0xd8b0 + bit_index; // opened_door_bit_array
            game.g_ram[door] |= mask as u8;
            // Preserve the original word read, including the following bit-array byte.
            let marked_argument = read_word(game, door) | 0x8000;
            write_word(game, 0x1dc7 + index, marked_argument);
        }
        write_word(game, 0x1cd7 + index, addr_PlmPreInstr_Empty4 as u16);
    };
    [8686285] PlmInstr_IncrementArgumentAndJGE [] {
        let v2: u8 =
            (*(ram_ptr!(game, u16; 0x1dc7) as *mut u8).offset(k as i32 as isize)).wrapping_add(1);
        if v2 >= *plmp {
            let v4: i32 = k as i32 >> 1;
            word!(game.plm_room_arguments[v4] = -1i32 as u16);
            word!(game.plm_pre_instrs[v4] = addr_locret_848AE0 as u16);
            PlmInstr_Goto(plmp.offset(1), k)
        } else {
            word!(game.plm_room_arguments[(k >> 1)] = v2 as u16);
            plmp.offset(3)
        }
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8686321] PlmInstr_SetBTS [plmp.add(1)] {
        let bts = *plmp;
        let index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
        let block_index = u16::from_le_bytes([game.g_ram[index_addr], game.g_ram[index_addr + 1]]) >> 1;
        game.g_ram[0x16402 + usize::from(block_index)] = bts;
    };
    [8686341 | 8686359] PlmInstr_DrawPlmBlock [null::<u8>()] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let block = read_word(game, 0x1e17 + index); // plm_variable
        let block_index = read_word(game, 0x1c87 + index);
        write_word(game, 0x10002 + usize::from(block_index >> 1) * 2, block);
        write_word(game, 0x1e69, block);
        write_word(game, 0x1e67, 1);
        write_word(game, 0x1e6b, 0);
        write_word(game, 0xde1c + index, 1); // plm_instruction_timer
        write_word(game, 0xde6c + index, 0x1e67); // plm_instruction_draw_ptr
        let instruction = plmp.offset_from(RomBankBase(0x84, game)) as u16;
        write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
        ProcessPlmDrawInstruction(k, game);
        let v4: u16 = word!(game.plm_id);
        CalculatePlmBlockCoords(word!(game.plm_id), game);
        DrawPlm(v4, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8686421] PlmInstr_ProcessAirScrollUpdate [plmp] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        write_word(game, 0x1e17 + index, 0); // plm_variable
        let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
        let mut entries = RomPtr_8F(argument, game);
        while *entries & 0x80 == 0 {
            let value = *entries.add(1);
            game.g_ram[0xcd20 + usize::from(*entries)] = value; // scrolls
            entries = entries.add(2);
        }
        let block_index = read_word(game, 0x1c87 + index); // plm_block_indices
        update_level_block_bits(usize::from(block_index >> 1), 0x0fff, 0x3000, game);
    };
    [8686483] PlmInstr_ProcessSolidScrollUpdate [plmp] {
        word!(game.plm_variable[(k >> 1)] = 0);
        let mut v5 = RomPtr_8F(word!(game.plm_room_arguments[(k >> 1)]), game);
        while (*v5 & 0x80 == 0) {
            *ram_ptr!(game; 0xcd20, *v5 as isize) = *v5.offset(1);
            v5 = v5.offset(2);
        }
        let v7: i32 = word!(game.plm_block_indices[(k >> 1)]) as i32 >> 1;
        word!(game.level_data[v7] = (word!(game.level_data[v7]) & 0xfff | 0xb000));
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8686545] PlmInstr_QueueMusic [plmp.offset(1)] { QueueMusic_Delayed8(*plmp as u16, game); };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8686599] PlmInstr_QueueSfx1_Max6 [plmp.offset(1)] { QueueSfx1_Max6(*plmp as u16, game); };
    [8686608] PlmInstr_QueueSfx2_Max6 [plmp.offset(1)] { QueueSfx2_Max6(*plmp as u16, game); };
    [8686617] PlmInstr_QueueSfx3_Max6 [plmp.offset(1)] { QueueSfx3_Max6(*plmp as u16, game); };
    [8686626] PlmInstr_QueueSfx1_Max15 [plmp.offset(1)] { QueueSfx1_Max15(*plmp as u16, game); };
    [8686635] PlmInstr_QueueSfx2_Max15 [plmp.offset(1)] { QueueSfx2_Max15(*plmp as u16, game); };
    [8686644] PlmInstr_QueueSfx3_Max15 [plmp.offset(1)] { QueueSfx3_Max15(*plmp as u16, game); };
    [8686653] PlmInstr_QueueSfx1_Max3 [plmp.offset(1)] { QueueSfx1_Max3(*plmp as u16, game); };
    [8686662] PlmInstr_QueueSfx2_Max3 [plmp.offset(1)] { QueueSfx2_Max3(*plmp as u16, game); };
    [8686671] PlmInstr_QueueSfx_Max3 [plmp.offset(1)] { QueueSfx3_Max3(*plmp as u16, game); };
    [8686680] PlmInstr_QueueSfx1_Max9 [plmp.offset(1)] { QueueSfx1_Max9(*plmp as u16, game); };
    [8686689] PlmInstr_QueueSfx2_Max9 [plmp.offset(1)] { QueueSfx2_Max9(*plmp as u16, game); };
    [8686698] PlmInstr_QueueSfx3_Max9 [plmp.offset(1)] { QueueSfx3_Max9(*plmp as u16, game); };
    [8686707] PlmInstr_QueueSfx1_Max1 [plmp.offset(1)] { QueueSfx1_Max1(*plmp as u16, game); };
    [8686716] PlmInstr_QueueSfx2_Max1 [plmp.offset(1)] { QueueSfx2_Max1(*plmp as u16, game); };
    [8686725] PlmInstr_QueueSfx3_Max1 [plmp.offset(1)] { QueueSfx3_Max1(*plmp as u16, game); };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8686767] PlmInstr_ActivateEnergyStation [plmp] {
        if word!(game.samus_max_health) != word!(game.samus_health) {
            DisplayMessageBox(0x15, game);
            word!(game.samus_health = word!(game.samus_max_health));
        }
        CallSomeSamusCode(1, game);
    };
    [8686800] PlmInstr_ActivateMissileStation [plmp] {
        if word!(game.samus_max_missiles) != word!(game.samus_missiles) {
            DisplayMessageBox(0x16, game);
            word!(game.samus_missiles = word!(game.samus_max_missiles));
        }
        CallSomeSamusCode(1, game);
    };
    [8686833] PlmInstr_ActivateSaveStationAndGotoIfNo [plmp.offset(2)] {
        let r: i32 = DisplayMessageBox_Poll(23, game);
        if r < 0 {
            return plmp.offset(-(2i32 as isize));
        }
        if r == 2 {
            return *(plmp as *mut u16) as usize as *const u8;
        }
        SpawnEprojWithRoomGfx(addr_kEproj_SaveStationElectricity as u16, 0, game);
        word!(
            game.load_station_index = (*(ram_ptr!(game, u16; 0x1dc7))
                .offset((word!(game.plm_id) >> 1) as isize,))
                & (7u16)
        );
        PrepareBitAccess(word!(game.load_station_index), game);
        crate::update_place! { reference fresh40 = *ram_ptr!(game; 0xd8f8, (2 * word!(game.area_index) as i32) as isize) => (*fresh40) | ((word!(game.bitmask)) as u8); }
        SaveToSram(word!(game.selected_save_slot), game);
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8694609] PlmInstr_Scroll_0_1_Blue [plmp] { *(ram_ptr!(game, u16; 0xcd20)) = 257; };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8694941] PlmInstr_DealDamage_2 [plmp] {
        word!(game.samus_periodic_damage, wrapping_add(2));
    };
    [8694961] PlmInstr_GiveInvincibility [plmp] { word!(game.samus_invincibility_timer = 48); };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8695107] PlmInstr_Draw0x38FramesOfRightTreadmill [plmp] {
        WriteRowOfLevelDataBlockAndBTS(k, 0x30ff, 0x8, 0x38, game);
    };
    [8695128] PlmInstr_Draw0x38FramesOfLeftTreadmill [plmp] {
        WriteRowOfLevelDataBlockAndBTS(k, 0x30ff, 0x9, 0x38, game);
    };
    [8695349] PlmInstr_GotoIfSamusHealthFull [*(plmp as *mut u16) as usize as *const u8] {
        if word!(game.samus_max_health) != word!(game.samus_health) {
            return plmp.offset(2);
        }
        CallSomeSamusCode(1, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8695822] PlmInstr_PlaceSamusOnSaveStation [plmp] {
        word!(
            game.samus_x_pos = (word!(game.samus_x_pos) as i32 + 8 & 0xfff0) as u16
        );
        MakeSamusFaceForward(game);
    };
    [8695844] PlmInstr_DisplayGameSavedMessageBox [plmp] { DisplayMessageBox(0x18, game); };
    [8695856] PlmInstr_EnableMovementAndSetSaveStationUsed [plmp] {
        CallSomeSamusCode(1, game);
        word!(game.save_station_lockout_flag = 1);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8698297] PlmInstr_SetCrittersEscapedEvent [plmp] { SetEventHappened(0xf, game); };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8698845] PlmInstr_ClearTrigger [plmp] { word!(game.plm_timers[(k >> 1)] = 0); };
    [8698849] PlmInstr_SpawnEproj [plmp.offset(2)] {
        SpawnEprojWithRoomGfx(*(plmp as *mut u16), 0, game);
    };
    [8698864] PlmInstr_WakeEprojAtPlmPos [plmp.offset(2)] {
        let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
        let mut i: i32 = 34;
        while i >= 0 {
            if v2 == word!(game.eproj_E[(i >> 1)]) {
                break;
            }
            i -= 2;
        }
        let v4: i32 = i >> 1;
        word!(game.eproj_instr_timers[v4] = 1);
        word!(game.eproj_instr_list_ptr[v4], wrapping_add(2));
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8704341] PlmInstr_FxBaseYPos_0x2D2 [plmp] { word!(game.fx_base_y_pos = 722); };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8704967] PlmInstr_QueueSong1MusicTrack [plmp] { QueueMusic_Delayed8(6, game); };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8704983] PlmInstr_TransferWreckedShipChozoSpikesToSlopes [plmp] {
        WriteLevelDataBlockTypeAndBts(0x1608, 0x1012, game);
        WriteLevelDataBlockTypeAndBts(0x160a, 0x1013, game);
    };
    [8705012] PlmInstr_TransferWreckedShipSlopesToChozoSpikes [plmp] {
        WriteLevelDataBlockTypeAndBts(0x1608, 0xa000, game);
        WriteLevelDataBlockTypeAndBts(0x160a, 0xa000, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8705324] PlmInstr_SpawnN00bTubeCrackEproj [plmp] { SpawnEprojWithRoomGfx(0xd904, 0, game); };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8705510] PlmInstr_DisableSamusControls [plmp] { CallSomeSamusCode(0, game); };
    [8705518] PlmInstr_EnableSamusControls [plmp] { CallSomeSamusCode(1, game); };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8705914] PlmInstr_ShootEyeDoorProjectileWithProjectileArg [plmp.offset(2)] {
        SpawnEprojWithRoomGfx(
            addr_kEproj_EyeDoorProjectile as u16,
            *(plmp as *mut u16),
            game,
        );
        QueueSfx2_Max6(0x4c, game);
    };
    [8705936] PlmInstr_SpawnEyeDoorSweatEproj [plmp.offset(2)] {
        SpawnEprojWithRoomGfx(addr_kEproj_EyeDoorSweat as u16, *(plmp as *mut u16), game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8705974] PlmInstr_SpawnEyeDoorSmokeProjectile [plmp] {
        SpawnEprojWithRoomGfx(addr_kEproj_EyeDoorSmoke as u16, 0xb, game);
    };
    [8705987] PlmInstr_MoveUpAndMakeBlueDoorFacingRight [plmp] {
        let v2: i32 = k as i32 >> 1;
        let blk: u16 = word!(
            game.plm_block_indices[v2],
            |fresh33| (fresh33 as i32 - word!(game.room_width_in_blocks) as i32 * 2) as u16,
            reread
        );
        WriteLevelDataBlockTypeAndBts(blk, 0xc041, game);
        sub_84D7EF(blk, game);
    };
    [8706010] PlmInstr_MoveUpAndMakeBlueDoorFacingLeft [plmp] {
        let v2: i32 = k as i32 >> 1;
        let blk: u16 = word!(
            game.plm_block_indices[v2],
            |fresh32| (fresh32 as i32 - word!(game.room_width_in_blocks) as i32 * 2) as u16,
            reread
        );
        WriteLevelDataBlockTypeAndBts(blk, 0xc040, game);
        sub_84D7EF(blk, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8707000] PlmInstr_DamageDraygonTurretFacingDownRight [plmp] {
        SetPlmVarPtr(k, 1, game);
        let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
        WriteLevelDataBlockTypeAndBts(v2, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v2).wrapping_add(2), 0xa003, game);
        let v3: u16 = *(ram_ptr!(game, u16; 0x1c87)).offset((word!(game.plm_id) >> 1) as isize);
        let v4: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v3 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v4, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v4).wrapping_add(2), 0, game);
    };
    [8707063] PlmInstr_DamageDraygonTurretFacingUpRight [plmp] {
        SetPlmVarPtr(k, 1, game);
        let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
        WriteLevelDataBlockTypeAndBts(v2, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v2).wrapping_add(2), 0, game);
        let v4: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v4, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v4).wrapping_add(2), 0xa003, game);
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8707168] PlmInstr_DamageDraygonTurretFacingDownLeft [plmp] {
        SetPlmVarPtr(k, 1, game);
        let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
        WriteLevelDataBlockTypeAndBts(v2, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v2).wrapping_sub(2), 0xa003, game);
        let v4: u16 = (word!(game.room_width_in_blocks) as i32 * 2 + v2 as i32) as u16;
        WriteLevelDataBlockTypeAndBts(v4, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v4).wrapping_sub(2), 0, game);
    };
    [8707231] PlmInstr_DamageDraygonTurretFacingUpLeft [plmp] {
        SetPlmVarPtr(k, 1, game);
        let v2: u16 = word!(game.plm_block_indices[(k >> 1)]);
        WriteLevelDataBlockTypeAndBts(v2, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v2).wrapping_sub(2), 0, game);
        let v4: u16 = (v2 as i32 + word!(game.room_width_in_blocks) as i32 * 2) as u16;
        WriteLevelDataBlockTypeAndBts(v4, 0xa003, game);
        WriteLevelDataBlockTypeAndBts((v4).wrapping_sub(2), 0xa003, game);
    };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8708223] PlmInstr_DrawItemFrame_Common [null::<u8>()] {
        let v2: i32 = k as i32 >> 1;
        word! {game; plm_instruction_timer[v2] = 4;
            plm_instr_list_ptrs[v2] = plmp.offset_from(RomBankBase(0x84, game)) as c_long as u16; }
        ProcessPlmDrawInstruction(k, game);
        let v3: u16 = word!(game.plm_id);
        CalculatePlmBlockCoords(word!(game.plm_id), game);
        DrawPlm(v3, game);
    };
    [8708175] PlmInstr_DrawItemFrame0 [PlmInstr_DrawItemFrame_Common(plmp, k, game)] {
        word!(
            game.plm_instruction_draw_ptr[(k >> 1)] =
                *(rom_ptr!(game, u16; 0x84e05f))
                    .offset((word!(game.plm_variables[(k >> 1)]) >> 1) as isize,)
        );
    };
    [8708199] PlmInstr_DrawItemFrame1 [PlmInstr_DrawItemFrame_Common(plmp, k, game)] {
        word!(
            game.plm_instruction_draw_ptr[(k >> 1)] =
                *(rom_ptr!(game, u16; 0x84e077))
                    .offset((word!(game.plm_variables[(k >> 1)]) >> 1) as isize,)
        );
    };
    @functions [pub unsafe fn (
    plmp: *const u8,
    k: u16,
    game: &mut Game,
) -> *const u8];
    [8708765] PlmInstr_ClearChargeBeamCounter [plmp] { word!(game.flare_counter = 0); };
    @functions [pub unsafe fn (plmp: *const u8, k: u16, game: &mut Game) -> *const u8];
    [8709691] PlmInstr_E63B [plmp] { word!(game.fx_y_vel = -32i32 as u16); };
    @external {
        8685348 => [PlmInstr_Goto(j, k)];
        8707126 => PlmInstr_DamageDraygonTurret2;
    }
}
