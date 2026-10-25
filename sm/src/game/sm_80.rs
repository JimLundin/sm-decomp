use crate::game::decompression::{self, decompress_to_ram};
use crate::game::display_refresh::ScrollingDmaLayout;
use crate::game::music::HandleMusicQueue;
pub use crate::game::prelude::*;
use crate::game::room_state::load_tiles_and_palette;

pub use crate::game::constants::bank_60::*;

use crate::game::room_mechanisms::reload_graphics;
use crate::rtl::RtlApuUpload;

const RomPtr_80: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x80>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnDoorTransition_FadeInScreenAndFinish = 0x82e737,
}
pub unsafe fn APU_UploadBank(addr: u32, game: &mut Game) {
    unsafe {
        if !game.g_use_my_apu_code {
            crate::native_audio::upload(addr, game);
            return;
        }
        RtlApuUpload(addr, game);
    }
}
pub unsafe fn NextRandom(game: &mut Game) -> u16 {
    unsafe {
        let mut RegWord: u16 =
            ((byte!(@pointer ram_ptr!(game, u16; 0x5e5); 0)) as u16).wrapping_mul(5);
        let Reg: u8 = (byte!(@pointer ram_ptr!(game, u16; 0x5e5); 1isize)).wrapping_mul(5);
        let carry: i32 = byte!(RegWord; 1) as i32 + Reg as i32 + 1;
        byte!(RegWord; 1) = carry as u8;
        let result: u16 = ((carry >> 8) + RegWord as i32 + 17) as u16;
        word!(game.random_number = result);
        #[cfg(sm_verify_dll)]
        crate::cpu_infra::player_transitions_composed::random_after(game);
        result
    }
}
pub fn Multiply16x16(a: u16, j: u16, game: &mut Game) -> u32 {
    game.g_ram[0x5e9..0x5eb].copy_from_slice(&a.to_le_bytes());
    game.g_ram[0x5eb..0x5ed].copy_from_slice(&j.to_le_bytes());
    let result: u32 = (a as u32).wrapping_mul(j as u32);
    result
}
pub unsafe fn memset7E(mut k: *mut u16, a: u16, mut j: u16) {
    unsafe {
        do_until! { j == 0 ;
            crate::update_place! { previous fresh99 = k => k.offset(1); }
            *fresh99 = a;
            j = (j).wrapping_sub(2);
        }
    }
}
pub unsafe fn InvalidInterrupt_Crash(game: &mut Game) {
    unsafe {
        printf(c"InvalidInterrupt_Crash\n".as_ptr());
        Unreachable(game);
        loop {}
    }
}
pub unsafe fn LoadMirrorOfExploredMapTiles(game: &mut Game) {
    unsafe {
        let mut v1: u16 = u16::swap_bytes(word!(game.area_index));
        for v2 in (0u16..256).step_by(2) {
            *(ram_ptr!(game, u16; 0x7f7, v2 as isize)) =
                word!(game.explored_map_tiles_saved[(v1 >> 1)]);
            v1 = (v1).wrapping_add(2);
        }
        word!(game.has_area_map = *ram_ptr!(game; 0xd908, word!(game.area_index) as isize) as u16);
    }
}
pub unsafe fn SaveExploredMapTilesToSaved(game: &mut Game) {
    unsafe {
        let mut v1: u16 = u16::swap_bytes(word!(game.area_index));
        for v2 in (0u16..256).step_by(2) {
            word!(
                game.explored_map_tiles_saved[(v1 >> 1)] =
                    *(ram_ptr!(game, u16; 0x7f7, v2 as isize))
            );
            v1 = (v1).wrapping_add(2);
        }
        if word!(game.has_area_map) != 0 {
            crate::update_place! { unaligned_word fresh88 = &raw mut (*(ram_ptr!(game, u16; 0xd908, word!(game.area_index) as isize))) => (u16::from_le(fresh88.read_unaligned())) | (0xffu16); }
        }
    }
}
pub fn ClearUnusedOam(game: &mut Game) {
    let oam_next_ptr = u16::from_le_bytes([game.g_ram[0x590], game.g_ram[0x591]]);
    let first_unused_sprite = usize::from(oam_next_ptr >> 2);
    for sprite in game.g_ram[0x370..0x570]
        .chunks_exact_mut(4)
        .skip(first_unused_sprite)
    {
        // Each four-byte OAM entry stores its Y coordinate in byte 1.
        sprite[1] = 0xf0;
    }
    game.g_ram[0x590..0x592].fill(0);
}

