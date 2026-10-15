//! Public captured-input controls: real C/Rust observations, source/path
//! changes, complete retained admission and Native replay without producers.
use anyhow::{Context, Result, ensure};
use offline_verification::{
    AuthenticateRequest, BatchAdmission, BatchInputRequest, BatchVerification, CatalogRequest, ConfinedEnvironment,
    FamilyEvidence, FamilyJob, HostInputs, NativeDeclaration, NativeDeclarationKind, NativeEvidence, NativeLimits,
    NativePreparation, NativeProvenance, NativeReceipt, NativeTarget, NoScopeAdmission, OfflineFamilies,
    PreparedNative, RegisteredRequest,
    fixtures::{self, Asset, Case, Manifest, Routine},
    reports::CaseRecord,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};
use tool_execution::{atomic_toml, bounded, digest};
use translation_execution::{
    OperationContext,
    model::{ExecutionPhase, sealed, sha256},
};

const HOST: &[(&str, &str)] = &[("crates/offline-verification/tests/bulk_inputs.rs", include_str!("bulk_inputs.rs"))];

fn verifier() -> Result<OfflineFamilies> {
    Ok(OfflineFamilies::new(HostInputs::new(HOST, digest(&std::env::current_exe()?)?)?))
}

struct Project {
    root: PathBuf,
    cleaned: bool,
}

impl Project {
    fn new(name: &str) -> Result<Self> {
        let proof =
            PathBuf::from(std::env::var("SM_BULK_PROOF_ROOT").context("explicit retained proof root required")?);
        let root = proof.join(format!("{name}-{}-{}", std::process::id(), tool_execution::unique_id()));
        fs::create_dir_all(root.join("source"))?;
        let mut project = Self { root, cleaned: false };
        project.initialize()?;
        println!("Retained bulk control: {}", project.root.display());
        Ok(project)
    }

