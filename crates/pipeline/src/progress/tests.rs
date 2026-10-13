use super::*;

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-progress-test-{}-{}",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn options(&self) -> Options {
        Options {
            root: self.0.clone(),
            registry: self.0.join("registry.json"),
            html: self.0.join("index.html"),
            source: None,
            update: None,
            bind: "127.0.0.1:0".parse().unwrap(),
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn registry(root: &Path) -> Registry {
    Registry {
        schema_version: 1,
        root: root.to_string_lossy().into_owned(),
        updated_at: 1,
        source: Source {
            revision: "a".repeat(40),
            inventory_sha256: "b".repeat(64),
            identity_sha256: "c".repeat(64),
            definition_count: 2,
            body_lines: 30,
            scope: "all pinned definitions".into(),
            source_root: None,
            source_verified_at: None,
        },
        routines: ["First", "Second"]
            .into_iter()
            .enumerate()
            .map(|(index, symbol)| Routine {
                symbol: symbol.into(),
                source: "src/sm_80.c".into(),
                line: index + 1,
                original_address: None,
                body_lines: (index + 1) * 10,
                definition_sha256: "d".repeat(64),
                calls: Vec::new(),
                declared_in: vec!["tools/port/declared.toml".into()],
            })
            .collect(),
        updates: Vec::new(),
        notices: Vec::new(),
        stale_update_ids: Vec::new(),
        superseded_update_ids: Vec::new(),
        summary: Summary::default(),
        coverage_history: None,
    }
}
fn observation(root: &Path, id: &str, symbol: &str, stage: Stage) -> Update {
    let evidence = root.join("evidence.txt");
    if !evidence.exists() {
        fs::write(&evidence, b"retained evidence").unwrap();
    }
    Update {
        id: id.into(),
        scope: "camera scroll ownership".into(),
        agent: "worker".into(),
        stage,
        evidence_context: EvidenceContext::Unspecified,
        supersedes: Vec::new(),
        routines: vec![symbol.into()],
        source_revision: "a".repeat(40),
        inventory_sha256: "b".repeat(64),
        commit: Some("e".repeat(40)),
        evidence: vec![Evidence { path: "evidence.txt".into(), sha256: crate::digest(&evidence).unwrap() }],
        limitations: vec!["offline scope only; no gameplay acceptance".into()],
        updated_at: 2,
    }
}

#[test]
fn committed_coverage_history_ignores_import_dates_deduplicates_and_detects_stale_inputs() {
    use std::process::Command;
    let scratch = Scratch::new();
    let root = &scratch.0;
    let run_git = |args: &[&str]| {
        let output = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
        assert!(output.status.success(), "Git {:?}: {}", args, String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    let commit = |subject: &str, at: u64| {
        run_git(&["add", "."]);
        let date = format!("{at} +0000");
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["-c", "commit.gpgsign=false", "commit", "-m", subject])
            .env("GIT_AUTHOR_DATE", &date)
            .env("GIT_COMMITTER_DATE", &date)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    };
    run_git(&["init"]);
    run_git(&["symbolic-ref", "HEAD", "refs/heads/main"]);
    run_git(&["config", "user.name", "Reporting test"]);
    run_git(&["config", "user.email", "reporting@example.invalid"]);
    fs::write(root.join("baseline.txt"), "before Rust port").unwrap();
    commit("baseline", 1_700_000_000);
    fs::create_dir_all(root.join("crates/port/src/nested")).unwrap();
    fs::create_dir_all(root.join("crates/pipeline/src/progress")).unwrap();
    fs::create_dir_all(root.join("tools/progress")).unwrap();
    for input in source::INPUTS.iter().chain(["crates/pipeline/src/progress/history.rs"].iter()) {
        fs::write(root.join(input), "").unwrap();
    }
    let first = r#"fn dispatch(name: &str) { match name { "First" => body(), _ => refused() } }
        // "Second" => comment only
        fn supported() { let names = ["Second"]; }"#;
    fs::write(root.join("crates/port/src/core.rs"), first).unwrap();
    commit("first body", 1_700_000_100);
    run_git(&["checkout", "-b", "unmerged-source"]);
    fs::write(
        root.join("crates/port/src/unmerged.rs"),
        r#"fn dispatch(n: &str) { match n { "Second" => body(), _ => refused() } }"#,
    )
    .unwrap();
    commit("unmerged source must receive no arrival credit", 1_700_000_150);
    run_git(&["checkout", "main"]);
    fs::write(root.join("crates/port/src/nested/duplicate.rs"), first).unwrap();
    commit("second selector for the same native body", 1_700_000_200);
    fs::write(
        root.join("crates/port/src/core.rs"),
        r#"fn dispatch(n: &str) { match n { "Second" => body(), _ => refused() } }"#,
    )
    .unwrap();
    fs::remove_file(root.join("crates/port/src/nested/duplicate.rs")).unwrap();
    // A backwards clock is clamped, rather than producing a backwards graph.
    commit("replace native coverage", 1_700_000_190);
    let mut data = registry(root);
    let mut report = observation(root, "import-today", "Second", Stage::Translated);
    report.evidence_context = EvidenceContext::CurrentCheckout;
    report.updated_at = 2_000_000_000;
    data.updates.push(report);
    let retained = history::reconstruct(root, &data).unwrap();
    assert_eq!(retained.checkpoints.len(), 3);
    assert_eq!(retained.checkpoints[0].at, 1_700_000_100);
    assert_eq!(retained.checkpoints[0].added, ["First"]);
    assert!(retained.checkpoints[1].added.is_empty());
    assert_eq!(retained.checkpoints[2].at, 1_700_000_200);
    assert_eq!(retained.checkpoints[2].added, ["Second"]);
    assert_eq!(retained.checkpoints[2].removed, ["First"]);
    assert_eq!(retained.current_symbols, ["Second"]);
    assert_eq!(retained.current.body_lines, 20);
    assert!(retained.uncommitted.is_empty());
    assert!(retained.limitations.iter().any(|s| s.contains("clamped")));
    data.coverage_history = Some(retained);
    history::observe(root, &mut data);
    assert_eq!(data.coverage_history.as_ref().unwrap().status, "current");
    fs::write(root.join("crates/port/src/nested/extra.rs"), first).unwrap();
    history::observe(root, &mut data);
    assert_eq!(data.coverage_history.as_ref().unwrap().status, "source-changed");
    assert!(history::reconstruct(root, &data).is_err(), "unreviewed extra bodies must not be silently included");
    fs::remove_file(root.join("crates/port/src/nested/extra.rs")).unwrap();
    fs::write(root.join(source::INPUTS[0]), "# changed mapping\n").unwrap();
    history::observe(root, &mut data);
    assert_eq!(data.coverage_history.as_ref().unwrap().status, "mapping-changed");
    data.source.inventory_sha256 = "f".repeat(64);
    history::observe(root, &mut data);
    assert_eq!(data.coverage_history.as_ref().unwrap().status, "inventory-changed");
}

fn inventory_root(root: &Path) {
    fs::create_dir_all(root.join("tools/port")).unwrap();
    fs::write(
        root.join("tools/dependencies.toml"),
        format!(
            "[[dependencies]]\nname = 'snesrev-sm'\nrevision = '{}'\nsha256 = '{}'\n",
            "a".repeat(40),
            "c".repeat(64)
        ),
    )
    .unwrap();
    fs::write(
        root.join(INVENTORY),
        format!(
            "schema_version = 1\ndefinition_count = 1\nscope = 'all sm_*.c definitions'\n\
         [source_identity]\nrevision = '{}'\narchive_sha256 = '{}'\nfreshness = 'pinned archive'\n\
         [source_identity.files]\n'src/sm_80.c' = '{}'\n\
         [banks.'src/sm_80.c']\ndefinitions = 1\nbody_lines = 10\n\
         [[routines]]\nsymbol = 'First'\nsource = 'src/sm_80.c'\nline = 1\n\
         signature = 'void First(void)'\nbody_lines = 10\ndefinition_sha256 = '{}'\n\
         calls = []\nindirect_indexed_calls = []\ndefined_dependencies = []\n\
         external_or_macro_calls = []\ntranslation_status = 'unverified'\n",
            "a".repeat(40),
            "c".repeat(64),
            "d".repeat(64),
            "e".repeat(64)
        ),
    )
    .unwrap();
    fs::write(
        root.join("tools/port/declared.toml"),
        "family = 'declaration'\n[[routines]]\nsymbol = 'First'\n[[routines]]\nsymbol = 'HostWrapper'\n",
    )
    .unwrap();
}

#[test]
fn declarations_skip_large_fixture_payloads_and_preserve_symbols() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.0.join("tools/port")).unwrap();
    let mut manifest = String::from("family = 'large'\n[[routines]]\nsymbol = 'First'\n[[routines.cases]]\ndata = [\n");
    for _ in 0..2_000_000 {
        manifest.push_str("0,");
    }
    manifest.push_str("\n]\n[[routines]]\nsymbol = 'First'\n[[routines]]\nsymbol = 'Unknown'\n");
    fs::write(scratch.0.join("tools/port/large.toml"), manifest).unwrap();
    let mut routines = registry(&scratch.0).routines;
    let mut notices = Vec::new();
    let started = Instant::now();
    add_declarations(&scratch.0, &mut routines, &mut notices).unwrap();
    let elapsed = started.elapsed();
    assert_eq!(routines[0].declared_in, ["tools/port/large.toml"]);
    assert!(routines[1].declared_in.is_empty());
    assert_eq!(
        notices,
        ["Declaration tools/port/large.toml names Unknown outside the C inventory; it contributes no C coverage."]
    );
    eprintln!("four-megabyte progress declaration scan: {elapsed:?}");
    assert!(elapsed < Duration::from_secs(2), "progress decoded fixture payloads: {elapsed:?}");
}

