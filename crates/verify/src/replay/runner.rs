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

fn record_cut(
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
    let mut origin = origin_checkpoint(case, &inputs, &production, &actual)?;
    let mut digest = Vec::new();
    let mut stop = None;
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
            // A one-opportunity touchdown resumes across the actual advance.
            checkpoints.push(
                origin
                    .take()
                    .map_or_else(|| capture_checkpoint(&production, &actual, *cut), Ok)?,
            );
        }
    }
    verify_checkpoints(
        CheckpointProof {
            reader: &reader,
            content: &content,
            rules: &rules,
            production: &production,
            actual: &actual,
            schedule: &schedule,
        },
        checkpoints,
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
    checkpoints: Vec<CutCheckpoint>,
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
    for (start, saved, mut restored_actual) in checkpoints {
        let failed = Outcome::CheckpointFailure { cut: start };
        let mut restored = Production::restore(&saved, content.clone(), rules.clone())
            .map_err(|_| failed.clone())?;
        let compared_suffix = schedule
            .iter()
            .filter(|cut| **cut > start)
            .take_while(|cut| stop.is_none_or(|(end, _)| *cut != end));
        for cut in compared_suffix {
            let resumed = reader.execute(&mut restored, &mut restored_actual, *cut);
            if stop.is_some() {
                resumed.map_err(|_| failed.clone())?;
            } else {
                resumed?;
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
        count(&mut result.counts.restorations, 1)?;
    }
    Ok(())
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
    let (InputReader::MovementTouchdown(_), Start::CacheCut { frame }) = (inputs, &case.start)
    else {
        return Ok(None);
    };
    capture_checkpoint(production, actual, *frame).map(Some)
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
