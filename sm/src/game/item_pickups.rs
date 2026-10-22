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

pub(crate) struct Loadout {
    pub owned: u16,
    pub active: u16,
}

pub(crate) struct Supply {
    pub current: u16,
    pub maximum: u16,
}

pub(crate) struct Inventory {
    pub equipment: Loadout,
    pub beams: Loadout,
    pub health: Supply,
    pub reserve_capacity: u16,
    pub reserve_mode: u16,
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

/// Room-object scripts resume after the committed instruction. The displayed
/// message and script slot remain live inputs, not part of this continuation.
pub(crate) enum MessageHandoff {
    Instructions,
    Displaying,
}

impl MessageHandoff {
    pub(crate) fn resume(saved: u8) -> Option<Self> {
        match saved {
            0 => Some(Self::Instructions),
            1 => Some(Self::Displaying),
            _ => None,
        }
    }
    pub(crate) fn displaying(&self) -> bool {
        matches!(self, Self::Displaying)
    }
    pub(crate) fn continuation(&self) -> u8 {
        u8::from(self.displaying())
    }
    pub(crate) fn after_instruction(&mut self, queued_message: u16) -> bool {
        if queued_message == 0 {
            return false;
        }
        *self = Self::Displaying;
        true
    }
    pub(crate) fn after_display(&mut self, yielded: u8) -> Result<(), u8> {
        if yielded != 0 {
            return Err(yielded);
        }
        *self = Self::Instructions;
        Ok(())
    }
}
