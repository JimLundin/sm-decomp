//! How Select, item cancel and a shot change the selection (§3).

use crate::events::Event;
use crate::inputs::{CancelButton, Controls, SelectButton, Status};
use crate::item::SelectableItem;
use crate::state::{HudState, Selection};

impl HudState {
    /// The buttons change the selection, and report `Switched` when they act.
    ///
    /// Spec: CyclingItems. While she can't switch, nothing changes. Spec: CancellingTheItem. Item
    /// cancel pressed clears the selection, and with it the one-shot item, whatever Select does.
    /// Otherwise Select pressed moves to the next item she has; spec: OneShotItem, with item
    /// cancel held the new selection is the one-shot item, without it it isn't. Spec: Switching.
    /// Either acting reports the item it lands on.
    pub(crate) fn press_buttons(&mut self, controls: Controls, status: &Status) -> Option<Event> {
        if !controls.can_switch {
            return None;
        }
        if controls.cancel == CancelButton::Pressed {
            self.selection = Selection::None;
            return Some(Event::Switched(None));
        }
        if controls.select == SelectButton::NotPressed {
            return None;
        }
        let next = next_item(self.selection.item(), status);
        self.selection = match (next, controls.cancel) {
            (None, _) => Selection::None,
            (Some(item), CancelButton::Held) => Selection::OneShot(item),
            (Some(item), CancelButton::NotPressed | CancelButton::Pressed) => Selection::Item(item),
        };
        Some(Event::Switched(next))
    }

    /// Spec: CancellingTheItem. After the buttons, a shot the weapons cancel clears the selection,
    /// and with it the one-shot item.
    pub(crate) const fn shot_cancels(&mut self, controls: Controls) {
        if controls.shot_cancels {
            self.selection = Selection::None;
        }
    }
}

/// Spec: CyclingItems. The first item after `current` in the HUD's order that she has (an
/// ammunition with a count, an item she has on), or `None` past the x-ray scope.
fn next_item(current: Option<SelectableItem>, status: &Status) -> Option<SelectableItem> {
    let mut order = SelectableItem::ORDER.into_iter();
    if let Some(current) = current {
        order.find(|&item| item == current);
    }
    order.find(|&item| status.has(item))
}
