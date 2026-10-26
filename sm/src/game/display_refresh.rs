//! Ordered display refresh and the NMI request/completion handshake.
use crate::game::constants::bank_60::{
    addr_unk_6053E0, addr_unk_605000, addr_unk_605400, addr_unk_605880,
};
use crate::game::decompression::{self, decompress_to_ram};
use crate::game::{animated_tiles, mode7_transfers, vram_read_queue};
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word};
use crate::rtl::{
    A1B0, A1B1, A1T0L, A1T1L, BG1HOFS, BG1SC, BG2SC, BG3SC, BG4SC, BG12NBA, BG34NBA, BGMODE, CGADD,
    CGADSUB, CGWSEL, COLDATA, DAS0L, DAS1L, DMAP0, DMAP1, HDMAEN, INIDISP, M7A, M7SEL, MDMAEN,
    MOSAIC, Mult8x8, NMITIMEN, OAMADDL, OBSEL, RDVRAML, ReadRegWord, SETINI, TM, TMW, TS, TSW,
    VMADDH, VMADDL, VMAIN, VMDATAL, W12SEL, W34SEL, WBGLOG, WH0, WH1, WH2, WH3, WMADDH, WMADDL,
    WMADDM, WOBJLOG, WOBJSEL, WriteReg, WriteRegWord,
};

/// Set the forced-blank mirror bit for the next display refresh.
pub(crate) fn set_blank(blank: bool, game: &mut Game) {
    game.g_ram[0x51] = game.g_ram[0x51] & 0x7f | if blank { 0x80 } else { 0 };
}

/// Requires a live, exclusively accessed SNES backend for ordered PPU writes.
pub(crate) unsafe fn pause_graphics(paused: bool, game: &mut Game) {
    let (objects, bg1, bg2, bg3) = if paused {
        (1, 0x31, 0x38, 0x58)
    } else {
        (3, 0x51, 0x49, 0x5a)
    };
    unsafe {
        crate::rtl::register_writes! { game;
            mirror(OBSEL as u16, objects, 0x52, false); mirror(BGMODE as u16, 9, 0x55, false);
            mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
            mirror(BG1SC as u16, bg1, 0x58, true); mirror(BG2SC as u16, bg2, 0x59, true);
            mirror(BG3SC as u16, bg3, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
        }
        if paused {
            crate::rtl::register_writes! { game;
                mirror(TM as u16, 0x17, 0x69, false); mirror(MOSAIC as u16, 0, 0x57, false);
            }
            for offset in 0x74..0x77 {
                game.g_ram[offset] &= 0xe0;
            }
            game.g_ram[0x71] = 0;
        }
    }
}

/// Requires live, exclusively accessed hardware for the ordered CPU I/O and OAM reset.
pub(crate) unsafe fn initialize_io_and_oam(brightness: Option<u8>, game: &mut Game) {
    use crate::snes::registers::{
        HTIMEH, HTIMEL, MEMSEL, OAMADDH, OAMDATA, VTIMEH, VTIMEL, WRDIVB, WRDIVH, WRDIVL, WRIO,
        WRMPYA, WRMPYB,
    };
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
        if let Some(value) = brightness {
            crate::rtl::write_register_and_mirror(INIDISP as u16, value, 0x51, false, game);
        }
        crate::rtl::register_writes! { game;
            mirror(OBSEL as u16, 3, 0x52, false);
            mirror(OAMADDL as u16, 0, 0x53, false);
            mirror(OAMADDH as u16, 0x80, 0x54, false);
            byte(OAMDATA as u16, 0); byte(OAMDATA as u16, 0);
        }
    }
}

