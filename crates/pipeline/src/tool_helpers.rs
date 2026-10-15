//! Application helper selection composed from the owning build interfaces.
use anyhow::{Result, ensure};
use offline_verification::authoring::FixtureWorkerRequest;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tool_execution::{
    bounded,
    build_cache::CacheResult,
    builds::{self, StandaloneArtifact, StandaloneBuildRequest, source_files, standalone_builds},
    metrics,
};

/// Existing CLI protocol for the application's maintained Cargo artifacts.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub package: String,
    pub member: String,
    pub destination: PathBuf,
    pub workers: usize,
}

pub(crate) fn cargo_build(request: CargoBuildRequest) -> Result<CacheResult> {
    fs::create_dir_all(&request.run)?;
    let mut metrics =
        metrics::Run::start(request.run.join(format!("{}-cache.metrics.toml", request.package)), "cargo-cache")?;
    let result = (|| -> Result<CacheResult> {
        let checkout = fs::canonicalize(request.checkout)?;
        ensure!(
            (request.package == "evidence" && request.member == "sm-evidence")
                || (request.package == "decomp" && request.member == "libdecomp.a"),
            "Cargo input closure is defined only for evidence and decomp"
        );
        let inputs = cargo_inputs(&checkout, &request.package)?;
        builds::cargo_build(
            builds::CargoBuildRequest {
                checkout,
                root: request.root,
                run: request.run,
                package: request.package,
                member: request.member,
                destination: request.destination,
                workers: request.workers,
                inputs,
            },
            &metrics.context,
        )
    })();
    if let Ok(cache) = &result {
        metrics.context.configuration("cache-key", &cache.key);
        metrics.context.configuration("cache-hit", cache.hit);
    }
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let finished = metrics.finish(metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

fn cargo_inputs(checkout: &Path, package: &str) -> Result<Vec<PathBuf>> {
    let mut inputs = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"].map(|p| checkout.join(p)).to_vec();
    inputs.push(checkout.join(format!("crates/{package}/Cargo.toml")));
    source_files(&checkout.join(format!("crates/{package}/src")), "rs", &mut inputs)?;
    if package == "evidence" {
        source_files(&checkout.join("crates/pipeline-model/src"), "rs", &mut inputs)?;
    }
    for path in bounded::entries(&checkout.join("crates"))? {
        let manifest = path.join("Cargo.toml");
        if manifest.is_file() {
            inputs.push(manifest);
        }
    }
    let config = checkout.join(".cargo");
    if config.is_dir() {
        for p in bounded::entries(&config)? {
            if p.is_file() {
                inputs.push(p);
            }
        }
    }
    let external = checkout.join("deps/rustysnes/crates/rustysnes-core/Cargo.toml");
    if external.exists() {
        inputs.push(external);
    }
    Ok(inputs)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Helper {
    FixtureWorker,
    RandomLibrary,
    RandomCli,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HelperBuildRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub workers: usize,
    pub helpers: Vec<Helper>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct HelperBuildResult {
    pub artifacts: Vec<CacheResult>,
}

pub(crate) fn helper_builds(request: HelperBuildRequest) -> Result<HelperBuildResult> {
    fs::create_dir_all(&request.run)?;
    let mut metrics = metrics::Run::start(request.run.join("helper-build.metrics.toml"), "helper-build")?;
    let result = prepare_helpers(request);
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    let finished = metrics.finish(metrics::outcome(error.as_deref()), error);
    if result.is_ok() {
        finished?;
    }
    result
}

fn prepare_helpers(request: HelperBuildRequest) -> Result<HelperBuildResult> {
    let HelperBuildRequest { checkout, root, run, workers, helpers } = request;
    ensure!(!helpers.is_empty() && helpers.len() <= 3, "invalid helper build extent");
    let mut artifacts = Vec::new();
    let mut random = Vec::new();
    let build_random = |random: &mut Vec<StandaloneArtifact>, artifacts: &mut Vec<CacheResult>| -> Result<()> {
        if !random.is_empty() {
            artifacts.extend(standalone_builds(StandaloneBuildRequest {
                checkout: checkout.clone(),
                root: root.clone(),
                run: run.clone(),
                workers,
                artifacts: std::mem::take(random),
            })?);
        }
        Ok(())
    };
    for helper in helpers {
        let (name, label, flags): (_, _, &[&str]) = match helper {
            Helper::FixtureWorker => {
                build_random(&mut random, &mut artifacts)?;
                artifacts.push(crate::offline::families()?.prepare_fixture_worker(FixtureWorkerRequest {
                    checkout: checkout.clone(),
                    root: root.clone(),
                    run: run.clone(),
                    workers,
                })?);
                continue;
            }
            Helper::RandomLibrary => ("librng.a", "rng-build", &["--crate-type=staticlib"]),
            Helper::RandomCli => ("rng-cli", "rng-cli-build", &["--cfg", "cli"]),
        };
        random.push(StandaloneArtifact {
            source: "tools/decomp/next_random.rs".into(),
            name: name.into(),
            label: label.into(),
            flags: flags.iter().map(|flag| (*flag).into()).collect(),
        });
    }
    build_random(&mut random, &mut artifacts)?;
    Ok(HelperBuildResult { artifacts })
}
