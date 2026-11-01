//! Names every menu module uses: the named-RAM macros, the shared constants,
//! the register addresses and the host traits.
pub use super::host::{Host, MenuRuntime as _, SaveRam as _};
pub use crate::cartridge::{LoadedImage as _, NativeImage as _};
pub use crate::constants::buttons::*;
pub use crate::constants::game_states::*;
pub use crate::display::registers::*;
pub use crate::effects::{DisplayRefresh as _, SoundEffects as _};
pub use crate::ram::{CompatibilityRam as _, RawRam as _, WorkRamAccess as _, word};

pub use crate::display::hdma_spawn::HdmaObjectDefinition;

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

/// Typed access to Session-owned words through a view's projected-byte `get`
/// and `set`, by its `Word` variant, in the shapes the named word accesses
/// used: read, store, a store of one value to several words, `+=`/`-=`/`&=`
/// (reading the old value first), and those with a reread.
macro_rules! owned {
    ([$($v:tt)*] $game:ident.$word:ident) => { $($v)*::get(&*$game, $($v)*::Word::$word) };
    ([$($v:tt)*] $game:ident.$word:ident = $value:expr $(,)?) => {{
        let value: u16 = $value;
        $($v)*::set($game, $($v)*::Word::$word, value)
    }};
    ([$($v:tt)*] $game:ident.$word:ident $op:tt $value:expr, reread $(,)?) => {{
        owned!([$($v)*] $game.$word $op $value);
        $($v)*::get(&*$game, $($v)*::Word::$word)
    }};
    ([$($v:tt)*] $game:ident.$word:ident $op:tt $value:expr $(,)?) => {{
        let old = $($v)*::get(&*$game, $($v)*::Word::$word);
        let value: u16 = owned!(@op old $op $value);
        $($v)*::set($game, $($v)*::Word::$word, value)
    }};
    (@op $old:ident += $value:expr) => { $old.wrapping_add($value) };
    (@op $old:ident -= $value:expr) => { $old.wrapping_sub($value) };
    (@op $old:ident &= $value:expr) => { $old & $value };
    (@op $old:ident |= $value:expr) => { $old | $value };
    (@op $old:ident ^= $value:expr) => { $old ^ $value };
    ($view:tt $game:ident, $value:expr; $($word:ident),+ $(,)?) => {
        $(owned!($view $game.$word = $value);)+
    };
    ($view:tt $game:ident; $($word:ident = $value:expr;)+) => {
        $(owned!($view $game.$word = $value);)+
    };
}
pub(crate) use owned;
/// Samus's inventory words ([`crate::inventory::Word`]).
macro_rules! inventory { ($($t:tt)*) => { owned!([$crate::inventory] $($t)*) }; }
pub(crate) use inventory;
/// The BG scroll mirrors ([`crate::display::sequence::scroll::Word`]).
macro_rules! scroll { ($($t:tt)*) => { owned!([$crate::display::sequence::scroll] $($t)*) }; }
pub(crate) use scroll;
/// Crocomire's melt words ([`crate::actors::crocomire_reactions::melt::Word`]).
macro_rules! melt { ($($t:tt)*) => { owned!([$crate::actors::crocomire_reactions::melt] $($t)*) }; }
pub(crate) use melt;
/// The bosses' shared RAM words ([`crate::actors::boss_lease::boss_ram::Word`]).
macro_rules! boss { ($($t:tt)*) => { owned!([$crate::actors::boss_lease::boss_ram] $($t)*) }; }
pub(crate) use boss;

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
// Field names follow the original's RAM map.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct EnemyData {
    pub base: crate::actors::layouts::EnemyBase,
    pub ai_var_a: u16,
    pub ai_var_b: u16,
    pub ai_var_c: u16,
    pub ai_var_d: u16,
    pub ai_var_e: u16,
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
