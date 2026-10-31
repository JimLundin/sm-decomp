//! Bank $8D's palette effect programs: definitions, colour runs and the
//! commands between them.
//!
//! A program is a list of phases. Each phase is a duration word, the colours it
//! writes (words with bit 15 clear) and then commands (words with bit 15 set,
//! the address of the command's handler), some followed by an operand.

/// The bank whose upper half holds the programs.
pub const BANK: u8 = 0x8d;

/// A program command, decoded from its handler's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// $C595: end the phase, resuming after this word and the next.
    Done,
    /// $C599-$C5C6: skip `colours` colours of the palette.
    Skip { colours: u16 },
    /// $C5CF: delete the effect.
    Delete,
    /// $C5D4: the next word is the new pre-instruction.
    SetPreInstruction,
    /// $C5DD: clear the pre-instruction ($C5E3, which does nothing).
    ClearPreInstruction,
    /// $C61E: go to the next word.
    Goto,
    /// $C639: decrement the counter and go to the next word unless it ran out.
    DecrementGoto,
    /// $C648: the next byte is the counter.
    SetCounter,
    /// $C655: the next word is the colour index.
    SetColourIndex,
    /// $C65E: queue the music track in the next byte.
    QueueMusic,
    /// $C66A, $C673 and $C67C: queue the sound in the next word on `library`.
    QueueSound { library: u8 },
    /// $F1C6: the next byte is the heat palette step.
    SetHeat,
}

impl Command {
    /// The command whose handler is at $8D:`handler`, if any.
    pub fn decode(handler: u16) -> Option<Self> {
        Some(match handler {
            0xc595 => Self::Done,
            0xc599 => Self::Skip { colours: 2 },
            0xc5a2 => Self::Skip { colours: 3 },
            0xc5ab => Self::Skip { colours: 4 },
            0xc5b4 => Self::Skip { colours: 8 },
            0xc5bd => Self::Skip { colours: 9 },
            0xc5c6 => Self::Skip { colours: 15 },
            0xc5cf => Self::Delete,
            0xc5d4 => Self::SetPreInstruction,
            0xc5dd => Self::ClearPreInstruction,
            0xc61e => Self::Goto,
            0xc639 => Self::DecrementGoto,
            0xc648 => Self::SetCounter,
            0xc655 => Self::SetColourIndex,
            0xc65e => Self::QueueMusic,
            0xc66a => Self::QueueSound { library: 1 },
            0xc673 => Self::QueueSound { library: 2 },
            0xc67c => Self::QueueSound { library: 3 },
            0xf1c6 => Self::SetHeat,
            _ => return None,
        })
    }
}
