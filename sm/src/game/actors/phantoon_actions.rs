//! Phantoon's actions run in `sm_game::actors::phantoon_actions`.
#![forbid(unsafe_code)]

// The destroyable flames are the decompilation's projectile.
const _: () = assert!(
    sm_game::actors::phantoon_actions::DESTROYABLE_FLAMES
        == crate::game::constants::bank_86::ADDR_K_EPROJ_DESTROYABLE_FIREBALLS as u16
);
