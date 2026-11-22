use super::*;
use verify::replay::{baseline::Claim, result::Outcome};

fn fixture() -> (PathBuf, Index, Mapping, Baseline) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let path = root.join("scenarios/replay/index.toml");
    let snapshot = replay::read_index(&path).unwrap();
    let mapping = Mapping::parse(
        &std::fs::read_to_string(root.join("scenarios/replay/code-map.toml")).unwrap(),
    )
    .unwrap();
    let baseline = Baseline::read(&root.join("scenarios/replay/baseline.toml")).unwrap();
    baseline.validate_index(&snapshot).unwrap();
    (root, snapshot.into_index(), mapping, baseline)
}

#[test]
fn committed_inventory_has_valid_targets_and_every_required_case() {
    #[derive(Deserialize)]
    struct InventoryCase {
        declaration: Case,
        reviewed_requirement: Floor,
    }
    #[derive(Deserialize)]
    struct Inventory {
        case: Vec<InventoryCase>,
    }

    let (root, index, mapping, baseline) = fixture();
    mapping.validate(&index, &root).unwrap();
    let args = vec![
        root.join("scenarios/replay/index.toml")
            .to_string_lossy()
            .into_owned(),
        "--repo".to_owned(),
        root.to_string_lossy().into_owned(),
    ];
    let report: Inventory = toml::from_str(&run(&args).unwrap()).unwrap();
    assert!(
        report
            .case
            .iter()
            .map(|view| toml::to_string(&view.declaration).unwrap())
            .eq(index
                .cases
                .iter()
                .map(|case| toml::to_string(case).unwrap()))
    );
    for view in report.case {
        let required = baseline
            .cases
            .iter()
            .find(|floor| floor.id == view.declaration.id)
            .unwrap();
        assert_eq!(
            toml::to_string(&view.reviewed_requirement).unwrap(),
            toml::to_string(required).unwrap()
        );
    }
}

#[test]
fn rule_and_file_lookup_preserve_passing_witness_and_held_boundaries() {
    let (_, index, mapping, baseline) = fixture();
    let by_rule = mapping
        .select(
            &index,
            &Selection {
                rule: Some("samus-movement/TravelledDistances".to_owned()),
                ..Selection::default()
            },
        )
        .unwrap();
    let by_file = mapping
        .select(
            &index,
            &Selection {
                path: Some("crates/clean/samus_movement/src/travel.rs".to_owned()),
                ..Selection::default()
            },
        )
        .unwrap();
    assert_eq!(
        by_rule.iter().map(|case| &case.id).collect::<Vec<_>>(),
        by_file.iter().map(|case| &case.id).collect::<Vec<_>>()
    );
    let report = views(&mapping, &baseline, &by_file).unwrap();
    assert_eq!(
        report
            .iter()
            .map(|view| view.declaration.id.0.as_str())
            .collect::<Vec<_>>(),
        [
            "movement/D-inherited-cut",
            "integrated/missing-dependency-closure",
            "movie/missing-power-on",
            "frame/ordinary-requested-nmi-return",
            "frame/adjacent-actual-trial",
            "frame/adjacent-held-trial",
            "frame/adjacent-actual-return",
            "frame/opening-three-native-returns",
            "frame/opening-eight-native-returns",
            "frame/adjacent-held-return",
            "frame/health-hud-stock",
            "frame/health-hud-pending-loss",
        ]
    );
    assert_eq!(
        report
            .iter()
            .filter(|view| view.reviewed_requirement.expected == Outcome::Pass)
            .map(|view| view.declaration.id.0.as_str())
            .collect::<Vec<_>>(),
        [
            "movement/D-inherited-cut",
            "frame/ordinary-requested-nmi-return",
            "frame/adjacent-actual-return",
            "frame/opening-three-native-returns",
            "frame/opening-eight-native-returns",
            "frame/adjacent-held-return",
            "frame/health-hud-stock",
            "frame/health-hud-pending-loss",
        ]
    );
    assert_eq!(
        report
            .iter()
            .filter(|view| matches!(
                view.reviewed_requirement.expected,
                Outcome::Unsupported { .. }
            ))
            .map(|view| view.declaration.id.0.as_str())
            .collect::<Vec<_>>(),
        [
            "integrated/missing-dependency-closure",
            "movie/missing-power-on",
            "frame/adjacent-actual-trial",
            "frame/adjacent-held-trial",
        ]
    );
    for case in by_file {
        let by_case = mapping
            .select(
                &index,
                &Selection {
                    ids: vec![case.id.clone()],
                    ..Selection::default()
                },
            )
            .unwrap();
        assert!(
            views(&mapping, &baseline, &by_case)
                .unwrap()
                .first()
                .unwrap()
                .declared_relevance
                .iter()
                .any(|rule| rule
                    .targets
                    .iter()
                    .any(|target| target.path == "crates/clean/samus_movement/src/travel.rs"))
        );
    }
}

