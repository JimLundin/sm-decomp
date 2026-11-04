//! Game's display refresh host: the owned display stores and the uploads it
//! still owns, plus unsafe entries for callers with a live backend.
use crate::game::{animated_tiles, mode7_transfers, vram_read_queue};
use crate::game_state::Game;
use crate::rtl::{DMAP1, MDMAEN, VMAIN, write_reg, write_reg_word};
pub(crate) use sm_game::display::transfer::ScrollingLayout as ScrollingDmaLayout;
use sm_game::display::video::Upload;
use sm_game::display_refresh as refresh;
pub(crate) use sm_game::display_refresh::{fade, set_blank, set_native_word, wait};

/// The native-order word at RAM `offset` through Game's compatibility
/// projection: the scrolling requests and the level data blocks.
pub(crate) fn native_word(offset: usize, game: &Game) -> u16 {
    u16::from_ne_bytes([
        game.compatibility_byte(offset),
        game.compatibility_byte(offset + 1),
    ])
}

impl refresh::DisplayOwner for Game {
    fn write_owned_byte(&mut self, at: usize, value: u8) {
        self.compatibility_write_owned_byte(at, value);
    }
}

impl refresh::ReadbackQueue for Game {
    fn readback_queue_tail(&self) -> u16 {
        crate::ram_map::checked_word!(self.VRAM_READ_QUEUE_TAIL)
    }
    fn set_readback_queue_tail(&mut self, tail: u16) {
        crate::ram_map::checked_word!(self.VRAM_READ_QUEUE_TAIL = tail);
    }
    fn store_readback_byte(&mut self, offset: usize, value: u8) {
        self.compatibility_write_byte(0x340 + offset, value);
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
    fn read_vram_data(&mut self) -> u16 {
        self.read_register_word(crate::rtl::RDVRAML as u16)
    }
}

/// The typed uploads' bracket around the encoder's register writes.
impl Game {
    /// The producer's register writes begin.
    pub(crate) fn producing(&mut self) {
        let ppu = self.execution_parts().0.unwrap().selected_video();
        crate::snes::ppu::producing(ppu);
    }
    /// The producer's register writes are done; `uploads` are what they did.
    pub(crate) fn produced(&mut self, producer: &'static str, uploads: &[Upload]) {
        let ppu = self.execution_parts().0.unwrap().selected_video();
        let self_ref = &*self;
        let ram = self.ram();
        let bytes = |source: usize, count: u16| {
            (source..source + usize::from(count)).map(|at| ram.read_byte(at))
        };
        crate::snes::ppu::produced(ppu, producer, |ports, memory| {
            for &upload in uploads {
                match upload {
                    Upload::Objects {
                        address,
                        source,
                        bytes: count,
                    } => {
                        ports.set_oam_address(address);
                        ports.upload_oam(memory, bytes(source, count));
                    }
                    Upload::Palette {
                        index,
                        source,
                        bytes: count,
                    } => {
                        ports.set_cgram_pointer(index);
                        ports.upload_cgram(memory, bytes(source, count));
                    }
                    Upload::Tiles {
                        address,
                        increment,
                        source,
                        bytes: count,
                    } => {
                        ports.set_vram_mode(increment, 0, true);
                        ports.set_vram_pointer(memory, address);
                        ports.upload_vram(
                            memory,
                            (0..=count.wrapping_sub(1)).map(|offset| {
                                bus_byte(self_ref, source.bank, source.address.wrapping_add(offset))
                            }),
                        );
                    }
                }
            }
        });
    }
}

impl sm_game::display::video::Video for Game {
    fn apply(&mut self, producer: &'static str, command: sm_game::display::video::Command) {
        crate::game::display_commands::apply(producer, command, self);
    }
}

/// A byte a DMA reads from the CPU bus: work RAM or the cartridge.
fn bus_byte(game: &Game, bank: u8, address: u16) -> u8 {
    use sm_game::display::queued_tilemaps::{LiveByte, live_byte};
    if let Some(owner) = live_byte(bank, address) {
        return match owner {
            LiveByte::Hud { tile, lane } => game.live_queued_hud_byte(tile, lane),
            LiveByte::Scrolling { word, lane } => {
                // compatibility_byte serves the current Session's scrolling
                // owner; a callback-invalidated owner reads the actual image.
                game.compatibility_byte(0xc8c8 + word * 2 + lane)
            }
        };
    }
    match bank {
        0x7e | 0x7f => game
            .ram()
            .read_byte((usize::from(bank & 1) << 16) | usize::from(address)),
        0x00..=0x3f | 0x80..=0xbf if address < 0x2000 => game.ram().read_byte(usize::from(address)),
        _ => game.read_rom_byte((u32::from(bank) << 16) | u32::from(address), 0),
    }
}

/// The ordered CPU I/O and OAM reset.
pub(crate) fn initialize_io_and_oam(
    brightness: Option<sm_game::display::control::Brightness>,
    game: &mut Game,
) {
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
    fn copy_to_vram_now(
        &mut self,
        address: u16,
        source: sm_game::display::transfer::Source,
        bytes: u16,
        assets: Option<&super::room_entry::PreparedRoomAssets>,
    ) {
        crate::game::display_commands::copy_to_vram_now(address, source, bytes, assets, self);
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

use sm_game::display::queued_tilemaps::{Coordinate, QueueAccess, Request, Scrolling, Selection};
fn scrolling_parameter(selection: Selection, field: Request) -> usize {
    let base = if matches!(selection.layout, ScrollingDmaLayout::Row) {
        0x964
    } else {
        0x956
    };
    base + usize::from(selection.layer_offset) + usize::from(field as u8) * 2
}
fn scrolling_coordinate(field: Coordinate) -> usize {
    match field {
        Coordinate::SourceX => 0x990,
        Coordinate::SourceY => 0x992,
        Coordinate::DestinationX => 0x994,
        Coordinate::DestinationY => 0x996,
        Coordinate::BackgroundSize => 0x98e,
    }
}
fn scrolling_probe(row: bool, background: bool) {
    crate::verify_probe::hit(match (row, background) {
        (false, false) => "scroll_tiles.column.bg1",
        (false, true) => "scroll_tiles.column.bg2",
        (true, false) => "scroll_tiles.row.bg1",
        (true, true) => "scroll_tiles.row.bg2",
    });
}
impl QueueAccess for Game {
    fn queue_live_source(
        &mut self,
        address: u16,
        bank: u8,
        source: sm_game::scene::state::TransferSource,
    ) {
        let mut runtime = crate::game::session_runtime::Runtime::new(self);
        QueueAccess::queue_live_source(&mut runtime, address, bank, source);
    }
    fn queue_tail(&self) -> u16 {
        self.vram_write_queue_tail()
    }
    fn store_queue_tail(&mut self, value: u16) {
        self.set_vram_write_queue_tail(value);
    }
    fn queue_native_word(&mut self, cursor: u16, value: u16) {
        let at = 0xd0 + usize::from(cursor);
        let [low, high] = value.to_ne_bytes();
        self.compatibility_write_byte(at, low);
        self.compatibility_write_byte(at + 1, high);
    }
    fn queue_byte(&mut self, cursor: u16, value: u8) {
        self.compatibility_write_byte(0xd0 + usize::from(cursor), value);
    }
}
impl Scrolling for Game {
    fn mode7_enabled(&self, selection: Selection) -> bool {
        let enabled = u16::from_le_bytes([
            self.compatibility_byte(0x783),
            self.compatibility_byte(0x784),
        ]) != 0;
        if !enabled {
            scrolling_probe(
                matches!(selection.layout, ScrollingDmaLayout::Row),
                selection.layer_offset != 0,
            );
        }
        enabled
    }
    fn coordinate(&self, field: Coordinate) -> u16 {
        let at = scrolling_coordinate(field);
        u16::from_le_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
    }
    fn room_width(&self) -> u16 {
        sm_game::gameplay_context::navigation(self).width
    }
    fn request(&self, selection: Selection, field: Request) -> u16 {
        native_word(scrolling_parameter(selection, field), self)
    }
    fn store_request(&mut self, selection: Selection, field: Request, value: u16) {
        set_native_word(scrolling_parameter(selection, field), value, self);
    }
    fn block(&self, source: u16, delta: u16) -> u16 {
        native_word(
            0x10000 + usize::from(source) + 2 * usize::from(delta >> 1),
            self,
        )
    }
    fn tile(&self, definition: usize, corner: usize) -> u16 {
        let at = 0xa000 + definition * 8 + corner * 2;
        u16::from_le_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
    }
    fn store_scrolling(&mut self, word: usize, value: u16) {
        let at = 0xc8c8 + word * 2;
        let [low, high] = value.to_le_bytes();
        self.compatibility_write_byte(at, low);
        self.compatibility_write_byte(at + 1, high);
    }
}
impl QueueAccess for crate::game::session_runtime::Runtime<'_> {
    fn queue_live_source(
        &mut self,
        address: u16,
        bank: u8,
        source: sm_game::scene::state::TransferSource,
    ) {
        let bytes = self.session().queued_tilemaps.native_order.encode(address);
        let token = [bytes[0], bytes[1], bank];
        // Only this actual query identity denotes the supplied symbolic HUD source.
        // Other native-order aliases retain the compatibility transfer path.
        if u16::from_le_bytes(bytes) != address {
            return;
        }
        if let Ok(owner) = self.session_mut().presentation.owner_mut() {
            owner.state.queued_sources.insert(token, source);
        }
    }
    fn queue_tail(&self) -> u16 {
        self.owned_session(|s| s.queued_tilemaps.tail, 0x330)
    }
    fn store_queue_tail(&mut self, value: u16) {
        self.write_word_le(value, 0x330);
    }
    fn queue_native_word(&mut self, cursor: u16, value: u16) {
        let at = 0xd0 + usize::from(cursor);
        let [low, high] = value.to_ne_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
    fn queue_byte(&mut self, cursor: u16, value: u8) {
        self.set_byte(0xd0 + usize::from(cursor), value);
    }
}
impl Scrolling for crate::game::session_runtime::Runtime<'_> {
    fn mode7_enabled(&self, selection: Selection) -> bool {
        let enabled = self.word(0x783) != 0;
        if !enabled {
            scrolling_probe(
                matches!(selection.layout, ScrollingDmaLayout::Row),
                selection.layer_offset != 0,
            );
        }
        enabled
    }
    fn coordinate(&self, field: Coordinate) -> u16 {
        self.word(scrolling_coordinate(field))
    }
    fn room_width(&self) -> u16 {
        self.owned_session(|s| s.room.width, 0x7a5)
    }
    fn request(&self, selection: Selection, field: Request) -> u16 {
        self.native_word(scrolling_parameter(selection, field))
    }
    fn store_request(&mut self, selection: Selection, field: Request, value: u16) {
        let at = scrolling_parameter(selection, field);
        let [low, high] = value.to_ne_bytes();
        sm_game::display_refresh::DisplayOwner::write_owned_byte(self, at, low);
        sm_game::display_refresh::DisplayOwner::write_owned_byte(self, at + 1, high);
    }
    fn block(&self, source: u16, delta: u16) -> u16 {
        self.native_word(0x10000 + usize::from(source) + 2 * usize::from(delta >> 1))
    }
    fn tile(&self, definition: usize, corner: usize) -> u16 {
        self.word(0xa000 + definition * 8 + corner * 2)
    }
    fn store_scrolling(&mut self, word: usize, value: u16) {
        self.write_word_le(value, 0xc8c8 + word * 2);
    }
}
