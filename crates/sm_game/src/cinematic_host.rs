//! What the bank $8B cinematic functions need beyond the menu host and the
//! display refresh: palette effects, delayed music, Mode 7 transfers and the
//! session operations the compatibility runtime still implements.

pub trait Host: crate::menus::host::Host + crate::display_refresh::Host {
    fn clear_palette_effects(&mut self);
    fn set_palette_effects_enabled(&mut self, enabled: bool);
    /// Queue music `track` after `delay` frames.
    fn queue_music_delayed(&mut self, track: u16, delay: u16);
    fn spawn_power_bomb(&mut self);
    /// Set up Samus for the intro.
    fn initialize_player(&mut self);
    fn disable_scanline_irq(&mut self);
    /// Queue the ending's next Mode 7 tile upload.
    fn advance_ending_tile_upload(&mut self);
    fn advance_palette_effects(&mut self);
    /// Prepare Samus's projectiles and run her frame.
    fn advance_intro_player(&mut self);
    /// The original's failed native assertion `expression` in `function`.
    fn native_assertion(
        &mut self,
        expression: &'static [u8],
        line: u32,
        function: &'static [u8],
    ) -> !;
}
