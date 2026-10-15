//! Application-owned Translation scope approval for Native verification.
//! Recipe selection, producer execution and retained admission belong to the
//! Offline verifier; ticket and assignment policy remains here.
use anyhow::{Context, Result, ensure};
use offline_verification::{AdmittedScope, ScopeAdmission, fixtures::Asset};
use std::{fs, path::Path};
use tool_execution::bounded;

pub(crate) struct ScopeApproval;

impl ScopeAdmission for ScopeApproval {
    fn admit(&self, checkout: &Path, scope: &Asset) -> Result<AdmittedScope> {
        let checkout = fs::canonicalize(checkout)?;
        let path = if scope.file.is_absolute() { scope.file.clone() } else { checkout.join(&scope.file) };
        ensure!(bounded::digest(&path)? == scope.sha256, "native scope input changed");
        crate::translation::validate_scope(&checkout, &path)?;
        let declared: crate::translation::Scope = bounded::object(&path)?;
        ensure!(bounded::digest(&path)? == scope.sha256, "native scope changed during admission");
        Ok(AdmittedScope {
            checkout,
            scope: scope.clone(),
            routines: declared.routines.into_iter().collect(),
            writable_paths: declared
                .writable_paths
                .into_iter()
                .map(|path| Ok(path.to_str().context("native scope path is not UTF-8")?.to_owned()))
                .collect::<Result<_>>()?,
            inputs: declared.inputs,
        })
    }
}

#[cfg(test)]
#[path = "translation_native_tests.rs"]
mod tests;