#[test]
fn declaration_projection_preserves_toml_metadata_layouts_and_validation() {
    let manifests = [
        "family = 'frame'\n[[routines]]\nsymbol = 'First'\n[[routines]]\nsymbol = 'Second'\n",
        "'family' = 'frame' # comment\n[[\"routines\"]] # header\n'symbol' = 'First'\n[[routines.cases]]\nstate = [1, 2]\n[[routines]]\nsymbol = 'Second'\n",
        "family = 'frame'\nroutines = [{symbol = 'First', cases = [{state = [1, 2]}]}, {'symbol' = 'Second', note = 'a,=#[x]'}]\n",
        "family = 'frame'\n[[routines]]\nsymbol = 'First'\n[routines.symbol]\nvalue = 'invalid symbol table'\n",
        "family.value = 'invalid family table'\n[[routines]]\nsymbol = 'First'\n",
        "[family]\nname = 'invalid family table'\n",
        "[routines]\nsymbol = 'invalid routine table'\n",
        "family = 7\n[[routines]]\nsymbol = 'First'\n",
        "family = 'frame'\nroutines = [1]\n",
        "family = 'frame'\n[[routines]]\nsymbol = 7\n",
        "family = 'frame'\n[[routines]]\n",
        "family = 'frame'\nfamily = 'duplicate'\n",
        "family = 'frame'\n[[routines]]\nsymbol = 'First'\nsymbol = 'Second'\n",
        "family = 'frame'\nroutines = [{symbol = 'First', symbol = 'Second'}]\n",
        "family = 'frame'\nroutines = []\n[[routines]]\nsymbol = 'First'\n",
        "[unrelated]\nfamily = 7\n[[unrelated.routines]]\nsymbol = 7\n",
    ];
    for manifest in manifests {
        let complete = toml::from_str::<Declaration>(manifest);
        let projected = declarations::parse(manifest);
        assert_eq!(complete.is_ok(), projected.is_ok(), "metadata validation differs for {manifest:?}");
        if let (Ok(complete), Ok(projected)) = (complete, projected) {
            assert_eq!(complete.family, projected.family, "family differs for {manifest:?}");
            let symbols = |declaration: Declaration| {
                declaration.routines.into_iter().map(|routine| routine.symbol).collect::<Vec<_>>()
            };
            assert_eq!(symbols(complete), symbols(projected), "symbols differ for {manifest:?}");
        }
    }
}

