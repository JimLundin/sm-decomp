//! Game's display refresh host: the owned display stores and the uploads it
//! still owns, plus unsafe entries for callers with a live backend.
use crate::game::{animated_tiles, mode7_transfers, vram_read_queue};
use crate::game_state::Game;
use crate::rtl::{DMAP1, MDMAEN, VMAIN, write_reg, write_reg_word};
pub(crate) use sm_game::display::transfer::ScrollingLayout as ScrollingDmaLayout;
use sm_game::display_refresh as refresh;
pub(crate) use sm_game::display_refresh::{
    fade, native_word, queue_fx_clear, queue_native_transfer, queue_scrolling_tiles, set_blank,
    set_native_word, wait,
};

impl refresh::DisplayOwner for Game {
    fn write_owned_byte(&mut self, at: usize, value: u8) {
        self.compatibility_write_owned_byte(at, value);
    }
}

impl refresh::Host for Game {
    fn hdma_objects(&self) -> sm_game::hdma::HdmaObjects {
        self.projected_hdma_objects()
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

/// The ordered CPU I/O and OAM reset.
pub(crate) fn initialize_io_and_oam(brightness: Option<u8>, game: &mut Game) {
    refresh::initialize_io_and_oam(brightness, game);
}

/// Enable or disable the NMI.
pub(crate) fn enable(enabled: bool, game: &mut Game) {
    refresh::enable(enabled, game);
}

/// Requires a live, exclusively accessed SNES backend throughout the queue drain.
pub(crate) unsafe fn flush_tilemaps(game: &mut Game) {
    unsafe {
        let tail = game.vram_write_queue_tail();
        if tail != 0 {
            // The sentinel uses the byte tail, which need not lie on an entry boundary.
            game.write_vram_queue_word(tail, 0, 0);
            write_reg_word(DMAP1 as u16, 0x1801, game);
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
            write_reg(VMAIN as u16, increment, game);
            for (parameters, sources) in buffers {
                crate::game::display_transfers::upload_scrolling_tilemap(
                    parameters, sources, layout, game,
                );
            }
        }
    }
}

/// The ordered display refresh uploads.
pub(crate) fn run(game: &mut Game) {
    refresh::run(game);
}

pub(crate) use sm_game::display_refresh::GraphicsAsset;

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
            None => write_reg(MDMAEN as u16, mask, game),
        }
    }
}

impl sm_game::display::transfer::RoomDma for Game {
    fn start_room_dma(&mut self, mask: u8, assets: Option<&super::room_entry::PreparedRoomAssets>) {
        // SAFETY: immediate copies run from the frame loop with the backend installed.
        unsafe { start_room_dma(mask, assets, self) }
    }
}

/// Decompress `src` into VRAM at `dst_addr`, with its PPU readback.
pub fn decompress_to_vram(src: u32, dst_addr: u16, game: &mut Game) {
    refresh::decompress_to_vram(src, dst_addr, game);
}
/// Decompress a prepared stream into VRAM, with its PPU readback.
pub(crate) fn decompress_prepared_vram(stream: &[u8], destination: u16, game: &mut Game) {
    refresh::decompress_prepared_vram(stream, destination, game);
}
