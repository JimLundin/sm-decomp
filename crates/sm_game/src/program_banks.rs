//! Reads of the interpreted program banks through their imported records.
//!
//! A read uses the imported bank while the loaded image is its allocation and
//! the bytes lie in $8000-$FFFF; any other read, including the low-window
//! diagnostic and a read running past $FFFF, keeps the original record read.
use crate::imported::Imported;
use crate::records::Records;
use crate::visual_host::{Content, Record, Source};
use sm_data::program_bank::ProgramBank;

/// An interpreter's program bank.
#[derive(Clone, Copy)]
pub enum Bank {
    /// $8D: the palette effect programs.
    Palette,
    /// $87: the animated tile programs.
    Animated,
    /// $B4: the sprite object programs.
    Sprite,
}

impl Bank {
    pub const fn number(self) -> u8 {
        match self {
            Self::Palette => sm_data::palette_programs::BANK,
            Self::Animated => sm_data::animated_programs::BANK,
            Self::Sprite => sm_data::sprite_programs::BANK,
        }
    }
}

pub fn import(image: &sm_data::cartridge::RomImage, bank: Bank) -> Option<Imported<ProgramBank>> {
    Imported::import(
        image,
        sm_data::program_bank::address(bank.number()),
        sm_data::program_bank::BYTES,
        ProgramBank::import,
    )
}

fn imported(
    host: &(impl Records + ?Sized),
    bank: Bank,
    cursor: u16,
    displacement: usize,
) -> Option<(&ProgramBank, u16)> {
    if cursor & 0x8000 == 0 {
        return None;
    }
    let at = u16::try_from(usize::from(cursor) + displacement).ok()?;
    let records = host.records();
    let table = match bank {
        Bank::Palette => &records.palette_programs,
        Bank::Animated => &records.animated_programs,
        Bank::Sprite => &records.sprite_programs,
    };
    Some((Imported::from(table, host.loaded_image())?, at))
}

/// The program word `displacement` bytes past `cursor` in `bank`.
pub fn word(
    host: &mut (impl Content + Records),
    bank: Bank,
    cursor: u16,
    displacement: usize,
) -> u16 {
    if let Some(word) = imported(host, bank, cursor, displacement).and_then(|(p, at)| p.word(at)) {
        return word;
    }
    host.record(Source::Loaded, bank.number(), cursor)
        .native_word(displacement)
}

/// The program byte `displacement` bytes past `cursor` in `bank`.
pub fn byte(
    host: &mut (impl Content + Records),
    bank: Bank,
    cursor: u16,
    displacement: usize,
) -> u8 {
    if let Some(byte) = imported(host, bank, cursor, displacement).and_then(|(p, at)| p.byte(at)) {
        return byte;
    }
    host.record(Source::Loaded, bank.number(), cursor)
        .byte(displacement)
}
