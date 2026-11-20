//! Development checks for immutable physical templates and current-owner bindings.
#[path = "../src/replay/frame_profiles.rs"]
mod frame_profiles;
use data::timing::{Cycle, ExposureKind};
use frame_profiles::{Effect, Profile, ProfileFault, ProfileName, Run};

#[test]
fn half_open_slices_preserve_repeated_physical_opportunities() {
    let profile = Profile {
        name: ProfileName::PreWait,
        exposures: 7,
        runs: vec![
            Run {
                count: 4,
                cycle: Cycle::Six,
                kind: ExposureKind::Read,
                poll_before: false,
                effect: Effect::None,
            },
            Run {
                count: 3,
                cycle: Cycle::Eight,
                kind: ExposureKind::Write,
                poll_before: true,
                effect: Effect::None,
            },
        ],
    };
    let slice = profile.slice(2, 6).unwrap();
    assert_eq!(slice.name, profile.name);
    assert_eq!(slice.exposures, 4);
    assert_eq!(
        slice.runs.iter().map(|run| run.count).collect::<Vec<_>>(),
        [2, 2]
    );
    assert_eq!(slice.runs.first().unwrap().cycle, Cycle::Six);
    assert_eq!(slice.runs.last().unwrap().cycle, Cycle::Eight);
    for (first, end) in [(0, 0), (4, 3), (0, 8)] {
        assert_eq!(profile.slice(first, end), Err(ProfileFault::Coverage));
    }
}