pub fn ClearOamExt(game: &mut Game) {
    game.g_ram[0x570..0x590].fill(0);
}
pub unsafe fn QueueMode7Transfers(db: u8, k: u16, game: &mut Game) {
    unsafe {
        let address = (u32::from(db) << 16) | u32::from(k);
        if address & 0x8000 == 0 {
            printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
            game.g_fail = true;
        }
        crate::game::mode7_transfers::enqueue_rom(address, game);
    }
}
// Byte-backed queue layout from ram_map: three channels, sixteen slots each.
pub(crate) fn queue_sfx(channel: usize, sound: u8, max_queued: u8, game: &mut Game) -> u16 {
    #[cfg(sm_verify_dll)]
    crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
    let read_pos = 0x643 + channel;
    let write_pos = 0x646 + channel;
    let limit_pos = 0x653 + channel;
    let buffer = 0x656 + 16 * channel;

    // This write also happens when capacity or the sound gates reject the request.
    game.g_ram[limit_pos] = max_queued;
    let queued = game.g_ram[write_pos].wrapping_sub(game.g_ram[read_pos]) & 15;
    if queued >= max_queued {
        return u16::from_be_bytes([sound, queued]);
    }
    if game.debug_disable_sounds() != 0
        || game.game_state() >= 40
        || game.power_bomb_explosion_status() & 0x8000 != 0
    {
        return u16::from_be_bytes([queued, sound]);
    }

    let write = game.g_ram[write_pos];
    let mut next = write.wrapping_add(1);
    if next >= 16 {
        next = 0;
    }
    // Preserve adjacent-RAM access for an out-of-range current write position.
    let slot = buffer + usize::from(write);
    if next == game.g_ram[read_pos] {
        // Lower sound numbers take priority when the ring is full.
        if sound < game.g_ram[slot] {
            game.g_ram[slot] = sound;
        }
    } else {
        game.g_ram[slot] = sound;
        game.g_ram[write_pos] = next;
        game.g_ram[buffer + usize::from(next)] = 0;
    }
    // Callers that forward the original accumulator need both queue depth and sound.
    u16::from_be_bytes([queued, sound])
}

crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx1_Max15 => queue_sfx(0, sound as u8, 15, game);
    QueueSfx1_Max9 => queue_sfx(0, sound as u8, 9, game);
    QueueSfx1_Max3 => queue_sfx(0, sound as u8, 3, game);
    QueueSfx1_Max1 => queue_sfx(0, sound as u8, 1, game);
    QueueSfx1_Max6 => queue_sfx(0, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx2_Max15 => queue_sfx(1, sound as u8, 15, game);
    QueueSfx2_Max9 => queue_sfx(1, sound as u8, 9, game);
    QueueSfx2_Max3 => queue_sfx(1, sound as u8, 3, game);
    QueueSfx2_Max1 => queue_sfx(1, sound as u8, 1, game);
    QueueSfx2_Max6 => queue_sfx(1, sound as u8, 6, game);
}
crate::forward_calls! { pub fn (sound: u16, game: &mut Game);
    QueueSfx3_Max15 => queue_sfx(2, sound as u8, 15, game);
    QueueSfx3_Max9 => queue_sfx(2, sound as u8, 9, game);
    QueueSfx3_Max3 => queue_sfx(2, sound as u8, 3, game);
    QueueSfx3_Max1 => queue_sfx(2, sound as u8, 1, game);
    QueueSfx3_Max6 => queue_sfx(2, sound as u8, 6, game);
}
pub unsafe fn SetupDmaTransfer(p: *const c_void, game: &mut Game) {
    unsafe {
        let s = p as *const StartDmaCopy;
        let v2: i32 = (*s).chan as i32 * 16;
        crate::rtl::register_writes! { game;
            word((v2 + DMAP0 as i32) as SnesRegs as u16,
            (&raw const (*s).dmap).cast::<u16>().read_unaligned(),); word((v2 + A1T0L as i32) as SnesRegs as u16,
            (&raw const (*s).a1.addr).read_unaligned(),);
            byte((v2 + A1B0 as i32) as SnesRegs as u16, (*s).a1.bank); word((v2 + DAS0L as i32) as SnesRegs as u16, (*s).das);
        }
    }
}
// Graphics loads keep source, destination, then transfer size together.
// Decompression uses RAM offsets; VRAM uses word addresses. WRAM transfers
// retain all three destination-register bytes, including ignored bank bits.
pub(crate) enum GraphicsAsset {
    Decompress(u32, usize),
    VramWords(u32, u16, u16),
    VramLowBytes(u32, u16, u16),
    VramHighBytes(u32, u16, u16),
    WramBytes(u32, u32, u16),
}

impl GraphicsAsset {
    pub(crate) unsafe fn load(self, game: &mut Game) {
        unsafe {
            crate::verify_probe::hit(match &self {
                Self::Decompress(..) => "graphics-load-decompress",
                Self::VramWords(..) => "graphics-load-vram-words",
                Self::VramLowBytes(..) => "graphics-load-vram-low",
                Self::VramHighBytes(..) => "graphics-load-vram-high",
                Self::WramBytes(..) => "graphics-load-wram",
            });
            let (transfer, vram) = match self {
                Self::Decompress(source, destination) => {
                    decompress_to_ram(source, destination, game);
                    return;
                }
                Self::VramWords(source, destination, size) => (
                    StartDmaCopy::vram_words(1, source, size),
                    Some((destination, 0x80)),
                ),
                Self::VramLowBytes(source, destination, size) => (
                    StartDmaCopy::vram_low_bytes(1, source, size),
                    Some((destination, 0)),
                ),
                Self::VramHighBytes(source, destination, size) => (
                    StartDmaCopy::vram_high_bytes(1, source, size),
                    Some((destination, 0x80)),
                ),
                Self::WramBytes(source, destination, size) => {
                    crate::rtl::register_writes! { game;
                        byte(WMADDL as u16, destination as u8);
                        byte(WMADDM as u16, (destination >> 8) as u8);
                        byte(WMADDH as u16, (destination >> 16) as u8);
                    }
                    (StartDmaCopy::wram_bytes(1, source, size), None)
                }
            };
            if let Some((destination, vmain)) = vram {
                crate::rtl::register_writes! { game;
                    byte(VMADDL as u16, destination as u8); byte(VMADDH as u16, (destination >> 8) as u8);
                    byte(VMAIN as u16, vmain);
                }
            }
            SetupDmaTransfer(&raw const transfer as *const c_void, game);
            WriteReg(MDMAEN as u16, 2, game);
        }
    }
}

