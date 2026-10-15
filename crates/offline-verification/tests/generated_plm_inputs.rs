//! Public generated-PLM capture controls using the complete real registration.
use anyhow::{Context, Result, ensure};
use offline_verification::{
    BatchAdmission, BatchInputRequest, BatchVerification, CatalogRequest, FamilyEvidence, FamilyJob, HostInputs,
    OfflineFamilies, RegisteredRequest, fixtures,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tool_execution::{atomic_toml, digest};

const HOST: &[(&str, &str)] =
    &[("crates/offline-verification/tests/generated_plm_inputs.rs", include_str!("generated_plm_inputs.rs"))];

struct Project {
    root: PathBuf,
    cleaned: bool,
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        ensure!(!kind.is_symlink(), "source snapshot contains a symbolic link");
        let target = destination.join(entry.file_name());
        if kind.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else {
            ensure!(kind.is_file(), "source snapshot input is not regular");
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

impl Project {
    fn new(label: &str) -> Result<Self> {
        Self::with_catalog(label, false)
    }

    fn with_catalog(label: &str, complete: bool) -> Result<Self> {
        let proof = PathBuf::from(std::env::var("SM_GENERATED_PLM_PROOF_ROOT")?);
        let source = PathBuf::from(std::env::var("SM_GENERATED_PLM_SOURCE")?);
        let root = proof.join(format!("{label}-{}-{}", std::process::id(), tool_execution::unique_id()));
        fs::create_dir_all(&root)?;
        let project = Self { root, cleaned: false };
        if complete {
            copy_directory(&source, &project.checkout())?;
        }
        for directory in ["crates/port/src", "deps/snesrev-sm"] {
            copy_directory(&source.join(directory), &project.checkout().join(directory))?;
        }
        for path in [
            "tools/dependencies.toml",
            "tools/port/plm-instructions.smf",
            "tools/decomp/headless.c",
            "tools/decomp/headless_frame.h",
            "rust-toolchain.toml",
            "deps/downloads/snesrev-sm.tar.gz",
            "rom/sm.sfc",
        ] {
            let destination = project.checkout().join(path);
            fs::create_dir_all(destination.parent().context("input parent")?)?;
            fs::copy(source.join(path), destination)?;
        }
        for (path, contents) in HOST
            .iter()
            .chain(offline_verification::IMPLEMENTATION_SOURCES)
            .chain(decomp_source::IMPLEMENTATION_SOURCES)
            .chain(tool_execution::IMPLEMENTATION_SOURCES)
            .chain(translation_execution::compiler_runtime::IMPLEMENTATION_SOURCES)
        {
            let destination = project.checkout().join(path);
            fs::create_dir_all(destination.parent().context("source parent")?)?;
            fs::write(destination, contents)?;
        }
        if !complete {
            fs::write(
                project.checkout().join("tools/port/batch.toml"),
                "[families]\nplm-instructions='plm-instructions'\n[profiles]\n",
            )?;
        }
        println!("Retained generated PLM control: {}", project.root.display());
        Ok(project)
    }

    fn checkout(&self) -> PathBuf {
        self.root.join("source")
    }
    fn source(&self) -> PathBuf {
        self.checkout().join("deps/snesrev-sm")
    }

    fn cleanup(&mut self) -> Result<()> {
        let reader = PathBuf::from(std::env::var("SM_PIPELINE_CLEANUP_BIN")?);
        for deduplicate in [false, true] {
            let mut command = Command::new(&reader);
            command.args(["cleanup", "--root"]).arg(&self.root).arg("--apply");
            if deduplicate {
                command.arg("--deduplicate");
            }
            let output = command.output()?;
            let prefix = if deduplicate { "cleanup-deduplicate" } else { "cleanup" };
            fs::write(self.root.join(format!("{prefix}.stdout")), &output.stdout)?;
            fs::write(self.root.join(format!("{prefix}.stderr")), &output.stderr)?;
            ensure!(output.status.success(), "cleanup failed: {}", String::from_utf8_lossy(&output.stderr));
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

fn verifier() -> Result<OfflineFamilies> {
    Ok(OfflineFamilies::new(HostInputs::new(HOST, digest(&std::env::current_exe()?)?)?))
}

#[test]
#[ignore = "real pinned C/ROM capture; retains source and cleanup evidence"]
fn generated_plm_capture_ignores_missing_obsolete_rom() -> Result<()> {
    capture_obsolete(false)
}

#[test]
#[ignore = "real pinned C/ROM capture; retains source and cleanup evidence"]
fn generated_plm_capture_ignores_stale_obsolete_rom() -> Result<()> {
    capture_obsolete(true)
}

fn capture_obsolete(stale: bool) -> Result<()> {
    let mut project = Project::new(if stale { "obsolete-stale" } else { "obsolete-missing" })?;
    let registration = project.checkout().join("tools/port/plm-instructions.smf");
    let mut raw = fixtures::read(&registration)?;
    let placeholder = project.root.join("obsolete-generated.sfc");
    raw.rom.as_mut().context("raw PLM ROM missing")?.file = placeholder.clone();
    fixtures::write(&registration, &raw)?;
    if stale {
        fs::write(&placeholder, b"stale generated PLM placeholder control\n")?;
    }
    ensure!(placeholder.exists() == stale, "placeholder control differs");
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let required = vec!["plm-instructions".to_owned()];
    let inputs = offline
        .capture_batch_inputs(BatchInputRequest { catalog: &catalog, source: &project.source(), required: &required })
        .context("generated PLM capture must use the complete effective recipe, not the obsolete ROM placeholder")?;
    inputs.validate_current()?;
    ensure!(placeholder.exists() == stale, "capture materialized an obsolete ROM placeholder");
    if stale {
        ensure!(
            fs::read(&placeholder)? == b"stale generated PLM placeholder control\n",
            "capture rewrote stale placeholder"
        );
    }
    drop((inputs, catalog));
    project.cleanup()
}

#[test]
#[ignore = "real pinned C/ROM capture and full catalog; retains measured diagnostics"]
fn complete_catalog_capture_measures_the_real_input_budget() -> Result<()> {
    let mut project = Project::with_catalog("full-catalog", true)?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let required = catalog.catalog().families.keys().cloned().collect::<Vec<_>>();
    ensure!(required.len() == 177, "complete production catalog membership changed");
    let captured = offline.capture_batch_inputs(BatchInputRequest {
        catalog: &catalog,
        source: &project.source(),
        required: &required,
    });
    atomic_toml(&project.root.join("capture-statistics.toml"), &catalog.statistics())?;
    if let Err(error) = &captured {
        fs::write(project.root.join("capture-error.txt"), format!("{error:#}\n"))?;
    }
    let inputs = captured?;
    let validated = inputs.validate_current();
    atomic_toml(&project.root.join("validation-statistics.toml"), &catalog.statistics())?;
    validated?;
    drop((inputs, catalog));
    project.cleanup()
}

#[test]
#[ignore = "real generated inputs; verifies current source before any producer"]
fn generated_plm_source_and_base_rom_mutations_refuse_before_producers() -> Result<()> {
    for (label, path) in [
        ("base-rom-mutation", "rom/sm.sfc"),
        ("effective-rust-mutation", "crates/port/src/audio.rs"),
        ("registration-mutation", "tools/port/plm-instructions.smf"),
    ] {
        let mut project = Project::new(label)?;
        let offline = verifier()?;
        let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
        let required = vec!["plm-instructions".to_owned()];
        let inputs = offline.capture_batch_inputs(BatchInputRequest {
            catalog: &catalog,
            source: &project.source(),
            required: &required,
        })?;
        let path = project.checkout().join(path);
        let original = fs::read(&path)?;
        let mut changed = original.clone();
        changed[0] ^= 1;
        fs::write(&path, changed)?;
        let output = project.root.join("must-not-launch");
        let result = offline.verify_batch(
            BatchVerification {
                catalog: &catalog,
                source: project.source(),
                root: pipeline_root(),
                workers: 4,
                family_jobs: 1,
                case_jobs: 4,
                families: vec![FamilyJob { family: "plm-instructions".into(), output: output.clone(), previous: None }],
                inputs: Some(&inputs),
            },
            |_| Ok(()),
        );
        let error = result.err().context("changed generated-family input was accepted")?;
        ensure!(!output.exists(), "mutated input reached a producer");
        ensure!(format!("{error:#}").contains("changed"), "unexpected mutation refusal: {error:#}");
        fs::write(project.root.join("rejection.txt"), format!("{error:#}\n"))?;
        fs::write(&path, original)?;
        inputs.validate_current()?;
        drop((inputs, catalog));
        project.cleanup()?;
    }
    Ok(())
}

fn pipeline_root() -> PathBuf {
    std::env::var_os("SM_PIPELINE_ROOT").map(PathBuf::from).unwrap_or_else(|| "/home/jim/sm-decomp/out/pipeline".into())
}

#[derive(Serialize, Deserialize)]
struct Replay {
    checkout: PathBuf,
    source: PathBuf,
    verdict: PathBuf,
    manifest_sha256: String,
    kind: String,
}

#[test]
#[ignore = "retained generated-family admission; requires no compiler"]
fn retained_generated_plm_authenticates_without_compilers() -> Result<()> {
    let replay: Replay = toml::from_str(&fs::read_to_string(std::env::var("SM_GENERATED_PLM_REPLAY")?)?)?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&replay.checkout)?)?;
    let required = vec!["plm-instructions".to_owned()];
    let admitted = offline.authenticate_batch(BatchAdmission {
        catalog: &catalog,
        required: &required,
        source: replay.source,
        evidence: vec![FamilyEvidence {
            family: "plm-instructions".into(),
            verdict: replay.verdict,
            manifest_sha256: replay.manifest_sha256,
            kind: replay.kind,
        }],
    })?;
    let family = &admitted.families()["plm-instructions"];
    ensure!(
        family.report().status == "offline-verified"
            && family.report().verified_cases == 1444
            && family.report().verified_routines == 6,
        "complete generated domain changed"
    );
    Ok(())
}

#[test]
#[ignore = "complete generated ordinary actual-C family; retains all evidence and cleanup"]
fn generated_plm_complete_ordinary_family_and_bulk_capture_compose() -> Result<()> {
    let mut project = Project::new("complete-ordinary")?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let required = vec!["plm-instructions".to_owned()];
    let inputs = offline.capture_batch_inputs(BatchInputRequest {
        catalog: &catalog,
        source: &project.source(),
        required: &required,
    })?;
    let batch = offline.verify_batch(
        BatchVerification {
            catalog: &catalog,
            source: project.source(),
            root: pipeline_root(),
            workers: 4,
            family_jobs: 1,
            case_jobs: 4,
            families: vec![FamilyJob {
                family: "plm-instructions".into(),
                output: project.root.join("bulk"),
                previous: None,
            }],
            inputs: Some(&inputs),
        },
        |_| Ok(()),
    )?;
    let run = batch.into_families().pop().context("generated family missing")?.into_result()?;
    ensure!(run.run.report().status == "offline-verified", "{:?}", run.run.report().error);
    ensure!(
        run.run.report().verified_cases == 1444 && run.run.report().verified_routines == 6,
        "complete generated coverage differs"
    );
    let reused = offline.verify_registered(RegisteredRequest {
        checkout: project.checkout(),
        source: project.source(),
        root: pipeline_root(),
        workers: 4,
        case_jobs: 4,
        family: "plm-instructions".into(),
        output: project.root.join("reuse"),
        previous: Some(offline_verification::PreviousFamily {
            verdict: run.run.evidence(),
            manifest_sha256: run.manifest_sha256.clone(),
            kind: run.kind.clone(),
        }),
    })?;
    ensure!(reused.run.reused(), "complete generated evidence did not replay");
    let replay = project.root.join("replay.toml");
    atomic_toml(
        &replay,
        &Replay {
            checkout: project.checkout(),
            source: project.source(),
            verdict: run.run.evidence(),
            manifest_sha256: run.manifest_sha256,
            kind: run.kind,
        },
    )?;
    drop((inputs, catalog));
    project.cleanup()?;
    let output = Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "retained_generated_plm_authenticates_without_compilers",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("SM_GENERATED_PLM_REPLAY", replay)
        .env("PATH", "/unavailable-generated-plm-compilers")
        .output()?;
    fs::write(project.root.join("reader.stdout"), &output.stdout)?;
    fs::write(project.root.join("reader.stderr"), &output.stderr)?;
    ensure!(output.status.success(), "retained generated reader failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}