#[test]
fn haze_settling_lookup_reports_the_reviewed_requirement_for_each_controller_case() {
    let (_, index, mapping, baseline) = fixture();
    let cases = mapping
        .select(
            &index,
            &Selection {
                rule: Some("world-haze/HazeSettles".to_owned()),
                ..Selection::default()
            },
        )
        .unwrap();
    let report = views(&mapping, &baseline, &cases).unwrap();
    assert_eq!(report.len(), 6);
    for (id, claim) in [
        ("frame/adjacent-actual-trial", Claim::Boundary),
        ("frame/adjacent-held-trial", Claim::Boundary),
        ("frame/adjacent-actual-return", Claim::Behaviour),
        ("frame/adjacent-held-return", Claim::Behaviour),
        ("frame/opening-three-native-returns", Claim::Behaviour),
        ("frame/opening-eight-native-returns", Claim::Behaviour),
    ] {
        let view = report
            .iter()
            .find(|view| view.declaration.id.0 == id)
            .unwrap();
        let floor = baseline
            .cases
            .iter()
            .find(|floor| floor.id.0 == id)
            .unwrap();
        assert_eq!(view.reviewed_requirement.id, floor.id);
        assert_eq!(view.reviewed_requirement.claim, floor.claim);
        assert_eq!(view.reviewed_requirement.claim, claim);
        assert_eq!(view.reviewed_requirement.expected, floor.expected);
    }
}

#[test]
fn borrowed_stimuli_are_not_added_to_witnessed_rules_or_expectations() {
    let (_, index, mapping, baseline) = fixture();
    let selected = mapping
        .select(
            &index,
            &Selection {
                ids: vec![CaseId("movement/W15".to_owned())],
                ..Selection::default()
            },
        )
        .unwrap();
    let view = views(&mapping, &baseline, &selected).unwrap().remove(0);
    assert_eq!(view.declared_relevance.len(), 2);
    assert_eq!(view.declaration.borrowed.len(), 2);
    assert_eq!(view.declaration.expectations.fields.len(), 3);
    assert_eq!(view.declaration.expectations.outputs.len(), 2);
    assert_eq!(view.coordinates, Coordinates::RowLocalCache);
    assert_eq!(view.first_compared_cut, Some(815));
    assert_eq!(view.declaration.end.cut_inclusive, 817);
}

#[test]
fn empty_rule_boundaries_and_native_and_power_on_starts_remain_distinct() {
    let (_, index, mapping, baseline) = fixture();
    let selected = mapping.select(&index, &Selection::default()).unwrap();
    let views = views(&mapping, &baseline, &selected).unwrap();
    let empty: Vec<_> = views
        .iter()
        .filter(|view| view.declaration.rules.is_empty())
        .collect();
    assert_eq!(empty.len(), 3);
    assert!(empty.iter().all(|view| view.declared_relevance.is_empty()
        && view.reviewed_requirement.claim == Claim::Boundary));
    let checkpoint = views
        .iter()
        .find(|view| view.declaration.id.0 == "audio/native-expire-post64")
        .unwrap();
    assert_eq!(checkpoint.coordinates, Coordinates::NativeFixture);
    assert_eq!(checkpoint.first_compared_cut, Some(65));
    let native = views
        .iter()
        .find(|view| view.declaration.id.0 == "audio/native-control")
        .unwrap();
    assert_eq!(native.coordinates, Coordinates::NativeFixture);
    assert_eq!(native.first_compared_cut, Some(0));
    let power = views
        .iter()
        .find(|view| view.declaration.id.0 == "movie/missing-power-on")
        .unwrap();
    assert_eq!(power.coordinates, Coordinates::PowerOn);
    assert_eq!(power.reviewed_requirement.claim, Claim::Boundary);
}

