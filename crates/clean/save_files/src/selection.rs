//! The slots' numbers and the remembered selection that names one.

use types::file_select::Slot;
use types::save_files::RememberedSelection;

/// The slots in menu order, A, B, C: the order boot and the menu read them.
pub const MENU_ORDER: &[Slot] = &[Slot::First, Slot::Second, Slot::Third];

/// Spec: `ChooseFile`. A slot's number: A is 0, B is 1, C is 2.
#[must_use]
pub const fn slot_number(slot: Slot) -> u16 {
    match slot {
        Slot::First => 0,
        Slot::Second => 1,
        Slot::Third => 2,
    }
}

/// The slot with this number, if one has it.
fn numbered_slot(number: u16) -> Option<Slot> {
    MENU_ORDER
        .iter()
        .copied()
        .find(|&slot| slot_number(slot) == number)
}

/// Spec: `ChooseFile`. The selection a choice writes: the slot's number and its 16-bit bitwise
/// complement.
#[must_use]
pub const fn selection_of(slot: Slot) -> RememberedSelection {
    let number = slot_number(slot);
    RememberedSelection {
        slot_word: number,
        check_word: !number,
    }
}

/// Spec: `MenuFiles`. The menu's first cursor, from the remembered selection.
///
/// It is the remembered slot when its word numbers a slot (`0 <= slot <= 2`) and shares no bit
/// with the selection check (`slot & check == 0`), and A otherwise. The test is weak: a check
/// of zero passes any slot, and slot A's zero passes any check.
#[must_use]
pub fn initial_cursor(selection: RememberedSelection) -> Slot {
    let check_passes = selection.slot_word & selection.check_word == 0;
    match numbered_slot(selection.slot_word) {
        Some(slot) if check_passes => slot,
        Some(_) | None => Slot::First,
    }
}
