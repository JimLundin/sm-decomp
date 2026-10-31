//! Access to the owned-Session view of a compatibility host.
//!
//! Code that drives several sm_game cores from one host asks for its
//! runtime view: a short-lived value that admits the owned Session for the
//! call and publishes it back when dropped, exactly as constructing the
//! compatibility Runtime does.

/// The host traits the owned-Session view provides.
pub trait SessionView:
    crate::session_access::SessionAccess
    + SessionRam
    + crate::hud::HudFrameHost
    + crate::player_position::PlayerPosition
    + crate::player_projectiles::host::Host
    + crate::room_mechanisms::host::Host
    + crate::room_mechanisms::host::Effects
    + crate::projectile_block_reactions::Definitions
    + crate::player_transitions::effects::Host
    + crate::player_collision::effects::Host
    + crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::visual_host::Host
    + crate::room_entry::effects::EntryHost
    + crate::player_movement::effects::Host
    + crate::progress::FlagHost
    + crate::room_scrolls::Host
{
}
impl<T> SessionView for T where
    T: crate::session_access::SessionAccess
        + SessionRam
        + crate::hud::HudFrameHost
        + crate::player_position::PlayerPosition
        + crate::player_projectiles::host::Host
        + crate::room_mechanisms::host::Host
        + crate::room_mechanisms::host::Effects
        + crate::projectile_block_reactions::Definitions
        + crate::player_transitions::effects::Host
        + crate::player_collision::effects::Host
        + crate::actors::effects::ActorHost
        + crate::enemy_projectiles::host::Host
        + crate::visual_host::Host
        + crate::room_entry::effects::EntryHost
        + crate::player_movement::effects::Host
        + crate::progress::FlagHost
        + crate::room_scrolls::Host
{
}

/// Work RAM as the owned-Session view projects it: owned fields where they
/// are current, the RAM bytes otherwise. Writes keep the view's ordering.
pub trait SessionRam {
    fn byte(&self, at: usize) -> u8;
    fn set_byte(&mut self, at: usize, value: u8);
    /// Store `bytes` from `at` in order after checking the whole extent.
    fn write_slice(&mut self, at: usize, bytes: &[u8]);
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn native_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn write_word_le(&mut self, value: u16, at: usize) {
        let [low, high] = value.to_le_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
    /// Store `value` in native byte order, a byte at a time.
    fn write_word_native(&mut self, value: u16, at: usize) {
        let [first, second] = value.to_ne_bytes();
        self.set_byte(at, first);
        self.set_byte(at + 1, second);
    }
    fn write_slice_le(&mut self, at: usize, value: u16) {
        self.write_slice(at, &value.to_le_bytes());
    }
    fn write_slice_native(&mut self, at: usize, value: u16) {
        self.write_slice(at, &value.to_ne_bytes());
    }
}

/// The owned-Session view Samus's drawing, projectiles and pose rules run on.
pub trait PlayerLease:
    SessionView
    + crate::player_drawing::Host
    + crate::player_projectiles::host::Host
    + crate::player_transitions::effects::Host
    + crate::player_transitions::effects::SamusLease
{
}
impl<
    T: SessionView
        + crate::player_drawing::Host
        + crate::player_projectiles::host::Host
        + crate::player_transitions::effects::Host
        + crate::player_transitions::effects::SamusLease
        + ?Sized,
> PlayerLease for T
{
}
pub trait RuntimeAccess {
    type Runtime<'a>: PlayerLease
        + crate::hdma_effects::HdmaEffects
        + crate::actors::mother_brain_rainbow::Host
    where
        Self: 'a;
    /// A fresh owned-Session view; publication happens when it is dropped.
    fn runtime(&mut self) -> Self::Runtime<'_>;
    /// Whether `runtime` can open now. It cannot while a retained persistent
    /// Session is out of date with RAM (opening would poison it) or while the
    /// Samus runtime rejects import; callers then use the RAM bridge.
    fn runtime_admissible(&self) -> bool {
        true
    }
    /// Whether a retained persistent Session is current, so that stores made
    /// on `runtime` keep it current. Without one a lease decodes the Session
    /// from RAM, and code holding a typed copy stores to RAM instead.
    fn session_retained(&self) -> bool {
        false
    }
}
