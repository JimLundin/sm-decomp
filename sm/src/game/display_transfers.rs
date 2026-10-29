//! Compat sink for `sm_game::display::transfer`: each decoded transfer becomes
//! the original DMA channel 1 register sequence, in the original order.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as at;
use crate::rtl::{ReadRegWord, WriteReg, WriteRegWord};
use crate::snes::registers::*;
use sm_game::display::control::{Brightness, FixedColor, Mode7Transform};
use sm_game::display::transfer::{
    Mode7Target, Mode7Upload, ScrollingLayout, TilemapUpload, TransferHost, VramReadback,
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

/// Requires a live, exclusively accessed SNES backend. RAM parameters are read
/// between the register writes exactly where the original read them.
pub(crate) unsafe fn upload_scrolling_tilemap(
    parameters: usize,
    initial_sources: [u16; 2],
    layout: ScrollingLayout,
    game: &mut Game,
) {
    unsafe {
        let enable = parameters + 12;
        if game.compatibility_byte(enable) == 0 {
            return;
        }
        // Only the low enable byte is consumed, before reading transfer parameters.
        game.compatibility_write_owned_byte(enable, 0);
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
    ) -> crate::game::samus_animation::NativeTable {
        let (image, base) = self.0.preflight_loaded_rom(address, offset, width);
        crate::game::samus_animation::NativeTable::capture(
            self.0,
            crate::game::scene::samus_content::Role::UploadDescriptor,
            crate::game::scene::samus_content::Registration::Loaded,
            crate::rom::RomRecord { image, base },
        )
    }
}
impl crate::game::scene::samus_tile_upload::Host for NativeSamusUpload<'_> {
    type Error = std::convert::Infallible;
    type Descriptor = NativeSamusDescriptor;
    type Source = u16;
    fn vmain(&mut self) -> Result<(), Self::Error> {
        unsafe { WriteReg(VMAIN as u16, 0x80, self.0) };
        Ok(())
    }
    fn enabled(&mut self, half: crate::game::scene::samus_body::Half) -> Result<u8, Self::Error> {
        Ok(self.0.compatibility_byte(match half {
            crate::game::scene::samus_body::Half::Top => 0x71d,
            crate::game::scene::samus_body::Half::Bottom => 0x71e,
        }))
    }
    fn descriptor(
        &mut self,
        half: crate::game::scene::samus_body::Half,
    ) -> Result<Self::Descriptor, Self::Error> {
        let (pointer, vram) = match half {
            crate::game::scene::samus_body::Half::Top => (0x71f, 0x6000),
            crate::game::scene::samus_body::Half::Bottom => (0x721, 0x6080),
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
            let _ = crate::game::samus_animation::NativeTable::capture(
                self.0,
                crate::game::scene::samus_content::Role::UploadDescriptor,
                crate::game::scene::samus_content::Registration::Native,
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
        p: crate::game::scene::samus_tile_upload::Part,
    ) -> Result<(), Self::Error> {
        let address = match p {
            crate::game::scene::samus_tile_upload::Part::First => d.vram,
            crate::game::scene::samus_tile_upload::Part::Second => d.vram.wrapping_add(0x100),
        };
        unsafe { WriteRegWord(VMADDL as u16, address, self.0) };
        Ok(())
    }
    fn configure_dma(&mut self) -> Result<(), Self::Error> {
        unsafe { WriteRegWord(DMAP1 as u16, 0x1801, self.0) };
        Ok(())
    }
    fn source(&mut self, d: Self::Descriptor) -> Result<u16, Self::Error> {
        Ok(self.table(d.address, 0, 2).word(0, true))
    }
    fn advance_source(&mut self, _: Self::Descriptor, s: u16, c: u16) -> Result<u16, Self::Error> {
        Ok(s.wrapping_add(c))
    }
    fn write_source(&mut self, s: &u16) -> Result<(), Self::Error> {
        unsafe { WriteRegWord(A1T1L as u16, *s, self.0) };
        Ok(())
    }
    fn bank(&mut self, d: Self::Descriptor) -> Result<(), Self::Error> {
        let bank = self.table(d.address, 2, 1).byte(0);
        unsafe { WriteReg(A1B1 as u16, bank, self.0) };
        Ok(())
    }
    fn count(
        &mut self,
        d: Self::Descriptor,
        p: crate::game::scene::samus_tile_upload::Part,
    ) -> Result<u16, Self::Error> {
        Ok(self
            .table(
                d.address,
                match p {
                    crate::game::scene::samus_tile_upload::Part::First => 3,
                    crate::game::scene::samus_tile_upload::Part::Second => 5,
                },
                2,
            )
            .word(0, true))
    }
    fn write_count(&mut self, c: u16) -> Result<(), Self::Error> {
        unsafe { WriteRegWord(DAS1L as u16, c, self.0) };
        Ok(())
    }
    fn dma(
        &mut self,
        _: Self::Descriptor,
        _: crate::game::scene::samus_tile_upload::Part,
        _: u16,
        _: u16,
    ) -> Result<(), Self::Error> {
        unsafe { WriteReg(MDMAEN as u16, 2, self.0) };
        Ok(())
    }
}
/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn upload_samus(game: &mut Game) {
    crate::game::scene::samus_tile_upload::upload(&mut NativeSamusUpload(game)).unwrap();
}

/// Requires a live, exclusively accessed SNES backend: fill VRAM's low bytes
/// with `fill` from address 0, resetting the address as one word or two bytes.
pub(crate) unsafe fn fill_vram_low_bytes(fill: u8, word_reset: bool, game: &mut Game) {
    unsafe {
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
    }
}

/// Requires a live, exclusively accessed SNES backend.
pub(crate) unsafe fn set_brightness(brightness: Brightness, game: &mut Game) {
    unsafe { WriteReg(INIDISP as u16, brightness.byte(), game) }
}

/// Requires a live, exclusively accessed SNES backend. One COLDATA write per
/// colour, in order.
pub(crate) unsafe fn set_fixed_color(colors: &[FixedColor], game: &mut Game) {
    for color in colors {
        unsafe { WriteReg(COLDATA as u16, color.byte(), game) }
    }
}

/// Requires a live, exclusively accessed SNES backend. Each register write is
/// followed by its RAM mirror word before the next register, as the original
/// did.
pub(crate) unsafe fn set_mode7_transform(transform: Mode7Transform, game: &mut Game) {
    use crate::ram_map::words;
    let registers = [M7A, M7B, M7C, M7D, M7X, M7Y];
    let mirrors = [
        words::reg_M7A,
        words::reg_M7B,
        words::reg_M7C,
        words::reg_M7D,
        words::reg_M7X,
        words::reg_M7Y,
    ];
    for ((register, mirror), value) in registers.into_iter().zip(mirrors).zip(transform.words()) {
        unsafe { WriteRegWord(register as u16, value, game) };
        crate::ram_map::checked_word!(game, mirror; = value);
    }
}
