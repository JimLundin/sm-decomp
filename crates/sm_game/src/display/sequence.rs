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
/// BGnHOFS/BGnVOFS and the owned Session holds as `controls.scroll_words`,
/// read and stored through the host's projected bytes.
pub mod scroll {
    use crate::ram::CompatibilityRam;
    /// A scroll mirror: BG1..BG4, horizontal then vertical, in RAM order.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        Bg1H,
        Bg1V,
        Bg2H,
        Bg2V,
        Bg3H,
        Bg3V,
        Bg4H,
        Bg4V,
    }
    const _: () = assert!(crate::ram::words::REG_BG1HOFS == 0xb1);
    const _: () = assert!(crate::ram::words::REG_BG4VOFS == 0xb1 + 2 * Word::Bg4V as usize);
    /// `word` as the host's projection holds it, low byte then high.
    pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
        let at = 0xb1 + 2 * word as usize;
        u16::from_le_bytes([h.compatibility_byte(at), h.compatibility_byte(at + 1)])
    }
    /// Store `value` to `word`, low byte then high, as the named word store did.
    pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
        let [low, high] = value.to_le_bytes();
        h.compatibility_write_byte(0xb1 + 2 * word as usize, low);
        h.compatibility_write_byte(0xb2 + 2 * word as usize, high);
    }
}
