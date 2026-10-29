//! Ordered register writes: the trait a host performs them through and the
//! `register_writes!` sequence over it.
use crate::ram::RawRam as _;

/// Register writes in the order the caller makes them.
pub trait Registers {
    fn write_register(&mut self, register: u16, value: u8);
    fn write_register_word(&mut self, register: u16, value: u16);
}

/// Write `value` to `register` and its RAM mirror at `mirror`, mirror first
/// when `mirror_first`.
pub fn write_register_and_mirror(
    host: &mut (impl Registers + crate::ram::WorkRamAccess + ?Sized),
    register: u16,
    value: u8,
    mirror: usize,
    mirror_first: bool,
) {
    if mirror_first {
        host.ram().write_byte(mirror, value);
        host.write_register(register, value);
    } else {
        host.write_register(register, value);
        host.ram().write_byte(mirror, value);
    }
}

/// An ordered register sequence: `byte`, `word`, `mirror`, and the typed
/// `show`/`show_mirror` display values, as in the compat `register_writes!`.
#[macro_export]
macro_rules! register_sequence {
    ($game:ident; $($kind:ident($($argument:tt)*);)*) => {
        $($crate::register_sequence!(@write $game; $kind($($argument)*));)*
    };
    (@write $game:ident; byte($address:expr, $value:expr $(,)?)) => {
        $crate::display::sequence::Registers::write_register(&mut *$game, $address, $value)
    };
    (@write $game:ident; word($address:expr, $value:expr $(,)?)) => {
        $crate::display::sequence::Registers::write_register_word(&mut *$game, $address, $value)
    };
    (@write $game:ident; mirror($address:expr, $value:expr, $offset:expr, $before:expr $(,)?)) => {
        $crate::display::sequence::write_register_and_mirror(&mut *$game, $address, $value, $offset, $before)
    };
    (@write $game:ident; show($display:expr $(,)?)) => {{
        let display = $display;
        $crate::display::sequence::Registers::write_register(
            &mut *$game,
            $crate::display::control::DisplayRegister::register(&display),
            $crate::display::control::DisplayRegister::value(&display),
        )
    }};
    (@write $game:ident; show_mirror($display:expr, $offset:expr, $before:expr $(,)?)) => {{
        let display = $display;
        $crate::display::sequence::write_register_and_mirror(
            &mut *$game,
            $crate::display::control::DisplayRegister::register(&display),
            $crate::display::control::DisplayRegister::value(&display),
            $offset,
            $before,
        )
    }};
}
pub use crate::register_sequence as register_writes;
