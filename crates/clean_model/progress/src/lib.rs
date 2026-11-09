//! The progress model, built from its behaviour spec alone: what the game remembers of her run
//! besides where she is.
//!
//! A `Progress` holds what she has (its `Inventory`), what has happened in the world and what
//! its places remember, each a public field other domains read as it is now, and a caller sets
//! and reads back whole: `events[event]`, `defeated[area].intersects`, `emptied.contains`,
//! `opened.contains` (and the ids remembered, `emptied.0` and `opened.0`),
//! `maps[area].is_known`, `zebetites`, `inventory.items_switched_on[item]` (what her motion and
//! her sprite read), `inventory.item_percentage`. Tables keyed by an enum are
//! `enum_map::EnumMap`s, and sets of an enum's members are `clean_types::Set`s, read and changed
//! as `table[member]`. Its default is what a new file holds. Domains that act tell it what
//! happened with a `Report`, which `Progress::apply` records. A room picks the version it loads
//! with `RoomStates::choose`.
#![expect(
    clippy::doc_markdown,
    reason = "spec entries such as EventsHappen are named plainly, as Spec: Name, not as Rust items"
)]

mod catalogue;
mod inventory;
mod percentage;
mod places;
mod progress;
mod report;
mod room_states;
mod rules;

pub use catalogue::{Behaviour, BehaviourId, catalogue};
pub use inventory::{Count, Inventory, Pickup, ReserveMode, Supply};
pub use places::{MapStatus, Remembered};
pub use progress::Progress;
pub use report::Report;
pub use room_states::{Condition, EnteredRoom, RoomStates, Version};
pub use rules::Rules;

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use enum_map::{Enum, EnumMap};

    use crate::catalogue::CATALOGUE;
    use crate::rules::{REFERENCE_DIVISORS, REFERENCE_SWITCHES_OFF};

    /// Each member of `K` that `rows` of `table` don't name exactly once, and how many do.
    fn not_named_once<K: Enum + Debug + PartialEq, R>(
        table: &str,
        rows: &[R],
        member: fn(&R) -> K,
    ) -> Vec<String> {
        EnumMap::<K, ()>::default()
            .into_iter()
            .filter_map(|(k, ())| {
                let named = rows.iter().filter(|row| member(row) == k).count();
                (named != 1).then(|| format!("{table}: {k:?} is named by {named} rows, not 1"))
            })
            .collect()
    }

    #[test]
    fn every_table_names_each_member_once() {
        let failures = [
            not_named_once("CATALOGUE", &CATALOGUE, |b| b.id),
            not_named_once("REFERENCE_SWITCHES_OFF", &REFERENCE_SWITCHES_OFF, |r| r.0),
            not_named_once("REFERENCE_DIVISORS", &REFERENCE_DIVISORS, |r| r.0),
        ]
        .concat();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