#[test]
fn ignored_fixture_strings_comments_and_arrays_cannot_declare_routines() {
    let manifest = r#"
family = "frame"
[[routines]]
symbol = "First"
[[routines.cases]]
note = '''
[[routines]]
symbol = 'Unknown'
'''
other = """
[[routines]]
symbol = "Other"
"""
state = [
    1, # [[routines]]
    {note = 'symbol = "Other"', nested = [2, 3]},
]
# [[routines]]
[[routines]]
symbol = "Second"
"#;
    let projected = declarations::parse(manifest).unwrap();
    assert_eq!(projected.family.as_deref(), Some("frame"));
    assert_eq!(projected.routines.into_iter().map(|routine| routine.symbol).collect::<Vec<_>>(), ["First", "Second"]);
    for malformed in [
        "family = 'frame'\n[[routines]]\nsymbol = 'First'\nstate = [1, 2\n",
        "family = 'frame'\n[[routines]]\nsymbol = 'First'\nstate = 'unterminated\n",
    ] {
        assert!(declarations::parse(malformed).is_err());
    }
}

#[test]
fn refresh_covers_inventory_and_validates_pin_and_definition_totals() {
    let scratch = Scratch::new();
    inventory_root(&scratch.0);
    let options = scratch.options();
    let first = refresh(&options).unwrap();
    assert_eq!(first.routines.len(), 1);
    assert_eq!(first.summary.declared_definitions, 1);
    assert!(first.summary.stages.values().all(|c| c.definitions == 0));
    assert!(first.notices.iter().any(|v| v.contains("HostWrapper")));
    assert!(options.html.exists());
    let original = fs::read_to_string(scratch.0.join(INVENTORY)).unwrap();
    fs::write(scratch.0.join(INVENTORY), original.replacen("definition_count = 1", "definition_count = 2", 1)).unwrap();
    assert!(refresh(&options).unwrap_err().to_string().contains("definition count differs"));
    assert_eq!(read_registry(&options).unwrap().routines.len(), 1);
    fs::write(scratch.0.join(INVENTORY), original.replace(&"a".repeat(40), &"b".repeat(40))).unwrap();
    assert!(refresh(&options).unwrap_err().to_string().contains("pinned C source"));
}

