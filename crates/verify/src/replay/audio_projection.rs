//! Independent semantic projection of native audio state.

use audio_transport::{PitchOffset, SoundGroup, State, TrackPosition};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticObservation {
    pub field: String,
    pub value: i64,
}

fn observation(out: &mut Vec<SemanticObservation>, field: String, value: impl Into<i64>) {
    out.push(SemanticObservation {
        field,
        value: value.into(),
    });
}

/// Admitted song projection. Inactive ramp target/increment has no retained semantic value.
#[must_use]
pub fn project(state: &State) -> Vec<SemanticObservation> {
    let mut out = Vec::new();
    for (group, saved) in &state.groups {
        let name = match group {
            SoundGroup::Action => "Action",
            SoundGroup::Ambient => "Ambient",
            SoundGroup::Contact => "Contact",
        };
        for (part, release) in saved.history.iter().enumerate() {
            observation(
                &mut out,
                format!("effect.{name}.part{part}.release_active"),
                u8::from(matches!(release, audio_transport::ReleaseMemory::Active(_))),
            );
            observation(
                &mut out,
                format!("effect.{name}.part{part}.release_remaining"),
                release.remaining(),
            );
        }
    }
    for (index, track) in state.song.tracks.iter().enumerate() {
        let repeats = match track.position {
            TrackPosition::Root { .. } => 0,
            TrackPosition::Repeated { remaining, .. } => remaining.get(),
        };
        let offset = match track.offset {
            PitchOffset::Zero => 0_u8,
        };
        for (name, value) in [
            ("note", i64::from(track.note.0.get())),
            ("release", i64::from(track.release)),
            ("duration", i64::from(track.duration.0.get())),
            ("articulation", i64::from(track.articulation)),
            ("repeats", i64::from(repeats)),
            ("pitch", i64::from(track.pitch.get())),
            ("loudness", i64::from(track.loudness.0)),
            (
                "ramp_remaining",
                i64::from(track.ramp.as_ref().map_or(0, |ramp| ramp.remaining.get())),
            ),
            ("timbre", i64::from(track.timbre.0)),
            ("offset", i64::from(offset)),
            ("strength", i64::from(track.strength)),
        ] {
            observation(&mut out, format!("track{index}.{name}"), value);
        }
        if let Some(ramp) = &track.ramp {
            observation(&mut out, format!("track{index}.ramp_target"), ramp.target);
            observation(
                &mut out,
                format!("track{index}.ramp_increment"),
                ramp.increment,
            );
        }
    }
    let mix = &state.song.mix;
    for (index, voice) in [
        audio_transport::VoiceId::FirstMusic,
        audio_transport::VoiceId::SecondMusic,
        audio_transport::VoiceId::Effect(audio_transport::EffectVoiceId::V2),
        audio_transport::VoiceId::Effect(audio_transport::EffectVoiceId::V3),
        audio_transport::VoiceId::Effect(audio_transport::EffectVoiceId::V4),
        audio_transport::VoiceId::Effect(audio_transport::EffectVoiceId::V5),
        audio_transport::VoiceId::Effect(audio_transport::EffectVoiceId::V6),
        audio_transport::VoiceId::Effect(audio_transport::EffectVoiceId::V7),
    ]
    .iter()
    .enumerate()
    {
        observation(
            &mut out,
            format!("mix.echo_voice{index}"),
            u8::from(mix.echo_voices.contains(voice)),
        );
    }
    observation(
        &mut out,
        String::from("mix.echo_enabled"),
        u8::from(mix.echo_enabled),
    );
    for (name, value) in [
        ("percussion", mix.percussion),
        ("master", mix.master.0),
        ("echo_left", mix.echo_left),
        ("echo_right", mix.echo_right),
        ("echo_delay", u16::from(mix.echo_delay)),
        ("feedback", u16::from(mix.feedback)),
        ("filter", mix.filter),
    ] {
        observation(&mut out, format!("mix.{name}"), value);
    }
    out
}
