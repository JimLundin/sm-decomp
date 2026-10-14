//! Public status/recovery boundary controls. These fixtures provide no gameplay
//! acceptance: status must authenticate an initial source boundary without
//! starting a supervisor, compiler, model request or capture.
use super::*;
use crate::{
    translation_cargo as cargo,
    translation_migration::{MigratedClaim, OriginalClaim},
};
use translation_execution::model::{ClaimLineage, Disposition};

struct Fixture {
    directory: bounded::tests::Directory,
    config: Config,
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let result = std::process::Command::new("/usr/bin/git")
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .current_dir(root)
        .output()?;
    ensure!(result.status.success(), "fixture Git: {}", String::from_utf8_lossy(&result.stderr));
    Ok(String::from_utf8(result.stdout)?.trim().into())
}

impl Fixture {
    fn new() -> Result<Self> {
        let directory = bounded::tests::Directory::new()?;
        let root = &directory.0;
        let checkout = root.join("checkout");
        fs::create_dir_all(&checkout)?;
        fs::write(checkout.join("source.rs"), "reviewed source\n")?;
        git(&checkout, &["init", "-q"])?;
        git(&checkout, &["config", "user.name", "Continuation test"])?;
        git(&checkout, &["config", "user.email", "continuation@invalid"])?;
        git(&checkout, &["add", "source.rs"])?;
        git(&checkout, &["commit", "-qm", "Reviewed boundary"])?;
        let head = git(&checkout, &["rev-parse", "HEAD"])?;
        let evidence = write(&root.join("evidence.json"), &"status fixture only")?;
        let ledger = write(&checkout.join(".scratch/effort/ledger.toml"), &"unchanged queue")?;
        let closure = cargo::CargoClosure { root: root.join("vendor"), files: BTreeMap::new() };
        let controls = cargo::CargoContract {
            schema: "translation-cargo-contract-v1".into(),
            kind: cargo::CargoKind::SharedGameHost,
            source_files: BTreeMap::new(),
            approved_files: vec![],
            vendor: closure,
            dependency_cache: None,
            limits: cargo::CargoLimits {
                timeout_seconds: 60,
                writable_bytes: 4096,
                stdout_bytes: 512,
                stderr_bytes: 512,
            },
        };
        let cargo_reference = write(&root.join("cargo.json"), &controls)?;
        let mut scopes = Vec::new();
        let mut migrations = Vec::new();
        for (index, kind) in [
            original::OriginalKind::SharedGameHost,
            original::OriginalKind::CompactController,
            original::OriginalKind::PauseEquipment,
        ]
        .into_iter()
        .enumerate()
        {
            let id = format!("scope-{index}");
            let claim =
                ClaimLineage { id: id.clone(), revision: "original".into(), attempt: 1, coordinator: "owner".into() };
            let original = OriginalClaim {
                schema: "translation-original-claim-v1".into(),
                checkout: checkout.clone(),
                effort: "effort".into(),
                job: assignment::Job {
                    id: id.clone(),
                    revision: "original".into(),
                    kind: assignment::Kind::Implement,
                    subject: id.clone(),
                    writable_paths: vec!["source.rs".into()],
                    blockers: vec![],
                    inputs: BTreeMap::new(),
                    instructions: "Status only; never accepted as an original gameplay contract".into(),
                    ticket: None,
                },
                claim: assignment::Claim {
                    id: id.clone(),
                    revision: "original".into(),
                    attempt: 1,
                    coordinator: "owner".into(),
                },
                state: assignment::State::Submitted,
                agent: Some("original-owner".into()),
                result: Some(assignment::WorkResult::Submitted { commit: head.clone(), evidence: vec![] }),
                observations: vec![],
                source_checkout: checkout.clone(),
                source_commit: head.clone(),
                source_files: BTreeMap::from([("source.rs".into(), sha256("reviewed source\n"))]),
            };
            let original_reference = write(&root.join(format!("original-{index}.json")), &original)?;
            let contract = original::OriginalContract {
                schema: "translation-original-contract-v1".into(),
                kind,
                original_claim: original_reference.clone(),
                legacy: None,
                verification_source: original::VerificationSource {
                    root: checkout.clone(),
                    files: original.source_files.clone(),
                },
                cargo: cargo_reference.clone(),
                cargo_executable: evidence.clone(),
                families: BTreeMap::new(),
                compact: None,
                catalog_sha256: None,
            };
            let prepared = original::PreparedOriginal {
                schema: "translation-original-prepared-v1".into(),
                contract: contract.clone(),
                original,
                dependencies_sha256: sha256("dependencies"),
                verification_sha256: sha256("verification"),
                pinned_c_sha256: sha256("pin"),
                native: BTreeMap::new(),
                compact: None,
                references: vec![],
                cargo: cargo::PreparedCargo {
                    contract: controls.clone(),
                    dependencies_sha256: sha256("dep"),
                    verification_sha256: sha256("verify"),
                    test_sources: BTreeMap::new(),
                },
            };
            let migration = Migration {
                schema: "translation-execution-migration-v1".into(),
                state: "active".into(),
                checkout: checkout.clone(),
                effort: "effort".into(),
                coordinator: "owner".into(),
                session_id: "logical-controller".into(),
                generation: 1,
                authority: evidence.clone(),
                ledger: ledger.clone(),
                historical: vec![evidence.clone()],
                stopped_owners: vec![],
                proof: evidence.clone(),
                limitations: vec!["Status fixture, no producer admission".into()],
                claims: vec![MigratedClaim {
                    claim,
                    scope_id: id.clone(),
                    policy_sha256: sha256("prior-policy"),
                    source_commit: head.clone(),
                    writable_files: vec!["source.rs".into()],
                    original_claim: original_reference.clone(),
                    installation: None,
                    execution: Some(MigrationExecution::ImportedSubmission),
                }],
            };
            let migration = write(&root.join(format!("migration-{index}.json")), &migration)?;
            migrations.push(migration.clone());
            scopes.push(Scope {
                id,
                migration,
                original_claim: original_reference,
                contract: write(&root.join(format!("contract-{index}.json")), &contract)?,
                prepared: write(&root.join(format!("prepared-{index}.json")), &prepared)?,
            });
        }
        let registry = production::Registry {
            schema: "translation-production-registry-v1".into(),
            checkout: checkout.clone(),
            request: evidence.clone(),
            authority: evidence.clone(),
            migrations,
            launch_plan: evidence.clone(),
            blocked: vec![],
        };
        let registry = write(&checkout.join(production::REGISTRY), &registry)?;
        let tool = serde_json::json!({"path": evidence.path, "sha256": evidence.sha256});
        let template = serde_json::json!({
            "schema":"translation-supervisor-installation-v1", "id":"status-fixture", "controller":tool,
            "helper":tool,"rustc":tool,"c_compiler":tool,"source_repository":checkout,"integration_repository":checkout,
            "integration_baseline":head,"pinned_source":root.join("pin"),"state_root":root.join("state"),
            "pipeline_root":root.join("pipeline"),"workers":4,"max_gate_bytes":4096,"max_checkout_bytes":4096,"max_context_bytes":4096,
            "rewrite":{"executable":tool,"recorded":false,"limits":{"timeout":{"secs":1,"nanos":0},"writable_bytes":4096,
                "stdout_bytes":512,"stderr_bytes":512,"stdin_bytes":512}},
            "policy":{"schema":"translation-execution-policy-v1","version":1,"scope_id":"fixture",
                "claim":{"id":"fixture","revision":"original","attempt":1,"coordinator":"owner"},
                "baseline_commit":head,"pinned_c_sha256":sha256("pin"),"dependencies_sha256":sha256("dep"),
                "verification_sha256":sha256("verify"),"adapter_sha256":sha256("adapter"),"approved_files":{},
                "verification_stages":["focused"],"budgets":{"workers":4,"max_attempts":1,"max_evidence_bytes":1048576,"gate_timeout_seconds":60},
                "rewrite_limits":{"max_context_bytes":4096,"max_response_bytes":4096,"max_file_bytes":4096,"max_total_bytes":4096}},
            "contract":{"kind":"original","contract":read::<original::OriginalContract>(&scopes[0].contract)?},
            "admission":{"kind":"isolated-proof","limitations":["Read-only status fixture"]}
        });
        let template = write(&root.join("template.json"), &template)?;
        let preserved =
            paths(&checkout)?.iter().map(|path| retention::seal_file(path, MAX_RECORD)).collect::<Result<_>>()?;
        let config = Config {
            schema: "translation-original-production-config-v1".into(),
            checkout: checkout.clone(),
            registry: registry.clone(),
            output_root: root.join("production"),
            template,
            reviewed_head: head,
            preserved,
            stopped_installation: evidence,
            stopped_view: View {
                run_id: sha256("stop"),
                attempt: 1,
                policy_sha256: sha256("policy"),
                state: State::Stopped {
                    disposition: Disposition::Failed,
                    reason: "unimplemented prerequisite".into(),
                    finalization: None,
                },
            },
            scopes,
            reason: "Read-only status/recovery boundary test; no acceptance".into(),
        };
        let reference = write(&root.join("config.json"), &config)?;
        write(
            &checkout.join(MARKER),
            &Activation { schema: "translation-original-production-v1".into(), registry, config: reference },
        )?;
        Ok(Self { directory, config })
    }
}

