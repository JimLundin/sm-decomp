//! The escape countdown (§5.1): the time remaining in the escape, shown over the room rather
//! than in the status bar. The spec covers only the frames before an escape countdown begins;
//! starting and advancing one, its digits and what its expiry does are outside it.

/// Spec: EscapeCountdown. The current countdown, part of the HUD's State. No other input of the
/// HUD, nor her energy or her protection from hits, replaces it.
///
/// An active or an expired countdown is outside the spec, and so is not a value of this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Countdown {
    /// No escape countdown has begun.
    Dormant,
}

/// Spec: EscapeCountdown. The countdown runs: an input each frame, the scene's choice, which the
/// ordinary HUD frame doesn't imply.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CountdownRuns {
    /// The scene runs the escape countdown this frame.
    MaintainCountdown,
    /// The scene only shows it: outside the spec, so the frame is refused
    /// (`UnsupportedFrame::CountdownOnlyShown`).
    PresentationOnly,
}

/// Spec: EscapeCountdown. Whether the escape's time has run out, read by the escape scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CountdownExpiry {
    /// The escape's time hasn't run out.
    NoExpiry,
}

/// Spec: EscapeCountdown. The countdown picture the frame contributes to the room's
/// presentation, apart from the status bar's cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CountdownPicture {
    /// No countdown picture this frame.
    Absent,
}

/// Spec: EscapeCountdown. The countdown's one answer for a frame of play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountdownAnswer {
    /// Whether the escape's time has run out.
    pub expiry: CountdownExpiry,
    /// The countdown picture contributed.
    pub picture: CountdownPicture,
}

/// A frame of play the spec leaves out, which the HUD refuses before any effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedFrame {
    /// Spec: EscapeCountdown. The scene only shows the countdown this frame.
    CountdownOnlyShown,
}

impl Countdown {
    /// Spec: EscapeCountdown. A frame whose scene runs the countdown, before one has begun: it
    /// stays dormant, the escape doesn't expire and no countdown picture is contributed. A frame
    /// that only shows it is refused.
    pub(crate) const fn answer(
        self,
        runs: CountdownRuns,
    ) -> Result<CountdownAnswer, UnsupportedFrame> {
        match (self, runs) {
            (Self::Dormant, CountdownRuns::MaintainCountdown) => Ok(CountdownAnswer {
                expiry: CountdownExpiry::NoExpiry,
                picture: CountdownPicture::Absent,
            }),
            (Self::Dormant, CountdownRuns::PresentationOnly) => {
                Err(UnsupportedFrame::CountdownOnlyShown)
            }
        }
    }
}
