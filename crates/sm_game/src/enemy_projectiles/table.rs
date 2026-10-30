//! The enemy projectile slots as Session-owned fields, for code outside the
//! projectile host: the owned Session's under one lease when one can open,
//! otherwise a copy decoded from RAM and stored back if it changed.
#![forbid(unsafe_code)]

use crate::ram::{RawRam as _, WorkRamAccess};
use crate::runtime_access::RuntimeAccess;
use crate::session::{EnemyProjectile, EnemyProjectiles};

/// Projectile slots the original runs at once.
pub const SLOTS: usize = 18;

/// The slots' RAM words, in `session_encoding` order.
pub(crate) fn slot_words(
    slots: &mut [EnemyProjectile; SLOTS],
    mut each: impl FnMut(usize, &mut u16),
) {
    for (index, slot) in slots.iter_mut().enumerate() {
        let at = 2 * index;
        for (base, word) in [
            (0x1997, &mut slot.definition),
            (0x19bb, &mut slot.graphics),
            (0x19df, &mut slot.counter),
            (0x1a03, &mut slot.pre_action),
            (0x1a27, &mut slot.x.fraction),
            (0x1a4b, &mut slot.x.whole),
            (0x1a6f, &mut slot.y.fraction),
            (0x1a93, &mut slot.y.whole),
            (0x1ab7, &mut slot.velocity.x),
            (0x1adb, &mut slot.velocity.y),
            (0x1aff, &mut slot.behavior_a),
            (0x1b23, &mut slot.behavior_b),
            (0x1b47, &mut slot.program),
            (0x1b6b, &mut slot.spritemap),
            (0x1b8f, &mut slot.delay),
            (0x1bb3, &mut slot.radius),
            (0x1bd7, &mut slot.properties),
            (0x1bfb, &mut slot.behavior_c),
            (0x97dc, &mut slot.angle),
            (0xf380, &mut slot.drop.flags),
            (0xf3c8, &mut slot.drop.enemy_definition),
            (0xf410, &mut slot.drop.killed_enemy),
        ] {
            each(base + at, word);
        }
    }
}

/// The table's scalar words the projectile host and the file select menu
/// share, then every slot word.
fn table_words(projectiles: &mut EnemyProjectiles, mut each: impl FnMut(usize, &mut u16)) {
    each(0x198d, &mut projectiles.flags);
    each(0x1991, &mut projectiles.selected);
    each(0x1993, &mut projectiles.spawn.primary);
    each(0x1995, &mut projectiles.spawn.secondary);
    slot_words(&mut projectiles.slots, each);
}

/// Run `action` on the projectile slots.
pub fn with_slots<H, T>(game: &mut H, action: impl FnOnce(&mut [EnemyProjectile; SLOTS]) -> T) -> T
where
    H: WorkRamAccess + RuntimeAccess + ?Sized,
{
    with_projectiles(game, |projectiles| action(&mut projectiles.slots))
}

/// Run `action` on the projectile table. On the RAM copy only the words
/// `table_words` lists are decoded and stored, so `action` must keep to them.
pub fn with_projectiles<H, T>(game: &mut H, action: impl FnOnce(&mut EnemyProjectiles) -> T) -> T
where
    H: WorkRamAccess + RuntimeAccess + ?Sized,
{
    use crate::session_access::SessionAccess;
    if game.runtime_admissible() {
        return action(&mut game.runtime().session_mut().enemy_projectiles);
    }
    let ram = game.ram();
    let mut projectiles = EnemyProjectiles::default();
    table_words(&mut projectiles, |at, word| {
        *word = u16::from_le_bytes([ram.read_byte(at), ram.read_byte(at + 1)]);
    });
    let before = projectiles.clone();
    let result = action(&mut projectiles);
    if projectiles != before {
        table_words(&mut projectiles, |at, word| {
            let [low, high] = word.to_le_bytes();
            ram.write_byte(at, low);
            ram.write_byte(at + 1, high);
        });
    }
    result
}

/// The file select menu's reuse of the table (RAM map $198D..$19BC), by the
/// RAM map's names. Helmet `n` is slot A, B or C's.
pub struct FileSelect<'a>(pub &'a mut EnemyProjectiles);

