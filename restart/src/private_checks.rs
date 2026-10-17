//! A fixed adapter to the assertions also registered by Cargo tests.
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

#[cfg(feature = "native-preparation")]
#[path = "../native-host/tests/prepared_integrity.rs"]
mod prepared_integrity;

fn authenticated_input(variable: &str, expected_size: usize, expected_hash: &str) -> Result<String> {
    let path = PathBuf::from(std::env::var_os(variable).ok_or_else(|| format!("missing {variable}"))?);
    if !path.is_absolute() {
        return Err(format!("{variable} requires an absolute path"));
    }
    let size = std::fs::metadata(&path).map_err(|e| format!("{variable}: {e}"))?.len();
    if size != expected_size as u64 {
        return Err(format!("{variable} differs from the exact input size"));
    }
    let mut bytes = Vec::with_capacity(expected_size);
    std::fs::File::open(&path)
        .map_err(|e| format!("{variable}: {e}"))?
        .take(expected_size as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{variable}: {e}"))?;
    if bytes.len() != expected_size {
        return Err(format!("{variable} changed from the exact input size"));
    }
    let hash = format!("{:x}", Sha256::digest(&bytes));
    if hash != expected_hash {
        return Err(format!("{variable} differs from the exact input SHA256"));
    }
    Ok(hash)
}

fn execute(assertions: &[(&str, fn())], out: &Path, suite: &str, inputs: Vec<String>) -> Result<()> {
    if assertions.is_empty() {
        return Err("private assertion registration is empty".into());
    }
    if (suite == "item" && assertions.len() != 13)
        || (suite == "energy-prefix" && assertions.len() != 7)
        || (suite == "evidence" && assertions.len() != 45)
        || (suite == "return" && assertions.len() != 84)
        || (suite == "adapter" && assertions.len() != 1)
        || (suite == "sha256" && assertions.len() != 1)
        || (suite == "author41-result" && assertions.len() != 1)
        || (suite == "retained-reports" && assertions.len() != 2)
        || (suite == "retained-receipt" && assertions.len() != 1)
        || (suite == "author41-controls" && assertions.len() != 2)
        || (suite == "native-integrity" && assertions.len() != 1)
        || (suite == "ppm" && assertions.len() != 1)
        || (suite == "producer-inputs" && assertions.len() != 1)
        || (suite == "prepare-receipt" && assertions.len() != 1)
    {
        return Err("private assertion registration differs from the exact suite count".into());
    }
    let mut log = std::fs::File::create(out.join("assertions.log")).map_err(|e| e.to_string())?;
    let mut passed = 0;
    let mut failed = 0;
    for &(name, assertion) in assertions {
        writeln!(log, "START {name}").and_then(|()| log.flush()).map_err(|e| e.to_string())?;
        match std::panic::catch_unwind(assertion) {
            Ok(()) => {
                passed += 1;
                writeln!(log, "PASS {name}").map_err(|e| e.to_string())?;
            }
            Err(payload) => {
                failed += 1;
                let message = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("non-string panic");
                writeln!(log, "FAIL {name}: {message}").map_err(|e| e.to_string())?;
            }
        }
        log.flush().map_err(|e| e.to_string())?;
    }
    let mut report = sm_binary_evidence::Map::new();
    report.insert("schema".into(), "sm-private-assertions-v1".into());
    report.insert("suite".into(), suite.into());
    report.insert("requested".into(), (assertions.len() as i64).into());
    report.insert("started".into(), (assertions.len() as i64).into());
    report.insert("passed".into(), passed.into());
    report.insert("failed".into(), failed.into());
    report.insert(
        "private_assertions".into(),
        if suite == "item" {
            12
        } else if suite == "energy-prefix" {
            7
        } else if suite == "prepare-receipt" {
            1
        } else if suite == "native-integrity" {
            1
        } else if suite == "author41-controls" {
            2
        } else if suite == "retained-receipt" {
            1
        } else if suite == "retained-reports" {
            2
        } else if suite == "author41-result" {
            1
        } else {
            0
        }
        .into(),
    );
    report.insert(
        "input_free_assertions".into(),
        (if matches!(suite, "evidence" | "return" | "adapter" | "ppm" | "producer-inputs") {
            assertions.len() as i64
        } else if suite == "item" {
            1
        } else {
            0
        })
        .into(),
    );
    if matches!(
        suite,
        "author41-result"
            | "retained-reports"
            | "retained-receipt"
            | "author41-controls"
            | "native-integrity"
            | "prepare-receipt"
    ) {
        report.insert("expected_input_sha256".into(), inputs.clone().into());
        report.insert("input_authenticated".into(), (failed == 0).into());
        report.insert("input_sha256".into(), if failed == 0 { inputs } else { vec![] }.into());
    } else {
        report.insert("input_sha256".into(), inputs.into());
    }
    report.insert("status".into(), if failed == 0 { "passed" } else { "failed" }.into());
    report.insert("construction_capture_regressions_executed".into(), 0.into());
    report.insert("unavailable_construction_capture_regressions".into(), 3.into());
    std::fs::write(
        out.join("result.sme"),
        sm_binary_evidence::encode(&sm_binary_evidence::Value::Table(report.clone()))?,
    )
    .map_err(|e| e.to_string())?;
    if failed != 0 {
        return Err(format!("{failed} private assertions failed"));
    }
    Ok(())
}

/// Called once per externally supervised DLL process, before any Game use.
pub(crate) fn run(args: &[OsString]) -> Result<()> {
    if args.len() != 5 || args[1] != "--suite" || args[3] != "--out" {
        return Err(
            "private-checks requires exactly --suite item|energy-prefix|evidence|return|adapter|author41-result|retained-reports|retained-receipt|author41-controls|native-integrity --out ABSOLUTE_PATH"
                .into(),
        );
    }
    let suite = args[2].to_str().ok_or("invalid private assertion suite")?;
    let out = PathBuf::from(&args[4]);
    if !out.is_absolute() {
        return Err("private-checks requires an absolute output path".into());
    }
    let (assertions, inputs): (Vec<_>, Vec<_>) = match suite {
        "item" => (
            sm_item_instructions::private_assertions().to_vec(),
            vec![
                authenticated_input(
                    "SM_ITEM_FIXTURE_SEED",
                    131072,
                    "7adc5568e4239e479fcef325fef05769da8f6399e5909174d5f6f00c2f436459",
                )?,
                authenticated_input(
                    "SM_ITEM_FIXTURE_ROM",
                    3145728,
                    "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72",
                )?,
            ],
        ),
        "energy-prefix" => (
            crate::energy::private_assertions().to_vec(),
            vec![authenticated_input(
                "SM_ENERGY_PREFIX_WRAM",
                131072,
                "cb3362278173187dcda825bea8c4ec5c06952c1db7a5a948affb1c8d96a97483",
            )?],
        ),
        "author41-result" => (
            vec![(
                "retained_author41_binary_result_agrees_with_final_wram",
                tests::retained_author41_binary_result_agrees_with_final_wram as fn(),
            )],
            vec![
                "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43".into(),
                "0c6ccc844b22e640f0fc718b42ac0b80381aec56829a59b96be48eb30e2ad668".into(),
            ],
        ),
        "return" => (crate::rio_return::private_assertions(), vec![]),
        "sha256" => (sha2::private_assertions(), vec![]),
        "prepare-receipt" => (
            vec![(
                "prepared_control_receipt_binds_actual_dll",
                tests::prepared_control_receipt_binds_actual_dll as fn(),
            )],
            vec!["74b28e668cc8c2cca4b0be6bc217937da71d4466cc177582d1e91aacf5332a83".into()],
        ),
        #[cfg(feature = "native-preparation")]
        "producer-inputs" => (
            vec![(
                "producer_inputs_are_bounded_before_preparation",
                sm_native_host::producer_input_assertion() as fn(),
            )],
            vec![],
        ),
        #[cfg(not(feature = "png-output"))]
        "ppm" => (
            vec![(
                "ppm_writer_checks_rgb_extent_and_io_failures",
                crate::runtime_cli::ppm_writer_checks_rgb_extent_and_io_failures as fn(),
            )],
            vec![],
        ),
        #[cfg(feature = "native-preparation")]
        "native-integrity" => (
            vec![(
                "rejects_modified_library_and_modified_retained_source_on_cache_hit",
                prepared_integrity::rejects_modified_library_and_modified_retained_source_on_cache_hit as fn(),
            )],
            // The pinned archive has an independently fixed byte extent. Never
            // let caller metadata choose the allocation bound before hashing.
            vec![authenticated_input("SM_NATIVE_ARCHIVE", 1_031_954, sm_native_host::ARCHIVE_SHA256)?],
        ),
        "author41-controls" => (
            vec![
                (
                    "author41_loader_refuses_changed_hash_and_wram_extent",
                    tests::author41_loader_refuses_changed_hash_and_wram_extent as fn(),
                ),
                (
                    "author41_semantics_refuse_each_missing_mistyped_or_changed_field",
                    tests::author41_semantics_refuse_each_missing_mistyped_or_changed_field as fn(),
                ),
            ],
            vec![
                "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43".into(),
                "0c6ccc844b22e640f0fc718b42ac0b80381aec56829a59b96be48eb30e2ad668".into(),
            ],
        ),
        "retained-receipt" => (
            vec![(
                "retained_reports03_receipt_records_all_five_authenticated_inputs",
                tests::retained_reports03_receipt_records_all_five_authenticated_inputs as fn(),
            )],
            vec!["4bed20f222d6969418f09ad88cd2853c96fa9e5d88c51c3897e90fa11f417552".into()],
        ),
        "retained-reports" => (
            vec![
                (
                    "retained_author41_reports_distinguish_expected_and_authenticated_inputs",
                    tests::retained_author41_reports_distinguish_expected_and_authenticated_inputs as fn(),
                ),
                (
                    "retained_author42_landing_report_agrees_with_native_wram",
                    tests::retained_author42_landing_report_agrees_with_native_wram as fn(),
                ),
            ],
            vec![
                "30422f8569edb40ec5bb42ecfd73b7cf1e4f7afc8578e596b807df29e5bf2079".into(),
                "906315c0899d26abfc7320a0babdb5abe2c4b89ce5fb1c9f132cea02564a7621".into(),
                "fdbe5074aead59972a82ae01ff9d6aa852969c959fa1a41ae496953c06ccebf9".into(),
                "76d364d900209a261b81ff5bd20dc783a2e081d256f92fbefa3b66563f8d5b6d".into(),
                "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43".into(),
            ],
        ),
        "adapter" => (
            vec![(
                "assertion_failure_is_reported_and_cannot_be_accepted",
                tests::assertion_failure_is_reported_and_cannot_be_accepted as fn(),
            )],
            vec![],
        ),
        "evidence" => (
            sm_binary_evidence::private_assertions()
                .into_iter()
                .chain(sm_trace::private_assertions())
                .chain(crate::replay_trace::private_assertions())
                .chain(sm_native_host::private_assertions())
                .collect(),
            vec![],
        ),
        _ => {
            return Err(
                "private assertion suite must be item, energy-prefix, evidence, return, adapter, author41-result, retained-reports, retained-receipt, author41-controls or native-integrity"
                    .into(),
            );
        }
    };
    std::fs::create_dir(&out).map_err(|e| format!("create fresh assertion output: {e}"))?;
    execute(&assertions, &out, suite, inputs)
}

#[cfg(any(test, feature = "private-checks"))]
mod tests {
    use super::*;

    #[cfg_attr(test, test)]
    pub(super) fn prepared_control_receipt_binds_actual_dll() {
        let root = PathBuf::from(std::env::var_os("SM_PREPARE_CONTROL_ROOT").expect("prepare root"));
        assert!(root.is_absolute());
        let bytes = sm_binary_evidence::read_bytes(root.join("control-receipt.sme")).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "74b28e668cc8c2cca4b0be6bc217937da71d4466cc177582d1e91aacf5332a83"
        );
        let report = sm_binary_evidence::decode(&bytes).unwrap();
        assert_eq!(report["schema"].as_str(), Some("production-prepare-control-v2"));
        for (key, value) in [
            ("fresh_reused", false),
            ("second_reused", true),
            ("game_booted", false),
            ("native_exports_invoked", false),
        ] {
            assert_eq!(report[key].as_bool(), Some(value), "{key}");
        }
        for (key, path) in [("runtime_sha256", "public-runtime"), ("producer_sha256", "prepare-control")] {
            assert_eq!(report[key].as_str(), Some("ecdd4cf3b554aa7c1b3b1496838232eabd7e755e809cc9d0a22156a1cbb23606"));
            assert_eq!(
                format!(
                    "{:x}",
                    Sha256::digest(sm_native_host::preparation_control::producer_bytes(&root.join(path)).unwrap())
                ),
                report[key].as_str().unwrap()
            );
        }
        let library = Path::new(report["library"].as_str().unwrap());
        assert_eq!(sm_native_host::authenticate_library(library).unwrap(), report["library_sha256"].as_str().unwrap());
        assert_eq!(Path::new(report["manifest"].as_str().unwrap()), library.parent().unwrap().join("library.sme"));
        assert_eq!(
            report["producer_source_sha256"].as_str(),
            Some("3afed7d8740bab10835f1782ee329cd8ddc8fd0cd485615e26cce730fb94ed85")
        );
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(
                    sm_native_host::preparation_control::producer_bytes(&root.join("preparation_control.rs")).unwrap()
                )
            ),
            report["producer_source_sha256"].as_str().unwrap()
        );
        match report.get("elapsed_seconds") {
            Some(sm_binary_evidence::Value::Float(seconds)) => assert!(seconds.is_finite() && *seconds >= 0.0),
            _ => panic!("elapsed seconds must be finite float"),
        };
    }

    #[cfg_attr(test, test)]
    pub(super) fn assertion_failure_is_reported_and_cannot_be_accepted() {
        let out = std::env::temp_dir().join(format!("sm-private-check-control-{}", std::process::id()));
        std::fs::create_dir(&out).unwrap();
        assert!(execute(&[("passes", || {}), ("fails", || panic!("known failure"))], &out, "control", vec![]).is_err());
        let report: sm_binary_evidence::Value =
            sm_binary_evidence::decode(&std::fs::read(out.join("result.sme")).unwrap()).unwrap();
        assert_eq!(report["started"].as_integer(), Some(2));
        assert_eq!(report["passed"].as_integer(), Some(1));
        assert_eq!(report["failed"].as_integer(), Some(1));
        assert_eq!(report["status"].as_str(), Some("failed"));
        std::fs::remove_dir_all(out).unwrap();
    }
    #[cfg_attr(test, test)]
    pub(super) fn retained_author41_reports_distinguish_expected_and_authenticated_inputs() {
        let root =
            std::path::PathBuf::from(std::env::var_os("SM_AUTHOR41_REPORTS_ROOT").expect("report root required"));
        assert!(root.is_absolute());
        let expected_hashes = [
            "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43",
            "0c6ccc844b22e640f0fc718b42ac0b80381aec56829a59b96be48eb30e2ad668",
        ];
        for (run, hash, authenticated, passed, failed) in [
            (
                "binary-dll-author41-result-07-worker",
                "30422f8569edb40ec5bb42ecfd73b7cf1e4f7afc8578e596b807df29e5bf2079",
                true,
                1,
                0,
            ),
            (
                "binary-dll-author41-missing-07-worker",
                "906315c0899d26abfc7320a0babdb5abe2c4b89ce5fb1c9f132cea02564a7621",
                false,
                0,
                1,
            ),
        ] {
            let bytes = sm_binary_evidence::read_bytes(root.join(run).join("result.sme")).unwrap();
            assert_eq!(format!("{:x}", Sha256::digest(&bytes)), hash);
            let report = sm_binary_evidence::decode(&bytes).unwrap();
            assert_eq!(
                report.get("schema").and_then(sm_binary_evidence::Value::as_str),
                Some("sm-private-assertions-v1")
            );
            assert_eq!(report.get("suite").and_then(sm_binary_evidence::Value::as_str), Some("author41-result"));
            for (key, expected) in [
                ("requested", 1),
                ("started", 1),
                ("passed", passed),
                ("failed", failed),
                ("private_assertions", 1),
                ("input_free_assertions", 0),
            ] {
                assert_eq!(
                    report.get(key).and_then(sm_binary_evidence::Value::as_integer),
                    Some(expected),
                    "{run}/{key}"
                );
            }
            assert_eq!(
                report.get("input_authenticated").and_then(sm_binary_evidence::Value::as_bool),
                Some(authenticated)
            );
            assert_eq!(
                report.get("status").and_then(sm_binary_evidence::Value::as_str),
                Some(if authenticated { "passed" } else { "failed" })
            );
            let expected = report.get("expected_input_sha256").and_then(sm_binary_evidence::Value::as_array).unwrap();
            assert_eq!(expected.len(), 2);
            for (value, hash) in expected.iter().zip(expected_hashes) {
                assert_eq!(value.as_str(), Some(hash));
            }
            let observed = report.get("input_sha256").and_then(sm_binary_evidence::Value::as_array).unwrap();
            if authenticated {
                assert_eq!(observed, expected);
            } else {
                assert!(observed.is_empty(), "failed input must not be reported as authenticated");
            }
            println!("PASS {run}: status, counts and expected/observed authentication fields");
        }
    }

    #[cfg_attr(test, test)]
    #[cfg_attr(test, test)]
    pub(super) fn retained_reports03_receipt_records_all_five_authenticated_inputs() {
        let root = PathBuf::from(std::env::var_os("SM_AUTHOR41_REPORTS_ROOT").expect("report root required"));
        assert!(root.is_absolute());
        let bytes = sm_binary_evidence::read_bytes(root.join("ordinary-landing-reports-03-worker/result.sme")).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "4bed20f222d6969418f09ad88cd2853c96fa9e5d88c51c3897e90fa11f417552"
        );
        let report = sm_binary_evidence::decode(&bytes).unwrap();
        for (key, expected) in
            [("schema", "sm-private-assertions-v1"), ("suite", "retained-reports"), ("status", "passed")]
        {
            assert_eq!(report.get(key).and_then(sm_binary_evidence::Value::as_str), Some(expected), "{key}");
        }
        for (key, expected) in [
            ("requested", 2),
            ("started", 2),
            ("passed", 2),
            ("failed", 0),
            ("private_assertions", 2),
            ("input_free_assertions", 0),
        ] {
            assert_eq!(report.get(key).and_then(sm_binary_evidence::Value::as_integer), Some(expected), "{key}");
        }
        assert_eq!(report.get("input_authenticated").and_then(sm_binary_evidence::Value::as_bool), Some(true));
        let hashes = [
            "30422f8569edb40ec5bb42ecfd73b7cf1e4f7afc8578e596b807df29e5bf2079",
            "906315c0899d26abfc7320a0babdb5abe2c4b89ce5fb1c9f132cea02564a7621",
            "fdbe5074aead59972a82ae01ff9d6aa852969c959fa1a41ae496953c06ccebf9",
            "76d364d900209a261b81ff5bd20dc783a2e081d256f92fbefa3b66563f8d5b6d",
            "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43",
        ];
        for key in ["expected_input_sha256", "input_sha256"] {
            let values = report.get(key).and_then(sm_binary_evidence::Value::as_array).unwrap();
            assert_eq!(values.len(), 5, "{key}");
            for (value, expected) in values.iter().zip(hashes) {
                assert_eq!(value.as_str(), Some(expected), "{key}");
            }
        }
        println!(
            "PASS historical reports03 receipt: two passed, five authenticated inputs, zero input-free assertions"
        );
    }

    #[cfg_attr(test, test)]
    pub(super) fn retained_author41_binary_result_agrees_with_final_wram() {
        author41_result_check().unwrap();
    }

    #[cfg_attr(test, test)]
    pub(super) fn retained_author42_landing_report_agrees_with_native_wram() {
        let root = PathBuf::from(std::env::var_os("SM_AUTHOR41_REPORTS_ROOT").expect("report root required"));
        assert!(root.is_absolute());
        let root = root.join("rio-return-author-42-02-worker");
        let bytes = sm_binary_evidence::read_bytes(root.join("result.sme")).unwrap();
        let file = std::fs::File::open(root.join("final.wram")).unwrap();
        assert_eq!(file.metadata().unwrap().len(), 131072);
        let mut ram = Vec::new();
        file.take(131073).read_to_end(&mut ram).unwrap();
        assert_eq!(ram.len(), 131072);
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "fdbe5074aead59972a82ae01ff9d6aa852969c959fa1a41ae496953c06ccebf9"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(&ram)),
            "76d364d900209a261b81ff5bd20dc783a2e081d256f92fbefa3b66563f8d5b6d"
        );
        let report = sm_binary_evidence::decode(&bytes).unwrap();
        assert_eq!(report["schema"].as_str(), Some("sm-native-result-v2"));
        assert_eq!(report["status"].as_str(), Some("milestone-not-reached"));
        assert_eq!(report["scenario"].as_str(), Some("rio-return"));
        assert_eq!(report["original_rom_comparison"].as_str(), Some("not-tested"));
        assert_eq!(report["rio_return_complete"].as_bool(), Some(false));
        assert_eq!(report["rio_return_descent_release_y_subspeed"].as_integer(), Some(0xa800));
        for (key, value) in [
            ("steps", 25916),
            ("rio_return_crossing_span_step", 25906),
            ("rio_return_descent_release_step", 25907),
            ("rio_return_descent_landing_step", 25916),
            ("rio_return_descent_polls", 10),
        ] {
            assert_eq!(report[key].as_integer(), Some(value), "{key}");
        }
        for (key, offset) in [
            ("x", 0xaf6),
            ("y", 0xafa),
            ("pose", 0xa1c),
            ("radius", 0xb00),
            ("x_subposition", 0xaf8),
            ("y_subposition", 0xafc),
            ("x_speed", 0xb46),
            ("x_subspeed", 0xb48),
            ("y_speed", 0xb2e),
            ("y_subspeed", 0xb2c),
            ("y_direction", 0xb36),
            ("collision", 0xdd0),
            ("x_acceleration", 0xb4a),
        ] {
            let native = i64::from(u16::from_le_bytes([ram[offset], ram[offset + 1]]));
            assert_eq!(
                report[format!("rio_return_descent_landing_{key}").as_str()].as_integer(),
                Some(native),
                "{key}"
            );
        }
        let old_bytes = sm_binary_evidence::read_bytes(
            root.parent().unwrap().join("rio-return-author-41-final-05-worker/result.sme"),
        )
        .unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&old_bytes)),
            "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43"
        );
        let old = sm_binary_evidence::decode(&old_bytes).unwrap();
        for (key, value) in old.as_table().unwrap() {
            if [
                "rio_return_crossing_",
                "rio_return_boarding_",
                "rio_return_platform_",
                "rio_return_departure_",
                "rio_return_lower_floor_",
            ]
            .iter()
            .any(|prefix| key.starts_with(prefix))
            {
                assert_eq!(report.get(key), Some(value), "frozen {key}");
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn author41_semantics_refuse_each_missing_mistyped_or_changed_field() {
        let root = PathBuf::from(std::env::var_os("SM_AUTHOR41_EVIDENCE_ROOT").expect("fixture root required"));
        assert!(root.is_absolute());
        let bytes = sm_binary_evidence::read_bytes(root.join("result.sme")).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43"
        );
        let file = std::fs::File::open(root.join("final.wram")).unwrap();
        assert_eq!(file.metadata().unwrap().len(), 131072);
        let mut ram = Vec::new();
        file.take(131073).read_to_end(&mut ram).unwrap();
        assert_eq!(ram.len(), 131072);
        assert_eq!(
            format!("{:x}", Sha256::digest(&ram)),
            "0c6ccc844b22e640f0fc718b42ac0b80381aec56829a59b96be48eb30e2ad668"
        );
        let value = sm_binary_evidence::decode(&bytes).unwrap();
        assert!(author41_result_matches(&value, &ram).is_ok());
        assert!(author41_result_matches(&sm_binary_evidence::Value::Boolean(false), &ram).is_err());
        assert!(author41_result_matches(&value, &ram[..131071]).is_err());
        let mut oversized = ram.clone();
        oversized.push(0);
        assert!(author41_result_matches(&value, &oversized).is_err());
        let baseline = value.as_table().unwrap();
        // Each required key is isolated; mutations never retain another defect.
        let keys = [
            "schema",
            "scenario",
            "status",
            "original_rom_comparison",
            "rio_return_complete",
            "steps",
            "rio_return_crossing_span_step",
            "rio_return_crossing_span_x",
            "rio_return_crossing_span_y",
            "rio_return_crossing_span_pose",
            "rio_return_crossing_span_radius",
        ];
        for key in keys {
            let mut missing = baseline.clone();
            missing.remove(key);
            assert!(
                author41_result_matches(&sm_binary_evidence::Value::Table(missing), &ram).is_err(),
                "missing {key}"
            );
            let mut mistyped = baseline.clone();
            mistyped.insert(key.into(), sm_binary_evidence::Value::Array(vec![]));
            assert!(author41_result_matches(&sm_binary_evidence::Value::Table(mistyped), &ram).is_err(), "type {key}");
            let mut changed = baseline.clone();
            let replacement = match baseline.get(key).unwrap() {
                sm_binary_evidence::Value::String(_) => sm_binary_evidence::Value::String("different".into()),
                sm_binary_evidence::Value::Boolean(_) => sm_binary_evidence::Value::Boolean(true),
                sm_binary_evidence::Value::Integer(n) => sm_binary_evidence::Value::Integer(n + 1),
                _ => panic!("unexpected independent fixture field type"),
            };
            changed.insert(key.into(), replacement);
            assert!(author41_result_matches(&sm_binary_evidence::Value::Table(changed), &ram).is_err(), "value {key}");
        }
        // WRAM mismatches isolate recovered offset mapping from serialized values.
        for offset in [0xaf6, 0xafa, 0xa1c, 0xb00] {
            let mut changed = ram.clone();
            changed[offset] ^= 1;
            assert!(author41_result_matches(&value, &changed).is_err(), "WRAM offset {offset:x}");
        }
        // x=1467: horizontal radius5 gives edge1471 (admitted),6 gives1472 (refused).
        // Keep serialized fields unchanged to isolate the geometry guard itself.
        for (radius, admitted) in [(0_u16, false), (1, true), (5, true), (6, false), (u16::MAX, false)] {
            let mut geometry = ram.clone();
            geometry[0xafe..0xb00].copy_from_slice(&radius.to_le_bytes());
            assert_eq!(author41_result_matches(&value, &geometry).is_ok(), admitted, "horizontal radius {radius}");
        }
        println!(
            "PASS comparator: positive subject, 33 independent field refusals, 4 WRAM mismatches, exact geometry and extent bounds"
        );
    }

    fn author41_result_check() -> Result<()> {
        let root = std::env::var("SM_AUTHOR41_EVIDENCE_ROOT").map_err(|_| "missing SM_AUTHOR41_EVIDENCE_ROOT")?;
        author41_result_at(std::path::Path::new(&root))
    }

    fn author41_result_at(root: &Path) -> Result<()> {
        if !root.is_absolute() {
            return Err("SM_AUTHOR41_EVIDENCE_ROOT must be absolute".into());
        }
        let evidence = sm_binary_evidence::read_bytes(root.join("result.sme"))?;
        let file = std::fs::File::open(root.join("final.wram")).map_err(|e| e.to_string())?;
        if file.metadata().map_err(|e| e.to_string())?.len() != 131072 {
            return Err("unexpected WRAM extent".into());
        }
        let mut ram = Vec::new();
        file.take(131073).read_to_end(&mut ram).map_err(|e| e.to_string())?;
        if ram.len() != 131072 {
            return Err("unexpected WRAM extent".into());
        }
        for (name, bytes, expected) in [
            ("result.sme", evidence.as_slice(), "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43"),
            ("final.wram", ram.as_slice(), "0c6ccc844b22e640f0fc718b42ac0b80381aec56829a59b96be48eb30e2ad668"),
        ] {
            if format!("{:x}", Sha256::digest(bytes)) != expected {
                return Err(format!("retained author41 {name} differs from exact input"));
            }
        }
        let value = sm_binary_evidence::decode(&evidence)?;
        author41_result_matches(&value, &ram)
    }

    fn author41_result_matches(value: &sm_binary_evidence::Value, ram: &[u8]) -> Result<()> {
        if ram.len() != 131072 {
            return Err("unexpected WRAM extent".into());
        }
        let table = value.as_table().ok_or("result must be a table")?;
        for (key, expected) in [
            ("schema", "sm-native-result-v2"),
            ("scenario", "rio-return"),
            ("status", "milestone-not-reached"),
            ("original_rom_comparison", "not-tested"),
        ] {
            if table.get(key).and_then(sm_binary_evidence::Value::as_str) != Some(expected) {
                return Err(format!("unexpected {key}: {:?}", table.get(key)));
            }
            println!("PASS {key}={expected}");
        }
        if table.get("rio_return_complete").and_then(sm_binary_evidence::Value::as_bool) != Some(false) {
            return Err("full return must remain incomplete".into());
        }
        println!("PASS rio_return_complete=false");
        let integer = |key: &str| {
            table
                .get(key)
                .and_then(sm_binary_evidence::Value::as_integer)
                .ok_or_else(|| format!("missing integer {key}"))
        };
        for (key, expected) in [("steps", 25906), ("rio_return_crossing_span_step", 25906)] {
            if integer(key)? != expected {
                return Err(format!("unexpected {key}"));
            }
            println!("PASS {key}={expected}");
        }

        // Literal recovered WRAM locations, independent of result producer helpers.
        for (key, offset) in [
            ("rio_return_crossing_span_x", 0xaf6),
            ("rio_return_crossing_span_y", 0xafa),
            ("rio_return_crossing_span_pose", 0xa1c),
            ("rio_return_crossing_span_radius", 0xb00),
        ] {
            let expected = i64::from(u16::from_le_bytes([ram[offset], ram[offset + 1]]));
            if integer(key)? != expected {
                return Err(format!("{key} disagrees with retained final WRAM"));
            }
            println!("PASS {key}={expected} agrees with final WRAM");
        }
        let x = integer("rio_return_crossing_span_x")?;
        let radius = i64::from(u16::from_le_bytes([ram[0xafe], ram[0xaff]]));
        if radius <= 0 || x.checked_add(radius).and_then(|edge| edge.checked_sub(1)).is_none_or(|edge| edge >= 1472) {
            return Err("first full-span geometry is not established".into());
        }
        println!("PASS right edge below platform boundary 1472; full landing acceptance remains open");
        Ok(())
    }

    #[cfg_attr(test, test)]
    pub(super) fn author41_loader_refuses_changed_hash_and_wram_extent() {
        let original = PathBuf::from(std::env::var_os("SM_AUTHOR41_EVIDENCE_ROOT").expect("author41 root"));
        author41_result_at(&original).unwrap();
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let scratch = std::env::temp_dir().join(format!("sm-author41-loader-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&scratch).unwrap();
        let outcome = std::panic::catch_unwind(|| {
            let evidence = sm_binary_evidence::read_bytes(original.join("result.sme")).unwrap();
            let file = std::fs::File::open(original.join("final.wram")).unwrap();
            assert_eq!(file.metadata().unwrap().len(), 131072);
            let mut ram = Vec::new();
            file.take(131073).read_to_end(&mut ram).unwrap();
            assert_eq!(ram.len(), 131072);
            assert_eq!(
                format!("{:x}", Sha256::digest(&evidence)),
                "82ad320fa389841ee148e0dcfde1624b2d0f82a2074d20e5346ab6c412425e43"
            );
            assert_eq!(
                format!("{:x}", Sha256::digest(&ram)),
                "0c6ccc844b22e640f0fc718b42ac0b80381aec56829a59b96be48eb30e2ad668"
            );
            std::fs::write(scratch.join("result.sme"), &evidence).unwrap();
            std::fs::write(scratch.join("final.wram"), &ram).unwrap();
            author41_result_at(&scratch).unwrap();
            let mut changed = evidence.clone();
            changed[0] ^= 1;
            std::fs::write(scratch.join("result.sme"), changed).unwrap();
            assert_eq!(
                author41_result_at(&scratch).unwrap_err(),
                "retained author41 result.sme differs from exact input"
            );
            std::fs::write(scratch.join("result.sme"), &evidence).unwrap();
            let mut changed = ram.clone();
            changed[0xaf6] ^= 1;
            std::fs::write(scratch.join("final.wram"), changed).unwrap();
            assert_eq!(
                author41_result_at(&scratch).unwrap_err(),
                "retained author41 final.wram differs from exact input"
            );
            for extent in [131071, 131073] {
                std::fs::File::create(scratch.join("final.wram")).unwrap().set_len(extent).unwrap();
                assert_eq!(author41_result_at(&scratch).unwrap_err(), "unexpected WRAM extent");
            }
        });
        std::fs::remove_dir_all(&scratch).expect("remove author41 loader control files");
        if let Err(payload) = outcome {
            std::panic::resume_unwind(payload);
        }
    }
}
