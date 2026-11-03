//! Ordered register writes: the trait a host performs them through and the
//! `register_writes!` sequence over it.
use crate::display::preparation::{Access, Field, Store};

/// Register writes in the order the caller makes them.
pub trait Registers {
    fn write_register(&mut self, register: u16, value: u8);
    fn write_register_word(&mut self, register: u16, value: u16);
    fn read_register_word(&mut self, register: u16) -> u16;
    fn read_register(&mut self, register: u16) -> u8;
}

/// Write `value` to `register` and store it in the display's `mirror` field
/// (a named store, as the original's RAM store), mirror first when
/// `mirror_first`.
pub fn write_register_and_mirror(
    host: &mut (impl Registers + Access + ?Sized),
    register: u16,
    value: u8,
    mirror: Field,
    mirror_first: bool,
) {
    if mirror_first {
        host.store_display_control_byte(mirror, value, Store::Named);
        host.write_register(register, value);
    } else {
        host.write_register(register, value);
        host.store_display_control_byte(mirror, value, Store::Named);
    }
}

/// An ordered register sequence: `byte`, `word`, `mirror`, and the typed
/// `show`/`show_mirror` display values, as in the compat `register_writes!`.
#[macro_export]
macro_rules! register_sequence {
    ($game:ident; $($kind:ident($($argument:tt)*);)*) => {
        $($crate::register_sequence!(@write $game; $kind($($argument)*));)*
    };
    // Address then value are evaluated before the write, as in the original call.
    (@write $game:ident; byte($address:expr, $value:expr $(,)?)) => {{
        let (address, value) = ($address, $value);
        $crate::display::sequence::Registers::write_register(&mut *$game, address, value)
    }};
    (@write $game:ident; word($address:expr, $value:expr $(,)?)) => {{
        let (address, value) = ($address, $value);
        $crate::display::sequence::Registers::write_register_word(&mut *$game, address, value)
    }};
    (@write $game:ident; mirror($address:expr, $value:expr, $mirror:expr, $before:expr $(,)?)) => {
        $crate::display::sequence::write_register_and_mirror(&mut *$game, $address, $value, $mirror, $before)
    };
    (@write $game:ident; show($display:expr $(,)?)) => {{
        let display = $display;
        $crate::display::sequence::Registers::write_register(
            &mut *$game,
            $crate::display::control::DisplayRegister::register(&display),
            $crate::display::control::DisplayRegister::value(&display),
        )
    }};
    (@write $game:ident; show_mirror($display:expr, $mirror:expr, $before:expr $(,)?)) => {{
        let display = $display;
        $crate::display::sequence::write_register_and_mirror(
            &mut *$game,
            $crate::display::control::DisplayRegister::register(&display),
            $crate::display::control::DisplayRegister::value(&display),
            $mirror,
            $before,
        )
    }};
}
pub use crate::register_sequence as register_writes;

pub use crate::display_scroll as scroll;
