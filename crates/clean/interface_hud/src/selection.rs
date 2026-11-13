//! How Select, item cancel and another domain's drop change the selection (spec:
//! CyclingItems, CancellingTheItem, OneShotItem, Switching).

use crate::events::HudEvent;
use crate::inputs::{CancelButton, Controls, Dropped, SelectButton, Status};
use crate::item::SelectableItem;
use crate::state::{HudState, Selection};

impl Selection {
    /// Spec: CancellingTheItem. A drop, at the point of the step it lands: the selection dropped
    /// leaves none, and with it no one-shot item; the one-shot item dropped alone keeps the item
    /// selected, no longer the one-shot item. It holds whether or not she can switch.
    pub(crate) const fn apply_drop(&mut self, dropped: Dropped) {
        *self = match (dropped, *self) {
            (Dropped::Selection, _) => Self::None,
            (Dropped::OneShotItemAlone, Self::OneShot(item)) => Self::Item(item),
            (Dropped::Nothing | Dropped::OneShotItemAlone, unchanged) => unchanged,
        };
    }

    /// Spec: CyclingItems. Select moves the selection to the first item after it, in the HUD's
    /// order, that she has (an ammunition with a count, an item she has on); with none left
    /// before the end, past the x-ray scope, to none. Spec: OneShotItem. With item cancel held
    /// the item it lands on is the one-shot item; otherwise it isn't (item cancel pressed drops
    /// the selection instead of cycling).
    fn cycle(&mut self, status: &Status, cancel: CancelButton) {
        let current = self.item();
        let next = SelectableItem::ORDER
            .into_iter()
            .find(|&item| Some(item) > current && status.has(item));
        *self = match (next, cancel) {
            (None, _) => Self::None,
            (Some(item), CancelButton::Held) => Self::OneShot(item),
            (Some(item), CancelButton::NotPressed | CancelButton::Pressed) => Self::Item(item),
        };
    }
}

impl HudState {
    /// The buttons change the selection, and report `Switched` when they act.
    ///
    /// Spec: CyclingItems. While she can't switch, nothing changes. Spec: CancellingTheItem. Item
    /// cancel pressed drops the selection, whatever Select does. Otherwise Select pressed cycles.
    /// Spec: Switching. Either acting reports the item it lands on.
    pub(crate) fn press_buttons(
        &mut self,
        controls: Controls,
        status: &Status,
    ) -> Option<HudEvent> {
        if !controls.can_switch {
            return None;
        }
        match (controls.cancel, controls.select) {
            (CancelButton::Pressed, _) => self.selection.apply_drop(Dropped::Selection),
            (cancel, SelectButton::Pressed) => self.selection.cycle(status, cancel),
            (_, SelectButton::NotPressed) => return None,
        }
        Some(HudEvent::Switched(self.selection.item()))
    }
}
