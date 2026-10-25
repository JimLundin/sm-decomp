use crate::game::decompression::{self, decompress_to_ram};
use crate::game::music::HandleMusicQueue;
pub use crate::game::prelude::*;
use crate::game::room_state::load_tiles_and_palette;
use crate::game::save_slots;

pub use crate::game::constants::bank_60::*;

use crate::game::animated_tiles;
use crate::game::room_mechanisms::reload_graphics;
use crate::game::sm_8b::InitializeIoDisplayLogo_Async;
use crate::rtl::RtlApuUpload;

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct SamusTileAnimationTileDefs {
        pub src: LongPtr;
        pub part1_size, part2_size: u16;
    }

}

const RomPtr_80: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x80>;
const RomPtr_92: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x92>;
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
pub unsafe fn CopySuperMetroidString(game: &mut Game) {
    unsafe {
        game.g_sram.write_bytes(0x1fe0, b"supermetroid");
        RtlWriteSram(game);
    }
}
pub unsafe fn VerifySRAM(game: &mut Game) {
    unsafe {
        word!(game.num_demo_sets = 3);
        if save_slots::load(0, game) != 0
            && save_slots::load(1, game) != 0
            && save_slots::load(2, game) != 0
        {
            game.g_sram.write_bytes(0x1fe0, b"madadameyohn");
            RtlWriteSram(game);
        } else if game.g_sram.bytes(0x1fe0..0x1fec) == b"supermetroid" {
            word!(game.num_demo_sets = 4);
        }
    }
}
pub fn Multiply16x16(a: u16, j: u16, game: &mut Game) -> u32 {
    game.g_ram[0x5e9..0x5eb].copy_from_slice(&a.to_le_bytes());
    game.g_ram[0x5eb..0x5ed].copy_from_slice(&j.to_le_bytes());
    let result: u32 = (a as u32).wrapping_mul(j as u32);
    result
}
pub unsafe fn WaitForNMI_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        if *ram_ptr!(game; 0x782) != 0 {
            *ram_ptr!(game; 0x782) = 0;
            return 0 as CoroutineRet;
        }
        *ram_ptr!(game; 0x5b4) = 1;
        *ram_ptr!(game; 0x782) = 1;
        1 as CoroutineRet
    }
}
pub unsafe fn WaitForNMI_NoUpdate_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        if *ram_ptr!(game; 0x782) != 0 {
            *ram_ptr!(game; 0x782) = 0;
            return 0 as CoroutineRet;
        }
        *ram_ptr!(game; 0x782) = 1;
        1 as CoroutineRet
    }
}
pub unsafe fn EnableNMI(game: &mut Game) {
    unsafe {
        let v0: u8 = (*ram_ptr!(game; 0x84)) | (0x80u8);
        WriteReg(NMITIMEN as u16, (*ram_ptr!(game; 0x84)) | (0x80u8), game);
        *ram_ptr!(game; 0x84) = v0;
    }
}
pub unsafe fn DisableNMI(game: &mut Game) {
    unsafe {
        let v0: u8 = (*ram_ptr!(game; 0x84)) & (0x7fu8);
        WriteReg(NMITIMEN as u16, (*ram_ptr!(game; 0x84)) & (0x7fu8), game);
        *ram_ptr!(game; 0x84) = v0;
    }
}
pub unsafe fn ScreenOff(game: &mut Game) {
    unsafe {
        crate::update_place! { reference fresh89 = *ram_ptr!(game; 0x51) => (*fresh89) | (0x80u8); }
    }
}
pub unsafe fn ScreenOn(game: &mut Game) {
    unsafe {
        crate::update_place! { reference fresh90 = *ram_ptr!(game; 0x51) => *fresh90 & !(0x80); }
    }
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
pub unsafe fn Vector_RESET_Async(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::verify_probe::hit("coroutine.reset");
        let resume_state: *mut u8 = ram_ptr!(game; 0x611);
        // Callbacks may change RAM; dispatch uses the captured resume byte.
        let resume = *resume_state;
        let mut bak: u16 = 0;
        if resume == 1 {
            crate::rtl::write_register_and_mirror(MEMSEL as u16, 1, 0x86, false, game);
            bak = word!(game.bug_fix_counter);
            memset(
                &raw mut game.g_ram as *mut u8 as *mut c_void,
                0,
                8192 as size_t,
            );
            word!(game.bug_fix_counter = bak);
        }
        if matches!(resume, 1 | 2) {
            let yielded: u8 = InitializeIoDisplayLogo_Async(game);
            if yielded != 0 {
                *resume_state = 2;
                return yielded;
            }
        }
        if matches!(resume, 1..=3) {
            APU_UploadBank(0xcf8000, game);
            WriteReg(INIDISP as u16, 0x8f, game);
            bak = word!(game.bug_fix_counter);
            memset(
                &raw mut game.g_ram as *mut u8 as *mut c_void,
                0,
                0x10000i32 as size_t,
            );
            word!(game.bug_fix_counter = bak);
            crate::rtl::write_register_and_mirror(NMITIMEN as u16, 0, 0x84, false, game);
            *ram_ptr!(game; 0x51) = 0x8f;
            InitializeCpuIoRegs(game);
            InitializePpuIoRegs(game);
            WriteLotsOf0x1c2f(game);
            *ram_ptr!(game; 0x643, 0) = 0;
            *ram_ptr!(game; 0x643, 1) = 0;
            *ram_ptr!(game; 0x643, 2) = 0;
            *ram_ptr!(game; 0x646, 0) = 0;
            *ram_ptr!(game; 0x646, 1) = 0;
            *ram_ptr!(game; 0x646, 2) = 0;
            *ram_ptr!(game; 0x649, 0) = 0;
            *ram_ptr!(game; 0x649, 1) = 0;
            *ram_ptr!(game; 0x649, 2) = 0;
            *ram_ptr!(game; 0x64d, 0) = 0;
            *ram_ptr!(game; 0x64d, 1) = 0;
            *ram_ptr!(game; 0x64d, 2) = 0;
            *ram_ptr!(game; 0x650, 0) = 0;
            *ram_ptr!(game; 0x650, 1) = 0;
            *ram_ptr!(game; 0x650, 2) = 0;
            *ram_ptr!(game; 0x656, 0) = 0;
            *ram_ptr!(game; 0x666, 0) = 0;
            *ram_ptr!(game; 0x676, 0) = 0;
            word! { game, 0; oam_next_ptr, reg_OAMaddr_UNUSED }
            ClearOamExt(game);
            ClearUnusedOam(game);
            word! { game, 0;
                nmi_copy_samus_halves, nmi_copy_samus_top_half_src, nmi_copy_samus_bottom_half_src,
            }
            EnableNMI(game);
            RtlApuWrite(APUI00, 0, game);
            RtlApuWrite(APUI01, 0, game);
            RtlApuWrite(APUI02, 0, game);
            RtlApuWrite(APUI03, 0, game);
            word!(game.random_number = 97);
            crate::game::music::reset_delays(game);
            word!(game.enable_debug = 0);
            VerifySRAM(game);
            word! { game, 0; debug_disable_sounds, sound_handler_downtime }
        }
        *resume_state = 2;
        0 as CoroutineRet
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
pub unsafe fn InitializeCpuIoRegs(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            mirror(NMITIMEN as u16, 1, 0x84, false); byte(WRIO as u16, 0);
            byte(WRMPYA as u16, 0); byte(WRMPYB as u16, 0);
            byte(WRDIVL as u16, 0); byte(WRDIVH as u16, 0);
            byte(WRDIVB as u16, 0); byte(HTIMEL as u16, 0);
            byte(HTIMEH as u16, 0); byte(VTIMEL as u16, 0);
            byte(VTIMEH as u16, 0); byte(MDMAEN as u16, 0);
            mirror(HDMAEN as u16, 0, 0x85, false); mirror(MEMSEL as u16, 1, 0x86, false);
        }
    }
}
pub unsafe fn InitializePpuIoRegs(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            mirror(INIDISP as u16, 0x8f, 0x51, false); mirror(OBSEL as u16, 3, 0x52, false);
            byte(OAMADDL as u16, 0);
        }
        *(ram_ptr!(game, u16; 0x53) as *mut u8) = 0;
        WriteReg(OAMADDH as u16, 0x80, game);
        byte!(@pointer ram_ptr!(game, u16; 0x53); 1) = 0x80;
        crate::rtl::register_writes! { game;
            byte(OAMDATA as u16, 0); byte(OAMDATA as u16, 0);
            mirror(BGMODE as u16, 9, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
            mirror(BG1SC as u16, 0x40, 0x58, false); mirror(BG2SC as u16, 0x44, 0x59, false);
            mirror(BG3SC as u16, 0x48, 0x5a, false); mirror(BG4SC as u16, 0, 0x5c, false);
            mirror(BG12NBA as u16, 0, 0x5d, false); mirror(BG34NBA as u16, 5, 0x5e, false);
            byte(BG1HOFS as u16, 0); byte(BG1HOFS as u16, 0);
            byte(BG1VOFS as u16, 0); byte(BG1VOFS as u16, 0);
            byte(BG2HOFS as u16, 0); byte(BG2HOFS as u16, 0);
            byte(BG2VOFS as u16, 0); byte(BG2VOFS as u16, 0);
            byte(BG3HOFS as u16, 0); byte(BG3HOFS as u16, 0);
            byte(BG3VOFS as u16, 0); byte(BG3VOFS as u16, 0);
            byte(BG4HOFS as u16, 0); byte(BG4HOFS as u16, 0);
            byte(BG4VOFS as u16, 0); byte(BG4VOFS as u16, 0);
            byte(VMAIN as u16, 0); mirror(M7SEL as u16, 0, 0x5f, false);
            byte(M7A as u16, 0); byte(M7B as u16, 0);
            byte(M7C as u16, 0); byte(M7D as u16, 0);
            byte(M7X as u16, 0); byte(M7Y as u16, 0);
            mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
            mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
            byte(WH1 as u16, 0xf8);
        }
        *ram_ptr!(game; 0x64) = -8i32 as u8;
        crate::rtl::register_writes! { game;
            mirror(WH2 as u16, 0, 0x65, false); mirror(WH3 as u16, 0, 0x66, false);
            mirror(WBGLOG as u16, 0, 0x67, false); mirror(WOBJLOG as u16, 0, 0x68, false);
            mirror(TM as u16, 0x11, 0x69, false); mirror(TMW as u16, 0x11, 0x6c, false);
            mirror(TS as u16, 2, 0x6b, false); mirror(TSW as u16, 2, 0x6d, false);
            mirror(CGWSEL as u16, 2, 0x6e, false); byte(CGADSUB as u16, 0xa1);
        }
        *ram_ptr!(game; 0x71) = -95i32 as u8;
        crate::rtl::register_writes! { game;
            byte(COLDATA as u16, 0xe0); byte(COLDATA as u16, 0xe0);
            byte(COLDATA as u16, 0x80);
        }
        *ram_ptr!(game; 0x74, 0) = 0x80;
        WriteReg(COLDATA as u16, 0x40, game);
        *ram_ptr!(game; 0x74, 1) = 64;
        WriteReg(COLDATA as u16, 0x20, game);
        *ram_ptr!(game; 0x74, 2) = 32;
        crate::rtl::write_register_and_mirror(SETINI as u16, 0, 0x77, false, game);
    }
}
pub unsafe fn WriteLotsOf0x1c2f(game: &mut Game) {
    unsafe {
        sub_8088EB(0x1c2f, game);
        sub_8088FE(0x1c2f, game);
        sub_808911(0x1c2f, game);
    }
}
crate::forward_calls! { pub unsafe fn (a: u16, game: &mut Game);
    sub_8088EB => memset7E(ram_ptr!(game, u16; 0x3000), a, 0x800);
    sub_8088FE => memset7E(ram_ptr!(game, u16; 0x4000), a, 0x800);
    sub_808911 => memset7E(ram_ptr!(game, u16; 0x6000), a, 0x800);
}
fn tick_screen_fade(game: &mut Game) -> bool {
    let screen_fade_counter = u16::from_le_bytes([game.g_ram[0x725], game.g_ram[0x726]]);
    // Test the sign after the word decrement, then reload before changing brightness.
    let remaining = screen_fade_counter.wrapping_sub(1);
    let step_due = (remaining as i16) < 0;
    let next_counter = if step_due {
        let screen_fade_delay = u16::from_le_bytes([game.g_ram[0x723], game.g_ram[0x724]]);
        screen_fade_delay
    } else {
        remaining
    };
    game.g_ram[0x725..0x727].copy_from_slice(&next_counter.to_le_bytes());
    step_due
}

