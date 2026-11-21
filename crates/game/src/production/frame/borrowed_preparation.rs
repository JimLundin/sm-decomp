//! Actual unselected preparation owner facts and separately borrowed completions.
use super::borrowed::{ActorEligibilityOrigin, SpriteBuilderOrigin};
use canonical::Canonical;
use serde::{Deserialize, Serialize};

/// Complete actual sprite-builder reset owner result.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct BuilderCompletionReceipt {
    /// Complete actual current builder, before its operation.
    pub dependency: SpriteBuilderOrigin,
    /// Complete builder after the current sprite-list reset.
    pub terminal: SpriteBuilderOrigin,
}

impl BuilderCompletionReceipt {
    /// Reset the high OAM backing and leave the previous low bytes available.
    #[must_use]
    pub fn reset(dependency: SpriteBuilderOrigin) -> Self {
        const LOW_BACKING_BYTES: usize = 512;
        let mut backing = dependency.backing;
        for byte in backing.iter_mut().skip(LOW_BACKING_BYTES) {
            *byte = 0;
        }
        Self {
            dependency,
            terminal: SpriteBuilderOrigin {
                backing,
                cursor: 0,
                prepared_flags: 0,
                prepared_top: 0,
                prepared_bottom: 0,
            },
        }
    }
}

/// Physical raw actor slot identity, independent of list position or geometry.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSlotIdentity(pub u8);

/// Actual completed actor membership, without executing eligibility traversal.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ActorEligibilityReceipt {
    /// Complete actual raw eligibility dependency.
    pub dependency: ActorEligibilityOrigin,
    /// Actual completed active actor list in append order.
    pub active: Vec<ActorSlotIdentity>,
    /// Actual completed interactive actor list in append order.
    pub interactive: Vec<ActorSlotIdentity>,
}

/// Actual admitted debug mode.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugMode {
    /// Ordinary debug processing.
    Normal,
}
/// Physical controller newly-pressed buttons, independent of mapped movement controls.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalPadEdges(pub u16);
/// Complete actual physical debug decision inputs.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugOrigin {
    /// Actual current debug mode.
    pub mode: DebugMode,
    /// Actual first controller physical edges.
    pub controller_one: PhysicalPadEdges,
    /// Actual second controller physical edges.
    pub controller_two: PhysicalPadEdges,
}
/// Actual completed debug action.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugAction {
    /// No debug action was performed.
    None,
}
/// Separately borrowed actual debug owner completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct DebugCompletionReceipt {
    /// Actual current physical debug dependency.
    pub dependency: DebugOrigin,
    /// Actual terminal debug mode.
    pub terminal: DebugMode,
    /// Actual completed action.
    pub action: DebugAction,
    /// Actual owner result permitting ordinary processing to continue.
    pub skip_processing: bool,
}

/// Current palette object occupancy.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteSlot {
    /// No active palette object occupies this slot.
    Empty,
}
/// Actual palette owner eligibility, retaining physical slots in descending owner order.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteOrigin {
    /// Actual owner enable premise.
    pub enabled: bool,
    /// Complete current object slots.
    pub slots: [PaletteSlot; 8],
}
/// Actual completed palette disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteDisposition {
    /// Retain the complete actual currently owned palette without replacement.
    RetainCurrent,
}
/// Separately borrowed actual palette object service completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct PaletteCompletionReceipt {
    /// Actual current palette object eligibility dependency.
    pub dependency: PaletteOrigin,
    /// Actual terminal palette ownership disposition.
    pub terminal: PaletteDisposition,
}
