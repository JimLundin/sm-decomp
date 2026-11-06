//! The scan-out of the previous picture (D6 step 2, D11): per line, the line's changes, the
//! HDMA channels and the interrupt strand's command due at that line.
//!
//! Slice 0's skeleton: the interrupt strand's commands do nothing yet, so the picture carries
//! only which command was due. B6 fills in the HUD commands, HDMA and the `LinePoint` changes.

use crate::state::State;
use crate::state::strand::{IrqCommand, IrqStrand};

/// What the scan-out of one picture produced.
// open: the `DisplayFrame` (its line program and the registers per line) arrives with B5 and
// B6; until then the picture is the interrupt strand's state at the start of the scan-out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Picture {
    /// The interrupt strand's command due during this picture, and its line.
    pub irq: IrqStrand,
}

/// Scan out the previous picture.
pub const fn scan_out(state: &mut State) -> Picture {
    let picture = Picture { irq: state.irq };
    state.irq = interrupt(state.irq);
    picture
}

/// The interrupt strand's command at its line: the next command and line.
// open: the HUD's split and the playfield's (B6) chain to each other at lines 0 and 31; until
// they do, every command leaves the strand as it is.
const fn interrupt(irq: IrqStrand) -> IrqStrand {
    match irq.command {
        IrqCommand::Off | IrqCommand::GameplayHud | IrqCommand::GameplayPlayfield => irq,
    }
}
