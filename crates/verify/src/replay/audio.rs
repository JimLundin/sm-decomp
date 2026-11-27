//! Native capture admission, one frame at a time.
//!
//! The capture's port log supplies each frame's borrowed inputs: its offers (each cue the CPU
//! newly presents, offered in the frame it is presented) and its audio cadence (the effect turns
//! completed before and after the frame's first CPU write, and its completed music turns). The
//! expected side reads only the capture's cut outcomes: the replies last published before each
//! cut, the request lines last written, and the decoded song and release fields.

use super::audio_projection::SemanticObservation;
use super::compare::Observation;
use super::index::{FixtureFile, valid_hash};
use audio_transport::frame::{EffectCadence, FrameCadence, TurnCount};
use audio_transport::{
    Content, CueId, GroupState, OfferPermission, QueueAllowance, Request, ResidentMusicReply,
    SongState, SoundGroup,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) mod room_load;

/// Independently reviewed provenance for native captures of the opening room.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AudioPacket {
    origin: FixtureFile,
    seed_row: String,
    roster_sha256: String,
    roster_files: usize,
    pins: usize,
    cuts: usize,
    publications: usize,
    writer_masks: usize,
    writers: BTreeSet<String>,
    max_inactive_ramp_observations: usize,
    expiry_prefix_end: u64,
    instruments: usize,
    samples: usize,
    resources_sha256: String,
}

