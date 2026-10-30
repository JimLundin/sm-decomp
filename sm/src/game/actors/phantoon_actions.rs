//! Phantoon's actions run in `sm_game::actors::phantoon_actions`.
#![forbid(unsafe_code)]
pub(crate) use sm_game::actors::phantoon_actions::spawn_casual_flame;

// The destroyable flames are the decompilation's projectile.
const _: () = assert!(
    sm_game::actors::phantoon_actions::DESTROYABLE_FLAMES
        == crate::game::constants::bank_86::addr_kEproj_DestroyableFireballs as u16
);
