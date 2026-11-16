//! The sole carried-state comparison loop, also used for every saved continuation.

use super::index::Case;
use super::readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader};
use super::result::{CaseResult, Outcome};
use super::{append_digest, compared, data_failure, pending};

fn count(counter: &mut usize, amount: usize) -> Result<(), Outcome> {
    *counter = counter
        .checked_add(amount)
        .ok_or_else(|| data_failure(None, "receipt count range"))?;
    Ok(())
}

fn execute(
    case: &Case,
    inputs: &InputReader,
    expected: &ExpectedReader,
    production: &mut game::production::Production,
    actual: &mut ActualReader,
    cut: u64,
) -> Result<ExpectedCut, Outcome> {
    let output = inputs
        .advance(production, cut)
        .map_err(|outcome| at_cut(outcome, cut))?;
    let observed = actual.accept(inputs, production, output, cut)?;
    let expected = expected
        .read(case, cut)
        .map_err(|outcome| at_cut(outcome, cut))?;
    compared(cut, &expected.fields, &observed)?;
    Ok(expected)
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

pub(super) fn run(
    case: &Case,
    admitted: AdmittedCase,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    if admitted.schedule.is_empty() || admitted.schedule.last() != Some(&case.end.cut_inclusive) {
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
    let null = actual.initial(&production)?;
    let mut checkpoints = Vec::new();
    let mut digest = Vec::new();
    for cut in &schedule {
        let expected = execute(
            case,
            &inputs,
            &expectation,
            &mut production,
            &mut actual,
            *cut,
        )?;
        count(&mut result.counts.cuts, 1)?;
        count(&mut result.counts.fields, expected.semantic_count)?;
        count(&mut result.counts.outputs, expected.output_count)?;
        let publications = actual.publication_count();
        let new_publications = publications
            .checked_sub(result.counts.publications)
            .ok_or_else(|| data_failure(Some(*cut), "publication prefix count"))?;
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
        if super::compare::compare(&expected.fields, &null).is_ok() {
            count(&mut result.counts.null_cuts, 1)?;
        }
        append_digest(&mut digest, *cut, &expected.fields);
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
        if case.checkpoints {
            let saved =
                production
                    .save()
                    .map_err(|game::production::SaveFault::Unsupported(scope)| {
                        Outcome::Unsupported {
                            cut: Some(*cut),
                            dependency: format!("checkpoint codec for {scope:?}"),
                        }
                    })?;
            checkpoints.push((*cut, saved, actual.clone()));
        }
    }
    for (start, saved, mut restored_actual) in checkpoints {
        let mut restored =
            game::production::Production::restore(&saved, content.clone(), rules.clone())
                .map_err(|_| Outcome::CheckpointFailure { cut: start })?;
        for cut in schedule.iter().filter(|cut| **cut > start) {
            execute(
                case,
                &inputs,
                &expectation,
                &mut restored,
                &mut restored_actual,
                *cut,
            )?;
        }
        if restored.state() != production.state()
            || restored_actual.publication_count() != actual.publication_count()
        {
            return Err(Outcome::CheckpointFailure { cut: start });
        }
        count(&mut result.counts.restorations, 1)?;
    }
    if let InputReader::Audio(tape) = &inputs
        && case.checkpoints
    {
        let ExpectedReader::Audio(expected) = &expectation else {
            return Err(data_failure(None, "audio expected scope"));
        };
        let pending = pending::run(tape, &expected.publications)?;
        result.counts.pending_boundaries = pending.0;
        result.counts.pending_restorations = pending.1;
    }
    result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
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
