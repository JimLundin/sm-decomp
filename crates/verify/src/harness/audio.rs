//! Continuous audio proof at independently fixed CPU cuts. Inputs and expected observations
//! are decoded outside the model; no cut supplies a replacement model prestate.
//!
//! LEGACY(admitted-audio-inputs): production workload clocks, port phases and resident
//! assets are missing. This harness cannot credit the ordinary engine's CPU timing.

use audio_transport::{
    AudioInstant, AudioPhase, CompletionResult, Content, Reply, ReplyLine, SoundGroup, State,
    TrackPosition, transport,
};
use serde::Deserialize;
use std::num::NonZeroU8;

pub mod admission;
pub mod cached;
pub mod native;
mod pending;

/// A physical publication. Original bytes are retained solely at this verification boundary.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Publication {
    /// Physical line, music first and then the three effect groups.
    pub line: ReplyLine,
    /// Original byte at that line.
    pub byte: u8,
    /// Exact `AudioCycles` since the loaded seed.
    pub at: AudioInstant,
}

/// Independently decoded fields at a cut. A comparison never prints ROM-derived values.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Observation {
    /// Stable semantic field identity.
    pub field: String,
    /// Original quantity, decoded at the capture boundary.
    pub value: i64,
}

/// One named CPU cut, independent of the native SMP execution frontier.
#[derive(Deserialize)]
pub struct Cut {
    /// CPU-cut identity, beginning at zero.
    pub index: usize,
    /// Fixed elapsed `AudioCycles` in the admitted clock relation.
    pub at: AudioInstant,
    /// Cumulative physical publication count.
    pub publications: usize,
    /// Last physical replies in music/action/ambient/contact order.
    pub replies: [u8; 4],
    /// Every admitted unmasked semantic observation, independently decoded from the capture.
    pub fields: Vec<Observation>,
    /// Declared pending music completion marker.
    pub pending_music: bool,
}

/// Loaded seed and immutable independent input/expectation stream for one native arm.
pub struct Arm {
    /// One loaded seed; later cuts never reseed it.
    pub seed: State,
    /// Validated resident content, independent of model predictions.
    pub content: Content,
    /// Ordered CPU port presentations.
    pub inputs: Vec<transport::Input>,
    /// Native physical publications through the fixed last CPU cut.
    pub publications: Vec<Publication>,
    /// Physical replies from the loaded seed.
    pub initial_replies: [u8; 4],
    /// Actual cut objects, in increasing order.
    pub cuts: Vec<Cut>,
    /// Controlled repeat-expiry intervention after cut64.
    pub expire: bool,
}

/// Honest counts. A missing/invalid decoding is an error before comparison, never a skip.
#[derive(Default, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    /// Complete matching named cuts.
    pub compared: usize,
    /// Matching physical publications.
    pub publications: usize,
    /// Compared physical replies.
    pub replies: usize,
    /// Compared admitted semantic quantities.
    pub fields: usize,
    /// Cuts whose complete observation a do-nothing model also matches.
    pub null_cuts: usize,
    /// Matching admitted fields for the do-nothing model.
    pub null_fields: usize,
    /// Saved cuts whose codec-restored continuation matches through the final cut.
    pub restorations: usize,
    /// Native effect publications checked immediately before and at their completion.
    pub pending_boundaries: usize,
    /// Pending effect states restored and checked without committing their result early.
    pub pending_restorations: usize,
}

