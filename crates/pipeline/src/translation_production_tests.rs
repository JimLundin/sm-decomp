use super::*;
use ::translation_execution::model::sha256;

struct Fixture {
    directory: tool_execution::testing::Directory,
    request: Request,
    reference: Reference,
    registry: Registry,
}

impl Fixture {
    fn new() -> Result<Self> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        let authority = write(&root.join(AUTHORITY), &serde_json::json!({"original": "authority"}))?;
        let proof = write(&root.join("package/proof.json"), &serde_json::json!({"proof": "retained"}))?;
        let original = write(&root.join("package/original.json"), &serde_json::json!({"claim": "unchanged"}))?;
        let installation =
            write(&root.join("package/installation.json"), &serde_json::json!({"installation": "sealed"}))?;
        let mut deployments = Vec::new();
        for effort in ["canonical", "legacy"] {
            let ledger =
                write(&root.join(format!(".scratch/{effort}/ledger.toml")), &serde_json::json!({"original": effort}))?;
            let mut migration = Migration {
                schema: "translation-execution-migration-v1".into(),
                state: "prepared".into(),
                checkout: root.clone(),
                effort: effort.into(),
                coordinator: "integrator".into(),
                session_id: "01a136a0-49bb-7690-a360-48def3c2c09b".into(),
                generation: 3,
                authority: authority.clone(),
                ledger,
                claims: vec![migration::MigratedClaim {
                    claim: ClaimLineage {
                        id: format!("{effort}-job"),
                        revision: "original-revision".into(),
                        attempt: 1,
                        coordinator: "integrator".into(),
                    },
                    scope_id: format!("{effort}-scope"),
                    policy_sha256: sha256(effort),
                    source_commit: "a".repeat(40),
                    writable_files: vec![format!("crates/port/src/{effort}.rs").into()],
                    original_claim: original.clone(),
                    installation: Some(installation.path.clone()),
                    execution: None,
                }],
                historical: vec![proof.clone()],
                stopped_owners: vec![proof.clone()],
                proof: proof.clone(),
                limitations: vec!["Test ownership transaction, no native producer or acceptance.".into()],
            };
            let prepared = write(&root.join(format!("package/{effort}-prepared.json")), &migration)?;
            migration.state = "active".into();
            let active = write(&root.join(format!("package/{effort}-active.json")), &migration)?;
            deployments.push(Deployment { prepared, active });
        }
        let plan = Plan {
            schema: "translation-launch-plan-v1".into(),
            id: "production".into(),
            state_root: root.join("launch-state"),
            scopes: vec![crate::translation_launch::Scope {
                id: "canonical".into(),
                after: vec![],
                installation: installation.clone(),
            }],
        };
        let launch_plan = write(&root.join("package/plan.json"), &plan)?;
        let request = Request {
            schema: "translation-production-request-v1".into(),
            checkout: root.clone(),
            authority,
            migrations: deployments,
            installations: vec![installation],
            launch_plan,
            blocked: vec![BlockedClaim {
                effort: "legacy".into(),
                claim: ClaimLineage {
                    id: "legacy-job".into(),
                    revision: "original-revision".into(),
                    attempt: 1,
                    coordinator: "integrator".into(),
                },
                original_claim: original,
                reason: "Exact original acceptance recipe remains unsupported.".into(),
            }],
        };
        let reference = write(&root.join("package/request.json"), &request)?;
        let registry = registry_for(&request, &reference);
        Ok(Self { directory, request, reference, registry })
    }

    fn root(&self) -> &Path {
        &self.directory.0
    }

    fn checked_intent(&self) -> Result<()> {
        let output = transaction(self.root(), &self.reference);
        let snapshot = output.join("original-authority.toml");
        write_once(&snapshot, &bounded::read(&self.request.authority.path)?)?;
        write(
            &output.join("intent.json"),
            &Intent {
                schema: "translation-production-intent-v1".into(),
                registry: self.registry.clone(),
                original_snapshots: vec![retention::seal_file(&snapshot, MAX_RECORD)?],
                preparation_reports: vec![],
            },
        )?;
        Ok(())
    }
}

