//! Source pin and selected-state admission; expected outcomes never seed production.

use super::index::{Case, ContentProvider, ExpectedProvider, InputProvider, Responsibility, Start};
use super::readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader, record};
use super::result::Outcome;
use super::{Sources, audio, data_failure, movement};
use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use game::production::{Production, ProductionContent, ProductionRules, ProductionSeed};

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    if case.inputs == InputProvider::Unavailable {
        let dependency = match case.responsibilities.as_slice() {
            [Responsibility::Blocks] => {
                "independent complete block seed and ordered reaction requests"
            }
            [Responsibility::Doors] => {
                "independent complete passage seed and opportunity-time inputs"
            }
            [Responsibility::AudioSender] => {
                "independent CPU sender seed and ordered queue/service inputs"
            }
            _ => return Err(data_failure(None, "unavailable provider scope")),
        };
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: dependency.to_owned(),
        });
    }
    if case.kind == super::index::CaseKind::Movie {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "complete production power-on and movie input pipeline".to_owned(),
        });
    }
    match case.responsibilities.as_slice() {
        [Responsibility::Audio] => audio(case, sources),
        [Responsibility::Movement] => movement(case, sources),
        [
            Responsibility::Ceres
            | Responsibility::Camera
            | Responsibility::Hud
            | Responsibility::Appearance
            | Responsibility::Progress,
        ]
        | [Responsibility::Weapons, Responsibility::Contact]
        | [Responsibility::Movement, Responsibility::Ceres] => {
            super::recorded::admit(case, sources)
        }
        _ => Err(Outcome::Unsupported {
            cut: None,
            dependency: "complete integrated production dependency closure".to_owned(),
        }),
    }
}

fn audio(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    if case.content != ContentProvider::NativeResidentContent
        || case.inputs != InputProvider::NativePhysicalPresentations
        || case.expectations.provider != ExpectedProvider::NativeDecodedCuts
    {
        return Err(data_failure(None, "audio provider contract"));
    }
    let (path, sha256, continuation) = match &case.start {
        Start::NativeSeed { path, sha256 } => (path, sha256, None),
        Start::NativeCheckpoint {
            path,
            sha256,
            seed_path,
            seed_sha256,
            frame,
        } => {
            let seed_path = if seed_path.is_absolute() {
                seed_path.clone()
            } else {
                sources.native.join(seed_path)
            };
            (path, sha256, Some((seed_path, seed_sha256, *frame)))
        }
        Start::CacheCut { .. } | Start::PowerOn => {
            return Err(data_failure(None, "audio start contract"));
        }
    };
    let path = if path.is_absolute() {
        path.clone()
    } else {
        sources.native.join(path)
    };
    let continuation = continuation
        .as_ref()
        .map(|(path, hash, frame)| (path.as_path(), hash.as_str(), *frame));
    let (input_tape, expected_tape) =
        audio::load(&path, sha256, case.end.cut_inclusive, continuation).map_err(|fault| {
            match fault {
                audio::ProviderFault::Admission(source) => data_failure(None, source),
                audio::ProviderFault::Pin => data_failure(None, "native fixture pin"),
                audio::ProviderFault::Missing => data_failure(None, "native fixture file"),
            }
        })?;
    let content = ProductionContent::Audio(input_tape.content.clone());
    let rules = ProductionRules::Audio;
    let production = Production::admit(
        ProductionSeed::Audio(Box::new(input_tape.seed.clone())),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(None, "production audio admission"))?;
    let actual = ActualReader::Audio {
        publications: Vec::new(),
        replies: input_tape.initial_replies,
    };
    let schedule = input_tape
        .stops
        .iter()
        .filter(|(cut, _)| *cut <= case.end.cut_inclusive)
        .map(|(cut, _)| *cut)
        .collect();
    Ok(AdmittedCase {
        production,
        content,
        rules,
        actual,
        schedule,
        inputs: InputReader::Audio(Box::new(input_tape)),
        expected: ExpectedReader::Audio(Box::new(expected_tape)),
    })
}

fn movement(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    if case.content != ContentProvider::PlayerRomImport
        || case.inputs != InputProvider::PrecedingCutMovementInputs
        || case.expectations.provider != ExpectedProvider::PinnedRecordCache
    {
        return Err(data_failure(None, "movement provider contract"));
    }
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "movement start contract"));
    };
    let lock = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let lock = Lock::parse(&lock).map_err(|_| data_failure(None, "record lock schema"))?;
    let row = lock
        .row(&RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "indexed lock row"))?;
    let input_cache = Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "input record cache pins"))?;
    let expected_cache = Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "expected record cache pins"))?;
    let content = movement::Content::load(&sources.rom)?;
    let seed = movement::seed(&record(&input_cache, start)?.wram)?;
    let production_content = ProductionContent::Movement(content.movement.clone());
    let rules = ProductionRules::Movement(samus_movement::Rules::default());
    let production = Production::admit(
        ProductionSeed::Movement(seed),
        production_content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(start), "movement production admission"))?;
    Ok(AdmittedCase {
        production,
        content: production_content,
        rules,
        actual: ActualReader::Movement,
        schedule: (start
            .checked_add(1)
            .ok_or_else(|| data_failure(Some(start), "movement start range"))?
            ..=case.end.cut_inclusive)
            .collect(),
        inputs: InputReader::Movement {
            cache: Box::new(input_cache),
            content: Box::new(content),
        },
        expected: ExpectedReader::Movement {
            cache: Box::new(expected_cache),
            row: case.row.clone(),
        },
    })
}
