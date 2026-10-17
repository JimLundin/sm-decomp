//! Actual cached C build/replay preserves routine observations; invalid input stops before a build.
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn native_baseline_keeps_real_calls_and_missing_input_failure_evidence() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let output = std::env::temp_dir()
        .join(format!("sm-baseline-controls-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&output).unwrap();
    let call = |inputs: &Path, destination: &Path| {
        Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .current_dir(checkout)
            .args(["decomp-baseline", "--frames", "300", "--inputs"])
            .arg(inputs)
            .arg("--output")
            .arg(destination)
            .output()
            .unwrap()
    };
    let missing = output.join("missing");
    let result = call(&output.join("missing.inputs"), &missing);
    assert!(!result.status.success());
    let failed: Value = toml::from_str(std::str::from_utf8(&result.stdout).unwrap()).unwrap();
    assert_eq!(failed["status"].as_str(), Some("error"));
    assert!(failed["error"].as_str().unwrap().contains("missing.inputs"));
    assert!(failed.get("terminal").is_none());
    if !checkout.join("rom/sm.sfc").is_file()
        || !checkout.join("movies/super-metroid-any-3653.inputs").is_file()
        || !checkout.join("deps/snesrev-sm/src/sm_80.c").is_file()
    {
        eprintln!("real baseline control requires local ROM, movie and pinned C source");
        fs::remove_dir_all(output).unwrap();
        return;
    }
    let result = call(&checkout.join("movies/super-metroid-any-3653.inputs"), &output.join("good"));
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stderr),
        String::from_utf8_lossy(&result.stdout)
    );
    let verdict: Value = toml::from_str(std::str::from_utf8(&result.stdout).unwrap()).unwrap();
    assert_eq!(verdict["observation_controls"]["fallback_calls"].as_integer(), Some(1));
    assert!(verdict["observation_controls"]["fallback_instructions"].as_integer().unwrap() > 0);
    assert_eq!(verdict["observation_controls"]["decompression_calls"].as_integer(), Some(1));
    assert_eq!(verdict["original_rom_parity"].as_str(), Some("not-tested"));
    assert!(verdict["in_memory_rom_patches"]["changed_bytes"].as_integer().unwrap() > 0);
    let run = Path::new(verdict["evidence"].as_str().unwrap());
    let manifest: Value =
        toml::from_str(&fs::read_to_string(run.join("replay/fixtures/manifest.toml")).unwrap()).unwrap();
    let calls = manifest["calls"].as_array().unwrap();
    assert!(calls.len() > 1, "capture must append multiple calls under one binary header");
    for (sequence, call) in calls.iter().enumerate() {
        assert_eq!(call["sequence"].as_integer(), Some(sequence as i64 + 1));
        for phase in ["before", "after"] {
            let file = run.join("replay/fixtures").join(call[phase]["file"].as_str().unwrap());
            assert_eq!(file.metadata().unwrap().len(), 0x20000);
            assert_eq!(call[phase]["sha256"].as_str().unwrap().len(), 64);
        }
    }
    fs::remove_dir_all(output).unwrap();
}