#[test]
fn activation_is_one_registry_commit_and_recovery_preserves_original_bytes() -> Result<()> {
    let fixture = Fixture::new()?;
    let before = preserved_inputs(&fixture.request, &migrations(&fixture.request)?)
        .iter()
        .map(|reference| Ok((reference.path.clone(), bounded::read(&reference.path)?)))
        .collect::<Result<Vec<_>>>()?;
    fixture.checked_intent()?;
    assert!(read_registry(fixture.root())?.is_none());
    // Model the interruption after the atomic ownership switch and before the
    // convenience receipt. Both efforts are already one immutable registry.
    write(&fixture.root().join(LEGACY_FENCE), &fence_for(&fixture.registry)?)?;
    write(&fixture.root().join(REGISTRY), &fixture.registry)?;
    let metadata = fixture.root().join(REGISTRY).metadata()?.modified()?;
    assert!(!transaction(fixture.root(), &fixture.reference).join("activation.json").exists());
    let loaded = read_registry(fixture.root())?.unwrap();
    assert_eq!(loaded.migrations.len(), 2);
    assert_eq!(loaded.blocked[0].reason, "Exact original acceptance recipe remains unsupported.");
    finish_activation(fixture.root(), &fixture.reference, &fixture.registry)?;
    let receipt = transaction(fixture.root(), &fixture.reference).join("activation.json");
    let receipt_metadata = receipt.metadata()?.modified()?;
    finish_activation(fixture.root(), &fixture.reference, &fixture.registry)?;
    assert_eq!(fixture.root().join(REGISTRY).metadata()?.modified()?, metadata);
    assert_eq!(receipt.metadata()?.modified()?, receipt_metadata);
    for (path, bytes) in before {
        assert_eq!(bounded::read(path)?, bytes);
    }
    Ok(())
}

#[test]
fn partial_registry_requires_the_exact_intent_and_tampering_fails_closed() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.checked_intent()?;
    let registry = fixture.root().join(REGISTRY);
    let bytes = serde_json::to_vec(&fixture.registry)?;
    let pending = registry.with_extension("pending");
    fs::write(&pending, &bytes[..bytes.len() / 2])?;
    finish_activation(fixture.root(), &fixture.reference, &fixture.registry)?;
    assert_eq!(read_registry(fixture.root())?, Some(fixture.registry.clone()));
    assert!(!pending.exists());
    let mut changed = fixture.registry.clone();
    changed.blocked[0].reason = "new unreviewed reason".into();
    assert!(finish_activation(fixture.root(), &fixture.reference, &changed).is_err());
    let active = &fixture.request.migrations[0].active.path;
    let original = bounded::read(active)?;
    fs::write(active, b"{}")?;
    assert!(read_registry(fixture.root()).is_err());
    fs::write(active, original)?;
    assert_eq!(read_registry(fixture.root())?, Some(fixture.registry));
    Ok(())
}

#[test]
fn foreign_partial_and_missing_original_claim_cannot_authorize_switch() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.checked_intent()?;
    let pending = fixture.root().join(REGISTRY).with_extension("pending");
    fs::write(&pending, b"foreign prefix")?;
    assert!(finish_activation(fixture.root(), &fixture.reference, &fixture.registry).is_err());
    assert!(read_registry(fixture.root())?.is_none());
    // Even without a final registry, an interrupted activation fences binaries
    // which only implement the historical single-effort Migration reader.
    assert!(crate::translation_migration_guard::read(&fixture.root().join(LEGACY_FENCE)).is_err());
    assert_eq!(bounded::read(pending)?, b"foreign prefix");
    let mut changed = fixture.request.clone();
    changed.blocked[0].claim.attempt += 1;
    assert!(migrations(&changed).is_err());
    let mut changed = fixture.request.clone();
    changed.migrations[0].active = changed.migrations[1].active.clone();
    assert!(migrations(&changed).is_err());
    Ok(())
}

#[test]
fn interruption_after_legacy_fence_blocks_old_readers_until_exact_recovery() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.checked_intent()?;
    write(&fixture.root().join(LEGACY_FENCE), &fence_for(&fixture.registry)?)?;
    assert!(read_registry(fixture.root())?.is_none());
    assert!(crate::translation_migration_guard::read(&fixture.root().join(LEGACY_FENCE)).is_err());
    finish_activation(fixture.root(), &fixture.reference, &fixture.registry)?;
    assert_eq!(read_registry(fixture.root())?, Some(fixture.registry));
    Ok(())
}
