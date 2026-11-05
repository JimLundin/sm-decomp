//! Compat sink for `sm_game::display::transfer`: each decoded transfer becomes
//! the original DMA channel 1 register sequence, in the original order.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::display_commands::{Lease, Wire, byte, word};
use crate::game_state::Game;
use crate::rtl::{write_reg, write_reg_word};
use crate::snes::registers::*;
use sm_game::display::control::Brightness;
use sm_game::display::transfer::{
    Mode7Target, Mode7Upload, ScrollingLayout, TilemapUpload, TransferHost, VramReadback,
};

/// One Mode 7 queue record's transfer through `wire`.
fn upload_mode7(upload: Mode7Upload, wire: &mut impl Wire) {
    byte(DMAP1, upload.mode, wire);
    word(A1T1L, upload.source.address, wire);
    byte(A1B1, upload.source.bank, wire);
    word(DAS1L, upload.bytes, wire);
    match upload.target {
        Mode7Target::Vram {
            high_byte,
            address,
            increment,
        } => {
            byte(BBAD1, if high_byte { 0x19 } else { 0x18 }, wire);
            word(VMADDL, address, wire);
            byte(VMAIN, increment, wire);
        }
        Mode7Target::Cgram { index } => {
            byte(BBAD1, 0x22, wire);
            byte(CGADD, index, wire);
        }
    }
    byte(MDMAEN, 2, wire);
}

/// One VRAM readback record's transfer through `wire`. The VRAM address write
/// and dummy data-port read prime the readback latch before the DMA.
fn read_vram(readback: VramReadback, wire: &mut impl Wire) {
    let destination = readback.destination;
    word(VMADDL, readback.vram_address, wire);
    wire.read(RDVRAML);
    wire.read(RDVRAML + 1);
    word(
        DMAP1,
        u16::from_le_bytes([readback.control, readback.port]),
        wire,
    );
    word(A1T1L, destination.address, wire);
    // This overlapping word sets the high address byte again, then the bank.
    word(
        A1T1H,
        u16::from_le_bytes([destination.address.to_le_bytes()[1], destination.bank]),
        wire,
    );
    word(DAS1L, readback.bytes, wire);
    word(DAS10, 0, wire);
    word(A2A1H, 0, wire);
    byte(MDMAEN, 2, wire);
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

/// What the Mode 7 and readback drains read and store between their register
/// writes: the frame lease's Session, through its transfer port (`Lease`).
pub(crate) trait DrainHost: Wire {
    /// A queue record byte, as the drain reads it between transfers.
    fn queue_byte(&self, at: usize) -> u8;
    fn store_queue_byte(&mut self, at: usize, value: u8);
    /// A queue's tail word.
    fn tail(&self, at: usize) -> u16;
    fn set_tail(&mut self, at: usize, value: u16);
}

/// The lease's drains: the queues and their tails are the lease's bytes, each
/// DMA start reads what the port published, and a readback's work RAM bytes
/// return to the Session.
impl DrainHost for Lease<'_, '_> {
    fn queue_byte(&self, at: usize) -> u8 {
        self.0.byte(at)
    }
    fn store_queue_byte(&mut self, at: usize, value: u8) {
        self.0.set_byte(at, value);
    }
    fn tail(&self, at: usize) -> u16 {
        self.0.word(at)
    }
    fn set_tail(&mut self, at: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.0.set_byte(at, low);
        self.0.set_byte(at + 1, high);
    }
}

/// The Mode 7 and readback drains over a drain host.
pub(crate) struct Queue<'a, H>(pub(crate) &'a mut H);

impl<H: DrainHost> TransferHost for Queue<'_, H> {
    fn byte(&self, at: usize) -> u8 {
        self.0.queue_byte(at)
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn upload_mode7(&mut self, upload: Mode7Upload) {
        upload_mode7(upload, self.0);
    }
    fn read_vram(&mut self, readback: VramReadback) {
        read_vram(readback, self.0);
    }
    fn upload_tilemap(&mut self, upload: TilemapUpload) {
        upload_tilemap(upload, self.0);
    }
}

/// What the tilemap flush reads and stores between its register writes: the
/// lease's Session, through its transfer port (`Lease`).
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
/// Samus's NMI upload over its descriptor bytes ($7E:071D..0722: the two
/// enable flags, then the two descriptor pointers), as the lease holds them,
/// with Game as the device and the cartridge.
struct NativeSamusUpload<'a>(&'a mut Game, [u8; 6]);
impl NativeSamusUpload<'_> {
    fn table(
        &self,
        address: u32,
        offset: usize,
        width: usize,
    ) -> sm_game::samus_drawing::RoleTable {
        let (image, base) = self.0.preflight_loaded_rom(address, offset, width);
        sm_game::samus_drawing::capture_table(
            &self.0.samus_content(),
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
        Ok(self.1[match half {
            sm_game::scene::samus_body::Half::Top => 0,
            sm_game::scene::samus_body::Half::Bottom => 1,
        }])
    }
    fn descriptor(
        &mut self,
        half: sm_game::scene::samus_body::Half,
    ) -> Result<Self::Descriptor, Self::Error> {
        let (pointer, vram) = match half {
            sm_game::scene::samus_body::Half::Top => (2, 0x6000),
            sm_game::scene::samus_body::Half::Bottom => (4, 0x6080),
        };
        let address = u16::from_le_bytes([self.1[pointer], self.1[pointer + 1]]);
        unsafe {
            let _ = crate::rtl::rom_bank_ptr::<0x92>(address, self.0);
        }
        if self.0.samus_execution().unwrap().is_some() {
            let image = self.0.capture_native_rom();
            let _ = sm_game::samus_drawing::capture_table(
                &self.0.samus_content(),
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
/// Samus's NMI tile upload over `descriptors`, the bytes $7E:071D..0722 as the
/// lease holds them: it writes registers and VRAM, and reads the cartridge.
/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn upload_samus(descriptors: [u8; 6], game: &mut Game) {
    sm_game::scene::samus_tile_upload::upload(&mut NativeSamusUpload(game, descriptors)).unwrap();
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn set_brightness(brightness: Brightness, game: &mut Game) {
    unsafe { write_reg(INIDISP as u16, brightness.byte(), game) }
}
