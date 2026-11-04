//! Compat sink for `sm_game::display::transfer`: each decoded transfer becomes
//! the original DMA channel 1 register sequence, in the original order.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::display_commands::{Lease, Wire, byte, word};
use crate::game_state::Game;
use crate::rtl::{read_reg_word, write_reg, write_reg_word};
use crate::snes::registers::*;
use sm_game::display::control::{Brightness, FixedColor};
use sm_game::display::transfer::{
    Mode7Target, Mode7Upload, ScrollingLayout, TilemapUpload, TransferHost, VramReadback,
};

/// Requires a live, exclusively accessed SNES backend.
unsafe fn upload_mode7(upload: Mode7Upload, game: &mut Game) {
    unsafe {
        write_reg(DMAP1 as u16, upload.mode, game);
        write_reg_word(A1T1L as u16, upload.source.address, game);
        write_reg(A1B1 as u16, upload.source.bank, game);
        write_reg_word(DAS1L as u16, upload.bytes, game);
        match upload.target {
            Mode7Target::Vram {
                high_byte,
                address,
                increment,
            } => {
                write_reg(BBAD1 as u16, if high_byte { 0x19 } else { 0x18 }, game);
                write_reg_word(VMADDL as u16, address, game);
                write_reg(VMAIN as u16, increment, game);
            }
            Mode7Target::Cgram { index } => {
                write_reg(BBAD1 as u16, 0x22, game);
                write_reg(CGADD as u16, index, game);
            }
        }
        write_reg(MDMAEN as u16, 2, game);
    }
}

/// Requires a live, exclusively accessed SNES backend. The VRAM address write
/// and dummy data-port read prime the readback latch before the DMA.
unsafe fn read_vram(readback: VramReadback, game: &mut Game) {
    let destination = readback.destination;
    unsafe {
        write_reg_word(VMADDL as u16, readback.vram_address, game);
        read_reg_word(RDVRAML as u16, game);
        write_reg_word(
            DMAP1 as u16,
            u16::from_le_bytes([readback.control, readback.port]),
            game,
        );
        write_reg_word(A1T1L as u16, destination.address, game);
        // This overlapping word sets the high address byte again, then the bank.
        write_reg_word(
            A1T1H as u16,
            u16::from_le_bytes([destination.address.to_le_bytes()[1], destination.bank]),
            game,
        );
        write_reg_word(DAS1L as u16, readback.bytes, game);
        write_reg_word(DAS10 as u16, 0, game);
        write_reg_word(A2A1H as u16, 0, game);
        write_reg(MDMAEN as u16, 2, game);
    }
}

/// One queued tilemap upload through `wire`. DMAP1 was set once for the whole
/// queue by the caller.
fn upload_tilemap(upload: TilemapUpload, wire: &mut impl Wire) {
    use sm_game::display::video::Upload;
    wire.producing();
    word(DAS1L, upload.bytes, wire);
    word(A1T1L, upload.source.address, wire);
    byte(A1B1, upload.source.bank, wire);
    // This word write also clears VMADDL before the full destination write.
    word(VMAIN, if upload.column { 0x81 } else { 0x80 }, wire);
    word(VMADDL, upload.vram_address, wire);
    byte(MDMAEN, 2, wire);
    wire.produced(
        "tilemaps",
        &[Upload::Tiles {
            address: upload.vram_address,
            increment: if upload.column { 32 } else { 1 },
            source: upload.source,
            bytes: upload.bytes,
        }],
    );
}

/// A queue read from raw work RAM, as the Mode 7 and readback drains read it.
pub(crate) struct RawQueue<'a>(&'a mut Game);

impl<'a> RawQueue<'a> {
    /// Requires a live, exclusively accessed SNES backend for the host's lifetime.
    pub(crate) unsafe fn new(game: &'a mut Game) -> Self {
        Self(game)
    }
}

macro_rules! sink {
    ($host:ident, $byte:expr) => {
        impl TransferHost for $host<'_> {
            fn byte(&self, at: usize) -> u8 {
                ($byte)(&*self.0, at)
            }
            fn word(&self, at: usize) -> u16 {
                u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
            }
            fn upload_mode7(&mut self, upload: Mode7Upload) {
                // Constructing the host required the caller's backend capability.
                unsafe { upload_mode7(upload, self.0) }
            }
            fn read_vram(&mut self, readback: VramReadback) {
                unsafe { read_vram(readback, self.0) }
            }
            fn upload_tilemap(&mut self, upload: TilemapUpload) {
                upload_tilemap(upload, self.0)
            }
        }
    };
}
sink!(RawQueue, |game: &Game, at| game.ram().read_byte(at));

