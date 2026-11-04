//! Word endpoints over captured RAM offsets, ROM images and tables, for the
//! cinematic transfers.
use sm_data::cartridge::RomImage;
pub use sm_data::cartridge::lorom_offset;

/// The loaded cartridge image, for ROM word reads.
pub trait LoadedRom {
    fn loaded_rom_image(&self) -> RomImage;
}

#[derive(Clone)]
pub enum WordEndpoint<'a> {
    // The flag selects native byte order; false selects SNES little-endian.
    Ram(isize, bool),
    Rom(RomImage, isize),
    Table(&'a [u16], isize),
    Value(u16),
}

impl WordEndpoint<'_> {
    pub fn shifted(&self, words: isize) -> Self {
        let mut result = self.clone();
        match &mut result {
            Self::Ram(base, _) | Self::Rom(_, base) | Self::Table(_, base) => {
                *base = base.checked_add(words.checked_mul(2).unwrap()).unwrap();
            }
            Self::Value(_) => {}
        }
        result
    }

    // RAM is handled at the transfer call site, without borrowing the RAM array.
    pub fn read(&self, byte_offset: isize) -> u16 {
        match self {
            Self::Rom(image, base) => {
                let offset = address(*base, byte_offset);
                u16::from_ne_bytes([image.read(offset), image.read(offset + 1)])
            }
            Self::Table(words, base) => {
                let offset = address(*base, byte_offset);
                u16::from_ne_bytes([
                    words[offset / 2].to_ne_bytes()[offset & 1],
                    words[offset.div_ceil(2)].to_ne_bytes()[(offset + 1) & 1],
                ])
            }
            Self::Value(value) => *value,
            Self::Ram(..) => unreachable!("RAM reads stay at the transfer call site"),
        }
    }
}

/// `N` words of the loaded cartridge from LoROM `address`, in native byte
/// order, as the `rom` endpoint reads them.
pub fn rom_words<const N: usize, H: LoadedRom + ?Sized>(host: &H, address: u32) -> [u16; N] {
    let image = host.loaded_rom_image();
    let base = usize::try_from(lorom_offset(address)).expect("LoROM offset");
    std::array::from_fn(|index| {
        image.table_word(base, isize::try_from(index).expect("ROM word index"))
    })
}

/// The little-endian Session word holding the bytes of `value` stored in
/// native byte order, as a `native` endpoint stores it.
pub fn stored_native(value: u16) -> u16 {
    u16::from_le_bytes(value.to_ne_bytes())
}

pub fn address(base: isize, displacement: isize) -> usize {
    base.checked_add(displacement).unwrap().try_into().unwrap()
}