/// Requires a live, exclusively accessed SNES backend and prepared cartridge graphics.
pub(crate) unsafe fn initialize_gameplay(game: &mut Game) {
    unsafe {
        initialize_io_and_oam(Some(0x80), game);
        crate::rtl::register_writes! { game;
            mirror(BGMODE as u16, 9, 0x55, false); mirror(MOSAIC as u16, 0, 0x57, false);
            mirror(BG12NBA as u16, 0, 0x5d, true); mirror(BG34NBA as u16, 4, 0x5e, true);
            mirror(BG1SC as u16, 0x51, 0x58, true); mirror(BG2SC as u16, 0x49, 0x59, true);
            mirror(BG3SC as u16, 0x5a, 0x5a, true); mirror(BG4SC as u16, 0, 0x5c, true);
            byte(VMAIN as u16, 0); mirror(W12SEL as u16, 0, 0x60, false);
            mirror(W12SEL as u16, 0, 0x60, false); mirror(W34SEL as u16, 0, 0x61, false);
            mirror(WOBJSEL as u16, 0, 0x62, false); mirror(WH0 as u16, 0, 0x63, false);
            mirror(WH1 as u16, 0, 0x64, false); mirror(WH2 as u16, 0, 0x65, false);
            mirror(WH3 as u16, 0, 0x66, false); mirror(WBGLOG as u16, 0, 0x67, false);
            mirror(WOBJLOG as u16, 0, 0x68, false); mirror(TM as u16, 0x17, 0x69, false);
            mirror(TMW as u16, 0, 0x6c, false); mirror(TS as u16, 0, 0x6b, false);
            mirror(TSW as u16, 0, 0x6d, false); mirror(CGWSEL as u16, 0, 0x6e, false);
            mirror(CGADSUB as u16, 0, 0x71, false); byte(COLDATA as u16, 0xe0);
            mirror(SETINI as u16, 0, 0x77, false);
        }
        word!(game.oam_next_ptr = 0);
        word_transfers!(game, ascending, 1023; native(0x3000) <- value(0));
        word_transfers!(game, ascending, 1023; native(0x4000) <- value(0x6f));
        word_transfers!(game, ascending, 127; native(0x4000) <- value(0x2c0f));
        GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x9ad200, 0x6000, 0x2e00).load(game);
        GraphicsAsset::VramWords(0x7e4000, 0x5000, 0x1000).load(game);
        GraphicsAsset::VramWords(0x7e4000, 0x5800, 0x800).load(game);
        word_transfers!(game, ascending, 256; le(palette_buffer, 0) <- rom(0x9a8000, 0));
    }
}

/// Requires a live, exclusively accessed SNES backend for VRAM readback.
pub(crate) unsafe fn capture_pause_background(game: &mut Game) {
    use crate::snes::registers::{A2A1L, BBAD1, DAS10, NTRL1};
    unsafe {
        crate::rtl::register_writes! { game;
            word(VMADDL as u16,
            (u16::from(game.g_ram[0x59] & 0xfc) << 8) | 1,); byte(VMAIN as u16, 0x80);
            byte(DMAP1 as u16, 0x81); byte(BBAD1 as u16, 0x39);
            word(A1T1L as u16, 0xdf5c); byte(A1B1 as u16, 0x7e);
            word(DAS1L as u16, 0x1000); byte(DAS10 as u16, 0);
            word(A2A1L as u16, 0); byte(NTRL1 as u16, 0);
            byte(MDMAEN as u16, 2);
        }
    }
}

/// Requires a live, exclusively accessed SNES backend and pause graphics assets.
pub(crate) unsafe fn load_pause_assets(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(0xb68000, 0, 0x4000).load(game);
        GraphicsAsset::VramWords(0xb6c000, 0x2000, 0x2000).load(game);
        GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x2000).load(game);
        GraphicsAsset::VramWords(0xb6e000, 0x3800, 0x800).load(game);
        GraphicsAsset::WramBytes(0xb6e400, 0x7e3400, 0x400).load(game);
        GraphicsAsset::WramBytes(0xb6e800, 0x7e3800, 0x800).load(game);
        crate::game::menu_tilemap::prepare_equipment_frame(game);
    }
}

