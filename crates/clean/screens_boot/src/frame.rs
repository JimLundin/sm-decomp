//! What the power-on reads at a frame end, and what it reports.

use types::power_on::PowerOnState;

/// Whether the frame's work finished by its end (from the platform).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameWork {
    /// The frame's work finished: the frame counts.
    Finished,
    /// The frame's work did not finish by its end.
    Unfinished,
}

/// Whether the boot's sound driver upload ended within the frame (from the audio side). It ends
/// once, while the sound loads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverUpload {
    /// The upload ended within this frame.
    Ended,
    /// The upload did not end within this frame: it is still going, or not yet begun or long done.
    Continuing,
}

/// One frame end, as the power-on reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameEnd {
    /// Whether the frame's work finished.
    pub work: FrameWork,
    /// Whether the driver upload ended within the frame.
    pub upload: DriverUpload,
}

impl FrameEnd {
    /// A frame end in which the driver upload did not end: every frame end but one.
    #[must_use]
    pub const fn without_upload_end(work: FrameWork) -> Self {
        Self {
            work,
            upload: DriverUpload::Continuing,
        }
    }
}

/// What a frame end comes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The power-on goes on. Its State reports the screen at this frame end and the count of
    /// finished frames.
    PowerOn(PowerOnState),
    /// Spec: `Settling`. The frame was the first to finish after the game settled: it is the
    /// title's, which begins here and which this model leaves to the title.
    TitleBegins,
}
