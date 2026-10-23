//! Direct morph-bomb producer over owned projectiles and live external inputs.
use super::{Context, Transaction};
use crate::game::bomb_creation::{self as shared, Control, Field, Input, Kind};
use crate::game::projectile_weapon_definitions as definitions;
impl<'g> shared::Context<Transaction<'g>> for Context {
    type CooldownKey = u8;
    fn input(&self, g: &Transaction<'g>, input: Input) -> u16 {
        match input {
            Input::Held => word!(g.joypad1_lastkeys),
            Input::Pressed => word!(g.joypad1_newkeys),
            Input::FireButton => word!(g.button_config_shoot_x),
            Input::EquippedItems => word!(g.equipped_items),
            Input::PowerAmmo => word!(g.samus_power_bombs),
            Input::PowerFlag => word!(g.power_bomb_flag),
            Input::SamusX => word!(g.samus_x_pos),
            Input::SamusY => word!(g.samus_y_pos),
        }
    }
    fn control(&self, g: &Transaction<'g>, field: Control) -> u16 {
        match field {
            Control::Charge => word!(g.flare_counter),
            Control::BombCount => word!(g.bomb_counter),
            Control::Cooldown => word!(g.cooldown_timer),
            Control::SpreadCharge => word!(g.bomb_spread_charge_timeout_counter),
            Control::Hud => word!(g.hud_item_index),
            Control::AutoHud => word!(g.samus_auto_cancel_hud_item_index),
        }
    }
    fn set_control(&mut self, g: &mut Transaction<'g>, field: Control, value: u16) {
        match field {
            Control::Charge => word!(g.flare_counter = value),
            Control::BombCount => word!(g.bomb_counter = value),
            Control::Cooldown => word!(g.cooldown_timer = value),
            Control::SpreadCharge => word!(g.bomb_spread_charge_timeout_counter = value),
            Control::Hud => word!(g.hud_item_index = value),
            Control::AutoHud => word!(g.samus_auto_cancel_hud_item_index = value),
        }
    }
    fn write(&mut self, g: &mut Transaction<'g>, slot: u16, field: Field, value: u16) {
        match field {
            Field::Direction => word!(g.projectile_dir[slot >> 1] = value),
            Field::X => word!(g.projectile_x_pos[slot >> 1] = value),
            Field::Y => word!(g.projectile_y_pos[slot >> 1] = value),
            Field::XFraction => word!(g.projectile_bomb_x_subpos[slot >> 1] = value),
            Field::YFraction => word!(g.projectile_bomb_y_subpos[slot >> 1] = value),
            Field::XSpeed => word!(g.projectile_bomb_x_speed[slot >> 1] = value),
            Field::SpeedFraction => word!(g.projectile_timers[slot >> 1] = value),
            Field::YSpeed => word!(g.projectile_bomb_y_speed[slot >> 1] = value),
            Field::Auxiliary => word!(g.projectile_unk_A[slot >> 1] = value),
            Field::Lifetime => word!(g.projectile_variables[slot >> 1] = value),
        }
    }
    fn vacant(&self, g: &Transaction<'g>, slot: u16) -> bool {
        word!(g.projectile_type[slot >> 1]) == 0
    }
    fn is_spread(&self, g: &Transaction<'g>, slot: u16) -> bool {
        word!(g.projectile_bomb_pre_instructions[slot >> 1]) == 0xd8f7
    }
    fn ordinary_kind(&mut self, g: &mut Transaction<'g>, slot: u16) {
        word!(g.projectile_type[slot >> 1] = 0x500);
    }
    fn spread_kind(&mut self, g: &mut Transaction<'g>, slot: u16) {
        word!(g.projectile_type[slot >> 1] = 0x8500);
    }
    fn power_kind(&mut self, g: &mut Transaction<'g>, slot: u16) -> u8 {
        let hud = word!(g.hud_item_index).swap_bytes();
        let kind = hud | word!(g.projectile_type[slot >> 1]);
        word!(g.projectile_type[slot >> 1] = kind);
        // The retired producer captured byte1 of a native local word.
        kind.to_ne_bytes()[1] & 15
    }
    fn motion(&mut self, g: &mut Transaction<'g>, slot: u16, kind: Kind) {
        word!(
            g.projectile_bomb_pre_instructions[slot >> 1] = match kind {
                Kind::Bomb => 0xb099,
                Kind::PowerBomb => 0xb0ae,
                Kind::Spread => 0xd8f7,
            }
        );
    }
    fn ordinary_cooldown(&self) -> u8 {
        5
    }
    fn spread_cooldown(&self, g: &Transaction<'g>) -> u8 {
        // This is a LIVE physical byte after all five initializers.
        g.byte(0xc23) & 15
    }
    fn lookup_cooldown(&mut self, mut g: Transaction<'g>, key: u8) -> (Transaction<'g>, u16) {
        (g, u16::from(definitions::COOLDOWNS[usize::from(key)]))
    }
    fn clear_flare_animation(&mut self, g: &mut Transaction<'g>) {
        super::weapons::clear_animation(g);
    }
    fn spend_power(&mut self, mut g: Transaction<'g>) -> (Transaction<'g>, bool) {
        let ammo = word!(g.samus_power_bombs).wrapping_sub(1);
        word!(g.samus_power_bombs = ammo);
        let spent = word!(g.samus_power_bombs) & 0x8000 == 0;
        if spent {
            word!(g.power_bomb_flag = 0xffff);
        }
        (g, spent)
    }
    fn initialize(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::animation::initialize(
            slot,
            crate::game::projectile_initialization::Initialization::Bomb,
            g,
        )
    }
    fn cancel_sound(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| crate::game::sm_80::QueueSfx1_Max9(2, g)).0
    }
    fn palette(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| unsafe { crate::game::sm_91::Samus_LoadSuitPalette(g) })
            .0
    }
}
pub(super) fn fire(g: Transaction<'_>) -> Transaction<'_> {
    shared::fire(g, &mut Context::default())
}
