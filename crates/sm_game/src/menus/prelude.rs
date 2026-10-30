//! Names every menu module uses: the named-RAM macros, the shared constants,
//! the register addresses and the host traits.
pub use super::host::{Host, MenuRuntime as _, MenuSprites as _, SaveRam as _};
pub use crate::cartridge::{LoadedImage as _, NativeImage as _};
pub use crate::constants::buttons::*;
pub use crate::constants::game_states::*;
pub use crate::display::registers::*;
pub use crate::effects::{DisplayRefresh as _, SoundEffects as _};
pub use crate::ram::{CompatibilityRam as _, RawRam as _, WorkRamAccess as _, word};

/// The original coroutine result: nonzero yields to the frame loop.
pub type CoroutineRet = u8;

/// Byte offsets of enemy record fields, from the record at $7E:0F78.
pub mod enemy_data {
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
