//! The item percentage the ending shows. Spec: ItemPercentage.

use core::num::Wrapping;

use crate::count::Count;
use crate::inventory::Inventory;
use crate::rules::Rules;
use crate::supplies::Supply;

impl Inventory {
    /// How much of the game's equipment she found: each capacity divided by its divisor, rounded
    /// down, summed, plus the count of her items and her beams, `Σ capacity ÷ divisor + items +
    /// beams`. Only what is hers counts, equipped or not, so a beam equipped that isn't hers adds
    /// nothing; and the 99 energy she starts with counts 0. Spec: ItemPercentage.
    #[must_use]
    pub fn item_percentage(&self, rules: &Rules) -> Count {
        let share = |supply| {
            let divisor = rules.percentage_divisors[supply];
            Wrapping(self.capacity[supply].0.div_euclid(divisor.0))
        };
        let tanks: Count = Supply::ALL.iter().copied().map(share).sum();
        tanks + self.items.count() + self.beams.count()
    }
}
