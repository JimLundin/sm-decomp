//! The item percentage the ending shows. Spec: ItemPercentage.

use core::num::Wrapping;

use clean_types::Set;

use crate::inventory::{Count, Inventory, Supply};
use crate::rules::Rules;

impl Inventory {
    /// How much of the game's equipment she found: each capacity divided by its divisor, rounded
    /// down, summed, plus the count of her items and her beams, `Σ capacity ÷ divisor + items +
    /// beams`. Only what is hers counts, switched on or not, so a beam switched on that isn't
    /// hers adds nothing; and the 99 energy she starts with counts 0. Spec: ItemPercentage.
    #[must_use]
    pub fn item_percentage(&self, rules: &Rules) -> Count {
        let share = |&supply| Wrapping(self.capacity[supply].0 / rules.percentage_divisors[supply]);
        let tanks: Count = Supply::ALL.iter().map(share).sum();
        tanks + members(&self.items) + members(&self.beams)
    }
}

/// How many members `set` holds: each member in it counts one.
fn members<K, const N: usize>(set: &Set<K, N>) -> Count {
    set.values()
        .filter(|&&is_in| is_in)
        .map(|_| Wrapping(1))
        .sum()
}
