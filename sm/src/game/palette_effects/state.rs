//! Ordinary palette state: no cartridge addresses, RAM projection or backend.
//! A Session can live across ticks in the independent host. The compatibility
//! host consumes it whenever opaque legacy code takes authority.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Slot {
    pub id: u16,
    pub color: u16,
    pub pre_action: u16,
    pub program: u16,
    pub delay: u16,
    pub counter: u16,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Field {
    Id,
    Color,
    PreAction,
    Program,
    Delay,
    Counter,
}
impl Slot {
    pub(crate) fn field(&self, field: Field) -> u16 {
        match field {
            Field::Id => self.id,
            Field::Color => self.color,
            Field::PreAction => self.pre_action,
            Field::Program => self.program,
            Field::Delay => self.delay,
            Field::Counter => self.counter,
        }
    }
    pub(crate) fn field_mut(&mut self, field: Field) -> &mut u16 {
        match field {
            Field::Id => &mut self.id,
            Field::Color => &mut self.color,
            Field::PreAction => &mut self.pre_action,
            Field::Program => &mut self.program,
            Field::Delay => &mut self.delay,
            Field::Counter => &mut self.counter,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PaletteEffects {
    pub slots: [Slot; 8],
    pub flags: u16,
    pub selected: u16,
    pub heat: u16,
    pub previous_heat: u16,
}
/// Inputs stay fixed until an explicit host effect returns a new observation.
/// They are sampled values, not a second writable copy of player/room state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Inputs {
    pub cinematic: u16,
    pub enemy_health: u16,
    pub equipped_items: u16,
    pub frame: u16,
    pub health: u16,
    pub player_y: u16,
    pub miniboss_dead: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PeriodicDamage {
    pub fraction: u16,
    pub amount: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Session {
    pub effects: PaletteEffects,
    pub colors: [u16; 256],
    pub inputs: Inputs,
    pub damage: PeriodicDamage,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            effects: PaletteEffects::default(),
            colors: [0; 256],
            inputs: Inputs::default(),
            damage: PeriodicDamage::default(),
        }
    }
}
impl Session {
    /// Reset the independent session. Runtime hardware resets import their actual bytes.
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
    pub(crate) fn snapshot(&self) -> Self {
        self.clone()
    }
    pub(crate) fn restore(&mut self, snapshot: Self) {
        *self = snapshot;
    }
    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.effects.flags = if enabled {
            self.effects.flags | 0x8000
        } else {
            self.effects.flags & !0x8000
        };
    }
    pub(crate) fn clear(&mut self) {
        for slot in self.effects.slots.iter_mut().rev() {
            slot.id = 0;
        }
    }
}
