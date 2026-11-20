//! What a frame reports for other domains to act on.

use serde::{Deserialize, Serialize};

use crate::item::SelectableItem;

/// Something the HUD reports for other domains to act on, in the order raised.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum HudEvent {
    /// Spec: Switching. Select or item cancel acted; the item it landed on, the same item or
    /// `None` included. The weapons and her look read it. Ordinary externally tagged Serde
    /// keeps the `Switched` identity even when TOML omits the absent optional item field.
    Switched {
        /// The selected item after the controls acted; `None` means no item selected.
        item: Option<SelectableItem>,
    },
    /// Spec: SwitchClick. A request for the switch's click; the sound domain decides when, or
    /// whether, it is heard.
    Click,
}
