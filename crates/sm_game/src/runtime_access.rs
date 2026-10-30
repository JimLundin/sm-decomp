//! Access to the owned-Session view of a compatibility host.
//!
//! Code that drives several sm_game cores from one host asks for its
//! runtime view: a short-lived value that admits the owned Session for the
//! call and publishes it back when dropped, exactly as constructing the
//! compatibility Runtime does.

/// The host traits the owned-Session view provides.
pub trait SessionView:
    crate::session_access::SessionAccess
    + crate::player_projectiles::host::Host
    + crate::room_mechanisms::host::Host
    + crate::room_mechanisms::host::Effects
    + crate::projectile_block_reactions::Definitions
    + crate::player_transitions::effects::Host
    + crate::player_collision::effects::Host
    + crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::visual_host::Host
{
}
impl<T> SessionView for T where
    T: crate::session_access::SessionAccess
        + crate::player_projectiles::host::Host
        + crate::room_mechanisms::host::Host
        + crate::room_mechanisms::host::Effects
        + crate::projectile_block_reactions::Definitions
        + crate::player_transitions::effects::Host
        + crate::player_collision::effects::Host
        + crate::actors::effects::ActorHost
        + crate::enemy_projectiles::host::Host
        + crate::visual_host::Host
{
}

pub trait RuntimeAccess {
    type Runtime<'a>: SessionView
    where
        Self: 'a;
    /// A fresh owned-Session view; publication happens when it is dropped.
    fn runtime(&mut self) -> Self::Runtime<'_>;
}