/// Requires a live, exclusively accessed SNES backend throughout map and label upload.
pub(crate) unsafe fn upload_pause_map(game: &mut Game) {
    use crate::snes::registers::{BBAD1, DAS1H};
    unsafe {
        word! {game; reg_BG1HOFS = word!(game.reg_BG4HOFS); reg_BG1VOFS = word!(game.reg_BG4VOFS); }
        crate::rtl::register_writes! { game;
            byte(VMADDL as u16, 0); byte(VMADDH as u16, 0x30);
            byte(VMAIN as u16, 0x80);
        }
        crate::game::map_menu::render_map(crate::game::map_menu::MapScreen::Pause, game);
        setup_transfer(u16::from_ne_bytes([1, 0x18]), 0x7e4000, 0x1000, game);
        crate::rtl::register_writes! { game;
            byte(MDMAEN as u16, 2); byte(VMADDL as u16, 0xaa);
            byte(VMADDH as u16, 0x38); byte(VMAIN as u16, 0x80);
            byte(DMAP1 as u16, 1); byte(BBAD1 as u16, 0x18);
            byte(DAS1L as u16, 0x18); byte(DAS1H as u16, 0);
        }
        let mut area: u16 = word!(game.area_index);
        if word!(game.area_index) as i32 - 7i32 & 0x8000 == 0 {
            area = 0;
        }
        crate::rtl::register_writes! { game;
            word(A1T1L as u16,
            crate::rtl::rom_table_word!(game, 0x82965f, area as isize),); byte(A1B1 as u16, 0x82);
            byte(MDMAEN as u16, 2);
        }
    }
}

/// Requires a live, exclusively accessed backend for DMA register writes.
pub(crate) unsafe fn finish_gameplay(clear_current_irq: bool, game: &mut Game) {
    unsafe {
        crate::game::visual_runtime::clear_hdma(game);
    }
    crate::game::scanline_irq::disable(game);
    reset_gameplay_layers(clear_current_irq, game);
}

pub(crate) fn reset_gameplay_layers(clear_current_irq: bool, game: &mut Game) {
    word!(game.fx_layer_blending_config_a = 0);
    if clear_current_irq {
        word!(game.cur_irq_handler = 0);
    }
    reset_layer_mirrors(game);
    game.g_ram[0x55] = 9;
}

pub(crate) fn reset_layer_mirrors(game: &mut Game) {
    game.g_ram[0x6e] = 0;
    game.g_ram[0x71] = 0;
    game.g_ram[0x69] = 16;
    game.g_ram[0x6b..0x6e].fill(0);
}

pub(crate) fn fade(fade_in: bool, game: &mut Game) {
    let remaining = word!(game.screen_fade_counter).wrapping_sub(1);
    let due = (remaining as i16) < 0;
    word!(
        game.screen_fade_counter = if due {
            word!(game.screen_fade_delay)
        } else {
            remaining
        }
    );
    if !due {
        return;
    }
    let brightness = game.g_ram[0x51] & 0xf;
    let next = if fade_in {
        if brightness == 15 {
            return;
        }
        brightness + 1
    } else {
        match brightness {
            0 => return,
            1 => 0x80,
            value => value - 1,
        }
    };
    game.g_ram[0x51] = next;
}

