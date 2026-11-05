//! Game's display refresh host: the owned display stores and the uploads it
//! still owns, plus unsafe entries for callers with a live backend.
use crate::game::{animated_tiles, mode7_transfers, vram_read_queue};
use crate::game_state::Game;
use crate::rtl::{MDMAEN, write_reg};
pub(crate) use sm_game::display::transfer::ScrollingLayout as ScrollingDmaLayout;
use sm_game::display::video::Upload;
use sm_game::display_refresh as refresh;

/// The NMI on the frame's lease, inside the tick: each upload goes through the
/// lease's transfer port, whose DMA starts publish the work RAM bytes they read
/// (`Runtime::start_dma`), so a readback's work RAM bytes return to the Session; the queue tails,
/// the sentinels and the consumed scrolling enables are the lease's stores.
impl refresh::Nmi for crate::game::session_runtime::Runtime<'_> {
    fn upload_samus(&mut self) {
        self.upload_samus_tiles();
    }
    fn upload_animated_tiles(&mut self) {
        // The lease lends itself, as Game's fresh lease ran it.
        self.owned_player_call(animated_tiles::execution::upload_pending);
    }
    fn process_mode7(&mut self) {
        self.owned_player_call(|runtime| {
            mode7_transfers::process(&mut crate::game::display_commands::Lease(runtime));
        });
    }
    fn process_readbacks(&mut self) {
        self.owned_player_call(|runtime| {
            vram_read_queue::process(&mut crate::game::display_commands::Lease(runtime));
        });
    }
    fn flush_tilemaps(&mut self) {
        self.owned_player_call(flush_tilemaps_on_lease);
    }
}

/// The typed uploads' bracket around the encoder's register writes.
impl Game {
    pub(crate) fn selected_video(&mut self) -> *mut crate::types::Ppu {
        // SAFETY: the bracket's callers make register writes, which require the
        // installed backend, and `&mut Game` excludes other access.
        unsafe { self.device_execution() }.backend.selected_video()
    }
    /// The producer's register writes begin. The bracket reaches the selected
    /// PPU alone, so it runs in a device interval: a lease's transfer port
    /// brackets its writes with it and stays current.
    pub(crate) fn producing(&mut self) {
        let ppu = self.selected_video();
        crate::snes::ppu::producing(ppu);
    }
    /// The producer's register writes are done; `uploads` are what they did,
    /// each work RAM byte as RAM holds it.
    pub(crate) fn produced(&mut self, producer: &'static str, uploads: &[Upload]) {
        let ppu = self.selected_video();
        let game = &*self;
        crate::snes::ppu::produced(ppu, producer, |ports, memory| {
            feed_native(uploads, &|at| game.ram().read_byte(at), game, ports, memory);
        });
    }
}

/// The native PPU takes `uploads` in place of the port writes that made them:
/// each work RAM byte from `wram`, the live HUD and scrolling bytes and the
/// cartridge's from `game`.
pub(crate) fn feed_native(
    uploads: &[Upload],
    wram: &dyn Fn(usize) -> u8,
    game: &Game,
    ports: &mut crate::rendering::memory::Ports,
    memory: &mut crate::rendering::memory::OwnedMemory,
) {
    let bytes =
        |source: usize, count: u16| (source..source + usize::from(count)).map(|at| wram(at));
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
                        bus_byte(game, wram, source.bank, source.address.wrapping_add(offset))
                    }),
                );
            }
        }
    }
}

impl Game {
    /// Apply a display command to the backend outside any lease, as
    /// `oracle_host::OracleHost`'s `Video` does.
    pub(crate) fn apply(
        &mut self,
        producer: &'static str,
        command: sm_game::display::video::Command,
    ) {
        crate::game::display_commands::apply(producer, command, self);
    }
}

/// A byte a DMA reads from the CPU bus: work RAM (from `wram`) or the cartridge.
fn bus_byte(game: &Game, wram: &dyn Fn(usize) -> u8, bank: u8, address: u16) -> u8 {
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
        0x7e | 0x7f => wram((usize::from(bank & 1) << 16) | usize::from(address)),
        0x00..=0x3f | 0x80..=0xbf if address < 0x2000 => wram(usize::from(address)),
        _ => game.read_rom_byte((u32::from(bank) << 16) | u32::from(address), 0),
    }
}

/// The tilemap flush on the lease: it drains the queue and the scrolling requests
/// through the lease's transfer port, so each DMA start reads the Session's bytes
/// (it publishes them), and the queue's end word, its tail and the consumed enable
/// bytes are the lease's stores.
pub(crate) fn flush_tilemaps_on_lease(runtime: &mut crate::game::session_runtime::Runtime<'_>) {
    crate::game::display_transfers::flush_tilemaps(&mut crate::game::display_commands::Lease(
        runtime,
    ));
}

/// The ordered display refresh uploads, on a lease of their own: the NMI as
/// the frame's lease runs it, for the verification routes that step frames by
/// hand.
pub(crate) fn run(game: &mut Game) {
    refresh::run(&mut crate::game::session_runtime::Runtime::new(game));
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
            None => write_reg(MDMAEN as u16, mask, game),
        }
    }
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
