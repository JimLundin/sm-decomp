//! Native capture admission separates immutable physical inputs from expected cut outcomes.

use super::audio_projection::SemanticObservation;
use audio_transport::{
    AudioInstant, Content, CueId, Reply, ReplyLine, Request, SoundGroup, State, transport,
};
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderFault {
    Admission(&'static str),
    Pin,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub line: ReplyLine,
    pub byte: u8,
    pub at: AudioInstant,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cut {
    pub index: usize,
    pub at: AudioInstant,
    pub publications: usize,
    pub replies: [u8; 4],
    pub fields: Vec<SemanticObservation>,
    pub pending_music: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    group: u8,
    byte: u8,
    at: AudioInstant,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alias {
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Continuation {
    schema: String,
    start_cut: usize,
    next_cut: usize,
    last_cut: usize,
    source_boundary: String,
    coordinate: String,
    content_sha256: String,
    fixture_sha256: String,
    publication_offset: usize,
    initial_replies: [u8; 4],
    seed: State,
    pins: Vec<Pin>,
}

/// Contains no later expected quantities. The clock tape is a declared measured input.
pub struct InputTape {
    pub seed: State,
    pub content: Content,
    pub presentations: Vec<transport::Input>,
    pub stops: Vec<(u64, AudioInstant)>,
    pub initial_replies: [u8; 4],
    pub aliases: Vec<Alias>,
    pub deadlines: Vec<AudioInstant>,
}

/// Capture outcomes are inaccessible through the production input tape.
pub struct ExpectedTape {
    pub cuts: Vec<Cut>,
    pub publications: Vec<Publication>,
}

pub const fn group(value: u8) -> Result<SoundGroup, ProviderFault> {
    match value {
        1 => Ok(SoundGroup::Action),
        2 => Ok(SoundGroup::Ambient),
        3 => Ok(SoundGroup::Contact),
        _ => Err(ProviderFault::Admission("unsupported physical group")),
    }
}

pub fn encode(aliases: &[Alias], line: ReplyLine, reply: Reply) -> Result<u8, ProviderFault> {
    match (line, reply) {
        (ReplyLine::Music, Reply::Music(resident)) => Ok(resident.original_byte),
        (ReplyLine::Group(_), Reply::Effect(Request::Absent)) => Ok(0),
        (ReplyLine::Group(sound_group), Reply::Effect(Request::Cue(cue))) => aliases
            .iter()
            .find(|alias| alias.cue == cue && group(alias.group).ok() == Some(sound_group))
            .map(|alias| alias.byte)
            .ok_or(ProviderFault::Admission("missing physical cue alias")),
        (ReplyLine::Music, Reply::Effect(_)) | (ReplyLine::Group(_), Reply::Music(_)) => {
            Err(ProviderFault::Admission("reply line kind"))
        }
    }
}

pub fn load(
    path: &Path,
    hash: &str,
    end: u64,
    continuation: Option<(&Path, &str, u64)>,
) -> Result<(InputTape, ExpectedTape), ProviderFault> {
    let bytes = std::fs::read(path).map_err(|_| ProviderFault::Missing)?;
    if crate::lock::Sha256::try_from(hash).ok() != Some(crate::lock::Sha256::of(&bytes)) {
        return Err(ProviderFault::Pin);
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| ProviderFault::Admission("native fixture encoding"))?;
    let mut fixture: Fixture =
        toml::from_str(text).map_err(|_| ProviderFault::Admission("native fixture schema"))?;
    if fixture.expire && continuation.is_none() && end > 64 {
        return Err(ProviderFault::Admission(
            "expiry requires an independently captured post-intervention seed",
        ));
    }
    validate_provenance(&fixture)?;
    super::audio_admission::validate(&fixture.seed, &fixture.content, fixture.timbres)?;
    if let Some((path, seed_hash, frame)) = continuation {
        continue_from(&mut fixture, text, hash, path, seed_hash, frame)?;
    }
    let presentations = fixture
        .inputs
        .into_iter()
        .map(|input| {
            let sound_group = group(input.group)?;
            if input.byte == 0 {
                return Ok(transport::Input::absence(input.at, sound_group));
            }
            let cue = fixture
                .aliases
                .iter()
                .find(|alias| alias.group == input.group && alias.byte == input.byte)
                .map(|alias| alias.cue)
                .ok_or(ProviderFault::Admission("unsupported presentation byte"))?;
            if audio_transport::effects::group(cue) != sound_group {
                return Err(ProviderFault::Admission("cue group mismatch"));
            }
            Ok(transport::Input::cue(input.at, cue))
        })
        .collect::<Result<Vec<_>, ProviderFault>>()?;
    if presentations
        .iter()
        .any(|input| input.at <= fixture.seed.now)
        || presentations
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left.at > right.at))
        || fixture
            .cuts
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left.at > right.at))
    {
        return Err(ProviderFault::Admission("native input clock order"));
    }
    let stops = fixture
        .cuts
        .iter()
        .map(|cut| (u64::try_from(cut.index).unwrap_or(u64::MAX), cut.at.clone()))
        .collect();
    let deadlines = fixture
        .publications
        .iter()
        .filter(|publication| {
            fixture
                .cuts
                .iter()
                .any(|cut| u64::try_from(cut.index).ok() == Some(end) && publication.at <= cut.at)
        })
        .map(|publication| publication.at.clone())
        .collect();
    Ok((
        InputTape {
            seed: fixture.seed,
            content: fixture.content,
            presentations,
            stops,
            initial_replies: fixture.initial_replies,
            aliases: fixture.aliases,
            deadlines,
        },
        ExpectedTape {
            cuts: fixture.cuts,
            publications: fixture.publications,
        },
    ))
}

