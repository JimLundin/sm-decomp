//! The sole carried-state comparison loop, also used for every saved continuation.

use super::compare::Observation;
use super::index::Case;
use super::readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader};
use super::result::{CaseResult, Outcome};
use super::{append_digest, compared, data_failure};
use game::production::{
    Production, ProductionContent, ProductionRules, SaveFault, SavedProduction,
};

fn save_cut(production: &Production, cut: u64) -> Result<SavedProduction, Outcome> {
    production
        .save()
        .map_err(|SaveFault::Unsupported(scope)| Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("checkpoint codec for {scope:?}"),
        })
}

fn count(counter: &mut usize, amount: usize) -> Result<(), Outcome> {
    *counter = counter
        .checked_add(amount)
        .ok_or_else(|| data_failure(None, "receipt count range"))?;
    Ok(())
}

#[cfg(test)]
pub(super) fn execute(
    case: &Case,
    inputs: &InputReader,
    expected: &ExpectedReader,
    production: &mut game::production::Production,
    actual: &mut ActualReader,
    cut: u64,
) -> Result<ExpectedCut, Outcome> {
    CutReader {
        case,
        inputs,
        expected,
    }
    .execute(production, actual, cut)
}

struct CutReader<'a> {
    case: &'a Case,
    inputs: &'a InputReader,
    expected: &'a ExpectedReader,
}

impl CutReader<'_> {
    fn execute(
        &self,
        production: &mut Production,
        actual: &mut ActualReader,
        cut: u64,
    ) -> Result<ExpectedCut, Outcome> {
        let output = self
            .inputs
            .advance(production, cut)
            .map_err(|outcome| at_cut(outcome, cut))?;
        let observed = actual.accept(self.inputs, production, output, cut)?;
        let expected = self
            .expected
            .read(self.case, cut)
            .map_err(|outcome| at_cut(outcome, cut))?;
        compared(cut, &expected.fields, &observed)?;
        Ok(expected)
    }
}

fn at_cut(outcome: Outcome, current: u64) -> Outcome {
    match outcome {
        Outcome::Unsupported { cut, dependency } => Outcome::Unsupported {
            cut: cut.or(Some(current)),
            dependency,
        },
        Outcome::DataFailure { cut, source } => Outcome::DataFailure {
            cut: cut.or(Some(current)),
            source,
        },
        Outcome::Difference { cut, field } => Outcome::Difference { cut, field },
        Outcome::CheckpointFailure { cut } => Outcome::CheckpointFailure { cut },
        Outcome::Pass => Outcome::Pass,
    }
}

pub(super) fn record_cut(
    result: &mut CaseResult,
    digest: &mut Vec<u8>,
    expected: &ExpectedCut,
    null: &[Observation],
    cut: u64,
) -> Result<(), Outcome> {
    count(&mut result.counts.cuts, 1)?;
    count(&mut result.counts.fields, expected.semantic_count)?;
    count(&mut result.counts.outputs, expected.output_count)?;
    count(&mut result.counts.replies, expected.replies)?;
    count(
        &mut result.counts.null_fields,
        expected
            .semantics
            .iter()
            .filter(|field| null.contains(field))
            .count(),
    )?;
    if super::compare::compare(&expected.fields, null).is_ok() {
        count(&mut result.counts.null_cuts, 1)?;
    }
    append_digest(digest, cut, &expected.fields);
    Ok(())
}

