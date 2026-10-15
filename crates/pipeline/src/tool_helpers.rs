//! Application helper selection composed from the owning build interfaces.
use anyhow::{Result, ensure};
use offline_verification::authoring::FixtureWorkerRequest;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tool_execution::{
    build_cache::CacheResult,
    builds::{StandaloneArtifact, StandaloneBuildRequest, standalone_builds},
    metrics,
};

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