fn validate_provenance(fixture: &Fixture) -> Result<(), ProviderFault> {
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
        || fixture.pins.iter().any(|pin| {
            pin.name.is_empty() || pin.size == 0 || !super::index::valid_hash(&pin.sha256)
        })
        || fixture.cuts.len() != 184
        || fixture.publications.len() != 764
        || fixture
            .cuts
            .iter()
            .enumerate()
            .any(|(index, cut)| cut.index != index || cut.fields.is_empty())
        || fixture.inactive_ramp_observations > 736
    {
        return Err(ProviderFault::Admission(
            "native source provenance or field contract",
        ));
    }
    Ok(())
}

fn continue_from(
    fixture: &mut Fixture,
    text: &str,
    hash: &str,
    path: &Path,
    seed_hash: &str,
    frame: u64,
) -> Result<(), ProviderFault> {
    let seed_bytes = std::fs::read(path).map_err(|_| ProviderFault::Missing)?;
    if crate::lock::Sha256::try_from(seed_hash).ok() != Some(crate::lock::Sha256::of(&seed_bytes)) {
        return Err(ProviderFault::Pin);
    }
    let seed_text = std::str::from_utf8(&seed_bytes)
        .map_err(|_| ProviderFault::Admission("native continuation encoding"))?;
    let saved: Continuation = toml::from_str(seed_text)
        .map_err(|_| ProviderFault::Admission("native continuation schema"))?;
    let content_line = text
        .lines()
        .find_map(|line| line.strip_prefix("content = "))
        .ok_or(ProviderFault::Admission("native content identity"))?;
    let boundary = fixture
        .cuts
        .get(saved.start_cut)
        .ok_or(ProviderFault::Admission("native continuation cut"))?;
    if !fixture.expire
        || saved.schema != "audio-expiry-post64-seed-v1"
        || saved.start_cut != 64
        || saved.next_cut != 65
        || saved.last_cut != 183
        || u64::try_from(saved.start_cut).ok() != Some(frame)
        || saved.source_boundary.is_empty()
        || saved.coordinate.is_empty()
        || saved.fixture_sha256 != hash
        || crate::lock::Sha256::try_from(saved.content_sha256.as_str()).ok()
            != Some(crate::lock::Sha256::of(content_line.as_bytes()))
        || saved.seed.now != boundary.at
        || saved.publication_offset != boundary.publications
        || saved.initial_replies != boundary.replies
        || saved.pins.len() != 184
        || saved.pins.iter().any(|pin| {
            pin.name.is_empty() || pin.size == 0 || !super::index::valid_hash(&pin.sha256)
        })
    {
        return Err(ProviderFault::Admission(
            "independent native continuation provenance and context",
        ));
    }
    super::audio_admission::validate_continuation(&saved.seed, &fixture.content, fixture.timbres)?;
    fixture.seed = saved.seed;
    fixture.initial_replies = saved.initial_replies;
    fixture.inputs.retain(|input| input.at > fixture.seed.now);
    fixture.publications = fixture
        .publications
        .get(saved.publication_offset..)
        .ok_or(ProviderFault::Admission(
            "native continuation publication offset",
        ))?
        .to_vec();
    fixture.cuts.retain(|cut| cut.index >= saved.next_cut);
    for cut in &mut fixture.cuts {
        cut.publications = cut
            .publications
            .checked_sub(saved.publication_offset)
            .ok_or(ProviderFault::Admission(
                "native continuation publication horizon",
            ))?;
    }
    Ok(())
}

pub const fn reply_slot(line: ReplyLine) -> usize {
    match line {
        ReplyLine::Music => 0,
        ReplyLine::Group(SoundGroup::Action) => 1,
        ReplyLine::Group(SoundGroup::Ambient) => 2,
        ReplyLine::Group(SoundGroup::Contact) => 3,
    }
}