pub(super) fn run(
    case: &Case,
    admitted: AdmittedCase,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    if admitted.schedule.last() != Some(&case.end.cut_inclusive) {
        return Err(data_failure(None, "complete declared comparison schedule"));
    }
    let AdmittedCase {
        mut production,
        mut actual,
        content,
        rules,
        inputs,
        expected: expectation,
        schedule,
    } = admitted;
    let reader = CutReader {
        case,
        inputs: &inputs,
        expected: &expectation,
    };
    let null = actual.initial(&production)?;
    let mut checkpoints = Vec::new();
    let mut origin = origin_checkpoint(case, &inputs, &production, &actual)?;
    if matches!(actual, ActualReader::StandingBody | ActualReader::ShaftHud) {
        // Restore the admitted picture, then every nonterminal continuation.
        checkpoints.extend(origin.take());
    }
    let mut digest = Vec::new();
    let mut stop = None;
    let mut compare_cuts = || -> Result<(), Outcome> {
        for cut in &schedule {
            let expected = match reader.execute(&mut production, &mut actual, *cut) {
                Ok(compared) => compared,
                Err(outcome @ Outcome::Unsupported { .. }) if !checkpoints.is_empty() => {
                    stop = Some((*cut, outcome));
                    break;
                }
                Err(outcome) => return Err(outcome),
            };
            record_cut(result, &mut digest, &expected, &null, *cut)?;
            if case.checkpoints {
                if pending_write(&production) {
                    count(&mut result.counts.pending_boundaries, 1)?;
                }
                if checkpoints.len() < schedule.len() {
                    // A one-opportunity floor witness resumes across the actual advance.
                    checkpoints.push(
                        origin
                            .take()
                            .map_or_else(|| capture_checkpoint(&production, &actual, *cut), Ok)?,
                    );
                }
            }
        }
        Ok(())
    };
    let comparison = compare_cuts();
    // The receipt covers the matching prefix even when a later cut stops the run.
    if !digest.is_empty() {
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    }
    comparison?;
    verify_checkpoints(
        CheckpointProof {
            reader: &reader,
            content: &content,
            rules: &rules,
            production: &production,
            actual: &actual,
            schedule: &schedule,
        },
        &checkpoints,
        stop.as_ref(),
        result,
    )?;
    if let Some((_, outcome)) = stop {
        return Err(outcome);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct CheckpointProof<'a> {
    reader: &'a CutReader<'a>,
    content: &'a ProductionContent,
    rules: &'a ProductionRules,
    production: &'a Production,
    actual: &'a ActualReader,
    schedule: &'a [u64],
}

fn verify_checkpoints(
    proof: CheckpointProof<'_>,
    checkpoints: &[CutCheckpoint],
    stop: Option<&(u64, Outcome)>,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let CheckpointProof {
        reader,
        content,
        rules,
        production,
        actual,
        schedule,
    } = proof;
    let mut joined = std::collections::BTreeMap::new();
    for (index, (start, saved, projection)) in checkpoints.iter().enumerate() {
        if let Some(pending) = joined.remove(&index) {
            credit_restoration(result, pending)?;
            continue;
        }
        let mut restored_actual = projection.clone();
        let failed = Outcome::CheckpointFailure { cut: *start };
        let mut restored = Production::restore(saved, content.clone(), rules.clone())
            .map_err(|_| failed.clone())?;
        let pending_save = pending_write(&restored);
        let compared_suffix = schedule
            .iter()
            .filter(|cut| *cut > start)
            .take_while(|cut| stop.is_none_or(|(end, _)| *cut != end));
        let mut later = checkpoints
            .iter()
            .enumerate()
            .skip(index.saturating_add(1))
            .peekable();
        for cut in compared_suffix {
            let resumed = reader.execute(&mut restored, &mut restored_actual, *cut);
            if stop.is_some() {
                resumed.map_err(|_| failed.clone())?;
            } else {
                resumed?;
            }
            if let Some((ordinal, (_, next_save, next_actual))) =
                later.next_if(|(_, (at, _, _))| at == cut)
                && &restored_actual == next_actual
                && let Ok(next) = Production::restore(next_save, content.clone(), rules.clone())
                && next == restored
            {
                joined.insert(ordinal, pending_write(&next));
                restored = next;
                restored_actual = next_actual.clone();
            }
        }
        if let Some((cut, direct_outcome)) = stop {
            let resumed = reader.execute(&mut restored, &mut restored_actual, *cut);
            if resumed.as_ref().err() != Some(direct_outcome)
                || &restored != production
                || !restored_actual.same_retained_outputs(actual)
            {
                return Err(failed);
            }
        }
        if restored.state() != production.state()
            || !touchdown_outputs_hold(reader.inputs, &restored_actual, actual)
        {
            return Err(failed);
        }
        credit_restoration(result, pending_save)?;
    }
    Ok(())
}

fn credit_restoration(result: &mut CaseResult, pending: bool) -> Result<(), Outcome> {
    if pending {
        count(&mut result.counts.pending_restorations, 1)?;
    }
    count(&mut result.counts.restorations, 1)
}

fn pending_write(production: &Production) -> bool {
    use game::state::owned::OwnedState;
    let cartridge = if let OwnedState::SaveFiles(state) = production.state() {
        &state.cartridge
    } else if let OwnedState::Blackout(state) = production.state() {
        &state.owners.0.files.cartridge
    } else {
        return false;
    };
    matches!(
        cartridge.pending,
        types::save_files::PendingAction::Write { .. }
    )
}

type CutCheckpoint = (u64, SavedProduction, ActualReader);

fn capture_checkpoint(
    production: &Production,
    actual: &ActualReader,
    cut: u64,
) -> Result<CutCheckpoint, Outcome> {
    Ok((cut, save_cut(production, cut)?, actual.clone()))
}

fn origin_checkpoint(
    case: &Case,
    inputs: &InputReader,
    production: &Production,
    actual: &ActualReader,
) -> Result<Option<CutCheckpoint>, Outcome> {
    use super::index::Start;
    if !case.checkpoints {
        return Ok(None);
    }
    if let (InputReader::Departure(_), Start::PlayedContinuation { frame, .. }) =
        (inputs, &case.start)
    {
        return capture_checkpoint(production, actual, *frame).map(Some);
    }
    let Start::CacheCut { frame } = &case.start else {
        return Ok(None);
    };
    if matches!(actual, ActualReader::StandingBody | ActualReader::ShaftHud)
        || matches!(
            inputs,
            InputReader::MovementTouchdown(_)
                | InputReader::ShaftEntry(_)
                | InputReader::Zebes(_)
                | InputReader::Explosion(_)
        )
        || case.id.0 == super::jump_history::LEAP_CASE
    {
        capture_checkpoint(production, actual, *frame).map(Some)
    } else {
        Ok(None)
    }
}

fn touchdown_outputs_hold(
    inputs: &InputReader,
    restored: &ActualReader,
    direct: &ActualReader,
) -> bool {
    !matches!(inputs, InputReader::MovementTouchdown(_)) || restored.same_retained_outputs(direct)
}

#[cfg(test)]
mod tests {
    use super::execute;
    use crate::replay::{
        Sources, admission, index::Index, readers::ExpectedReader, result::Outcome,
    };

    #[test]
    fn an_early_difference_keeps_the_matching_prefix_digest() {
        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let mut case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "sender-offer/ordered-pair")
            .unwrap()
            .clone();
        case.checkpoints = false;
        let sources = Sources::default();
        let complete = crate::replay::run(&case, &sources);
        assert!(complete.passed());

        let mut two_cuts = case.clone();
        two_cuts.end.cut_inclusive = 1;
        let mut admitted = admission::admit(&case, &sources).unwrap();
        let super::InputReader::SenderOffer(offers) = &mut admitted.inputs else {
            panic!("sender offer input");
        };
        // The first cut still accepts both offers. The higher allowance makes the second cut
        // accept both again, so only the first cut's expectation matches.
        for offer in offers {
            offer.allowance = types::audio::CueAllowance::Nine;
        }
        admitted.schedule.push(1);
        let mut partial = complete.clone();
        partial.end_cut = 1;
        partial.counts = crate::replay::result::Counts::default();
        partial.observation_sha256.clear();

        assert_eq!(
            super::run(&two_cuts, admitted, &mut partial),
            Err(Outcome::Difference {
                cut: 1,
                field: "sender.queue".to_owned(),
            }),
        );
        assert_eq!(partial.counts, complete.counts);
        assert_eq!(partial.observation_sha256, complete.observation_sha256);
    }

    #[test]
    fn a_divergent_checkpoint_runs_its_own_suffix_in_capture_order() {
        use super::{CheckpointProof, CutReader, capture_checkpoint, verify_checkpoints};
        use crate::replay::readers::ActualReader;
        use crate::replay::result::Counts;

        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let find = |name| index.cases.iter().find(|case| case.id.0 == name).unwrap();
        let case = find("sender-offer/ordered-pair");
        let sources = Sources::default();
        let mut admitted = admission::admit(case, &sources).unwrap();
        let other = admission::admit(find("sender-offer/deny-smaller-limit"), &sources).unwrap();
        let reader = CutReader {
            case,
            inputs: &admitted.inputs,
            expected: &admitted.expected,
        };
        let mut checkpoints = Vec::new();
        for cut in 0..3 {
            reader
                .execute(&mut admitted.production, &mut admitted.actual, cut)
                .unwrap();
            checkpoints
                .push(capture_checkpoint(&admitted.production, &admitted.actual, cut).unwrap());
        }
        let proof = CheckpointProof {
            reader: &reader,
            content: &admitted.content,
            rules: &admitted.rules,
            production: &admitted.production,
            actual: &admitted.actual,
            schedule: &[0, 1, 2],
        };
        let mut result = crate::replay::run(case, &sources);
        result.counts = Counts::default();
        verify_checkpoints(proof, &checkpoints, None, &mut result).unwrap();
        assert_eq!(result.counts.restorations, 3);
        let original = checkpoints[1].clone();
        for middle in [
            capture_checkpoint(&other.production, &admitted.actual, 1).unwrap(),
            (
                1,
                original.1,
                ActualReader::SenderOffer(audio_transport::SoundGroup::Action),
            ),
        ] {
            checkpoints[1] = middle;
            result.counts = Counts::default();
            assert_eq!(
                verify_checkpoints(proof, &checkpoints, None, &mut result),
                Err(Outcome::Difference {
                    cut: 2,
                    field: "sender.queue".to_owned()
                })
            );
            assert_eq!(result.counts.restorations, 1);
        }
    }

    #[test]
    #[ignore = "private pinned native capture"]
    fn native_expected_mutation_cannot_change_carried_production() {
        let index = Index::parse(include_str!("../../../../scenarios/replay/index.toml")).unwrap();
        let mut case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "audio/native-control")
            .unwrap()
            .clone();
        case.end.cut_inclusive = 1;
        case.checkpoints = false;
        let sources = Sources {
            native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
            rom: std::env::var_os("SM_ROM").unwrap().into(),
            ..Sources::default()
        };
        let mut reference = admission::admit(&case, &sources).unwrap();
        let mut altered = admission::admit(&case, &sources).unwrap();
        for admitted in [&mut reference, &mut altered] {
            execute(
                &case,
                &admitted.inputs,
                &admitted.expected,
                &mut admitted.production,
                &mut admitted.actual,
                0,
            )
            .unwrap();
        }
        let ExpectedReader::Audio(tape) = &mut altered.expected else {
            panic!()
        };
        let observed = tape
            .cuts
            .iter_mut()
            .find(|(cut, _)| *cut == 1)
            .unwrap()
            .1
            .fields
            .first_mut()
            .unwrap();
        observed.bytes[0] ^= 1;
        let field = observed.field.clone();
        execute(
            &case,
            &reference.inputs,
            &reference.expected,
            &mut reference.production,
            &mut reference.actual,
            1,
        )
        .unwrap();
        assert!(
            matches!(execute(&case, &altered.inputs, &altered.expected, &mut altered.production, &mut altered.actual, 1),
            Err(Outcome::Difference { cut: 1, field: actual }) if actual == field)
        );
        assert!(altered.production.state().eq(reference.production.state()));
        let ExpectedReader::Audio(tape) = &mut altered.expected else {
            panic!()
        };
        tape.cuts.retain(|(cut, _)| *cut != 1);
        assert!(matches!(
            altered.expected.read(&case, 1),
            Err(Outcome::DataFailure { cut: Some(1), .. })
        ));
    }
}
