//! Ordered word transfers over captured RAM offsets, ROM images and tables.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use crate::rom::RomImage;

#[derive(Clone)]
pub(crate) enum WordEndpoint<'a> {
    // The flag selects native byte order; false selects SNES little-endian.
    Ram(isize, bool),
    Rom(RomImage, isize),
    Table(&'a [u16], isize),
    Value(u16),
}

impl WordEndpoint<'_> {
    pub(crate) fn shifted(&self, words: isize) -> Self {
        let mut result = self.clone();
        match &mut result {
            Self::Ram(base, _) | Self::Rom(_, base) | Self::Table(_, base) => {
                *base = base.checked_add(words.checked_mul(2).unwrap()).unwrap();
            }
            Self::Value(_) => {}
        }
        result
    }

    // RAM is handled at the macro call site, without borrowing the RAM array.
    pub(crate) fn read(&self, byte_offset: isize) -> u16 {
        match self {
            Self::Rom(image, base) => {
                let offset = address(*base, byte_offset);
                u16::from_ne_bytes([image.read(offset), image.read(offset + 1)])
            }
            Self::Table(words, base) => {
                let offset = address(*base, byte_offset);
                u16::from_ne_bytes([
                    words[offset / 2].to_ne_bytes()[offset & 1],
                    words[(offset + 1) / 2].to_ne_bytes()[(offset + 1) & 1],
                ])
            }
            Self::Value(value) => *value,
            Self::Ram(..) => unreachable!("RAM reads stay at the transfer call site"),
        }
    }
}

pub(crate) fn address(base: isize, displacement: isize) -> usize {
    base.checked_add(displacement).unwrap().try_into().unwrap()
}

// Callers retain any legacy address diagnostics before capturing the image.
pub(crate) fn rom_source(address: u32, game: &mut Game) -> WordEndpoint<'static> {
    WordEndpoint::Rom(
        game.loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone(),
        crate::rtl::lorom_offset(address),
    )
}

// Capture count, stride and endpoints in order before touching output. Every
// lane reads then writes one word, so overlaps propagate in traversal order.
// The loop copies individual bytes rather than borrowing live u8 storage: callers may
// still retain legacy enemy pointers outside this fully checked operation.
macro_rules! word_transfers {
    ($game:ident, $direction:tt, $count:expr;
        $($destination:ident($($dst:tt)*) <- $source:ident($($src:tt)*)),+ $(,)?) => {{
        match (
            $count,
            word_transfers!(@step $direction),
            &[$((word_transfers!(@endpoint $game, $destination($($dst)*)),
                 word_transfers!(@endpoint $game, $source($($src)*)))),+],
        ) { (count, step, lanes) => {
        let (count, step, lanes): (usize, isize, &[_]) = (count, step, lanes);
        #[forbid(unsafe_code)]
        {
            use $crate::word_transfers::{address, WordEndpoint};
            for index in 0..count {
                let offset = (index as isize).checked_mul(step).unwrap();
                for (destination, source) in lanes {
                    let value = match source {
                        WordEndpoint::Ram(base, native) => {
                            let at = address(*base, offset);
                            let bytes = [$game.ram().read_byte(at), $game.ram().read_byte(at + 1)];
                            if *native { u16::from_ne_bytes(bytes) } else { u16::from_le_bytes(bytes) }
                        }
                        source => source.read(offset),
                    };
                    let WordEndpoint::Ram(base, native) = destination else {
                        unreachable!("a transfer destination must be RAM");
                    };
                    let at = address(*base, offset);
                    let bytes = if *native { value.to_ne_bytes() } else { value.to_le_bytes() };
                    $game.ram().write_byte(at, bytes[0]);
                    $game.ram().write_byte(at + 1, bytes[1]);
                }
            }
        }
        } }
    }};
    (@step ascending) => { 2isize };
    (@step descending) => { -2isize };
    (@step $stride:expr) => { $stride };
    (@endpoint $game:ident, le($field:ident, $index:expr)) => {
        $crate::word_transfers::WordEndpoint::Ram(
            ($crate::ram_map::word_arrays::$field as isize)
                .checked_add(($index as isize).checked_mul(2).unwrap()).unwrap(), false,
        )
    };
    (@endpoint $game:ident, native($address:expr)) => {
        $crate::word_transfers::WordEndpoint::Ram($address as isize, true)
    };
    (@endpoint $game:ident, little($address:expr)) => {
        $crate::word_transfers::WordEndpoint::Ram($address as isize, false)
    };
    (@endpoint $game:ident, rom($address:expr, $index:expr)) => {{
        let image = $game.loaded_rom.as_ref().expect("ROM must be loaded").clone();
        let base = $crate::rtl::lorom_offset($address);
        let offset = ($index as isize).checked_mul(2).unwrap();
        $crate::word_transfers::WordEndpoint::Rom(image, base.checked_add(offset).unwrap())
    }};
    (@endpoint $game:ident, table($table:ident, $index:expr)) => {
        $crate::word_transfers::WordEndpoint::Table(&$table, 0).shifted($index as isize)
    };
    (@endpoint $game:ident, value($value:expr)) => {
        $crate::word_transfers::WordEndpoint::Value($value)
    };
    (@endpoint $game:ident, captured($source:ident, $index:expr)) => {
        $source.shifted($index as isize)
    };
    (@endpoint $game:ident, pause_map($index:expr)) => {
        word_transfers!(@endpoint $game, native(0x3000 + 2 * ($index)))
    };
    (@endpoint $game:ident, cinematic_bg($index:expr)) => {
        word_transfers!(@endpoint $game, native(0x3800 + 2 * ($index)))
    };
    (@endpoint $game:ident, xray($index:expr)) => {
        word_transfers!(@endpoint $game, native(0x4000 + 2 * ($index)))
    };
}