impl FileSelect<'_> {
    /// $198D: `menu_selection_missile_animation_timer`.
    pub fn missile_animation_timer(&mut self) -> &mut u16 {
        &mut self.0.flags
    }
    /// $1991, $1993, $1995: `slot_[abc]_helmet_animation_timer`.
    pub fn helmet_animation_timer(&mut self, helmet: usize) -> &mut u16 {
        match helmet {
            0 => &mut self.0.selected,
            1 => &mut self.0.spawn.primary,
            _ => &mut self.0.spawn.secondary,
        }
    }
    /// $1997: `menu_selection_missile_animation_frame`.
    pub fn missile_animation_frame(&mut self) -> &mut u16 {
        &mut self.0.slots[0].definition
    }
    /// $199B..$199F: `slot_[abc]_helmet_animation_frame`.
    pub fn helmet_animation_frame(&mut self, helmet: usize) -> &mut u16 {
        &mut self.0.slots[2 + helmet].definition
    }
    /// $19A1: `menu_selection_missile_x_pos`.
    pub fn missile_x(&mut self) -> &mut u16 {
        &mut self.0.slots[5].definition
    }
    /// $19A5..$19A9: `slot_[abc]_helmet_x_pos`.
    pub fn helmet_x(&mut self, helmet: usize) -> &mut u16 {
        &mut self.0.slots[7 + helmet].definition
    }
    /// $19AB: `menu_selection_missile_y_pos`.
    pub fn missile_y(&mut self) -> &mut u16 {
        &mut self.0.slots[10].definition
    }
    /// $19AF..$19B3: `slot_[abc]_helmet_y_pos`.
    pub fn helmet_y(&mut self, helmet: usize) -> &mut u16 {
        &mut self.0.slots[12 + helmet].definition
    }
    /// $1999, $19A3, $19AD: the RAM map's unused words between the
    /// missile's and helmets' frames and positions.
    pub fn clear_unused_words(&mut self) {
        for slot in [1, 6, 11] {
            self.0.slots[slot].definition = 0;
        }
    }
    /// $19B5: `file_copy_selection`.
    pub fn file_copy_selection(&mut self) -> &mut u16 {
        &mut self.0.slots[15].definition
    }
    /// $19B7: `file_copy_source_slot`.
    pub fn file_copy_source_slot(&mut self) -> &mut u16 {
        &mut self.0.slots[16].definition
    }
    /// $19B9: `file_copy_destination_slot`.
    pub fn file_copy_destination_slot(&mut self) -> &mut u16 {
        &mut self.0.slots[17].definition
    }
}

/// Run `action` on the file select menu's view of the table.
pub fn with_file_select<H, T>(game: &mut H, action: impl FnOnce(&mut FileSelect<'_>) -> T) -> T
where
    H: WorkRamAccess + RuntimeAccess + ?Sized,
{
    with_projectiles(game, |projectiles| action(&mut FileSelect(projectiles)))
}

/// Clear the definition of the projectile at byte slot `slot`, freeing it.
/// A slot past the table keeps the original's store to the word it names.
pub fn free<H>(slot: u16, game: &mut H)
where
    H: WorkRamAccess + RuntimeAccess + ?Sized,
{
    let index = usize::from(slot >> 1);
    if slot & 1 == 0 && index < SLOTS {
        with_slots(game, |slots| slots[index].definition = 0);
    } else {
        game.ram()
            .copy_in(0x1997 + usize::from(slot), &0u16.to_ne_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_words_cover_the_encoded_table() {
        let mut slots = [EnemyProjectile::default(); SLOTS];
        let mut seen = Vec::new();
        slot_words(&mut slots, |at, _| seen.push(at));
        seen.sort_unstable();
        let mut expected: Vec<usize> = (0x1997..0x1c1f).step_by(2).collect();
        for base in [0x97dc, 0xf380, 0xf3c8, 0xf410] {
            expected.extend((0..SLOTS).map(|index| base + 2 * index));
        }
        expected.sort_unstable();
        assert_eq!(seen, expected);
    }
}
