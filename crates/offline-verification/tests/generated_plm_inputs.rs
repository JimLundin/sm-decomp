//! Public generated-PLM capture controls using the complete real registration.
use anyhow::{Context, Result, ensure};
use offline_verification::{BatchInputRequest, CatalogRequest, HostInputs, OfflineFamilies, fixtures};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tool_execution::digest;

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
        let proof = PathBuf::from(std::env::var("SM_GENERATED_PLM_PROOF_ROOT")?);
        let source = PathBuf::from(std::env::var("SM_GENERATED_PLM_SOURCE")?);
        let root = proof.join(format!("{label}-{}-{}", std::process::id(), tool_execution::unique_id()));
        fs::create_dir_all(&root)?;
        let project = Self { root, cleaned: false };
        for directory in ["crates/port/src", "deps/snesrev-sm"] {
            copy_directory(&source.join(directory), &project.checkout().join(directory))?;
        }
        for path in [
            "tools/dependencies.toml",
            "tools/port/plm-instructions.smf",
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
        fs::write(
            project.checkout().join("tools/port/batch.toml"),
            "[families]\nplm-instructions='plm-instructions'\n[profiles]\n",
        )?;
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
