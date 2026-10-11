use crate::{family, pattern_contract};
use anyhow::{Result, ensure};
use std::collections::BTreeSet;
use toml::Value;
pub const VERSION: u64 = 2;
pub const MANDATORY: [&str; 12] = [
    "preflight",
    "check",
    "lint",
    "instruction",
    "timing",
    "fallback-only-parity",
    "generated-parity",
    "changed-entry-coverage",
    "synchronization",
    "checkpoint-validation",
    "feedback-demonstrations",
    "instruction-diagnosis",
];
pub const CANDIDATE: [&str; 3] = ["generated-parity", "changed-entry-coverage", "instruction-diagnosis"];
pub const REUSABLE: [&str; 4] = ["fallback-only-parity", "synchronization", "check", "lint"];
pub fn domain() -> Value {
    crate :: report ! ({ "domain" : "Japan/USA any%" , "frames" : 18000 , })
}
pub fn validate(name: &str, report: &Value) -> Result<()> {
    ensure!(
        report.get("status").and_then(Value::as_str) == Some("pass")
            && report.get("complete").and_then(Value::as_bool) == Some(true),
        "mandatory gate failed or incomplete: {name}"
    );
    if ["instruction", "timing"].contains(&name) {
        ensure!(
            report["schema_version"] == crate::readiness_metadata::value(&3).expect("validated native metadata"),
            "fresh pattern evidence requires declaration contract"
        );
        pattern_contract::validate_report(report, name == "timing")?;
    }
    if ["fallback-only-parity", "generated-parity", "synchronization"].contains(&name) {
        ensure!(
            report["expected_frames"] == crate::readiness_metadata::value(&18000).expect("validated native metadata")
                && report["completed_frames"]
                    == crate::readiness_metadata::value(&18000).expect("validated native metadata"),
            "mandatory frame scope incomplete: {name}"
        );
    }
    if ["fallback-only-parity", "generated-parity"].contains(&name) {
        ensure!(
            report["matched_frames"] == crate::readiness_metadata::value(&18000).expect("validated native metadata"),
            "mandatory parity matched scope incomplete"
        );
    }
    if name == "generated-parity" {
        ensure!(
            report["generated_steps"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                && report["compiled_table_entries"]
                    .as_integer()
                    .and_then(|n| u64::try_from(n).ok())
                    .is_some_and(|n| n > 0),
            "generated gate lacks actual compiled execution"
        );
    }
    if name == "fallback-only-parity" {
        ensure!(
            report["generated_steps"] == crate::readiness_metadata::value(&0).expect("validated native metadata"),
            "fallback gate executed compiled code"
        );
    }
    if name == "instruction-diagnosis" {
        ensure!(
            report["expected_instructions"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                && report["completed_instructions"] == report["expected_instructions"],
            "focused instruction scope incomplete"
        );
    }
    if name == "synchronization" {
        ensure!(
            report["checkpoint_pc"] == crate::readiness_metadata::value(&"828948").expect("validated native metadata")
                && report["start_frame"] == crate::readiness_metadata::value(&272).expect("validated native metadata")
                && report["end_frame"] == crate::readiness_metadata::value(&18000).expect("validated native metadata")
                && report["candidate_frame_offset"]
                    == crate::readiness_metadata::value(&0).expect("validated native metadata")
                && report["events"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                && report["candidate_completed_frames"]
                    == crate::readiness_metadata::value(&18000).expect("validated native metadata"),
            "strict independent synchronization scope differs"
        );
    }
    if name == "changed-entry-coverage" {
        let executed = family::array(report, "executed")?;
        ensure!(
            report["generated_instructions"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                && report["uncovered_keys"] == crate::report!(vec![])
                && !executed.is_empty()
                && report["validated_distinct_keys"]
                    == crate::readiness_metadata::value(&executed.len()).expect("validated native metadata")
                && !family::field(report, "candidate_inventory_sha256")?.is_empty()
                && !family::field(report, "profile_sha256")?.is_empty(),
            "changed-entry actual execution evidence incomplete"
        );
        for key in family::array(report, "required_keys")? {
            ensure!(
                executed.iter().any(|row| row["address"] == key["address"]
                    && row["mx"] == key["mx"]
                    && row["count"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)),
                "required changed key did not execute"
            );
        }
    }
    if name == "checkpoint-validation" {
        ensure!(
            report["completed_frames"] == crate::readiness_metadata::value(&1).expect("validated native metadata")
                && report["observed_rollovers"]
                    == crate::readiness_metadata::value(&2).expect("validated native metadata"),
            "checkpoint cold validation scope incomplete"
        );
        for side in ["reference", "subject"] {
            let detail = &report[side];
            ensure!(
                detail["observed_rollovers"]
                    == crate::readiness_metadata::value(&2).expect("validated native metadata")
                    && detail["instructions"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                    && !family::field(detail, "ordered_trace_sha256")?.is_empty()
                    && !family::field(detail, "ending_sha256")?.is_empty(),
                "checkpoint complete cold side validation missing"
            );
        }
    }
    if name == "feedback-demonstrations" {
        let expected = [
            "correct-execution",
            "wrong-instruction",
            "transient-effect",
            "uncovered-key",
            "empty-compilation",
            "preflight-build-failure",
            "interrupted-truncated",
            "incompatible-stale",
            "paired-cache",
        ]
        .map(str::to_owned)
        .into_iter()
        .collect::<BTreeSet<_>>();
        let cases = report["cases"].as_table().ok_or_else(|| anyhow::anyhow!("feedback cases missing"))?;
        ensure!(
            cases.keys().cloned().collect::<BTreeSet<_>>() == expected
                && report["expected_cases"] == crate::readiness_metadata::value(&9).expect("validated native metadata")
                && report["completed_cases"]
                    == crate::readiness_metadata::value(&9).expect("validated native metadata")
                && cases.values().all(|r| r["status"]
                    == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                    && r["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")),
            "feedback demonstrations incomplete"
        );
    }
    Ok(())
}