pub(crate) fn wait(update: bool, game: &mut Game) -> u8 {
    if game.g_ram[0x782] != 0 {
        game.g_ram[0x782] = 0;
        return 0;
    }
    if update {
        game.g_ram[0x5b4] = 1;
    }
    game.g_ram[0x782] = 1;
    1
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn enable(enabled: bool, game: &mut Game) {
    let mask = if enabled { 0x80 } else { 0 };
    let value = game.g_ram[0x84] & 0x7f | mask;
    unsafe { WriteReg(NMITIMEN as u16, game.g_ram[0x84] & 0x7f | mask, game) };
    // The hardware write may change RAM; publish the value captured before it.
    game.g_ram[0x84] = value;
}

/// Requires a live, exclusively accessed SNES backend throughout the queue drain.
pub(crate) unsafe fn flush_tilemaps(game: &mut Game) {
    unsafe {
        let tail = game.vram_write_queue_tail();
        if tail != 0 {
            // The sentinel uses the byte tail, which need not lie on an entry boundary.
            game.write_vram_queue_word(tail, 0, 0);
            WriteRegWord(DMAP1 as u16, 0x1801, game);
            // Entries are seven packed bytes; the consumer advances without bank wrap.
            let mut entry = 0xd0;
            loop {
                let size = at!(game, entry;);
                if size == 0 {
                    break;
                }
                crate::rtl::register_writes! { game;
                    word(DAS1L as u16, size); word(A1T1L as u16, at!(game, entry + 2;));
                    byte(A1B1 as u16, game.g_ram[entry + 4]);
                }
                let increment = if at!(game, entry + 5;) & 0x8000 != 0 {
                    0x81
                } else {
                    0x80
                };
                // This word write also clears VMADDL before the full destination write.
                crate::rtl::register_writes! { game;
                    word(VMAIN as u16, increment); word(VMADDL as u16, at!(game, entry + 5;));
                    byte(MDMAEN as u16, 2);
                }
                entry += 7;
            }
        }
        game.set_vram_write_queue_tail(0);
        for (increment, layout, buffers) in [
            (
                0x81,
                ScrollingDmaLayout::Column,
                [(0x956, [0xc8c8, 0xc908]), (0x972, [0xc9d0, 0xca10])],
            ),
            (
                0x80,
                ScrollingDmaLayout::Row,
                [(0x964, [0xc948, 0xc98c]), (0x980, [0xca50, 0xca94])],
            ),
        ] {
            WriteReg(VMAIN as u16, increment, game);
            for (parameters, sources) in buffers {
                transfer_scrolling_tilemap(parameters, sources, layout, game);
            }
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum ScrollingDmaLayout {
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
        let destination = at!(game, parameters + 4;);
        crate::rtl::register_writes! { game;
            word(VMADDL as u16, destination); word(DMAP1 as u16, 0x1801);
            word(A1T1L as u16, initial_sources[0]); byte(A1B1 as u16, 0x7e);
        }
        let size = at!(game, parameters + 0;);
        // A zero unwrapped size still triggers a full 65,536-byte DMA.
        crate::rtl::register_writes! { game;
            word(DAS1L as u16, size); byte(MDMAEN as u16, 2);
            word(VMADDL as u16,
            layout.second_half_destination(destination),); word(DAS1L as u16, size);
            word(A1T1L as u16, initial_sources[1]); byte(MDMAEN as u16, 2);
        }

        // Prepare the wrapped source even when there is no wrapped transfer.
        WriteRegWord(A1T1L as u16, at!(game, parameters + 8;), game);
        let wrapped_size = at!(game, parameters + 2;);
        if wrapped_size != 0 {
            WriteRegWord(DAS1L as u16, wrapped_size, game);
            let wrapped_destination = at!(game, parameters + 6;);
            crate::rtl::register_writes! { game;
                word(VMADDL as u16, wrapped_destination); byte(MDMAEN as u16, 2);
                word(VMADDL as u16,
                layout.second_half_destination(wrapped_destination),); word(DAS1L as u16, wrapped_size);
                word(A1T1L as u16, at!(game, parameters + 10;)); byte(MDMAEN as u16, 2);
            }
        }
    }
}

unsafe fn upload_latches(register: u16, source: usize, count: u16, game: &mut Game) {
    unsafe {
        for index in 0..count {
            let address = register + index;
            let offset = source + usize::from(index) * 2;
            WriteReg(address, game.g_ram[offset], game);
            WriteReg(address, game.g_ram[offset + 1], game);
        }
    }
}
unsafe fn upload_registers(game: &mut Game) {
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

        // Scroll and Mode 7 words latch low then high through the same register.
        upload_latches(BG1HOFS as u16, 0xb1, 8, game);
        game.g_ram[0x7ec] = game.g_ram[0x56];
        if game.g_ram[0x55] & 7 == 7 || game.g_ram[0x56] & 7 == 7 {
            upload_latches(M7A as u16, 0x78, 6, game);
        }
    }
}
unsafe fn upload_palette_and_oam(game: &mut Game) {
    unsafe {
        // Channel 0 uploads the OAM entries and their high table from RAM.
        crate::rtl::register_writes! { game;
            word(DMAP0 as u16, 0x0400); word(A1T0L as u16, 0x0370);
            byte(A1B0 as u16, 0); word(DAS0L as u16, 0x0220);
            word(OAMADDL as u16, 0);
        }

        // Channel 1 uploads palette_buffer to CGRAM.
        setup_transfer(0x2200, 0x7ec000, 0x0200, game);
        crate::rtl::register_writes! { game;
            byte(CGADD as u16, 0); byte(MDMAEN as u16, 3);
        }
        crate::verify_probe::hit("palette.dma.uploaded");
    }
}
unsafe fn transfer_samus_tile_half(descriptor_addr: u16, vram_addr: u16, game: &mut Game) {
    unsafe {
        // Keep the diagnostic and failure flag for low-address ROM mirrors.
        let _ = crate::rtl::rom_bank_ptr::<0x92>(descriptor_addr, game);
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

unsafe fn upload_samus(game: &mut Game) {
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
/// Requires a live, exclusively accessed SNES backend for every upload.
pub(crate) unsafe fn run(game: &mut Game) {
    unsafe {
        const BG_MODE: usize = 0x55;
        const BG_MODE_FAKE: usize = 0x56;
        const HDMA_ENABLE: usize = 0x85;
        const WAITING_FOR_NMI: usize = 0x5b4;
        const FRAME_COUNTER_BYTE: usize = 0x5b5;
        const FRAMES_MISSED: usize = 0x5ba;
        const FRAMES_MISSED_MAX: usize = 0x5bb;

        if game.g_ram[WAITING_FOR_NMI] != 0 {
            upload_palette_and_oam(game);
            upload_samus(game);
            animated_tiles::upload_pending(game);
            upload_registers(game);
            for slot in 0..6 {
                if word!(game.hdma_object_channels_bitmask[slot]) != 0 {
                    // The bank-slot word stores the DMA register offset in its low byte.
                    let bank_slot = u16::from(game.g_ram[0x18c0 + slot * 2]);
                    let table_pointer = word!(game.hdma_object_table_pointers[slot]);
                    WriteRegWord(A1T0L as u16 + bank_slot, table_pointer, game);
                }
            }
            if game.g_ram[BG_MODE] == 7 || game.g_ram[BG_MODE_FAKE] == 7 {
                mode7_transfers::process(game);
            }
            flush_tilemaps(game);
            vram_read_queue::process(game);
            WriteReg(HDMAEN as u16, game.g_ram[HDMA_ENABLE], game);
            game.g_ram[WAITING_FOR_NMI] = 0;
            game.g_ram[FRAMES_MISSED] = 0;
            game.g_ram[FRAME_COUNTER_BYTE] = game.g_ram[FRAME_COUNTER_BYTE].wrapping_add(1);
            word!(game.nmi_frame_counter_word += 1);
        } else {
            game.g_ram[FRAMES_MISSED] = game.g_ram[FRAMES_MISSED].wrapping_add(1);
            if game.g_ram[FRAMES_MISSED] >= game.g_ram[FRAMES_MISSED_MAX] {
                game.g_ram[FRAMES_MISSED_MAX] = game.g_ram[FRAMES_MISSED];
            }
        }
        word!(game.nmi_frame_counter_including_lag += 1);
    }
}

/// Requires a live, exclusively accessed SNES backend for ordered channel-1 setup.
pub(crate) unsafe fn setup_transfer(mode: u16, source: u32, bytes: u16, game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            word(DMAP1 as u16, mode); word(A1T1L as u16, source as u16);
            byte(A1B1 as u16, (source >> 16) as u8); word(DAS1L as u16, bytes);
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
    pub(crate) unsafe fn load_planes(self, fill: u8, word_reset: bool, game: &mut Game) {
        unsafe {
            Self::VramHighBytes(0x7f0000, 0x0000, 0x4000).load(game);
            WriteReg(VMAIN as u16, 0, game);
            if word_reset {
                WriteRegWord(VMADDL as u16, 0, game);
            } else {
                WriteReg(VMADDL as u16, 0, game);
                WriteReg(VMADDH as u16, 0, game);
            }
            for _ in 0i16..0x4000i16 {
                WriteReg(VMDATAL as u16, fill, game);
            }
            self.load(game);
            Self::VramWords(0x7f5000, 0x6000, 0x4000).load(game);
        }
    }

    pub(crate) unsafe fn load(self, game: &mut Game) {
        unsafe {
            crate::verify_probe::hit(match &self {
                Self::Decompress(..) => "graphics-load-decompress",
                Self::VramWords(..) => "graphics-load-vram-words",
                Self::VramLowBytes(..) => "graphics-load-vram-low",
                Self::VramHighBytes(..) => "graphics-load-vram-high",
                Self::WramBytes(..) => "graphics-load-wram",
            });
            let (mode, source, size, vram) = match self {
                Self::Decompress(source, destination) => {
                    decompress_to_ram(source, destination, game);
                    return;
                }
                Self::VramWords(source, destination, size) => {
                    ([1, 0x18], source, size, Some((destination, 0x80)))
                }
                Self::VramLowBytes(source, destination, size) => {
                    ([0, 0x18], source, size, Some((destination, 0)))
                }
                Self::VramHighBytes(source, destination, size) => {
                    ([0, 0x19], source, size, Some((destination, 0x80)))
                }
                Self::WramBytes(source, destination, size) => {
                    crate::rtl::register_writes! { game;
                        byte(WMADDL as u16, destination as u8);
                        byte(WMADDM as u16, (destination >> 8) as u8);
                        byte(WMADDH as u16, (destination >> 16) as u8);
                    }
                    ([0, 0x80], source, size, None)
                }
            };
            if let Some((destination, vmain)) = vram {
                crate::rtl::register_writes! { game;
                    byte(VMADDL as u16, destination as u8); byte(VMADDH as u16, (destination >> 8) as u8);
                    byte(VMAIN as u16, vmain);
                }
            }
            // The old packed descriptor read its adjacent mode/register bytes natively.
            setup_transfer(u16::from_ne_bytes(mode), source, size, game);
            WriteReg(MDMAEN as u16, 2, game);
        }
    }
}

pub unsafe fn CopyToVramNow(vram_dst: u16, src: u32, size: u16, game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            byte(INIDISP as u16, 0x80); word(VMADDL as u16, vram_dst);
        }
        setup_transfer(0x1801, src, size, game);
        crate::rtl::register_writes! { game;
            byte(VMAIN as u16, 0x80); byte(MDMAEN as u16, 2);
            byte(INIDISP as u16, 0xf);
        }
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

pub(crate) fn queue_scrolling_tiles(k: u16, layout: ScrollingDmaLayout, game: &mut Game) {
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
    let source = 0x10000 + usize::from(source_offset);
    let parameters = (if row { 0x964 } else { 0x956 }) + usize::from(k);
    // Columns store the wrapped size first; rows store the unwrapped size first.
    if row {
        let within_screen = word!(game.vram_blocks_to_update_x_block) & 15;
        set_native_word(parameters, 4 * (16 - within_screen), game);
        set_native_word(parameters + 2, 4 * (within_screen + 1), game);
    } else {
        let wrapped = 4 * (word!(game.vram_blocks_to_update_y_block) & 15);
        set_native_word(parameters + 2, wrapped, game);
        set_native_word(parameters, (wrapped ^ 0x3f) + 1, game);
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
    set_native_word(parameters + 4, origin.wrapping_add(screen), game);
    let wrapped_destination = if row {
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
    set_native_word(parameters + 6, wrapped_destination, game);
    let buffer_base = if row {
        word_arrays::bg1_column_update_tilemap_top_halves
    } else {
        word_arrays::bg1_column_update_tilemap_left_halves
    };
    let buffer = buffer_base + if k != 0 { 264 } else { 0 };
    let half_bytes = if row { 68 } else { 64 };
    let wrapped_source = native_word(parameters, game).wrapping_add(buffer as u16);
    set_native_word(parameters + 8, wrapped_source, game);
    set_native_word(
        parameters + 10,
        wrapped_source.wrapping_add(half_bytes),
        game,
    );
    let mut source_delta = 0u16;
    for block_index in 0..if row { 17 } else { 16 } {
        let block = native_word(source + 2 * usize::from(source_delta >> 1), game);
        let first = buffer + 4 * block_index;
        let second = first + usize::from(half_bytes);
        let destinations = if row {
            [first, first + 2, second, second + 2]
        } else {
            [first, second, first + 2, second + 2]
        };
        let flip = usize::from((block >> 10) & 3);
        let tile_flip = (block & 0x0c00) << 4;
        let first_tile = usize::from(block & 0x03ff) * 4;
        for (corner, destination) in destinations.into_iter().enumerate() {
            // Read each tile immediately before storing it: staging may overlap the table.
            let source = 0xa000 + 2 * (first_tile + (corner ^ flip));
            let tile = at!(game, source;) ^ tile_flip;
            at!(game, destination; = tile);
        }
        // The column stride is reread after drawing each block.
        source_delta = source_delta.wrapping_add(if row {
            2
        } else {
            word!(game.room_width_in_blocks).wrapping_mul(2)
        });
    }
    set_native_word(
        parameters + 12,
        native_word(parameters + 12, game).wrapping_add(1),
        game,
    );
}

fn native_word(offset: usize, game: &Game) -> u16 {
    u16::from_ne_bytes([game.g_ram[offset], game.g_ram[offset + 1]])
}

fn set_native_word(offset: usize, value: u16, game: &mut Game) {
    let bytes = value.to_ne_bytes();
    game.g_ram[offset] = bytes[0];
    game.g_ram[offset + 1] = bytes[1];
}

/// Publish a native queue record, evaluating each source after preceding stores.
#[forbid(unsafe_code)]
pub(crate) fn queue_native_transfer(
    mut cursor: u16,
    size: u16,
    source: impl FnOnce(&Game) -> u16,
    bank: u8,
    destination: impl FnOnce(&Game) -> u16,
    game: &mut Game,
) -> u16 {
    set_native_word(0xd0 + usize::from(cursor), size, game);
    cursor = cursor.wrapping_add(2);
    let source = source(game);
    set_native_word(0xd0 + usize::from(cursor), source, game);
    cursor = cursor.wrapping_add(2);
    game.g_ram[0xd0 + usize::from(cursor)] = bank;
    cursor = cursor.wrapping_add(1);
    let destination = destination(game);
    set_native_word(0xd0 + usize::from(cursor), destination, game);
    cursor = cursor.wrapping_add(2);
    word!(game.vram_write_queue_tail = cursor);
    cursor
}

pub(crate) fn queue_fx_clear(game: &mut Game) {
    word_transfers!(game, descending, 1920; native(0x4efe) <- value(6222));
    let tail = word!(game.vram_write_queue_tail);
    let entry = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
    set_native_word(entry, 3840, game);
    set_native_word(entry + 2, 0x4000, game);
    // The bank store is deliberately a native word, overlapping the destination low byte.
    set_native_word(entry + 4, 126, game);
    set_native_word(entry + 5, addr_unk_605880 as u16, game);
    word!(game.vram_write_queue_tail = tail.wrapping_add(7));
}