#[test]
fn missing_duplicate_unknown_and_conflicting_mappings_fail_closed() {
    let (root, index, mapping, _) = fixture();
    let mut missing = mapping.clone();
    missing.rule.remove(0);
    assert!(matches!(
        missing.validate(&index, &root),
        Err(Fault::MissingRule(_))
    ));
    let mut duplicate = mapping.clone();
    duplicate.rule.push(duplicate.rule.first().unwrap().clone());
    assert!(matches!(
        duplicate.validate(&index, &root),
        Err(Fault::DuplicateRule(_))
    ));
    let mut unknown = mapping.clone();
    unknown.rule.first_mut().unwrap().id = "samus-movement/SpaceJump".to_owned();
    assert!(matches!(
        unknown.validate(&index, &root),
        Err(Fault::UnknownRule(_))
    ));
    let mut conflicting = mapping.clone();
    let target = conflicting
        .rule
        .first()
        .unwrap()
        .targets
        .first()
        .unwrap()
        .clone();
    conflicting.rule.first_mut().unwrap().targets.push(target);
    assert!(matches!(
        conflicting.validate(&index, &root),
        Err(Fault::DuplicateTarget(_))
    ));
    let mut missing_role = mapping;
    missing_role
        .rule
        .first_mut()
        .unwrap()
        .targets
        .retain(|target| target.role != Role::Adapter);
    assert!(matches!(
        missing_role.validate(&index, &root),
        Err(Fault::MissingRole(_))
    ));
}

#[test]
fn unsafe_dangling_and_undeclared_targets_and_invalid_selectors_fail_closed() {
    let (root, index, mapping, _) = fixture();
    for path in [
        "../AGENTS.md",
        "/AGENTS.md",
        "C:/AGENTS.md",
        "crates\\game\\src\\production.rs",
        "crates/game/src/missing.rs",
        "crates/game/src",
    ] {
        let mut invalid = mapping.clone();
        invalid
            .rule
            .first_mut()
            .unwrap()
            .targets
            .first_mut()
            .unwrap()
            .path = path.to_owned();
        assert!(matches!(
            invalid.validate(&index, &root),
            Err(Fault::InvalidTarget(_))
        ));
    }
    assert!(matches!(
        mapping.select(
            &index,
            &Selection {
                path: Some("AGENTS.md".to_owned()),
                ..Selection::default()
            }
        ),
        Err(Fault::UnknownPath(_))
    ));
    assert!(matches!(
        mapping.select(
            &index,
            &Selection {
                rule: Some("unknown/rule".to_owned()),
                ..Selection::default()
            }
        ),
        Err(Fault::UnknownRule(_))
    ));
    let id = CaseId("movement/W15".to_owned());
    assert!(matches!(
        mapping.select(
            &index,
            &Selection {
                ids: vec![id.clone(), id],
                ..Selection::default()
            }
        ),
        Err(Fault::Index(IndexFault::DuplicateSelection(_)))
    ));
    assert!(matches!(
        Mapping::parse("format = 1\nextra = true\nrule = []"),
        Err(Fault::Syntax)
    ));
}

#[test]
fn path_defaults_and_explicit_overrides_resolve_without_changing_selectors() {
    let (root, _, _, _) = fixture();
    let index = root.join("scenarios/replay/index.toml");
    let default = options(&index, &[]).unwrap();
    assert_eq!(default.mapping, index.with_file_name("code-map.toml"));
    assert_eq!(default.baseline, index.with_file_name("baseline.toml"));
    assert_eq!(default.root, Path::new(".").canonicalize().unwrap());
    let args = vec![
        "--repo".to_owned(),
        root.to_string_lossy().into_owned(),
        "--mapping".to_owned(),
        "alternate-map.toml".to_owned(),
        "--baseline".to_owned(),
        "alternate-baseline.toml".to_owned(),
        "--case".to_owned(),
        "movement/W15".to_owned(),
    ];
    let explicit = options(&index, &args).unwrap();
    assert_eq!(explicit.root, root);
    assert_eq!(explicit.mapping, Path::new("alternate-map.toml"));
    assert_eq!(explicit.baseline, Path::new("alternate-baseline.toml"));
    assert_eq!(explicit.selection.ids, [CaseId("movement/W15".to_owned())]);
    assert!(matches!(
        options(&index, &["--rule".to_owned()]),
        Err(Fault::Arguments)
    ));
    assert!(matches!(
        options(
            &index,
            &[
                "--rule".to_owned(),
                "a".to_owned(),
                "--rule".to_owned(),
                "b".to_owned()
            ]
        ),
        Err(Fault::Arguments)
    ));
}

