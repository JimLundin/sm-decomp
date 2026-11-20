use super::super::frame_origin::profiles::{Portion, Profile, Run};
use super::*;
use data::timing::{Cycle, ExposureKind, Sample};

fn templates() -> Templates {
    let portion: Portion = toml::from_str(
        "profile = \"PreWait\"\nfirst_exposure = 4615\nend_exposure = 4905\n\
         semantic_owner = \"S2.HudWeapons\"\nreceipt_contract = \"S2.HudWeapons\"\n\
         completed_producer_receipts = []\nintrinsic_ticks = 1740\n\
         first_run = 0\nfirst_repetition = 4615\nend_run = 0\nend_repetition = 4905\n\
         end_is_exclusive = true\npublication_inputs = []\nhardware_result_exposures = []\n",
    )
    .unwrap();
    Templates {
        profiles: vec![Profile {
            name: ProfileName::PreWait,
            exposures: 4905,
            runs: vec![Run {
                count: 4905,
                cycle: Cycle::Six,
                kind: ExposureKind::Internal,
                poll_before: true,
                effect: Effect::None,
            }],
        }],
        portions: vec![portion],
        materializations: Vec::new(),
    }
}

#[test]
fn whole_quiet_owner_retains_all_exposures_and_interrupt_polls() {
    let template = whole_template(&templates()).unwrap();
    assert_eq!(
        template
            .runs
            .iter()
            .map(|run| u64::from(run.count))
            .sum::<u64>(),
        290
    );
    assert!(template.runs.iter().all(|run| run.poll_before));
}

#[test]
fn shortened_duplicate_or_wrong_contract_scope_rejects() {
    let mut changed = templates();
    changed.portions.first_mut().unwrap().end_exposure = 4904;
    assert_eq!(whole_template(&changed), Err(Fault::Scope));
    let mut changed = templates();
    changed
        .portions
        .push(changed.portions.first().unwrap().clone());
    assert_eq!(whole_template(&changed), Err(Fault::Scope));
    let mut changed = templates();
    changed.portions.first_mut().unwrap().receipt_contract = "S2.Projectiles".to_owned();
    assert_eq!(whole_template(&changed), Err(Fault::Scope));
}

#[test]
fn no_device_effect_may_be_omitted_from_the_whole_owner() {
    let mut changed = templates();
    let run = changed
        .profiles
        .first_mut()
        .unwrap()
        .runs
        .first_mut()
        .unwrap();
    run.kind = ExposureKind::Read;
    run.effect = Effect::Sample(Sample::Blanking, None);
    assert_eq!(whole_template(&changed), Err(Fault::Effects));
}

#[test]
#[ignore = "requires explicit accepted whole frame profile directory"]
fn accepted_native_whole_hud_weapons_scope_is_complete_and_has_no_device_effects() {
    let directory = std::path::PathBuf::from(std::env::var_os("SM_FRAME_PROFILES").unwrap());
    let texts = [
        "profiles.toml",
        "portions.toml",
        "publication-bindings.toml",
        "sample-bindings.toml",
        "materialization-bindings.toml",
        "arithmetic-receipts.toml",
    ]
    .map(|name| std::fs::read_to_string(directory.join(name)).unwrap());
    let templates =
        super::super::frame_origin::profiles::decode(texts.each_ref().map(String::as_str)).unwrap();
    assert!(whole_template(&templates).is_ok());
}
