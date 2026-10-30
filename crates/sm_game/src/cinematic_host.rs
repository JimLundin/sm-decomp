//! What the bank $8B cinematic functions need beyond the menu host and the
//! display refresh: palette effects, delayed music, Mode 7 transfers and the
//! session operations the compatibility runtime still implements.
#![forbid(unsafe_code)]

pub trait Host: crate::menus::host::Host + crate::display_refresh::Host {
    fn clear_palette_effects(&mut self);
    fn set_palette_effects_enabled(&mut self, enabled: bool);
    /// Queue music `track` after `delay` frames.
    fn queue_music_delayed(&mut self, track: u16, delay: u16);
    /// Queue the bank $8B Mode 7 transfer list at `cursor`.
    fn enqueue_mode7_transfers(&mut self, cursor: u16);
    fn spawn_power_bomb(&mut self);
    /// Set up Samus for the intro.
    fn initialize_player(&mut self);
    fn use_cinematic_bindings(&mut self);
    fn disable_scanline_irq(&mut self);
    /// Whether game event `event` has happened.
    fn event_happened(&mut self, event: u16) -> bool;
}
