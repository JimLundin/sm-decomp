use std::{env, path::PathBuf};

use game::production::frame::frame_history::EnergyChange;

use super::*;
use crate::replay::{index::Index, runner};

fn fixture() -> (Case, AdmittedCase) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let index = Index::read(&root.join("scenarios/replay/index.toml")).unwrap();
    let case = index
        .cases
        .iter()
        .find(|case| case.id.0 == "health/ceres-tail-drawing-gauge")
        .unwrap();
    let sources = Sources {
        rom: PathBuf::from(env::var_os("SM_ROM").unwrap()),
        native: PathBuf::from(env::var_os("SM_AUDIO_NATIVE").unwrap()),
        ..Sources::default()
    };
    (case.clone(), admit(case, &sources).unwrap())
}

fn restored(at: &Production, content: &ProductionContent, rules: &ProductionRules) -> Production {
    let saved = at.save().unwrap();
    let restored = Production::restore(&saved, content.clone(), rules.clone()).unwrap();
    assert!(restored.eq(at));
    assert!(restored.save().unwrap().eq(&saved));
    restored
}

#[test]
#[ignore = "uses the authenticated Health and HUD tail fixture"]
fn every_health_tail_semantic_stop_restores_exactly() {
    let AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::HealthTail(inputs),
        ..
    } = fixture().1
    else {
        panic!("scoped Health input provider");
    };
    let mut other_origin = content.clone();
    let ProductionContent::HealthTail(contents) = &mut other_origin else {
        panic!("scoped Health content");
    };
    contents.origin_identity[0] ^= 1;
    assert!(matches!(
        Production::restore(&production.save().unwrap(), other_origin, rules.clone()),
        Err(game::production::AdmissionFault::ContentChanged)
    ));
    let mut direct = production;
    for (cut, opportunity) in CUTS.into_iter().zip(inputs.0) {
        let steps = Inputs::steps(opportunity).collect::<Vec<_>>();
        let mut complete = direct.clone();
        let expected = advance(&mut complete, cut, &steps).unwrap();
        let change = if cut == CUTS[0] {
            EnergyChange::Decreased
        } else {
            EnergyChange::Held
        };
        assert!(expected.iter().any(
            |output| matches!(output, Output::FirstDrawingRead(read, _) if read.change == change)
        ));
        for split in 0..=steps.len() {
            let (prefix, suffix) = steps.split_at(split);
            let mut stopped = direct.clone();
            let published = advance(&mut stopped, cut, prefix).unwrap();
            let mut resumed = restored(&stopped, &content, &rules);
            let continued = advance(&mut resumed, cut, suffix).unwrap();
            assert!(
                published
                    .into_iter()
                    .chain(continued)
                    .eq(expected.iter().copied())
            );
            assert!(resumed.eq(&complete));
        }
        direct = complete;
    }
}

#[test]
#[ignore = "uses the authenticated carried hit as origin of a constructed duplicate"]
fn constructed_protected_duplicate_preserves_complete_carried_health() {
    let AdmittedCase {
        mut production,
        content,
        rules,
        inputs: InputReader::HealthTail(inputs),
        ..
    } = fixture().1
    else {
        panic!("scoped Health input provider");
    };
    let [hit_cut, protected_cut] = CUTS;
    inputs.advance(&mut production, hit_cut).unwrap();
    let [hit, protected] = inputs.0.as_slice() else {
        panic!("native scope schedule");
    };
    let offer = hit.delivery.unwrap();
    let native_steps = Inputs::steps(*protected).collect::<Vec<_>>();
    let (actor_read, regular_suffix) = native_steps.split_at(2);
    advance(&mut production, protected_cut, actor_read).unwrap();
    let current = owner(&production).unwrap().health();
    assert!(current.protection != ProtectionFrames::ZERO);

    let mut restored = restored(&production, &content, &rules);
    for candidate in [&mut production, &mut restored] {
        let before = candidate.clone();
        assert!(matches!(
            candidate.advance(AdvanceInput::HealthTail(&[Input::ActiveClose])),
            Err(game::production::ProductionFault::Unsupported(_))
        ));
        assert!((*candidate).eq(&before));
        let output = advance(candidate, protected_cut, &[Input::Impact(offer)]).unwrap();
        assert!(output.eq(&[Output::Impact(current)]));
        assert!(owner(candidate).unwrap().health().eq(&current));
    }
    let direct = advance(&mut production, protected_cut, regular_suffix).unwrap();
    let resumed = advance(&mut restored, protected_cut, regular_suffix).unwrap();
    assert!(resumed.eq(&direct));
    assert!(restored.eq(&production));
    assert!(resumed.iter().any(|output| matches!(
        output,
        Output::Maintenance(_, samus_health::WarningSound::None)
    )));
}

