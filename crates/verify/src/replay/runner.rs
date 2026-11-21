//! The sole carried-state comparison loop, also used for every saved continuation.

use super::compare::Observation;
use super::frame::HeldAdvance;
use super::index::Case;
use super::readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader};
use super::result::{CaseResult, Outcome};
use super::{append_digest, compared, data_failure, pending};
use game::production::{
    Production, ProductionContent, ProductionRules, SaveFault, SavedCompletedFrame, SavedProduction,
};

enum SavedCut {
    Frame(SavedCompletedFrame),
    Scoped(SavedProduction),
}

fn save_cut(inputs: &InputReader, production: &Production, cut: u64) -> Result<SavedCut, Outcome> {
    if matches!(inputs, InputReader::Frame(_)) {
        return production
            .save_completed_frame()
            .map(SavedCut::Frame)
            .map_err(|_| Outcome::CheckpointFailure { cut });
    }
    production
        .save()
        .map(SavedCut::Scoped)
        .map_err(|SaveFault::Unsupported(scope)| Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("checkpoint codec for {scope:?}"),
        })
}

fn restore_cut(
    saved: &SavedCut,
    content: ProductionContent,
    rules: ProductionRules,
    failed: Outcome,
) -> Result<Production, Outcome> {
    match saved {
        SavedCut::Frame(saved) => {
            let restored = Production::restore_completed_frame(saved, content, rules)
                .map_err(|_| failed.clone())?;
            // Later advancement can replace return eligibility, so check it at restore.
            if restored.save_completed_frame().as_ref() != Ok(saved) {
                return Err(failed);
            }
            Ok(restored)
        }
        SavedCut::Scoped(saved) => Production::restore(saved, content, rules).map_err(|_| failed),
    }
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
    .execute(production, actual, cut, &mut HeldAdvance::default())
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
        held: &mut HeldAdvance,
    ) -> Result<ExpectedCut, Outcome> {
        let output = self
            .inputs
            .advance_observing_stop(production, cut, held)
            .map_err(|outcome| at_cut(outcome, cut))?;
        let observed = actual.accept_observing_stop(self.inputs, production, output, cut, held)?;
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

fn record_cut(
    result: &mut CaseResult,
    digest: &mut Vec<u8>,
    expected: &ExpectedCut,
    null: &[Observation],
    publications: usize,
    cut: u64,
) -> Result<(), Outcome> {
    count(&mut result.counts.cuts, 1)?;
    count(&mut result.counts.fields, expected.semantic_count)?;
    count(&mut result.counts.outputs, expected.output_count)?;
    let new_publications = publications
        .checked_sub(result.counts.publications)
        .ok_or_else(|| data_failure(Some(cut), "publication prefix count"))?;
    count(&mut result.counts.outputs, new_publications)?;
    result.counts.publications = publications;
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
    result.observation_sha256 = crate::lock::Sha256::of(digest).to_string();
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
    let mut digest = Vec::new();
    let mut held = HeldAdvance::default();
    let mut stop = None;
    for cut in &schedule {
        let expected = match reader.execute(&mut production, &mut actual, *cut, &mut held) {
            Ok(compared) => compared,
            Err(outcome @ Outcome::Unsupported { .. }) if !checkpoints.is_empty() => {
                stop = Some((*cut, outcome));
                break;
            }
            Err(outcome) => return Err(outcome),
        };
        record_cut(
            result,
            &mut digest,
            &expected,
            &null,
            actual.publication_count(),
            *cut,
        )?;
        if case.checkpoints {
            let saved = save_cut(&inputs, &production, *cut)?;
            checkpoints.push((*cut, saved, actual.clone()));
        }
    }
    for (start, saved, mut restored_actual) in checkpoints {
        let failed = Outcome::CheckpointFailure { cut: start };
        let mut restored = restore_cut(&saved, content.clone(), rules.clone(), failed.clone())?;
        let compared_suffix = schedule
            .iter()
            .filter(|cut| **cut > start)
            .take_while(|cut| stop.as_ref().is_none_or(|(end, _)| *cut != end));
        for cut in compared_suffix {
            let resumed = reader.execute(
                &mut restored,
                &mut restored_actual,
                *cut,
                &mut HeldAdvance::default(),
            );
            if stop.is_some() {
                resumed.map_err(|_| failed.clone())?;
            } else {
                resumed?;
            }
        }
        if let Some((cut, direct_outcome)) = &stop {
            let mut restored_held = HeldAdvance::default();
            let resumed = reader.execute(
                &mut restored,
                &mut restored_actual,
                *cut,
                &mut restored_held,
            );
            if resumed.as_ref().err() != Some(direct_outcome)
                || restored_held != held
                || restored != production
                || !restored_actual.same_publication_prefix(&actual)
            {
                return Err(failed);
            }
        }
        if restored.state() != production.state()
            || restored_actual.publication_count() != actual.publication_count()
        {
            return Err(failed);
        }
        count(&mut result.counts.restorations, 1)?;
    }
    if let Some((_, outcome)) = stop {
        return Err(outcome);
    }
    if case.checkpoints {
        record_pending(&inputs, &expectation, result)?;
    }
    Ok(())
}

fn record_pending(
    inputs: &InputReader,
    expected: &ExpectedReader,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    if let InputReader::Audio(tape) = inputs {
        let ExpectedReader::Audio(expected) = expected else {
            return Err(data_failure(None, "audio expected scope"));
        };
        let pending = pending::run(tape, &expected.publications)?;
        result.counts.pending_boundaries = pending.0;
        result.counts.pending_restorations = pending.1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::execute;
    use crate::replay::{
        Sources, admission, index::Index, readers::ExpectedReader, result::Outcome,
    };

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
            .find(|cut| cut.index == 1)
            .unwrap()
            .fields
            .first_mut()
            .unwrap();
        observed.value = observed.value.wrapping_add(1);
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
        tape.cuts.retain(|cut| cut.index != 1);
        assert!(matches!(
            altered.expected.read(&case, 1),
            Err(Outcome::DataFailure { cut: Some(1), .. })
        ));
    }
}
