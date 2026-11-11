//! Model. Docs may say std::fs, unsafe or Command: comments are not code.
use clean_types::Speed;
use std::fmt::Write as _;
use std::io::Write;
use std::collections::{BTreeMap, HashMap};
mod shapes;

/* a block comment with std::net::TcpStream and include_bytes!("x") */
pub fn label(s: &Speed) -> String {
    let mut out = String::new();
    let _ = write!(out, "std::fs::read unsafe Command {}", s.0);
    let _ = '"';
    let _: Option<BTreeMap<u8, HashMap<u8, u8>>> = None;
    out
}

/// A tile's size in pixels: a constant's value may be any number.
pub const TILE: u16 = 16;
const ROWS: [u16; 4] = [3, 5, 7, 9];
static LIMIT: u32 = 0x7FFF;
const MASK: u16 = 0b1100;
/// A typed table is data: literals, paths, references, tuples, arrays, struct literals and calls
/// whose every argument is data.
const STEPS: &[(Speed, Kind)] = &[(Speed(-3), Kind::A), (Speed(5), Kind::B)];
const DIGITS: [Speed; 3] = [Speed::new(0x009), Speed::new(0x000), Speed::new(0x001)];
const START: Grid = Grid::filled(Speed::new(7), [TILE; 3], ::clean_types::Speed(-2));
static BASE: Pad = Pad { a: true, b, ..Pad::new(3) };
/// A string that runs over lines is one literal.
const NOTED: (&str, u16) = ("a note that runs \
    over two lines", 7);
/// A range whose ends are literals or constants is data.
const WINDOWS: [Range<u16>; 3] = [0x280..0x500, 2..=TILE, -3..Grid::MAX];

/// An exhaustive lookup is data: a guard-free match from variants, constants or literals to
/// data values, and enum_map! rows. Its values may be any number; its keys name theirs.
pub const fn steps(kind: Kind) -> u16 {
    match kind {
        Kind::A => 3,
        Kind::B => 0x7FFF,
    }
}
pub const fn digit(n: u8) -> Speed {
    match n {
        0 | 1 => Speed::new(0x009),
        _ => Speed(-2),
    }
}
/// A key may nest variants in a tuple variant's fields, so long as it binds nothing.
pub const fn screen(at: Option<Kind>) -> u16 {
    match at {
        Some(Kind::A) => 7,
        Some(Kind::B) | None => 0x30,
    }
}
pub fn tiles() -> EnumMap<Kind, u16> {
    enum_map! { Kind::A => 16, Kind::B => TILE }
}

/// An enum's discriminants are declarations.
pub enum Kind {
    /// Two.
    A = 2,
    /// Three.
    B = 3,
}

/// Array lengths in types are declarations.
pub struct Grid {
    cells: [u8; 16],
}

/// A lint denied only through a group may be expected, with its reason.
#[expect(clippy::struct_excessive_bools, reason = "a pad's buttons are independent")]
pub struct Pad {
    a: bool,
    b: bool,
    c: bool,
    d: bool,
}

/// Zero and one in any spelling, array lengths, tuple fields and attributes are allowed.
#[must_use]
#[inline(always)]
pub fn tile(x: u16, t: (u8, u8, u8), g: [u16; 3]) -> u16 {
    let _ = [0_u8; 16];
    let _ = (t.2, 0x0, 0b1, 1u8, 0_u16, 1.0, 0..=1, g);
    x & TILE.wrapping_sub(1) & MASK & ROWS[0]
}

#[cfg(test)]
mod tests {
    #[test]
    #[expect(clippy::arithmetic_side_effects, reason = "test arithmetic")]
    fn sizes() {
        assert_eq!(super::TILE + 4, 20);
    }
}
