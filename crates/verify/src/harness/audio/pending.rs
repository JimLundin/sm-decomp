//! Temporal publication proof. Native instants select the stops; saved semantic values prove
//! retention, not an independently decoded oracle history at an unpublished source frontier.

use super::{Arm, Defect, EncodeReply, Publication};
use audio_transport::{AudioPhase, CompletionResult, ReplyLine, State, time, transport};

#[derive(Clone)]
struct Stream {
    state: State,
    published: State,
    publications: Vec<Publication>,
    expired: bool,
}

/// Native effect boundaries whose complete pending State survived codec and split advances.
pub struct Counts {
    pub boundaries: usize,
    pub restorations: usize,
}

fn advance(
    stream: &mut Stream,
    arm: &Arm,
    stop: &audio_transport::AudioInstant,
    encode: &EncodeReply<'_>,
) -> Result<(), Defect> {
    let inputs: Vec<_> = arm
        .inputs
        .iter()
        .filter(|input| input.at > stream.state.now && input.at <= *stop)
        .cloned()
        .collect();
    let elapsed = stop.difference(&stream.state.now);
    for output in transport::advance(&mut stream.state, &arm.content, elapsed, &inputs) {
        if let transport::Output::Publication {
            at,
            line,
            reply,
            result,
        } = output
        {
            match result {
                CompletionResult::Effect {
                    group,
                    result,
                    echo_voices,
                } => {
                    stream.published.groups[group] = result;
                    stream.published.song.mix.echo_voices = echo_voices;
                }
                CompletionResult::Music(song) => stream.published.song = song,
            }
            stream.publications.push(Publication {
                at,
                line,
                byte: encode(line, reply)?,
            });
        }
    }
    let expected = arm
        .publications
        .get(..stream.publications.len())
        .ok_or(Defect::Admission("pending publication horizon"))?;
    super::physical(expected, &stream.publications, 0)
}

fn advance_to(
    stream: &mut Stream,
    arm: &Arm,
    stop: &audio_transport::AudioInstant,
    encode: &EncodeReply<'_>,
) -> Result<(), Defect> {
    let expiry = arm
        .cuts
        .get(64)
        .ok_or(Defect::Admission("pending expiry cut"))?;
    if arm.expire && !stream.expired && expiry.at <= *stop {
        advance(stream, arm, &expiry.at, encode)?;
        super::intervene(&mut stream.state, arm, expiry)?;
        stream.expired = true;
    }
    advance(stream, arm, stop, encode)
}

fn committed(stream: &Stream, index: usize) -> Result<(), Defect> {
    for (group, saved) in &stream.state.groups {
        let published = &stream.published.groups[group];
        // Stable request sampling may update staged between publications. Playback/history,
        // handled and reply instead belong to the completed effect result.
        if saved.history != published.history
            || saved.playback != published.playback
            || saved.handled != published.handled
            || saved.reply != published.reply
        {
            return Err(Defect::Pending(
                index,
                "committed effect before publication",
            ));
        }
    }
    if stream.state.song.mix.echo_voices != stream.published.song.mix.echo_voices {
        return Err(Defect::Pending(index, "committed echo before publication"));
    }
    Ok(())
}

pub fn run(arm: &Arm, encode: &EncodeReply<'_>) -> Result<Counts, Defect> {
    let mut stream = Stream {
        state: arm.seed.clone(),
        published: arm.seed.clone(),
        publications: Vec::new(),
        expired: false,
    };
    let mut counts = Counts {
        boundaries: 0,
        restorations: 0,
    };
    for (index, expected) in arm.publications.iter().enumerate() {
        let before = expected.at.difference(&time::cycles(1));
        advance_to(&mut stream, arm, &before, encode)?;
        if stream.publications.len() != index {
            return Err(Defect::Pending(index, "publication before native deadline"));
        }
        let phase = stream.state.phase.clone();
        let line = match &phase {
            AudioPhase::Work { at, result } if *at == expected.at => match result {
                CompletionResult::Effect { group, .. } => ReplyLine::Group(*group),
                CompletionResult::Music(_) => ReplyLine::Music,
            },
            AudioPhase::ObserveClock { .. }
            | AudioPhase::Distribute { .. }
            | AudioPhase::Work { .. }
            | AudioPhase::Sample { .. }
            | AudioPhase::MusicStart { .. }
            | AudioPhase::Idle { .. } => {
                return Err(Defect::Pending(index, "native pending deadline"));
            }
        };
        if line != expected.line {
            return Err(Defect::Pending(index, "native pending group"));
        }
        committed(&stream, index)?;
        let encoded = toml::to_string(&stream.state).map_err(|_| Defect::Codec)?;
        let restored: State = toml::from_str(&encoded).map_err(|_| Defect::Codec)?;
        if restored != stream.state {
            return Err(Defect::Pending(index, "codec pending result"));
        }
        let mut resumed = stream.clone();
        resumed.state = restored;
        let zero = resumed.state.clone();
        if !transport::advance(&mut resumed.state, &arm.content, time::cycles(0), &[]).is_empty()
            || resumed.state != zero
        {
            return Err(Defect::Pending(index, "zero elapsed pending state"));
        }
        let within = time::add(&before, time::halves(1));
        advance_to(&mut stream, arm, &within, encode)?;
        advance_to(&mut resumed, arm, &within, encode)?;
        committed(&stream, index)?;
        if stream.state.phase != phase
            || resumed.state != stream.state
            || stream.publications.len() != index
            || resumed.publications != stream.publications
        {
            return Err(Defect::Pending(index, "retained pending result"));
        }
        advance_to(&mut stream, arm, &expected.at, encode)?;
        advance_to(&mut resumed, arm, &expected.at, encode)?;
        committed(&stream, index)?;
        if stream.publications.len() != index.saturating_add(1)
            || resumed.state != stream.state
            || resumed.publications != stream.publications
        {
            return Err(Defect::Pending(index, "restored publication once"));
        }
        if matches!(line, ReplyLine::Group(_)) {
            counts.boundaries = counts.boundaries.saturating_add(1);
            counts.restorations = counts.restorations.saturating_add(1);
        }
    }
    Ok(counts)
}