pub fn HandleFadeOut(game: &mut Game) {
    if !tick_screen_fade(game) {
        return;
    }
    let reg_inidisp = &mut game.g_ram[0x51];
    match *reg_inidisp & 0x0f {
        0 => {}
        1 => *reg_inidisp = 0x80,
        brightness => *reg_inidisp = brightness - 1,
    }
}

pub fn HandleFadeIn(game: &mut Game) {
    if !tick_screen_fade(game) {
        return;
    }
    let reg_inidisp = &mut game.g_ram[0x51];
    let brightness = reg_inidisp.wrapping_add(1) & 0x0f;
    if brightness != 0 {
        *reg_inidisp = brightness;
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
pub unsafe fn NMI_ProcessVramWriteQueue(game: &mut Game) {
    unsafe {
        let tail = game.vram_write_queue_tail();
        if tail != 0 {
            // The sentinel uses the byte tail, which need not lie on an entry boundary.
            game.write_vram_queue_word(tail, 0, 0);
            WriteRegWord(DMAP1 as u16, 0x1801, game);
            let read_word = |game: &Game, address: usize| {
                u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
            };
            // Entries are seven packed bytes; the consumer advances without bank wrap.
            let mut entry = 0xd0;
            loop {
                let size = read_word(game, entry);
                if size == 0 {
                    break;
                }
                crate::rtl::register_writes! { game;
                    word(DAS1L as u16, size); word(A1T1L as u16, read_word(game, entry + 2));
                    byte(A1B1 as u16, game.g_ram[entry + 4]);
                }
                let increment = if read_word(game, entry + 5) & 0x8000 != 0 {
                    0x81
                } else {
                    0x80
                };
                // This word write also clears VMADDL before the full destination write.
                crate::rtl::register_writes! { game;
                    word(VMAIN as u16, increment); word(VMADDL as u16, read_word(game, entry + 5));
                    byte(MDMAEN as u16, 2);
                }
                entry += 7;
            }
        }
        game.set_vram_write_queue_tail(0);
        Nmi_ProcessHorizScrollingDma(game);
        Nmi_ProcessVertScrollingDma(game);
    }
}
#[derive(Clone, Copy)]
enum ScrollingDmaLayout {
    Column,
    Row,
}

impl ScrollingDmaLayout {
    fn second_half_destination(self, address: u16) -> u16 {
        match self {
            Self::Column => address.wrapping_add(1),
            Self::Row => address | 0x20,
        }
    }
}

unsafe fn transfer_scrolling_tilemap(
    parameters: usize,
    initial_sources: [u16; 2],
    layout: ScrollingDmaLayout,
    game: &mut Game,
) {
    unsafe {
        let enable = parameters + 12;
        if game.g_ram[enable] == 0 {
            return;
        }
        // Only the low enable byte is consumed, before reading transfer parameters.
        game.g_ram[enable] = 0;
        // Sizes, destinations and wrapped sources are three pairs of RAM words.
        let read_parameter = |game: &Game, offset: usize| {
            u16::from_le_bytes([
                game.g_ram[parameters + offset],
                game.g_ram[parameters + offset + 1],
            ])
        };

        let destination = read_parameter(game, 4);
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, destination); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, initial_sources[0]); byte(A1B1 as u16, 0x7e);
        }
        let size = read_parameter(game, 0);
        // A zero unwrapped size still triggers a full 65,536-byte DMA.
        crate::rtl::register_writes! { game;
            word(DAS1L as u16, size); byte(MDMAEN as u16, 2);
            word(VMADDL as u16,
            layout.second_half_destination(destination),); word(DAS1L as u16, size);
            word(A1T1L as u16, initial_sources[1]); byte(MDMAEN as u16, 2);
        }

        // Prepare the wrapped source even when there is no wrapped transfer.
        WriteRegWord(A1T1L as u16, read_parameter(game, 8), game);
        let wrapped_size = read_parameter(game, 2);
        if wrapped_size != 0 {
            WriteRegWord(DAS1L as u16, wrapped_size, game);
            let wrapped_destination = read_parameter(game, 6);
            crate::rtl::register_writes! { game;
                word(VMADDL as u16, wrapped_destination); byte(MDMAEN as u16, 2);
                word(VMADDL as u16,
                layout.second_half_destination(wrapped_destination),); word(DAS1L as u16, wrapped_size);
                word(A1T1L as u16, read_parameter(game, 10)); byte(MDMAEN as u16, 2);
            }
        }
    }
}

