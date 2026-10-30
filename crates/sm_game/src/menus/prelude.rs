//! Names every menu module uses: the named-RAM macros, the shared constants,
//! the register addresses and the host traits.
pub use super::host::{Host, MenuRuntime as _, SaveRam as _};
pub use crate::cartridge::{LoadedImage as _, NativeImage as _};
pub use crate::constants::buttons::*;
pub use crate::constants::game_states::*;
pub use crate::display::registers::*;
pub use crate::effects::{DisplayRefresh as _, SoundEffects as _};
pub use crate::ram::{CompatibilityRam as _, RawRam as _, WorkRamAccess as _, word};

pub use crate::display::hdma_spawn::SpawnHdmaObject_Args;

/// A bank-local address, as the original's `VoidP`.
pub type VoidP = u16;

/// Join two words as `(hi << 16) | lo`, optionally as `i32`.
macro_rules! words {
    (i32, $game:ident.$hi:ident, $lo:ident $(,)?) => {
        (($crate::ram::word!($game.$hi) as i32) << 16i32) | ($crate::ram::word!($game.$lo) as i32)
    };
    ($game:ident.$hi:ident, $lo:ident $(,)?) => {
        (($crate::ram::word!($game.$hi) as u32) << 16i32) | ($crate::ram::word!($game.$lo) as u32)
    };
    (i32, ($hi:expr, $lo:expr $(,)?) $(,)?) => {
        (($hi as i32) << 16i32) | ($lo as i32)
    };
    (($hi:expr, $lo:expr $(,)?) $(,)?) => {
        (($hi as u32) << 16i32) | ($lo as u32)
    };
}
pub(crate) use words;

/// Typed access to Samus's inventory words through the host's projected
/// bytes, by [`crate::inventory::Word`] variant, in the shapes the named word
/// accesses used: read, store, a store of one value to several words,
/// `+=`/`-=`/`&=` (reading the old value first), and those with a reread of
/// the stored value.
macro_rules! inventory {
    ($game:ident.$word:ident) => {
        $crate::inventory::get(&*$game, $crate::inventory::Word::$word)
    };
    ($game:ident.$word:ident = $value:expr $(,)?) => {{
        let value: u16 = $value;
        $crate::inventory::set($game, $crate::inventory::Word::$word, value)
    }};
    ($game:ident.$word:ident += $value:expr $(,)?) => {
        $crate::menus::prelude::inventory!(@update $game.$word, |old| old.wrapping_add($value))
    };
    ($game:ident.$word:ident -= $value:expr $(,)?) => {
        $crate::menus::prelude::inventory!(@update $game.$word, |old| old.wrapping_sub($value))
    };
    ($game:ident.$word:ident &= $value:expr $(,)?) => {
        $crate::menus::prelude::inventory!(@update $game.$word, |old| old & $value)
    };
    ($game:ident.$word:ident $op:tt $value:expr, reread $(,)?) => {{
        $crate::menus::prelude::inventory!($game.$word $op $value);
        $crate::inventory::get(&*$game, $crate::inventory::Word::$word)
    }};
    ($game:ident, $value:expr; $($word:ident),+ $(,)?) => {
        $($crate::menus::prelude::inventory!($game.$word = $value);)+
    };
    ($game:ident; $($word:ident = $value:expr;)+) => {
        $($crate::menus::prelude::inventory!($game.$word = $value);)+
    };
    (@update $game:ident.$word:ident, |$old:ident| $value:expr) => {{
        let $old = $crate::inventory::get(&*$game, $crate::inventory::Word::$word);
        let value: u16 = $value;
        $crate::inventory::set($game, $crate::inventory::Word::$word, value)
    }};
}
pub(crate) use inventory;

/// A word pair addressed as an enemy-slot record, as the original's `PairU16`.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PairU16 {
    pub k: u16,
    pub j: u16,
}

/// Bytes in one VRAM write queue record: size, 24-bit source, destination.
pub const VRAM_WRITE_ENTRY_BYTES: usize = 7;

/// The full enemy record at $7E:0F78 + slot, as the original's `EnemyData`.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyData {
    pub base: crate::actors::layouts::EnemyBase,
    pub ai_var_A: u16,
    pub ai_var_B: u16,
    pub ai_var_C: u16,
    pub ai_var_D: u16,
    pub ai_var_E: u16,
    pub ai_preinstr: u16,
    pub parameter_1: u16,
    pub parameter_2: u16,
}

/// The original coroutine result: nonzero yields to the frame loop.
pub type CoroutineRet = u8;

/// Byte offsets of enemy record fields, from the record at $7E:0F78.
pub mod enemy_data {
    /// The first enemy record, `enemy_data[0]`.
    pub const FIRST: usize = 0xf78;
    pub const X_POS: usize = 2;
    pub const PALETTE_INDEX: usize = 30;
}

/// Run `body`, then stop once `guard` holds.
macro_rules! do_until {
    ($guard:expr; $($body:tt)*) => {
        loop {
            $($body)*
            if $guard {
                break;
            }
        }
    };
}
pub(crate) use do_until;
