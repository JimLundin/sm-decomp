use std::{env, path::PathBuf};

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
fn every_health_tail_frame_restores_exactly() {
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
    for cut in CUTS {
        let mut resumed = restored(&direct, &content, &rules);
        let expected = inputs.advance(&mut direct, cut).unwrap();
        assert!(inputs.advance(&mut resumed, cut).unwrap().eq(&expected));
        assert!(resumed.eq(&direct));
    }
}

#[test]
#[ignore = "uses the authenticated carried hit as origin of a constructed duplicate"]
fn constructed_protected_duplicate_preserves_complete_carried_health() {
    let AdmittedCase {
        mut production,
        inputs: InputReader::HealthTail(inputs),
        ..
    } = fixture().1
    else {
        panic!("scoped Health input provider");
    };
    let [hit_cut, protected_cut] = CUTS;
    inputs.advance(&mut production, hit_cut).unwrap();
    assert!(owner(&production).unwrap().health().protection != ProtectionFrames::ZERO);
    let [hit, quiet] = inputs.0.as_slice() else {
        panic!("native scope schedule");
    };
    assert_eq!(hit.tail, TailImpact::EligibleCeresTail);
    let mut duplicate = production.clone();
    let native = advance(&mut production, protected_cut, quiet).unwrap();
    let repeated = Frame {
        tail: TailImpact::EligibleCeresTail,
        ..*quiet
    };
    let protected = advance(&mut duplicate, protected_cut, &repeated).unwrap();
    assert!(protected.eq(&native));
    assert!(duplicate.eq(&production));
    assert_eq!(native.warning_sound, samus_health::WarningSound::None);
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
            original.drawn_energy(),
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

    for frame in &mut inputs.0 {
        let map = &mut frame.hud.map;
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
        current.drawn_energy(),
    );
    let mut constructed =
        Production::admit(ProductionSeed::HealthTail(Box::new(state)), content, rules).unwrap();
    let [_, quiet] = inputs.0[..] else {
        panic!("native scope schedule");
    };
    let output = advance(
        &mut constructed,
        protected_cut,
        &Frame { played: 0, ..quiet },
    )
    .unwrap();
    assert!(!output.gauge_redrawn);
    assert!(
        observations::gauge_words(owner(&constructed).unwrap().hud())
            .unwrap()
            .eq(&stale_words)
    );
}
