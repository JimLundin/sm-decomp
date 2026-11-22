//! Actual unselected preparation owner facts and separately borrowed completions.
use super::borrowed::{ActorEligibilityOrigin, ActorSlot, SpriteBuilderOrigin};
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

/// Actual completed actor membership in physical slot order.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ActorEligibilityReceipt {
    /// Complete actual raw eligibility dependency.
    pub dependency: ActorEligibilityOrigin,
    /// Actual completed active actor list in append order.
    pub active: Vec<ActorSlotIdentity>,
    /// Actual completed interactive actor list in append order.
    pub interactive: Vec<ActorSlotIdentity>,
}

impl ActorEligibilityOrigin {
    pub(super) fn supports_marked_offscreen(&self) -> bool {
        self.permit_marked_offscreen
            && matches!(
                &self.slots[..],
                [
                    ActorSlot::Occupied {
                        deleted: false,
                        process_offscreen: true,
                        ..
                    },
                    ActorSlot::Occupied {
                        deleted: false,
                        process_offscreen: true,
                        intangible: false,
                    },
                    rest @ ..
                ] if rest.iter().all(|slot| *slot == ActorSlot::Empty)
            )
    }
}

impl ActorEligibilityReceipt {
    pub(super) fn marked_offscreen(current: &ActorEligibilityOrigin) -> Option<Self> {
        if !current.supports_marked_offscreen() {
            return None;
        }
        let mut active = Vec::new();
        let mut interactive = Vec::new();
        for (index, slot) in current.slots.iter().enumerate() {
            if let ActorSlot::Occupied { intangible, .. } = slot {
                let identity = ActorSlotIdentity(u8::try_from(index).ok()?);
                active.push(identity);
                if !intangible {
                    interactive.push(identity);
                }
            }
        }
        Some(Self {
            dependency: current.clone(),
            active,
            interactive,
        })
    }
}

/// Actual admitted debug mode.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugMode {
    /// Ordinary debug processing.
    Normal,
    /// A mode whose operation requires other current owners.
    Other,
}
/// Physical controller newly-pressed buttons, independent of mapped movement controls.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalPadEdges(pub u16);
/// Complete actual physical debug decision inputs.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugOrigin {
    /// Actual current debug mode.
    pub mode: DebugMode,
    /// First controller edges qualified by the physical debug shortcut latch.
    pub shortcut: PhysicalPadEdges,
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

impl DebugOrigin {
    pub(super) fn supports_quiet(self) -> bool {
        const SHORTCUT_ACTIONS: u16 = 0x00c0; // A and X
        const CONTROLLER_TWO_ACTIONS: u16 = 0x30b0; // Start, Select, L, R and A
        self.mode == DebugMode::Normal
            && self.shortcut.0 & SHORTCUT_ACTIONS == 0
            && self.controller_two.0 & CONTROLLER_TWO_ACTIONS == 0
    }

    pub(super) fn complete_quiet(self) -> Option<DebugCompletionReceipt> {
        self.supports_quiet().then_some(DebugCompletionReceipt {
            dependency: self,
            terminal: DebugMode::Normal,
            action: DebugAction::None,
            skip_processing: false,
        })
    }
}

/// Current palette object occupancy.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteSlot {
    /// No active palette object occupies this slot.
    Empty,
    /// An active object that needs its own service.
    Occupied,
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

impl PaletteOrigin {
    pub(super) fn supports_empty_scan(self) -> bool {
        self.enabled && self.slots.iter().all(|slot| *slot == PaletteSlot::Empty)
    }

    pub(super) fn complete_empty_scan(self) -> Option<PaletteCompletionReceipt> {
        self.supports_empty_scan()
            .then_some(PaletteCompletionReceipt {
                dependency: self,
                terminal: PaletteDisposition::RetainCurrent,
            })
    }
}
