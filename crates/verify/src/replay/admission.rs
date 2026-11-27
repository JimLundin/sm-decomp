//! Source pin and selected-state admission; expected outcomes never seed production.

use super::index::{Case, ContentProvider, ExpectedProvider, InputProvider, Responsibility, Start};
use super::readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader, record};
use super::result::Outcome;
use super::{Sources, audio, data_failure, movement};
use game::production::{Production, ProductionContent, ProductionRules, ProductionSeed};

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    if case.inputs == InputProvider::Blackout {
        return super::blackout::admit(case, sources);
    }
    if case.inputs == InputProvider::ScopedIntroInputs {
        return super::played::intro::indexed::admit(case, sources);
    }
    if case.inputs == InputProvider::IntroRoomJoinInputs {
        return super::intro_rooms::admit(case, sources);
    }
    if case.inputs == InputProvider::FlashbackInputs {
        return super::flashback::admit(case, sources);
    }
    if case.inputs == InputProvider::SaveFileBoundaries {
        return super::save_files::admit(case, sources);
    }
    if case.inputs == InputProvider::IntroMissileInputs {
        return super::intro_missiles::admit(case, sources);
    }
    if case.inputs == InputProvider::DiscoveryInputs {
        return super::discovery::admit(case, sources);
    }
    if case.inputs == InputProvider::LandingSiteTiming {
        return super::landing_site::admit(case, sources);
    }
    if case.inputs == InputProvider::EarlyMapInputs {
        return super::early_map::admit(case, sources);
    }
    if case.inputs == InputProvider::NativeHealthTailInputs {
        return super::health_tail::admit(case, sources);
    }
    if case.inputs == InputProvider::NativeTouchdownInputs {
        return super::movement_touchdown::admit(case, sources);
    }
    if case.inputs == InputProvider::Unavailable {
        let dependency = match case.responsibilities.as_slice() {
            [Responsibility::Blocks] => {
                "independent complete block seed and ordered reaction requests"
            }
            [Responsibility::AudioSender] => {
                "independent CPU sender seed and ordered queue/service inputs"
            }
            [Responsibility::Appearance] => {
                "actual palette opportunities past the admitted opening returns"
            }
            _ => return Err(data_failure(None, "unavailable provider scope")),
        };
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: dependency.to_owned(),
        });
    }
    match case.responsibilities.as_slice() {
        [Responsibility::Audio] => audio(case, sources),
        [Responsibility::Playback] => super::playback::admit(case, sources),
        [Responsibility::AudioSender] => super::sender_offer::admit(case, sources),
        [Responsibility::Movement] => movement(case, sources),
        [
            Responsibility::Ceres
            | Responsibility::Camera
            | Responsibility::Hud
            | Responsibility::Appearance
            | Responsibility::Progress,
        ]
        | [Responsibility::Weapons, Responsibility::Contact]
        | [
            Responsibility::Movement,
            Responsibility::Appearance,
            Responsibility::Camera,
        ] => super::recorded::admit(case, sources),
        _ => Err(Outcome::Unsupported {
            cut: None,
            dependency: "complete integrated production dependency closure".to_owned(),
        }),
    }
}

fn audio(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    if case.inputs != InputProvider::NativeFrameInputs
        || case.expectations.provider != ExpectedProvider::NativeDecodedCuts
        || case.content != ContentProvider::OpeningCeresAudio
    {
        return Err(data_failure(None, "audio provider contract"));
    }
    let Start::NativeSeed { path, sha256 } = &case.start else {
        return Err(data_failure(None, "audio start contract"));
    };
    let path = if path.is_absolute() {
        path.clone()
    } else {
        sources.native.join(path)
    };
    let packet = case.audio_packet()?;
    let (input_tape, expected_tape) = audio::load(&path, sha256, case.end.cut_inclusive, packet)
        .map_err(|fault| match fault {
            audio::ProviderFault::Admission(source) => data_failure(None, source),
            audio::ProviderFault::Pin => data_failure(None, "native fixture pin"),
            audio::ProviderFault::Missing => data_failure(None, "native fixture file"),
        })?;
    let (seed, content) = audio::room_load::admit(sources, packet, &input_tape, sha256)?;
    let rules = ProductionRules::Audio;
    let production = Production::admit(seed, content.clone(), rules.clone())
        .map_err(|_| data_failure(None, "production audio admission"))?;
    let schedule = input_tape.frames.iter().map(|(cut, _)| *cut).collect();
    Ok(AdmittedCase {
        production,
        content,
        rules,
        actual: ActualReader::Audio(input_tape.aliases.clone()),
        schedule,
        inputs: InputReader::Audio(Box::new(input_tape)),
        expected: ExpectedReader::Audio(Box::new(expected_tape)),
    })
}

fn movement(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    if case.inputs == InputProvider::NativeSearchInputs {
        let (inputs, expected) = super::native_search::load(case, sources)?;
        let content = ProductionContent::Movement(inputs.content.movement.clone());
        let rules = ProductionRules::Movement(samus_movement::Rules::default());
        let production = Production::admit(
            ProductionSeed::Movement(inputs.seed),
            content.clone(),
            rules.clone(),
        )
        .map_err(|_| data_failure(None, "native search production admission"))?;
        return Ok(AdmittedCase {
            production,
            content,
            rules,
            actual: ActualReader::NativeSearch(None),
            schedule: vec![case.end.cut_inclusive],
            inputs: InputReader::NativeSearch(Box::new(inputs)),
            expected: ExpectedReader::NativeSearch(expected),
        });
    }
    if case.content != ContentProvider::PlayerRomImport
        || case.inputs != InputProvider::PrecedingCutMovementInputs
        || !matches!(
            case.expectations.provider,
            ExpectedProvider::PinnedRecordCache | ExpectedProvider::WitnessedRecordCache
        )
    {
        return Err(data_failure(None, "movement provider contract"));
    }
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "movement start contract"));
    };
    let input_cache = super::row_cache(sources, &case.row, case.end.cut_inclusive)?;
    let expected_cache = super::row_cache(sources, &case.row, case.end.cut_inclusive)?;
    if let Some(witness) = case.ordinary_witness() {
        witness.qualify_origin(&expected_cache)?;
    }
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
        expected: ExpectedReader::Movement(Box::new(expected_cache)),
    })
}
