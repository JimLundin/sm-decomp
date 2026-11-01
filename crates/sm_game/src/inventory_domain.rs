//! Inventory authority and reserve transitions, independent of compatibility memory.
#![forbid(unsafe_code)]

use crate::item_pickups::Inventory;

/// The same inventory is either resident in Session or in its domain slot.
/// Epochs belong to the adapter's selected image, not to persisted gameplay.
pub enum InventoryDomain {
    Unowned,
    Owned { inventory: Inventory, epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}

#[derive(Clone, Copy)]
pub struct ReserveState {
    pub mode: u16,
    pub capacity: u16,
    pub health: u16,
    pub maximum_health: u16,
    pub reserve: u16,
    pub beams: u16,
    pub equipment: u16,
    pub hyper_beam: u16,
}

impl From<&Inventory> for ReserveState {
    fn from(inventory: &Inventory) -> Self {
        Self {
            mode: inventory.reserve_mode,
            capacity: inventory.reserve_capacity,
            health: inventory.health.current,
            maximum_health: inventory.health.maximum,
            reserve: inventory.reserve_health,
            beams: inventory.beams.owned,
            equipment: inventory.equipment.owned,
            hyper_beam: inventory.hyper_beam,
        }
    }
}

/// Only changed words are committed, in the inventory cohort's original order.
#[derive(Clone, Copy, Default)]
pub struct ReserveChange {
    pub mode: Option<u16>,
    pub health: Option<u16>,
    pub reserve: Option<u16>,
}

impl ReserveChange {
    pub fn is_empty(self) -> bool {
        self.mode.is_none() && self.health.is_none() && self.reserve.is_none()
    }
    pub fn apply(self, inventory: &mut Inventory) {
        if let Some(mode) = self.mode {
            inventory.reserve_mode = mode;
        }
        if let Some(health) = self.health {
            inventory.health.current = health;
        }
        if let Some(reserve) = self.reserve {
            inventory.reserve_health = reserve;
        }
    }
}

impl ReserveState {
    pub fn select_mode(self, mode: u16) -> ReserveChange {
        ReserveChange {
            mode: (mode != self.mode).then_some(mode),
            ..ReserveChange::default()
        }
    }
    pub fn transfer_manual(self) -> (ReserveChange, bool) {
        let mut health = self.health.wrapping_add(1);
        let mut reserve = self.reserve;
        let more = if (health as i16).wrapping_sub(self.maximum_health as i16) >= 0 {
            health = self.maximum_health;
            false
        } else {
            reserve = reserve.wrapping_sub(1);
            if reserve as i16 > 0 {
                true
            } else {
                health = health.wrapping_add(reserve);
                false
            }
        };
        if !more {
            reserve = 0;
        }
        (
            ReserveChange {
                mode: None,
                health: (health != self.health).then_some(health),
                reserve: (reserve != self.reserve).then_some(reserve),
            },
            more,
        )
    }
}

/// Reads use the domain owner without unrelated gameplay admission. Changed
/// stores retain the caller's existing admission and publication seam.
pub trait InventoryAccess {
    fn reserve_state(&self) -> ReserveState;
    fn commit_reserve(&mut self, change: ReserveChange);
}