/// What the tilemap flush reads and stores between its register writes: Game's
/// compatibility projection, or the lease's Session through its transfer port.
pub(crate) trait FlushHost: Wire {
    fn flush_byte(&self, at: usize) -> u8;
    fn flush_word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.flush_byte(at), self.flush_byte(at + 1)])
    }
    fn tilemap_tail(&self) -> u16;
    fn set_tilemap_tail(&mut self, tail: u16);
    /// The queue's zero end word at byte cursor `tail`.
    fn store_sentinel(&mut self, tail: u16);
    /// Consume a scrolling request's enable byte.
    fn clear_enable(&mut self, at: usize);
}

impl crate::game::display_transfers::FlushHost for Game {
    fn flush_byte(&self, at: usize) -> u8 {
        self.compatibility_byte(at)
    }
    fn tilemap_tail(&self) -> u16 {
        self.vram_write_queue_tail()
    }
    fn set_tilemap_tail(&mut self, tail: u16) {
        self.set_vram_write_queue_tail(tail);
    }
    fn store_sentinel(&mut self, tail: u16) {
        self.write_vram_queue_word(tail, 0, 0);
    }
    fn clear_enable(&mut self, at: usize) {
        self.compatibility_write_owned_byte(at, 0);
    }
}

/// The lease's flush: the queue, its tail and the requests are the lease's
/// bytes, and each DMA start reads what the port published.
impl FlushHost for Lease<'_, '_> {
    fn flush_byte(&self, at: usize) -> u8 {
        self.0.byte(at)
    }
    fn tilemap_tail(&self) -> u16 {
        sm_game::display::transfer::TilemapQueue::tilemap_queue_tail(&*self.0)
    }
    fn set_tilemap_tail(&mut self, tail: u16) {
        sm_game::display::transfer::TilemapQueue::set_tilemap_queue_tail(&mut *self.0, tail);
    }
    fn store_sentinel(&mut self, tail: u16) {
        let at = 0xd0 + usize::from(tail);
        self.0.set_byte(at, 0);
        self.0.set_byte(at + 1, 0);
    }
    fn clear_enable(&mut self, at: usize) {
        sm_game::display_refresh::DisplayOwner::write_owned_byte(&mut *self.0, at, 0);
    }
}

/// The queued tilemap drain over a flush host; it uploads tilemaps alone.
struct Drain<'a, H>(&'a mut H);

impl<H: FlushHost> TransferHost for Drain<'_, H> {
    fn byte(&self, at: usize) -> u8 {
        self.0.flush_byte(at)
    }
    fn word(&self, at: usize) -> u16 {
        self.0.flush_word(at)
    }
    fn upload_mode7(&mut self, _upload: Mode7Upload) {
        unreachable!("the tilemap drain uploads tilemaps alone")
    }
    fn read_vram(&mut self, _readback: VramReadback) {
        unreachable!("the tilemap drain uploads tilemaps alone")
    }
    fn upload_tilemap(&mut self, upload: TilemapUpload) {
        upload_tilemap(upload, self.0);
    }
}

/// Drain the tilemap queue, then the four scrolling requests, in the original's
/// order: the queue's end word, DMAP1 once, each queued upload, the cleared
/// tail, then the column and row requests with their VRAM steps.
pub(crate) fn flush_tilemaps(host: &mut impl FlushHost) {
    let tail = host.tilemap_tail();
    if tail != 0 {
        // The sentinel uses the byte tail, which need not lie on an entry boundary.
        host.store_sentinel(tail);
        word(DMAP1, 0x1801, host);
        sm_game::display::transfer::drain_tilemaps(&mut Drain(host));
    }
    host.set_tilemap_tail(0);
    for (increment, layout, buffers) in [
        (
            0x81,
            ScrollingLayout::Column,
            [(0x956, [0xc8c8, 0xc908]), (0x972, [0xc9d0, 0xca10])],
        ),
        (
            0x80,
            ScrollingLayout::Row,
            [(0x964, [0xc948, 0xc98c]), (0x980, [0xca50, 0xca94])],
        ),
    ] {
        byte(VMAIN, increment, host);
        for (parameters, sources) in buffers {
            upload_scrolling_tilemap(parameters, sources, layout, host);
        }
    }
}

