mod access;
mod actor_runtime;
pub(crate) mod coordinates;
mod effects_runtime;
mod graphics_runtime;
pub(crate) mod host;
mod runtime;
use crate::game::enemy_script::EnemyScript;
use crate::game::session_runtime::Runtime;
use host::Effects;
// Enemy-projectile ownership and complete behavior behind one compatibility boundary.
// State is exclusive during each operation; legacy effects publish and reacquire it.
mod attachments_spore_maw;
mod attachments_spore_maw_facade;
mod attachments_waffle_botwoon;
mod attachments_waffle_botwoon_facade;
mod collision;
mod commands;
mod contact;
mod content;
mod drops;
pub(crate) use drops::reset_room_metadata as reset_room_metadata_owned;
mod legacy;
pub(in crate::game) mod lifecycle;
mod pre_actions;
mod pre_helpers;
pub(in crate::game) mod presentation;
mod script;
mod setup;
mod setup_helpers;
mod shared;
mod shared_pre;
mod tables;
use crate::game::constants::{bank_86::*, bank_a0::*};
use crate::game_state::Game;
pub(crate) use attachments_spore_maw_facade::*;
pub(crate) use attachments_waffle_botwoon_facade::{
    advance_botwoon_history, advance_waffle_orbit, follow_botwoon_history, orient_botwoon_body,
    record_botwoon_head,
};
use collision::*;
pub use drops::DropSource;
use drops::collect as collect_drop;
pub use presentation::Priority as EprojPriority;
use setup::*;
use shared::*;
use shared_pre::*;
use tables::*;

const fnEprojPreInstr_nullsub_83: u32 = 8815867;
const fnEproj_Init_0x8bde_SkreeDownLeft: u32 = 8817429;
const fnEprojPreInstr_Empty: i32 = 0x86efdf;

fn with_execution<R>(game: &mut Game, operation: impl FnOnce(&mut Runtime<'_>) -> R) -> R {
    operation(&mut Runtime::new(game))
}
pub fn EnableEprojs(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::set_enabled(ctx, true));
}
pub fn DisableEprojs(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::set_enabled(ctx, false));
}
pub fn ClearEprojs(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::clear(ctx));
}
pub fn SpawnEprojWithGfx(parameter: u16, enemy: u16, definition: u16, game: &mut Game) -> u16 {
    with_execution(game, |ctx| {
        lifecycle::spawn_from_enemy(ctx, parameter, enemy, definition)
    })
}
pub(crate) fn spawn_scripted(definition: u16, script: EnemyScript, game: &mut Game) -> EnemyScript {
    let parameter = script.word(0);
    let index = game.cur_enemy_index();
    SpawnEprojWithGfx(parameter, index, definition, game);
    script.offset(1)
}
pub fn SpawnEprojWithRoomGfx(definition: u16, parameter: u16, game: &mut Game) -> u16 {
    with_execution(game, |ctx| {
        lifecycle::spawn_room(ctx, definition, parameter)
    })
}
pub fn EprojRunAll(game: &mut Game) {
    with_execution(game, |ctx| lifecycle::advance(ctx));
}
pub fn DrawEprojsWithPriority(priority: EprojPriority, game: &mut Game) {
    with_execution(game, |ctx| presentation::draw(priority, ctx));
}
pub fn Math_MultBySin(amplitude: u16, angle: u16) -> u16 {
    shared_pre::Math_MultBySin(amplitude, angle)
}
pub fn Math_MultByCos(amplitude: u16, angle: u16) -> u16 {
    shared_pre::Math_MultByCos(amplitude, angle)
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
