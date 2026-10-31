//! Bank $87's animated tile programs: definitions, frames and the commands
//! between them.
//!
//! A definition is a program pointer, an upload size and a VRAM address. A
//! program is a list of frames, each a duration word (bit 15 clear) and a
//! source pointer, with commands between them (words with bit 15 set, the
//! address of the command's handler), some followed by an operand.

/// The bank whose upper half holds the programs.
pub const BANK: u8 = 0x87;

/// A program command, decoded from its handler's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// $80B2: delete the animation.
    Delete,
    /// $80B7: go to the next word.
    Goto,
    /// $80BC: go to the next byte's signed displacement.
    GotoRelative,
    /// $80D4 and $80DC: decrement the counter and go to the operand unless it
    /// ran out; `relative` takes a byte displacement instead of a word.
    DecrementGoto { relative: bool },
    /// $80E3: the next byte is the counter.
    SetCounter,
    /// $80F0: queue the music track in the next byte.
    QueueMusic,
    /// $80FC, $8108 and $8114: queue the sound in the next byte on `library`.
    QueueSound { library: u8 },
    /// $8120: go to the word after the next byte if that boss is defeated.
    GotoIfBossDefeated,
    /// $8133: mark the boss in the next byte defeated.
    MarkBossDefeated,
    /// $813F: go to the word after the next word if that event happened.
    GotoIfEvent,
    /// $8150: set the event in the next word.
    SetEvent,
    /// $81BA: wait while the area's main boss is alive.
    WaitForBoss,
    /// $8303: go to the word after the next two bytes if the boss bits in the
    /// first are set for the area in the second.
    GotoIfBossBits,
    /// $8320: spawn a statue eye glow with the next word as parameter.
    SpawnEyeGlow,
    /// $832F: spawn a statue soul with the next word as parameter.
    SpawnSoul,
    /// $833E: go to the next word if the statue animation's bit 15 is set.
    GotoIfStatueAnimating,
    /// $8349: set the statue animation bits in the next word.
    SetStatueBits,
    /// $8352: clear the statue animation bits in the next word.
    ClearStatueBits,
    /// $835B: clear three colours from the palette index in the next word.
    ClearColours,
    /// $8372: spawn the palette effect in the next word.
    SpawnPaletteEffect,
    /// $837F: write the statue's target palette from the index in the next word.
    WriteStatuePalette,
}

impl Command {
    /// The command whose handler is at $87:`handler`, if any.
    pub fn decode(handler: u16) -> Option<Self> {
        Some(match handler {
            0x80b2 => Self::Delete,
            0x80b7 => Self::Goto,
            0x80bc => Self::GotoRelative,
            0x80d4 => Self::DecrementGoto { relative: false },
            0x80dc => Self::DecrementGoto { relative: true },
            0x80e3 => Self::SetCounter,
            0x80f0 => Self::QueueMusic,
            0x80fc => Self::QueueSound { library: 1 },
            0x8108 => Self::QueueSound { library: 2 },
            0x8114 => Self::QueueSound { library: 3 },
            0x8120 => Self::GotoIfBossDefeated,
            0x8133 => Self::MarkBossDefeated,
            0x813f => Self::GotoIfEvent,
            0x8150 => Self::SetEvent,
            0x81ba => Self::WaitForBoss,
            0x8303 => Self::GotoIfBossBits,
            0x8320 => Self::SpawnEyeGlow,
            0x832f => Self::SpawnSoul,
            0x833e => Self::GotoIfStatueAnimating,
            0x8349 => Self::SetStatueBits,
            0x8352 => Self::ClearStatueBits,
            0x835b => Self::ClearColours,
            0x8372 => Self::SpawnPaletteEffect,
            0x837f => Self::WriteStatuePalette,
            _ => return None,
        })
    }
}
