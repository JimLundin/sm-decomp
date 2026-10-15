//! Catalog selection, complete verification and authenticated ordinary reuse.
use crate::{AuthenticateRequest, FamilyRun, OfflineFamilies, VerifyRequest, catalog, family, inputs, plm_fixtures};
use anyhow::{Result, ensure};
use std::path::PathBuf;

pub struct RegisteredRequest {
    pub checkout: PathBuf,
    pub source: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub case_jobs: usize,
    pub family: String,
    /// Family-owned output directory; generated inputs and evidence are children.
    pub output: PathBuf,
    pub previous: Option<PreviousFamily>,
}

pub struct PreviousFamily {
    pub verdict: PathBuf,
    pub manifest_sha256: String,
    pub kind: String,
}

pub struct RegisteredRun {
    pub run: FamilyRun,
    pub manifest_sha256: String,
    pub kind: String,
}

impl OfflineFamilies {
    /// Resolve one catalog family, authenticate optional prior evidence, and
    /// perform complete declared verification when reuse is unavailable.
    pub fn verify_registered(&self, request: RegisteredRequest) -> Result<RegisteredRun> {
        inputs::with_host(self.host_inputs(), || {
            let catalog = catalog::catalog(&request.checkout)?;
            let entry = catalog::entry(&catalog, &request.family)?;
            ensure!(
                [
                    "family",
                    "offline-family",
                    "dispatcher",
                    "projectile",
                    "offline-projectile",
                    "callback",
                    "power-bomb",
                    "plm",
                    "plm-instructions",
                    "ridley-subsystem"
                ]
                .contains(&entry.kind.as_str()),
                "unsupported native family kind: {}",
                entry.kind
            );
            ensure!(entry.prepare.is_none(), "unreviewed family preparation command");
            let mut manifest = request.checkout.join(&entry.manifest);
            let manifest_sha256 = tool_execution::bounded::digest(&manifest)?;
            if let Some(previous) = &request.previous
                && previous.manifest_sha256 == manifest_sha256
                && previous.kind == entry.kind
                && let Ok(report) = family::authenticate_with_source(
                    AuthenticateRequest { checkout: request.checkout.clone(), verdict: previous.verdict.clone() },
                    &request.source,
                )
            {
                return Ok(RegisteredRun {
                    run: FamilyRun { report, reused: true },
                    manifest_sha256,
                    kind: entry.kind,
                });
            }
            if entry.kind == "plm-instructions" {
                let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
                manifest = plm_fixtures::generate(
                    &request.checkout,
                    &request.output.join("fixtures").join(stamp.to_string()),
                    None,
                )?
                .manifest;
            }
            let run = self.verify(VerifyRequest {
                checkout: request.checkout,
                source: request.source,
                root: request.root,
                workers: request.workers,
                case_jobs: request.case_jobs,
                manifest,
                output: request.output.join("evidence"),
                adapter: entry.kind.clone(),
                execution: String::new(),
            })?;
            Ok(RegisteredRun { run, manifest_sha256, kind: entry.kind })
        })
    }
}
