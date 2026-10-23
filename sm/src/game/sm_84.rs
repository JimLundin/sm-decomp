pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
use crate::game::enemy_script::{bank_base, EnemyScript as PlmScript};
use crate::game::item_pickups::{MessageHandoff, PickupKind, VisualUpdate};
use crate::game::music::{PlayRoomMusicTrackAfterAFrames, QueueMusic_Delayed8};
pub use crate::game::prelude::*;

use crate::game::message_boxes;
use crate::game::progress_flags::{
    CheckBossBitForCurArea, CheckEventHappened, PrepareBitAccess, SetEventHappened,
};
use crate::game::sm_80::{
    AddGrappleToHudTilemap, AddMissilesToHudTilemap, AddPowerBombsToHudTilemap,
    AddSuperMissilesToHudTilemap, AddXrayToHudTilemap, QueueSfx1_Max1, QueueSfx1_Max15,
    QueueSfx1_Max3, QueueSfx1_Max6, QueueSfx1_Max9, QueueSfx2_Max1, QueueSfx2_Max15,
    QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max1, QueueSfx3_Max15,
    QueueSfx3_Max3, QueueSfx3_Max6, QueueSfx3_Max9,
};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_90::{CallSomeSamusCode, UpdateBeamTilesAndPalette};
use crate::game::sm_91::{GravitySuitPickup, MakeSamusFaceForward, VariaSuitPickup};