#[test]
#[ignore = "uses authenticated Health and HUD tail inputs"]
fn dormant_off_map_inputs_preserve_the_native_gauge_and_map() {
    let (
        case,
        AdmittedCase {
            mut production,
            content,
            rules,
            inputs: InputReader::HealthTail(mut inputs),
            expected,
            mut actual,
            ..
        },
    ) = fixture()
    else {
        panic!("scoped Health input provider");
    };
    let mut direct = production.clone();
    let mut direct_actual = actual.clone();
    let native_inputs = InputReader::HealthTail(Inputs(inputs.0.clone()));
    let original = owner(&production).unwrap();
    let mut powered_on_hud = original.hud().clone();
    powered_on_hud.mini_map = interface_hud::MiniMapPower::On;
    let mut other_content = content;
    let ProductionContent::HealthTail(contents) = &mut other_content else {
        panic!("scoped Health content");
    };
    contents.origin_identity[0] ^= 1;
    let mut powered_on = Production::admit(
        ProductionSeed::HealthTail(Box::new(State::new(
            contents.origin_identity,
            original.health(),
            powered_on_hud,
            original.previous_draw_energy(),
        ))),
        other_content,
        rules,
    )
    .unwrap();
    let before = powered_on.clone();
    assert!(matches!(
        inputs.advance(&mut powered_on, CUTS[0]),
        Err(Outcome::Unsupported { .. })
    ));
    assert!(powered_on.eq(&before));

    for opportunity in &mut inputs.0 {
        let Input::HudMap(map) = &mut opportunity.hud.map else {
            panic!("borrowed powered-off map opportunity");
        };
        map.position = types::Pair {
            x: u16::MAX,
            y: u16::MAX,
        };
        map.room.cell = types::Pair { x: 55, y: 22 };
        map.room.blocks = types::Pair { x: 1, y: 1 };
        map.map_collected = !map.map_collected;
    }
    let dormant_inputs = InputReader::HealthTail(inputs);
    for cut in CUTS {
        runner::execute(
            &case,
            &native_inputs,
            &expected,
            &mut direct,
            &mut direct_actual,
            cut,
        )
        .unwrap();
        runner::execute(
            &case,
            &dormant_inputs,
            &expected,
            &mut production,
            &mut actual,
            cut,
        )
        .unwrap();
        assert!(production.eq(&direct));
    }
}

#[test]
#[ignore = "uses authenticated HUD content and a constructed stale gauge"]
fn equal_energy_preserves_the_stale_gauge() {
    let AdmittedCase {
        mut production,
        mut content,
        rules,
        inputs: InputReader::HealthTail(inputs),
        ..
    } = fixture().1
    else {
        panic!("scoped Health input provider");
    };
    let [hit_cut, protected_cut] = CUTS;
    let mut stale_hud = owner(&production).unwrap().hud().clone();
    let stale_words = observations::gauge_words(&stale_hud).unwrap();
    inputs.advance(&mut production, hit_cut).unwrap();
    let current = owner(&production).unwrap();
    assert!(!stale_words.eq(&observations::gauge_words(current.hud()).unwrap()));
    stale_hud.shown.energy = current.health().energy.whole.get();

    let ProductionContent::HealthTail(scoped) = &mut content else {
        panic!("scoped Health content");
    };
    scoped.origin_identity[0] ^= 1;
    let state = State::new(
        scoped.origin_identity,
        current.health(),
        stale_hud,
        current.previous_draw_energy(),
    );
    let mut constructed =
        Production::admit(ProductionSeed::HealthTail(Box::new(state)), content, rules).unwrap();
    let [_, mut quiet] = inputs.0[..] else {
        panic!("native scope schedule");
    };
    let Input::HudControls { opportunity, .. } = &mut quiet.hud.controls else {
        panic!("early HUD controls");
    };
    *opportunity = 0;
    let steps = Inputs::steps(quiet).collect::<Vec<_>>();
    let outputs = advance(&mut constructed, protected_cut, &steps).unwrap();
    assert!(
        outputs
            .iter()
            .any(|output| matches!(output, Output::Status(_, _, false)))
    );
    assert!(
        observations::gauge_words(owner(&constructed).unwrap().hud())
            .unwrap()
            .eq(&stale_words)
    );
}