#[test]
fn status_authenticates_initial_source_boundary_without_creating_an_attempt() -> Result<()> {
    let fixture = Fixture::new()?;
    let before = crate::translation_execution::tests::records(&fixture.directory.0)?;
    let report = run(&fixture.config.checkout, "status")?.context("status missing")?;
    assert_eq!(report.state, "ready");
    assert_eq!(report.current.as_deref(), Some("scope-0"));
    assert!(report.view.is_none() && report.completed.is_empty());
    assert_eq!(before, crate::translation_execution::tests::records(&fixture.directory.0)?);
    fs::write(fixture.config.checkout.join(".scratch/unreviewed.md"), "new unreviewed input")?;
    assert!(
        run(&fixture.config.checkout, "status").is_err(),
        "status accepted new mutable metadata outside the reviewed boundary"
    );
    Ok(())
}

#[test]
fn status_refuses_changed_preserved_ledger_source_and_reviewed_order() -> Result<()> {
    let fixture = Fixture::new()?;
    let ledger = fixture.config.checkout.join(".scratch/effort/ledger.toml");
    let original = fs::read(&ledger)?;
    fs::set_permissions(&ledger, fs::Permissions::from_mode(0o600))?;
    fs::write(&ledger, "changed original queue")?;
    assert!(run(&fixture.config.checkout, "status").is_err());
    fs::write(&ledger, original)?;
    fs::set_permissions(&ledger, fs::Permissions::from_mode(0o400))?;
    fs::write(fixture.config.checkout.join("source.rs"), "foreign source\n")?;
    assert!(run(&fixture.config.checkout, "status").is_err());
    git(&fixture.config.checkout, &["add", "source.rs"])?;
    git(&fixture.config.checkout, &["commit", "-qm", "Unreviewed source advancement"])?;
    assert!(run(&fixture.config.checkout, "status").is_err());
    assert!(!fixture.config.output_root.exists(), "read-only refusals allocated a production attempt");
    Ok(())
}

#[test]
fn changed_frozen_contract_is_refused_before_any_successor_allocation() -> Result<()> {
    let fixture = Fixture::new()?;
    let contract = &fixture.config.scopes[1].contract.path;
    fs::set_permissions(contract, fs::Permissions::from_mode(0o600))?;
    let mut contents = fs::read(contract)?;
    contents.push(b' ');
    fs::write(contract, contents)?;
    fs::set_permissions(contract, fs::Permissions::from_mode(0o400))?;
    assert!(run(&fixture.config.checkout, "run").is_err());
    assert!(!fixture.config.output_root.exists(), "changed policy allocated a successor or controller lease");
    Ok(())
}
