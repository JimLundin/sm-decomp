//! What a frame reports for other domains to act on.

use crate::item::SelectableItem;

/// Something the HUD reports, in the order raised.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    /// Spec: Switching. Select or item cancel acted; the item it landed on, the same item or
    /// `None` included. The weapons and her look read it.
    Switched(Option<SelectableItem>),
    /// Spec: SwitchClick. A request for the switch's click; the sound domain decides when, or
    /// whether, it is heard.
    Click,
}