fn packet() -> Result<Vec<String>, ProfileFault> {
    let directory = std::env::var_os("SM_FRAME_PROFILE_PACKET").ok_or(ProfileFault::Unsupported)?;
    [
        "profiles.toml",
        "portions.toml",
        "publication-bindings.toml",
        "sample-bindings.toml",
        "materialization-bindings.toml",
        "arithmetic-receipts.toml",
    ]
    .iter()
    .map(|name| {
        std::fs::read_to_string(std::path::Path::new(&directory).join(name))
            .map_err(|_| ProfileFault::Format)
    })
    .collect()
}
fn decode(texts: &[String]) -> Result<frame_profiles::Templates, ProfileFault> {
    frame_profiles::decode(
        texts
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| ProfileFault::Format)?,
    )
}
#[test]
#[ignore = "requires accepted private immutable profile packet"]
fn accepted_templates_and_complete_bindings_decode() {
    let templates = decode(&packet().unwrap()).unwrap();
    assert_eq!(templates.profiles.len(), 6);
    assert_eq!(templates.materializations.len(), 5);
    let publications: Vec<_> = templates
        .profiles
        .iter()
        .flat_map(|profile| &profile.runs)
        .filter_map(|run| {
            if let Effect::Publication(publication) = &run.effect {
                Some(publication)
            } else {
                None
            }
        })
        .collect();
    assert!(publications.iter().any(|publication| {
        publication
            .arithmetic
            .as_ref()
            .is_some_and(|task| task.name == "InsideBlockQueryOne")
    }));
    assert!(
        publications
            .iter()
            .any(|publication| publication.producer == "FootstepsQueue")
    );
    for profile in &templates.profiles {
        for portion in templates
            .portions
            .iter()
            .filter(|portion| portion.profile == profile.name)
        {
            let sliced = profile
                .slice(portion.first_exposure, portion.end_exposure)
                .unwrap();
            assert_eq!(sliced.name, profile.name);
            assert_eq!(
                sliced.exposures,
                portion.end_exposure - portion.first_exposure
            );
        }
    }
}
#[test]
#[ignore = "requires accepted private immutable profile packet"]
fn broken_namespace_binding_and_coverage_reject() {
    let originals = packet().unwrap();
    for (file, before, after) in [
        (
            0,
            "device = \"Multiplicand\"",
            "device = \"SmpMultiplicand\"",
        ),
        (0, "count = 3", "count = 0"),
        (1, "first_exposure = 0", "first_exposure = 1"),
        (1, "end_is_exclusive = true", "end_is_exclusive = false"),
        (
            2,
            "producer = \"FrameMathTasks\"",
            "producer = \"ExpectedCallback\"",
        ),
        (
            5,
            "actual_owner_receipt_required = true",
            "actual_owner_receipt_required = false",
        ),
    ] {
        let mut texts = originals.clone();
        texts[file] = texts[file].replacen(before, after, 1);
        assert!(decode(&texts).is_err());
    }
}
fn binding_profile(task: &frame_profiles::ArithmeticTask) -> Profile {
    use data::timing::{PublicationPort, Sample};
    use frame_profiles::Publication;
    Profile {
        name: ProfileName::PreWait,
        exposures: 7,
        runs: vec![
            Run {
                count: 2,
                cycle: Cycle::Six,
                kind: ExposureKind::Write,
                poll_before: false,
                effect: Effect::Publication(Publication {
                    port: PublicationPort(0x4202),
                    device: "Multiplicand".into(),
                    producer: "FrameMathTasks".into(),
                    arithmetic: Some(task.clone()),
                }),
            },
            Run {
                count: 2,
                cycle: Cycle::Six,
                kind: ExposureKind::Read,
                poll_before: false,
                effect: Effect::Sample(Sample::ProductLow, Some(task.clone())),
            },
            Run {
                count: 1,
                cycle: Cycle::Six,
                kind: ExposureKind::Read,
                poll_before: false,
                effect: Effect::Sample(Sample::ProductHigh, Some(task.clone())),
            },
            Run {
                count: 2,
                cycle: Cycle::Six,
                kind: ExposureKind::Internal,
                poll_before: true,
                effect: Effect::None,
            },
        ],
    }
}
#[test]
fn binding_uses_each_current_write_and_retains_ordered_sample_contracts() {
    use data::timing::{DeviceEffect, DeviceWrite, Sample};
    use frame_profiles::ArithmeticTask;
    let task = ArithmeticTask {
        name: "InventedTerrainQuery".into(),
        quantities: vec!["CurrentRow".into()],
    };
    let profile = binding_profile(&task);
    let writes = [
        DeviceWrite::from_cpu_port(0x4202, 7).unwrap(),
        DeviceWrite::from_cpu_port(0x4202, 9).unwrap(),
    ];
    let bound = profile.bind(&writes).unwrap();
    assert_eq!(bound.name, profile.name);
    assert_eq!(
        bound
            .profile
            .runs
            .iter()
            .map(|run| run.count)
            .collect::<Vec<_>>(),
        [1, 1, 2, 1, 2]
    );
    assert_eq!(
        bound.profile.runs.first().unwrap().exposure.effect,
        DeviceEffect::Publish(writes[0].clone())
    );
    assert_eq!(
        bound.profile.runs.get(1).unwrap().exposure.effect,
        DeviceEffect::Publish(writes[1].clone())
    );
    assert_eq!(
        bound
            .samples
            .iter()
            .map(|sample| sample.device)
            .collect::<Vec<_>>(),
        [Sample::ProductLow, Sample::ProductLow, Sample::ProductHigh]
    );
    assert!(
        bound
            .samples
            .iter()
            .all(|sample| sample.arithmetic.as_ref() == Some(&task))
    );
    let original = profile.clone();
    for incomplete in [
        vec![],
        vec![writes[0].clone()],
        vec![writes[0].clone(), writes[1].clone(), writes[0].clone()],
    ] {
        assert!(
            profile
                .bind(&incomplete)
                .is_err_and(|fault| fault == ProfileFault::Binding)
        );
    }
    let wrong_port = [
        writes[0].clone(),
        DeviceWrite::from_cpu_port(0x4203, 9).unwrap(),
    ];
    assert_eq!(profile.bind(&wrong_port), Err(ProfileFault::Binding));
    let mut inconsistent = writes;
    inconsistent[1].value = 10;
    assert!(
        profile
            .bind(&inconsistent)
            .is_err_and(|fault| fault == ProfileFault::Binding)
    );
    assert_eq!(profile, original);
    let empty = Profile {
        name: ProfileName::PreWait,
        exposures: 0,
        runs: vec![],
    };
    assert_eq!(empty.bind(&[]), Err(ProfileFault::Profile));
}
