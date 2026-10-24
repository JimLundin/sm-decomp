//! Item awards commit before their visual, music and message effects.
#![forbid(unsafe_code)]

#[derive(Clone, Copy)]
pub(crate) enum PickupKind {
    Beam,
    Equipment,
    Grapple,
    Xray,
    Energy,
    Reserve,
    Missiles,
    SuperMissiles,
    PowerBombs,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub(crate) struct Loadout {
    pub owned: u16,
    pub active: u16,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub(crate) struct Supply {
    pub current: u16,
    pub maximum: u16,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub(crate) struct Inventory {
    pub equipment: Loadout,
    pub beams: Loadout,
    pub health: Supply,
    pub health_fraction: u16,
    pub reserve_capacity: u16,
    pub reserve_mode: u16,
    pub reserve_health: u16,
    pub reserve_missiles: u16,
    pub ammunition: [Supply; 3],
}

pub(crate) enum VisualUpdate {
    None,
    Beams,
    Grapple,
    Xray,
    Missiles,
    SuperMissiles,
    PowerBombs,
}

pub(crate) struct PickupEffects {
    pub visual: VisualUpdate,
    /// None means read the script's message byte after visual and music effects.
    pub message: Option<u16>,
}

impl Inventory {
    pub(crate) fn collect(&mut self, kind: PickupKind, amount: u16) -> PickupEffects {
        use PickupKind::*;
        match kind {
            Beam => {
                self.beams.owned |= amount;
                self.beams.active |= amount;
                self.beams.active &= !((amount << 1) & 8);
                self.beams.active &= !((amount >> 1) & 4);
            }
            Equipment | Grapple | Xray => {
                self.equipment.active |= amount;
                self.equipment.owned |= amount;
            }
            Energy => {
                self.health.maximum = self.health.maximum.wrapping_add(amount);
                self.health.current = self.health.maximum;
            }
            Reserve => {
                self.reserve_capacity = self.reserve_capacity.wrapping_add(amount);
                if self.reserve_mode == 0 {
                    self.reserve_mode = 1;
                }
            }
            Missiles | SuperMissiles | PowerBombs => {
                let index = match kind {
                    Missiles => 0,
                    SuperMissiles => 1,
                    _ => 2,
                };
                let supply = &mut self.ammunition[index];
                supply.maximum = supply.maximum.wrapping_add(amount);
                supply.current = supply.current.wrapping_add(amount);
            }
        }
        let (visual, message) = match kind {
            Beam => (VisualUpdate::Beams, None),
            Equipment => (VisualUpdate::None, None),
            Grapple => (VisualUpdate::Grapple, Some(5)),
            Xray => (VisualUpdate::Xray, Some(6)),
            Energy => (VisualUpdate::None, Some(1)),
            Reserve => (VisualUpdate::None, Some(25)),
            Missiles => (VisualUpdate::Missiles, Some(2)),
            SuperMissiles => (VisualUpdate::SuperMissiles, Some(3)),
            PowerBombs => (VisualUpdate::PowerBombs, Some(4)),
        };
        PickupEffects { visual, message }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Health,
    Missiles,
    SuperMissiles,
    PowerBombs,
}

impl Inventory {
    /// Restore into the sole inventory, preserving signed SNES comparisons.
    pub(crate) fn restore(&mut self, resource: Resource, amount: u16) {
        match resource {
            Resource::Health => {
                let health = self.health.current.wrapping_add(amount);
                self.health.current = health;
                if (health.wrapping_sub(self.health.maximum) as i16) >= 0 {
                    let mut reserve = self
                        .reserve_health
                        .wrapping_add(health)
                        .wrapping_sub(self.health.maximum);
                    if (reserve.wrapping_sub(self.reserve_capacity) as i16) >= 0 {
                        reserve = self.reserve_capacity;
                    }
                    self.reserve_health = reserve;
                    if reserve != 0 && self.reserve_mode == 0 {
                        self.reserve_mode = 1;
                    }
                    self.health.current = self.health.maximum;
                }
            }
            Resource::Missiles => {
                let missiles = self.ammunition[0].current.wrapping_add(amount);
                self.ammunition[0].current = missiles;
                if (missiles.wrapping_sub(self.ammunition[0].maximum) as i16) >= 0 {
                    // This is the original signed/wrapping branch, not min(max,99).
                    let reserve_limit = if self.ammunition[0].maximum.wrapping_sub(99) & 0x8000 != 0
                    {
                        self.ammunition[0].maximum
                    } else {
                        99
                    };
                    self.reserve_missiles = self
                        .reserve_missiles
                        .wrapping_add(missiles.wrapping_sub(self.ammunition[0].maximum));
                    if (self.reserve_missiles.wrapping_sub(reserve_limit) as i16) >= 0 {
                        self.reserve_missiles = reserve_limit;
                    }
                    self.ammunition[0].current = self.ammunition[0].maximum;
                }
            }
            Resource::SuperMissiles | Resource::PowerBombs => {
                let index = if resource == Resource::SuperMissiles {
                    1
                } else {
                    2
                };
                let supply = &mut self.ammunition[index];
                let ammo = supply.current.wrapping_add(amount);
                supply.current = ammo;
                if (ammo.wrapping_sub(supply.maximum) as i16) >= 0 && ammo != supply.maximum {
                    supply.current = supply.maximum;
                }
            }
        }
    }
}
