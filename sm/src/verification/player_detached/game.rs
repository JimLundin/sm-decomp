#[path = "actors.rs"]
pub(crate) mod actors;
#[path = "../../game/launch_motion.rs"]
pub(crate) mod launch_motion;
#[path = "player_projectiles.rs"]
pub(crate) mod player_projectiles;
pub(crate) use crate::palette as palette_effects;
pub(crate) use sm_game::projectile_block_reactions;
pub(crate) use sm_game::projectile_geometry;
pub(crate) use sm_game::projectile_room;

pub(crate) use sm_game::external_inputs;
pub(crate) use sm_game::item_pickups;
pub(crate) use sm_game::session;

#[path = "room_entry.rs"]
pub(crate) mod room_entry;

#[path = "room.rs"]
pub(crate) mod room_mechanisms;

#[path = "../../game/constants.rs"]
pub(crate) mod constants;

pub(crate) use sm_game::combat_rules;
pub(crate) use sm_game::contact_rules;
pub(crate) use sm_game::drop_rules;
#[path = "enemy_projectiles.rs"]
pub(crate) mod enemy_projectiles;
pub(crate) use sm_game::enemy_script;
#[path = "scene.rs"]
pub(crate) mod scene;
pub(crate) use sm_game::session_access;

pub(crate) use sm_game::visual_state;

pub(crate) use sm_game::visual_host;

#[path = "../../game/samus_palette.rs"]
pub(crate) mod samus_palette;

pub(crate) use sm_game::liquid_surface;

#[path = "../../game/session_encoding.rs"]
pub(crate) mod session_encoding;

pub(crate) use sm_game::sprite_effect_state;

pub(crate) use sm_game::sprite_effect_host;

pub(crate) use sm_game::sprite_objects;

pub(crate) use sm_game::sprite_effect_controllers;


pub(crate) use sm_game::gameplay_flag_host;

#[path = "../../game/progress_flags.rs"]
pub(crate) mod progress_flags;

#[allow(
    unused_imports,
    reason = "Detached collision subset omits operations facade reexport consumers."
)]
#[path = "../../game/player_collision/mod.rs"]
pub(crate) mod player_collision;
#[path = "player_collision_host.rs"]
pub(crate) mod player_collision_host;
#[cfg(test)]
#[path = "../player_collision_owned_checks.rs"]
pub(crate) mod player_collision_owned_checks;
#[allow(
    unused_imports,
    reason = "Detached transition subset omits runtime facade reexport consumers."
)]
#[path = "../../game/player_transitions/mod.rs"]
pub(crate) mod player_transitions;
#[path = "player_transitions_host.rs"]
pub(crate) mod player_transitions_host;

#[allow(
    unused_imports,
    reason = "Detached movement subset omits runtime facade reexport consumers."
)]
#[path = "../../game/player_movement/mod.rs"]
pub(crate) mod player_movement;

#[path = "../../game/player_update/mod.rs"]
pub(crate) mod player_update;

pub(crate) use sm_game::samus_periodic_damage;

