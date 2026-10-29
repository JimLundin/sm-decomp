//! Actual candidate enemy algorithms; wrappers expose existing core entries.
#[path = "../../game/enemy_projectiles/access.rs"]
pub(crate) mod access;
#[path = "../../game/enemy_projectiles/attachments_spore_maw.rs"]
pub(crate) mod attachments_spore_maw;
#[path = "../../game/enemy_projectiles/attachments_waffle_botwoon.rs"]
pub(crate) mod attachments_waffle_botwoon;
#[path = "../../game/enemy_projectiles/collision.rs"]
pub(crate) mod collision;
#[path = "../../game/enemy_projectiles/commands.rs"]
pub(crate) mod commands;
#[path = "../../game/enemy_projectiles/contact.rs"]
pub(crate) mod contact;
#[path = "../../game/enemy_projectiles/content.rs"]
pub(crate) mod content;
#[path = "../../game/enemy_projectiles/drops.rs"]
pub(crate) mod drops;
#[path = "../../game/enemy_projectiles/host.rs"]
pub(crate) mod host;
#[path = "../../game/enemy_projectiles/lifecycle.rs"]
pub(crate) mod lifecycle;
#[path = "../../game/enemy_projectiles/pre_actions.rs"]
pub(crate) mod pre_actions;
#[path = "../../game/enemy_projectiles/pre_helpers.rs"]
pub(crate) mod pre_helpers;
#[path = "../../game/enemy_projectiles/presentation.rs"]
pub(crate) mod presentation;
#[path = "../../game/enemy_projectiles/setup.rs"]
pub(crate) mod setup;
#[path = "../../game/enemy_projectiles/setup_helpers.rs"]
pub(crate) mod setup_helpers;
#[path = "../../game/enemy_projectiles/shared.rs"]
pub(crate) mod shared;
#[path = "../../game/enemy_projectiles/shared_pre.rs"]
pub(crate) mod shared_pre;
#[path = "../../game/enemy_projectiles/tables.rs"]
pub(crate) mod tables;
use crate::game::enemy_script::EnemyScript;
use host::Host;
pub(crate) fn spawn_room(h: &mut impl Host, definition: u16, parameter: u16) -> u16 {
    lifecycle::spawn_room(h, definition, parameter)
}

pub(crate) fn receive_shots(h: &mut impl Host) {
    contact::shoot_enemy_projectiles(h);
}
pub(crate) fn hit_player(h: &mut impl Host) {
    contact::enemy_projectiles_hit_samus(h);
}
pub(crate) fn collect(h: &mut impl Host, raw: u16) {
    drops::collect(raw, h);
}
pub(crate) fn spawn_drop(h: &mut impl Host, definition: u16, enemy: u16, parameter: u16) {
    h.session_mut().compatibility_enemy_scratch = parameter;
    drops::spawn(definition, enemy, h);
}

#[path = "../../game/enemy_projectiles/coordinates.rs"]
pub(crate) mod coordinates;

// Match the production parent's pure imports required by maintained sibling cores.
use crate::game::constants::{bank_86::*, bank_a0::*};
use collision::*;
use drops::collect as collect_drop;
use setup::*;
use shared::*;
use shared_pre::*;
use tables::*;

// Maintained numeric dispatch identities from the production parent module.
const fnEprojPreInstr_nullsub_83: u32 = 8815867;
const fnEproj_Init_0x8bde_SkreeDownLeft: u32 = 8817429;
const fnEprojPreInstr_Empty: i32 = 0x86efdf;

// Expose the maintained command dispatcher, without reimplementing its spawn rule.
pub(crate) fn sprite_command(h: &mut impl Host, k: u16, address: u16) {
    let script = EnemyScript::at(h.script_image(), 0x28000 + usize::from(address));
    assert!(!matches!(
        commands::dispatch(8842467, k, &script, h),
        EnemyScript::Stop
    ));
}

// Expose actual maintained flag-dependent entrypoints; no behavior is duplicated.
pub(crate) fn flag_command(h: &mut impl Host, identity: u32, k: u16, address: u16) -> usize {
    let image = h.script_image();
    let script = EnemyScript::at(image.clone(), 0x28000 + usize::from(address));
    let next = commands::dispatch(identity, k, &script, h)
        .resolve(&image, 0x28000)
        .expect("flag command continues");
    let EnemyScript::At { position, .. } = next else {
        unreachable!()
    };
    position
}
pub(crate) fn eye_door(h: &mut impl Host, k: u16) {
    pre_actions::EprojPreInstr_EyeDoorProjectile(k, h);
}
