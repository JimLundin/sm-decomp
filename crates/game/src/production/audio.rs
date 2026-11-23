//! Audio transport played one frame at a time, with each frame's cadence borrowed from its caller.
//!
//! The frame's audio cadence (effect turns before and after the sender service, and music turns)
//! is a platform timing input until T1 models the CPU's frame timing and the audio clock
//! (MODEL §1 decision 35), so the caller supplies it with the frame's offers.

mod admission;
#[cfg(test)]
mod tests;

use super::AdmissionFault;
use audio_transport::frame::FrameCadence;
use audio_transport::{Offer, State};
use data::assets::audio::ResidentAudio;

/// Why audio content or a saved transport State could not be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A required content or saved-state relation is invalid.
    Invalid(&'static str),
    /// Saved state belongs to a different resident arrangement.
    ContentIdentity,
}

/// Checked effect clips and song graph, with the resources their events name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    graph: audio_transport::Content,
    timbres: usize,
    echo_delay: u8,
    music_reply: Option<audio_transport::ResidentMusicReply>,
}

impl Content {
    /// Admit a ROM-imported resident bank: its whole cue roster, aliases, resources and song.
    ///
    /// # Errors
    /// Returns a typed fault if the bank is incomplete or a graph names a missing resource.
    pub fn resident(bank: ResidentAudio) -> Result<Self, AdmissionFault> {
        admission::validate_bank(&bank).map_err(AdmissionFault::Audio)?;
        Ok(Self {
            timbres: bank.instruments.len(),
            echo_delay: bank.echo_delay,
            music_reply: Some(bank.music_reply),
            graph: bank.content,
        })
    }

    /// Admit checked clips and song graph whose events name `timbres` resources and one echo
    /// delay, such as an invented arrangement.
    ///
    /// # Errors
    /// Returns a typed fault if a graph names a missing resource or is malformed.
    pub fn checked(
        graph: audio_transport::Content,
        timbres: usize,
        echo_delay: u8,
    ) -> Result<Self, AdmissionFault> {
        admission::validate_graph(&graph, timbres, echo_delay).map_err(AdmissionFault::Audio)?;
        Ok(Self {
            graph,
            timbres,
            echo_delay,
            music_reply: None,
        })
    }

    /// Content for frames that play no sound: no clips, no song and no resources.
    ///
    /// A State admitted with it holds no song tracks and no playback, so its frames can only
    /// offer and serve cues.
    #[must_use]
    pub const fn silent() -> Self {
        Self {
            graph: audio_transport::Content {
                effects: std::collections::BTreeMap::new(),
                tracks: Vec::new(),
            },
            timbres: 0,
            echo_delay: 0,
            music_reply: None,
        }
    }

    /// The checked clips and song graph.
    #[must_use]
    pub const fn graph(&self) -> &audio_transport::Content {
        &self.graph
    }
}

/// Check a saved transport State against the content it plays.
///
/// # Errors
/// Returns a typed fault for a song, history, request or voice that the content can't play.
pub fn validate(state: &State, content: &Content) -> Result<(), AdmissionFault> {
    if content
        .music_reply
        .as_ref()
        .is_some_and(|reply| *reply != state.resident_music_reply)
    {
        return Err(AdmissionFault::Audio(Fault::ContentIdentity));
    }
    admission::validate_state(state, &content.graph, content.timbres, content.echo_delay)
        .map_err(AdmissionFault::Audio)
}

/// Play one frame: its offers, then its effect turns, sender service and music turns as the
/// borrowed cadence places them. The published replies are read back from the State.
pub(super) fn play(state: &mut State, content: &Content, offers: &[Offer], cadence: FrameCadence) {
    audio_transport::frame::play(state, &content.graph, offers, cadence);
}