pub unsafe fn CopyToVramNow(vram_dst: u16, src: u32, size: u16, game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            byte(INIDISP as u16, 0x80); word(VMADDL as u16, vram_dst);
            word(DMAP1 as u16, 0x1801); word(A1T1L as u16, src as u16);
            byte(A1B1 as u16, (src >> 16) as u8); word(DAS1L as u16, size);
            byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
            byte(INIDISP as u16, 0xf);
        }
    }
}
pub unsafe fn StartGameplay_Async(game: &mut Game) -> CoroutineRet {
    let poll = crate::game::room_entry::protocol::resume(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::room_entry::EntrySignal::Initial,
    );
    poll.returned_byte()
}
pub unsafe fn Play20FramesOfMusic_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::verify_probe::hit("coroutine.music");
        let resume_state: *mut u8 = ram_ptr!(game; 0x614);
        let resume = *resume_state;
        if resume == 0 {
            crate::game::display_refresh::enable(true, game);
            word!(game.my_counter = 0);
        }
        if resume <= 1 {
            let mut waiting = resume == 1;
            loop {
                if !waiting {
                    if word!(game.my_counter) == 20 {
                        crate::game::display_refresh::enable(false, game);
                        break;
                    }
                    HandleMusicQueue(game);
                }
                let yielded: u8 = crate::game::display_refresh::wait(true, game);
                if yielded != 0 {
                    *resume_state = 1;
                    return yielded;
                }
                word!(game.my_counter += 1);
                waiting = false;
            }
        }
        *resume_state = 0;
        0 as CoroutineRet
    }
}
pub unsafe fn ResumeGameplay(game: &mut Game) {
    unsafe {
        WriteRegWord(MDMAEN as u16, 0, game);
        crate::game::display_refresh::enable(false, game);
        crate::game::scanline_irq::disable(game);
        load_tiles_and_palette(game);
        crate::game::room_entry::background::load(
            &mut crate::game::session_runtime::Runtime::new(game),
            crate::game::room_entry::content::LoadMode::Initial,
        );
        DisplayViewablePartOfRoom(game);
        reload_graphics(game);
        crate::game::display_refresh::enable(true, game);
        crate::game::scanline_irq::enable(false, game);
    }
}
pub unsafe fn DisplayViewablePartOfRoom(game: &mut Game) {
    unsafe {
        let mut v1: i32 = (*ram_ptr!(game; 0x58) as i32 - *ram_ptr!(game; 0x59) as i32) << 8;
        word!(game.size_of_bg2 = (v1 & 0xf800) as u16);
        CalculateBgScrollAndLayerPositionBlocks(game);
        let mut v2: i32 = 0;
        do_until! { v3 == 16 ;
            let mut v3: i32 = v2;
            word!(game.blocks_to_update_x_block = word!(game.layer1_x_block));
            word!(game.blocks_to_update_y_block = word!(game.layer1_y_block));
            word!(game.vram_blocks_to_update_x_block = word!(game.bg1_x_block));
            word!(game.vram_blocks_to_update_y_block = word!(game.bg1_y_block));
            UploadLevelDataColumn(game);
            if *ram_ptr!(game; 0x91b) & 1 == 0 {
                word!(game.blocks_to_update_x_block = word!(game.layer2_x_block));
                word!(game.blocks_to_update_y_block = word!(game.layer2_y_block));
                word!(game.vram_blocks_to_update_x_block = word!(game.bg2_x_block));
                word!(game.vram_blocks_to_update_y_block = word!(game.bg2_y_block));
                UploadBackgroundDataColumn(game);
            }
            crate::game::display_refresh::flush_tilemaps(game);
            word!(game.layer1_x_block += 1);
            word!(game.bg1_x_block += 1);
            word!(game.layer2_x_block += 1);
            word!(game.bg2_x_block += 1);
            v2 += 1;
        }
    }
}
pub unsafe fn QueueClearingOfFxTilemap(game: &mut Game) {
    unsafe {
        word_transfers!(game, descending, 1920; native(0x4efe) <- value(6222));
        let v1: u16 = word!(game.vram_write_queue_tail);
        let v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v2).size = 3840;
        (*v2).src.addr = (ram_ptr!(game, Ram4000; 0x4000) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            as VoidP;
        *(&raw mut (*v2).src.bank as *mut u16) = 126;
        (*v2).vram_dst = addr_unk_605880 as u16;
        word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
    }
}
pub unsafe fn CalculateLayer2Xpos(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0x91b) == 0 {
            word!(game.layer2_x_pos = word!(game.layer1_x_pos));
            return 0;
        }
        if *ram_ptr!(game; 0x91b) != 1 {
            let t: i32 = *ram_ptr!(game; 0x91b) as i32 & 0xfe;
            let var933: u8 = (t * byte!(@pointer ram_ptr!(game, u16; 0x911); 0) as i32 >> 8) as u8;
            word!(
                game.layer2_x_pos = (t * *ram_ptr!(game; 0x911, 1) as i32 + var933 as i32) as u16
            );
            return 0;
        }
        1
    }
}
pub unsafe fn CalculateLayer2Ypos(game: &mut Game) -> u8 {
    unsafe {
        if *ram_ptr!(game; 0x91c) == 0 {
            word!(game.layer2_y_pos = word!(game.layer1_y_pos));
            return 0;
        }
        if *ram_ptr!(game; 0x91c) != 1 {
            let t: i32 = *ram_ptr!(game; 0x91c) as i32 & 0xfe;
            let var933: u8 = (t * word!(game.layer1_y_pos) as u8 as i32 >> 8) as u8;
            word!(
                game.layer2_y_pos = (t * *ram_ptr!(game; 0x915, 1) as i32 + var933 as i32) as u16
            );
            return 0;
        }
        1
    }
}
pub fn CalculateBgScrolls(game: &mut Game) {
    {
        word!(game.reg_BG1HOFS = (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos)));
        word!(game.reg_BG1VOFS = (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos)));
        word!(game.reg_BG2HOFS = (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos)));
        word!(game.reg_BG2VOFS = (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos)));
    }
}
pub unsafe fn UpdateScrollVarsUpdateMap(game: &mut Game) {
    unsafe {
        CalculateBgScrolls(game);
        UpdateBgGraphicsWhenScrolling(game);
    }
}
pub unsafe fn CalculateLayer2PosAndScrollsWhenScrolling(game: &mut Game) {
    unsafe {
        if word!(game.time_is_frozen_flag) == 0 {
            word!(
                game.reg_BG1HOFS =
                    (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos))
            );
            word!(
                game.reg_BG1VOFS =
                    (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos))
            );
            if CalculateLayer2Xpos(game) == 0 {
                word!(
                    game.reg_BG2HOFS =
                        (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos))
                );
            }
            if CalculateLayer2Ypos(game) == 0 {
                word!(
                    game.reg_BG2VOFS =
                        (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos))
                );
            }
            UpdateBgGraphicsWhenScrolling(game);
        }
    }
}
pub unsafe fn UpdateBgGraphicsWhenScrolling(game: &mut Game) {
    unsafe {
        CalculateBgScrollAndLayerPositionBlocks(game);
        let mut v0: i32 = 0;
        let v1: bool = ((word!(game.layer1_x_block)) as i16)
            .wrapping_sub((word!(game.previous_layer1_x_block)) as i16)
            < 0;
        if word!(game.layer1_x_block) != word!(game.previous_layer1_x_block) {
            word!(game.previous_layer1_x_block = word!(game.layer1_x_block));
            if !v1 {
                v0 = 16;
            }
            word!(game.blocks_to_update_x_block = (word!(game.layer1_x_block) as i32 + v0) as u16);
            word!(
                game.vram_blocks_to_update_x_block = (word!(game.bg1_x_block) as i32 + v0) as u16
            );
            word!(game.blocks_to_update_y_block = word!(game.layer1_y_block));
            word!(game.vram_blocks_to_update_y_block = word!(game.bg1_y_block));
            UploadLevelDataColumn(game);
        }
        if *ram_ptr!(game; 0x91b) & 1 == 0 {
            let mut v2: i32 = 0;
            let v3: bool = ((word!(game.layer2_x_block)) as i16)
                .wrapping_sub((word!(game.previous_layer2_x_block)) as i16)
                < 0;
            if word!(game.layer2_x_block) != word!(game.previous_layer2_x_block) {
                word!(game.previous_layer2_x_block = word!(game.layer2_x_block));
                if !v3 {
                    v2 = 16;
                }
                word!(
                    game.blocks_to_update_x_block = (word!(game.layer2_x_block) as i32 + v2) as u16
                );
                word!(
                    game.vram_blocks_to_update_x_block =
                        (word!(game.bg2_x_block) as i32 + v2) as u16
                );
                word!(game.blocks_to_update_y_block = word!(game.layer2_y_block));
                word!(game.vram_blocks_to_update_y_block = word!(game.bg2_y_block));
                UploadBackgroundDataColumn(game);
            }
        }
        let mut v4: i32 = 1;
        let v5: bool = ((word!(game.layer1_y_block)) as i16)
            .wrapping_sub((word!(game.previous_layer1_y_block)) as i16)
            < 0;
        if word!(game.layer1_y_block) != word!(game.previous_layer1_y_block) {
            word!(game.previous_layer1_y_block = word!(game.layer1_y_block));
            if !v5 {
                v4 = 15;
            }
            word!(game.blocks_to_update_y_block = (word!(game.layer1_y_block) as i32 + v4) as u16);
            word!(
                game.vram_blocks_to_update_y_block = (word!(game.bg1_y_block) as i32 + v4) as u16
            );
            word!(game.blocks_to_update_x_block = word!(game.layer1_x_block));
            word!(game.vram_blocks_to_update_x_block = word!(game.bg1_x_block));
            UpdateLevelDataRow(game);
        }
        if *ram_ptr!(game; 0x91c) & 1 == 0 {
            let mut v6: i32 = 1;
            let v7: bool = ((word!(game.layer2_y_block)) as i16)
                .wrapping_sub((word!(game.previous_layer2_y_block)) as i16)
                < 0;
            if word!(game.layer2_y_block) != word!(game.previous_layer2_y_block) {
                word!(game.previous_layer2_y_block = word!(game.layer2_y_block));
                if !v7 {
                    v6 = 15;
                }
                word!(
                    game.blocks_to_update_y_block = (word!(game.layer2_y_block) as i32 + v6) as u16
                );
                word!(
                    game.vram_blocks_to_update_y_block =
                        (word!(game.bg2_y_block) as i32 + v6) as u16
                );
                word!(game.blocks_to_update_x_block = word!(game.layer2_x_block));
                word!(game.vram_blocks_to_update_x_block = word!(game.bg2_x_block));
                UpdateBackgroundDataRow(game);
            }
        }
    }
}
pub unsafe fn CalculateBgScrollAndLayerPositionBlocks(game: &mut Game) {
    {
        crate::verify_probe::hit("camera.blocks");
        word! {game; bg1_x_block = word!(game.reg_BG1HOFS) >> 4;
        bg2_x_block = word!(game.reg_BG2HOFS) >> 4;
        layer1_x_block = ((word!(game.layer1_x_pos) as i16) >> 4) as u16;
        layer2_x_block = ((word!(game.layer2_x_pos) as i16) >> 4) as u16;
        bg1_y_block = word!(game.reg_BG1VOFS) >> 4; bg2_y_block = word!(game.reg_BG2VOFS) >> 4;
        layer1_y_block = ((word!(game.layer1_y_pos) as i16) >> 4) as u16;
        layer2_y_block = ((word!(game.layer2_y_pos) as i16) >> 4) as u16; }
    }
}
pub fn DebugScrollPosSaveLoad(game: &mut Game) {
    {
        if (word!(game.joypad2_new_keys) & 0x40) != 0 {
            word!(game.debug_saveload_scrollpos_toggle += 1);
        }
        if word!(game.debug_saveload_scrollpos_toggle) & 1 != 0 {
            word!(game.layer1_x_pos = word!(game.debug_saved_xscroll));
            word!(game.layer1_y_pos = word!(game.debug_saved_yscroll));
        } else {
            word!(game.debug_saved_xscroll = word!(game.layer1_x_pos));
            word!(game.debug_saved_yscroll = word!(game.layer1_y_pos));
        };
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    UploadBackgroundDataColumn => UpdateLevelOrBackgroundDataColumn(0x1c, game);
    UploadLevelDataColumn => UpdateLevelOrBackgroundDataColumn(0, game);
}
unsafe fn build_scrolling_tilemap(k: u16, layout: ScrollingDmaLayout, game: &mut Game) {
    unsafe {
        use crate::ram_map::word_arrays;
        if word!(game.irq_enable_mode7) != 0 {
            return;
        }
        let row = matches!(layout, ScrollingDmaLayout::Row);
        crate::verify_probe::hit(match (row, k != 0) {
            (false, false) => "scroll_tiles.column.bg1",
            (false, true) => "scroll_tiles.column.bg2",
            (true, false) => "scroll_tiles.row.bg1",
            (true, true) => "scroll_tiles.row.bg2",
        });
        let product = Mult8x8(
            word!(game.blocks_to_update_y_block) as u8,
            word!(game.room_width_in_blocks) as u8,
        );
        let mut source_offset = product
            .wrapping_add(word!(game.blocks_to_update_x_block))
            .wrapping_mul(2)
            .wrapping_add(2);
        if k != 0 {
            source_offset = source_offset.wrapping_add(0x9600);
        }
        let source = ram_ptr!(game, u16; 0x10000, source_offset as isize);
        let parameters = ram_ptr!(game, u16; if row { 0x964 } else { 0x956 }, k as isize);
        // Columns store the wrapped size first; rows store the unwrapped size first.
        if row {
            let within_screen = word!(game.vram_blocks_to_update_x_block) & 15;
            *parameters = 4 * (16 - within_screen);
            *parameters.add(1) = 4 * (within_screen + 1);
        } else {
            let wrapped = 4 * (word!(game.vram_blocks_to_update_y_block) & 15);
            *parameters.add(1) = wrapped;
            *parameters = (wrapped ^ 0x3f) + 1;
        }
        let tile_row = Mult8x8(word!(game.vram_blocks_to_update_y_block) as u8 & 15, 0x40);
        let tile_column = word!(game.vram_blocks_to_update_x_block) & 31;
        let origin = tile_row.wrapping_add(2 * tile_column);
        let mut screen = if tile_column >= 16 {
            addr_unk_6053E0 as u16
        } else {
            addr_unk_605000 as u16
        };
        if k != 0 {
            screen = screen.wrapping_sub(word!(game.size_of_bg2));
        }
        *parameters.add(2) = origin.wrapping_add(screen);
        *parameters.add(3) = if row {
            let mut wrapped_screen = if tile_column >= 16 {
                addr_unk_605000 as u16
            } else {
                addr_unk_605400 as u16
            };
            if k != 0 {
                wrapped_screen = wrapped_screen.wrapping_sub(word!(game.size_of_bg2));
            }
            tile_row.wrapping_add(wrapped_screen)
        } else {
            (2 * tile_column).wrapping_add(screen)
        };
        let buffer_base = if row {
            word_arrays::bg1_column_update_tilemap_top_halves
        } else {
            word_arrays::bg1_column_update_tilemap_left_halves
        };
        let buffer = buffer_base + if k != 0 { 264 } else { 0 };
        let half_bytes = if row { 68 } else { 64 };
        let wrapped_source = (*parameters).wrapping_add(buffer as u16);
        *parameters.add(4) = wrapped_source;
        *parameters.add(5) = wrapped_source.wrapping_add(half_bytes);
        let mut source_delta = 0u16;
        for block_index in 0..if row { 17 } else { 16 } {
            let block = *source.add((source_delta >> 1) as usize);
            let first = ram_ptr!(game, u16; buffer as isize).add(2 * block_index);
            let second = first.add(usize::from(half_bytes / 2));
            let destinations = if row {
                [first, first.add(1), second, second.add(1)]
            } else {
                [first, second, first.add(1), second.add(1)]
            };
            TileTables::draw_block(
                ram_ptr!(game; 0xa000).cast(),
                block,
                destinations,
                TileDrawOrder::Rows,
            );
            // The column stride is reread after drawing each block.
            source_delta = source_delta.wrapping_add(if row {
                2
            } else {
                word!(game.room_width_in_blocks).wrapping_mul(2)
            });
        }
        *parameters.add(6) = (*parameters.add(6)).wrapping_add(1);
    }
}

pub unsafe fn UpdateLevelOrBackgroundDataColumn(k: u16, game: &mut Game) {
    unsafe {
        build_scrolling_tilemap(k, ScrollingDmaLayout::Column, game);
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    UpdateBackgroundDataRow => UpdateLevelOrBackgroundDataRow(0x1c, game);
    UpdateLevelDataRow => UpdateLevelOrBackgroundDataRow(0, game);
}
pub unsafe fn UpdateLevelOrBackgroundDataRow(v0: u16, game: &mut Game) {
    unsafe {
        build_scrolling_tilemap(v0, ScrollingDmaLayout::Row, game);
    }
}
// The hardware adapter is constructed only by the unsafe VRAM entry point.
// The shared decoder and RAM output do not contain unsafe operations.
struct DecompressionVram<'a>(&'a mut Game);
impl decompression::Output for DecompressionVram<'_> {
    fn next_byte(&mut self) -> u8 {
        decompression::next_byte(self.0)
    }
    fn read(&mut self, position: i32) -> u8 {
        unsafe { ReadPpuByte(position as u16, self.0) }
    }
    fn write(&mut self, position: i32, value: u8, back_reference: bool) {
        unsafe {
            if back_reference {
                WriteRegWord(VMADDL as u16, (position >> 1) as u16, self.0);
            }
            WriteReg((VMDATAL as i32 + (position & 1)) as u16, value, self.0);
        }
    }
}
unsafe fn ReadPpuByte(addr: u16, game: &mut Game) -> u8 {
    unsafe {
        WriteRegWord(VMADDL as u16, addr >> 1, game);
        ReadRegWord(RDVRAML as u16, game);
        let data: u16 = ReadRegWord(RDVRAML as u16, game);
        (if addr & 1 != 0 {
            (data as i32 & 0xff00) >> 8
        } else {
            data as i32
        }) as u8
    }
}
pub unsafe fn DecompressToVRAM(src: u32, dst_addr: u16, game: &mut Game) {
    game.decompress_src = src;
    decompression::decode(i32::from(dst_addr), &mut DecompressionVram(game));
}
pub unsafe fn LoadFromLoadStation(game: &mut Game) {
    crate::game::room_entry::begin(
        &mut crate::game::session_runtime::Runtime::new(game),
        crate::game::room_entry::ArrivalRequest::Start(
            crate::game::room_entry::StartSite::CurrentLoadStation,
        ),
    );
}
pub unsafe fn SetElevatorsAsUsed(game: &mut Game) {
    unsafe {
        let v0 = RomPtr_80(
            (crate::rtl::rom_table_word!(game, 0x80cd46, word!(game.area_index) as isize) as i32
                + 4 * ((word!(game.elevator_door_properties_orientation) as i32 & 0xf) - 1i32))
                as u16,
            game,
        );
        crate::update_place! { reference fresh92 = *ram_ptr!(game; 0xd8f8, *v0 as isize) => (*fresh92) | (*v0.offset(1)); }
        crate::update_place! { reference fresh93 = *ram_ptr!(game; 0xd8f8, *v0.offset(2) as isize) => (*fresh93) | (*v0.offset(3)); }
    }
}
