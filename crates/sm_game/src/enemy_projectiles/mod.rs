//! Enemy projectiles: owned state, behaviour and the host they run against.
// Translated modules keep the compat crate's lint policy for their original names.
pub mod access;
pub mod attachments_spore_maw;
pub mod attachments_waffle_botwoon;
pub mod collision;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod commands;
pub mod contact;
pub mod content;
pub mod coordinates;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod drops;
pub mod host;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod lifecycle;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod pre_actions;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod pre_helpers;
pub mod presentation;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod setup;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod setup_helpers;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod shared;
#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]
pub mod shared_pre;
pub mod tables;

// Names the projectile modules share from this scope, as in the compat module.
use crate::constants::{bank_86::*, bank_a0::*};
use collision::*;
use drops::collect as collect_drop;
use setup::*;
use shared::*;
use shared_pre::*;
use tables::*;
