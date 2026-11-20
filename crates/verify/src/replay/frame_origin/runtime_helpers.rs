use super::*;
use data::timing::MasterTicks;
use game::{production::AdvanceInput, state::owned::OwnedState};

pub(super) fn admitted() -> AdmittedCase {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    admit(&case, &sources).unwrap()
}

fn complete_frame_case() -> Case {
    if let Some(path) = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE") {
        return toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    }
    crate::replay::index::Index::parse(include_str!("../../../../../scenarios/replay/index.toml"))
        .unwrap()
        .cases
        .into_iter()
        .find(|case| case.id.0 == "frame/ordinary-requested-nmi-return")
        .unwrap()
}

pub(super) fn retained_return() -> (Case, AdmittedCase) {
    let case = complete_frame_case();
    case.validate().unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        native: std::env::var_os("SM_AUDIO_NATIVE")
            .unwrap_or_default()
            .into(),
        ..Sources::default()
    };
    let admitted = admit(&case, &sources).unwrap();
    (case, admitted)
}

pub(super) fn advance_until(
    admitted: &mut AdmittedCase,
    budget: MasterTicks,
    reached: impl Fn(&frame::State) -> bool,
) {
    for _ in 0..500_000 {
        if let OwnedState::Frame(state) = admitted.production.state()
            && reached(state)
        {
            return;
        }
        admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap();
    }
    panic!("actual producer boundary absent");
}

pub(super) fn restore_next(
    admitted: &mut AdmittedCase,
    budget: MasterTicks,
) -> Result<game::production::ProductionOutputs, game::production::ProductionFault> {
    let saved = admitted.production.save().unwrap();
    let mut receiver =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(receiver.state().eq(admitted.production.state()));
    let actual = admitted.production.advance(AdvanceInput::Frame { budget });
    let restored = receiver.advance(AdvanceInput::Frame { budget });
    assert!(actual.eq(&restored));
    assert!(receiver.state().eq(admitted.production.state()));
    actual
}
