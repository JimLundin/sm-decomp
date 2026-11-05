//! Item restores run on the lease's owned inventory, which gameplay cores share.
#![forbid(unsafe_code)]
use crate::game::{item_pickups::Resource, session_runtime::Runtime};

/// Restore `amount` of `resource` on `runtime`'s inventory. The verifier
/// observes the lease's projected words at entry and at return.
pub(crate) fn restore(resource: Resource, amount: u16, runtime: &mut Runtime<'_>) {
    #[cfg(sm_verify_dll)]
    let tag = match resource {
        Resource::Health => 0,
        Resource::Missiles => 1,
        Resource::SuperMissiles => 2,
        Resource::PowerBombs => 3,
    };
    #[cfg(sm_verify_dll)]
    crate::game::enemy_projectiles::checks::supplemental_cases::restore_cases::observe(
        tag, 0, amount, runtime,
    );
    runtime.session_mut().inventory.restore(resource, amount);
    #[cfg(sm_verify_dll)]
    crate::game::enemy_projectiles::checks::supplemental_cases::restore_cases::observe(
        tag, 1, amount, runtime,
    );
}
