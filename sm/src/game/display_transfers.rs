//! Compat sink for `sm_game::display::transfer`: each decoded transfer becomes
//! the original DMA channel 1 register sequence, in the original order.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as at;
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

/// Requires a live, exclusively accessed SNES backend. DMAP1 was set once for
/// the whole queue by the caller.
unsafe fn upload_tilemap(upload: TilemapUpload, game: &mut Game) {
    unsafe {
        write_reg_word(DAS1L as u16, upload.bytes, game);
        write_reg_word(A1T1L as u16, upload.source.address, game);
        write_reg(A1B1 as u16, upload.source.bank, game);
        // This word write also clears VMADDL before the full destination write.
        write_reg_word(VMAIN as u16, if upload.column { 0x81 } else { 0x80 }, game);
        write_reg_word(VMADDL as u16, upload.vram_address, game);
        write_reg(MDMAEN as u16, 2, game);
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
        write_reg_word(A1T1L as u16, at!(game, parameters + 8;), game);
        let wrapped_size = at!(game, parameters + 2;);
        if wrapped_size != 0 {
            write_reg_word(DAS1L as u16, wrapped_size, game);
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
        unsafe { write_reg(VMAIN as u16, 0x80, self.0) };
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
        unsafe { write_reg_word(VMADDL as u16, address, self.0) };
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
        unsafe { write_reg(MDMAEN as u16, 2, self.0) };
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
