//! Native deadline checkpoints use the same production entry and complete saved state.

use super::result::Outcome;
use super::{
    AudioInputReader, actual_audio_from, audio, compared, data_failure,
    publication_observations_range,
};
use audio_transport::{AudioPhase, CompletionResult, ReplyLine, State, time};
use game::production::{Production, ProductionContent, ProductionOutputs, ProductionRules};

#[derive(Clone)]
struct Stream {
    production: Production,
    committed: State,
    publications: Vec<audio::Publication>,
    replies: [u8; 4],
}

impl Stream {
    fn state(&self) -> Result<&State, Outcome> {
        self.production
            .transport()
            .ok_or_else(|| data_failure(None, "pending audio scope"))
    }

    fn advance(
        &mut self,
        input: &AudioInputReader<'_>,
        stop: &audio_transport::AudioInstant,
        cut: u64,
    ) -> Result<(), Outcome> {
        let ProductionOutputs::Audio(outputs) = input.advance(&mut self.production, cut, stop)?
        else {
            return Err(data_failure(Some(cut), "pending audio output scope"));
        };
        for output in outputs {
            if let audio_transport::transport::Output::Publication {
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
                        self.committed.groups[group] = result;
                        self.committed.song.mix.echo_voices = echo_voices;
                    }
                    CompletionResult::Music(song) => self.committed.song = song,
                }
                let byte = audio::encode(&input.tape.aliases, line, reply)
                    .map_err(|_| data_failure(Some(cut), "pending physical alias"))?;
                *self
                    .replies
                    .get_mut(audio::reply_slot(line))
                    .ok_or_else(|| data_failure(Some(cut), "pending reply line"))? = byte;
                self.publications
                    .push(audio::Publication { line, byte, at });
            }
        }
        Ok(())
    }

    fn retained(&self, cut: u64) -> Result<(), Outcome> {
        let state = self.state()?;
        for (group, saved) in &state.groups {
            let published = &self.committed.groups[group];
            if saved.history != published.history
                || saved.playback != published.playback
                || saved.handled != published.handled
                || saved.reply != published.reply
            {
                return Err(Outcome::Difference {
                    cut,
                    field: "audio.committed_effect_before_publication".to_owned(),
                });
            }
        }
        if state.song.mix.echo_voices != self.committed.song.mix.echo_voices {
            return Err(Outcome::Difference {
                cut,
                field: "audio.committed_echo_before_publication".to_owned(),
            });
        }
        if state.song != self.committed.song {
            return Err(Outcome::Difference {
                cut,
                field: "audio.committed_song_before_publication".to_owned(),
            });
        }
        Ok(())
    }
}

fn pending_line(
    phase: &AudioPhase,
    deadline: &audio_transport::AudioInstant,
    cut: u64,
) -> Result<ReplyLine, Outcome> {
    let line = match phase {
        AudioPhase::Work { at, result } if at == deadline => match result {
            CompletionResult::Effect { group, .. } => ReplyLine::Group(*group),
            CompletionResult::Music(_) => ReplyLine::Music,
        },
        AudioPhase::ObserveClock { .. }
        | AudioPhase::Distribute { .. }
        | AudioPhase::Work { .. }
        | AudioPhase::Sample { .. }
        | AudioPhase::MusicStart { .. }
        | AudioPhase::Idle { .. } => {
            return Err(Outcome::Difference {
                cut,
                field: "audio.native_pending_deadline".to_owned(),
            });
        }
    };
    Ok(line)
}

pub(super) fn run(
    production: Production,
    content: &ProductionContent,
    rules: &ProductionRules,
    tape: &audio::InputTape,
    expected: &[audio::Publication],
) -> Result<(usize, usize), Outcome> {
    let mut stream = Stream {
        production,
        committed: tape.seed.clone(),
        publications: Vec::new(),
        replies: tape.initial_replies,
    };
    let inputs = AudioInputReader { tape };
    let mut boundaries: usize = 0;
    let mut restorations: usize = 0;
    for (index, deadline) in tape.deadlines.iter().enumerate() {
        let cut =
            u64::try_from(index).map_err(|_| data_failure(None, "pending boundary identity"))?;
        let before = deadline.difference(&time::cycles(1));
        stream.advance(&inputs, &before, cut)?;
        if stream.publications.len() != index {
            return Err(Outcome::Difference {
                cut,
                field: "audio.publication_before_native_deadline".to_owned(),
            });
        }
        let phase = stream.state()?.phase.clone();
        let line = pending_line(&phase, deadline, cut)?;
        stream.retained(cut)?;
        let saved = stream.production.save().map_err(
            |game::production::SaveFault::Unsupported(scope)| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("checkpoint codec for {scope:?}"),
            },
        )?;
        let mut resumed = stream.clone();
        resumed.production = Production::restore(&saved, content.clone(), rules.clone())
            .map_err(|_| Outcome::CheckpointFailure { cut })?;
        let before_zero = resumed.production.state().clone();
        let outputs = inputs.advance(&mut resumed.production, cut, &before)?;
        if outputs != ProductionOutputs::Audio(Vec::new())
            || resumed.production.state() != &before_zero
        {
            return Err(Outcome::Difference {
                cut,
                field: "audio.zero_elapsed_pending_state".to_owned(),
            });
        }
        let within = time::add(&before, time::halves(1));
        stream.advance(&inputs, &within, cut)?;
        resumed.advance(&inputs, &within, cut)?;
        stream.retained(cut)?;
        if stream.state()?.phase != phase
            || resumed.production != stream.production
            || stream.publications.len() != index
            || resumed.publications != stream.publications
        {
            return Err(Outcome::Difference {
                cut,
                field: "audio.retained_pending_result".to_owned(),
            });
        }
        stream.advance(&inputs, deadline, cut)?;
        resumed.advance(&inputs, deadline, cut)?;
        stream.retained(cut)?;
        if stream.publications.len() != index.saturating_add(1)
            || resumed.production != stream.production
            || resumed.publications != stream.publications
        {
            return Err(Outcome::Difference {
                cut,
                field: "audio.restored_publication_once".to_owned(),
            });
        }
        let horizon = expected
            .get(index..stream.publications.len())
            .ok_or_else(|| data_failure(Some(cut), "pending expected horizon"))?;
        compared(
            cut,
            &publication_observations_range(horizon, stream.publications.len(), index)?,
            &actual_audio_from(
                &stream.production,
                &stream.publications,
                stream.replies,
                index,
            )?,
        )?;
        if matches!(line, ReplyLine::Group(_)) {
            boundaries = boundaries.saturating_add(1);
            restorations = restorations.saturating_add(1);
        }
    }
    Ok((boundaries, restorations))
}