impl AudioPacket {
    pub(super) fn valid(&self) -> bool {
        self.origin.valid()
            && !self.seed_row.is_empty()
            && valid_hash(&self.roster_sha256)
            && valid_hash(&self.resources_sha256)
            && self.roster_files > 0
            && self.pins > 0
            && usize::try_from(self.expiry_prefix_end).is_ok_and(|end| end < self.cuts)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderFault {
    Admission(&'static str),
    Pin,
    Missing,
}

/// A captured instant, an exact non-negative fraction of audio cycles since the capture began.
#[derive(Clone, Copy, Debug)]
struct Instant {
    numerator: u128,
    denominator: u128,
}

/// A captured fraction's integer: a sign and little-endian 32-bit limbs.
#[derive(Deserialize)]
struct Integer(i8, Vec<u32>);

impl Integer {
    fn value(&self) -> Option<u128> {
        // Two limbs keep every cross product of two fractions within u128.
        if self.0 < 0 || self.1.len() > 2 {
            return None;
        }
        Some(
            self.1
                .iter()
                .rev()
                .fold(0, |value, limb| (value << 32) | u128::from(*limb)),
        )
    }
}

#[derive(Deserialize)]
struct Fraction {
    numerator: Integer,
    denominator: Integer,
}

impl<'de> Deserialize<'de> for Instant {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fraction = Fraction::deserialize(deserializer)?;
        match (fraction.numerator.value(), fraction.denominator.value()) {
            (Some(numerator), Some(denominator)) if denominator > 0 => Ok(Self {
                numerator,
                denominator,
            }),
            _ => Err(serde::de::Error::custom("captured instant fraction")),
        }
    }
}

impl PartialEq for Instant {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Instant {}

impl PartialOrd for Instant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Instant {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.numerator.saturating_mul(other.denominator);
        let right = other.numerator.saturating_mul(self.denominator);
        left.cmp(&right)
    }
}

/// The line a captured publication was written on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum Line {
    Music,
    Group(SoundGroup),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    line: Line,
    // Replies are compared through each cut's own reply bytes.
    #[serde(rename = "byte")]
    _byte: u8,
    at: Instant,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cut {
    index: usize,
    at: Instant,
    publications: usize,
    replies: [u8; 4],
    fields: Vec<SemanticObservation>,
    // The old transport's pending-music flag; the frame form has no pending work to compare.
    #[serde(rename = "pending_music")]
    _pending_music: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    group: u8,
    byte: u8,
    at: Instant,
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
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

/// The loaded seed's audio side. The capture also records the driver's clock, cadence
/// remainders and scheduler phase; they are T1's and the frame form reads none of them.
#[derive(Clone, Deserialize)]
pub struct Seed {
    pub groups: enum_map::EnumMap<SoundGroup, GroupState>,
    pub song: SongState,
    pub reservations: BTreeSet<audio_transport::EffectVoiceId>,
    pub requests: enum_map::EnumMap<SoundGroup, Request>,
    pub resident_music_reply: ResidentMusicReply,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    seed: Seed,
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

/// One frame's borrowed inputs.
pub struct Frame {
    pub offers: Vec<audio_transport::Offer>,
    pub cadence: FrameCadence,
}

/// Contains no expected quantities: the seed, content and each frame's borrowed inputs.
pub struct InputTape {
    pub seed: Seed,
    pub content: Content,
    pub initial_replies: [u8; 4],
    pub aliases: Vec<Alias>,
    pub frames: Vec<(u64, Frame)>,
}

/// One cut's expected observations, and how many of them are replies.
pub struct ExpectedCut {
    pub fields: Vec<Observation>,
    pub semantic_count: usize,
    pub replies: usize,
}

/// Capture outcomes are inaccessible through the production input tape.
pub struct ExpectedTape {
    pub cuts: Vec<(u64, ExpectedCut)>,
}

pub const fn group(value: u8) -> Result<SoundGroup, ProviderFault> {
    match value {
        1 => Ok(SoundGroup::Action),
        2 => Ok(SoundGroup::Ambient),
        3 => Ok(SoundGroup::Contact),
        _ => Err(ProviderFault::Admission("unsupported physical group")),
    }
}

const fn slot(group: SoundGroup) -> usize {
    match group {
        SoundGroup::Action => 1,
        SoundGroup::Ambient => 2,
        SoundGroup::Contact => 3,
    }
}

/// The physical byte of a request on its group's line.
pub fn encode(aliases: &[Alias], line: SoundGroup, request: Request) -> Option<u8> {
    match request {
        Request::Absent => Some(0),
        Request::Cue(cue) => aliases
            .iter()
            .find(|alias| alias.cue == cue && group(alias.group).ok() == Some(line))
            .map(|alias| alias.byte),
    }
}

fn read(path: &Path, hash: &str) -> Result<Fixture, ProviderFault> {
    let bytes = std::fs::read(path).map_err(|_| ProviderFault::Missing)?;
    if crate::lock::Sha256::try_from(hash).ok() != Some(crate::lock::Sha256::of(&bytes)) {
        return Err(ProviderFault::Pin);
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| ProviderFault::Admission("native fixture encoding"))?;
    // The capture's music events still carry the old transport's per-event extra work, which
    // the frame form ignores. The TOML reader can't skip that field in place, so the text is
    // read as a table first.
    toml::from_str::<toml::Table>(text)
        .ok()
        .and_then(|table| toml::Value::Table(table).try_into().ok())
        .ok_or(ProviderFault::Admission("native fixture schema"))
}

pub fn load(
    path: &Path,
    hash: &str,
    end: u64,
    packet: &AudioPacket,
) -> Result<(InputTape, ExpectedTape), ProviderFault> {
    let fixture = read(path, hash)?;
    if fixture.expire && end > packet.expiry_prefix_end {
        return Err(ProviderFault::Admission(
            "expiry ends before the external repeat-count intervention",
        ));
    }
    validate_provenance(&fixture, packet)?;
    validate_order(&fixture)?;
    let writes = fixture
        .inputs
        .iter()
        .map(|input| Ok((group(input.group)?, input.byte, input.at)))
        .collect::<Result<Vec<_>, ProviderFault>>()?;
    let frames = frames(&fixture, &writes, end)?;
    let expected = expected(&fixture, &writes, end)?;
    Ok((
        InputTape {
            seed: fixture.seed,
            content: fixture.content,
            initial_replies: fixture.initial_replies,
            aliases: fixture.aliases,
            frames,
        },
        expected,
    ))
}

fn validate_provenance(fixture: &Fixture, packet: &AudioPacket) -> Result<(), ProviderFault> {
    if fixture.source_writer_masks != packet.writer_masks
        || fixture.masks.len() != packet.writer_masks
        || fixture.masks.iter().any(|mask| {
            mask.cut >= fixture.cuts.len()
                || mask.field.is_empty()
                || !packet.writers.contains(&mask.writer)
        })
        || fixture.pins.len() != packet.pins
        || fixture.roster_files != packet.roster_files
        || fixture.roster_sha256 != packet.roster_sha256
        || fixture
            .pins
            .iter()
            .any(|pin| pin.name.is_empty() || pin.size == 0 || !valid_hash(&pin.sha256))
        || fixture.cuts.len() != packet.cuts
        || fixture.publications.len() != packet.publications
        || fixture
            .cuts
            .iter()
            .enumerate()
            .any(|(index, cut)| cut.index != index || cut.fields.is_empty())
        || fixture.inactive_ramp_observations > packet.max_inactive_ramp_observations
        || fixture.timbres == 0
        || !fixture.seed.reservations.is_empty()
    {
        return Err(ProviderFault::Admission(
            "native source provenance or field contract",
        ));
    }
    Ok(())
}

/// Instants are ordered, and each cut's publication count is its prefix.
fn validate_order(fixture: &Fixture) -> Result<(), ProviderFault> {
    let ordered = |at: &[Instant]| at.is_sorted();
    let cuts: Vec<_> = fixture.cuts.iter().map(|cut| cut.at).collect();
    let publications: Vec<_> = fixture.publications.iter().map(|at| at.at).collect();
    let inputs: Vec<_> = fixture.inputs.iter().map(|input| input.at).collect();
    let prefixes = fixture.cuts.iter().all(|cut| {
        fixture
            .publications
            .iter()
            .filter(|publication| publication.at <= cut.at)
            .count()
            == cut.publications
    });
    if !ordered(&cuts) || !ordered(&publications) || !ordered(&inputs) || !prefixes {
        return Err(ProviderFault::Admission("native capture order"));
    }
    Ok(())
}

/// The instants after the previous cut and through this one.
fn within(cuts: &[Cut], index: usize, at: Instant) -> bool {
    let after_previous = index
        .checked_sub(1)
        .and_then(|previous| cuts.get(previous))
        .is_none_or(|previous| at > previous.at);
    cuts.get(index).is_some_and(|cut| at <= cut.at) && after_previous
}

fn turns(count: usize) -> Result<TurnCount, ProviderFault> {
    u8::try_from(count)
        .ok()
        .and_then(TurnCount::new)
        .ok_or(ProviderFault::Admission(
            "more than two audio turns in a frame",
        ))
}

/// Each frame's offers and cadence, from the CPU writes and the SPC's publications.
///
/// An effect turn publishes every group's reply in turn order, so it is complete at its Contact
/// publication. The frame's sender service is placed at its first CPU write; a frame that
/// writes nothing still runs its service, after its turns.
fn frames(
    fixture: &Fixture,
    writes: &[(SoundGroup, u8, Instant)],
    end: u64,
) -> Result<Vec<(u64, Frame)>, ProviderFault> {
    let completed: Vec<Instant> = fixture
        .publications
        .iter()
        .filter(|publication| publication.line == Line::Group(SoundGroup::Contact))
        .map(|publication| publication.at)
        .collect();
    let music: Vec<Instant> = fixture
        .publications
        .iter()
        .filter(|publication| publication.line == Line::Music)
        .map(|publication| publication.at)
        .collect();
    let mut presented = enum_map::EnumMap::<SoundGroup, u8>::default();
    let mut frames = Vec::new();
    for (index, cut) in fixture.cuts.iter().enumerate() {
        let number = u64::try_from(cut.index).map_err(|_| ProviderFault::Admission("cut index"))?;
        if number > end {
            break;
        }
        let mut offers = Vec::new();
        let mut service = None;
        for &(line, byte, at) in writes
            .iter()
            .filter(|(_, _, at)| within(&fixture.cuts, index, *at))
        {
            service.get_or_insert(at);
            if byte != 0 && byte != presented[line] {
                let cue = fixture
                    .aliases
                    .iter()
                    .find(|alias| group(alias.group).ok() == Some(line) && alias.byte == byte)
                    .map(|alias| alias.cue)
                    .ok_or(ProviderFault::Admission("unsupported presentation byte"))?;
                if cue.group() != line {
                    return Err(ProviderFault::Admission("cue group mismatch"));
                }
                offers.push(audio_transport::Offer {
                    cue,
                    allowance: QueueAllowance::Fifteen,
                    permission: OfferPermission::Enabled,
                });
            }
            presented[line] = byte;
        }
        let in_frame = |at: &&Instant| within(&fixture.cuts, index, **at);
        let effect = completed.iter().filter(in_frame).count();
        let before = service.map_or(effect, |service| {
            completed
                .iter()
                .filter(in_frame)
                .filter(|at| **at < service)
                .count()
        });
        let after = effect.saturating_sub(before);
        frames.push((
            number,
            Frame {
                offers,
                cadence: FrameCadence {
                    effects: EffectCadence::AroundService {
                        before: turns(before)?,
                        after: turns(after)?,
                    },
                    music: turns(music.iter().filter(in_frame).count())?,
                },
            },
        ));
    }
    Ok(frames)
}

/// Each cut's expected song fields, replies and request lines.
///
/// A cut can fall inside an effect turn: the turn has published some groups' replies but not
/// Contact's. The frame form completes that turn in the next frame, so the replies it had
/// already published are T1's and are not compared at that cut.
fn expected(
    fixture: &Fixture,
    writes: &[(SoundGroup, u8, Instant)],
    end: u64,
) -> Result<ExpectedTape, ProviderFault> {
    let mut cuts = Vec::new();
    for cut in &fixture.cuts {
        let number = u64::try_from(cut.index).map_err(|_| ProviderFault::Admission("cut index"))?;
        if number > end {
            break;
        }
        let published = |line: Line| {
            fixture
                .publications
                .iter()
                .filter(|publication| publication.line == line && publication.at <= cut.at)
                .count()
        };
        let complete = published(Line::Group(SoundGroup::Contact));
        let mut fields: Vec<_> = cut
            .fields
            .iter()
            .map(|field| Observation::quantity(field.field.clone(), field.value))
            .collect();
        let semantic_count = fields.len();
        let reply = |index: usize| cut.replies.get(index).copied().unwrap_or_default();
        fields.push(Observation::quantity("audio.reply0", i64::from(reply(0))));
        let mut replies = 1_usize;
        for line in SoundGroup::IN_TURN_ORDER {
            let index = slot(line);
            if published(Line::Group(line)) == complete {
                fields.push(Observation::quantity(
                    format!("audio.reply{index}"),
                    i64::from(reply(index)),
                ));
                replies = replies.saturating_add(1);
            }
            let written = writes
                .iter()
                .rfind(|(group, _, at)| *group == line && *at <= cut.at)
                .map_or_else(
                    || encode(&fixture.aliases, line, fixture.seed.requests[line]),
                    |(_, byte, _)| Some(*byte),
                )
                .ok_or(ProviderFault::Admission("seed request alias"))?;
            fields.push(Observation::quantity(
                format!("audio.request{index}"),
                i64::from(written),
            ));
        }
        cuts.push((
            number,
            ExpectedCut {
                fields,
                semantic_count,
                replies,
            },
        ));
    }
    Ok(ExpectedTape { cuts })
}

/// Play the cut's frame with its borrowed offers and cadence.
pub(super) fn advance(
    production: &mut game::production::Production,
    tape: &InputTape,
    cut: u64,
) -> Result<game::production::ProductionOutputs, super::Outcome> {
    let frame = tape
        .frames
        .iter()
        .find(|(number, _)| *number == cut)
        .map(|(_, frame)| frame)
        .ok_or_else(|| super::data_failure(Some(cut), "borrowed audio frame"))?;
    production
        .advance(game::production::AdvanceInput::Audio {
            offers: &frame.offers,
            cadence: frame.cadence,
        })
        .map_err(|fault| super::production_failure(cut, fault))
}

/// The held State's song and release fields, its published replies and its request lines.
pub(super) fn actual(
    production: &game::production::Production,
    aliases: &[Alias],
) -> Result<Vec<Observation>, super::Outcome> {
    let state = production
        .transport()
        .ok_or_else(|| super::data_failure(None, "audio state scope"))?;
    let mut fields: Vec<_> = super::audio_projection::project(state)
        .into_iter()
        .map(|field| Observation::quantity(field.field, field.value))
        .collect();
    fields.push(Observation::quantity(
        "audio.reply0",
        i64::from(state.music_reply.original_byte),
    ));
    for line in SoundGroup::IN_TURN_ORDER {
        let index = slot(line);
        let byte = |request| {
            encode(aliases, line, request)
                .map(i64::from)
                .ok_or_else(|| super::data_failure(None, "physical cue alias"))
        };
        fields.push(Observation::quantity(
            format!("audio.reply{index}"),
            byte(state.groups[line].reply)?,
        ));
        fields.push(Observation::quantity(
            format!("audio.request{index}"),
            byte(state.request_lines[line])?,
        ));
    }
    Ok(fields)
}

#[cfg(test)]
mod import_witness;
