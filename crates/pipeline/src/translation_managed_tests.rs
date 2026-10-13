//! Disposable managed diagnostic execution. The original exercised Scope is
//! immutable and separate from the host, installed tools and pinned C source.
use super::*;
use crate::{assignment, progress, translation, translation_migration as migration};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    process::Command,
    time::{Duration, Instant},
};

const EFFORT: &str = "managed-native-diagnostic";
const COORDINATOR: &str = "managed-diagnostic-integrator";
const ORIGINAL_AGENT: &str = "01234567-89ab-cdef-0123-456789abcdef";
const FAMILY: &str = "managed-diagnostic";
const MANIFEST: &str = "tools/port/managed-diagnostic.toml";
const CANDIDATE: &str = "crates/port/src/example.rs";
const FENCE: &str = ".scratch/decomp-parallel/deterministic-execution.json";
const LIMITATION: &str = "Disposable diagnostic Observed/Helper scope only. Synthetic stopped original worker observation; no production liveness, shipped gameplay, original-ROM parity or Live replay acceptance.";

const CANDIDATE_SOURCE: &str = concat!(
    "use crate::memory::Wram;\n",
    "// diagnostic-before\n",
    "pub fn dispatch(_wram: &mut Wram<'_>, symbol: &str, _a: usize, _b: usize) -> Result<usize, &'static str> {\n",
    "    match symbol { \"Observed\" | \"Helper\" => Ok(0), _ => Err(\"unknown diagnostic routine\") }\n",
    "}\n",
);

const MEMORY_SOURCE: &str = concat!(
    "pub const WRAM_SIZE: usize = 0x20000;\n",
    "pub struct Wram<'a> { pub bytes: &'a mut [u8] }\n",
    "impl<'a> Wram<'a> {\n",
    "    pub fn new(bytes: &'a mut [u8]) -> Result<Self, &'static str> {\n",
    "        if bytes.len() != WRAM_SIZE { return Err(\"full WRAM required\"); }\n",
    "        Ok(Self { bytes })\n",
    "    }\n",
    "}\n",
);