/// One scrolling request's uploads. Its RAM parameters are read between the
/// register writes exactly where the original read them.
fn upload_scrolling_tilemap(
    parameters: usize,
    initial_sources: [u16; 2],
    layout: ScrollingLayout,
    host: &mut impl FlushHost,
) {
    let enable = parameters + 12;
    if host.flush_byte(enable) == 0 {
        return;
    }
    // Only the low enable byte is consumed, before reading transfer parameters.
    host.clear_enable(enable);
    // Sizes, destinations and wrapped sources are three pairs of RAM words.
    let destination = host.flush_word(parameters + 4);
    word(VMADDL, destination, host);
    word(DMAP1, 0x1801, host);
    word(A1T1L, initial_sources[0], host);
    byte(A1B1, 0x7e, host);
    let size = host.flush_word(parameters);
    // A zero unwrapped size still triggers a full 65,536-byte DMA.
    word(DAS1L, size, host);
    byte(MDMAEN, 2, host);
    word(VMADDL, layout.second_half_destination(destination), host);
    word(DAS1L, size, host);
    word(A1T1L, initial_sources[1], host);
    byte(MDMAEN, 2, host);

    // Prepare the wrapped source even when there is no wrapped transfer.
    word(A1T1L, host.flush_word(parameters + 8), host);
    let wrapped_size = host.flush_word(parameters + 2);
    if wrapped_size != 0 {
        word(DAS1L, wrapped_size, host);
        let wrapped_destination = host.flush_word(parameters + 6);
        word(VMADDL, wrapped_destination, host);
        byte(MDMAEN, 2, host);
        word(
            VMADDL,
            layout.second_half_destination(wrapped_destination),
            host,
        );
        word(DAS1L, wrapped_size, host);
        word(A1T1L, host.flush_word(parameters + 10), host);
        byte(MDMAEN, 2, host);
    }
}

