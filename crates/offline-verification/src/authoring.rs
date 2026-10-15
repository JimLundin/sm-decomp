//! Explicit developer-tool inputs for generated Fixture wrappers.
//! These operations generate source and record compiler input provenance; they
//! do not admit an Offline family or replace the complete verification call.
use crate::{OfflineFamilies, builds, inputs};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WrapperRequest {
    pub checkout: PathBuf,
    pub run: PathBuf,
    pub modules: Vec<PathBuf>,
    pub imports: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependenciesRequest {
    pub checkout: PathBuf,
    pub run: PathBuf,
    pub extra: Vec<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WrapperResult {
    pub path: PathBuf,
}

/// Location and shared resource limits for the fixed pristine Fixture worker.
pub struct FixtureWorkerRequest {
    pub checkout: PathBuf,
    pub root: PathBuf,
    pub run: PathBuf,
    pub workers: usize,
}

impl OfflineFamilies {
    pub fn generate_wrapper(&self, request: WrapperRequest) -> Result<WrapperResult> {
        inputs::with_host(self.host_inputs(), || inputs::wrapper(request))
    }

    pub fn record_dependencies(&self, request: DependenciesRequest) -> Result<BTreeMap<String, String>> {
        inputs::with_host(self.host_inputs(), || inputs::dependencies(request))
    }

    pub fn prepare_fixture_worker(
        &self,
        request: FixtureWorkerRequest,
    ) -> Result<tool_execution::build_cache::CacheResult> {
        let mut result = builds::helper_builds(builds::HelperBuildRequest {
            checkout: request.checkout,
            root: request.root,
            run: request.run,
            workers: request.workers,
            helpers: vec![builds::Helper::FixtureWorker],
        })?;
        Ok(result.artifacts.remove(0))
    }
}