// This installed no-argv protocol worker deliberately transforms only the
// diagnostic marker. Its process has no source/evidence directory capabilities.
const PROTOCOL_SOURCE: &str = r#"
use std::io::{Read, Write};
fn main() {
    assert_eq!(std::env::args().count(), 1, "no host argv admitted");
    let mut bytes = Vec::new();
    std::io::stdin().take(256 * 1024 + 1).read_to_end(&mut bytes).unwrap();
    assert!(bytes.len() <= 256 * 1024, "capsule extent exceeded");
    let capsule: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let files = capsule["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    let file = &files[0];
    assert_eq!(file["path"], "crates/port/src/example.rs");
    let content = file["content"].as_str().unwrap();
    assert_eq!(content.matches("diagnostic-before").count(), 1);
    let response = serde_json::json!({
        "capsule_id": capsule["id"],
        "files": [{"path": file["path"], "base_sha256": file["sha256"],
            "content": content.replace("diagnostic-before", "diagnostic-after")}]
    });
    std::io::stdout().write_all(&serde_json::to_vec(&response).unwrap()).unwrap();
}
"#;

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("/usr/bin/git")
        .args(["-c", "core.hooksPath=/dev/null", "-c", "core.fsmonitor=false"])
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .current_dir(root)
        .output()?;
    ensure!(output.status.success(), "diagnostic Git failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

fn put(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    fs::create_dir_all(path.parent().context("diagnostic fixture parent missing")?)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn inject_record(path: &Path, bytes: &[u8]) -> Result<()> {
    // Failure injection explicitly replaces a controller-owned immutable
    // record, then restores its sealed mode before exercising authentication.
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o400))?;
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let metadata = entry.path().symlink_metadata()?;
        ensure!(!metadata.file_type().is_symlink(), "diagnostic fixture contains a symlink");
        let target = destination.join(entry.file_name());
        if metadata.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            ensure!(metadata.is_file(), "diagnostic fixture contains a special file");
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn reference(path: &Path) -> Result<Reference> {
    retention::seal_file(path, 4 * 1024 * 1024)
}

fn artifact(path: &Path) -> Result<assignment::Artifact> {
    Ok(assignment::Artifact { path: path.into(), sha256: digest(path)? })
}

fn installed(path: PathBuf) -> Result<InstalledFile> {
    Ok(InstalledFile { sha256: digest(&path)?, path })
}

fn compile_protocol(root: &Path, rustc: &Path) -> Result<PathBuf> {
    let project = root.join("protocol-source");
    let source = project.join("src/main.rs");
    put(&source, PROTOCOL_SOURCE)?;
    let manifest = project.join("Cargo.toml");
    put(
        &manifest,
        "[package]\nname = \"managed-diagnostic-protocol\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\nserde_json = \"=1.0.151\"\n",
    )?;
    let cargo = rustc.parent().context("diagnostic toolchain directory missing")?.join("cargo");
    let lock = Command::new(&cargo)
        .args(["generate-lockfile", "--offline", "--manifest-path"])
        .arg(&manifest)
        .env("RUSTC", rustc)
        .current_dir(&project)
        .output()?;
    put(&root.join("protocol-lock.stdout.log"), &lock.stdout)?;
    put(&root.join("protocol-lock.stderr.log"), &lock.stderr)?;
    ensure!(lock.status.success(), "diagnostic protocol offline lock generation failed");
    // Let the installed Cargo resolve its exact metadata/linker layout. The
    // executable comes from its artifact receipt, never a guessed deps path.
    let output = Command::new(&cargo)
        .args(["build", "--locked", "--offline", "--message-format=json", "--manifest-path"])
        .arg(&manifest)
        .arg("--target-dir")
        .arg(root.join("disposable-protocol-target"))
        .env("RUSTC", rustc)
        .env("CARGO_BUILD_JOBS", "1")
        .current_dir(&project)
        .output()?;
    put(&root.join("protocol-build.stdout.log"), &output.stdout)?;
    put(&root.join("protocol-build.stderr.log"), &output.stderr)?;
    ensure!(output.status.success(), "diagnostic protocol compilation failed");
    let mut artifacts = Vec::new();
    for line in std::str::from_utf8(&output.stdout)?.lines() {
        let artifact: serde_json::Value = serde_json::from_str(line)?;
        if artifact["reason"] == "compiler-artifact" && artifact["target"]["name"] == "managed-diagnostic-protocol" {
            if let Some(executable) = artifact["executable"].as_str() {
                artifacts.push(PathBuf::from(executable));
            }
        }
    }
    ensure!(artifacts.len() == 1, "diagnostic protocol executable receipt missing or ambiguous");
    let executable = root.join("installed/protocol-worker");
    fs::copy(&artifacts[0], &executable)?;
    Ok(executable)
}

struct ManagedProject {
    root: PathBuf,
    owner: PathBuf,
    host: PathBuf,
    installation: Installation,
    original: migration::OriginalClaim,
}

impl ManagedProject {
    fn new(root: &Path) -> Result<Self> {
        let host = root.join("host");
        let owner = root.join("original-owner");
        fs::create_dir_all(&owner)?;
        let mut scope = translation::managed_native_scope_fixture(&owner, Path::new("/home/jim/sm-decomp/rom/sm.sfc"))?;
        put(&owner.join(CANDIDATE), CANDIDATE_SOURCE)?;
        put(&owner.join("crates/port/src/memory.rs"), MEMORY_SOURCE)?;
        for (name, text) in tests::native_harness_sources() {
            put(&owner.join("crates/pipeline/src").join(name), text)?;
        }
        let manifest = crate::family_model::Manifest {
            family: FAMILY.into(),
            rust_module: CANDIDATE.into(),
            bank: "sm_86.c".into(),
            routines: ["Observed", "Helper"]
                .into_iter()
                .map(|symbol| crate::family_model::Routine {
                    symbol: symbol.into(),
                    args: 0,
                    returns: "void".into(),
                    cases: vec![crate::family_model::Case {
                        name: format!("{symbol}-full-wram"),
                        fill: Some(if symbol == "Observed" { 0xa5 } else { 0x5a }),
                        expected_return: Some(0),
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        put(&owner.join(MANIFEST), toml::to_string(&manifest)?)?;
        put(&owner.join("tools/port/batch.toml"), format!("[families]\n{FAMILY} = \"family\"\n[profiles]\n"))?;
        let reviewed_inventory: toml::Value = toml::from_str(&fs::read_to_string(owner.join("inventory.toml"))?)?;
        crate::source_inventory::run(&owner, &owner.join("deps/snesrev-sm"), &owner.join("inventory.toml"))?;
        let mut inventory: toml::Value = toml::from_str(&fs::read_to_string(owner.join("inventory.toml"))?)?;
        // The lexical producer recognizes shipped // address annotations; this
        // diagnostic's original reviewed definition uses a block comment. Keep
        // that one reviewed address only on the exact same generated definition.
        // Normal Scope admission still authenticates both its body and hit table.
        for reviewed in reviewed_inventory["routines"].as_array().context("reviewed diagnostic routines missing")? {
            let Some(address) = reviewed.get("original_address") else { continue };
            let generated = inventory["routines"]
                .as_array_mut()
                .context("generated diagnostic routines missing")?
                .iter_mut()
                .find(|routine| routine["symbol"] == reviewed["symbol"])
                .context("reviewed diagnostic definition disappeared")?;
            for field in ["source", "line", "definition_sha256", "defined_dependencies"] {
                ensure!(generated[field] == reviewed[field], "diagnostic address definition changed: {field}");
            }
            generated.as_table_mut().unwrap().insert("original_address".into(), address.clone());
        }
        put(&owner.join("inventory.toml"), toml::to_string(&inventory)?)?;
        fs::copy(owner.join("inventory.toml"), owner.join("tools/port/source-inventory.toml"))?;
        scope.source_inventory_sha256 = digest(&owner.join("inventory.toml"))?;
        for name in [CANDIDATE, "crates/port/src/memory.rs", MANIFEST, "tools/dependencies.toml"] {
            scope.inputs.insert(name.into(), digest(&owner.join(name))?);
        }
        let argv =
            ["sm-pipeline", "family-verify", "--manifest", MANIFEST].into_iter().map(str::to_owned).collect::<Vec<_>>();
        scope.verification = [false, true]
            .into_iter()
            .map(|integration| translation::Verification { argv: argv.clone(), timeout_seconds: 60, integration })
            .collect();
        let scope_path = owner.join("scope.toml");
        put(&scope_path, toml::to_string(&scope)?)?;
        let mut job = translation::validate_scope(&owner, &scope_path)?;
        copy_tree(&owner, &host)?;
        git(&host, &["init", "-q"])?;
        git(&host, &["config", "user.name", "Managed native diagnostic"])?;
        git(&host, &["config", "user.email", "diagnostic@invalid"])?;
        // Only this exact private fence is ignored. Every other .scratch file
        // is tracked before execution or admitted by finite controller policy.
        put(&host.join(".git/info/exclude"), format!("/{FENCE}\n"))?;
        git(&host, &["add", "."])?;
        git(&host, &["commit", "-q", "-m", "Frozen diagnostic native source and original exercise"])?;
        job.inputs.insert("base-head".into(), git(&host, &["rev-parse", "HEAD"])?);
        let original_ticket = job.ticket.as_ref().context("diagnostic origin task missing")?;
        let ticket = fs::read_to_string(host.join(original_ticket))?.replace("Status: claimed", "Status: open");
        put(&host.join(original_ticket), ticket)?;
        let request = assignment::Request {
            operation: "next".into(),
            checkout: host.clone(),
            effort: EFFORT.into(),
            coordinator: COORDINATOR.into(),
            capacity: 3,
            available_slots: 1,
            jobs: vec![job.clone()],
            ..Default::default()
        };
        let reserved = assignment::operate(request.clone())?;
        let claim = reserved.assignment.context("diagnostic original claim was not reserved")?.claim;
        let running = assignment::operate(assignment::Request {
            operation: "record".into(),
            jobs: vec![],
            update: Some(assignment::Update::Started { claim: claim.clone(), agent: ORIGINAL_AGENT.into() }),
            ..request
        })?;
        let entry =
            running.jobs.iter().find(|entry| entry.job.id == claim.id).context("running diagnostic claim missing")?;
        ensure!(entry.state == assignment::State::Running && entry.agent.as_deref() == Some(ORIGINAL_AGENT));
        let session_id =
            std::env::var("CODEX_THREAD_ID").context("managed proof needs its actual controller thread")?;
        ensure!(assignment::authority::real_session_id(&session_id), "current diagnostic thread identity invalid");
        let evidence = root.join("observations/diagnostic.txt");
        put(&evidence, LIMITATION)?;
        let owner_session = assignment::authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: ORIGINAL_AGENT.into(),
            status: "interrupted".into(),
            observed_at: "2026-10-13 23:00:00 UTC".into(),
            evidence: vec![artifact(&evidence)?],
        };
        let owner_session_path = root.join("observations/original-session.json");
        write_once(&owner_session_path, &owner_session)?;
        let stop = migration::OriginalOwnerStop {
            schema: "translation-original-owner-stop-v1".into(),
            checkout: host.clone(),
            effort: EFFORT.into(),
            claim: claim.clone(),
            agent: Some(ORIGINAL_AGENT.into()),
            observed_at: owner_session.observed_at.clone(),
            status: owner_session.status,
            session: Some(reference(&owner_session_path)?),
            evidence: vec![reference(&evidence)?],
            processes: vec![],
        };
        let stopped_path = root.join("observations/original-stop.json");
        write_once(&stopped_path, &stop)?;
        let bridge = root.join("observations/bridge.toml");
        put(&bridge, "schema = \"decomp-legacy-owner-bridge-v1\"\n")?;
        let controller_session = root.join("observations/controller-session.toml");
        put(
            &controller_session,
            toml::to_string(&assignment::authority::SessionObservation {
                schema: "decomp-session-observation-v1".into(),
                session_id: session_id.clone(),
                status: "active".into(),
                observed_at: owner_session.observed_at,
                evidence: vec![artifact(&evidence)?],
            })?,
        )?;
        let authority = serde_json::json!({
            "schema": "decomp-session-authority-v1", "state": "active",
            "prepared_head": job.inputs["base-head"],
            "preparation": {"canonical_effort": "decomp-parallel", "capacity": 3,
                "admitted_efforts": [EFFORT], "drill_efforts": [], "bridge": artifact(&bridge)?,
                "integrator": {"token": COORDINATOR, "kind": "integrator", "session_id": session_id,
                    "observation": artifact(&controller_session)?}},
            "roles": {(COORDINATOR): {"token": COORDINATOR, "kind": "integrator", "session_id": session_id,
                "generation": 1, "registration": artifact(&controller_session)?, "transfers": []}},
            "drill": artifact(&evidence)?
        });
        let authority_path = host.join(".scratch/decomp-parallel/authority.toml");
        put(&authority_path, toml::to_string(&toml::Value::try_from(authority)?)?)?;
        progress::cli(&["refresh".into(), "--root".into(), host.display().to_string()])?;
        git(&host, &["add", "."])?;
        git(&host, &["commit", "-q", "-m", "Checkpoint exact original running claim and diagnostic authority"])?;
        let baseline = git(&host, &["rev-parse", "HEAD"])?;
        ensure!(
            git(&host, &["status", "--porcelain", "--untracked-files=all"])?.is_empty(),
            "diagnostic host baseline dirty"
        );
        let approved_files = BTreeMap::from([(CANDIDATE.into(), digest(&owner.join(CANDIDATE))?)]);
        let original = migration::OriginalClaim {
            schema: "translation-original-claim-v1".into(),
            checkout: host.clone(),
            effort: EFFORT.into(),
            job,
            claim: claim.clone(),
            state: assignment::State::Running,
            agent: entry.agent.clone(),
            result: None,
            observations: entry.observations.clone(),
            source_checkout: owner.clone(),
            source_commit: baseline.clone(),
            source_files: approved_files,
        };
        let original_path = root.join("original-claim.json");
        write_once(&original_path, &original)?;
        let representative = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .context("crate workspace missing")?
            .parent()
            .context("crate workspace missing")?
            .join(".scratch/translation-scope-execution/supervisor-native/actual-c-02");
        let measurement: serde_json::Value = read(&representative.join("proof.json"))?;
        let proof = migration::RetainedProof {
            schema: "translation-supervisor-proof-v1".into(),
            installation: reference(&representative.join("installation.json"))?,
            completion: reference(Path::new(
                measurement["completion"].as_str().context("retained representative completion missing")?,
            ))?,
        };
        let proof_path = root.join("representative-proof.json");
        write_once(&proof_path, &proof)?;
        let lineage = ClaimLineage {
            id: claim.id,
            revision: claim.revision,
            attempt: claim.attempt,
            coordinator: claim.coordinator,
        };
        let migration_path = host.join(FENCE);
        let mut domain = migration::Migration {
            schema: "translation-execution-migration-v1".into(),
            state: "active".into(),
            checkout: host.clone(),
            effort: EFFORT.into(),
            coordinator: COORDINATOR.into(),
            session_id: session_id.clone(),
            generation: 1,
            authority: reference(&authority_path)?,
            ledger: reference(&running.ledger)?,
            claims: vec![migration::MigratedClaim {
                claim: lineage.clone(),
                scope_id: scope.subject.clone(),
                policy_sha256: "0".repeat(64),
                source_commit: baseline.clone(),
                writable_files: scope.writable_paths.clone(),
                original_claim: reference(&original_path)?,
                installation: None,
            }],
            historical: vec![],
            stopped_owners: vec![reference(&stopped_path)?],
            proof: reference(&proof_path)?,
            limitations: vec![LIMITATION.into()],
        };
        put(&migration_path, serde_json::to_vec(&domain)?)?;
        let pinned = root.join("pinned-source/snesrev-sm");
        copy_tree(&owner.join("deps/snesrev-sm"), &pinned)?;
        copy_tree(&owner.join("deps/downloads"), &root.join("pinned-source/downloads"))?;
        let contract = native::contract_for_family(
            &host,
            Path::new(MANIFEST),
            Asset { file: "inventory.toml".into(), sha256: scope.source_inventory_sha256 },
            native::NativeProvenance::NativeScope {
                scope: Asset { file: scope_path.clone(), sha256: digest(&scope_path)? },
                checkout: owner.clone(),
            },
            native::NativeLimits {
                timeout_seconds: 30,
                writable_bytes: 128 * 1024 * 1024,
                stdout_bytes: 4096,
                stderr_bytes: 256 * 1024,
                stdin_bytes: 0,
            },
            native::NativeLimits {
                timeout_seconds: 5,
                writable_bytes: 512 * 1024,
                stdout_bytes: 4096,
                stderr_bytes: 4096,
                stdin_bytes: 0,
            },
        )
        .context("bind declared managed diagnostic harness")?;
        let prepared = native::prepare(&host, &pinned, &contract).context("prepare pinned managed diagnostic")?;
        ensure!(prepared.source.revision == "a".repeat(40), "diagnostic pin must be full and archive-bound");
        let mut policy = tests::policy(
            &host,
            baseline.clone(),
            &[CANDIDATE],
            sealed(&vec![&prepared.dependencies_sha256])?,
            sealed(&vec![&prepared.verification_sha256])?,
            sealed(&vec![&prepared.source])?,
        )?;
        policy.scope_id = scope.subject;
        policy.claim = lineage;
        policy.budgets.max_attempts = 1;
        policy.budgets.gate_timeout_seconds = 60;
        policy.verification_stages = vec![VerificationStage::Focused, VerificationStage::Offline];
        let helper = std::env::var_os("SM_TRANSLATION_TEST_HELPER")
            .map(PathBuf::from)
            .unwrap_or_else(|| representative.join("installed/helper"));
        fs::create_dir_all(root.join("installed"))?;
        fs::copy(helper, root.join("installed/helper"))?;
        fs::copy("/usr/bin/x86_64-linux-gnu-gcc-9", root.join("installed/gcc-9"))?;
        let compiler = Command::new("rustup").args(["which", "rustc"]).output()?;
        ensure!(compiler.status.success(), "diagnostic Rust compiler missing");
        let rustc = PathBuf::from(String::from_utf8(compiler.stdout)?.trim());
        let protocol = compile_protocol(root, &rustc)?;
        // The sealed Installation retains its own exact reader/controller;
        // later test builds and disposable target retirement cannot replace it.
        fs::copy(std::env::current_exe()?, root.join("installed/controller"))?;
        let mut installation = Installation {
            schema: "translation-supervisor-installation-v1".into(),
            id: "managed-native-diagnostic".into(),
            controller: installed(root.join("installed/controller"))?,
            helper: installed(root.join("installed/helper"))?,
            rustc: installed(rustc)?,
            c_compiler: installed(root.join("installed/gcc-9"))?,
            source_repository: host.clone(),
            integration_repository: host.clone(),
            integration_baseline: baseline,
            pinned_source: pinned,
            state_root: root.join("state"),
            pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
            workers: 4,
            // The two-routine diagnostic reserves 814,900,336 bytes across its
            // bounded compiler/runtime operations before producing any output.
            max_gate_bytes: 1024 * 1024 * 1024,
            max_checkout_bytes: 64 * 1024 * 1024,
            max_context_bytes: 256 * 1024,
            rewrite: RewriteInstallation {
                executable: installed(protocol)?,
                recorded: false,
                limits: RunLimits {
                    timeout: Duration::from_secs(10),
                    writable_bytes: 1024 * 1024,
                    stdout_bytes: 256 * 1024,
                    stderr_bytes: 4096,
                    stdin_bytes: 256 * 1024,
                },
            },
            policy,
            contract: Contract::Native {
                contracts: vec![contract],
                catalog_sha256: digest(&host.join("tools/port/batch.toml"))?,
                focused_family: FAMILY.into(),
            },
            admission: Admission::Managed {
                checkout: host.clone(),
                effort: EFFORT.into(),
                coordinator: COORDINATOR.into(),
                session_id,
                generation: 1,
                migration: Asset { file: migration_path.clone(), sha256: "0".repeat(64) },
            },
        };
        installation.policy.adapter_sha256 = installation_identity(&installation)?;
        domain.claims[0].policy_sha256 = sealed(&installation.policy)?;
        put(&migration_path, serde_json::to_vec(&domain)?)?;
        let Admission::Managed { migration: seal, .. } = &mut installation.admission else { unreachable!() };
        seal.sha256 = digest(&migration_path)?;
        ensure!(
            installation.policy.adapter_sha256 == installation_identity(&installation)?,
            "diagnostic migration seal cycle"
        );
        write_once(&root.join("installation.json"), &installation)?;
        write_once(&root.join("original-owner-files.json"), &tests::records(&owner)?)?;
        Ok(Self { root: root.into(), owner, host, installation, original })
    }

    fn adapter(&self) -> Result<Adapter> {
        Adapter::new(self.installation.clone())
    }
    fn open(&self) -> Result<Supervisor<Adapter>> {
        let adapter = self.adapter()?;
        Supervisor::open(&adapter.run_root.join("journal"), self.installation.policy.clone(), adapter)
    }
    fn inspect(&self) -> Result<View> {
        let adapter = Adapter::existing(self.installation.clone())?;
        Supervisor::inspect_retained(&adapter.run_root.join("journal"), self.installation.policy.clone(), adapter)
    }
    fn queue(&self) -> Result<assignment::Receipt> {
        assignment::operate(assignment::Request {
            checkout: self.host.clone(),
            effort: EFFORT.into(),
            coordinator: COORDINATOR.into(),
            capacity: 3,
            ..Default::default()
        })
    }
}

#[test]
#[ignore = "one bounded managed actual-C diagnostic proof; requires installed confinement, private ROM and current thread"]
fn managed_native_publication_preserves_original_claim_and_recovers_exact_pending_once() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_MANAGED_NATIVE_PROOF_ROOT")?);
    ensure!(root.is_absolute() && !root.exists(), "managed diagnostic proof needs a fresh absolute retained root");
    fs::create_dir_all(&root)?;
    let _ = crate::BINARY_SHA256.set(crate::digest(&std::env::current_exe()?)?);
    let result = run_managed_proof(&root);
    if let Err(error) = &result {
        put(&root.join("failure.log"), format!("{error:#}\n"))?;
        eprintln!("Retained managed diagnostic failure: {}", root.display());
    }
    result
}

#[test]
#[ignore = "read-only authentication of retained managed diagnostic evidence; no producer execution"]
fn retained_managed_native_publication_authenticates_without_producers() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_MANAGED_NATIVE_PROOF_ROOT")?);
    ensure!(root.is_absolute() && root.is_dir(), "managed retained proof root missing");
    let _ = crate::BINARY_SHA256.set(crate::digest(&std::env::current_exe()?)?);
    let installation: Installation = read(&root.join("installation.json"))?;
    let project = ManagedProject {
        host: installation.integration_repository.clone(),
        owner: root.join("original-owner"),
        original: read(&root.join("original-claim.json"))?,
        installation,
        root: root.clone(),
    };
    let adapter = Adapter::existing(project.installation.clone())?;
    let run_before = tests::records(&adapter.run_root)?;
    let host_before = tests::records(&project.host)?;
    let owner_before = tests::records(&project.owner)?;
    let expected: View = read(&root.join("first-view.json"))?;
    ensure!(project.inspect()? == expected, "retained managed publication differs");
    let State::Complete { completion, .. } = &expected.state else {
        bail!("managed proof has no accepted completion");
    };
    ensure!(
        authenticate_managed_completion(&project.installation, completion)?.native_gates.len() == 4,
        "retained managed actual-C gates missing"
    );
    ensure!(
        tests::records(&adapter.run_root)? == run_before
            && tests::records(&project.host)? == host_before
            && tests::records(&project.owner)? == owner_before,
        "read-only managed authentication mutated source, queue or producer evidence"
    );
    Ok(())
}

fn run_managed_proof(root: &Path) -> Result<()> {
    let project = ManagedProject::new(root).context("construct exact original managed diagnostic")?;
    let owner_before = tests::records(&project.owner)?;
    let baseline = git(&project.host, &["rev-parse", "HEAD"])?;
    ensure!(baseline == project.installation.policy.baseline_commit);
    let adapter = project.adapter()?;
    let started = Instant::now();
    let mut supervisor = project.open()?;
    let completed = supervisor.run_with_rewrite(|adapter, context, capsule| adapter.rewrite(context, capsule, None))?;
    let loop_elapsed_seconds = started.elapsed().as_secs_f64();
    write_once(&root.join("first-view.json"), &completed)?;
    let State::Complete { completion, publication } = &completed.state else {
        bail!("managed diagnostic did not complete: {:?}", completed.state);
    };
    write_once(&root.join("completion.json"), completion)?;
    ensure!(
        completion.gates.len() == 4 && completion.gates.iter().all(|gate| gate.passed),
        "managed native prescribed gates incomplete"
    );
    ensure!(
        completion.finalization.worker_retired
            && completion.finalization.cleanup_complete
            && completion.finalization.retained_authenticatable,
        "managed producer/source cleanup incomplete"
    );
    let authenticated = authenticate_managed_completion(&project.installation, completion)?;
    ensure!(authenticated.native_gates.len() == 4, "managed actual-C receipts missing");
    let mut gate_producers = BTreeSet::new();
    let mut gate_producer_references = 0;
    for gate in &authenticated.native_gates {
        ensure!(
            gate.verdict.verified_routines == 2
                && gate.verdict.verified_cases == 2
                && gate.verdict.cases.len() == 2
                && gate.verdict.original_rom_parity == "not-tested"
                && !gate.verdict.live_exercised,
            "managed diagnostic fixture domain differs"
        );
        for operation in &gate.receipt.operations {
            gate_producer_references += 1;
            gate_producers.insert(operation.context.operation_key.clone());
        }
    }
    let host_head = git(&project.host, &["rev-parse", "HEAD"])?;
    ensure!(
        host_head == completion.integrated.commit && host_head != baseline,
        "shared source CAS did not publish integrated commit"
    );
    ensure!(
        git(&project.host, &["write-tree"])? == git(&project.host, &["rev-parse", "HEAD^{tree}"])?,
        "shared index CAS differs from published tree"
    );
    ensure!(
        git(&project.host, &["diff", "--name-only", &baseline, &host_head])? == CANDIDATE,
        "shared publication changed undeclared source"
    );
    ensure!(
        fs::read_to_string(project.host.join(CANDIDATE))?.contains("diagnostic-after"),
        "installed source replacement not published"
    );
    ensure!(tests::records(&project.owner)? == owner_before, "managed publication changed original owner evidence");
    let queue = project.queue()?;
    let original = queue
        .jobs
        .iter()
        .find(|entry| entry.job.id == project.original.claim.id)
        .context("original completed claim missing")?;
    ensure!(
        original.job == project.original.job
            && original.claim.as_ref() == Some(&project.original.claim)
            && original.agent == project.original.agent
            && original.attempts == 1
            && original.state == assignment::State::Complete
            && original.history.is_empty()
            && matches!(&original.result, Some(assignment::WorkResult::Submitted { commit, .. }) if commit == &completion.candidate.commit),
        "managed publication replaced original source/claim/worker lineage"
    );
    ensure!(queue.jobs.len() == 2 && queue.jobs.iter().any(|entry| entry.job.kind == assignment::Kind::Integrate
        && entry.state == assignment::State::Complete && entry.attempts == 1
        && matches!(&entry.result, Some(assignment::WorkResult::Integrated { submissions, .. }) if submissions == &[project.original.claim.clone()])),
        "managed standard derived integration transition missing or duplicated");
    let ticket = project.host.join(project.original.job.ticket.as_ref().unwrap());
    ensure!(fs::read_to_string(&ticket)?.contains("Status: resolved"), "managed origin ticket unresolved");
    let registry: serde_json::Value = read(&project.host.join(".scratch/decomp-progress/registry.json"))?;
    let updates = registry["updates"].as_array().context("managed progress history missing")?;
    ensure!(
        updates.len() == 3
            && updates.iter().map(|update| update["stage"].as_str().unwrap()).collect::<BTreeSet<_>>()
                == BTreeSet::from(["translated", "verified", "integrated"]),
        "managed stages incomplete or duplicated"
    );
    let record_path = adapter.attempt(1).join("publication/accepted.json");
    let record: PublicationRecord = read(&record_path)?;
    let managed = record.managed.as_ref().context("managed acceptance reference missing")?;
    migration::authenticate_publication(&project.installation, completion, managed)?;
    let accepted_path = &managed.accepted.path;
    let pending = accepted_path.with_extension("pending");
    let accepted_bytes = fs::read(accepted_path)?;
    let run_before = tests::records(&adapter.run_root)?;
    let ledger_before = fs::read(&queue.ledger)?;
    let host_before = git(&project.host, &["status", "--porcelain", "--untracked-files=all"])?;
    let queue_before = serde_json::to_vec(&queue.jobs)?;
    write_once(&root.join("producer-records-before-recovery.json"), &run_before)?;
    ensure!(supervisor.advance()? == completed && project.inspect()? == completed, "ordinary completed resume differs");
    ensure!(tests::records(&adapter.run_root)? == run_before, "completed resume allocated new producer evidence");
    for partial in [true, false] {
        fs::rename(accepted_path, &pending)?;
        let bytes = if partial {
            b"{\"schema\":".to_vec()
        } else {
            let mut changed: serde_json::Value = serde_json::from_slice(&accepted_bytes)?;
            changed["schema"] = "changed-managed-pending".into();
            serde_json::to_vec(&changed)?
        };
        inject_record(&pending, &bytes)?;
        ensure!(project.inspect().is_err(), "read-only inspect admitted partial/changed pending");
        ensure!(project.open()?.advance().is_err(), "resume admitted partial/changed pending");
        ensure!(fs::read(&pending)? == bytes && !accepted_path.exists(), "refused pending evidence was changed");
        put(&root.join(if partial { "refused-partial.pending" } else { "refused-changed.pending" }), bytes)?;
        inject_record(&pending, &accepted_bytes)?;
        fs::rename(&pending, accepted_path)?;
    }
    let exact_copy = root.join("exact-accepted-copy.json");
    put(&exact_copy, &accepted_bytes)?;
    for hardlink in [false, true] {
        fs::remove_file(accepted_path)?;
        if hardlink {
            fs::hard_link(&exact_copy, &pending)?;
        } else {
            symlink(&exact_copy, &pending)?;
        }
        ensure!(project.inspect().is_err(), "read-only inspect admitted aliased pending");
        ensure!(project.open()?.advance().is_err(), "resume admitted aliased pending");
        ensure!(pending.symlink_metadata().is_ok() && !accepted_path.exists(), "aliased refused pending removed");
        fs::remove_file(&pending)?;
        put(accepted_path, &accepted_bytes)?;
    }
    fs::rename(accepted_path, &pending)?;
    ensure!(
        project.inspect().is_err() && pending.exists() && !accepted_path.exists(),
        "read-only inspect promoted pending acceptance"
    );
    let recovered = project.open()?.advance()?;
    ensure!(
        recovered == completed && fs::read(accepted_path)? == accepted_bytes && !pending.exists(),
        "exact pending acceptance failed one-time byte-identical recovery"
    );
    let ledger_after = fs::read(&queue.ledger)?;
    ensure!(
        ledger_after == ledger_before && serde_json::to_vec(&project.queue()?.jobs)? == queue_before,
        "pending recovery duplicated/changed ordinary queue transitions"
    );
    ensure!(
        git(&project.host, &["rev-parse", "HEAD"])? == host_head
            && git(&project.host, &["status", "--porcelain", "--untracked-files=all"])? == host_before
            && tests::records(&adapter.run_root)? == run_before
            && tests::records(&project.owner)? == owner_before,
        "pending recovery changed source/original owner or launched duplicate producer"
    );
    let source_bytes = fs::read(project.host.join(CANDIDATE))?;
    put(&project.host.join(CANDIDATE), b"// foreign post-publication source\n")?;
    ensure!(project.inspect().is_err() && project.open()?.advance().is_err(), "managed host tamper granted admission");
    put(&project.host.join(CANDIDATE), source_bytes)?;
    let mut changed_acceptance: serde_json::Value = serde_json::from_slice(&accepted_bytes)?;
    changed_acceptance["schema"] = "changed-managed-acceptance".into();
    inject_record(accepted_path, &serde_json::to_vec(&changed_acceptance)?)?;
    ensure!(
        project.inspect().is_err() && project.open()?.advance().is_err(),
        "retained managed acceptance tamper admitted"
    );
    inject_record(accepted_path, &accepted_bytes)?;
    ensure!(
        project.open()?.advance()? == completed && project.inspect()? == completed,
        "restored managed completion no longer authenticates"
    );
    migration::authenticate_publication(&project.installation, completion, managed)?;
    ensure!(
        tests::records(&adapter.run_root)? == run_before
            && fs::read(&queue.ledger)? == ledger_before
            && tests::records(&project.owner)? == owner_before,
        "tamper refusal spawned work or changed original lineage"
    );
    let operations = run_before
        .keys()
        .filter(|path| path.starts_with(adapter.run_root.join("operations")))
        .filter(|path| path.file_name().is_some_and(|name| name == "result.json"))
        .count();
    ensure!(operations > 0, "managed producer measurement omitted operation results");
    let logical_bytes = run_before.values().map(|(_, bytes)| bytes).sum::<u64>();
    write_once(
        &root.join("proof.json"),
        &serde_json::json!({
            "schema": "translation-managed-native-diagnostic-proof-v1", "installation": root.join("installation.json"),
            "completion": root.join("completion.json"), "original_claim": root.join("original-claim.json"),
            "elapsed_seconds": started.elapsed().as_secs_f64(), "gates": completion.gates.len(),
            "loop_elapsed_seconds": loop_elapsed_seconds,
            "fixture_pairs_per_gate": 2, "routines": 2, "operation_receipts": operations,
            "unique_gate_producers": gate_producers.len(), "gate_producer_references": gate_producer_references,
            "reused_gate_references": gate_producer_references - gate_producers.len(),
            "run_logical_bytes": logical_bytes, "retained_finalization_bytes": completion.finalization.retained_bytes,
            "retained_publication_bytes": publication.retained_bytes,
            "reclaimed_checkout_bytes": completion.finalization.reclaimed_bytes,
            "pending_recovery_allocated_bytes": 0, "original_claim_attempt": 1,
            "host_baseline": baseline, "host_published_commit": host_head, "host_index_matches_commit": true,
            "partial_pending_refused": true, "changed_pending_refused": true,
            "symlink_pending_refused": true, "hardlinked_pending_refused": true,
            "exact_pending_promoted_once": true, "original_owner_unchanged": true,
            "limitations": [LIMITATION]
        }),
    )?;
    write_once(&root.join("producer-records-after-recovery.json"), &tests::records(&adapter.run_root)?)?;
    eprintln!("Retained complete managed diagnostic proof: {}", project.root.display());
    Ok(())
}
