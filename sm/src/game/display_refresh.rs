//! Game's display refresh host: the owned display stores and the uploads it
//! still owns, plus unsafe entries for callers with a live backend.
use crate::game::{animated_tiles, mode7_transfers, vram_read_queue};
use crate::game_state::Game;
use crate::rtl::{DMAP1, MDMAEN, VMADDL, VMAIN, WriteReg, WriteRegWord};
use sm_game::display::control::Brightness;
pub(crate) use sm_game::display::transfer::ScrollingLayout as ScrollingDmaLayout;
use sm_game::display_refresh as refresh;
pub(crate) use sm_game::display_refresh::{
    fade, native_word, queue_fx_clear, queue_native_transfer, queue_scrolling_tiles,
    reset_gameplay_layers, reset_layer_mirrors, set_blank, set_native_word, wait,
};

impl refresh::Host for Game {
    fn write_owned_byte(&mut self, at: usize, value: u8) {
        self.compatibility_write_owned_byte(at, value);
    }
    fn start_graphics_dma(&mut self, source: u32) {
        let assets = if source == 0x9ab200 {
            self.world().prepared_room_assets()
        } else {
            None
        };
        // SAFETY: refresh hosts run from the frame loop with the backend installed.
        unsafe { start_room_dma(2, assets.as_deref(), self) }
    }
    fn upload_samus(&mut self) {
        // SAFETY: as for `start_graphics_dma`.
        unsafe { crate::game::display_transfers::upload_samus(self) }
    }
    fn upload_animated_tiles(&mut self) {
        animated_tiles::upload_pending(self);
    }
    fn process_mode7(&mut self) {
        // SAFETY: as for `start_graphics_dma`.
        unsafe { mode7_transfers::process(self) }
    }
    fn process_readbacks(&mut self) {
        // SAFETY: as for `start_graphics_dma`.
        unsafe { vram_read_queue::process(self) }
    }
    fn flush_tilemaps(&mut self) {
        // SAFETY: as for `start_graphics_dma`.
        unsafe { flush_tilemaps(self) }
    }
    fn fill_vram_low_bytes(&mut self, fill: u8, word_reset: bool) {
        // SAFETY: as for `start_graphics_dma`.
        unsafe { crate::game::display_transfers::fill_vram_low_bytes(fill, word_reset, self) }
    }
}

/// Requires a live, exclusively accessed SNES backend for ordered PPU writes.
pub(crate) unsafe fn pause_graphics(paused: bool, game: &mut Game) {
    refresh::pause_graphics(paused, game);
}

/// Requires live, exclusively accessed hardware for the ordered CPU I/O and OAM reset.
pub(crate) unsafe fn initialize_io_and_oam(brightness: Option<u8>, game: &mut Game) {
    refresh::initialize_io_and_oam(brightness, game);
}

/// Requires a live, exclusively accessed SNES backend and prepared cartridge graphics.
pub(crate) unsafe fn initialize_gameplay(game: &mut Game) {
    refresh::initialize_gameplay(game);
}

/// Requires a live, exclusively accessed SNES backend for VRAM readback.
pub(crate) unsafe fn capture_pause_background(game: &mut Game) {
    refresh::capture_pause_background(game);
}

/// Requires a live, exclusively accessed SNES backend and pause graphics assets.
pub(crate) unsafe fn load_pause_assets(game: &mut Game) {
    refresh::load_pause_assets(game);
}

/// Requires a live, exclusively accessed SNES backend throughout map and label upload.
pub(crate) unsafe fn upload_pause_map(game: &mut Game) {
    refresh::upload_pause_map(game);
}

