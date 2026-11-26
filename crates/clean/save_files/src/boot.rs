//! Spec: `BootFiles`. The cartridge check and the startup reads.

use types::save_files::{DemoCount, MenuInspection};

use crate::files::{Refusal, SaveFiles};
use crate::format::SaveFormat;
use crate::read::ReadReceipt;
use crate::selection::MENU_ORDER;

/// Spec: `BootFiles`. What boot found: each startup read in menu order, and the demo count.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootReport {
    /// The receipts of the slots read, in menu order, up to the first valid file.
    pub inspected: Vec<ReadReceipt>,
    /// How many attract sequences, the recorded scenes on the title screen, are available.
    pub demos: DemoCount,
}

impl<F: SaveFormat> SaveFiles<F> {
    /// Spec: `BootFiles`. Check the cartridge, then read the three player slots in menu order up
    /// to the first valid file. A valid file makes four attract sequences available if the mark
    /// says completed and three otherwise; no valid file marks the game incomplete and makes
    /// three available. Boot begins a cartridge session,
    /// so the menu is uninspected.
    ///
    /// The cartridge check restores every byte as it finishes, before any read can clear a
    /// damaged file, so it leaves the cartridge as supplied; an interrupted check is the
    /// platform's, and reaches the model as the cartridge it supplies.
    ///
    /// # Errors
    ///
    /// `ActionPending` while a choice or write stands between its milestones.
    pub fn boot_files(&mut self) -> Result<BootReport, Refusal> {
        self.admit()?;
        self.state.menu = MenuInspection::Uninspected;
        let mut inspected = Vec::new();
        for &slot in MENU_ORDER {
            let receipt = self.read(slot);
            let found = receipt.is_valid();
            inspected.push(receipt);
            if found {
                let demos = if self.state.cartridge.mark == self.format.completed_mark() {
                    DemoCount::Four
                } else {
                    DemoCount::Three
                };
                return Ok(BootReport { inspected, demos });
            }
        }
        self.state.cartridge.mark = self.format.incomplete_mark();
        Ok(BootReport {
            inspected,
            demos: DemoCount::Three,
        })
    }
}
