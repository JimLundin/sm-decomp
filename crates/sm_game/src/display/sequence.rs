//! Ordered register writes: the trait a host performs them through and the
//! `register_writes!` sequence over it.
use crate::ram::RawRam as _;

/// Register writes in the order the caller makes them.
pub trait Registers {
    fn write_register(&mut self, register: u16, value: u8);
    fn write_register_word(&mut self, register: u16, value: u16);
    fn read_register_word(&mut self, register: u16) -> u16;
    fn read_register(&mut self, register: u16) -> u8;
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
    // Address then value are evaluated before the write, as in the original call.
    (@write $game:ident; byte($address:expr, $value:expr $(,)?)) => {{
        let (address, value) = ($address, $value);
        $crate::display::sequence::Registers::write_register(&mut *$game, address, value)
    }};
    (@write $game:ident; word($address:expr, $value:expr $(,)?)) => {{
        let (address, value) = ($address, $value);
        $crate::display::sequence::Registers::write_register_word(&mut *$game, address, value)
    }};
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

/// The BG scroll register mirrors $00B1..$00C0, which the NMI writes to
/// BGnHOFS/BGnVOFS and the owned Session holds as `controls.scroll_words`
/// while its presentation is Owned.
pub mod scroll {
    /// The BG1..BG4 scroll mirrors, each horizontal then vertical.
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub struct BgScroll {
        pub bg1: [u16; 2],
        pub bg2: [u16; 2],
        pub bg3: [u16; 2],
        pub bg4: [u16; 2],
    }

    /// The mirrors are Session fields while its presentation is Owned, and
    /// RAM otherwise.
    impl crate::session_cohort::Cohort for BgScroll {
        fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
            let layers = [&mut self.bg1, &mut self.bg2, &mut self.bg3, &mut self.bg4];
            for (layer, words) in layers.into_iter().enumerate() {
                for (axis, word) in words.iter_mut().enumerate() {
                    visit(FIRST + 4 * layer + 2 * axis, word);
                }
            }
        }
        fn session_words(
            session: &mut crate::session::Session,
            mut visit: impl FnMut(usize, &mut u16),
        ) {
            if let Ok(owner) = session.presentation.owner_mut() {
                for (index, word) in owner.state.controls.scroll_words.iter_mut().enumerate() {
                    visit(FIRST + 2 * index, word);
                }
            }
        }
    }

    /// The first mirror, BG1's horizontal scroll.
    const FIRST: usize = 0xb1;
    const _: () = assert!(crate::ram::words::REG_BG1HOFS == FIRST);
    const _: () = assert!(crate::ram::words::REG_BG4VOFS == FIRST + 14);
}
