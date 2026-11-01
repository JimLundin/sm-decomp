//! Ordered word transfers over captured RAM offsets, ROM images and tables.
use sm_data::cartridge::RomImage;
pub use sm_data::cartridge::lorom_offset;

/// The loaded cartridge image, for `word_transfers!`'s `rom` endpoint.
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

    // RAM is handled at the macro call site, without borrowing the RAM array.
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

pub fn address(base: isize, displacement: isize) -> usize {
    base.checked_add(displacement).unwrap().try_into().unwrap()
}

// Capture count, stride and endpoints in order before touching output. Every
// lane reads then writes one word, so overlaps propagate in traversal order.
// The loop copies individual bytes rather than borrowing live u8 storage: callers may
// still retain legacy enemy pointers outside this fully checked operation.
#[macro_export]
macro_rules! word_transfers {
    ($game:ident, $direction:tt, $count:expr;
        $($destination:ident($($dst:tt)*) <- $source:ident($($src:tt)*)),+ $(,)?) => {{
        match (
            $count,
            $crate::word_transfers!(@step $direction),
            &[$(($crate::word_transfers!(@endpoint $game, $destination($($dst)*)),
                 $crate::word_transfers!(@endpoint $game, $source($($src)*)))),+],
        ) { (count, step, lanes) => {
        let (count, step, lanes): (usize, isize, &[_]) = (count, step, lanes);
        {
            use $crate::word_transfers::{address, WordEndpoint};
            for index in 0..count {
                let offset = (index as isize).checked_mul(step).unwrap();
                for (destination, source) in lanes {
                    let value = match source {
                        WordEndpoint::Ram(base, native) => {
                            let at = address(*base, offset);
                            let bytes = [$game.compatibility_byte(at), $game.compatibility_byte(at + 1)];
                            if *native { u16::from_ne_bytes(bytes) } else { u16::from_le_bytes(bytes) }
                        }
                        source => source.read(offset),
                    };
                    let WordEndpoint::Ram(base, native) = destination else {
                        unreachable!("a transfer destination must be RAM");
                    };
                    let at = address(*base, offset);
                    let bytes = if *native { value.to_ne_bytes() } else { value.to_le_bytes() };
                    $game.compatibility_write_byte(at, bytes[0]);
                    $game.compatibility_write_byte(at + 1, bytes[1]);
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
            ($crate::ram::word_arrays::$field as isize)
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
        let image = $crate::word_transfers::LoadedRom::loaded_rom_image(&*$game);
        let base = $crate::word_transfers::lorom_offset($address);
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
        $crate::word_transfers!(@endpoint $game, native(0x3000 + 2 * ($index)))
    };
    (@endpoint $game:ident, cinematic_bg($index:expr)) => {
        $crate::word_transfers!(@endpoint $game, native(0x3800 + 2 * ($index)))
    };
    (@endpoint $game:ident, xray($index:expr)) => {
        $crate::word_transfers!(@endpoint $game, native(0x4000 + 2 * ($index)))
    };
}

#[cfg(test)]
mod tests {
    use crate::ram::{CompatibilityRam, word};

    /// The named-RAM macros over a plain byte array instead of the compat Game.
    struct Ram(Vec<u8>);

    impl crate::display_scroll::Access for Ram {
        fn scroll_word(&self, word: crate::display_scroll::Word) -> u16 {
            let at = 0xb1 + 2 * usize::from(word);
            u16::from_le_bytes([self.0[at], self.0[at + 1]])
        }
        fn set_scroll_word(&mut self, word: crate::display_scroll::Word, value: u16) {
            let at = 0xb1 + 2 * usize::from(word);
            self.0[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        fn physical_scroll_byte(&self, word: crate::display_scroll::Word, high: bool) -> u8 {
            self.0[0xb1 + 2 * usize::from(word) + usize::from(high)]
        }
        fn write_physical_scroll_byte(
            &mut self,
            word: crate::display_scroll::Word,
            high: bool,
            value: u8,
        ) {
            self.0[0xb1 + 2 * usize::from(word) + usize::from(high)] = value;
        }
    }

    impl crate::sequence_state::Access for Ram {
        fn set_cinematic_function(&mut self, value: u16) {
            for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.compatibility_write_byte(0x1f51 + part, byte);
            }
        }
        fn set_sequence_word(&mut self, word: crate::session_words::Word, value: u16) {
            for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.compatibility_write_byte((word as usize) + part, byte);
            }
        }
        fn sequence_flow(&self) -> crate::flow::Flow {
            panic!("sequence controls were not supplied")
        }
        fn sequence_words(&self) -> crate::session_words::SessionWords {
            panic!("sequence controls were not supplied")
        }
        fn commit_flow(&mut self, before: &crate::flow::Flow, after: crate::flow::Flow) {
            let _ = (before, after);
            panic!("sequence controls were not supplied");
        }
        fn commit_sequence_words(
            &mut self,
            before: &crate::session_words::SessionWords,
            after: crate::session_words::SessionWords,
        ) {
            let _ = (before, after);
            panic!("sequence controls were not supplied");
        }
    }
    impl CompatibilityRam for Ram {
        fn compatibility_byte(&self, at: usize) -> u8 {
            self.0[at]
        }
        fn compatibility_write_byte(&mut self, at: usize, value: u8) {
            self.0[at] = value;
        }
    }

    #[test]
    fn named_words_and_transfers_work_over_any_host() {
        let mut ram = Ram(vec![0; 0x20000]);
        let game = &mut ram;
        word!(game.SAMUS_HEALTH = 0x1234);
        assert_eq!(word!(game.SAMUS_HEALTH), 0x1234);
        assert_eq!(game.0[crate::ram::words::SAMUS_HEALTH], 0x34);
        word!(game.SAMUS_HEALTH += 1);
        assert_eq!(word!(game.SAMUS_HEALTH), 0x1235);
        crate::word_transfers!(game, ascending, 2;
            native(0x100) <- value(0xabcd),
            little(0x200) <- native(0x100),
        );
        assert_eq!(
            &game.0[0x100..0x104],
            &0xabcdu16.to_ne_bytes().repeat(2)[..]
        );
        assert_eq!(&game.0[0x200..0x202], &0xabcdu16.to_le_bytes());
    }
}