pub const NULL: *mut c_void = null_mut::<c_void>();
crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
    kCoroutineNone = 0,
}
#[inline]
unsafe fn RomPtr_RAM(addr: u16, game: &mut Game) -> *mut u8 {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: addr < 0x2000;
            b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
        };
        ram_ptr!(game; addr as i32 as isize)
    }
}
const RomPtr_84: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x84>;
const RomPtr_8F: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x8f>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
pub unsafe fn SetGoldenTorizoPalette(mut a: u16, game: &mut Game) {
    unsafe {
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
}
pub unsafe fn LoadRoomPlmGfx(game: &mut Game) {
    unsafe {
        word!(game.plm_item_gfx_index = 0);
        for v0 in (0i32..8).step_by(2) {
            if word!(game.plm_item_gfx_ptrs[(v0 >> 1)]) != 0 {
                let record =
                    crate::rtl::rom_record(0x84, word!(game.plm_item_gfx_ptrs[(v0 >> 1)]), game);
                PlmInstr_LoadItemPlmGfx(PlmScript::at(record.image, record.base), v0 as u16, game);
            }
        }
    }
}
pub unsafe fn ClearSoundsWhenGoingThroughDoor(game: &mut Game) {
    unsafe {
        CallSomeSamusCode(0x1d, game);
    }
}
pub unsafe fn PlaySpinJumpSoundIfSpinJumping(game: &mut Game) {
    unsafe {
        CallSomeSamusCode(0x1c, game);
    }
}
pub unsafe fn CalculatePlmBlockCoords(k: u16, game: &mut Game) {
    unsafe {
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
}
pub fn WriteLevelDataBlockTypeAndBts(k: u16, a: u16, game: &mut Game) {
    // The level-data byte uses the exact offset, while BTS rounds down to a word.
    let block_high = 0x10003 + usize::from(k);
    game.g_ram[block_high] = (a >> 8) as u8 | (game.g_ram[block_high] & 0x0f);
    game.g_ram[0x16402 + usize::from(k >> 1)] = a as u8;
}
pub fn EnablePLMs(game: &mut Game) {
    // plm_flag: enable bit 15 while preserving the other flag bits.
    let flags = u16::from_le_bytes([game.g_ram[0x1c23], game.g_ram[0x1c24]]);
    game.g_ram[0x1c23..0x1c25].copy_from_slice(&(flags | 0x8000).to_le_bytes());
}
pub unsafe fn DisablePLMs(game: &mut Game) {
    unsafe {
        word!(game.plm_flag &= !(0x8000));
    }
}
pub fn ClearPLMs(game: &mut Game) {
    for slot in (0..40).rev() {
        let header_addr = 0x1c37 + slot * 2; // plm_header_ptr
        game.g_ram[header_addr..header_addr + 2].copy_from_slice(&0u16.to_le_bytes());
    }
    game.g_ram[0x1c2d..0x1c2f].copy_from_slice(&0u16.to_le_bytes()); // plm_item_gfx_index
}
unsafe fn collect_item(plmp: PlmScript, kind: PickupKind, game: &mut Game) -> PlmScript {
    unsafe {
        let amount = match kind {
            PickupKind::Beam | PickupKind::SuperMissiles => {
                u16::from_le_bytes([plmp.byte(0), plmp.byte(1)])
            }
            _ => plmp.word(0),
        };
        let effects = crate::game::item_pickups_ram::collect(game, kind, amount);
        match effects.visual {
            VisualUpdate::None => {}
            VisualUpdate::Beams => UpdateBeamTilesAndPalette(game),
            VisualUpdate::Grapple => AddGrappleToHudTilemap(game),
            VisualUpdate::Xray => AddXrayToHudTilemap(game),
            VisualUpdate::Missiles => AddMissilesToHudTilemap(game),
            VisualUpdate::SuperMissiles => AddSuperMissilesToHudTilemap(game),
            VisualUpdate::PowerBombs => AddPowerBombsToHudTilemap(game),
        }
        PlayRoomMusicTrackAfterAFrames(0x168, game);
        let message = effects.message.unwrap_or_else(|| u16::from(plmp.byte(2)));
        message_boxes::queue(message, game);
        plmp.advance(if effects.message.is_none() { 3 } else { 2 })
    }
}
pub unsafe fn PlmHandler_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        use crate::game::enemy_ram::{native_read, native_write};
        use crate::ram_map::checked_word as word;
        let base = bank_base(0x84);
        crate::verify_probe::hit("coroutine.plm");
        let resume = game.g_ram[0x613];

        if let Some(mut handoff) = MessageHandoff::resume(resume) {
            let mut image = crate::rom::RomImage::default();
            let mut script = PlmScript::Stop;
            if resume == 0 {
                if (word!(game.plm_flag) & 0x8000) == 0 {
                    return kCoroutineNone as CoroutineRet;
                }
                word!(game.plm_draw_tilemap_index = 0);
                word!(game.plm_id = 78);
            }
            loop {
                if !handoff.displaying() {
                    if (word!(game.plm_id) & 0x8000) != 0 {
                        word!(game.plm_id = 0);
                        break;
                    }
                    if native_read(game, 0x1c37 + usize::from(word!(game.plm_id) >> 1) * 2) == 0 {
                        word!(game.plm_id = word!(game.plm_id).wrapping_sub(2));
                        continue;
                    }
                    CallPlmPreInstr(
                        ((native_read(game, 0x1cd7 + usize::from(word!(game.plm_id) >> 1) * 2))
                            as u32)
                            | (0x840000u32),
                        word!(game.plm_id),
                        game,
                    );

                    let timer_slot = word!(game.plm_id) >> 1;
                    let remaining = word!(game.plm_instruction_timer[timer_slot]).wrapping_sub(1);
                    word!(game.plm_instruction_timer[timer_slot] = remaining);
                    if remaining != 0 {
                        word!(game.plm_id = word!(game.plm_id).wrapping_sub(2));
                        continue;
                    }
                    image = game
                        .loaded_rom
                        .as_ref()
                        .expect("ROM must be loaded")
                        .clone();
                    script = PlmScript::at(
                        image.clone(),
                        (base
                            + native_read(game, 0x1d27 + usize::from(word!(game.plm_id) >> 1) * 2)
                                as isize) as usize,
                    );
                }
                loop {
                    if handoff.displaying() {
                        crate::verify_probe::hit("coroutine.plm.message");
                        let yielded: u8 =
                            message_boxes::display(word!(game.queued_message_box_index), game);
                        if let Err(yielded) = handoff.after_display(yielded) {
                            game.g_ram[0x613] = handoff.continuation();
                            return yielded;
                        }
                        word!(game.queued_message_box_index = 0);
                        image = game
                            .loaded_rom
                            .as_ref()
                            .expect("ROM must be loaded")
                            .clone();
                        script = PlmScript::at(
                            image.clone(),
                            (base
                                + native_read(
                                    game,
                                    0x1d27 + usize::from(word!(game.plm_id) >> 1) * 2,
                                ) as isize) as usize,
                        );
                    }
                    if (script.word(0) & 0x8000) == 0 {
                        let mut v7: i32 = word!(game.plm_id) as i32 >> 1;
                        word!(game.plm_instruction_timer[v7] = script.word(0));
                        word!(game.plm_instruction_draw_ptr[v7] = script.word(2));
                        word!(
                            game.plm_instr_list_ptrs[v7] =
                                script.advance(4).offset_from(base) as c_long as u16
                        );
                        ProcessPlmDrawInstruction(word!(game.plm_id), game);
                        CalculatePlmBlockCoords(word!(game.plm_id), game);
                        DrawPlm(word!(game.plm_id), game);

                        break;
                    }
                    let next = CallPlmInstr(
                        ((script.word(0)) as u32) | (0x840000u32),
                        script.advance(2),
                        word!(game.plm_id),
                        game,
                    );

                    let Some(next) = next.resolve(&image, base) else {
                        break;
                    };
                    script = next;
                    if handoff.after_instruction(word!(game.queued_message_box_index)) {
                        native_write(
                            game,
                            0x1d27 + usize::from(word!(game.plm_id) >> 1) * 2,
                            script.offset_from(base) as u16,
                        );
                    }
                }
                word!(game.plm_id = word!(game.plm_id).wrapping_sub(2));
            }
        }
        game.g_ram[0x613] = 0;
        0 as CoroutineRet
    }
}
#[inline]
unsafe fn RomPtr_84orRAM(addr: u16, game: &mut Game) -> *mut u8 {
    unsafe {
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
}
pub unsafe fn ProcessPlmDrawInstruction(k: u16, game: &mut Game) {
    unsafe {
        let slot = usize::from(k >> 1) * 2;
        let draw_instruction =
            u16::from_le_bytes([game.g_ram[0xde6c + slot], game.g_ram[0xde6d + slot]]);
        let image = if draw_instruction & 0x8000 != 0 {
            Some(
                game.loaded_rom
                    .as_ref()
                    .expect("ROM must be loaded")
                    .clone(),
            )
        } else {
            crate::native_assertion! {
                '_c2rust_label: draw_instruction < 0x2000;
                b"addr < 0x2000\0", b"src/sm_84.c\0", 265, b"uint8 *RomPtr_84orRAM(uint16_t)\0"
            };
            None
        };
        let mut cursor = usize::from(draw_instruction)
            + if image.is_some() {
                bank_base(0x84) as usize
            } else {
                0
            };
        // RAM streams stay live: block writes can change later words in the stream.
        // Each read borrows only for its duration; ROM streams retain their allocation.
        let read = |position: usize, game: &Game| {
            u16::from_ne_bytes(core::array::from_fn(|i| match &image {
                Some(image) => image.read(position + i),
                None => game.g_ram[position + i],
            }))
        };
        let mut destination =
            u16::from_le_bytes([game.g_ram[0x1c87 + slot], game.g_ram[0x1c88 + slot]]);
        let origin = destination;
        loop {
            let command = read(cursor, game);
            cursor += 2;
            let stride = if command & 0x8000 != 0 {
                usize::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]])) * 2
            } else {
                2
            };
            for block in 0..usize::from(command as u8) {
                let word = read(cursor, game);
                cursor += 2;
                // The original word indexing rounds odd byte destinations down.
                let offset = 0x10002 + ((usize::from(destination) + block * stride) >> 1) * 2;
                game.g_ram[offset..offset + 2].copy_from_slice(&word.to_le_bytes());
            }
            let displacement = read(cursor, game);
            if displacement == 0 {
                break;
            }
            let x = i32::from(displacement as u8 as i8);
            let y = i32::from((displacement >> 8) as u8 as i8);
            let width = i32::from(u16::from_le_bytes([game.g_ram[0x7a5], game.g_ram[0x7a6]]));
            destination = (i32::from(origin) + 2 * (x + y * width)) as u16;
            cursor += 2;
        }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallPlmInstrFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        9557220 => VariaSuitPickup; 9557434 => GravitySuitPickup;
    }
}
pub fn PlmInstr_Goto(plmp: PlmScript, _k: u16) -> PlmScript {
    let target = plmp.word(0);
    PlmScript::jump(target)
}
unsafe fn PartiallyQueueVramForSingleScreenPlm(
    a: u16,
    k: u16,
    r20: u16,
    r28: u16,
    game: &mut Game,
) {
    unsafe {
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
    unsafe {
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
}
pub unsafe fn DrawPlm(mut k: u16, game: &mut Game) {
    unsafe {
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
                    word!(game.plm_draw_tilemap_index += 8);
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
                    if ((v3) as i16).wrapping_sub(228) >= 0
                        || (32i16).wrapping_sub((R34) as i16) < 0
                    {
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
                    word!(game.plm_draw_tilemap_index += 8);
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
                y = (word!(game.plm_y_block))
                    .wrapping_add((*RomPtr_84orRAM(v32, game) as i8) as u16);
                v1 = (v32).wrapping_add(1);
            } else {
                return;
            }
        }
    }
}
fn advance_plm_instruction(word_index: usize, bytes: u16, game: &mut Game) {
    let offset = 0x1d27 + word_index * 2;
    let instruction = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    game.g_ram[offset..offset + 2].copy_from_slice(&instruction.wrapping_add(bytes).to_le_bytes());
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
pub unsafe fn UNUSED_sub_84B3E3(j: u16) -> u8 {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label;
            b"0\0", b"src/sm_84.c\0", 1370, b"uint8 UNUSED_sub_84B3E3(uint16)\0"
        };
        0
    }
}
pub use self::PlmSetup_ReturnCarryClear as PlmSetup_B73B_B73F_SandFalls;
pub unsafe fn WakePlmIfSamusIsBelowAndRightOfTarget(
    k: u16,
    x_r18: u16,
    y_r20: u16,
    game: &mut Game,
) -> u8 {
    unsafe {
        if x_r18 < word!(game.samus_x_pos) && y_r20 < word!(game.samus_y_pos) {
            let v2: i32 = k as i32 >> 1;
            word!(game.plm_instr_list_ptrs[v2] += 2);
            word!(game.plm_instruction_timer[v2] = 1);
            return false_0 as u8;
        }
        true_0 as u8
    }
}
pub fn PlmSetup_B974(j: u16) -> u8 {
    0
}
pub fn PlmSetup_D0D8_SetVFlag(j: u16) -> u8 {
    0x41
}
pub fn PlmSetup_D0D8_ClearVflag(j: u16) -> u8 {
    1
}

