//! Live queued tilemaps: packed queue, scrolling composition and supplied storage.
//! No tile content is captured across an effect or a suspended update.
#![forbid(unsafe_code)]
use super::transfer_types::ScrollingLayout;

pub const QUEUE_BYTES: usize = 512;
pub const SCROLL_WORDS: usize = 264;
pub const BACKGROUND_BLOCKS: usize = 12_800;

/// Byte-compatible queue storage is owned, including incomplete records and
/// unknown bytes. The compatibility cursor retains its full identity.
#[derive(Clone, Debug, PartialEq)]
pub struct QueuedTilemaps {
    pub queue: [u8; QUEUE_BYTES],
    pub tail: u16,
    pub native_order: crate::scene::state::WordOrder,
    pub scrolling: [u16; SCROLL_WORDS],
    background: Box<[u16; BACKGROUND_BLOCKS]>,
    pub source_block: [u16; 2],
    pub destination_block: [u16; 2],
}
impl QueuedTilemaps {
    pub fn from_owned(
        queue: [u8; QUEUE_BYTES],
        tail: u16,
        scrolling: [u16; SCROLL_WORDS],
        background: Box<[u16]>,
        source_block: [u16; 2],
        destination_block: [u16; 2],
        native_order: crate::scene::state::WordOrder,
    ) -> Option<Self> {
        let background = background.try_into().ok()?;
        Some(Self {
            queue,
            tail,
            scrolling,
            background,
            source_block,
            destination_block,
            native_order,
        })
    }
    pub(crate) fn initialized() -> Self {
        Self {
            queue: [0; QUEUE_BYTES],
            tail: 0,
            native_order: if cfg!(target_endian = "little") {
                crate::scene::state::WordOrder::Little
            } else {
                crate::scene::state::WordOrder::Big
            },
            scrolling: [0; SCROLL_WORDS],
            background: Box::new([0; BACKGROUND_BLOCKS]),
            source_block: [0; 2],
            destination_block: [0; 2],
        }
    }
    pub fn background(&self) -> &[u16; BACKGROUND_BLOCKS] {
        &self.background
    }
    pub fn background_mut(&mut self) -> &mut [u16; BACKGROUND_BLOCKS] {
        &mut self.background
    }
}

/// Publication lane distinguishes native queue words from a scrolling request
/// word. Compatibility hosts retain their distinct per-byte write policies.
pub trait QueueAccess {
    fn queue_tail(&self) -> u16;
    fn store_queue_tail(&mut self, value: u16);
    fn queue_native_word(&mut self, cursor: u16, value: u16);
    fn queue_live_source(
        &mut self,
        address: u16,
        bank: u8,
        source: crate::scene::state::TransferSource,
    );
}

/// Publish the actual HUD tilemap source. Its bank is a native word whose high
/// byte overlaps the following destination. Wrap each cursor before widening.
pub fn queue_hud(host: &mut (impl QueueAccess + ?Sized)) {
    let tail = host.queue_tail();
    for (offset, value) in [(0, 192), (2, 0xc608), (4, 0x7e), (5, 0x5820)] {
        host.queue_native_word(tail.wrapping_add(offset), value);
    }
    host.queue_live_source(
        0xc608,
        0x7e,
        crate::scene::state::TransferSource::Hud { offset: 0 },
    );
    host.store_queue_tail(tail.wrapping_add(7));
}

#[derive(Clone, Copy)]
pub enum Coordinate {
    SourceX,
    SourceY,
    DestinationX,
    DestinationY,
    BackgroundSize,
}
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Request {
    Size,
    WrappedSize,
    Destination,
    WrappedDestination,
    WrappedFirst,
    WrappedSecond,
    Enable,
}

/// A live selection of the four physical scrolling requests. `layer_offset`
/// retains odd and full-u16 identities for the compatibility alias protocol.
#[derive(Clone, Copy)]
pub struct Selection {
    pub layout: ScrollingLayout,
    pub layer_offset: u16,
}

pub trait Scrolling {
    fn mode7_enabled(&self, selection: Selection) -> bool;
    fn coordinate(&self, field: Coordinate) -> u16;
    fn room_width(&self) -> u16;
    fn request(&self, selection: Selection, field: Request) -> u16;
    fn store_request(&mut self, selection: Selection, field: Request, value: u16);
    /// Native block read after the source's u16 arithmetic, before widening.
    fn block(&self, source: u16, delta: u16) -> u16;
    /// A fresh LE tile read, after every preceding request or tile store.
    fn tile(&self, definition: usize, corner: usize) -> u16;
    fn store_scrolling(&mut self, word: usize, value: u16);
}

fn product(a: u16, b: u16) -> u16 {
    u16::from(a.to_le_bytes()[0]) * u16::from(b.to_le_bytes()[0])
}

