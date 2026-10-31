//! HDMA object program commands. Their handlers are in bank $88, whatever
//! bank the program itself is in.
//!
//! An instruction is a timer word and a table pointer; a word with bit 15 set
//! is instead the address of a command's handler, some followed by operands.

/// A program command, decoded from its handler's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// $DB2E: does nothing.
    Nothing,
    /// $8569: delete the object.
    Delete,
    /// $8570: the next three bytes are the pre-instruction.
    SetPreInstruction,
    /// $8584: clear the pre-instruction ($858A, which returns at once).
    ClearPreInstruction,
    /// $85B4: call the routine at the next three bytes.
    Call,
    /// $85EC: go to the next word.
    Goto,
    /// $85F1: go to the next byte's signed displacement.
    GotoRelative,
    /// $8607 and $860F: decrement the timer and go to the operand unless it
    /// ran out; `relative` takes a byte displacement instead of a word.
    DecrementGoto { relative: bool },
    /// $8616: the next byte is the timer.
    SetTimer,
    /// $8622, $8637, $8655 and $866A: write the next byte to one of the
    /// object's DMA channel registers.
    SetChannelRegister(ChannelRegister),
    /// $864C: the next word is the HDMA table.
    SetTable,
    /// $8682: sleep, resuming at this command next frame.
    Sleep,
    /// $A66C, $B4CE, $C467 and $C582: set the object's variable B to 1.
    SetVariableB,
    /// $B3A9: set the object's variable C to 112.
    SetVariableC,
    /// $D916 and $D949: select the Ceres HDMA data table.
    SelectCeresTable,
    /// $D981: pick the BG3 scroll velocity at random.
    PickBg3Velocity,
    /// $DCCB: set up the scroll offset by event $0A, going to the next word if
    /// it has happened.
    EventScroll,
    /// $E4BD: initialise Phantoon's wave.
    InitializePhantoonWave,
    /// $E917: initialise the x-ray or morph eye beam.
    InitializeEyeBeam,
    /// $EC9F: reset the wave.
    ResetWave,
    /// $EB9F: restart the title gradient.
    RestartTitleGradient,
}

/// The DMA channel register a command writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelRegister {
    /// DMAPx, the transfer control.
    Control,
    /// BBADx, the B-bus target.
    Target,
    /// A1Bx, the table's bank.
    TableBank,
    /// DASBx, the indirect data bank.
    IndirectBank,
}

impl Command {
    /// The command whose handler is at $88:`handler`, if any.
    pub fn decode(handler: u16) -> Option<Self> {
        use ChannelRegister::*;
        Some(match handler {
            0xdb2e => Self::Nothing,
            0x8569 => Self::Delete,
            0x8570 => Self::SetPreInstruction,
            0x8584 => Self::ClearPreInstruction,
            0x85b4 => Self::Call,
            0x85ec => Self::Goto,
            0x85f1 => Self::GotoRelative,
            0x8607 => Self::DecrementGoto { relative: false },
            0x860f => Self::DecrementGoto { relative: true },
            0x8616 => Self::SetTimer,
            0x8622 => Self::SetChannelRegister(Control),
            0x8637 => Self::SetChannelRegister(Target),
            0x8655 => Self::SetChannelRegister(TableBank),
            0x866a => Self::SetChannelRegister(IndirectBank),
            0x864c => Self::SetTable,
            0x8682 => Self::Sleep,
            0xa66c | 0xb4ce | 0xc467 | 0xc582 => Self::SetVariableB,
            0xb3a9 => Self::SetVariableC,
            0xd916 | 0xd949 => Self::SelectCeresTable,
            0xd981 => Self::PickBg3Velocity,
            0xdccb => Self::EventScroll,
            0xe4bd => Self::InitializePhantoonWave,
            0xe917 => Self::InitializeEyeBeam,
            0xec9f => Self::ResetWave,
            0xeb9f => Self::RestartTitleGradient,
            _ => return None,
        })
    }
}
