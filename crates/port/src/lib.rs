//! Readable gameplay translated from pinned snesrev/sm.
//! Upstream MIT notices: ../decomp/LICENSE.upstream.txt.

mod memory;
pub use memory::{WRAM_SIZE, Wram};

pub mod animated_tiles;
pub mod audio;
pub mod camera_scroll;
pub mod collision;
pub mod enemy;
pub mod enemy_ai;
pub mod enemy_crawler;
pub mod enemy_instructions;
pub mod enemy_jump;
pub mod enemy_projectile_collision;
pub mod enemy_projectile_contacts;
pub mod enemy_projectile_instructions;
pub mod enemy_projectile_rendering;
pub mod enemy_projectile_sparks;
pub mod enemy_projectiles;
pub mod enemy_rio;
pub mod game_host;
pub mod glass_shattering;
pub mod mother_brain_bomb;
pub mod n00b_tube;
pub mod observed_projectile_ownership;
pub mod plm;
pub mod plm_instructions;
pub mod plm_render;
pub mod power_bomb;
pub mod projectile_flight;
pub mod projectile_special;
pub mod projectile_trails;
pub mod projectiles;
pub mod ridley_enemy_instructions;
pub mod ridley_subsystem;
pub mod room;
pub mod room_scroll;
pub mod samus;
pub mod samus_control;
pub mod samus_projectile_frame;
pub mod sprite_objects;
pub mod system;
