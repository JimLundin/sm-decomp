#[path = "actors.rs"]
pub(crate) mod actors;
#[path = "player_projectiles.rs"]
pub(crate) mod player_projectiles;

pub(crate) use sm_game::external_inputs;
pub(crate) use sm_game::session;

#[path = "room_entry.rs"]
pub(crate) mod room_entry;

#[path = "room.rs"]
pub(crate) mod room_mechanisms;

#[path = "enemy_projectiles.rs"]
pub(crate) mod enemy_projectiles;
#[path = "scene.rs"]
pub(crate) mod scene;
pub(crate) use sm_game::session_access;

pub(crate) use sm_game::visual_state;

pub(crate) use sm_game::visual_host;

pub(crate) use sm_game::samus_palette;

pub(crate) use sm_game::session_encoding;

pub(crate) use sm_game::sprite_effect_state;

pub(crate) use sm_game::sprite_effect_host;

pub(crate) use sm_game::sprite_objects;

pub(crate) use sm_game::sprite_effect_controllers;

pub(crate) use sm_game::gameplay_flag_host;

pub(crate) use sm_game::progress as progress_flags;

#[allow(
    unused_imports,
    reason = "Detached collision subset omits operations facade reexport consumers."
)]
pub(crate) use sm_game::player_collision;
#[path = "player_collision_host.rs"]
pub(crate) mod player_collision_host;
#[cfg(test)]
#[path = "../player_collision_owned_checks.rs"]
pub(crate) mod player_collision_owned_checks;
pub(crate) use sm_game::player_transitions;
#[path = "player_transitions_host.rs"]
pub(crate) mod player_transitions_host;
