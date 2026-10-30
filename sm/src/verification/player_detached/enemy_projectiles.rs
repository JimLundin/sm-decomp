//! Actual candidate enemy algorithms; wrappers expose existing core entries.
pub(crate) use sm_game::enemy_projectiles::commands;
pub(crate) use sm_game::enemy_projectiles::contact;
pub(crate) use sm_game::enemy_projectiles::content;
pub(crate) use sm_game::enemy_projectiles::drops;
pub(crate) use sm_game::enemy_projectiles::host;
pub(crate) use sm_game::enemy_projectiles::lifecycle;
pub(crate) use sm_game::enemy_projectiles::pre_actions;
use sm_game::enemy_script::EnemyScript;
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


// Match the production parent's pure imports required by maintained sibling cores.

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