/// Requires a live, exclusively accessed backend for DMA register writes.
pub(crate) unsafe fn finish_gameplay(clear_current_irq: bool, game: &mut Game) {
    unsafe {
        crate::game::visual_runtime::clear_hdma(game);
    }
    crate::game::scanline_irq::disable(game);
    reset_gameplay_layers(clear_current_irq, game);
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn enable(enabled: bool, game: &mut Game) {
    refresh::enable(enabled, game);
}

/// Requires a live, exclusively accessed SNES backend throughout the queue drain.
pub(crate) unsafe fn flush_tilemaps(game: &mut Game) {
    unsafe {
        let tail = game.vram_write_queue_tail();
        if tail != 0 {
            // The sentinel uses the byte tail, which need not lie on an entry boundary.
            game.write_vram_queue_word(tail, 0, 0);
            WriteRegWord(DMAP1 as u16, 0x1801, game);
            sm_game::display::transfer::drain_tilemaps(
                &mut crate::game::display_transfers::CompatibilityQueue::new(game),
            );
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
                crate::game::display_transfers::upload_scrolling_tilemap(
                    parameters, sources, layout, game,
                );
            }
        }
    }
}

/// Requires a live, exclusively accessed SNES backend for every upload.
pub(crate) unsafe fn run(game: &mut Game) {
    refresh::run(game);
}

/// Requires a live, exclusively accessed SNES backend for ordered channel-1 setup.
pub(crate) unsafe fn setup_transfer(mode: u16, source: u32, bytes: u16, game: &mut Game) {
    sm_game::display::transfer::setup_transfer(mode, source, bytes, game)
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
    fn shared(self) -> refresh::GraphicsAsset {
        match self {
            Self::Decompress(source, destination) => {
                refresh::GraphicsAsset::Decompress(source, destination)
            }
            Self::VramWords(source, destination, size) => {
                refresh::GraphicsAsset::VramWords(source, destination, size)
            }
            Self::VramLowBytes(source, destination, size) => {
                refresh::GraphicsAsset::VramLowBytes(source, destination, size)
            }
            Self::VramHighBytes(source, destination, size) => {
                refresh::GraphicsAsset::VramHighBytes(source, destination, size)
            }
            Self::WramBytes(source, destination, size) => {
                refresh::GraphicsAsset::WramBytes(source, destination, size)
            }
        }
    }

    /// Requires a live, exclusively accessed SNES backend.
    pub(crate) unsafe fn load_planes(self, fill: u8, word_reset: bool, game: &mut Game) {
        self.shared().load_planes(fill, word_reset, game);
    }

    /// Requires a live, exclusively accessed SNES backend.
    pub(crate) unsafe fn load(self, game: &mut Game) {
        self.shared().load(game);
    }
}

pub(crate) unsafe fn start_room_dma(
    mask: u8,
    assets: Option<&super::room_entry::PreparedRoomAssets>,
    game: &mut Game,
) {
    unsafe {
        match assets {
            Some(assets) => {
                let (first, bytes) = assets.raw_window();
                crate::rtl::write_dma_start(
                    mask,
                    &crate::execution::DmaRomSource::new(first, bytes),
                    game,
                )
            }
            None => WriteReg(MDMAEN as u16, mask, game),
        }
    }
}

pub unsafe fn CopyToVramNow(vram_dst: u16, src: u32, size: u16, game: &mut Game) {
    unsafe {
        copy_to_vram_with_source(vram_dst, src, size, None, game);
    }
}
pub(crate) unsafe fn copy_to_vram_with_source(
    vram_dst: u16,
    src: u32,
    size: u16,
    assets: Option<&super::room_entry::PreparedRoomAssets>,
    game: &mut Game,
) {
    unsafe {
        crate::rtl::register_writes! { game;
            show(Brightness::BLANK_DARK); word(VMADDL as u16, vram_dst);
        }
        setup_transfer(0x1801, src, size, game);
        crate::rtl::register_writes! { game;
            byte(VMAIN as u16, 0x80);
        }
        start_room_dma(2, assets, game);
        crate::game::display_transfers::set_brightness(Brightness::FULL, game);
    }
}

/// Requires a live, exclusively accessed SNES backend for the PPU readback.
pub unsafe fn DecompressToVRAM(src: u32, dst_addr: u16, game: &mut Game) {
    refresh::decompress_to_vram(src, dst_addr, game);
}
/// Requires a live, exclusively accessed SNES backend for the PPU readback.
pub(crate) unsafe fn decompress_prepared_vram(stream: &[u8], destination: u16, game: &mut Game) {
    refresh::decompress_prepared_vram(stream, destination, game);
}
