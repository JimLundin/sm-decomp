//! Direct shared missile producer; inputs remain live in the one lane view.
use super::{Context, Transaction};
use crate::game::missile_firing::{self as shared, Control, Weapon};
use crate::game::projectile_weapon_definitions as definitions;
impl<'g> shared::Context<Transaction<'g>> for Context {
    type SoundKey = u16;
    type Kind = u16;
    fn shoot(&self, g: &Transaction<'g>) -> bool {
        word!(g.button_config_shoot_x) & word!(g.joypad1_newkeys) != 0
            || word!(g.button_config_shoot_x) & word!(g.joypad1_newinput_samusfilter) != 0
    }
    fn weapon(&self, g: &Transaction<'g>) -> Weapon {
        if word!(g.hud_item_index) == 2 {
            Weapon::SuperMissile
        } else {
            Weapon::Missile
        }
    }
    fn control(&self, g: &Transaction<'g>, field: Control) -> u16 {
        match field {
            Control::Count => word!(g.projectile_counter),
            Control::Cooldown => word!(g.cooldown_timer),
            Control::Protection => word!(g.projectile_invincibility_timer),
        }
    }
    fn set_control(&mut self, g: &mut Transaction<'g>, field: Control, value: u16) {
        match field {
            Control::Count => word!(g.projectile_counter = value),
            Control::Cooldown => word!(g.cooldown_timer = value),
            Control::Protection => word!(g.projectile_invincibility_timer = value),
        }
    }
    fn damage(&self, g: &Transaction<'g>, slot: u16) -> u16 {
        word!(g.projectile_damage[slot >> 1])
    }
    fn ammo(&self, g: &Transaction<'g>, weapon: Weapon) -> bool {
        match weapon {
            Weapon::Missile => word!(g.samus_missiles) != 0,
            Weapon::SuperMissile => word!(g.samus_super_missiles) != 0,
        }
    }
    fn place(&mut self, mut g: Transaction<'g>, slot: u16) -> (Transaction<'g>, bool) {
        super::placement::place(slot, g)
    }
    fn spend(&mut self, mut g: Transaction<'g>, weapon: Weapon) -> Transaction<'g> {
        match weapon {
            Weapon::Missile => {
                let value = word!(g.samus_missiles).wrapping_sub(1);
                word!(g.samus_missiles = value);
            }
            Weapon::SuperMissile => {
                let value = word!(g.samus_super_missiles).wrapping_sub(1);
                word!(g.samus_super_missiles = value);
            }
        };
        g
    }
    fn trail(&mut self, g: &mut Transaction<'g>, slot: u16, value: u16) {
        word!(g.projectile_timers[slot >> 1] = value);
    }
    fn arm(&mut self, g: &mut Transaction<'g>, slot: u16) -> u16 {
        let key = word!(g.hud_item_index);
        let kind = word!(g.hud_item_index).swap_bytes();
        let kind = word!(g.projectile_type[slot >> 1]) | kind | 0x8000;
        word!(g.projectile_type[slot >> 1] = kind);
        key
    }
    fn cinematic(&self, g: &Transaction<'g>) -> bool {
        word!(g.cinematic_function) != 0
    }
    fn sound(&mut self, mut g: Transaction<'g>, key: u16) -> Transaction<'g> {
        g.foreign(|g| {
            crate::game::sm_80::QueueSfx1_Max6(definitions::SOUNDS[usize::from(key & 15)], g)
        })
        .0
    }
    fn speed(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::motion::initialize_speed(slot, 0, g)
    }
    fn initialize(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::animation::initialize(
            slot,
            crate::game::projectile_initialization::Initialization::Projectile,
            g,
        )
    }
    fn kind(&self, g: &Transaction<'g>, slot: u16) -> u16 {
        word!(g.projectile_type[slot >> 1])
    }
    fn super_motion(&self, kind: u16) -> bool {
        kind & 0x200 != 0
    }
    fn motion(&mut self, g: &mut Transaction<'g>, slot: u16, weapon: Weapon) {
        word!(
            g.projectile_bomb_pre_instructions[slot >> 1] = match weapon {
                Weapon::Missile => 0xaf68,
                Weapon::SuperMissile => 0xafe5,
            }
        );
    }
    fn cooldown(&mut self, mut g: Transaction<'g>, kind: u16) -> (Transaction<'g>, u16) {
        // The retired code inspected byte1 of a native local u16, not RAM.
        (
            g,
            u16::from(definitions::COOLDOWNS[usize::from(kind.to_ne_bytes()[1] & 15)]),
        )
    }
    fn automatic(&self, g: &Transaction<'g>) -> bool {
        word!(g.samus_auto_cancel_hud_item_index) != 0
    }
    fn clear_selection(&mut self, g: &mut Transaction<'g>, automatic: bool) {
        word!(g.hud_item_index = 0);
        if automatic {
            word!(g.samus_auto_cancel_hud_item_index = 0);
        }
    }
}
pub(super) fn fire(g: Transaction<'_>) -> Transaction<'_> {
    shared::fire(g, &mut Context::default())
}