/// Structured first failure; intentionally contains identities, never original bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Defect {
    /// Unsupported or malformed independent fixture.
    Admission(&'static str),
    /// State could not be encoded or restored.
    Codec,
    /// First differing physical publication, with the differing quantity.
    Publication(usize, &'static str),
    /// First differing named cut and semantic/physical field identity.
    Cut(usize, String),
    /// Restored continuation first differs from the uninterrupted run.
    Continuation(usize),
    /// Temporal publication property differs at an independently timed native publication.
    Pending(usize, &'static str),
}

/// Original cue encoding at the physical boundary, supplied by the independent decoder.
pub type EncodeReply<'a> = dyn Fn(ReplyLine, Reply) -> Result<u8, Defect> + 'a;

/// Semantic projection supplied by the verification decoder, with no expectation access.
pub type Project = fn(&State) -> Vec<Observation>;

const fn slot(line: ReplyLine) -> usize {
    match line {
        ReplyLine::Music => 0,
        ReplyLine::Group(SoundGroup::Action) => 1,
        ReplyLine::Group(SoundGroup::Ambient) => 2,
        ReplyLine::Group(SoundGroup::Contact) => 3,
    }
}

fn interval(
    state: &mut State,
    arm: &Arm,
    cut: &Cut,
    encode: &EncodeReply<'_>,
) -> Result<Vec<Publication>, Defect> {
    if cut.at < state.now {
        return Err(Defect::Admission("cut order"));
    }
    let inputs: Vec<_> = arm
        .inputs
        .iter()
        .filter(|input| input.at > state.now)
        .filter(|input| input.at <= cut.at)
        .cloned()
        .collect();
    let elapsed = cut.at.difference(&state.now);
    transport::advance(state, &arm.content, elapsed, &inputs)
        .into_iter()
        .filter_map(|output| match output {
            transport::Output::Publication {
                at, line, reply, ..
            } => Some(encode(line, reply).map(|byte| Publication { line, byte, at })),
            transport::Output::Clock { .. }
            | transport::Output::Observation { .. }
            | transport::Output::Staged { .. } => None,
        })
        .collect()
}

fn intervene(state: &mut State, arm: &Arm, cut: &Cut) -> Result<(), Defect> {
    if !arm.expire || cut.index != 64 {
        return Ok(());
    }
    if matches!(
        state.phase,
        AudioPhase::Work {
            result: CompletionResult::Music(_),
            ..
        }
    ) {
        return Err(Defect::Admission("expiry during pending music"));
    }
    for track in &mut state.song.tracks {
        match &mut track.position {
            TrackPosition::Repeated { remaining, .. } => *remaining = NonZeroU8::MIN,
            TrackPosition::Root { .. } => return Err(Defect::Admission("expiry outside phrase")),
        }
    }
    Ok(())
}

fn first_field(expected: &[Observation], actual: &[Observation]) -> Option<String> {
    expected.iter().find_map(|expected| {
        (!actual.iter().any(|actual| actual == expected)).then(|| expected.field.clone())
    })
}

fn physical(expected: &[Publication], actual: &[Publication], offset: usize) -> Result<(), Defect> {
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        let field = if expected.line != actual.line {
            Some("group")
        } else if expected.byte != actual.byte {
            Some("byte")
        } else if expected.at != actual.at {
            Some("instant")
        } else {
            None
        };
        if let Some(field) = field {
            return Err(Defect::Publication(offset.saturating_add(index), field));
        }
    }
    if expected.len() != actual.len() {
        return Err(Defect::Publication(
            offset.saturating_add(expected.len().min(actual.len())),
            "count",
        ));
    }
    Ok(())
}

/// Advance one loaded native seed continuously and replay every saved cut to the horizon.
///
/// Expected fields carry only the declared native writer exclusions; the model is never used
/// to select masks. Inactive ramp configuration is absent from the admitted projection.
///
/// # Errors
/// Returns the first structured difference or a malformed fixture/codec defect.
pub fn run(arm: &Arm, encode: &EncodeReply<'_>, project: Project) -> Result<Counts, Defect> {
    if arm.cuts.len() != 184
        || arm.publications.len() != 764
        || arm.inputs.iter().any(|input| input.at <= arm.seed.now)
        || arm.inputs.windows(2).any(|pair| {
            pair.first()
                .zip(pair.last())
                .is_some_and(|(first, last)| first.at > last.at)
        })
    {
        return Err(Defect::Admission("frozen arm dimensions or input order"));
    }
    let mut state = arm.seed.clone();
    let mut replies = arm.initial_replies;
    let mut publications = Vec::new();
    let mut snapshots = Vec::new();
    let mut counts = Counts::default();
    let null = project(&arm.seed);
    for (index, cut) in arm.cuts.iter().enumerate() {
        if cut.index != index {
            return Err(Defect::Admission("cut identity"));
        }
        let added = interval(&mut state, arm, cut, encode)?;
        for publication in &added {
            if let Some(reply) = replies.get_mut(slot(publication.line)) {
                *reply = publication.byte;
            }
        }
        publications.extend(added);
        if publications.len() != cut.publications {
            return Err(Defect::Cut(index, String::from("publication count")));
        }
        let expected = arm
            .publications
            .get(..cut.publications)
            .ok_or(Defect::Admission("cut publication count"))?;
        physical(expected, &publications, 0)?;
        if replies != cut.replies {
            return Err(Defect::Cut(index, String::from("physical replies")));
        }
        let pending = matches!(
            state.phase,
            AudioPhase::Work {
                result: CompletionResult::Music(_),
                ..
            }
        );
        if pending != cut.pending_music {
            return Err(Defect::Cut(index, String::from("pending music")));
        }
        let actual = project(&state);
        if let Some(field) = first_field(&cut.fields, &actual) {
            return Err(Defect::Cut(index, field));
        }
        counts.fields = counts.fields.saturating_add(cut.fields.len());
        counts.null_fields = counts.null_fields.saturating_add(
            cut.fields
                .iter()
                .filter(|field| null.contains(field))
                .count(),
        );
        if first_field(&cut.fields, &null).is_none()
            && cut.publications == 0
            && cut.replies == arm.initial_replies
            && !cut.pending_music
        {
            counts.null_cuts = counts.null_cuts.saturating_add(1);
        }
        counts.compared = counts.compared.saturating_add(1);
        counts.replies = counts.replies.saturating_add(4);
        let encoded = toml::to_string(&state).map_err(|_| Defect::Codec)?;
        let restored: State = toml::from_str(&encoded).map_err(|_| Defect::Codec)?;
        if restored != state {
            return Err(Defect::Continuation(index));
        }
        snapshots.push((restored, publications.len()));
        intervene(&mut state, arm, cut)?;
    }
    physical(&arm.publications, &publications, 0)?;
    counts.publications = publications.len();
    for (start, (mut restored, offset)) in snapshots.into_iter().enumerate() {
        let saved = arm.cuts.get(start).ok_or(Defect::Admission("saved cut"))?;
        intervene(&mut restored, arm, saved)?;
        let mut suffix = Vec::new();
        for cut in arm.cuts.iter().skip(start.saturating_add(1)) {
            suffix.extend(interval(&mut restored, arm, cut, encode)?);
            if first_field(&cut.fields, &project(&restored)).is_some() {
                return Err(Defect::Continuation(start));
            }
            intervene(&mut restored, arm, cut)?;
        }
        let expected = publications
            .get(offset..)
            .ok_or(Defect::Admission("saved suffix"))?;
        physical(expected, &suffix, offset)?;
        if restored != state {
            return Err(Defect::Continuation(start));
        }
        counts.restorations = counts.restorations.saturating_add(1);
    }
    Ok(counts)
}
