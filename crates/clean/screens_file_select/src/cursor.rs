//! The cursor: where it starts (`CursorStarts`) and how Up and Down move it round the menu,
//! passing copy and clear by when they are not offered (`CursorMoves`).

use types::Members;
use types::file_select::{CopyAndClear, MenuItem, Slot};

use crate::input::SaveMemory;

/// A move of the cursor, one item up or down the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorStep {
    /// To the item above, from slot A round to exit.
    Up,
    /// To the item below, from exit round to slot A.
    Down,
}

/// Spec: `CursorMoves`. With copy and clear not offered, the cursor reaching `onto` goes on to
/// `to` instead.
struct Skip {
    /// The item the step would land on.
    onto: MenuItem,
    /// Where the cursor lands in its place.
    to: MenuItem,
}

/// Spec: `CursorMoves`. The rule that passes by the items not on offer, for each way the cursor
/// steps. Going down, copy and clear each go on to exit. Going up, only clear goes on, straight
/// to slot C: so from clear itself the cursor stops on copy.
const fn skips(step: CursorStep) -> &'static [Skip] {
    match step {
        CursorStep::Up => &[Skip {
            onto: MenuItem::Clear,
            to: MenuItem::File(Slot::Third),
        }],
        CursorStep::Down => &[
            Skip {
                onto: MenuItem::Copy,
                to: MenuItem::Exit,
            },
            Skip {
                onto: MenuItem::Clear,
                to: MenuItem::Exit,
            },
        ],
    }
}

/// Spec: `CursorMoves`. Where one step takes the cursor from `from`.
#[must_use]
pub fn stepped(from: MenuItem, step: CursorStep, offer: CopyAndClear) -> MenuItem {
    let next = neighbour(from, step);
    match offer {
        CopyAndClear::Offered => next,
        CopyAndClear::NotOffered => skips(step)
            .iter()
            .find(|skip| skip.onto == next)
            .map_or(next, |skip| skip.to),
    }
}

/// The item next to `from` on the menu, one step up or down, round from one end to the other.
fn neighbour(from: MenuItem, step: CursorStep) -> MenuItem {
    match step {
        CursorStep::Up => next_after(from, MenuItem::TOP_TO_BOTTOM.iter().rev()),
        CursorStep::Down => next_after(from, MenuItem::TOP_TO_BOTTOM.iter()),
    }
}

/// The item after `from` in `order`, round from its end to its start.
fn next_after<'a>(from: MenuItem, order: impl Iterator<Item = &'a MenuItem> + Clone) -> MenuItem {
    order
        .cycle()
        .skip_while(|&&item| item != from)
        .nth(1)
        .copied()
        .unwrap_or(from)
}

/// Spec: `CursorStarts`. The cursor starts on the slot chosen last, or on slot A when save
/// memory holds none valid.
#[must_use]
pub fn starting_item(save: &impl SaveMemory) -> MenuItem {
    MenuItem::File(save.last_chosen().unwrap_or(Slot::First))
}

/// Spec: `CursorStarts`. Copy and clear are offered when some slot holds a file.
#[must_use]
pub fn offer(save: &impl SaveMemory) -> CopyAndClear {
    if save.files().count().0 == 0 {
        CopyAndClear::NotOffered
    } else {
        CopyAndClear::Offered
    }
}
