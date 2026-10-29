//! Names for work RAM ($7E:0000-$7F:FFFF) offsets.
//!
//! Extracted from snesrev/sm `variables.h` and `variables_extra.h` (rev 578f90b).
//! c2rust expanded those macros, so the transpiled code reads e.g.
//! `*(g_ram.offset(0x998) as *mut uint16)` where the C said `game_state`.
//! The verifier uses this table to name RAM mismatches. Use these names when
//! replacing raw gameplay accesses (phase 3 of docs/PLAN.md).

pub use sm_game::ram::{RAM_VARS, RamVar, Shape};
pub(crate) use sm_game::ram::{checked_word, word, word_arrays, words};

// Checked enemy-record fields at RAM 0x0f78 + signed slot, replacing raw
// `Get_X(slot) -> *mut` records (`slot!(game, Alias[slot].field)`). Like `word!`, each access copies Cell-backed RAM bytes
// at the caller and forms no live u8 reference; native byte order matches the old
// dereference. Writes evaluate the value before the slot and keep its temporaries
// through the store. Word forms check at compile time that the field is a u16.
macro_rules! slot {
    (@at $layout:ident, $slot:expr, $($field:ident).+) => {
        ::core::primitive::usize::try_from(
            { let slot: ::core::primitive::i32 = $slot; slot as ::core::primitive::isize }
                + 0x0f78
                + ::core::mem::offset_of!($layout, $($field).+) as ::core::primitive::isize,
        )
        .expect("enemy field outside RAM")
    };
    (byte $game:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {
        match ($value, $crate::ram_map::slot!(@at $layout, $slot, $($field).+)) {
            (value, at) => {
                let value: ::core::primitive::u8 = value;
                let at = ::core::primitive::usize::try_from(at as ::core::primitive::isize + ($lane) as ::core::primitive::isize)
                    .expect("enemy field outside RAM");
                $game.ram().write_byte(at, value);
            }
        }
    };
    (byte $game:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        match $crate::ram_map::slot!(@at $layout, $slot, $($field).+) {
            at => {
                let at = ::core::primitive::usize::try_from(
                    at as ::core::primitive::isize + ($lane) as ::core::primitive::isize,
                )
                .expect("enemy field outside RAM");
                $game.ram().read_byte(at)
            },
        }
    };
    // Split-word pairs, as `rtl::AddToHiLo`/`SetHiLo`: the value first, then low
    // is read before high and written before high.
    ($game:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {
        match $value {
            value => {
                let value: ::core::primitive::u32 = value;
                let low = ::core::primitive::u32::from($crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$lo));
                let high = ::core::primitive::u32::from($crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$hi));
                let sum = (low | (high << 16)).wrapping_add(value);
                $crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$lo = sum as ::core::primitive::u16);
                $crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as ::core::primitive::u16);
            }
        }
    };
    ($game:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) = $value:expr) => {
        match $value {
            value => {
                let value: ::core::primitive::u32 = value;
                $crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$lo = value as ::core::primitive::u16);
                $crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$hi = (value >> 16) as ::core::primitive::u16);
            }
        }
    };
    // Ordered word stores to several fields; the slot is re-evaluated per store.
    ($game:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($stores:tt)+) => {
        $crate::ram_map::slot!(@stores $game, $layout[$slot] ($($prefix)*); $($stores)+)
    };
    (@stores $game:ident, $layout:ident[$slot:expr] $prefix:tt; $($field:ident = $value:expr),+ $(,)?) => {
        $($crate::ram_map::slot!(@store $game, $layout[$slot] $prefix $field = $value);)+
    };
    (@store $game:ident, $layout:ident[$slot:expr] ($($prefix:ident)*) $field:ident = $value:expr) => {
        $crate::ram_map::slot!($game, $layout[$slot] $(.$prefix)* .$field = $value)
    };
    // Read-modify-write: the old word is read first, as the leading operand of
    // the replaced `field = field OP ..` value was; the slot is evaluated once.
    ($game:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {
        match $crate::ram_map::slot!(@at $layout, $slot, $($field).+) {
            at => {
                $crate::ram_map::word_field::<$layout>(|record| record $(.$field)+);
                let $old = ::core::primitive::u16::from_ne_bytes([$game.ram().read_byte(at), $game.ram().read_byte(at + 1)]);
                let bytes = ::core::primitive::u16::to_ne_bytes($value);
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            }
        }
    };
    ($game:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {
        match ($value, $crate::ram_map::slot!(@at $layout, $slot, $($field).+)) {
            (value, at) => {
                $crate::ram_map::word_field::<$layout>(|record| record $(.$field)+);
                let bytes = ::core::primitive::u16::to_ne_bytes(value);
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            }
        }
    };
    ($game:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        match $crate::ram_map::slot!(@at $layout, $slot, $($field).+) {
            at => {
                $crate::ram_map::word_field::<$layout>(|record| record $(.$field)+);
                ::core::primitive::u16::from_ne_bytes([$game.ram().read_byte(at), $game.ram().read_byte(at + 1)])
            }
        }
    };
}
pub(crate) use slot;
/// Compile-time proof that an enemy-record field is one u16 word.
pub(crate) fn word_field<L>(_: fn(L) -> u16) {}