    fn checkout(&self) -> PathBuf {
        self.root.join("source")
    }
    fn source(&self) -> PathBuf {
        self.checkout().join("deps/snesrev-sm")
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) -> Result<()> {
        let path = self.checkout().join(name);
        fs::create_dir_all(path.parent().context("fixture parent")?)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        for (name, content) in HOST
            .iter()
            .chain(offline_verification::IMPLEMENTATION_SOURCES)
            .chain(decomp_source::IMPLEMENTATION_SOURCES)
            .chain(tool_execution::IMPLEMENTATION_SOURCES)
            .chain(translation_execution::compiler_runtime::IMPLEMENTATION_SOURCES)
        {
            self.write(name, content)?;
        }
        self.write("crates/port/src/memory.rs", include_str!("../../port/src/memory.rs"))?;
        let rust_path = self.checkout().join("crates/port/src/fixture.rs");
        self.write(
            "crates/port/src/fixture.rs",
            format!(
                r#"use crate::memory::Wram;
pub fn dispatch(ram: &mut Wram<'_>, name: &str, a: u16, _b: u16) -> Result<u32, &'static str> {{
    if name != "Touch" {{ return Err("wrong routine"); }}
    let path = file!();
    if path != {rust_path:?} && path != "/inputs/rust/crates/port/src/fixture.rs" {{ return Err("Rust path changed"); }}
    ram.set_byte(17, a as u8);
    ram.set_byte(18, if path == {rust_path:?} {{ 1 }} else {{ 2 }});
    Ok(a as u32 + 1)
}}
"#,
                rust_path = rust_path.display().to_string()
            ),
        )?;
        let c_path = self.source().join("src/sm_80.c").display().to_string();
        self.write(
            "deps/snesrev-sm/src/sm_80.c",
            format!(
                r#"#include <stdint.h>
#include <string.h>
extern uint8_t g_ram[0x20000];
uint16_t Touch(uint16_t a) {{
  const char *path = __FILE__;
  int original = strcmp(path, {c_path:?}) == 0;
  if (!original && strcmp(path, "/inputs/c/src/sm_80.c") != 0) return 65000;
  g_ram[17] = (uint8_t)a;
  g_ram[18] = original ? 1 : 2;
  return a + 1;
}}
"#
            ),
        )?;
        self.write("deps/snesrev-sm/src/sm_rtl.h", "#include <stdbool.h>\n")?;
        self.write("deps/snesrev-sm/src/funcs.h", "")?;
        self.write("deps/snesrev-sm/LICENSE.txt", "synthetic actual-C source control\n")?;
        self.pin()?;
        self.write("tools/port/batch.toml", "[families]\na='family'\nb='family'\n[profiles]\nall=['a','b']\n")?;
        let before = (0..0x20000).map(|index| (index * 17) as u8).collect::<Vec<_>>();
        self.write("fixtures/before.wram", &before)?;
        for (name, a, mut image) in
            [("root", 3, vec![7u8; 0x20000]), ("continued", 5, vec![7u8; 0x20000]), ("captured", 9, before.clone())]
        {
            image[17] = a;
            image[18] = 1;
            self.write(&format!("fixtures/{name}.wram"), image)?;
        }
        let manifest = Manifest {
            family: "fixture".into(),
            bank: "sm_80.c".into(),
            rust_module: "crates/port/src/fixture.rs".into(),
            routines: vec![Routine {
                symbol: "Touch".into(),
                args: 1,
                returns: "uint16".into(),
                cases: [("root", 3, Some(7), None), ("continued", 5, None, Some("root")), ("captured", 9, None, None)]
                    .into_iter()
                    .map(|(name, a, fill, predecessor)| {
                        let expected = self.checkout().join(format!("fixtures/{name}.wram"));
                        Ok(Case {
                            name: name.into(),
                            a,
                            fill,
                            continue_from: predecessor.map(str::to_owned),
                            expected_return: Some(a as u64 + 1),
                            before_file: (name == "captured").then(|| self.checkout().join("fixtures/before.wram")),
                            before_sha256: (name == "captured").then(|| sha256(&before)),
                            expected_after_file: Some(expected.clone()),
                            expected_after_sha256: Some(digest(&expected)?),
                            ..Default::default()
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
                ..Default::default()
            }],
            ..Default::default()
        };
        fixtures::write(&self.checkout().join("tools/port/a.smf"), &manifest)?;
        fixtures::write(&self.checkout().join("tools/port/b.smf"), &manifest)?;
        Ok(())
    }

    fn pin(&self) -> Result<()> {
        let mut tar = Vec::new();
        for name in ["src/sm_80.c", "src/sm_rtl.h", "src/funcs.h", "LICENSE.txt"] {
            decomp_source::testing::append_tar(
                &mut tar,
                &format!("snesrev-sm/{name}"),
                b'0',
                &fs::read(self.source().join(name))?,
            );
        }
        tar.resize(tar.len() + 1024, 0);
        fs::create_dir_all(self.checkout().join("deps/downloads"))?;
        let hash = decomp_source::testing::gzip_archive(&self.checkout().join("deps/downloads/oracle.tar.gz"), &tar)?;
        self.write("tools/dependencies.toml", format!(
            "schema_version=1\n[[dependencies]]\nname='snesrev-sm'\narchive='oracle.tar.gz'\nsha256={hash:?}\nrevision='captured-input-control'\n"
        ))
    }

    fn request(&self, family: &str, label: &str) -> RegisteredRequest {
        RegisteredRequest {
            checkout: self.checkout(),
            source: self.source(),
            root: pipeline_root(),
            workers: 4,
            case_jobs: 2,
            family: family.into(),
            output: self.root.join(label),
            previous: None,
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        let cleanup =
            PathBuf::from(std::env::var("SM_PIPELINE_CLEANUP_BIN").context("retained cleanup reader required")?);
        for deduplicate in [false, true] {
            let mut command = Command::new(&cleanup);
            command.args(["cleanup", "--root"]).arg(&self.root).arg("--apply");
            if deduplicate {
                command.arg("--deduplicate");
            }
            let output = command.output()?;
            fs::write(
                self.root.join(if deduplicate { "cleanup-deduplicate.stdout" } else { "cleanup.stdout" }),
                &output.stdout,
            )?;
            fs::write(
                self.root.join(if deduplicate { "cleanup-deduplicate.stderr" } else { "cleanup.stderr" }),
                &output.stderr,
            )?;
            ensure!(output.status.success(), "scoped cleanup failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        self.cleaned = true;
        Ok(())
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        if !self.cleaned
            && let Err(error) = self.cleanup()
        {
            eprintln!("Required cleanup remains at {}: {error:#}", self.root.display());
        }
    }
}

fn pipeline_root() -> PathBuf {
    std::env::var_os("SM_PIPELINE_ROOT").map(PathBuf::from).unwrap_or_else(|| "/home/jim/sm-decomp/out/pipeline".into())
}

#[derive(Serialize, Deserialize)]
struct Replay {
    checkout: PathBuf,
    source: PathBuf,
    ordinary: Vec<PathBuf>,
    native: Option<(PathBuf, PathBuf)>,
}

fn replay_reader(path: &Path) -> Result<()> {
    let output = Command::new(std::env::current_exe()?)
        .args(["--exact", "retained_capture_admission_without_compilers", "--ignored", "--nocapture"])
        .env("SM_BULK_REPLAY", path)
        .env("PATH", "/unavailable-bulk-compilers")
        .output()?;
    fs::write(path.with_extension("reader.stdout"), &output.stdout)?;
    fs::write(path.with_extension("reader.stderr"), &output.stderr)?;
    ensure!(output.status.success(), "independent reader failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

#[test]
#[ignore = "retained independent reader; invoked with an exact replay manifest"]
fn retained_capture_admission_without_compilers() -> Result<()> {
    let replay: Replay = bounded::object(&PathBuf::from(std::env::var("SM_BULK_REPLAY")?))?;
    let offline = verifier()?;
    for verdict in replay.ordinary {
        let admitted = offline.authenticate(AuthenticateRequest { checkout: replay.checkout.clone(), verdict })?;
        ensure!(admitted.report().verified_cases == 3, "retained ordinary coverage changed");
    }
    if let Some((prepared, receipt)) = replay.native {
        let prepared: PreparedNative = bounded::object(&prepared)?;
        let receipt: NativeReceipt = bounded::object(&receipt)?;
        let admitted = prepared.authenticate(NativeEvidence { checkout: &replay.checkout, receipt: &receipt })?;
        ensure!(
            admitted.report().verified_cases == 3 && receipt.operation_count()? == 8,
            "retained Native coverage changed"
        );
    }
    Ok(())
}

fn observations(run: &offline_verification::Verdict) -> Result<(Vec<CaseRecord>, Vec<Vec<u8>>)> {
    let mut images = Vec::new();
    for case in &run.cases {
        let directory = run.run.join(&case.name);
        let c = fs::read(directory.join("c-after.wram"))?;
        ensure!(c.len() == 0x20000 && c == fs::read(directory.join("rust-after.wram"))?, "canonical state differs");
        images.push(c);
    }
    Ok((run.cases.clone(), images))
}

#[test]
fn ordinary_bulk_shares_capture_and_preserves_complete_path_sensitive_observations() -> Result<()> {
    let mut project = Project::new("ordinary")?;
    let offline = verifier()?;
    let single = offline.verify_registered(project.request("a", "single"))?;
    ensure!(single.run.report().status == "offline-verified", "{:?}", single.run.report().error);
    let expected = observations(single.run.report())?;
    let snapshot = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let required = vec!["a".into(), "b".into()];
    let inputs = offline.capture_batch_inputs(BatchInputRequest {
        catalog: &snapshot,
        source: &project.source(),
        required: &required,
    })?;
    let prepared_stats = snapshot.statistics();
    ensure!(prepared_stats.manifest_decodes == 1 && prepared_stats.source_reads > 0, "capture was not shared");
    let started = Instant::now();
    let mut completed = std::sync::Mutex::new(Vec::new());
    let batch = offline.verify_batch(
        BatchVerification {
            catalog: &snapshot,
            source: project.source(),
            root: pipeline_root(),
            workers: 4,
            family_jobs: 2,
            case_jobs: 2,
            inputs: Some(&inputs),
            families: required
                .iter()
                .map(|name| FamilyJob { family: name.clone(), output: project.root.join(name), previous: None })
                .collect(),
        },
        |outcome| {
            completed.lock().unwrap().push(outcome.family().to_owned());
            Ok(())
        },
    )?;
    ensure!(completed.get_mut().unwrap().len() == 2, "incremental publication lost a family");
    let mut evidence = Vec::new();
    let mut verdicts = vec![single.run.evidence()];
    for outcome in batch.into_families() {
        let name = outcome.family().to_owned();
        let result = outcome.into_result()?;
        ensure!(result.run.report().status == "offline-verified", "{:?}", result.run.report().error);
        ensure!(observations(result.run.report())? == expected, "single/bulk complete observations differ");
        verdicts.push(result.run.evidence());
        evidence.push(FamilyEvidence {
            family: name,
            verdict: result.run.evidence(),
            manifest_sha256: result.manifest_sha256,
            kind: result.kind,
        });
    }
    let stats = snapshot.statistics();
    ensure!(
        stats.source_reads == prepared_stats.source_reads && stats.manifest_decodes == 1,
        "batch recaptured source or declarations"
    );
    ensure!(
        stats.source_freshness_reads > prepared_stats.source_freshness_reads && stats.transient_bytes == 0,
        "freshness or temporary-buffer accounting changed"
    );
    let admitted = offline.authenticate_batch(BatchAdmission {
        catalog: &snapshot,
        required: &required,
        source: project.source(),
        evidence,
    })?;
    ensure!(admitted.families().len() == 2, "batch admission membership differs");
    atomic_toml(&project.root.join("capture-statistics.toml"), &stats)?;
    atomic_toml(
        &project.root.join("timing.toml"),
        &BTreeMap::from([("bulk_seconds", started.elapsed().as_secs_f64())]),
    )?;
    drop((inputs, snapshot, admitted));
    let replay = project.root.join("replay.toml");
    atomic_toml(
        &replay,
        &Replay { checkout: project.checkout(), source: project.source(), ordinary: verdicts, native: None },
    )?;
    project.cleanup()?;
    replay_reader(&replay)
}

#[test]
fn changed_source_after_batch_capture_refuses_before_any_producer() -> Result<()> {
    let mut project = Project::new("source-mutation")?;
    let offline = verifier()?;
    let snapshot = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let required = vec!["a".into(), "b".into()];
    let inputs = offline.capture_batch_inputs(BatchInputRequest {
        catalog: &snapshot,
        source: &project.source(),
        required: &required,
    })?;
    let candidate = project.checkout().join("crates/port/src/fixture.rs");
    fs::write(&candidate, fs::read_to_string(&candidate)?.replace("a as u32 + 1", "a as u32 + 2"))?;
    let output = project.root.join("must-not-launch");
    let result = offline.verify_batch(
        BatchVerification {
            catalog: &snapshot,
            source: project.source(),
            root: pipeline_root(),
            workers: 4,
            family_jobs: 2,
            case_jobs: 2,
            inputs: Some(&inputs),
            families: required
                .iter()
                .map(|name| FamilyJob { family: name.clone(), output: output.join(name), previous: None })
                .collect(),
        },
        |_| Ok(()),
    );
    let error = result.err().context("changed captured candidate was accepted")?;
    ensure!(
        format!("{error:#}").contains("changed after capture") && !output.exists(),
        "unexpected mutation outcome: {error:#}"
    );
    fs::write(project.root.join("rejection.txt"), format!("{error:#}\n"))?;
    project.cleanup()
}

#[test]
fn macro_and_continued_c_includes_cannot_escape_captured_inputs() -> Result<()> {
    for (label, directive) in
        [("macro", "#define ESCAPED_HEADER {path}\n#include ESCAPED_HEADER\n"), ("continued", "#include \\\n {path}\n")]
    {
        let mut project = Project::new(label)?;
        let outside = project.root.join("uncaptured.h");
        fs::write(&outside, "#define ESCAPED 1\n")?;
        let bank = project.source().join("src/sm_80.c");
        let original = fs::read_to_string(&bank)?;
        fs::write(
            &bank,
            format!("{}{}", directive.replace("{path}", &format!("{:?}", outside.display().to_string())), original),
        )?;
        project.pin()?;
        let result = verifier()?.verify_registered(project.request("a", "rejected"))?;
        ensure!(result.run.report().status == "rejected", "uncaptured preprocessor input accepted");
        ensure!(
            result
                .run
                .report()
                .error
                .as_deref()
                .is_some_and(|error| error.contains("C dependency is outside captured inputs")),
            "wrong escape rejection: {:?}",
            result.run.report().error
        );
        project.cleanup()?;
    }
    Ok(())
}

#[test]
fn native_capture_replays_without_tools_and_binds_mapping_and_source() -> Result<()> {
    let mut project = Project::new("native")?;
    let offline = verifier()?;
    let manifest_path = project.checkout().join("tools/port/a.smf");
    let mut manifest = fixtures::read(&manifest_path)?;
    for case in &mut manifest.routines[0].cases {
        case.expected_after_file = None;
        case.expected_after_sha256 = None;
    }
    fixtures::write(&manifest_path, &manifest)?;
    let inventory = project.root.join("inventory.toml");
    atomic_toml(&inventory, &decomp_source::inventory(&project.checkout(), &project.source())?)?;
    let catalog = project.root.join("catalog.toml");
    atomic_toml(&catalog, offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?.catalog())?;
    let requirements = project.root.join("requirements.md");
    fs::write(
        &requirements,
        "Complete three-Fixture actual-C source capture and independent Native replay controls.\n",
    )?;
    let contract = offline.declare_native(NativeDeclaration {
        checkout: &project.checkout(),
        kind: NativeDeclarationKind::Family(Path::new("tools/port/a.smf")),
        source_inventory: Asset { file: inventory.clone(), sha256: digest(&inventory)? },
        provenance: NativeProvenance::ReviewedCatalog {
            catalog: Asset { file: catalog.clone(), sha256: digest(&catalog)? },
            requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? },
            checkout: project.checkout(),
            rationale: "Complete finite path-sensitive source capture controls".into(),
        },
        compiler_limits: NativeLimits {
            timeout_seconds: 180,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        },
        fixture_limits: NativeLimits {
            timeout_seconds: 5,
            writable_bytes: 512 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        },
        deadline_unix_ms: None,
        max_input_bytes: bounded::DEFAULT_BYTES,
    })?;
    let prepared = offline
        .prepare_native(NativePreparation::new(&project.checkout(), &project.source(), &contract), &NoScopeAdmission)?;
    let prepared_path = project.root.join("prepared.toml");
    atomic_toml(&prepared_path, &prepared)?;
    let output = project.root.join("run");
    let runtime = ConfinedEnvironment {
        helper: PathBuf::from(std::env::var("SM_BULK_NATIVE_HELPER")?),
        rustc: PathBuf::from(std::env::var("SM_BULK_NATIVE_RUSTC")?),
        c_compiler: "/usr/bin/x86_64-linux-gnu-gcc-9".into(),
        operation_root: project.root.join("operations"),
        pipeline_root: pipeline_root(),
        workers: 4,
        gate_timeout_seconds: 600,
        deadline_unix_ms: None,
        max_output_bytes: bounded::DEFAULT_BYTES,
    };
    let context = OperationContext {
        run_id: sha256(project.root.display().to_string()),
        attempt: 1,
        policy_sha256: contract.manifest.sha256.clone(),
        operation_key: sha256("captured-native-inputs-control"),
        phase: ExecutionPhase::Candidate,
    };
    let receipt = prepared.verify(
        NativeTarget { checkout: &project.checkout(), output: &output, context: &context, fixture_jobs: Some(2) },
        &runtime,
        offline.host_inputs(),
    )?;
    ensure!(receipt.passed, "{}", receipt.diagnostic);
    let admitted = prepared.authenticate(NativeEvidence { checkout: &project.checkout(), receipt: &receipt })?;
    ensure!(
        receipt.operation_count()? == 8 && admitted.report().verified_cases == 3,
        "Native producer membership changed"
    );
    for image in observations(admitted.report())?.1 {
        ensure!(image[18] == 2, "confined compiler path semantics changed");
    }
    let captures = output.join("family/captured-inputs");
    let candidate = captures.join("checkout/crates/port/src/fixture.rs");
    let original = fs::read(&candidate)?;
    fs::write(&candidate, b"changed captured source\n")?;
    ensure!(
        prepared.authenticate(NativeEvidence { checkout: &project.checkout(), receipt: &receipt }).is_err(),
        "captured source tampering admitted"
    );
    fs::write(&candidate, &original)?;
    fs::write(captures.join("extra"), b"undeclared")?;
    ensure!(
        prepared.authenticate(NativeEvidence { checkout: &project.checkout(), receipt: &receipt }).is_err(),
        "extra captured member admitted"
    );
    fs::remove_file(captures.join("extra"))?;
    let record = output.join("family/captured-inputs.toml");
    let record_before = (digest(&record)?, fs::metadata(&record)?.modified()?);
    let before = admitted
        .references()
        .iter()
        .map(|reference| {
            Ok((reference.path.clone(), digest(&reference.path)?, fs::metadata(&reference.path)?.modified()?))
        })
        .collect::<Result<Vec<_>>>()?;
    drop((admitted, prepared));
    let prepared: PreparedNative = bounded::object(&prepared_path)?;
    let absent_runtime = ConfinedEnvironment {
        helper: project.root.join("unavailable-helper"),
        rustc: project.root.join("unavailable-rustc"),
        c_compiler: project.root.join("unavailable-cc"),
        ..runtime
    };
    let replay = prepared.verify(
        NativeTarget { checkout: &project.checkout(), output: &output, context: &context, fixture_jobs: Some(2) },
        &absent_runtime,
        offline.host_inputs(),
    )?;
    ensure!(sealed(&replay)? == sealed(&receipt)?, "Native replay changed outcome");
    ensure!(
        record_before == (digest(&record)?, fs::metadata(&record)?.modified()?),
        "replay rematerialized captured inputs"
    );
    for (path, hash, modified) in before {
        ensure!(
            digest(&path)? == hash && fs::metadata(&path)?.modified()? == modified,
            "replay rewrote retained producer evidence"
        );
    }
    let current_candidate = project.checkout().join("crates/port/src/fixture.rs");
    let source_before = fs::read(&current_candidate)?;
    fs::write(&current_candidate, b"changed current source\n")?;
    ensure!(
        prepared.authenticate(NativeEvidence { checkout: &project.checkout(), receipt: &receipt }).is_err(),
        "current candidate mutation admitted"
    );
    fs::write(&current_candidate, source_before)?;
    let replay_file = project.root.join("replay.toml");
    atomic_toml(
        &replay_file,
        &Replay {
            checkout: project.checkout(),
            source: project.source(),
            ordinary: vec![],
            native: Some((prepared_path, output.join("native-receipt.toml"))),
        },
    )?;
    project.cleanup()?;
    replay_reader(&replay_file)
}
