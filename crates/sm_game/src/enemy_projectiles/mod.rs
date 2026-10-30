//! Enemy projectiles: owned state, behaviour and the host they run against.
// Translated modules keep the compat crate's lint policy for their original names.
pub mod access;
pub mod addresses;
pub mod attachments_fire_arc_botwoon;
pub mod attachments_spore_maw;
pub mod collision;
pub mod commands;
pub mod contact;
pub mod content;
pub mod coordinates;
pub mod drops;
pub mod host;
pub mod lifecycle;
pub mod pre_actions;
pub mod pre_helpers;
pub mod presentation;
pub mod setup;
pub mod setup_helpers;
pub mod shared;
pub mod shared_pre;
pub mod table;
pub mod tables;

// Names the projectile modules share from this scope, as in the compat module.
use crate::constants::{bank_86::*, bank_a0::*};
use collision::*;
use drops::collect as collect_drop;
use setup::*;
use shared::*;
use shared_pre::*;
use tables::*;
