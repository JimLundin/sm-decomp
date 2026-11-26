//! Spec: `MenuFiles`. The file menu's scan of all three slots.

use types::file_select::Slot;
use types::save_files::MenuInspection;

use crate::files::{Refusal, SaveFiles};
use crate::format::SaveFormat;
use crate::read::ReadReceipt;
use crate::selection::{MENU_ORDER, initial_cursor};

/// Spec: `MenuFiles`. What the menu scan found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuReport {
    /// The three receipts, in menu order; each gives its file's summary.
    pub receipts: Vec<ReadReceipt>,
    /// The menu's initial cursor.
    pub cursor: Slot,
}

impl<F: SaveFormat> SaveFiles<F> {
    /// Spec: `MenuFiles`. Read all three slots in menu order, each successful read carrying its
    /// profile and maps into the next, and place the cursor from the remembered selection. The
    /// finished scan admits a choice.
    ///
    /// # Errors
    ///
    /// `ActionPending` while a choice or write stands between its milestones.
    pub fn menu_files(&mut self) -> Result<MenuReport, Refusal> {
        self.admit()?;
        let receipts = MENU_ORDER.iter().map(|&slot| self.read(slot)).collect();
        self.state.menu = MenuInspection::Inspected;
        Ok(MenuReport {
            receipts,
            cursor: initial_cursor(self.state.cartridge.selection),
        })
    }
}
