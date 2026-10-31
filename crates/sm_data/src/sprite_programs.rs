//! Bank $B4's sprite object programs: the definition table and the
//! instruction lists it points at.
//!
//! An instruction is a timer word and a spritemap pointer; a timer with bit 15
//! set is instead the address of a command's handler.

/// The bank whose upper half holds the programs.
pub const BANK: u8 = 0xb4;
/// $B4:BDA8: each sprite object definition's instruction list.
pub const DEFINITIONS: u16 = 0xbda8;

/// A program command, decoded from its handler's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// $BCF0: hold on the previous instruction for good.
    Hold,
    /// $BD07: delete the sprite object.
    Delete,
    /// $BD12: go to the instruction list in the next word.
    Goto,
}

impl Command {
    /// The command whose handler is at $B4:`handler`, if any.
    pub fn decode(handler: u16) -> Option<Self> {
        Some(match handler {
            0xbcf0 => Self::Hold,
            0xbd07 => Self::Delete,
            0xbd12 => Self::Goto,
            _ => return None,
        })
    }
}
