//! The cartridge's live owner: the format it was built with and the State it carries.

use types::save_files::{PendingAction, SaveFilesState};

/// The cartridge's live owner. Every request runs through it, and its State is the caller's to
/// carry, inspect and restore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveFiles<F> {
    /// The format that encodes runs and names the completion marks.
    pub format: F,
    /// Everything the model carries between requests.
    pub state: SaveFilesState,
}

/// Why a request was not admitted. A refused request changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A choice or write has reached its first milestone: only its completion runs.
    ActionPending,
    /// No menu scan has finished this cartridge session, so no file can be chosen.
    MenuUninspected,
    /// No file is chosen, so there is no file to save to.
    NoActiveSlot,
    /// There is no choice or write of that kind to complete.
    NothingToComplete,
}

impl<F> SaveFiles<F> {
    /// The owner of this State, with this format.
    #[must_use]
    pub const fn new(format: F, state: SaveFilesState) -> Self {
        Self { format, state }
    }

    /// A new request runs only while no choice or write stands between its milestones.
    pub(crate) const fn admit(&self) -> Result<(), Refusal> {
        match self.state.pending {
            PendingAction::Idle => Ok(()),
            PendingAction::Choice(_) | PendingAction::Write { .. } => Err(Refusal::ActionPending),
        }
    }
}
