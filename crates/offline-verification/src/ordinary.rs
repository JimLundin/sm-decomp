//! Catalog selection, complete verification and authenticated ordinary reuse.
use crate::{
    AuthenticateRequest, CatalogRequest, CatalogSnapshot, FamilyRun, OfflineFamilies, VerifyRequest, family, inputs,
    plm_fixtures,
};
use anyhow::{Result, ensure};
use std::{path::PathBuf, time::Instant};
use tool_execution::metrics;

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
    /// Advisory time spent authenticating previous evidence on this invocation,
    /// including a failed attempt followed by fresh verification.
    pub reuse_authentication_seconds: Option<f64>,
    pub reuse_authentication_error: Option<String>,
}

impl OfflineFamilies {
    /// Resolve one catalog family, authenticate optional prior evidence, and
    /// perform complete declared verification when reuse is unavailable.
    pub fn verify_registered(&self, request: RegisteredRequest) -> Result<RegisteredRun> {
        let snapshot = self.capture_catalog(CatalogRequest::new(&request.checkout)?)?;
        snapshot.validate_current()?;
        let result = self.verify_registered_captured(request, &snapshot, None)?;
        snapshot.validate_current()?;
        Ok(result)
    }

    pub(crate) fn verify_registered_captured(
        &self,
        request: RegisteredRequest,
        snapshot: &CatalogSnapshot,
        inputs: Option<&std::sync::Arc<crate::input_view::InputView>>,
    ) -> Result<RegisteredRun> {
        snapshot.declarations().enter(|| {
            inputs::with_host(self.host_inputs(), || {
                let context = metrics::active();
                if let Some(context) = &context {
                    context.expect_stages(&[
                        "registered-input-authentication",
                        "reuse-authentication",
                        "family-verification",
                        "family-evidence-admission",
                    ]);
                }
                let (facts, mut manifest) = metrics::measure("registered-input-authentication", || {
                    metrics::check_cancelled()?;
                    ensure!(
                        std::fs::canonicalize(&request.checkout)? == snapshot.checkout(),
                        "registered request checkout differs from captured catalog"
                    );
                    snapshot.validate_family(&request.family)?;
                    let facts = snapshot.family(&request.family)?;
                    let entry = facts.entry();
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
                    let manifest = request.checkout.join(&entry.manifest);
                    Ok((facts, manifest))
                })?;
                let entry = facts.entry().clone();
                let manifest_sha256 = facts.manifest_sha256().to_owned();
                let mut reuse_authentication_seconds = None;
                let mut reuse_authentication_error = None;
                if let Some(previous) = &request.previous
                    && previous.manifest_sha256 == manifest_sha256
                    && previous.kind == entry.kind
                {
                    let started = Instant::now();
                    let authenticated = metrics::measure("reuse-authentication", || {
                        let admitted = family::admit_with_source(
                            AuthenticateRequest {
                                checkout: request.checkout.clone(),
                                verdict: previous.verdict.clone(),
                            },
                            &request.source,
                            snapshot.declarations(),
                        )?;
                        crate::bulk::check_admitted_binding(
                            &facts,
                            &admitted,
                            &request.checkout,
                            snapshot.declarations(),
                        )?;
                        snapshot.validate_family(&request.family)?;
                        Ok(admitted.report)
                    });
                    reuse_authentication_seconds = Some(started.elapsed().as_secs_f64());
                    match authenticated {
                        Ok(report) => {
                            if let Some(context) = &context {
                                context.stage_unavailable(
                                    "family-verification",
                                    "accepted retained evidence; no fresh producers",
                                );
                            }
                            return Ok(RegisteredRun {
                                run: FamilyRun { report, reused: true },
                                manifest_sha256,
                                kind: entry.kind,
                                reuse_authentication_seconds,
                                reuse_authentication_error: None,
                            });
                        }
                        Err(error) => {
                            reuse_authentication_error = Some(format!("{error:#}"));
                            metrics::check_cancelled()?;
                        }
                    }
                } else if let Some(context) = &context {
                    context.stage_unavailable(
                        "reuse-authentication",
                        "no eligible previous record with matching manifest and kind",
                    );
                }
                let mut generated_inputs = None;
                if entry.kind == "plm-instructions" {
                    let base = if let Some(inputs) = inputs {
                        inputs.clone()
                    } else {
                        crate::input_view::InputView::capture(
                            self.host_inputs(),
                            &request.checkout,
                            &request.source,
                            &[],
                            true,
                            snapshot.declarations(),
                        )?
                    };
                    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos();
                    let generated = base.enter(|| {
                        plm_fixtures::generate(
                            &request.checkout,
                            &request.output.join("fixtures").join(stamp.to_string()),
                            None,
                        )
                    })?;
                    generated_inputs = Some(base.generated_plm(&generated)?);
                    manifest = generated.manifest;
                }
                let verifying = context.as_ref().map(|context| context.stage("family-verification")).transpose()?;
                let generated = entry.kind == "plm-instructions";
                let report = family::verify_with_declarations(
                    VerifyRequest {
                        checkout: request.checkout.clone(),
                        source: request.source.clone(),
                        root: request.root,
                        workers: request.workers,
                        case_jobs: request.case_jobs,
                        manifest,
                        output: request.output.join("evidence"),
                        adapter: entry.kind.clone(),
                        execution: String::new(),
                    },
                    snapshot.declarations(),
                    (!generated).then(|| facts.captured()),
                    generated_inputs.as_ref().or(inputs),
                )?;
                if report.status == "offline-verified" {
                    let admitted = family::admit_with_source(
                        AuthenticateRequest {
                            checkout: request.checkout.clone(),
                            verdict: report.run.join("verdict.toml"),
                        },
                        &request.source,
                        snapshot.declarations(),
                    )?;
                    crate::bulk::check_admitted_binding(&facts, &admitted, &request.checkout, snapshot.declarations())?;
                    snapshot.validate_family(&request.family)?;
                }
                let run = FamilyRun { report, reused: false };
                if let Some(verifying) = verifying {
                    verifying.finish(metrics::outcome(run.report().error.as_deref()))?;
                }
                Ok(RegisteredRun {
                    run,
                    manifest_sha256,
                    kind: entry.kind,
                    reuse_authentication_seconds,
                    reuse_authentication_error,
                })
            })
        })
    }
}