#[test]
fn inventory_refresh_retains_historical_stage_observations_without_current_coverage() {
    let scratch = Scratch::new();
    inventory_root(&scratch.0);
    let options = scratch.options();
    let mut current = refresh(&options).unwrap();
    let mut update = observation(&scratch.0, "historical", "First", Stage::Verified);
    update.source_revision = current.source.revision.clone();
    update.inventory_sha256 = current.source.inventory_sha256.clone();
    insert_update(&mut current, update.clone()).unwrap();
    observe_evidence(&scratch.0, &mut current).unwrap();
    persist(&options, &current).unwrap();
    assert_eq!(current.summary.stages["verified"].definitions, 1);
    let inventory = scratch.0.join(INVENTORY);
    let bytes = fs::read_to_string(&inventory)
        .unwrap()
        .replace("scope = 'all sm_*.c definitions'", "scope = 'corrected lexical inventory'");
    fs::write(inventory, bytes).unwrap();
    let refreshed = refresh(&options).unwrap();
    assert_eq!(refreshed.updates, [update]);
    assert_eq!(refreshed.stale_update_ids, ["historical"]);
    assert_eq!(refreshed.summary.stages["verified"].definitions, 0);
    assert!(refreshed.notices.iter().any(|notice| notice.contains("source inventory changed")));
}

#[test]
fn update_command_rechecks_retained_evidence_and_source_before_publication() {
    let scratch = Scratch::new();
    inventory_root(&scratch.0);
    let mut options = scratch.options();
    let initial = refresh(&options).unwrap();
    fs::write(scratch.0.join("evidence.txt"), b"retained evidence").unwrap();
    let path = scratch.0.join("update.toml");
    let payload = format!(
        "id = 'scope-one-translated'\nscope = 'camera scroll ownership'\nagent = 'worker'\n\
         stage = 'translated'\nroutines = ['First']\nsource_revision = '{}'\n\
         inventory_sha256 = '{}'\nlimitations = ['offline only']\n\
         [[evidence]]\npath = 'evidence.txt'\nsha256 = '{}'\n",
        initial.source.revision,
        initial.source.inventory_sha256,
        crate::digest(&scratch.0.join("evidence.txt")).unwrap()
    );
    fs::write(&path, &payload).unwrap();
    options.update = Some(path.clone());
    assert_eq!(update(&options).unwrap().summary.stages["translated"].definitions, 1);
    assert_eq!(update(&options).unwrap().updates.len(), 1);
    fs::write(
        &path,
        payload.replace("['First']", "['Unknown']").replace("scope-one-translated", "scope-two-translated"),
    )
    .unwrap();
    assert!(update(&options).unwrap_err().to_string().contains("unknown C routine"));
    fs::write(&path, &payload).unwrap();
    fs::write(scratch.0.join("evidence.txt"), b"changed evidence").unwrap();
    assert!(update(&options).unwrap_err().to_string().contains("evidence digest differs"));
    assert_eq!(read_registry(&options).unwrap().updates.len(), 1);
}

#[test]
fn stages_are_independent_and_declarations_do_not_grant_progress() {
    let scratch = Scratch::new();
    let mut registry = registry(&scratch.0);
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.summary.declared_definitions, 2);
    assert!(registry.summary.stages.values().all(|c| c.definitions == 0));
    insert_update(&mut registry, observation(&scratch.0, "one", "First", Stage::Translated)).unwrap();
    insert_update(&mut registry, observation(&scratch.0, "two", "Second", Stage::Integrated)).unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.summary.stages["translated"].definitions, 1);
    assert_eq!(registry.summary.stages["integrated"].definitions, 1);
    assert_eq!(registry.summary.stages["verified"].definitions, 0);
    assert_eq!(registry.summary.stages["live-exercised"].definitions, 0);
}

#[test]
fn idempotent_retry_preserves_timestamp_and_conflicting_id_is_rejected() {
    let scratch = Scratch::new();
    let mut registry = registry(&scratch.0);
    let update = observation(&scratch.0, "one", "First", Stage::Translated);
    assert!(insert_update(&mut registry, update.clone()).unwrap());
    let mut retry = update.clone();
    retry.updated_at = 100;
    assert!(!insert_update(&mut registry, retry).unwrap());
    assert_eq!(registry.updates.len(), 1);
    assert_eq!(registry.updates[0].updated_at, 2);
    let mut conflict = update;
    conflict.stage = Stage::Verified;
    assert!(insert_update(&mut registry, conflict).unwrap_err().to_string().contains("conflicting"));
}

