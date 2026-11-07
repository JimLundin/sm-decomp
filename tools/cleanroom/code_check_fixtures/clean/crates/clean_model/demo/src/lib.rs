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
/// A table row of constructors and operators on constants is data.
const MASK: u16 = TILE << 4 | 0b1100;

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
