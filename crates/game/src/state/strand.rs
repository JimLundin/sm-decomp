//! The main strand's continuation frames and the interrupt strand (D10).
//!
//! A Checkpoint sits at a cut, so these say where each routine resumes after a wait; a
//! routine that can't span a Tick never appears here.

use canonical::Canonical;
use data::id::Line;

use super::room::Plm;
use super::slots::{Frames, SlotIndex};

/// The most frames the main strand nests; opening a Session checks it.
pub const MAIN_STRAND_DEPTH: usize = 8;

/// Where the main loop resumes within an iteration.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MainLoopAt {
    /// Before the HDMA object handler.
    #[default]
    Start,
    /// After it: the beam-sync point has passed (D7).
    AfterHdmaObjects,
    /// After the game state dispatch, before the sound handler and OAM finalisation.
    AfterGameState,
}

/// Where the gameplay pass resumes.
// open: one variant per phase of the gameplay pass, in the original's order. The bank $82
// main gameplay routine's call list decides them; slice 0 only resumes inside the PLM pass.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameplayAt {
    /// Before the first phase.
    #[default]
    Start,
    /// After the PLM pass returned.
    AfterPlms,
}

/// The message box, opened from inside a PLM instruction (the item's pickup).
///
/// Bank $85 log, $85:8080 display message box: it sets up the PPU, clears the BG3 tilemap,
/// writes the message, plays two lag frames, opens the box, handles the interaction, closes
/// it, clears the tilemap again, restores the PPU and plays two more lag frames. Each variant
/// is a wait inside one of those steps, in that order.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageBoxAt {
    /// Setting up the PPU for the box ($85:8143).
    PreparingPpu(PpuPreparation),
    /// Clearing the box's BG3 tilemap ($85:81F3): its upload to VRAM comes next.
    ClearingTilemap(Side),
    /// Writing the message into the box ($85:8241).
    Writing(MessageWrite),
    /// Playing lag frames of music and sound effects, counted down.
    Lag {
        /// Which countdown.
        at: LagAt,
        /// Lag frames still to come after this one.
        left: Frames,
    },
    /// Growing open.
    Opening {
        /// Frames of the animation left.
        left: Frames,
    },
    /// Fully shown, waiting for a button.
    Shown,
    /// Shrinking closed.
    Closing {
        /// Frames of the animation left.
        left: Frames,
    },
    /// Restoring the PPU after the box ($85:861A).
    RestoringPpu(PpuRestoration),
}

/// Which side of the box's animation a step shared by both sides is on.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Before the box opens.
    BeforeOpen,
    /// After it has closed.
    AfterClose,
}

/// A message box lag countdown.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LagAt {
    /// Bank $85 log, $85:8574 play 2 lag frames, before the box opens.
    BeforeOpen,
    /// Bank $85 log, $85:846D's countdown at $8493 before the button wait: 360 frames, or 10
    /// for the station and save messages.
    Shown,
    /// $85:8574 play 2 lag frames, after the box has closed.
    AfterClose,
}

/// Where setting up the PPU for the box resumes (bank $85 log, $85:8143).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PpuPreparation {
    /// At the registers: HDMA off, the BG3 colours, layers and scroll.
    Registers,
    /// At the backup of the BG3 tilemap the box covers.
    TilemapBackup,
}

/// Where writing the message resumes (bank $85 log, $85:8241 initialise message box).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageWrite {
    /// At the message's tilemap ($85:82B8).
    Tilemap,
    /// At enabling the BG3 Y scroll HDMA, its table written ($85:8363).
    ScrollHdma,
    /// At the tilemap's upload to VRAM ($85:831E).
    TilemapUpload,
}

/// Where restoring the PPU resumes (bank $85 log, $85:861A).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PpuRestoration {
    /// At the BG3 tilemap's restore from its backup.
    Tilemap,
    /// At the registers: HDMA, BG3 tilemap base, layers, colour math and BG3 colours.
    Registers,
}

/// A resumable routine on the main strand.
// open: the other routines (door, pause, death, cinematics, boot) join with their clusters.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    /// The main loop.
    MainLoop(MainLoopAt),
    /// The gameplay pass.
    Gameplay(GameplayAt),
    /// The PLM pass, resuming at a slot.
    Plms {
        /// The slot whose instruction called out.
        slot: SlotIndex<Plm>,
    },
    /// The message box.
    MessageBox(MessageBoxAt),
}

/// What a test-each-Tick wait waits for.
// open: the rest (door scroll done, door VRAM updated, APU ready) join with doors and sound.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// The music queue has drained.
    MusicQueueEmpty,
}

/// What a yielded main strand waits for.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    /// A requested NMI: the batch commits and the pad latches.
    Nmi,
    /// A lag NMI: counters only.
    LagFrame,
    /// An NMI, then the condition tested at the next resume.
    Until(Condition),
}

/// The main strand: a short stack of continuation frames, innermost last.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct MainStrand {
    /// At most `MAIN_STRAND_DEPTH`.
    pub frames: Vec<Frame>,
    /// What it waits for, once it has yielded.
    pub waiting: Option<Wait>,
}

impl Default for MainStrand {
    /// The main loop at its start.
    fn default() -> Self {
        Self {
            frames: vec![Frame::MainLoop(MainLoopAt::Start)],
            waiting: None,
        }
    }
}

/// The interrupt strand's command.
// open: the door commands and Draygon's HUD join with their clusters.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IrqCommand {
    /// No interrupt work.
    #[default]
    Off,
    /// The gameplay HUD's split at its top.
    GameplayHud,
    /// The split back to the playfield below the HUD.
    GameplayPlayfield,
}

/// The interrupt strand: the next command and the line it fires at.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IrqStrand {
    /// The command.
    pub command: IrqCommand,
    /// Its line.
    pub line: Line,
}