#[test]
fn combined_selectors_intersect_and_unrelated_cases_fail() {
    let (_, index, mapping, _) = fixture();
    let selection = Selection {
        ids: vec![
            CaseId("movement/D-inherited-cut".to_owned()),
            CaseId("movie/missing-power-on".to_owned()),
        ],
        rule: Some("samus-movement/TravelledDistances".to_owned()),
        path: Some("crates/clean/samus_movement/src/travel.rs".to_owned()),
    };
    assert_eq!(mapping.select(&index, &selection).unwrap().len(), 2);
    let unrelated = Selection {
        ids: vec![CaseId("audio/native-control".to_owned())],
        ..selection
    };
    assert!(matches!(
        mapping.select(&index, &unrelated),
        Err(Fault::Index(IndexFault::Empty))
    ));
}

#[test]
fn command_queries_round_trip_typed_contracts_and_reviewed_requirements() {
    #[derive(Deserialize)]
    struct QueryCase {
        declaration: Case,
        reviewed_requirement: Floor,
        declared_relevance: Vec<Rule>,
    }
    #[derive(Deserialize)]
    struct QueryReport {
        case: Vec<QueryCase>,
    }
    let (root, index, _, _) = fixture();
    let prefix = vec![
        root.join("scenarios/replay/index.toml")
            .to_string_lossy()
            .into_owned(),
        "--repo".to_owned(),
        root.to_string_lossy().into_owned(),
        "--mapping".to_owned(),
        root.join("scenarios/replay/code-map.toml")
            .to_string_lossy()
            .into_owned(),
        "--baseline".to_owned(),
        root.join("scenarios/replay/baseline.toml")
            .to_string_lossy()
            .into_owned(),
    ];
    let travel = "samus-movement/TravelledDistances";
    for (flag, selector) in [
        ("--case", "movement/W15"),
        ("--rule", travel),
        ("--rule", "samus-movement/MotionAttempts"),
        ("--rule", "samus-movement/ReachedEnemyCandidates"),
        ("--rule", "samus-movement/PerformedCollisionTests"),
        ("--rule", "samus-movement/PerformedSupportChecks"),
        ("--rule", "samus-movement/PerformedBodyFit"),
        ("--path", "crates/clean/samus_movement/src/travel.rs"),
    ] {
        let mut args = prefix.clone();
        args.extend([flag.to_owned(), selector.to_owned()]);
        let report: QueryReport = toml::from_str(&run(&args).unwrap()).unwrap();
        let expected = index.cases.iter().filter(|case| match flag {
            "--case" => case.id.0 == selector,
            "--path" => case.rules.iter().any(|rule| rule == travel),
            _ => case.rules.iter().any(|rule| rule == selector),
        });
        assert!(!report.case.is_empty());
        assert!(
            report
                .case
                .iter()
                .map(|view| &view.declaration.id)
                .eq(expected.map(|case| &case.id))
        );
        for view in report.case {
            let original = index
                .cases
                .iter()
                .find(|case| case.id == view.declaration.id)
                .unwrap();
            assert_eq!(view.declaration.rules, original.rules);
            assert_eq!(view.declaration.borrowed.len(), original.borrowed.len());
            assert_eq!(view.declared_relevance.len(), original.rules.len());
            assert_eq!(view.reviewed_requirement.id, original.id);
        }
    }
}

#[cfg(unix)]
#[test]
fn symlink_cannot_escape_repository_root() {
    use std::os::unix::fs::symlink;
    let sandbox = std::env::temp_dir().join(format!("replay-map-symlink-{}", std::process::id()));
    let root = sandbox.join("root");
    std::fs::create_dir_all(&root).unwrap();
    let outside = sandbox.join("outside.rs");
    std::fs::write(&outside, b"").unwrap();
    symlink(&outside, root.join("escape.rs")).unwrap();
    let result = safe_path(&root.canonicalize().unwrap(), "escape.rs");
    std::fs::remove_file(root.join("escape.rs")).unwrap();
    std::fs::remove_file(outside).unwrap();
    std::fs::remove_dir(root).unwrap();
    std::fs::remove_dir(sandbox).unwrap();
    assert!(matches!(result, Err(Fault::InvalidTarget(_))));
}
