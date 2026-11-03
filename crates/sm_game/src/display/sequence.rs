//! Ordered register writes: the trait a host performs them through and the
//! `register_writes!` sequence over it. Producers that set the display name a
//! typed `display::video::Command` instead; what is left here serves the NMI
//! refresh adapter and the HDMA objects.

/// Register writes in the order the caller makes them.
pub trait Registers {
    fn write_register(&mut self, register: u16, value: u8);
    fn write_register_word(&mut self, register: u16, value: u16);
    fn read_register_word(&mut self, register: u16) -> u16;
    fn read_register(&mut self, register: u16) -> u8;
}

/// An ordered register sequence of `byte` and `word` writes, as in the compat
/// `register_writes!`.
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
}
pub use crate::register_sequence as register_writes;

pub use crate::display_scroll as scroll;
