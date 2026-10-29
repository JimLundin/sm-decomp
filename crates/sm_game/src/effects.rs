//! Effects game modules request from the runtime: queued sound effects and
//! the screen brightness the next display refresh uses.

/// The three sound-effect queues.
pub trait SoundEffects {
    /// Queue `sound` on `channel` (0 to 2) unless `max_queued` sounds are
    /// already waiting there.
    fn queue_sound(&mut self, channel: usize, sound: u8, max_queued: u8);
}

/// Brightness requests for the next display refresh.
pub trait DisplayRefresh {
    /// Step the screen fade one frame toward full brightness or black.
    fn fade(&mut self, fade_in: bool);
    /// Set or clear forced blank.
    fn set_blank(&mut self, blank: bool);
}
