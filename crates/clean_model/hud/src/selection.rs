//! How Select, item cancel and a shot change the selection (§3).

use crate::events::Event;
use crate::inputs::{CancelButton, Controls, SelectButton, Status};
use crate::item::SelectableItem;
use crate::state::Selection;

impl Selection {
    /// The selection after the buttons, and the `Switched` event when they act.
    ///
    /// Spec: CyclingItems. While she can't switch, nothing changes. Spec: CancellingTheItem. Item
    /// cancel pressed clears the selection and the one-shot item, whatever Select does. Otherwise
    /// Select pressed moves to the next item she has; spec: OneShotItem, with item cancel held the
    /// new selection is also the one-shot item, without it there is none. Spec: Switching. Either
    /// acting reports the item it lands on.
    #[must_use]
    pub fn after_buttons(self, controls: &Controls, status: &Status) -> (Self, Option<Event>) {
        if !controls.can_switch {
            return (self, None);
        }
        if controls.cancel == CancelButton::Pressed {
            return (Self::NONE, Some(Event::Switched(None)));
        }
        if controls.select == SelectButton::Up {
            return (self, None);
        }
        let selected = next_item(self.selected, status);
        let one_shot = match controls.cancel {
            CancelButton::Held => selected,
            CancelButton::Up | CancelButton::Pressed => None,
        };
        (Self { selected, one_shot }, Some(Event::Switched(selected)))
    }

    /// Spec: CancellingTheItem. After the buttons, a shot the weapons cancel clears the selection
    /// and the one-shot item.
    #[must_use]
    pub const fn after_shot(self, controls: &Controls) -> Self {
        if controls.shot_cancels {
            Self::NONE
        } else {
            self
        }
    }
}

/// Spec: CyclingItems. The first item after `current` in the HUD's order that she has (an
/// ammunition with a count, an item she has on), or `None` past the x-ray scope.
fn next_item(current: Option<SelectableItem>, status: &Status) -> Option<SelectableItem> {
    let start = current.map_or(0, |item| {
        SelectableItem::ORDER
            .iter()
            .position(|&other| other == item)
            .map_or(SelectableItem::ORDER.len(), |at| at.saturating_add(1))
    });
    SelectableItem::ORDER
        .iter()
        .skip(start)
        .copied()
        .find(|&item| status.has(item))
}