/// Compose one scrolling request from live terrain and write each corner
/// immediately. Request aliases remain visible to all later live reads.
pub fn queue_scrolling(selection: Selection, host: &mut (impl Scrolling + ?Sized)) {
    use Coordinate::*;
    use Request::*;
    if host.mode7_enabled(selection) {
        return;
    }
    let row = matches!(selection.layout, ScrollingLayout::Row);
    let background = selection.layer_offset != 0;
    let mut source = product(host.coordinate(SourceY), host.room_width())
        .wrapping_add(host.coordinate(SourceX))
        .wrapping_mul(2)
        .wrapping_add(2);
    if background {
        source = source.wrapping_add(0x9600);
    }
    if row {
        let within = host.coordinate(DestinationX) & 15;
        host.store_request(selection, Size, 4 * (16 - within));
        host.store_request(selection, WrappedSize, 4 * (within + 1));
    } else {
        let wrapped = 4 * (host.coordinate(DestinationY) & 15);
        host.store_request(selection, WrappedSize, wrapped);
        host.store_request(selection, Size, (wrapped ^ 0x3f) + 1);
    }
    let tile_row = product(host.coordinate(DestinationY) & 15, 0x40);
    let tile_column = host.coordinate(DestinationX) & 31;
    let origin = tile_row.wrapping_add(2 * tile_column);
    let mut screen: u16 = if tile_column >= 16 { 0x53e0 } else { 0x5000 };
    if background {
        screen = screen.wrapping_sub(host.coordinate(BackgroundSize));
    }
    host.store_request(selection, Destination, origin.wrapping_add(screen));
    let wrapped_destination = if row {
        let mut wrapped_screen: u16 = if tile_column >= 16 { 0x5000 } else { 0x5400 };
        if background {
            wrapped_screen = wrapped_screen.wrapping_sub(host.coordinate(BackgroundSize));
        }
        tile_row.wrapping_add(wrapped_screen)
    } else {
        (2 * tile_column).wrapping_add(screen)
    };
    host.store_request(selection, WrappedDestination, wrapped_destination);
    let buffer_word = (if row { 64 } else { 0 }) + (if background { 132 } else { 0 });
    let half_words = if row { 34 } else { 32 };
    let buffer_source = 0xc8c8 + u16::try_from(buffer_word * 2).unwrap();
    let wrapped_source = host.request(selection, Size).wrapping_add(buffer_source);
    host.store_request(selection, WrappedFirst, wrapped_source);
    host.store_request(
        selection,
        WrappedSecond,
        wrapped_source.wrapping_add(u16::try_from(half_words * 2).unwrap()),
    );
    let mut delta = 0u16;
    for block_index in 0..if row { 17 } else { 16 } {
        let block = host.block(source, delta);
        let first = buffer_word + 2 * block_index;
        let second = first + half_words;
        let destinations = if row {
            [first, first + 1, second, second + 1]
        } else {
            [first, second, first + 1, second + 1]
        };
        let flip = usize::from((block >> 10) & 3);
        let tile_flip = (block & 0x0c00) << 4;
        for (corner, destination) in destinations.into_iter().enumerate() {
            let tile = host.tile(usize::from(block & 0x03ff), corner ^ flip) ^ tile_flip;
            host.store_scrolling(destination, tile);
        }
        delta = delta.wrapping_add(if row {
            2
        } else {
            host.room_width().wrapping_mul(2)
        });
    }
    let enable = host.request(selection, Enable).wrapping_add(1);
    host.store_request(selection, Enable, enable);
}

/// The live queue reader. Offsets advance linearly, independently of the u16
/// producer cursor, and each next record is read only after the prior upload.
pub trait TilemapDrain {
    fn queue_byte(&self, offset: usize) -> u8;
    fn queue_word(&self, offset: usize) -> u16;
    fn upload_tilemap(&mut self, upload: super::transfer_types::TilemapUpload);
}
pub fn drain(host: &mut (impl TilemapDrain + ?Sized)) {
    use super::transfer_types::{Source, TilemapUpload};
    let mut entry = 0;
    loop {
        let bytes = host.queue_word(entry);
        if bytes == 0 {
            return;
        }
        let destination = host.queue_word(entry + 5);
        let upload = TilemapUpload {
            bytes,
            source: Source {
                address: host.queue_word(entry + 2),
                bank: host.queue_byte(entry + 4),
            },
            column: destination & 0x8000 != 0,
            vram_address: destination,
        };
        host.upload_tilemap(upload);
        entry += 7;
    }
}

/// The real live owner selected by a DMA bus byte. This carries no captured
/// content or manufactured asset identity and is resolved after each effect.
#[derive(Clone, Copy)]
pub enum LiveByte {
    Hud { tile: usize, lane: usize },
    Scrolling { word: usize, lane: usize },
}
pub fn live_byte(bank: u8, address: u16) -> Option<LiveByte> {
    if bank != 0x7e {
        return None;
    }
    let at = usize::from(address);
    if (0xc608..0xc6c8).contains(&at) {
        let offset = at - 0xc608;
        Some(LiveByte::Hud {
            tile: offset / 2,
            lane: offset % 2,
        })
    } else if (0xc8c8..0xcad8).contains(&at) {
        let offset = at - 0xc8c8;
        Some(LiveByte::Scrolling {
            word: offset / 2,
            lane: offset % 2,
        })
    } else {
        None
    }
}
