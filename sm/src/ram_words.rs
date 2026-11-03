//! Named work RAM word and enemy-record access for the compatibility layer.
//!
//! `word!`, `checked_word!` and `slot!` read and store RAM by the names in
//! [`crate::ram_map`]. Only the RAM-based compatibility code uses them; the
//! target packages reach the same state through typed Session owners.
#![forbid(unsafe_code)]

// Named SNES word reads, assignments and updates share one caller-side interface.
// `word!(game.field)` reads; `word!(game.field = value)` writes. Arrays use
// `field[index]`. Updates use an expression `(game.field, |old| value)`, with
// an optional `reread`. Ordered stores
// of one literal use `(game, value; field, other[index])`; different values use
// `(game; field = value; other[index] = next;)`.
//
// Checked byte places stay at the caller: borrowing the whole Game/RAM here
// could invalidate retained raw enemy pointers. No reference escapes an access.
// Odd addresses and signed array offsets outside a named table remain valid.
// Reads evaluate the index once; assignments evaluate RHS before the
// destination, retaining RHS temporaries through the store. Updates capture
// the destination and read its old value before evaluating the RHS. `reread`
// loads that destination after RHS temporaries are dropped. No value or index
// is hoisted across ordered stores; update expressions create no closure.
macro_rules! word {
    // Compound arithmetic wraps at 16 bits; `&=` keeps the ordinary u16 operation.
    // Delegate destination capture, old load, RHS evaluation and store to the update arm.
    ($game:ident.$field:ident$([$index:expr])? &= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_words::word!($game.$field$([$index])?, |old| old & $value $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? += $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_words::word!($game.$field$([$index])?, |old| old.wrapping_add($value) $(, $result)?)
    };
    ($game:ident.$field:ident$([$index:expr])? -= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_words::word!($game.$field$([$index])?, |old| old.wrapping_sub($value) $(, $result)?)
    };
    // The promoted subtraction retains its original i32 RHS and final u16 cast,
    // using ordinary Rust overflow rules before truncation.
    (checked_i32, $game:ident.$field:ident$([$index:expr])? -= $value:expr $(, $result:ident)? $(,)?) => {
        $crate::ram_words::word!($game.$field$([$index])?, |old| ((old as i32) - $value) as u16 $(, $result)?)
    };

    ($game:ident.$field:ident[$index:expr]) => {
        $crate::ram_words::word!(@read $game,
            $crate::ram_words::word!(@offset $field[$index]))
    };
    ($game:ident.$field:ident) => {
        $crate::ram_words::word!(@read $game, $crate::ram_map::words::$field)
    };
    ($game:ident.$field:ident[$index:expr] = $value:expr $(,)?) => {
        $crate::ram_words::word!(@write $game,
            $crate::ram_words::word!(@offset $field[$index]), $value)
    };
    ($game:ident.$field:ident = $value:expr $(,)?) => {
        $crate::ram_words::word!(@write $game, $crate::ram_map::words::$field, $value)
    };
    ($game:ident; $($field:ident$([$index:expr])? = $value:expr;)+) => {
        $($crate::ram_words::word!($game.$field$([$index])? = $value);)+
    };
    ($game:ident, $value:literal; $($field:ident$([$index:expr])?),+ $(,)?) => {
        $($crate::ram_words::word!($game.$field$([$index])? = $value);)+
    };
    ($game:ident.$field:ident[$index:expr], |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = $crate::ram_words::word!(@offset $field[$index]);
        let $old = $crate::ram_words::word!(@read $game, destination);
        $crate::ram_words::word!(@write $game, destination, $value);
        $crate::ram_words::word!(@result $game, destination $(, $result)?)
    }};
    ($game:ident.$field:ident, |$old:ident| $value:expr $(, $result:ident)? $(,)?) => {{
        let destination = $crate::ram_map::words::$field;
        let $old = $crate::ram_words::word!(@read $game, destination);
        $crate::ram_words::word!(@write $game, destination, $value);
        $crate::ram_words::word!(@result $game, destination $(, $result)?)
    }};
    (@offset $field:ident[$index:expr]) => {
        $crate::ram_map::word_arrays::$field
            .checked_add_signed(($index as isize).checked_mul(2).expect("word index overflow"))
            .expect("word address outside RAM")
    };
    (@read $game:ident, $offset:expr) => {
        match $offset {
            offset => {
                // Check both bytes without borrowing any part of the RAM array.
                assert!(offset < 131071, "word address outside RAM");
                u16::from_le_bytes([$game.compatibility_byte(offset), $game.compatibility_byte(offset + 1)])
            }
        }
    };
    (@write $game:ident, $offset:expr, $value:expr) => {
        // Match keeps RHS/index temporaries alive through both stores, unlike
        // a let-value statement. The caller's semicolon drops them before reread.
        match ($value, $offset) {
            (value, offset) => {
                assert!(offset < 131071, "word address outside RAM");
                let bytes = u16::to_le_bytes(value);
                $game.compatibility_write_byte(offset, bytes[0]);
                $game.compatibility_write_byte(offset + 1, bytes[1]);
            }
        }
    };
    (@result $game:ident, $destination:ident) => { () };
    (@result $game:ident, $destination:ident, reread) => {
        $crate::ram_words::word!(@read $game, $destination)
    };
}

