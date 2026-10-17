//! Complete actual-C Offline family verification and retained evidence admission.
//!
//! Ordinary callers submit one request. Fixture scheduling, adapter recipes,
//! compilation, comparison and evidence seals stay behind this interface.
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

mod admission_reader;
mod builds;
mod bulk;
mod callback;
mod declarations;
mod family;
mod family_model;
mod fixture_execution;
mod fixture_format;
mod host;
mod identities;
mod input_view;
mod inputs;
mod native;
mod ordinary;
mod plm_fixtures;
mod plm_spawn;
mod recipe;
mod ridley_subsystem_adapter;
mod sources;
pub(crate) use bulk::work_graph;

pub mod authoring;
pub mod catalog;
pub use bulk::{
    AuthenticatedBatch, BatchAdmission, BatchInputRequest, BatchInputs, BatchRun, BatchVerification, FamilyCompletion,
    FamilyEvidence, FamilyJob,
};
pub use catalog::{CatalogRequest, CatalogSnapshot, CatalogStatistics, FamilyFacts};
pub mod fixtures;
pub mod source_facts;
pub use family_model::{AuthenticateRequest, Verdict, VerifyRequest};
pub use host::HostInputs;
pub use native::{
    AdmittedScope, AuthenticatedNative, ConfinedEnvironment, NATIVE_FAMILY_RECORD_BYTES, NativeContract,
    NativeDeclaration, NativeDeclarationKind, NativeEvidence, NativeFamilyReceipt, NativeLimits, NativePreparation,
    NativeProvenance, NativeReceipt, NativeSequence, NativeSequenceJob, NativeSequenceRun, NativeTarget,
    NoScopeAdmission, PreparedNative, ScopeAdmission,
};
pub use ordinary::{PreviousFamily, RegisteredRequest, RegisteredRun};
pub use sources::IMPLEMENTATION_SOURCES;

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

    pub fn capture_catalog(&self, request: CatalogRequest) -> Result<CatalogSnapshot> {
        catalog::capture(request)
    }

    pub fn verify(&self, request: VerifyRequest) -> Result<FamilyRun> {
        inputs::with_host(&self.host, || {
            let checkout = request.checkout.clone();
            let source = request.source.clone();
            let declarations = Arc::new(declarations::Declarations::standard());
            let report =
                declarations.enter(|| family::verify_with_declarations(request, &declarations, None, None, None))?;
            if report.status == "offline-verified" {
                family::admit_with_source(
                    AuthenticateRequest { checkout, verdict: report.run.join("verdict.toml") },
                    &source,
                    &declarations,
                )?;
            }
            Ok(FamilyRun { report, reused: false })
        })
    }

    pub fn authenticate(&self, request: AuthenticateRequest) -> Result<AuthenticatedFamily> {
        inputs::with_host(&self.host, || {
            decomp_source::context::limited(|| {
                let source = request.checkout.join("deps/snesrev-sm");
                Ok(AuthenticatedFamily::from_admitted(family::admit_with_source(
                    request,
                    &source,
                    &declarations::Declarations::standard(),
                )?))
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
    declaration: Arc<fixtures::Manifest>,
}

impl AuthenticatedFamily {
    fn from_admitted(admitted: family::AdmittedFamily) -> Self {
        Self { report: admitted.report, declaration: admitted.effective }
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