#[test]
fn evidence_context_defaults_preserve_legacy_observations_and_retries() {
    let scratch = Scratch::new();
    let legacy = observation(&scratch.0, "legacy", "First", Stage::Verified);
    let bytes = serde_json::to_string(&legacy).unwrap();
    assert!(!bytes.contains("evidence_context"));
    assert!(!bytes.contains("supersedes"));
    let restored: Update = serde_json::from_str(&bytes).unwrap();
    assert_eq!(restored.evidence_context, EvidenceContext::Unspecified);
    assert!(restored.supersedes.is_empty());
    assert_eq!(serde_json::to_string(&restored).unwrap(), bytes);
    let mut registry = registry(&scratch.0);
    insert_update(&mut registry, restored.clone()).unwrap();
    let mut retry = restored.clone();
    retry.updated_at = 99;
    assert!(!insert_update(&mut registry, retry).unwrap());
    let mut relabeled = restored;
    relabeled.evidence_context = EvidenceContext::CurrentCheckout;
    assert!(insert_update(&mut registry, relabeled).is_err());
}

#[test]
fn evidence_context_counts_overlap_and_exclude_stale_evidence() {
    let scratch = Scratch::new();
    let mut registry = registry(&scratch.0);
    let mut historical = observation(&scratch.0, "historical", "First", Stage::Verified);
    historical.evidence_context = EvidenceContext::HistoricalFrozen;
    let mut current = observation(&scratch.0, "current", "Second", Stage::Verified);
    current.routines.push("First".into());
    current.evidence_context = EvidenceContext::CurrentCheckout;
    insert_update(&mut registry, historical).unwrap();
    insert_update(&mut registry, current).unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.summary.stages["verified"].definitions, 2);
    assert_eq!(registry.summary.context_stages["historical-frozen"]["verified"].definitions, 1);
    assert_eq!(registry.summary.context_stages["current-checkout"]["verified"].definitions, 2);
    assert_eq!(registry.summary.context_stages["unspecified"]["verified"].definitions, 0);
    fs::write(scratch.0.join("evidence.txt"), b"changed").unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.summary.stages["verified"].definitions, 0);
    assert!(
        registry.summary.context_stages.values().flat_map(|stages| stages.values()).all(|count| count.definitions == 0)
    );
}

#[test]
fn reviewed_supersession_preserves_history_without_resurrecting_stale_claims() {
    let scratch = Scratch::new();
    let options = scratch.options();
    let mut registry = registry(&scratch.0);
    let mut original = observation(&scratch.0, "overstated", "First", Stage::Translated);
    original.routines.push("Second".into());
    let original_bytes = serde_json::to_string(&original).unwrap();
    insert_update(&mut registry, original.clone()).unwrap();
    let mut reviewed = observation(&scratch.0, "reviewed", "Second", Stage::Translated);
    reviewed.evidence_context = EvidenceContext::CurrentCheckout;
    fs::write(scratch.0.join("review.txt"), b"reviewed source correspondence").unwrap();
    reviewed.evidence =
        vec![Evidence { path: "review.txt".into(), sha256: crate::digest(&scratch.0.join("review.txt")).unwrap() }];
    let mut payload = serde_json::to_value(&reviewed).unwrap();
    payload["supersedes"] = serde_json::json!(["overstated"]);
    let reviewed: Update = serde_json::from_value(payload).unwrap();
    insert_update(&mut registry, reviewed).unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.summary.stages["translated"].definitions, 1);
    assert_eq!(registry.summary.stages["translated"].body_lines, 20);
    assert_eq!(registry.superseded_update_ids, ["overstated"]);
    assert_eq!(registry.summary.context_stages["unspecified"]["translated"].definitions, 0);
    assert_eq!(registry.summary.context_stages["current-checkout"]["translated"].definitions, 1);
    assert_eq!(serde_json::to_string(&registry.updates[0]).unwrap(), original_bytes);
    assert!(
        registry.notices.iter().any(|notice| notice.contains("Superseded update overstated: replaced by reviewed"))
    );

    fs::write(scratch.0.join("review.txt"), b"source changed after review").unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.stale_update_ids, ["reviewed"]);
    assert_eq!(registry.superseded_update_ids, ["overstated"]);
    assert_eq!(registry.summary.stages["translated"].definitions, 0);
    assert!(registry.summary.context_stages.values().all(|stages| stages["translated"].definitions == 0));
    persist(&options, &registry).unwrap();
    let mut restored = read_registry(&options).unwrap();
    observe_evidence(&scratch.0, &mut restored).unwrap();
    assert_eq!(restored.summary.stages["translated"].definitions, 0);
    assert_eq!(restored.updates[0], original);
}

