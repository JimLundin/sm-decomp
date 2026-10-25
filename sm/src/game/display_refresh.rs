//! Ordered display refresh and the NMI request/completion handshake.
use crate::game::{animated_tiles, mode7_transfers, vram_read_queue};
use crate::game_state::Game;
use crate::ram_map::{checked_word as at, word};
use crate::rtl::{
    A1B0, A1B1, A1T0L, A1T1L, BG1HOFS, BG1SC, BG2SC, BG3SC, BG4SC, BG12NBA, BG34NBA, BGMODE, CGADD,
    CGADSUB, CGWSEL, COLDATA, DAS0L, DAS1L, DMAP0, DMAP1, HDMAEN, INIDISP, M7A, M7SEL, MDMAEN,
    MOSAIC, NMITIMEN, OAMADDL, OBSEL, SETINI, TM, TMW, TS, TSW, VMADDL, VMAIN, W12SEL, W34SEL,
    WBGLOG, WH0, WH1, WH2, WH3, WOBJLOG, WOBJSEL, WriteReg, WriteRegWord,
};

/// Set the forced-blank mirror bit for the next display refresh.
pub(crate) fn set_blank(blank: bool, game: &mut Game) {
    game.g_ram[0x51] = game.g_ram[0x51] & 0x7f | if blank { 0x80 } else { 0 };
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
