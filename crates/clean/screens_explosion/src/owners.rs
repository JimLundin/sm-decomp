//! The owners whose readiness the scene reads each frame.

/// T1, the scheduler: it finishes scene frames and completes the scene's setup.
pub trait Scheduler {
    /// Whether this scene frame finished. A stalled frame holds the last picture and publishes
    /// nothing.
    fn frame_finished(&self) -> bool;
    /// Whether the setup, scheduled while the display is blank, has completed.
    fn setup_complete(&self) -> bool;
}

/// U1, the audio service.
pub trait MusicService {
    /// Whether music is still busy, sampled after this frame's audio service.
    fn music_busy(&self) -> bool;
}
