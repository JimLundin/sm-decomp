//! Import the private independently decoded paired fixture. No clean vector supplies an
//! expectation, and unsupported physical bytes are decoding defects.

use super::{Arm, Counts, Cut, Defect, Observation, Publication};
use audio_transport::{
    AudioInstant, Content, CueId, PitchOffset, Reply, ReplyLine, Request, SoundGroup, State,
    TrackPosition, transport,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    group: u8,
    byte: u8,
    at: AudioInstant,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Alias {
    group: u8,
    byte: u8,
    cue: CueId,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    name: String,
    size: usize,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriterMask {
    cut: usize,
    field: String,
    writer: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    seed: State,
    content: Content,
    inputs: Vec<Presentation>,
    publications: Vec<Publication>,
    initial_replies: [u8; 4],
    cuts: Vec<Cut>,
    expire: bool,
    aliases: Vec<Alias>,
    source_writer_masks: usize,
    masks: Vec<WriterMask>,
    inactive_ramp_observations: usize,
    pins: Vec<Pin>,
    roster_sha256: String,
    roster_files: usize,
    timbres: usize,
}

const fn group(value: u8) -> Result<SoundGroup, Defect> {
    match value {
        1 => Ok(SoundGroup::Action),
        2 => Ok(SoundGroup::Ambient),
        3 => Ok(SoundGroup::Contact),
        _ => Err(Defect::Admission("unsupported physical group")),
    }
}

fn observation(out: &mut Vec<Observation>, field: String, value: impl Into<i64>) {
    out.push(Observation {
        field,
        value: value.into(),
    });
}

/// Admitted song projection. Inactive ramp target/increment has no retained semantic value.
#[must_use]
pub fn project(state: &State) -> Vec<Observation> {
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

/// Load one compact independently decoded native arm and prove its complete continuations.
/// The encrypted bundle pins the fixture; its provenance preserves the frozen source pins.
///
/// # Errors
/// Missing/unsupported/malformed fixtures and model differences are structured defects.
pub fn run(path: &Path) -> Result<Counts, Defect> {
    let text =
        std::fs::read_to_string(path).map_err(|_| Defect::Admission("missing native fixture"))?;
    let fixture: Fixture =
        toml::from_str(&text).map_err(|_| Defect::Admission("native fixture decode"))?;
    if fixture.source_writer_masks != 77
        || fixture.masks.len() != 77
        || fixture.masks.iter().any(|mask| {
            mask.cut >= 184
                || mask.field.is_empty()
                || !matches!(
                    mask.writer.as_str(),
                    "music157"
                        | "music913-914/196-197"
                        | "music175"
                        | "music510"
                        | "music785-795"
                        | "music517-520"
                )
        })
        || fixture.pins.len() != 27
        || fixture.roster_files != 7206
        || fixture.roster_sha256
            != "7a9305e8f4a582254da3c91304e535523739617376cf98bb9874d1b04e1d6a60"
        || fixture
            .pins
            .iter()
            .any(|pin| pin.name.is_empty() || pin.size == 0 || pin.sha256.len() != 64)
        || fixture.cuts.iter().any(|cut| cut.fields.is_empty())
        || fixture.inactive_ramp_observations > 736
    {
        return Err(Defect::Admission(
            "native fixture provenance or field contract",
        ));
    }
    let aliases = fixture.aliases;
    super::admission::validate(&fixture.seed, &fixture.content, fixture.timbres)?;
    let encode = |line, reply| match reply {
        Reply::Music(resident) => {
            if line == ReplyLine::Music {
                Ok(resident.original_byte)
            } else {
                Err(Defect::Admission("resident reply on effect line"))
            }
        }
        Reply::Effect(Request::Absent) => {
            if line == ReplyLine::Music {
                Err(Defect::Admission("effect absence on music line"))
            } else {
                Ok(0)
            }
        }
        Reply::Effect(Request::Cue(cue)) => {
            let ReplyLine::Group(sound_group) = line else {
                return Err(Defect::Admission("music cue outside admission"));
            };
            aliases
                .iter()
                .find_map(|alias| {
                    (alias.cue == cue && group(alias.group).ok() == Some(sound_group))
                        .then_some(alias.byte)
                })
                .ok_or(Defect::Admission("missing physical cue alias"))
        }
    };
    let inputs = fixture
        .inputs
        .into_iter()
        .map(|input| {
            let sound_group = group(input.group)?;
            if input.byte == 0 {
                return Ok(transport::Input::absence(input.at, sound_group));
            }
            let cue = aliases
                .iter()
                .find(|alias| alias.group == input.group && alias.byte == input.byte)
                .map(|alias| alias.cue)
                .ok_or(Defect::Admission("unsupported presentation byte"))?;
            if audio_transport::effects::group(cue) != sound_group {
                return Err(Defect::Admission("cue group mismatch"));
            }
            Ok(transport::Input::cue(input.at, cue))
        })
        .collect::<Result<Vec<_>, Defect>>()?;
    let arm = Arm {
        seed: fixture.seed,
        content: fixture.content,
        inputs,
        publications: fixture.publications,
        initial_replies: fixture.initial_replies,
        cuts: fixture.cuts,
        expire: fixture.expire,
    };
    let mut counts = super::run(&arm, &encode, project)?;
    let pending = super::pending::run(&arm, &encode)?;
    counts.pending_boundaries = pending.boundaries;
    counts.pending_restorations = pending.restorations;
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::run;
    use serde::Deserialize;
    use std::io::Write;
    use std::path::PathBuf;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct NativeFloors {
        control: super::Counts,
        expire: super::Counts,
    }

    /// Explicit private evidence runner until the compact encrypted fixture is pinned in CI.
    #[test]
    #[ignore = "requires SM_AUDIO_NATIVE pointing to independently pinned private fixtures"]
    fn native_paired_arms_are_exact() {
        let directory = std::env::var_os("SM_AUDIO_NATIVE").map(PathBuf::from);
        assert!(directory.is_some());
        let Some(directory) = directory else {
            return;
        };
        let floors: NativeFloors = toml::from_str(include_str!("native_floors.toml")).unwrap();
        let results =
            [("control", floors.control), ("expire", floors.expire)].map(|(arm, floor)| {
                (
                    arm,
                    floor,
                    run(&directory.join(format!("{arm}.local.toml"))),
                )
            });
        let mut failures = Vec::new();
        for (arm, floor, result) in results {
            if let Err(defect) = &result {
                assert!(
                    writeln!(
                        std::io::stdout().lock(),
                        "NATIVE audio-transport {arm} failure {defect:?}"
                    )
                    .is_ok()
                );
                failures.push((arm, defect.clone()));
            }
            let Ok(counts) = result else {
                continue;
            };
            assert!(
                writeln!(
                    std::io::stdout().lock(),
                    "NATIVE audio-transport {arm} compared{} publications{} replies{} fields{} null_cuts{} null_fields{} restorations{} skipped0 defects0 cpu_timing0/140",
                    counts.compared, counts.publications, counts.replies, counts.fields,
                    counts.null_cuts, counts.null_fields, counts.restorations
                )
                .is_ok()
            );
            assert_eq!(counts, floor);
            assert!(writeln!(std::io::stdout().lock(),
                "PENDING audio-transport {arm} boundaries{} restorations{} zero_elapsed_exact committed_retained pending_retained publication_once",
                counts.pending_boundaries, counts.pending_restorations).is_ok());
            assert_eq!(
                (counts.pending_boundaries, counts.pending_restorations),
                (573, 573)
            );
        }
        assert!(failures.is_empty(), "native arm failures: {failures:?}");
    }
}
