mod actor_runtime;
mod effects_runtime;
mod graphics_runtime;
pub(crate) use sm_game::enemy_projectiles::host;
mod runtime;
use crate::game::session_runtime::Runtime;
use host::Effects;
use sm_game::enemy_script::EnemyScript;
// Enemy-projectile ownership and complete behavior behind one compatibility boundary.
// State is exclusive during each operation; legacy effects publish and reacquire it.
use sm_game::enemy_projectiles::attachments_spore_maw;
mod attachments_spore_maw_facade;
use sm_game::enemy_projectiles::attachments_fire_arc_botwoon;
mod attachments_waffle_botwoon_facade;
use sm_game::enemy_projectiles::collision;
use sm_game::enemy_projectiles::commands;
use sm_game::enemy_projectiles::contact;
use sm_game::enemy_projectiles::content;
use sm_game::enemy_projectiles::drops;
mod legacy;
use crate::game_state::Game;
pub(crate) use attachments_spore_maw_facade::*;
pub(crate) use attachments_waffle_botwoon_facade::{
    advance_botwoon_history, advance_fire_arc_orbit, follow_botwoon_history, orient_botwoon_body,
    record_botwoon_head,
};
pub use drops::DropSource;
pub use presentation::Priority as EprojPriority;
pub(in crate::game) use sm_game::enemy_projectiles::lifecycle;
use sm_game::enemy_projectiles::pre_actions;
pub(in crate::game) use sm_game::enemy_projectiles::presentation;
use sm_game::enemy_projectiles::setup;
use sm_game::enemy_projectiles::shared;
use sm_game::enemy_projectiles::shared_pre;

fn with_execution<R>(game: &mut Game, operation: impl FnOnce(&mut Runtime<'_>) -> R) -> R {
    operation(&mut Runtime::new(game))
}
pub fn enable_eprojs(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::set_enabled(ctx, true));
}
pub fn disable_eprojs(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::set_enabled(ctx, false));
}
pub fn clear_eprojs(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::clear(ctx));
}
pub fn spawn_with_enemy_gfx(parameter: u16, enemy: u16, definition: u16, game: &mut Game) -> u16 {
    with_execution(game, |ctx| {
        lifecycle::spawn_from_enemy(ctx, parameter, enemy, definition)
    })
}
pub(crate) fn spawn_scripted(definition: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let parameter = script.word(0);
    let index = game.cur_enemy_index();
    spawn_with_enemy_gfx(parameter, index, definition, game);
    script.offset(1)
}
pub fn spawn_with_room_gfx(definition: u16, parameter: u16, game: &mut Game) -> u16 {
    with_execution(game, |ctx| {
        lifecycle::spawn_room(ctx, definition, parameter)
    })
}
pub fn eproj_run_all(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::advance(ctx));
}
pub fn x_velocity_at_angle(amplitude: u16, angle: u16) -> u16 {
    shared_pre::x_velocity_at_angle(amplitude, angle)
}
pub fn y_velocity_at_angle(amplitude: u16, angle: u16) -> u16 {
    shared_pre::y_velocity_at_angle(amplitude, angle)
}
pub fn enemy_projectiles_hit_samus(game: &mut Game) {
    with_execution(game, |ctx| contact::enemy_projectiles_hit_samus(ctx));
}
pub fn shoot_enemy_projectiles(game: &mut Game) {
    with_execution(game, |ctx| contact::shoot_enemy_projectiles(ctx));
}
pub fn spawn_drop(definition: u16, index: u16, game: &mut Game) {
    with_execution(game, |ctx| drops::spawn(definition, index, ctx));
}
pub fn scatter_drop(source: DropSource, index: u16, game: &mut Game) {
    with_execution(game, |ctx| drops::scatter(source, index, ctx));
}
pub fn reset_room_metadata(game: &mut Game) {
    with_execution(game, |ctx| drops::reset_room_metadata(ctx));
}

#[path = "../../verification/enemy_projectiles_candidate.rs"]
pub(crate) mod checks;

pub fn reset_room_enemy_workspace(game: &mut Game) {
    with_execution(game, |ctx| ctx.reset_room_enemy_workspace());
}

#[path = "../../verification/native_content_checks.rs"]
pub(crate) mod native_content_checks;
#[path = "../../verification/reset_metadata_checks.rs"]
pub(crate) mod reset_metadata_checks;
