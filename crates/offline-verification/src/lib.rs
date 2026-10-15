//! Complete actual-C Offline family verification and retained evidence admission.
//!
//! Ordinary callers submit one request. Fixture scheduling, adapter recipes,
//! compilation, comparison and evidence seals stay behind this interface.
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

mod builds;
mod callback;
mod family;
mod family_model;
mod fixture_execution;
mod fixture_format;
mod host;
mod inputs;
mod ordinary;
mod plm_fixtures;
mod plm_spawn;
mod recipe;
mod ridley_subsystem_adapter;
mod sources;

pub mod authoring;
pub mod catalog;
pub mod fixtures;
pub use family_model::{AuthenticateRequest, Verdict, VerifyRequest};
pub use host::HostInputs;
pub use ordinary::{PreviousFamily, RegisteredRequest, RegisteredRun};
pub use sources::IMPLEMENTATION_SOURCES;

use tool_execution::build_cache::CacheResult;
use tool_execution::{acquire_lease, atomic_toml, digest, metrics};

#[cfg(test)]
#[path = "testing_cleanup.rs"]
mod cleanup;

/// The application-owned verifier configuration; source and run paths are
/// operation inputs, so constructing it performs no filesystem access.
pub struct OfflineFamilies {
    host: HostInputs,
}

impl OfflineFamilies {
    pub fn new(host: HostInputs) -> Self {
        Self { host }
    }

    pub fn host_inputs(&self) -> &HostInputs {
        &self.host
    }

    pub fn verify(&self, request: VerifyRequest) -> Result<FamilyRun> {
        inputs::with_host(&self.host, || {
            let report = family::verify(request)?;
            Ok(FamilyRun { report, reused: false })
        })
    }

    pub fn authenticate(&self, request: AuthenticateRequest) -> Result<AuthenticatedFamily> {
        inputs::with_host(&self.host, || {
            decomp_source::context::limited(|| {
                let report = family::authenticate(request)?;
                AuthenticatedFamily::from_report(report)
            })
        })
    }
}

/// A completed run, including a retained rejected outcome.
pub struct FamilyRun {
    report: Verdict,
    reused: bool,
}

impl FamilyRun {
    pub fn report(&self) -> &Verdict {
        &self.report
    }
    pub fn into_report(self) -> Verdict {
        self.report
    }
    pub fn reused(&self) -> bool {
        self.reused
    }
    pub fn evidence(&self) -> PathBuf {
        self.report.run.join("verdict.toml")
    }
}

/// A passing family declaration admitted with its complete retained evidence.
/// Deserializing a verdict cannot construct this value.
pub struct AuthenticatedFamily {
    report: Verdict,
    declaration: fixtures::Manifest,
}

impl AuthenticatedFamily {
    fn from_report(report: Verdict) -> Result<Self> {
        let mut declaration = fixture_format::read_authenticated(
            &report.run.join(fixture_format::RUN_MANIFEST),
            report.artifacts.get(fixture_format::RUN_MANIFEST).context("missing admitted family declaration")?,
        )?;
        if report.implementation.as_ref().context("missing family implementation")?.dependency_contract == "rust-plm-v2"
        {
            declaration = plm_spawn::normalize(&declaration)?;
        }
        Ok(Self { report, declaration })
    }

    pub fn report(&self) -> &Verdict {
        &self.report
    }
    pub fn declaration(&self) -> &fixtures::Manifest {
        &self.declaration
    }
    pub fn into_report(self) -> Verdict {
        self.report
    }
    pub fn run(&self) -> &Path {
        &self.report.run
    }
}

/// Serialized report values are data, distinct from authenticated admission.
pub mod reports {
    pub use crate::family_model::{
        CallbackMetrics, CaseRecord, Implementation, Mismatch, RomIdentity, SpawnMetrics, Verdict,
    };
}
