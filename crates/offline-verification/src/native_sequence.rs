//! Closed composition of a frozen Native family prefix. Original acceptance
//! keeps canonical serial family ordering and the existing per-family waves.
use super::{ConfinedEnvironment, NativeEvidence, NativeReceipt, NativeTarget, OfflineFamilies, PreparedNative};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};
use tool_execution::bounded;
use translation_execution::{
    OperationContext,
    model::sealed,
    retention::{self, Reference},
};

pub const NATIVE_FAMILY_RECORD_BYTES: u64 = 64 * 1024 * 1024;

pub struct NativeSequenceJob {
    pub family: String,
    /// Preserve preparation failures in canonical family order.
    pub prepared: Result<PreparedNative>,
    pub required_prepared_sha256: String,
    pub context: OperationContext,
}

pub struct NativeSequence<'a> {
    pub checkout: &'a Path,
    pub output: &'a Path,
    pub environment: &'a ConfinedEnvironment,
    /// Already retained source/composite metadata in the same output budget.
    pub retained_bytes: u64,
    pub families: Vec<NativeSequenceJob>,
}

/// The identical prepared/receipt representation retained by Original gates.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeFamilyReceipt {
    pub prepared: PreparedNative,
    pub receipt: NativeReceipt,
}

/// A report projection. Original acceptance still authenticates every family
/// and its complete required prefix independently before accepting this result.
pub struct NativeSequenceRun {
    pub families: BTreeMap<String, Reference>,
    pub passed: bool,
    pub diagnostic: String,
    pub retained_bytes: u64,
}

impl OfflineFamilies {
    pub fn verify_native_sequence(&self, request: NativeSequence<'_>) -> Result<NativeSequenceRun> {
        ensure!(!request.families.is_empty(), "native family sequence is empty");
        ensure!(
            request.retained_bytes <= request.environment.max_output_bytes,
            "native sequence initial output bound exceeded"
        );
        let mut previous = None;
        for job in &request.families {
            ensure!(
                Path::new(&job.family).components().count() == 1
                    && matches!(Path::new(&job.family).components().next(), Some(Component::Normal(_)))
                    && previous.is_none_or(|previous: &str| previous < job.family.as_str()),
                "native sequence families must have unique canonical names and order"
            );
            previous = Some(job.family.as_str());
        }
        let mut result = NativeSequenceRun {
            families: BTreeMap::new(),
            passed: true,
            diagnostic: String::new(),
            retained_bytes: request.retained_bytes,
        };
        for job in request.families {
            let prepared = job.prepared?;
            ensure!(
                sealed(&prepared)? == job.required_prepared_sha256,
                "original native preparation changed: {}",
                job.family
            );
            let capacity = prepared
                .required_output_bytes()?
                .min(request.environment.max_output_bytes.saturating_sub(result.retained_bytes));
            ensure!(
                capacity >= NATIVE_FAMILY_RECORD_BYTES,
                "original catalog evidence budget exhausted before family launch"
            );
            let environment = ConfinedEnvironment {
                helper: request.environment.helper.clone(),
                rustc: request.environment.rustc.clone(),
                c_compiler: request.environment.c_compiler.clone(),
                operation_root: request.environment.operation_root.clone(),
                pipeline_root: request.environment.pipeline_root.clone(),
                workers: request.environment.workers,
                gate_timeout_seconds: request.environment.gate_timeout_seconds,
                deadline_unix_ms: request.environment.deadline_unix_ms,
                max_output_bytes: capacity,
            };
            let output = request.output.join("native").join(&job.family);
            let receipt = prepared.verify(
                NativeTarget { checkout: request.checkout, output: &output, context: &job.context, fixture_jobs: None },
                &environment,
                self.host_inputs(),
            )?;
            prepared.authenticate(NativeEvidence { checkout: request.checkout, receipt: &receipt })?;
            let advisory_bytes = receipt.advisory_output_bytes()?;
            result.retained_bytes = result
                .retained_bytes
                .checked_add(receipt.evidence_bytes)
                .and_then(|bytes| bytes.checked_add(advisory_bytes))
                .context("original catalog extent overflow")?;
            result.passed = receipt.passed;
            result.diagnostic = receipt.diagnostic.clone();
            let bytes = serde_json::to_vec(&NativeFamilyReceipt { prepared, receipt })?;
            ensure!(bytes.len() as u64 <= NATIVE_FAMILY_RECORD_BYTES, "original acceptance record extent exceeded");
            let retained_bytes = result
                .retained_bytes
                .checked_add(bytes.len() as u64)
                .context("original catalog record extent overflow")?;
            ensure!(
                retained_bytes <= request.environment.max_output_bytes,
                "original catalog exceeded its reserved capacity"
            );
            let path = request.output.join("families").join(format!("{}.json", job.family));
            bounded::safe(&path)?;
            fs::create_dir_all(path.parent().context("native sequence record parent missing")?)?;
            bounded::immutable_bytes(&path, &bytes)?;
            let reference = retention::seal_file(&path, NATIVE_FAMILY_RECORD_BYTES)?;
            ensure!(reference.bytes == bytes.len() as u64, "native sequence record extent changed during publication");
            result.retained_bytes = retained_bytes;
            result.families.insert(job.family, reference);
            if !result.passed {
                break;
            }
        }
        Ok(result)
    }
}