#[test]
fn supersession_requires_existing_same_stage_inventory_targets_and_rejects_cycles() {
    let scratch = Scratch::new();
    let options = scratch.options();
    let mut registry = registry(&scratch.0);
    insert_update(&mut registry, observation(&scratch.0, "original", "First", Stage::Translated)).unwrap();
    let mut reviewed = observation(&scratch.0, "reviewed", "Second", Stage::Translated);
    reviewed.supersedes = vec!["absent".into()];
    assert!(insert_update(&mut registry, reviewed.clone()).unwrap_err().to_string().contains("unknown superseded"));
    reviewed.supersedes = vec!["reviewed".into()];
    assert!(insert_update(&mut registry, reviewed.clone()).unwrap_err().to_string().contains("itself"));
    reviewed.supersedes = vec!["original".into(), "original".into()];
    assert!(insert_update(&mut registry, reviewed.clone()).unwrap_err().to_string().contains("duplicate superseded"));
    reviewed.supersedes = vec!["original".into()];
    reviewed.stage = Stage::Verified;
    assert!(insert_update(&mut registry, reviewed.clone()).unwrap_err().to_string().contains("different stage"));
    reviewed.stage = Stage::Translated;
    registry.updates[0].inventory_sha256 = "f".repeat(64);
    assert!(
        insert_update(&mut registry, reviewed.clone()).unwrap_err().to_string().contains("different source inventory")
    );
    registry.updates[0].inventory_sha256 = registry.source.inventory_sha256.clone();
    insert_update(&mut registry, reviewed.clone()).unwrap();
    let mut retry = reviewed;
    retry.updated_at = 99;
    assert!(!insert_update(&mut registry, retry).unwrap());
    let mut final_review = observation(&scratch.0, "final-review", "Second", Stage::Translated);
    final_review.supersedes = vec!["reviewed".into()];
    insert_update(&mut registry, final_review).unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.superseded_update_ids, ["original", "reviewed"]);
    assert_eq!(registry.summary.stages["translated"].definitions, 1);
    persist(&options, &registry).unwrap();
    read_registry(&options).unwrap();

    // A corrupt persisted history cannot bypass the append-time graph rules.
    registry.updates[0].supersedes = vec!["final-review".into()];
    persist(&options, &registry).unwrap();
    assert!(read_registry(&options).unwrap_err().to_string().contains("supersession cycle"));
    assert!(observe_evidence(&scratch.0, &mut registry).unwrap_err().to_string().contains("supersession cycle"));
}

#[test]
fn update_command_publishes_reviewed_correction_and_preserves_legacy_serialization() {
    let scratch = Scratch::new();
    inventory_root(&scratch.0);
    let mut options = scratch.options();
    let mut initial = refresh(&options).unwrap();
    let mut original = observation(&scratch.0, "original", "First", Stage::Translated);
    original.inventory_sha256 = initial.source.inventory_sha256.clone();
    insert_update(&mut initial, original.clone()).unwrap();
    observe_evidence(&scratch.0, &mut initial).unwrap();
    persist(&options, &initial).unwrap();
    assert!(!fs::read_to_string(&options.registry).unwrap().contains("superseded_update_ids"));
    let mut payload = serde_json::to_value(original).unwrap();
    payload["id"] = serde_json::json!("reviewed");
    payload["supersedes"] = serde_json::json!(["original"]);
    payload.as_object_mut().unwrap().remove("updated_at");
    let path = scratch.0.join("reviewed.toml");
    fs::write(&path, toml::to_string(&payload).unwrap()).unwrap();
    options.update = Some(path);
    let reviewed = update(&options).unwrap();
    assert_eq!(reviewed.superseded_update_ids, ["original"]);
    assert_eq!(reviewed.updates.len(), 2);
    assert_eq!(reviewed.summary.stages["translated"].definitions, 1);
    assert_eq!(update(&options).unwrap().updates, reviewed.updates);
    assert!(fs::read_to_string(options.html).unwrap().contains("Superseded update original"));
}

#[test]
fn stale_source_unknown_symbols_and_missing_evidence_are_rejected() {
    let scratch = Scratch::new();
    let mut registry = registry(&scratch.0);
    let unknown = observation(&scratch.0, "one", "Unknown", Stage::Translated);
    assert!(insert_update(&mut registry, unknown).unwrap_err().to_string().contains("unknown C routine"));
    let mut stale = observation(&scratch.0, "two", "First", Stage::Translated);
    stale.inventory_sha256 = "f".repeat(64);
    assert!(insert_update(&mut registry, stale).unwrap_err().to_string().contains("stale"));
    assert!(evidence_path(&scratch.0, "absent.txt").is_err());
    assert!(evidence_path(&scratch.0, "../evidence.txt").is_err());
    insert_update(&mut registry, observation(&scratch.0, "three", "First", Stage::Verified)).unwrap();
    fs::write(scratch.0.join("evidence.txt"), b"changed after publication").unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert_eq!(registry.summary.stale_updates, 1);
    assert_eq!(registry.summary.stages["verified"].definitions, 0);
    assert!(registry.notices.iter().any(|v| v.contains("evidence changed")));
    registry.source.inventory_sha256 = "f".repeat(64);
    observe_evidence(&scratch.0, &mut registry).unwrap();
    assert!(registry.notices.iter().any(|v| v.contains("source inventory changed")));
}