pub use self::PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused as PlmSetup_D70C_NoobTube;
pub unsafe fn SetPlmVarPtr(k: u16, a: u16, game: &mut Game) {
    unsafe {
        *(ram_ptr!(game, u16; word!(game.plm_variable[(k >> 1)]) as isize)) = a;
    }
}
pub use self::PlmInstr_DamageDraygonTurret as PlmInstr_DamageDraygonTurret2;
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
    [8705663] PlmSetup_D700_MakePlmAirBlock_Unused [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0x44, game);
        0
    };
    [8705673] PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused [] {
        WriteLevelDataBlockTypeAndBts(word!(game.plm_block_indices[(j >> 1)]), 0x8044, game);
        0
    };
    @functions [pub fn (j: u16, game: &mut Game) -> u8];
    @functions [pub fn (_j: u16, game: &mut Game) -> u8];
    @functions [pub unsafe fn (j: u16, game: &mut Game) -> u8];
    @prefix { let v1: i32 = j as i32 >> 1; };
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
    @functions [pub unsafe fn (v0: u16, game: &mut Game) -> u8];
    @external {
        8704264 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::Blank, game)];
        8705558 => [crate::game::mechanism_hands_ram::prepare_ship_trigger(j, game)];
        8703957 => [crate::game::samus_periodic_damage_ram::grapple_contact(game)];
        8698353 => [crate::game::mechanism_doors_ram::setup(j, crate::game::mechanism_doors::Setup::CeresEscape, game)];
        8695882 => [crate::game::mechanism_treadmill_ram::setup(j, game)];
        8694568 => [crate::game::mechanism_crumbling_wall_ram::setup(j, game)];
        8696811 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::Ice, displacement, game)];
        8696840 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::Surface, displacement, game)];
        8696983 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::Submerge, displacement, game)];
        8697000 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::SlowFall, displacement, game)];
        8697014 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::FastFall, displacement, game)];
        8697153 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::StopVertical, displacement, game)];
        8697028 => [crate::game::mechanism_contact_physics_ram::setup(crate::game::mechanism_contact_physics::Setup::CollisionSurface, displacement, game)];
        8701261 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Left,Color::Green), setup_a, game) }];
        8701292 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Right,Color::Green), setup_a, game) }];
        8701323 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Left,Color::Red), setup_a, game) }];
        8701359 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Right,Color::Red), setup_a, game) }];
        8701395 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Left,Color::Yellow), setup_a, game) }];
        8701425 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Right,Color::Yellow), setup_a, game) }];
        8701456 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Left,Color::Blue), setup_a, game) }];
        8701479 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Trigger(Side::Right,Color::Blue), setup_a, game) }];
        8701630 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Placement(Direction::Down,true), setup_a, game) }];
        8701643 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Placement(Direction::Up,true), setup_a, game) }];
        8701656 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Placement(Direction::Down,false), setup_a, game) }];
        8701660 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Placement(Direction::Up,false), setup_a, game) }];
        8701664 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Shootblock(Direction::Down), setup_a, game) }];
        8701754 => [{ use crate::game::mechanism_gates::{Setup,Side,Color,Direction}; crate::game::mechanism_gates_ram::setup(j, Setup::Shootblock(Direction::Up), setup_a, game) }];
        8703426 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::ScrewAttack, game)];
        8703466 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::SpeedBoost, game)];
        8703543 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::Crumble, game)];
        8703595 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::Shot, game)];
        8703619 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::BombContact, game)];
        8703706 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::BombShot, game)];
        8703790 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::PowerBombShot, game)];
        8703847 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::SuperMissileShot, game)];
        8703904 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::PowerBombTrigger, game)];
        8698232 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::CrittersEscape, game)];
        8703925 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::Grapple, game)];
        8696769 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::Deactivate, game)];
        8696788 => [crate::game::mechanism_terrain_setup_ram::setup(j, crate::game::mechanism_terrain_setup::Setup::EnemyBreakable, game)];
        8701844 => [crate::game::mechanism_doors_ram::setup(j, crate::game::mechanism_doors::Setup::Grey, game)];
        8701873 => [crate::game::mechanism_doors_ram::setup(j, crate::game::mechanism_doors::Setup::Colored, game)];
        8701883 => [crate::game::mechanism_doors_ram::setup(j, crate::game::mechanism_doors::Setup::Blue, game)];
        8701922 => [crate::game::mechanism_doors_ram::setup(j, crate::game::mechanism_doors::Setup::ShotSignal, game)];
        8694567 | 8696783 | 8698095 | 8698618 | 8705782 | 8705783 | 8704398 => [0];
        8696784 => [PlmSetup_ReturnCarryClear(j)]; 8696786 => [PlmSetup_ReturnCarrySet(j)];
        8696803 => [UNUSED_sub_84B3E3(j)];
        8697167 => [PlmSetup_B73B_B73F_SandFalls(j)];
        8698220 => [PlmSetup_B974(j)];
        8703949 => [PlmSetup_D0D8_SetVFlag(j)];
        8703953 => [PlmSetup_D0D8_ClearVflag(j)];
        8696264 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Access(crate::game::mechanism_stations::Service::Map, crate::game::mechanism_stations::Side::Right), game)];
        8696304 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Access(crate::game::mechanism_stations::Service::Map, crate::game::mechanism_stations::Side::Left), game)];
        8696429 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Access(crate::game::mechanism_stations::Service::Energy, crate::game::mechanism_stations::Side::Right), game)];
        8696477 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Access(crate::game::mechanism_stations::Service::Energy, crate::game::mechanism_stations::Side::Left), game)];
        8696528 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Access(crate::game::mechanism_stations::Service::Missile, crate::game::mechanism_stations::Side::Right), game)];
        8696576 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Access(crate::game::mechanism_stations::Service::Missile, crate::game::mechanism_stations::Side::Left), game)];
        8697232 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::SaveTrigger, game)];
        8697326 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Save, game)];
        8696203 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Place(crate::game::mechanism_stations::Service::Map), game)];
        8696349 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Place(crate::game::mechanism_stations::Service::Energy), game)];
        8696389 => [crate::game::mechanism_stations_ram::setup(j, crate::game::mechanism_stations::Setup::Place(crate::game::mechanism_stations::Service::Missile), game)];
        8696634 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Place(crate::game::mechanism_scroll_setup::Tile::Extension(crate::game::mechanism_scroll_setup::Direction::Right)), game)];
        8696645 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Place(crate::game::mechanism_scroll_setup::Tile::Extension(crate::game::mechanism_scroll_setup::Direction::Left)), game)];
        8696656 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Place(crate::game::mechanism_scroll_setup::Tile::Extension(crate::game::mechanism_scroll_setup::Direction::Down)), game)];
        8696667 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Place(crate::game::mechanism_scroll_setup::Tile::Extension(crate::game::mechanism_scroll_setup::Direction::Up)), game)];
        8696689 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Place(crate::game::mechanism_scroll_setup::Tile::Scroll(crate::game::mechanism_scroll_setup::Solidity::Air)), game)];
        8696706 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Place(crate::game::mechanism_scroll_setup::Tile::Scroll(crate::game::mechanism_scroll_setup::Solidity::Solid)), game)];
        8696723 => [crate::game::mechanism_scroll_setup_ram::setup(j, crate::game::mechanism_scroll_setup::Setup::Touch, game)];
        8711757 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::Visible,Some(crate::game::mechanism_item_setup::Graphic::EnergyTank)), game)];
        8711762 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::Visible,Some(crate::game::mechanism_item_setup::Graphic::MissileTank)), game)];
        8711767 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::Visible,Some(crate::game::mechanism_item_setup::Graphic::SuperMissileTank)), game)];
        8711772 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::Visible,Some(crate::game::mechanism_item_setup::Graphic::PowerBombTank)), game)];
        8711780 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::Visible,None), game)];
        8711799 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::ShotBlock,Some(crate::game::mechanism_item_setup::Graphic::EnergyTank)), game)];
        8711804 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::ShotBlock,Some(crate::game::mechanism_item_setup::Graphic::MissileTank)), game)];
        8711809 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::ShotBlock,Some(crate::game::mechanism_item_setup::Graphic::SuperMissileTank)), game)];
        8711814 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::ShotBlock,Some(crate::game::mechanism_item_setup::Graphic::PowerBombTank)), game)];
        8711822 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Initialize(crate::game::mechanism_item_setup::Container::ShotBlock,None), game)];
        8711851 => [crate::game::mechanism_item_setup_ram::setup(j, crate::game::mechanism_item_setup::Setup::Trigger, game)];
        8707732 => [crate::game::mechanism_cannon_setup_ram::setup(j, crate::game::mechanism_cannon_setup::Setup::Right, game)];
        8707769 => [crate::game::mechanism_cannon_setup_ram::setup(j, crate::game::mechanism_cannon_setup::Setup::DiagonalRight, game)];
        8707824 => [crate::game::mechanism_cannon_setup_ram::setup(j, crate::game::mechanism_cannon_setup::Setup::Left, game)];
        8707861 => [crate::game::mechanism_cannon_setup_ram::setup(j, crate::game::mechanism_cannon_setup::Setup::DiagonalLeft, game)];
        8707916 => [crate::game::mechanism_cannon_setup_ram::setup(j, crate::game::mechanism_cannon_setup::Setup::Unshielded, game)];
        8696028 => [crate::game::mechanism_plants_ram::begin(j, crate::game::mechanism_plants::Surface::Floor, game)];
        8696083 => [crate::game::mechanism_plants_ram::begin(j, crate::game::mechanism_plants::Surface::Ceiling, game)];
        8706700 => [crate::game::mechanism_eye_door_ram::begin(j, crate::game::mechanism_eye_door::Part::Eye, game)];
        8706745 => [crate::game::mechanism_eye_door_ram::begin(j, crate::game::mechanism_eye_door::Part::Body, game)];
        8706846 => [crate::game::mechanism_metroid_quota_ram::begin(j, game)];
        8705526 => [crate::game::mechanism_mother_brain_glass_ram::setup(j, game)];
        8697336 => [crate::game::mechanism_doors_ram::setup(j, crate::game::mechanism_doors::Setup::MotherBrainEscape, game)];
        8697169 => [crate::game::mechanism_tourian_wall_ram::setup(j, crate::game::mechanism_tourian_wall::Setup::Open, game)];
        8697199 => [crate::game::mechanism_tourian_wall_ram::setup(j, crate::game::mechanism_tourian_wall::Setup::Close, game)];
        8698309 => [crate::game::mechanism_animal_rescue_ram::setup(j, crate::game::mechanism_animal_rescue::Setup::RescueWall, game)];
        8698633 => [crate::game::mechanism_animal_rescue_ram::setup(j, crate::game::mechanism_animal_rescue::Setup::MainStreetPassage, game)];
        8698076 => [crate::game::mechanism_shaktool_ram::begin(game)];
        8698012 => [crate::game::mechanism_booster_escape_ram::begin(j, game)];
        8697795 => [crate::game::mechanism_sound_delay_ram::begin(j, game)];
        8704399 => [crate::game::mechanism_hands_ram::setup(j, crate::game::mechanism_hands::Hand::LowerNorfair, displacement, game)];
        8705568 => [crate::game::mechanism_hands_ram::setup(j, crate::game::mechanism_hands::Hand::WreckedShip, displacement, game)];
        8705740 => PlmSetup_D70C_NoobTube;
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPlmPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
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
            word!(game.plm_instr_list_ptrs[v1] += 1);
            word!(game.plm_instr_list_ptrs[v1] += 1);
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
            word!(game.plm_instr_list_ptrs[v2] += 1);
            word!(game.plm_instr_list_ptrs[v2] += 1);
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
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    @prefix { let v1: i32 = k as i32 >> 1; };
    [8698706] PlmPreInstr_WakePlmIfTriggered [] {
        if word!(game.plm_timers[v1]) != 0 {
            word!(game.plm_instr_list_ptrs[v1] += 1);
            word!(game.plm_instr_list_ptrs[v1] += 1);
            word!(game.plm_instruction_timer[v1] = 1);
            word!(game.plm_pre_instrs[v1] = addr_locret_84BB6A as u16);
        }
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
        8704348 => [crate::game::mechanism_hands_ram::monitor_norfair(game)];
        8704486 => [crate::game::mechanism_mother_brain_glass_ram::poll(k, game)];
        8694921 => [crate::game::mechanism_plants_ram::hold(k, game)];
        8705875 => [crate::game::mechanism_eye_door_ram::wake(k, game)];
        8706782 => [crate::game::mechanism_metroid_quota_ram::poll(crate::game::mechanism_metroid_quota::Room::First, game)];
        8706798 => [crate::game::mechanism_metroid_quota_ram::poll(crate::game::mechanism_metroid_quota::Room::Second, game)];
        8706814 => [crate::game::mechanism_metroid_quota_ram::poll(crate::game::mechanism_metroid_quota::Room::Third, game)];
        8706830 => [crate::game::mechanism_metroid_quota_ram::poll(crate::game::mechanism_metroid_quota::Room::Fourth, game)];
        8697839 => [crate::game::mechanism_booster_escape_ram::poll(crate::game::mechanism_booster_escape::Stage::BoosterCollected, k, game)];
        8697898 => [crate::game::mechanism_booster_escape_ram::poll(crate::game::mechanism_booster_escape::Stage::PlayerFarLeft, k, game)];
        8697926 => [crate::game::mechanism_booster_escape_ram::poll(crate::game::mechanism_booster_escape::Stage::AdvanceLava, k, game)];
        8699440 => [crate::game::mechanism_animal_rescue_ram::poll_rescue_gate(k, game)];
        8698032 => [crate::game::mechanism_shaktool_ram::poll(k, game)];
        8697821 => [crate::game::mechanism_sound_delay_ram::poll(k, game)];
        8684649 => [()]; 8708008 => [()]; 8685264 => [()]; 8684774 => [()]; 8684861 => [()];
        8686246 => [()]; 8698787 => [()]; 8704854 => [()]; 8698618 => [()];
        8696028 => [{
                    crate::game::mechanism_plants_ram::begin(k, crate::game::mechanism_plants::Surface::Floor, game);
                }];
        8696083 => [{
                    crate::game::mechanism_plants_ram::begin(k, crate::game::mechanism_plants::Surface::Ceiling, game);
                }];
        8698730 => [()];
        8706773 | 8706774 | 8706775 | 8706776 | 8706777 | 8706778 | 8706779 | 8706780 | 8706781
                | 8686304 => [()];
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallPlmInstr(ea: u32, j: PlmScript, k: u16, game: &mut Game) -> PlmScript => ea;
    call (j, k, game); default {
            Unreachable(game);
            PlmScript::Stop
        };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8685236] PlmInstr_Sleep [] {
        let instruction = (plmp.offset_from(bank_base(0x84)) as u16).wrapping_sub(2);
        let offset = 0x1d27 + usize::from(k >> 1) * 2; // plm_instruction_list_ptrs
        game.g_ram[offset..offset + 2].copy_from_slice(&instruction.to_le_bytes());
        PlmScript::Stop
    };
    [8685249] PlmInstr_PreInstr [] {
        let pre_instruction = plmp.word(0);
        let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
        game.g_ram[offset..offset + 2].copy_from_slice(&pre_instruction.to_le_bytes());
        plmp.advance(2)
    };
    [8685323] PlmInstr_CallFunction [] {
        CallPlmInstrFunc(plmp.load24(0), game);
        plmp.advance(3)
    };
    [8685390] PlmInstr_SetTimer [] {
        game.g_ram[0x1d77 + usize::from(k)] = plmp.byte(0); // Only one byte of plm_timers.
        plmp.advance(1)
    };
    [8686194] PlmInstr_GotoIfDoorBitSet [] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let argument = read_word(game, 0x1dc7 + index); // plm_room_arguments
        if argument & 0x8000 != 0 {
            return plmp.advance(2);
        }
        let bit_index = usize::from(PrepareBitAccess(argument, game));
        let mask = read_word(game, 0x5e7); // bitmask
        if mask & u16::from(game.g_ram[0xd8b0 + bit_index]) != 0 {
            return PlmScript::jump(plmp.word(0));
        }
        plmp.advance(2)
    };
    [8686913] PlmInstr_GotoIfSamusNear [] {
        CalculatePlmBlockCoords(k, game);
        let v2: u8 =
            abs16(((word!(game.samus_x_pos) as i32 >> 4) - word!(game.plm_x_block) as i32) as u16)
                as u8;
        let v4: u8 =
            abs16(((word!(game.samus_y_pos) as i32 >> 4) - word!(game.plm_y_block) as i32) as u16)
                as u8;
        if v2 <= plmp.byte(0) && v4 <= plmp.byte(1) {
            PlmScript::jump(plmp.word(2))
        } else {
            plmp.advance(4)
        }
    };
    [8694742] PlmInstr_ABD6 [] {
        word!(game.plm_block_indices[(k >> 1)] += 2);
        plmp
    };
    [8695487] PlmInstr_GotoIfMissilesFull [] {
        if word!(game.samus_max_missiles) != word!(game.samus_missiles) {
            return plmp.advance(2);
        }
        CallSomeSamusCode(1, game);
        PlmScript::jump(plmp.word(0))
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
    [8705317] PlmInstr_EnableWaterPhysics [] {
        word!(game.fx_liquid_options &= !(4));
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
    @functions [pub fn (_plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8685244] PlmInstr_Delete [] {
        let offset = 0x1c37 + usize::from(k >> 1) * 2; // plm_header_ptr
        game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
        PlmScript::Stop
    };
    [8686138] PlmInstr_Return [] {
        let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
        let link = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
        PlmScript::jump(link)
    };
    @functions [pub fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8685258] PlmInstr_ClearPreInstr [] {
        let offset = 0x1cd7 + usize::from(k >> 1) * 2; // plm_pre_instrs
        game.g_ram[offset..offset + 2].copy_from_slice(&0x86d0u16.to_le_bytes());
        plmp
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8685375] PlmInstr_DecrementAndBranchNonzero [] {
        crate::update_place! { unaligned_word fresh65 [fresh66] = &raw mut (*(ram_ptr!(game, u16; 0x1d77)).offset((k >> 1) as isize)) => (u16::from_le(fresh65.read_unaligned())).wrapping_sub(1); }
        if fresh66 == 1 {
            plmp.advance(2)
        } else {
            PlmInstr_Goto(plmp, k)
        }
    };
    [8686075] PlmInstr_CollectAmmoPowerBombTank [collect_item(plmp, PickupKind::PowerBombs, game)] {};
    [8686557] PlmInstr_ClearMusicQueueAndQueueTrack [] {
        crate::game::music::clear_queue(game);
        QueueMusic_Delayed8(plmp.byte(0) as u16, game);
        plmp.advance(1)
    };
    [8698479] PlmInstr_JumpIfSamusHasNoBombs [] {
        if (word!(game.collected_items) & 0x1000) != 0 {
            plmp.advance(2)
        } else {
            PlmScript::jump(plmp.word(0))
        }
    };
    [8704855] PlmInstr_SpawnTorizoStatueBreaking [] {
        SpawnEprojWithRoomGfx(0xa993, plmp.word(0), game);
        plmp.advance(2)
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
    @functions [pub unsafe fn (mut plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
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
        let instruction = plmp.offset_from(bank_base(0x84)) as u16;
        let pointer_offset = 0x1c2f + usize::from(gfx_index >> 1) * 2; // plm_item_gfx_ptrs
        game.g_ram[pointer_offset..pointer_offset + 2].copy_from_slice(&instruction.to_le_bytes());
        let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
        game.g_ram[entry..entry + 2].copy_from_slice(&256u16.to_le_bytes());
        let source = plmp.word(0);
        game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
        game.g_ram[entry + 4] = 0x89;
        game.g_ram[entry + 5..entry + 7].copy_from_slice(&vram_destination.to_le_bytes());
        game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
        plmp = plmp.advance(2);
        for tile in 0..8 {
            let attributes = u16::from(plmp.byte(0)) << 10;
            let word = tile_number.wrapping_add(attributes);
            let destination = 0xa000 + tile_offset + tile * 2; // tile_table.tables[0].top_left
            game.g_ram[destination..destination + 2].copy_from_slice(&word.to_le_bytes());
            tile_number = tile_number.wrapping_add(1);
            plmp = plmp.advance(1);
        }
        plmp
    };
    @functions [pub unsafe fn (plmp: PlmScript, _k: u16, game: &mut Game) -> PlmScript];
    [8685541] PlmInstr_CopyFromRamToVram [] {
        let queue_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry_tail = u16::from_le_bytes([game.g_ram[0x330], game.g_ram[0x331]]);
        let entry = 0xd0 + usize::from(entry_tail); // vram_write_queue
        let size = plmp.word(0);
        game.g_ram[entry..entry + 2].copy_from_slice(&size.to_le_bytes());
        let source = plmp.word(2);
        game.g_ram[entry + 2..entry + 4].copy_from_slice(&source.to_le_bytes());
        game.g_ram[entry + 4] = plmp.byte(4);
        let destination = plmp.word(5);
        game.g_ram[entry + 5..entry + 7].copy_from_slice(&destination.to_le_bytes());
        game.g_ram[0x330..0x332].copy_from_slice(&queue_tail.wrapping_add(7).to_le_bytes());
        plmp.advance(7)
    };
    [8686735] PlmInstr_ActivateMapStation [] {
        // map_station_byte_array; word OR with 00FF preserves the neighboring byte.
        let station = 0xd908 + usize::from(game.area_index());
        game.g_ram[station] = 0xff;
        message_boxes::queue(0x14, game);
        game.set_has_area_map(1);
        plmp
    };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8685582] PlmInstr_GotoIfBossBitSet [] {
        if CheckBossBitForCurArea(plmp.byte(0) as u16, game) != 0 {
            PlmInstr_Goto(plmp.advance(1), k)
        } else {
            plmp.advance(3)
        }
    };
    [8685613] PlmInstr_GotoIfEventSet [] {
        if CheckEventHappened(plmp.word(0), game) != 0 {
            PlmInstr_Goto(plmp.advance(2), k)
        } else {
            plmp.advance(4)
        }
    };
    [8685630] PlmInstr_SetEvent [plmp.advance(2)] { SetEventHappened(plmp.word(0), game); };
    [8685640] PlmInstr_GotoIfChozoSet [] {
        let v3: u16 = word!(game.plm_room_arguments[(k >> 1)]);
        let idx: i32 = PrepareBitAccess(v3, game) as i32;
        if v3 & 0x8000 == 0
            && (word!(game.bitmask) & *ram_ptr!(game; 0xd830, idx as isize) as u16) != 0
        {
            PlmInstr_Goto(plmp, k)
        } else {
            plmp.advance(2)
        }
    };
    [8685669] PlmInstr_SetRoomChozoBit [plmp] {
        let v3: u16 = word!(game.plm_room_arguments[(k >> 1)]);
        if v3 & 0x8000 == 0 {
            let idx: i32 = PrepareBitAccess(v3, game) as i32;
            crate::update_place! { reference fresh64 = *ram_ptr!(game; 0xd830, idx as isize) => (*fresh64) | ((word!(game.bitmask)) as u8); }
        }
    };
    [8685692] PlmInstr_GotoIfItemBitSet [plmp.advance(2)] {
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
    plmp: PlmScript,
    _k: u16,
    game: &mut Game,
) -> PlmScript];
    [8685744] PlmInstr_PickupBeamAndShowMessage [collect_item(plmp, PickupKind::Beam, game)] {};
    [8685811] PlmInstr_PickupEquipmentAndShowMessage [collect_item(plmp, PickupKind::Equipment, game)] {};
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8685850] PlmInstr_PickupEquipmentAddGrappleShowMessage [collect_item(plmp, PickupKind::Grapple, game)] {};
    [8685889] PlmInstr_PickupEquipmentAddXrayShowMessage [collect_item(plmp, PickupKind::Xray, game)] {};
    [8685928] PlmInstr_CollectHealthEnergyTank [collect_item(plmp, PickupKind::Energy, game)] {};
    [8685958] PlmInstr_CollectHealthReserveTank [collect_item(plmp, PickupKind::Reserve, game)] {};
    [8685993] PlmInstr_CollectAmmoMissileTank [collect_item(plmp, PickupKind::Missiles, game)] {};
    @functions [pub unsafe fn (
    plmp: PlmScript,
    _k: u16,
    game: &mut Game,
) -> PlmScript];
    [8686034] PlmInstr_CollectAmmoSuperMissileTank [collect_item(plmp, PickupKind::SuperMissiles, game)] {};
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8686116] PlmInstr_SetLinkReg [plmp.advance(2)] {
        let link = plmp.word(0);
        let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
        game.g_ram[offset..offset + 2].copy_from_slice(&link.to_le_bytes());
    };
    [8686126] PlmInstr_Call [PlmScript::jump(target)] {
        let return_address = (plmp.offset_from(bank_base(0x84)) as u16).wrapping_add(2);
        let offset = 0xdebc + usize::from(k >> 1) * 2; // plm_instruction_list_link_reg
        game.g_ram[offset..offset + 2].copy_from_slice(&return_address.to_le_bytes());
        let target = plmp.word(0);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8686225] PlmInstr_IncrementDoorHitCounterAndJGE [PlmInstr_Goto(plmp.advance(1), k)] {
        let index = usize::from(k >> 1) * 2;
        let read_word = |game: &Game, offset: usize| {
            u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
        };
        let write_word = |game: &mut Game, offset: usize, value: u16| {
            game.g_ram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        };
        let hits = read_word(game, 0xdf0c + index).wrapping_add(1); // plm_variables
        write_word(game, 0xdf0c + index, hits);
        if (hits as u8) < plmp.byte(0) {
            return plmp.advance(3);
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
            (*ram_ptr!(game; 0x1dc7, k as i32 as isize)).wrapping_add(1);
        if v2 >= plmp.byte(0) {
            let v4: i32 = k as i32 >> 1;
            word!(game.plm_room_arguments[v4] = -1i32 as u16);
            word!(game.plm_pre_instrs[v4] = addr_locret_848AE0 as u16);
            PlmInstr_Goto(plmp.advance(1), k)
        } else {
            word!(game.plm_room_arguments[(k >> 1)] = v2 as u16);
            plmp.advance(3)
        }
    };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8686321] PlmInstr_SetBTS [plmp.advance(1)] {
        let bts = plmp.byte(0);
        let index_addr = 0x1c87 + usize::from(k >> 1) * 2; // plm_block_indices
        let block_index = u16::from_le_bytes([game.g_ram[index_addr], game.g_ram[index_addr + 1]]) >> 1;
        game.g_ram[0x16402 + usize::from(block_index)] = bts;
    };
    [8686341 | 8686359] PlmInstr_DrawPlmBlock [PlmScript::Stop] {
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
        let instruction = plmp.offset_from(bank_base(0x84)) as u16;
        write_word(game, 0x1d27 + index, instruction); // plm_instr_list_ptrs
        ProcessPlmDrawInstruction(k, game);
        let v4: u16 = word!(game.plm_id);
        CalculatePlmBlockCoords(word!(game.plm_id), game);
        DrawPlm(v4, game);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
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
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8686545] PlmInstr_QueueMusic [plmp.advance(1)] { QueueMusic_Delayed8(plmp.byte(0) as u16, game); };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8686599] PlmInstr_QueueSfx1_Max6 [plmp.advance(1)] { QueueSfx1_Max6(plmp.byte(0) as u16, game); };
    [8686608] PlmInstr_QueueSfx2_Max6 [plmp.advance(1)] { QueueSfx2_Max6(plmp.byte(0) as u16, game); };
    [8686617] PlmInstr_QueueSfx3_Max6 [plmp.advance(1)] { QueueSfx3_Max6(plmp.byte(0) as u16, game); };
    [8686626] PlmInstr_QueueSfx1_Max15 [plmp.advance(1)] { QueueSfx1_Max15(plmp.byte(0) as u16, game); };
    [8686635] PlmInstr_QueueSfx2_Max15 [plmp.advance(1)] { QueueSfx2_Max15(plmp.byte(0) as u16, game); };
    [8686644] PlmInstr_QueueSfx3_Max15 [plmp.advance(1)] { QueueSfx3_Max15(plmp.byte(0) as u16, game); };
    [8686653] PlmInstr_QueueSfx1_Max3 [plmp.advance(1)] { QueueSfx1_Max3(plmp.byte(0) as u16, game); };
    [8686662] PlmInstr_QueueSfx2_Max3 [plmp.advance(1)] { QueueSfx2_Max3(plmp.byte(0) as u16, game); };
    [8686671] PlmInstr_QueueSfx_Max3 [plmp.advance(1)] { QueueSfx3_Max3(plmp.byte(0) as u16, game); };
    [8686680] PlmInstr_QueueSfx1_Max9 [plmp.advance(1)] { QueueSfx1_Max9(plmp.byte(0) as u16, game); };
    [8686689] PlmInstr_QueueSfx2_Max9 [plmp.advance(1)] { QueueSfx2_Max9(plmp.byte(0) as u16, game); };
    [8686698] PlmInstr_QueueSfx3_Max9 [plmp.advance(1)] { QueueSfx3_Max9(plmp.byte(0) as u16, game); };
    [8686707] PlmInstr_QueueSfx1_Max1 [plmp.advance(1)] { QueueSfx1_Max1(plmp.byte(0) as u16, game); };
    [8686716] PlmInstr_QueueSfx2_Max1 [plmp.advance(1)] { QueueSfx2_Max1(plmp.byte(0) as u16, game); };
    [8686725] PlmInstr_QueueSfx3_Max1 [plmp.advance(1)] { QueueSfx3_Max1(plmp.byte(0) as u16, game); };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8686767] PlmInstr_ActivateEnergyStation [plmp] {
        if word!(game.samus_max_health) != word!(game.samus_health) {
            message_boxes::queue(0x15, game);
            word!(game.samus_health = word!(game.samus_max_health));
        }
        CallSomeSamusCode(1, game);
    };
    [8686800] PlmInstr_ActivateMissileStation [plmp] {
        if word!(game.samus_max_missiles) != word!(game.samus_missiles) {
            message_boxes::queue(0x16, game);
            word!(game.samus_missiles = word!(game.samus_max_missiles));
        }
        CallSomeSamusCode(1, game);
    };
    [8686833] PlmInstr_ActivateSaveStationAndGotoIfNo [plmp.advance(2)] {
        let r: i32 = message_boxes::poll(23, game);
        if r < 0 {
            return plmp.advance(-(2i32 as isize));
        }
        if r == 2 {
            return PlmScript::jump(plmp.word(0));
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
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8695349] PlmInstr_GotoIfSamusHealthFull [PlmScript::jump(plmp.word(0))] {
        if word!(game.samus_max_health) != word!(game.samus_health) {
            return plmp.advance(2);
        }
        CallSomeSamusCode(1, game);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8695822] PlmInstr_PlaceSamusOnSaveStation [plmp] {
        word!(
            game.samus_x_pos = (word!(game.samus_x_pos) as i32 + 8 & 0xfff0) as u16
        );
        MakeSamusFaceForward(game);
    };
    [8695844] PlmInstr_DisplayGameSavedMessageBox [plmp] { message_boxes::queue(0x18, game); };
    [8695856] PlmInstr_EnableMovementAndSetSaveStationUsed [plmp] {
        CallSomeSamusCode(1, game);
        word!(game.save_station_lockout_flag = 1);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8698845] PlmInstr_ClearTrigger [plmp] { word!(game.plm_timers[(k >> 1)] = 0); };
    [8698849] PlmInstr_SpawnEproj [plmp.advance(2)] {
        SpawnEprojWithRoomGfx(plmp.word(0), 0, game);
    };
    [8698864] PlmInstr_WakeEprojAtPlmPos [plmp.advance(2)] {
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
        word!(game.eproj_instr_list_ptr[v4] += 2);
    };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8704967] PlmInstr_QueueSong1MusicTrack [plmp] { QueueMusic_Delayed8(6, game); };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8704983] PlmInstr_TransferWreckedShipChozoSpikesToSlopes [plmp] {
        WriteLevelDataBlockTypeAndBts(0x1608, 0x1012, game);
        WriteLevelDataBlockTypeAndBts(0x160a, 0x1013, game);
    };
    [8705012] PlmInstr_TransferWreckedShipSlopesToChozoSpikes [plmp] {
        WriteLevelDataBlockTypeAndBts(0x1608, 0xa000, game);
        WriteLevelDataBlockTypeAndBts(0x160a, 0xa000, game);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8705324] PlmInstr_SpawnN00bTubeCrackEproj [plmp] { SpawnEprojWithRoomGfx(0xd904, 0, game); };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8705510] PlmInstr_DisableSamusControls [plmp] { CallSomeSamusCode(0, game); };
    [8705518] PlmInstr_EnableSamusControls [plmp] { CallSomeSamusCode(1, game); };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8705914] PlmInstr_ShootEyeDoorProjectileWithProjectileArg [plmp.advance(2)] {
        SpawnEprojWithRoomGfx(
            addr_kEproj_EyeDoorProjectile as u16,
            plmp.word(0),
            game,
        );
        QueueSfx2_Max6(0x4c, game);
    };
    [8705936] PlmInstr_SpawnEyeDoorSweatEproj [plmp.advance(2)] {
        SpawnEprojWithRoomGfx(addr_kEproj_EyeDoorSweat as u16, plmp.word(0), game);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8705974] PlmInstr_SpawnEyeDoorSmokeProjectile [plmp] {
        SpawnEprojWithRoomGfx(addr_kEproj_EyeDoorSmoke as u16, 0xb, game);
    };
    @functions [pub unsafe fn (
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
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
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
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
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8708223] PlmInstr_DrawItemFrame_Common [PlmScript::Stop] {
        let v2: i32 = k as i32 >> 1;
        word! {game; plm_instruction_timer[v2] = 4;
            plm_instr_list_ptrs[v2] = plmp.offset_from(bank_base(0x84)) as c_long as u16; }
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
    plmp: PlmScript,
    k: u16,
    game: &mut Game,
) -> PlmScript];
    [8708765] PlmInstr_ClearChargeBeamCounter [plmp] { word!(game.flare_counter = 0); };
    @functions [pub unsafe fn (plmp: PlmScript, k: u16, game: &mut Game) -> PlmScript];
    [8709691] PlmInstr_E63B [plmp] { word!(game.fx_y_vel = -32i32 as u16); };
    @external {
        8704341 => [{ crate::game::mechanism_hands_ram::lower_acid(game); j }];
        8695107 | 8695128 => [{ crate::game::mechanism_treadmill_ram::activate(k, if ea == 8695107 { crate::game::mechanism_treadmill::Direction::Right } else { crate::game::mechanism_treadmill::Direction::Left }, game); j }];
        8694609 => [{ crate::game::mechanism_crumbling_wall_ram::open_passage(game); j }];
        8694528 | 8694617 => [{ crate::game::mechanism_crumbling_wall_ram::move_down(k, game); j }];
        8704761 => [crate::game::mechanism_mother_brain_glass_ram::command(j, k, crate::game::mechanism_mother_brain_glass::Command::Threshold, game, |parameter, game| { SpawnEprojWithRoomGfx(0xcefc, parameter, game); })];
        8704779 => [crate::game::mechanism_mother_brain_glass_ram::command(j, k, crate::game::mechanism_mother_brain_glass::Command::Shatter, game, |parameter, game| { SpawnEprojWithRoomGfx(0xcefc, parameter, game); })];
        8698297 => [{crate::game::mechanism_animal_rescue_ram::mark_rescued(game); j}];
        8698661 => [{crate::game::mechanism_animal_rescue_ram::move_passage(k, game); j}];
        8685348 => [PlmInstr_Goto(j, k)];
        8694941 => [{crate::game::mechanism_plants_ram::hurt(game); j}];
        8694961 => [{crate::game::mechanism_plants_ram::protect(game); j}];
        8705987 => [{crate::game::mechanism_eye_door_ram::convert(k, crate::game::mechanism_eye_door::Side::Right, game); j}];
        8706010 => [{crate::game::mechanism_eye_door_ram::convert(k, crate::game::mechanism_eye_door::Side::Left, game); j}];
        8707126 => PlmInstr_DamageDraygonTurret2;
    }
}
