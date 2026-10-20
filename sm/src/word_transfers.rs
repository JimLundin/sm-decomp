//! Ordered native-word and little-endian RAM transfers.
//!
//! Endpoints are raw pointers: this seam never borrows Game or its RAM. Each
//! iteration reads one source and then writes its destination before moving to
//! the next lane. Overlapping copies therefore retain directional propagation.
//! Callers must preserve bounds, traversal, scalar snapshots and live rereads.

#[derive(Clone, Copy)]
pub(crate) enum WordEndpoint {
    Native(*mut u16),
    NativeUnaligned(*mut u16),
    LittleEndian(*mut u16),
    Value(u16),
}

impl WordEndpoint {
    unsafe fn read(self, byte_offset: isize) -> u16 {
        match self {
            Self::Native(pointer) => pointer.byte_offset(byte_offset).read(),
            Self::NativeUnaligned(pointer) => pointer.byte_offset(byte_offset).read_unaligned(),
            Self::LittleEndian(pointer) => {
                u16::from_le(pointer.byte_offset(byte_offset).read_unaligned())
            }
            Self::Value(value) => value,
        }
    }

    unsafe fn write(self, byte_offset: isize, value: u16) {
        match self {
            Self::Native(pointer) => pointer.byte_offset(byte_offset).write(value),
            Self::NativeUnaligned(pointer) => {
                pointer.byte_offset(byte_offset).write_unaligned(value);
            }
            Self::LittleEndian(pointer) => {
                pointer
                    .byte_offset(byte_offset)
                    .write_unaligned(value.to_le());
            }
            Self::Value(_) => unreachable!("a transfer destination must be memory"),
        }
    }
}

/// `step` is a signed byte stride from the explicitly supplied first word.
/// Lanes retain their order, including duplicate or overlapping destinations.
pub(crate) unsafe fn transfer_words(
    count: usize,
    step: isize,
    lanes: &[(WordEndpoint, WordEndpoint)],
) {
    for index in 0..count {
        let offset = index as isize * step;
        for &(destination, source) in lanes {
            let value = source.read(offset);
            destination.write(offset, value);
        }
    }
}

// These forms name actual memory endpoints, not arbitrary loop bodies.
// `value` takes an invariant scalar; memory values must remain source endpoints.
macro_rules! word_transfers {
    ($game:ident, $direction:tt, $count:expr;
        $($destination:ident($($dst:tt)*) <- $source:ident($($src:tt)*)),+ $(,)?) => {
        $crate::word_transfers::transfer_words(
            $count,
            word_transfers!(@step $direction),
            &[$((word_transfers!(@endpoint $game, $destination($($dst)*)),
                 word_transfers!(@endpoint $game, $source($($src)*)))),+],
        )
    };
    (@step ascending) => { 2 };
    (@step descending) => { -2 };
    (@step $stride:expr) => { $stride };
    (@endpoint $game:ident, le($field:ident, $index:expr)) => {
        $crate::word_transfers::WordEndpoint::LittleEndian(
            (&raw mut $game.g_ram).cast::<u8>()
                .add($crate::ram_map::word_arrays::$field)
                .cast::<u16>().offset($index as isize),
        )
    };
    (@endpoint $game:ident, native($address:expr)) => {
        $crate::word_transfers::WordEndpoint::Native(
            (&raw mut $game.g_ram).cast::<u8>().offset($address as isize).cast::<u16>(),
        )
    };
    (@endpoint $game:ident, little($address:expr)) => {
        $crate::word_transfers::WordEndpoint::LittleEndian(
            (&raw mut $game.g_ram).cast::<u8>().offset($address as isize).cast::<u16>(),
        )
    };
    (@endpoint $game:ident, rom($address:expr, $index:expr)) => {
        $crate::word_transfers::WordEndpoint::NativeUnaligned(
            $game.g_rom.offset($crate::rtl::lorom_offset($address))
                .cast::<u16>().offset($index as isize).cast_mut(),
        )
    };
    (@endpoint $game:ident, rom_native($address:expr, $index:expr)) => {
        $crate::word_transfers::WordEndpoint::Native(
            $game.g_rom.offset($crate::rtl::lorom_offset($address))
                .cast::<u16>().offset($index as isize).cast_mut(),
        )
    };
    (@endpoint $game:ident, table($table:ident, $index:expr)) => {
        $crate::word_transfers::WordEndpoint::Native(
            ($table.as_ptr() as *mut u16).offset($index as isize),
        )
    };
    (@endpoint $game:ident, value($value:expr)) => {
        $crate::word_transfers::WordEndpoint::Value($value)
    };
    (@endpoint $game:ident, native_ptr($pointer:expr)) => {
        $crate::word_transfers::WordEndpoint::Native($pointer as *mut u16)
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

pub(crate) use word_transfers;