pub unsafe fn Nmi_ProcessHorizScrollingDma(game: &mut Game) {
    unsafe {
        WriteReg(VMAIN as u16, 0x81, game);
        let layout = ScrollingDmaLayout::Column;
        transfer_scrolling_tilemap(0x956, [0xc8c8, 0xc908], layout, game);
        transfer_scrolling_tilemap(0x972, [0xc9d0, 0xca10], layout, game);
    }
}

pub unsafe fn Nmi_ProcessVertScrollingDma(game: &mut Game) {
    unsafe {
        WriteReg(VMAIN as u16, 0x80, game);
        let layout = ScrollingDmaLayout::Row;
        transfer_scrolling_tilemap(0x964, [0xc948, 0xc98c], layout, game);
        transfer_scrolling_tilemap(0x980, [0xca50, 0xca94], layout, game);
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

pub unsafe fn NmiUpdateIoRegisters(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            byte(NMITIMEN as u16, game.g_ram[0x84]); byte(INIDISP as u16, game.g_ram[0x51]);
            byte(OBSEL as u16, game.g_ram[0x52]); byte(BGMODE as u16, game.g_ram[0x55]);
            byte(MOSAIC as u16, game.g_ram[0x57]); byte(BG1SC as u16, game.g_ram[0x58]);
            byte(BG2SC as u16, game.g_ram[0x59]); byte(BG3SC as u16, game.g_ram[0x5a]);
            byte(BG4SC as u16, game.g_ram[0x5c]); byte(BG12NBA as u16, game.g_ram[0x5d]);
            byte(BG34NBA as u16, game.g_ram[0x5e]); byte(M7SEL as u16, game.g_ram[0x5f]);
            byte(W12SEL as u16, game.g_ram[0x60]); byte(W34SEL as u16, game.g_ram[0x61]);
            byte(WOBJSEL as u16, game.g_ram[0x62]); byte(WH0 as u16, game.g_ram[0x63]);
            byte(WH1 as u16, game.g_ram[0x64]); byte(WH2 as u16, game.g_ram[0x65]);
            byte(WH3 as u16, game.g_ram[0x66]); byte(WBGLOG as u16, game.g_ram[0x67]);
            byte(WOBJLOG as u16, game.g_ram[0x68]);
        }
        game.g_ram[0x6a] = game.g_ram[0x69];
        crate::rtl::register_writes! { game;
            byte(TM as u16, game.g_ram[0x69]); byte(TMW as u16, game.g_ram[0x6c]);
            byte(TS as u16, game.g_ram[0x6b]); byte(TSW as u16, game.g_ram[0x6d]);
            byte(CGWSEL as u16, game.g_ram[0x6f]); byte(CGADSUB as u16, game.g_ram[0x72]);
        }
        game.g_ram[0x70] = game.g_ram[0x6e];
        game.g_ram[0x73] = game.g_ram[0x71];
        crate::rtl::register_writes! { game;
            byte(COLDATA as u16, game.g_ram[0x74]); byte(COLDATA as u16, game.g_ram[0x75]);
            byte(COLDATA as u16, game.g_ram[0x76]); byte(SETINI as u16, game.g_ram[0x77]);
        }

        // Scroll and Mode 7 words latch low then high bytes through the same address.
        crate::rtl::register_writes! { game;
            byte(BG1HOFS as u16, game.g_ram[0xb1]); byte(BG1HOFS as u16, game.g_ram[0xb2]);
            byte(BG1VOFS as u16, game.g_ram[0xb3]); byte(BG1VOFS as u16, game.g_ram[0xb4]);
            byte(BG2HOFS as u16, game.g_ram[0xb5]); byte(BG2HOFS as u16, game.g_ram[0xb6]);
            byte(BG2VOFS as u16, game.g_ram[0xb7]); byte(BG2VOFS as u16, game.g_ram[0xb8]);
            byte(BG3HOFS as u16, game.g_ram[0xb9]); byte(BG3HOFS as u16, game.g_ram[0xba]);
            byte(BG3VOFS as u16, game.g_ram[0xbb]); byte(BG3VOFS as u16, game.g_ram[0xbc]);
            byte(BG4HOFS as u16, game.g_ram[0xbd]); byte(BG4HOFS as u16, game.g_ram[0xbe]);
            byte(BG4VOFS as u16, game.g_ram[0xbf]); byte(BG4VOFS as u16, game.g_ram[0xc0]);
        }
        game.g_ram[0x7ec] = game.g_ram[0x56];
        if game.g_ram[0x55] & 7 == 7 || game.g_ram[0x56] & 7 == 7 {
            crate::rtl::register_writes! { game;
                byte(M7A as u16, game.g_ram[0x78]); byte(M7A as u16, game.g_ram[0x79]);
                byte(M7B as u16, game.g_ram[0x7a]); byte(M7B as u16, game.g_ram[0x7b]);
                byte(M7C as u16, game.g_ram[0x7c]); byte(M7C as u16, game.g_ram[0x7d]);
                byte(M7D as u16, game.g_ram[0x7e]); byte(M7D as u16, game.g_ram[0x7f]);
                byte(M7X as u16, game.g_ram[0x80]); byte(M7X as u16, game.g_ram[0x81]);
                byte(M7Y as u16, game.g_ram[0x82]); byte(M7Y as u16, game.g_ram[0x83]);
            }
        }
    }
}
pub unsafe fn NmiUpdatePalettesAndOam(game: &mut Game) {
    unsafe {
        // Channel 0 uploads the OAM entries and their high table from RAM.
        crate::rtl::register_writes! { game;
            word(DMAP0 as u16, 0x0400); word(A1T0L as u16, 0x0370);
            byte(A1B0 as u16, 0); word(DAS0L as u16, 0x0220);
            word(OAMADDL as u16, 0);
        }

        // Channel 1 uploads palette_buffer to CGRAM.
        crate::rtl::register_writes! { game;
            word(DMAP1 as u16, 0x2200); word(A1T1L as u16, 0xc000);
            byte(A1B1 as u16, 0x7e); word(DAS1L as u16, 0x0200);
            byte(CGADD as u16, 0); byte(MDMAEN as u16, 3);
        }
        crate::verify_probe::hit("palette.dma.uploaded");
    }
}
unsafe fn transfer_samus_tile_half(descriptor_addr: u16, vram_addr: u16, game: &mut Game) {
    unsafe {
        // Keep the diagnostic and failure flag for low-address ROM mirrors.
        let _ = RomPtr_92(descriptor_addr, game);
        let descriptor = 0x928000 | u32::from(descriptor_addr & 0x7fff);
        // Packed fields: source address at 0, bank at 2, and byte counts at 3 and 5.
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, vram_addr); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, game.read_rom_word(descriptor, 0)); byte(A1B1 as u16, game.read_rom_byte(descriptor, 2));
        }
        // A zero first count still runs a 65,536-byte DMA.
        crate::rtl::register_writes! { game;
            word(DAS1L as u16, game.read_rom_word(descriptor, 3)); byte(MDMAEN as u16, 2);
        }

        // Prepare the second destination/source even when its count is zero.
        WriteRegWord(VMADDL as u16, vram_addr.wrapping_add(0x100), game);
        let second_source = game
            .read_rom_word(descriptor, 0)
            .wrapping_add(game.read_rom_word(descriptor, 3));
        WriteRegWord(A1T1L as u16, second_source, game);
        if game.read_rom_word(descriptor, 5) != 0 {
            crate::rtl::register_writes! { game;
                word(DAS1L as u16, game.read_rom_word(descriptor, 5)); byte(MDMAEN as u16, 2);
            }
        }
    }
}

