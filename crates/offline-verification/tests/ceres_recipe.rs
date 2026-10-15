//! Complete ordinary Ceres recipe and separate retained-admission gate.
//! Native attribution is verified by the complete reviewed-recipe matrix.
use anyhow::{Result, ensure};
use offline_verification::{AuthenticateRequest, HostInputs, OfflineFamilies, RegisteredRequest, fixtures};
use std::{fs, path::PathBuf};

fn verifier() -> Result<OfflineFamilies> {
    Ok(OfflineFamilies::new(HostInputs::new(
        &[("crates/offline-verification/tests/ceres_recipe.rs", include_str!("ceres_recipe.rs"))],
        tool_execution::digest(&std::env::current_exe()?)?,
    )?))
}

#[test]
#[ignore = "complete 1296-pair actual-C Ceres gate; requires pinned C and private ROM copies"]
fn complete_registered_ceres_recipe_matches_all_declared_fixtures() -> Result<()> {
    let checkout = fs::canonicalize(PathBuf::from(std::env::var("SM_CERES_CHECKOUT")?))?;
    let output = PathBuf::from(std::env::var("SM_CERES_ORDINARY_PROOF")?);
    ensure!(!output.exists(), "complete Ceres proof requires a fresh output directory");
    let manifest = fixtures::read(&checkout.join("tools/port/ceres-pad.smf"))?;
    ensure!(
        manifest.routines.iter().map(|routine| routine.cases.len()).sum::<usize>() == 1296,
        "complete Ceres declaration changed"
    );
    let workers = std::env::var("SM_PIPELINE_WORKERS")?.parse::<usize>()?;
    ensure!(workers == 4, "complete Ceres proof preserves shared four-worker capacity");
    let result = verifier()?.verify_registered(RegisteredRequest {
        source: checkout.join("deps/snesrev-sm"),
        checkout,
        root: std::env::var("SM_PIPELINE_ROOT")?.into(),
        workers,
        case_jobs: 4,
        family: "ceres-pad".into(),
        output: output.clone(),
        previous: None,
    })?;
    let report = result.run.report();
    println!("Complete Ceres verdict: {}", result.run.evidence().display());
    ensure!(report.status == "offline-verified", "complete Ceres rejected: {:?}", report.error);
    ensure!(
        report.verified_cases == 1296 && report.verified_routines == manifest.routines.len(),
        "complete Ceres coverage differs"
    );
    ensure!(!result.run.reused(), "complete Ceres requires fresh fixture producers");
    fs::write(output.join("verdict.path"), result.run.evidence().as_os_str().as_encoded_bytes())?;
    Ok(())
}

#[test]
#[ignore = "independent retained Ceres admission; executes no compiler or fixture"]
fn retained_complete_ceres_authenticates_without_producers() -> Result<()> {
    let checkout = fs::canonicalize(PathBuf::from(std::env::var("SM_CERES_CHECKOUT")?))?;
    let verdict = PathBuf::from(std::env::var("SM_CERES_VERDICT")?);
    let admitted = verifier()?.authenticate(AuthenticateRequest { checkout, verdict })?;
    ensure!(
        admitted.report().family == "ceres-pad" && admitted.report().verified_cases == 1296,
        "retained complete Ceres domain differs"
    );
    println!("Independently authenticated all 1296 Ceres fixture pairs");
    Ok(())
}