#[derive(Clone, Copy)]
struct NativeSamusDescriptor {
    address: u32,
    vram: u16,
}
struct NativeSamusUpload<'a>(&'a mut Game);
impl NativeSamusUpload<'_> {
    fn table(
        &self,
        address: u32,
        offset: usize,
        width: usize,
    ) -> sm_game::samus_drawing::RoleTable {
        let (image, base) = self.0.preflight_loaded_rom(address, offset, width);
        sm_game::samus_drawing::capture_table(
            self.0,
            sm_game::scene::samus_content::Role::UploadDescriptor,
            sm_game::scene::samus_content::Registration::Loaded,
            crate::rom::RomRecord { image, base },
        )
    }
}
impl sm_game::scene::samus_tile_upload::Host for NativeSamusUpload<'_> {
    type Error = std::convert::Infallible;
    type Descriptor = NativeSamusDescriptor;
    type Source = u16;
    fn vmain(&mut self) -> Result<(), Self::Error> {
        crate::rtl::samus_vram_mode(self.0);
        Ok(())
    }
    fn enabled(&mut self, half: sm_game::scene::samus_body::Half) -> Result<u8, Self::Error> {
        Ok(self.0.compatibility_byte(match half {
            sm_game::scene::samus_body::Half::Top => 0x71d,
            sm_game::scene::samus_body::Half::Bottom => 0x71e,
        }))
    }
    fn descriptor(
        &mut self,
        half: sm_game::scene::samus_body::Half,
    ) -> Result<Self::Descriptor, Self::Error> {
        let (pointer, vram) = match half {
            sm_game::scene::samus_body::Half::Top => (0x71f, 0x6000),
            sm_game::scene::samus_body::Half::Bottom => (0x721, 0x6080),
        };
        let address = u16::from_le_bytes([
            self.0.compatibility_byte(pointer),
            self.0.compatibility_byte(pointer + 1),
        ]);
        unsafe {
            let _ = crate::rtl::rom_bank_ptr::<0x92>(address, self.0);
        }
        if self.0.samus_execution().unwrap().is_some() {
            let image = self.0.capture_native_rom();
            let _ = sm_game::samus_drawing::capture_table(
                self.0,
                sm_game::scene::samus_content::Role::UploadDescriptor,
                sm_game::scene::samus_content::Registration::Native,
                crate::rom::RomRecord {
                    image,
                    base: crate::rtl::lorom_offset(0x920000 | u32::from(address)) as usize,
                },
            );
        }
        Ok(NativeSamusDescriptor {
            address: 0x928000 | u32::from(address & 0x7fff),
            vram,
        })
    }
    fn destination(
        &mut self,
        d: Self::Descriptor,
        p: sm_game::scene::samus_tile_upload::Part,
    ) -> Result<(), Self::Error> {
        let address = match p {
            sm_game::scene::samus_tile_upload::Part::First => d.vram,
            sm_game::scene::samus_tile_upload::Part::Second => d.vram.wrapping_add(0x100),
        };
        crate::rtl::samus_vram_pointer(address, self.0);
        Ok(())
    }
    fn configure_dma(&mut self) -> Result<(), Self::Error> {
        unsafe { write_reg_word(DMAP1 as u16, 0x1801, self.0) };
        Ok(())
    }
    fn source(&mut self, d: Self::Descriptor) -> Result<u16, Self::Error> {
        Ok(self.table(d.address, 0, 2).word(0, true))
    }
    fn advance_source(&mut self, _: Self::Descriptor, s: u16, c: u16) -> Result<u16, Self::Error> {
        Ok(s.wrapping_add(c))
    }
    fn write_source(&mut self, s: &u16) -> Result<(), Self::Error> {
        unsafe { write_reg_word(A1T1L as u16, *s, self.0) };
        Ok(())
    }
    fn bank(&mut self, d: Self::Descriptor) -> Result<(), Self::Error> {
        let bank = self.table(d.address, 2, 1).byte(0);
        unsafe { write_reg(A1B1 as u16, bank, self.0) };
        Ok(())
    }
    fn count(
        &mut self,
        d: Self::Descriptor,
        p: sm_game::scene::samus_tile_upload::Part,
    ) -> Result<u16, Self::Error> {
        Ok(self
            .table(
                d.address,
                match p {
                    sm_game::scene::samus_tile_upload::Part::First => 3,
                    sm_game::scene::samus_tile_upload::Part::Second => 5,
                },
                2,
            )
            .word(0, true))
    }
    fn write_count(&mut self, c: u16) -> Result<(), Self::Error> {
        unsafe { write_reg_word(DAS1L as u16, c, self.0) };
        Ok(())
    }
    fn dma(
        &mut self,
        _: Self::Descriptor,
        _: sm_game::scene::samus_tile_upload::Part,
        _: u16,
        _: u16,
    ) -> Result<(), Self::Error> {
        // Keep the original channel state, physical pre-reads and per-byte drain.
        // Only its VRAM sink uses the typed port implementation.
        let ppu = self.0.execution_parts().0.unwrap().selected_video();
        crate::snes::ppu::samus_vram_dma(ppu, || unsafe { write_reg(MDMAEN as u16, 2, self.0) });
        crate::snes::ppu::produced(ppu, "samus tiles", |_, _| {});
        Ok(())
    }
}
/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn upload_samus(game: &mut Game) {
    sm_game::scene::samus_tile_upload::upload(&mut NativeSamusUpload(game)).unwrap();
}

/// Requires a live, exclusively accessed SNES backend: fill VRAM's low bytes
/// with `fill` from address 0, resetting the address as one word or two bytes.
pub(crate) unsafe fn fill_vram_low_bytes(fill: u8, word_reset: bool, game: &mut Game) {
    unsafe {
        write_reg(VMAIN as u16, 0, game);
        if word_reset {
            write_reg_word(VMADDL as u16, 0, game);
        } else {
            write_reg(VMADDL as u16, 0, game);
            write_reg(VMADDH as u16, 0, game);
        }
        for _ in 0i16..0x4000i16 {
            write_reg(VMDATAL as u16, fill, game);
        }
    }
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn set_brightness(brightness: Brightness, game: &mut Game) {
    unsafe { write_reg(INIDISP as u16, brightness.byte(), game) }
}

/// Requires a live, exclusively accessed SNES backend. One COLDATA write per
/// colour, in order.
pub(crate) unsafe fn set_fixed_color(colors: &[FixedColor], game: &mut Game) {
    for color in colors {
        unsafe { write_reg(COLDATA as u16, color.byte(), game) }
    }
}

pub(crate) use sm_game::display::control::DisplayRegister;
