use super::*;
use crate::family_model::Case;

fn contract_for_family(
    checkout: &Path,
    manifest: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
) -> Result<NativeContract> {
    let host = HostInputs::new(
        &[("crates/offline-verification/src/native_tests.rs", include_str!("native_tests.rs"))],
        sha256("native-unit-test-producer"),
    )?;
    OfflineFamilies::new(host).declare_native(NativeDeclaration {
        checkout,
        kind: NativeDeclarationKind::Family(manifest),
        source_inventory,
        provenance,
        compiler_limits,
        fixture_limits,
        deadline_unix_ms: None,
        max_input_bytes: bounded::DEFAULT_BYTES,
    })
}

#[test]
fn native_receipt_storage_preserves_large_operation_sets_within_controller_record_bound() -> Result<()> {
    let directory = tool_execution::testing::Directory::new()?;
    fs::create_dir_all(directory.0.join("family"))?;
    let mut retained = directory.0.join("retained");
    for level in 0..16 {
        retained.push(format!("{level}-{}", "source".repeat(16)));
    }
    fs::create_dir_all(&retained)?;
    let retained = retained.join("result.json");
    fs::write(&retained, b"retained producer storage control")?;
    let reference = translation_execution::retention::seal_file(&retained, 4096)?;
    let context = OperationContext {
        run_id: sha256("receipt-storage"),
        attempt: 1,
        policy_sha256: sha256("frozen-policy"),
        operation_key: sha256("native-family"),
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    let operations = (0..2048)
        .map(|index| {
            let mut operation_context = context.clone();
            operation_context.operation_key = sha256(format!("producer-{index}"));
            NativeOperation {
                directory: directory.0.join(format!("family/pair-{index}")),
                label: "rust".into(),
                invocation: Asset { file: reference.path.clone(), sha256: reference.sha256.clone() },
                context: operation_context,
                result: reference.clone(),
                result_sha256: reference.sha256.clone(),
                references: vec![reference.clone()],
            }
        })
        .collect::<Vec<_>>();
    // This exercises storage only; these declarations grant no producer or
    // Offline acceptance. Actual invocation authentication remains separate.
    let expected = operations.iter().map(|operation| operation.context.operation_key.clone()).collect::<Vec<_>>();
    let operation_index =
        operation_storage::store(&directory.0, &sha256("prepared"), &sha256("candidate"), &context, &operations)?;
    let receipt = NativeReceipt {
        prepared_sha256: sha256("prepared"),
        candidate_sha256: sha256("candidate"),
        verdict: Asset { file: directory.0.join("family/verdict.toml"), sha256: sha256("verdict") },
        passed: false,
        diagnostic: "storage control only".into(),
        evidence_bytes: 0,
        context,
        operation_root: directory.0.join("operations"),
        operation_index,
        max_output_bytes: 64 * 1024 * 1024,
        reserved_bytes: 64 * 1024 * 1024,
    };
    let encoded = serde_json::to_vec(&receipt)?;
    ensure!(
        encoded.len() <= (4 * 1024 * 1024),
        "complete native producer attribution exceeds the controller record bound: {} bytes",
        encoded.len()
    );
    let restored: NativeReceipt = serde_json::from_slice(&encoded)?;
    let mut actual = Vec::new();
    restored.visit_operations(|operation| {
        actual.push(operation.context.operation_key.clone());
        Ok(())
    })?;
    assert_eq!(actual, expected);
    let retained = references(&restored)?;
    ensure!(retained.iter().any(|item| item == &reference), "producer input missing from retention");
    let index = &restored.operation_index;
    ensure!(retained.iter().any(|item| item == index), "operation index missing from retention");
    let storage_bytes = retained
        .iter()
        .filter(|item| item.path.starts_with(directory.0.join("native-operations")))
        .map(|item| item.bytes)
        .sum::<u64>();
    bounded::until(now_unix_ms()? + 60_000, storage_bytes, || {
        assert_eq!(references_bounded(&restored)?, retained);
        assert_eq!(bounded::charged_bytes(), storage_bytes, "attribution was read more than once");
        Ok(())
    })?;
    ensure!(
        bounded::until(now_unix_ms()? + 60_000, storage_bytes, || {
            let bytes = evidence_bytes(&directory.0.join("family"), &restored)?;
            assert_eq!(bounded::charged_bytes(), storage_bytes, "attribution accounting reread storage");
            Ok(bytes)
        })? == retained.iter().map(|item| item.bytes).sum::<u64>(),
        "indexed attribution omitted from aggregate evidence extent"
    );
    println!("Retained all {} operations in a {}-byte controller receipt", expected.len(), encoded.len());
    Ok(())
}

#[test]
fn declared_tree_budget_rejects_inflated_extent_and_coverage() -> Result<()> {
    let checkout = current_checkout();
    let mut manifest =
        recipes::manifest(&checkout, &checkout.join("tools/port/ridley-subsystem.smf"), Some(&NativeRecipe::Ridley))?;
    let limits = declared_tree_limits(&manifest, 64 * 1024 * 1024 * 1024)?;
    assert_eq!((limits.files, limits.paths), (1_003_072, 2_006_144));
    assert!(declared_tree_limits(&manifest, u64::MAX).is_err());
    let mut fixture = manifest.routines[0].cases[0].clone();
    manifest.routines[0].cases.clear();
    for index in 0..16385 {
        fixture.name = format!("case-{index}");
        manifest.routines[0].cases.push(fixture.clone());
    }
    assert!(declared_tree_limits(&manifest, 64 * 1024 * 1024 * 1024).is_err());
    Ok(())
}

#[test]
fn native_receipt_retention_rejects_changed_incomplete_or_redirected_attribution() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let directory = tool_execution::testing::Directory::new()?;
    fs::create_dir_all(directory.0.join("family"))?;
    let context = OperationContext {
        run_id: sha256("retention-storage"),
        attempt: 1,
        policy_sha256: sha256("policy"),
        operation_key: sha256("gate"),
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    let operation = NativeOperation {
        directory: directory.0.join("family/pair"),
        label: "rust".into(),
        invocation: Asset { file: directory.0.join("invocation"), sha256: sha256("invocation") },
        context: context.clone(),
        result: Reference { path: directory.0.join("result"), sha256: sha256("result"), bytes: 6 },
        result_sha256: sha256("result"),
        references: vec![],
    };
    let operation_index = operation_storage::store(
        &directory.0,
        &sha256("prepared"),
        &sha256("candidate"),
        &context,
        std::slice::from_ref(&operation),
    )?;
    let receipt = NativeReceipt {
        prepared_sha256: sha256("prepared"),
        candidate_sha256: sha256("candidate"),
        verdict: Asset { file: directory.0.join("family/verdict.toml"), sha256: sha256("verdict") },
        passed: false,
        diagnostic: "storage only; no native acceptance".into(),
        evidence_bytes: 0,
        context,
        operation_root: directory.0.join("operations"),
        operation_index,
        max_output_bytes: 16 * 1024 * 1024,
        reserved_bytes: 16 * 1024 * 1024,
    };
    let mut inline = serde_json::to_value(&receipt)?;
    inline["operations"] = serde_json::json!([operation]);
    assert!(serde_json::from_value::<NativeReceipt>(inline.clone()).is_err());
    inline.as_object_mut().unwrap().remove("operation_index");
    assert!(serde_json::from_value::<NativeReceipt>(inline).is_err());
    references(&receipt)?;
    let index = receipt.operation_index.clone();
    let index_bytes = fs::read(&index.path)?;
    let chunk = directory.0.join("native-operations/00000.json");
    let chunk_bytes = fs::read(&chunk)?;
    fs::set_permissions(&index.path, fs::Permissions::from_mode(0o600))?;
    fs::set_permissions(&chunk, fs::Permissions::from_mode(0o600))?;

    let mut changed = receipt.clone();
    changed.operation_index.path = directory.0.join("foreign-index.json");
    ensure!(references(&changed).is_err(), "redirected index admitted");
    changed = receipt.clone();
    changed.candidate_sha256 = sha256("foreign candidate");
    ensure!(references(&changed).is_err(), "index admitted a different candidate");
    changed = receipt.clone();
    changed.context.operation_key = sha256("foreign operation");
    ensure!(references(&changed).is_err(), "index admitted a different operation");

    let extra = directory.0.join("native-operations/00001.json");
    fs::write(&extra, &chunk_bytes)?;
    ensure!(references(&receipt).is_err(), "unlisted operation chunk admitted");
    fs::remove_file(&extra)?;
    fs::remove_file(&chunk)?;
    ensure!(references(&receipt).is_err(), "missing operation chunk admitted");
    fs::write(&chunk, &chunk_bytes)?;
    fs::write(&chunk, b"changed operation chunk")?;
    ensure!(references(&receipt).is_err(), "changed operation chunk admitted");

    // A newly sealed corrupt index cannot redirect a chunk or erase the
    // complete ordered count. Independent producer checks still follow.
    let original_index: serde_json::Value = serde_json::from_slice(&index_bytes)?;
    for alteration in ["redirect", "omit", "ordering", "extra-operation"] {
        fs::write(&chunk, &chunk_bytes)?;
        let mut value = original_index.clone();
        match alteration {
            "redirect" => value["chunks"][0]["path"] = serde_json::json!(directory.0.join("foreign.json")),
            "omit" => value["chunks"] = serde_json::json!([]),
            _ => {
                let mut value_chunk: serde_json::Value = serde_json::from_slice(&chunk_bytes)?;
                if alteration == "ordering" {
                    value_chunk["first"] = serde_json::json!(1);
                } else {
                    let duplicate = value_chunk["operations"][0].clone();
                    value_chunk["operations"].as_array_mut().unwrap().push(duplicate);
                }
                let bytes = serde_json::to_vec(&value_chunk)?;
                fs::write(&chunk, &bytes)?;
                value["chunks"][0]["sha256"] = serde_json::json!(sha256(&bytes));
                value["chunks"][0]["bytes"] = serde_json::json!(bytes.len());
            }
        }
        let bytes = serde_json::to_vec(&value)?;
        fs::write(&index.path, &bytes)?;
        changed = receipt.clone();
        changed.operation_index =
            Reference { path: index.path.clone(), sha256: sha256(&bytes), bytes: bytes.len() as u64 };
        ensure!(references(&changed).is_err(), "{alteration} admitted by native retention");
    }
    fs::write(&index.path, &index_bytes)?;
    fs::write(&chunk, &chunk_bytes)?;
    references(&receipt)?;
    Ok(())
}

#[test]
fn frozen_shared_host_contracts_preserve_both_original_matrices() -> Result<()> {
    let checkout = current_checkout();
    let limits = NativeLimits {
        timeout_seconds: 60,
        writable_bytes: 128 * 1024 * 1024,
        stdout_bytes: 4096,
        stderr_bytes: 256 * 1024,
        stdin_bytes: 0,
    };
    let mut pairs = 0;
    for name in ["game-host-projectile", "game-host-room-scroll"] {
        let contract = contract_for_family(
            &checkout,
            &PathBuf::from(format!("tools/port/{name}.smf")),
            Asset::default(),
            NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
            limits.clone(),
            limits.clone(),
        )?;
        pairs += contract.coverage.values().map(Vec::len).sum::<usize>();
    }
    assert_eq!(pairs, 10);
    Ok(())
}

#[test]
fn discovered_recipe_rejects_a_declaration_from_an_alias_path() -> Result<()> {
    let directory = tool_execution::testing::Directory::new()?;
    let checkout = &directory.0;
    fs::create_dir_all(checkout.join("tools/port"))?;
    fs::write(checkout.join("tools/port/batch.toml"), "[families]\n[profiles]\n")?;
    let manifest = Manifest { family: "registered".into(), ..Default::default() };
    crate::fixture_format::write(&checkout.join("tools/port/alias.smf"), &manifest)?;
    let limits = NativeLimits {
        timeout_seconds: 1,
        writable_bytes: 1024 * 1024,
        stdout_bytes: 4096,
        stderr_bytes: 4096,
        stdin_bytes: 0,
    };
    let error = contract_for_family(
        checkout,
        Path::new("tools/port/alias.smf"),
        Asset::default(),
        NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
        limits.clone(),
        limits,
    )
    .unwrap_err();
    ensure!(error.to_string().contains("registration path differs"), "unexpected rejection: {error:#}");
    Ok(())
}

#[test]
fn registered_catalog_alias_preserves_declared_family_and_coverage() -> Result<()> {
    let checkout = current_checkout();
    let path = Path::new("tools/port/instructions.smf");
    let declared = crate::fixture_format::read(&checkout.join(path))?;
    let limits = NativeLimits {
        timeout_seconds: 60,
        writable_bytes: 128 * 1024 * 1024,
        stdout_bytes: 4096,
        stderr_bytes: 4096,
        stdin_bytes: 0,
    };
    let contract = contract_for_family(
        &checkout,
        path,
        Asset::default(),
        NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
        limits.clone(),
        limits,
    )?;
    assert_eq!(contract.family, "enemy-instructions");
    assert_eq!(contract.coverage, coverage(&declared)?);
    let registration =
        recipes::registration(&checkout, &contract.manifest, &contract.family, contract.recipe.as_ref(), &declared)?;
    assert_eq!(registration.catalog_family, "instructions");
    assert_eq!(registration.entry.manifest, path);
    Ok(())
}

#[test]
fn discovered_captured_ridley_keeps_its_original_callback_recipe() -> Result<()> {
    let checkout = current_checkout();
    let path = Path::new("tools/port/ridley-subsystem-captured.smf");
    let original = crate::fixture_format::read(&checkout.join(path))?;
    let recipe = recipes::for_family(&checkout, path, &original)?;
    assert_eq!(recipe, Some(NativeRecipe::Ridley));
    let normalized = recipes::normalize_manifest(&checkout, original.clone(), recipe.as_ref())?;
    assert_eq!(normalized, original);
    assert!(normalized.routines.iter().flat_map(|routine| &routine.cases).any(|case| case.before_file.is_some()));
    let limits = NativeLimits {
        timeout_seconds: 1,
        writable_bytes: 1024 * 1024,
        stdout_bytes: 4096,
        stderr_bytes: 4096,
        stdin_bytes: 0,
    };
    let contract = contract_for_family(
        &checkout,
        path,
        Asset::default(),
        NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
        limits.clone(),
        limits,
    )?;
    assert_eq!(contract.coverage, coverage(&original)?);
    assert!(original.rust_dependencies.len() > 4096);
    assert!(contract.frozen_inputs.len() > 4096 && contract.frozen_inputs.len() <= 8192);
    let event = Path::new(inputs::CAPTURED_RIDLEY_EVENT);
    assert!(event.metadata()?.len() > bounded::MAX_FILE_BYTES);
    assert!(recipes::dependency_digest(None, event).is_err());
    assert!(recipes::dependency_digest(Some(&NativeRecipe::Callback), event).is_err());
    assert_eq!(
        contract.frozen_inputs[inputs::CAPTURED_RIDLEY_EVENT],
        recipes::dependency_digest(Some(&NativeRecipe::Ridley), event)?
    );
    let mut unknown = original;
    unknown.callback_adapter = Some("unknown-captured-adapter".into());
    assert!(recipes::for_family(&checkout, path, &unknown).is_err());
    Ok(())
}

#[test]
fn scope_candidate_dependencies_are_mutable_and_bound_to_receipt_identity() -> Result<()> {
    let directory = tool_execution::testing::Directory::new()?;
    let checkout = &directory.0;
    fs::create_dir_all(checkout.join("crates/port/src"))?;
    let names = ["root", "helper", "fixed"].map(|name| format!("crates/port/src/{name}.rs"));
    let mut frozen_inputs = BTreeMap::new();
    for name in &names {
        fs::write(checkout.join(name), format!("// original {name}"))?;
        frozen_inputs.insert(name.clone(), digest(&checkout.join(name))?);
    }
    crate::fixture_format::write(
        &checkout.join("manifest.smf"),
        &Manifest { family: "test".into(), ..Default::default() },
    )?;
    let limits =
        NativeLimits { timeout_seconds: 1, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 };
    let baseline = PreparedNative {
        contract: NativeContract {
            family: "test".into(),
            manifest: Asset { file: "manifest.smf".into(), sha256: digest(&checkout.join("manifest.smf"))? },
            source_inventory: Asset::default(),
            coverage: BTreeMap::new(),
            provenance: NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
            frozen_inputs,
            compiler_limits: limits.clone(),
            fixture_limits: limits,
            recipe: None,
        },
        manifest: Manifest::default(),
        declarations: Default::default(),
        source: Source::default(),
        pinned_source: checkout.clone(),
        candidate_path: names[0].clone(),
        approved_candidate_paths: BTreeSet::from([names[0].clone()]),
        rust_inputs: names.to_vec(),
        c_inputs: Vec::new(),
        dependencies_sha256: sha256("dependencies"),
        verification_sha256: sha256("verification"),
    };
    let mut single = baseline.clone();
    bind_scope_candidates(&mut single, &[names[0].clone()])?;
    assert_ne!(single.dependencies_sha256, baseline.dependencies_sha256);
    assert_eq!(single.approved_candidate_paths, BTreeSet::from([names[0].clone()]));
    assert!(toml::to_string(&single)?.contains("approved_candidate_paths"));
    assert_eq!(
        candidate_digest(&single, checkout)?,
        sealed(&BTreeMap::from([(&names[0], digest(&checkout.join(&names[0]))?)]))?
    );
    let mut scoped = baseline.clone();
    bind_scope_candidates(&mut scoped, &names[..2])?;
    assert_ne!(scoped.dependencies_sha256, baseline.dependencies_sha256);
    let original = candidate_digest(&scoped, checkout)?;
    fs::write(checkout.join(&names[1]), b"// changed approved helper")?;
    check_candidate(&scoped, checkout)?;
    assert_ne!(candidate_digest(&scoped, checkout)?, original);
    assert!(check_candidate(&baseline, checkout).is_err());
    fs::write(checkout.join(&names[2]), b"// changed frozen helper")?;
    assert!(check_candidate(&scoped, checkout).is_err());
    for name in &names {
        fs::write(checkout.join(name), format!("// original {name}"))?;
    }
    let mut dependency_only = baseline.clone();
    bind_scope_candidates(&mut dependency_only, &[names[1].clone()])?;
    fs::write(checkout.join(&names[0]), b"// unauthorized root change")?;
    assert!(check_candidate(&dependency_only, checkout).is_err());
    let mut uncertain = baseline.clone();
    bind_scope_candidates(&mut uncertain, &["crates/port/src/unrelated.rs".into()])?;
    assert!(uncertain.approved_candidate_paths.is_empty());
    assert!(check_candidate(&uncertain, checkout).is_err());
    for invalid in ["crates/port/src/memory.rs", "crates/port/src/../helper.rs", "tools/port/family.toml"] {
        assert!(bind_scope_candidates(&mut baseline.clone(), &[invalid.into()]).is_err());
    }
    Ok(())
}

#[test]
fn coverage_rejects_empty_duplicate_and_incomplete_registered_domain() {
    let fixture = crate::family_model::Case { name: "step".into(), ..Default::default() };
    let routine = crate::family_model::Routine { symbol: "Step".into(), cases: vec![fixture], ..Default::default() };
    let mut manifest = Manifest { routines: vec![routine.clone()], ..Default::default() };
    assert_eq!(coverage(&manifest).unwrap()["Step"], vec!["step"]);
    manifest.routines.push(routine);
    assert!(coverage(&manifest).is_err());
    manifest.routines.clear();
    assert!(coverage(&manifest).is_err());
    assert!(
        NativeLimits { timeout_seconds: 0, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 }
            .run_limits()
            .is_err()
    );
}

#[test]
fn expired_outer_deadline_refuses_before_accessing_or_creating_any_source() -> Result<()> {
    let absent =
        std::env::temp_dir().join(format!("sm-expired-native-{}-{}", std::process::id(), tool_execution::unique_id()));
    let limits =
        NativeLimits { timeout_seconds: 1, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 };
    let prepared = PreparedNative {
        contract: NativeContract {
            family: "expired".into(),
            manifest: Asset { file: "manifest.smf".into(), sha256: sha256("manifest") },
            source_inventory: Asset { file: "inventory.toml".into(), sha256: sha256("inventory") },
            coverage: BTreeMap::new(),
            provenance: NativeProvenance::NativeScope {
                scope: Asset { file: "scope.toml".into(), sha256: sha256("scope") },
                checkout: absent.clone(),
            },
            frozen_inputs: BTreeMap::new(),
            compiler_limits: limits.clone(),
            fixture_limits: limits,
            recipe: None,
        },
        manifest: Manifest::default(),
        declarations: Default::default(),
        source: Source::default(),
        pinned_source: absent.clone(),
        candidate_path: "candidate.rs".into(),
        approved_candidate_paths: BTreeSet::from(["candidate.rs".into()]),
        rust_inputs: Vec::new(),
        c_inputs: Vec::new(),
        dependencies_sha256: sha256("dependencies"),
        verification_sha256: sha256("verification"),
    };
    let mut runtime = ConfinedEnvironment {
        helper: absent.join("helper"),
        rustc: absent.join("rustc"),
        c_compiler: absent.join("cc"),
        operation_root: absent.join("operations"),
        pipeline_root: absent.join("pipeline"),
        workers: 4,
        gate_timeout_seconds: 60,
        deadline_unix_ms: Some(0),
        max_output_bytes: 4096,
    };
    let context = OperationContext {
        run_id: sha256("expired"),
        attempt: 1,
        policy_sha256: sha256("policy"),
        operation_key: sha256("key"),
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    let error = run(&prepared, &absent.join("source"), &absent.join("output"), &runtime, &context).unwrap_err();
    ensure!(
        error.to_string().contains("whole-stage time budget exhausted"),
        "expired outer stage accessed undeclared inputs: {error}"
    );
    ensure!(!absent.exists(), "expired outer stage created resource directories");
    runtime.deadline_unix_ms = Some(1001);
    ensure!(new_deadline(1000, &runtime)? == 1001, "outer stage did not cap family deadline");
    runtime.deadline_unix_ms = Some(u64::MAX);
    ensure!(new_deadline(1000, &runtime)? == 61000, "outer stage expanded local deadline");
    Ok(())
}

#[test]
fn expired_saved_failure_replays_authentication_without_runtime_or_dispatch() -> Result<()> {
    saved_failure_replay(false)
}

#[test]
fn expired_saved_plm_failure_replays_flat_manifest_without_runtime_or_dispatch() -> Result<()> {
    saved_failure_replay(true)
}

fn saved_failure_replay(retain_plm: bool) -> Result<()> {
    let directory = tool_execution::testing::Directory::new()?;
    let checkout = directory.0.join("checkout");
    fs::create_dir(&checkout)?;
    let output = directory.0.join("output");
    let run_path = output.join("family");
    fs::create_dir_all(&run_path)?;
    fs::write(checkout.join("candidate.rs"), b"pub fn Step() {}")?;
    let manifest = Manifest {
        family: "failure".into(),
        rust_module: "candidate.rs".into(),
        cases: if retain_plm {
            ["terrain", "room", "hardcoded"]
                .into_iter()
                .map(|allocator| Case {
                    name: allocator.into(),
                    allocator: Some(allocator.into()),
                    ..Default::default()
                })
                .collect()
        } else {
            Vec::new()
        },
        ..Default::default()
    };
    crate::fixture_format::write(&checkout.join("manifest.smf"), &manifest)?;
    let recipe = retain_plm.then_some(NativeRecipe::Plm);
    let normalized = recipes::normalize_manifest(&checkout, manifest.clone(), recipe.as_ref())?;
    let limits =
        NativeLimits { timeout_seconds: 1, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 };
    let mut prepared = PreparedNative {
        contract: NativeContract {
            family: "failure".into(),
            manifest: Asset { file: "manifest.smf".into(), sha256: digest(&checkout.join("manifest.smf"))? },
            source_inventory: Asset::default(),
            coverage: if retain_plm { coverage(&normalized)? } else { BTreeMap::new() },
            provenance: NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
            frozen_inputs: BTreeMap::new(),
            compiler_limits: limits.clone(),
            fixture_limits: limits,
            recipe,
        },
        manifest: normalized,
        declarations: Default::default(),
        source: Source::default(),
        pinned_source: directory.0.join("unavailable-source"),
        candidate_path: "candidate.rs".into(),
        approved_candidate_paths: BTreeSet::from(["candidate.rs".into()]),
        rust_inputs: Vec::new(),
        c_inputs: Vec::new(),
        dependencies_sha256: sha256("deps"),
        verification_sha256: sha256("verification"),
    };
    prepared.verification_sha256 = sealed(&(&prepared.contract, crate::fixture_format::identity(&prepared.manifest)?))?;
    let serialized = serde_json::to_value(&prepared)?;
    assert!(serialized.get("manifest").is_none(), "prepared records must not repeat fixture payloads");
    let prepared: PreparedNative = serde_json::from_value(serialized)?;
    assert!(prepared.manifest.family.is_empty(), "decoded cache should start empty");
    assert_eq!(prepared.declaration(&checkout)?.family, "failure");
    let context = OperationContext {
        run_id: sha256("failed-replay"),
        attempt: 1,
        policy_sha256: sha256("policy"),
        operation_key: sha256("key"),
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    let runtime = ConfinedEnvironment {
        helper: directory.0.join("absent-helper"),
        rustc: directory.0.join("absent-rustc"),
        c_compiler: directory.0.join("absent-cc"),
        operation_root: directory.0.join("operations"),
        pipeline_root: directory.0.join("pipeline"),
        workers: 4,
        gate_timeout_seconds: 1,
        deadline_unix_ms: Some(0),
        max_output_bytes: 16 * 1024 * 1024,
    };
    let intent = NativeIntent {
        context: context.clone(),
        prepared_sha256: sealed(&prepared)?,
        candidate_sha256: candidate_digest(&prepared, &checkout)?,
        deadline_unix_ms: 0,
        outer_deadline_unix_ms: Some(0),
        max_output_bytes: runtime.max_output_bytes,
        operation_root: runtime.operation_root.clone(),
        parent_reservation_bytes: 16 * 1024 * 1024,
        operation_metadata_bytes: operation_storage::capacity(
            operation_slots(&open_prepared(&prepared, &checkout)?, &run_path)?.len(),
        )?,
        fixture_jobs: 1,
        helper: runtime.helper.clone(),
        rustc: runtime.rustc.clone(),
        c_compiler: runtime.c_compiler.clone(),
        compiler_runtime_sha256: sha256("none"),
    };
    atomic_toml(&output.join("native-intent.toml"), &intent)?;
    if retain_plm {
        crate::fixture_format::write(&run_path.join("manifest.smf"), &manifest)?;
    }
    let mut verdict = Verdict {
        status: "rejected".into(),
        run: run_path.clone(),
        family: "failure".into(),
        error: Some("original bounded stop".into()),
        ..Default::default()
    };
    family::inventory_with_limits(&run_path, &run_path, &mut verdict.artifacts, retained_tree_limits(&run_path)?)?;
    atomic_toml(&run_path.join("verdict.toml"), &verdict)?;
    let operation_index =
        operation_storage::store(&output, &intent.prepared_sha256, &intent.candidate_sha256, &context, &[])?;
    let mut receipt = NativeReceipt {
        prepared_sha256: intent.prepared_sha256,
        candidate_sha256: intent.candidate_sha256,
        verdict: Asset { file: run_path.join("verdict.toml"), sha256: digest(&run_path.join("verdict.toml"))? },
        passed: false,
        diagnostic: "original bounded stop".into(),
        evidence_bytes: 0,
        context: context.clone(),
        operation_root: runtime.operation_root.clone(),
        operation_index,
        max_output_bytes: runtime.max_output_bytes,
        reserved_bytes: intent.parent_reservation_bytes,
    };
    receipt.evidence_bytes = evidence_bytes(&run_path, &receipt)?;
    atomic_toml(&output.join("native-receipt.toml"), &receipt)?;
    let receipt_path = output.join("native-receipt.toml");
    let metadata = receipt_path.metadata()?.modified()?;
    let receipt_sha256 = digest(&receipt_path)?;
    let replay = run(&prepared, &checkout, &output, &runtime, &context)?;
    assert!(!replay.passed);
    assert_eq!(replay.diagnostic, "original bounded stop");
    assert_eq!(receipt_path.metadata()?.modified()?, metadata);
    assert_eq!(digest(&receipt_path)?, receipt_sha256);
    assert!(!runtime.operation_root.exists() && !runtime.pipeline_root.exists());
    fs::write(checkout.join("candidate.rs"), b"pub fn Changed() {}")?;
    assert!(run(&prepared, &checkout, &output, &runtime, &context).is_err());
    Ok(())
}

fn current_checkout() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}