pub unsafe fn NmiTransferSamusToVram(game: &mut Game) {
    unsafe {
        WriteReg(VMAIN as u16, 0x80, game);
        if game.g_ram[0x71d] != 0 {
            let top_half_src = u16::from_le_bytes([game.g_ram[0x71f], game.g_ram[0x720]]);
            transfer_samus_tile_half(top_half_src, 0x6000, game);
        }
        if game.g_ram[0x71e] != 0 {
            let bottom_half_src = u16::from_le_bytes([game.g_ram[0x721], game.g_ram[0x722]]);
            transfer_samus_tile_half(bottom_half_src, 0x6080, game);
        }
    }
}
pub unsafe fn Vector_NMI(game: &mut Game) {
    unsafe {
        const BG_MODE: usize = 0x55;
        const BG_MODE_FAKE: usize = 0x56;
        const HDMA_ENABLE: usize = 0x85;
        const WAITING_FOR_NMI: usize = 0x5b4;
        const FRAME_COUNTER_BYTE: usize = 0x5b5;
        const FRAME_COUNTER_WORD: usize = 0x5b6;
        const FRAME_COUNTER_INCLUDING_LAG: usize = 0x5b8;
        const FRAMES_MISSED: usize = 0x5ba;
        const FRAMES_MISSED_MAX: usize = 0x5bb;

        if game.g_ram[WAITING_FOR_NMI] != 0 {
            NmiUpdatePalettesAndOam(game);
            NmiTransferSamusToVram(game);
            animated_tiles::upload_pending(game);
            NmiUpdateIoRegisters(game);
            for slot in 0..6 {
                let offset = slot * 2;
                let channels =
                    u16::from_le_bytes([game.g_ram[0x18b4 + offset], game.g_ram[0x18b5 + offset]]);
                if channels != 0 {
                    // The bank-slot word stores the DMA register offset in its low byte.
                    let bank_slot = u16::from(game.g_ram[0x18c0 + offset]);
                    let table_pointer = u16::from_le_bytes([
                        game.g_ram[0x18d8 + offset],
                        game.g_ram[0x18d9 + offset],
                    ]);
                    WriteRegWord(A1T0L as u16 + bank_slot, table_pointer, game);
                }
            }
            if game.g_ram[BG_MODE] == 7 || game.g_ram[BG_MODE_FAKE] == 7 {
                crate::game::mode7_transfers::process(game);
            }
            NMI_ProcessVramWriteQueue(game);
            crate::game::vram_read_queue::process(game);
            WriteReg(HDMAEN as u16, game.g_ram[HDMA_ENABLE], game);
            game.g_ram[WAITING_FOR_NMI] = 0;
            game.g_ram[FRAMES_MISSED] = 0;
            game.g_ram[FRAME_COUNTER_BYTE] = game.g_ram[FRAME_COUNTER_BYTE].wrapping_add(1);
            let frame_counter = u16::from_le_bytes([
                game.g_ram[FRAME_COUNTER_WORD],
                game.g_ram[FRAME_COUNTER_WORD + 1],
            ])
            .wrapping_add(1);
            game.g_ram[FRAME_COUNTER_WORD..FRAME_COUNTER_WORD + 2]
                .copy_from_slice(&frame_counter.to_le_bytes());
        } else {
            game.g_ram[FRAMES_MISSED] = game.g_ram[FRAMES_MISSED].wrapping_add(1);
            if game.g_ram[FRAMES_MISSED] >= game.g_ram[FRAMES_MISSED_MAX] {
                game.g_ram[FRAMES_MISSED_MAX] = game.g_ram[FRAMES_MISSED];
            }
        }
        let frame_counter = u16::from_le_bytes([
            game.g_ram[FRAME_COUNTER_INCLUDING_LAG],
            game.g_ram[FRAME_COUNTER_INCLUDING_LAG + 1],
        ])
        .wrapping_add(1);
        game.g_ram[FRAME_COUNTER_INCLUDING_LAG..FRAME_COUNTER_INCLUDING_LAG + 2]
            .copy_from_slice(&frame_counter.to_le_bytes());
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
            EnableNMI(game);
            word!(game.my_counter = 0);
        }
        if resume <= 1 {
            let mut waiting = resume == 1;
            loop {
                if !waiting {
                    if word!(game.my_counter) == 20 {
                        DisableNMI(game);
                        break;
                    }
                    HandleMusicQueue(game);
                }
                let yielded: u8 = WaitForNMI_Async(game);
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
        DisableNMI(game);
        crate::game::scanline_irq::disable(game);
        load_tiles_and_palette(game);
        crate::game::room_entry::background::load(
            &mut crate::game::session_runtime::Runtime::new(game),
            crate::game::room_entry::content::LoadMode::Initial,
        );
        DisplayViewablePartOfRoom(game);
        reload_graphics(game);
        EnableNMI(game);
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
            NMI_ProcessVramWriteQueue(game);
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
