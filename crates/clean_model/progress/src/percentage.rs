//! The item percentage the ending shows. Spec: ItemPercentage.

use core::num::Wrapping;

use clean_types::Members;

use crate::inventory::{Count, Inventory};
use crate::rules::Rules;

impl Inventory {
    /// How much of the game's equipment she found: each capacity divided by its divisor, rounded
    /// down, summed, plus the count of her items and her beams, `Σ capacity ÷ divisor + items +
    /// beams`. Only what is hers counts, switched on or not, so a beam switched on that isn't
    /// hers adds nothing; and the 99 energy she starts with counts 0. Spec: ItemPercentage.
    #[must_use]
    pub fn item_percentage(&self, rules: &Rules) -> Count {
        let share = |(supply, capacity): (_, &Count)| {
            capacity.0 / rules.percentage_divisors[supply].divisor
        };
        let tanks: Count = self.capacity.iter().map(share).map(Wrapping).sum();
        tanks + self.items.count() + self.beams.count()
    }
}