// Checked word access for migrated callers that retain offsets, not pointers.
// Byte places avoid borrowing the whole RAM array. Assignments evaluate the
// value before the address; updates capture the address and old value first.
// This smaller interface retains its existing low-byte-first failure prefix.
macro_rules! checked_word {
    ($game:ident.$field:ident[$index:expr] $($tail:tt)*) => {
        $crate::ram_words::checked_word!($game,
            $crate::ram_map::word_arrays::$field.checked_add_signed(($index as isize).checked_mul(2).unwrap()).unwrap(); $($tail)*)
    };
    ($game:ident.$field:ident $($tail:tt)*) => {
        $crate::ram_words::checked_word!($game, $crate::ram_map::words::$field; $($tail)*)
    };
    ($game:ident, $offset:expr;) => {{
        let offset: usize = $offset;
        u16::from_le_bytes([$game.compatibility_byte(offset), $game.compatibility_byte(offset + 1)])
    }};
    ($game:ident, $offset:expr; = $value:expr) => {
        match ($value, $offset) {
            (value, offset) => {
                let bytes = u16::to_le_bytes(value);
                $game.compatibility_write_byte(offset, bytes[0]);
                $game.compatibility_write_byte(offset + 1, bytes[1]);
            }
        }
    };
    ($game:ident, $offset:expr; += $value:expr) => {{
        let offset: usize = $offset;
        let old = $crate::ram_words::checked_word!($game, offset;);
        $crate::ram_words::checked_word!($game, offset; = old.wrapping_add($value));
    }};
}

/// Named word access through any `CompatibilityRam`; the receiver must have
/// `compatibility_byte`/`compatibility_write_byte` methods in scope.
pub(crate) use {checked_word, word};

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
    ($game:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {
        match ($value, $crate::ram_words::slot!(@at $layout, $slot, $($field).+)) {
            (value, at) => {
                $crate::ram_words::word_field::<$layout>(|record| record $(.$field)+);
                let bytes = ::core::primitive::u16::to_ne_bytes(value);
                $game.ram().write_byte(at, bytes[0]);
                $game.ram().write_byte(at + 1, bytes[1]);
            }
        }
    };
    ($game:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        match $crate::ram_words::slot!(@at $layout, $slot, $($field).+) {
            at => {
                $crate::ram_words::word_field::<$layout>(|record| record $(.$field)+);
                ::core::primitive::u16::from_ne_bytes([$game.ram().read_byte(at), $game.ram().read_byte(at + 1)])
            }
        }
    };
}
pub(crate) use slot;
/// Compile-time proof that an enemy-record field is one u16 word.
pub fn word_field<L>(_: fn(L) -> u16) {}
