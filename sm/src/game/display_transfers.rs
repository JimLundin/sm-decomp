//! Compat sink for `sm_game::display::transfer`: each decoded transfer becomes
//! the original DMA channel 1 register sequence, in the original order.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::rtl::{ReadRegWord, WriteReg, WriteRegWord};
use crate::snes::registers::*;
use sm_game::display::transfer::{
    Mode7Target, Mode7Upload, TilemapUpload, TransferHost, VramReadback,
};

/// Requires a live, exclusively accessed SNES backend.
unsafe fn upload_mode7(upload: Mode7Upload, game: &mut Game) {
    unsafe {
        WriteReg(DMAP1 as u16, upload.mode, game);
        WriteRegWord(A1T1L as u16, upload.source.address, game);
        WriteReg(A1B1 as u16, upload.source.bank, game);
        WriteRegWord(DAS1L as u16, upload.bytes, game);
        match upload.target {
            Mode7Target::Vram {
                high_byte,
                address,
                increment,
            } => {
                WriteReg(BBAD1 as u16, if high_byte { 0x19 } else { 0x18 }, game);
                WriteRegWord(VMADDL as u16, address, game);
                WriteReg(VMAIN as u16, increment, game);
            }
            Mode7Target::Cgram { index } => {
                WriteReg(BBAD1 as u16, 0x22, game);
                WriteReg(CGADD as u16, index, game);
            }
        }
        WriteReg(MDMAEN as u16, 2, game);
    }
}

/// Requires a live, exclusively accessed SNES backend. The VRAM address write
/// and dummy data-port read prime the readback latch before the DMA.
unsafe fn read_vram(readback: VramReadback, game: &mut Game) {
    let destination = readback.destination;
    unsafe {
        WriteRegWord(VMADDL as u16, readback.vram_address, game);
        ReadRegWord(RDVRAML as u16, game);
        WriteRegWord(
            DMAP1 as u16,
            u16::from_le_bytes([readback.control, readback.port]),
            game,
        );
        WriteRegWord(A1T1L as u16, destination.address, game);
        // This overlapping word sets the high address byte again, then the bank.
        WriteRegWord(
            A1T1H as u16,
            u16::from_le_bytes([destination.address.to_le_bytes()[1], destination.bank]),
            game,
        );
        WriteRegWord(DAS1L as u16, readback.bytes, game);
        WriteRegWord(DAS10 as u16, 0, game);
        WriteRegWord(A2A1H as u16, 0, game);
        WriteReg(MDMAEN as u16, 2, game);
    }
}

/// Requires a live, exclusively accessed SNES backend. DMAP1 was set once for
/// the whole queue by the caller.
unsafe fn upload_tilemap(upload: TilemapUpload, game: &mut Game) {
    unsafe {
        WriteRegWord(DAS1L as u16, upload.bytes, game);
        WriteRegWord(A1T1L as u16, upload.source.address, game);
        WriteReg(A1B1 as u16, upload.source.bank, game);
        // This word write also clears VMADDL before the full destination write.
        WriteRegWord(VMAIN as u16, if upload.column { 0x81 } else { 0x80 }, game);
        WriteRegWord(VMADDL as u16, upload.vram_address, game);
        WriteReg(MDMAEN as u16, 2, game);
    }
}

/// A queue read from raw work RAM, as the Mode 7 and readback drains read it.
pub(crate) struct RawQueue<'a>(&'a mut Game);

/// A queue read through the compatibility projection, as the tilemap drain reads it.
pub(crate) struct CompatibilityQueue<'a>(&'a mut Game);

impl<'a> RawQueue<'a> {
    /// Requires a live, exclusively accessed SNES backend for the host's lifetime.
    pub(crate) unsafe fn new(game: &'a mut Game) -> Self {
        Self(game)
    }
}

impl<'a> CompatibilityQueue<'a> {
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
                unsafe { upload_tilemap(upload, self.0) }
            }
        }
    };
}
sink!(RawQueue, |game: &Game, at| game.ram().read_byte(at));
sink!(CompatibilityQueue, |game: &Game, at| game
    .compatibility_byte(at));