#[test]
fn concurrent_writers_preserve_scope_observations_and_valid_json() {
    let scratch = Scratch::new();
    let options = scratch.options();
    fs::write(scratch.0.join("evidence.txt"), b"retained evidence").unwrap();
    persist(&options, &registry(&scratch.0)).unwrap();
    let handles = (0..6)
        .map(|index| {
            let options = options.clone();
            std::thread::spawn(move || {
                let _lock = registry_lock(&options.registry, Duration::from_secs(3)).unwrap();
                let mut registry = read_registry(&options).unwrap();
                let mut update = observation(&options.root, &format!("agent-{index}"), "First", Stage::Translating);
                update.scope = format!("scope-{index}");
                update.agent = format!("agent-{index}");
                insert_update(&mut registry, update).unwrap();
                observe_evidence(&options.root, &mut registry).unwrap();
                persist(&options, &registry).unwrap();
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.join().unwrap();
    }
    let registry = read_registry(&options).unwrap();
    assert_eq!(registry.updates.len(), 6);
    assert_eq!(registry.summary.stages["translating"].definitions, 1);
    assert_eq!(registry.updates.iter().map(|u| &u.scope).collect::<BTreeSet<_>>().len(), 6);
    assert!(options.registry.with_extension("json.lock").exists());
    assert!(fs::read_to_string(options.html).unwrap().contains("agent-5"));
}

#[test]
fn contention_is_bounded_and_lock_releases_after_owner_exit() {
    let scratch = Scratch::new();
    let path = scratch.0.join("registry.json");
    let lock = registry_lock(&path, Duration::from_secs(1)).unwrap();
    assert!(registry_lock(&path, Duration::from_millis(30)).unwrap_err().to_string().contains("timed out"));
    drop(lock);
    registry_lock(&path, Duration::from_millis(30)).unwrap();
}

#[test]
fn admitted_progress_preserves_exact_stages_and_repairs_only_accepted_observations() {
    let scratch = Scratch::new();
    inventory_root(&scratch.0);
    fs::create_dir_all(scratch.0.join(".scratch/decomp-progress")).unwrap();
    let options = admitted_options(&scratch.0);
    let registry = refresh(&options).unwrap();
    let evidence = scratch.0.join("completion.json");
    fs::write(&evidence, b"{\"accepted_fixture_domain\":true}").unwrap();
    let pointers = [(evidence.clone(), crate::digest(&evidence).unwrap())];
    let publication = prepare_admitted(
        &scratch.0,
        "original-claim-attempt-one",
        "coherent frame",
        "original-owner",
        &["First".into()],
        &registry.source.revision,
        &registry.source.inventory_sha256,
        &"d".repeat(40),
        &"e".repeat(40),
        &pointers,
        &["actual-C fixture domain; Live replay not tested".into()],
    )
    .unwrap();
    let mut publication = publication;
    for (index, update) in publication.updates.iter_mut().enumerate() {
        update.updated_at = 1_000_000 + index as u64;
    }
    let baseline = scratch.0.join("original-progress.json");
    fs::copy(&options.registry, &baseline).unwrap();
    validate_admitted_plan(
        &publication,
        "original-claim-attempt-one",
        "coherent frame",
        "original-owner",
        &["First".into()],
        &registry.source.revision,
        &registry.source.inventory_sha256,
        &"d".repeat(40),
        &"e".repeat(40),
        &pointers,
        &["actual-C fixture domain; Live replay not tested".into()],
    )
    .unwrap();
    validate_admitted_projection(&scratch.0, &publication, &baseline).unwrap();
    assert!(read_registry(&options).unwrap().updates.is_empty());
    let ids = publish_admitted(&scratch.0, &publication).unwrap();
    authenticate_admitted(&scratch.0, &publication).unwrap();
    validate_admitted_projection(&scratch.0, &publication, &baseline).unwrap();
    let before = fs::read(&options.registry).unwrap();
    assert_eq!(publish_admitted(&scratch.0, &publication).unwrap(), ids);
    assert_eq!(fs::read(&options.registry).unwrap(), before);
    let saved = read_registry(&options).unwrap();
    assert_eq!(saved.summary.stages["translated"].definitions, 1);
    assert_eq!(saved.summary.stages["verified"].definitions, 1);
    assert_eq!(saved.summary.stages["integrated"].definitions, 1);
    assert_eq!(saved.summary.stages["live-exercised"].definitions, 0);
    fs::remove_file(&options.html).unwrap();
    assert!(authenticate_admitted(&scratch.0, &publication).is_err());
    validate_admitted_projection(&scratch.0, &publication, &baseline).unwrap();
    assert_eq!(publish_admitted(&scratch.0, &publication).unwrap(), ids);
    authenticate_admitted(&scratch.0, &publication).unwrap();
    let mut lost_projection = read_registry(&options).unwrap();
    lost_projection.updates.retain(|update| update.stage != Stage::Verified);
    observe_evidence(&scratch.0, &mut lost_projection).unwrap();
    persist(&options, &lost_projection).unwrap();
    assert!(authenticate_admitted(&scratch.0, &publication).is_err());
    validate_admitted_projection(&scratch.0, &publication, &baseline).unwrap();
    publish_admitted(&scratch.0, &publication).unwrap();
    authenticate_admitted(&scratch.0, &publication).unwrap();
    validate_admitted_projection(&scratch.0, &publication, &baseline).unwrap();
    let exact_registry = fs::read(&options.registry).unwrap();
    let mut unrelated = read_registry(&options).unwrap();
    unrelated.notices.push("unrelated mutable controller text".into());
    persist(&options, &unrelated).unwrap();
    assert!(validate_admitted_projection(&scratch.0, &publication, &baseline).is_err());
    fs::write(&options.registry, exact_registry).unwrap();
    let restored = read_registry(&options).unwrap();
    persist(&options, &restored).unwrap();
    let mut conflict = publication.clone();
    conflict.updates[0].routines = vec!["Second".into()];
    let before = fs::read(&options.registry).unwrap();
    assert!(publish_admitted(&scratch.0, &conflict).is_err());
    assert_eq!(fs::read(&options.registry).unwrap(), before);
    fs::write(&evidence, b"tampered completion").unwrap();
    assert!(publish_admitted(&scratch.0, &publication).is_err());
    assert!(authenticate_admitted(&scratch.0, &publication).is_err());
    assert_eq!(fs::read(&options.registry).unwrap(), before);
}

#[test]
fn malformed_updates_and_unsafe_embedding_are_handled() {
    let scratch = Scratch::new();
    let mut registry = registry(&scratch.0);
    let mut update = observation(&scratch.0, "one", "First", Stage::Integrated);
    update.commit = None;
    assert!(validate_update_shape(&update).is_err());
    assert!(toml::from_str::<UpdateInput>("id = 'one'\nstage = 'accepted'\n").is_err());
    assert!(options("status", &["--unexpected".into(), "value".into()]).is_err());
    registry.notices.push("</script><script>untrusted()</script>".into());
    let html = render(&registry).unwrap();
    assert!(!html.contains("</script><script>untrusted()"));
    assert!(html.contains("\\u003c/script>"));
}

#[test]
fn server_routes_only_bounded_gets_without_directory_access() {
    assert_eq!(request_target(b"GET /registry.json?time=1 HTTP/1.1\r\n\r\n").unwrap(), "/registry.json");
    assert!(request_target(b"POST /registry.json HTTP/1.1\r\n\r\n").is_err());
    assert!(request_target(b"GET / HTTP/1.1 extra\r\n\r\n").is_err());
    let bytes = response("200 OK", "application/json", b"{}");
    let response = String::from_utf8(bytes).unwrap();
    assert!(response.contains("Cache-Control: no-store\r\n"));
    assert!(response.contains("Content-Length: 2\r\n"));
}

#[test]
fn live_http_snapshot_excludes_changed_evidence_and_rejects_stale_inventory() {
    let scratch = Scratch::new();
    inventory_root(&scratch.0);
    let options = scratch.options();
    let mut registry = refresh(&options).unwrap();
    let mut update = observation(&scratch.0, "one", "First", Stage::Verified);
    update.source_revision = registry.source.revision.clone();
    update.inventory_sha256 = registry.source.inventory_sha256.clone();
    insert_update(&mut registry, update).unwrap();
    observe_evidence(&scratch.0, &mut registry).unwrap();
    persist(&options, &registry).unwrap();
    let request = |options: &Options, target: &str| {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let options = options.clone();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            respond(&mut stream, &options).unwrap();
        });
        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes()).unwrap();
        let mut text = String::new();
        client.read_to_string(&mut text).unwrap();
        server.join().unwrap();
        text
    };
    let current = request(&options, "/registry.json");
    let current: Registry = serde_json::from_str(current.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(current.summary.stages["verified"].definitions, 1);
    fs::write(scratch.0.join("evidence.txt"), b"changed").unwrap();
    let stale = request(&options, "/registry.json");
    let stale: Registry = serde_json::from_str(stale.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(stale.stale_update_ids, ["one"]);
    assert_eq!(stale.summary.stages["verified"].definitions, 0);
    assert!(request(&options, "/../evidence.txt").starts_with("HTTP/1.1 404"));
    fs::write(scratch.0.join(INVENTORY), b"changed inventory").unwrap();
    assert!(request(&options, "/registry.json").starts_with("HTTP/1.1 409"));
}
