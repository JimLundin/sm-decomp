//! The solid enemies Samus's moves meet.
//!
//! Each interactive enemy (`$17EC`) that is frozen or whose hitbox is solid to her, as the
//! samus-motion model's world gives them (`samus_world`); the model decides the contact.

use crate::state::room::{Enemy, EnemyBody, EnemyCommon, EnemyProperty};
use crate::state::samus::Point;
use crate::state::slots::{Frames, Slot, SlotIndex};

/// An interactive enemy as her moves meet it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Obstacle {
    /// Its centre; the subpixels play no part.
    pub position: Point,
    /// Half its width.
    pub radius_x: u16,
    /// Half its height.
    pub radius_y: u16,
    /// Its hitbox is solid to Samus.
    pub solid: bool,
    /// It is frozen, which makes any enemy solid.
    pub frozen: bool,
}

impl From<&EnemyCommon> for Obstacle {
    fn from(enemy: &EnemyCommon) -> Self {
        Self {
            position: enemy.position,
            radius_x: enemy.radius_x,
            radius_y: enemy.radius_y,
            solid: enemy.properties.contains(EnemyProperty::SolidToSamus),
            frozen: enemy.frozen != Frames(0),
        }
    }
}

/// The interactive `slots` (in list order) as obstacles, from the enemies' table.
pub fn obstacles<'a, B>(
    slots: &'a [SlotIndex<Enemy>],
    enemies: &'a [Slot<EnemyCommon, B>],
) -> impl Iterator<Item = (SlotIndex<Enemy>, Obstacle)> + 'a {
    slots.iter().filter_map(|&slot| {
        enemies
            .get(usize::from(slot.0))
            .map(|enemy| (slot, Obstacle::from(&enemy.common)))
    })
}

/// The enemies her moves can run into: the frame's interactive list (`$17EC`) and the enemies'
/// table it indexes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SolidEnemies<'a> {
    /// The interactive slots, in list order.
    pub interactive: &'a [SlotIndex<Enemy>],
    /// The enemies' table.
    pub slots: &'a [Slot<EnemyCommon, EnemyBody>],
}

#[cfg(test)]
mod tests;
