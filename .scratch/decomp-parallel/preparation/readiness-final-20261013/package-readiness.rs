//! Bounded Rust orchestration for this successor's preparation, seals and cleanup.
use std::{env, fs, path::{Path, PathBuf}, process::Command};

const ROOT: &str = "/home/jim/sm-decomp";
const PREDECESSOR: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-retry-20261013";
const PACKAGE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013";
const SUBJECT: &str = "/tmp/sm-decomp-preparation-retry-source-20261013";
const ROOT_ID: &str = "01a13554-bc0d-7662-9378-28eecfa6122f";
const PREDECESSOR_SHA: &str = "76e5c8541c32af2c1cd0f00bf43d9ec8f2f92b05bb6d53692a369228b0b8cd6b";
const LEDGER: &str = "/home/jim/sm-decomp/.scratch/ceres-continuation/ledger.toml";
const LEDGER_SHA: &str = "77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89";
const BRIDGE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml";
const BRIDGE_SHA: &str = "cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9";

fn hash(path: impl AsRef<Path>) -> String {
    let result = Command::new("sha256sum").arg(path.as_ref()).output().unwrap();
    assert!(result.status.success(), "hash inspection failed");
    String::from_utf8(result.stdout).unwrap().split_whitespace().next().unwrap().into()
}
fn write(name: &str, body: impl AsRef<[u8]>) { fs::write(Path::new(PACKAGE).join(name), body).unwrap(); }
fn field(body: &str, key: &str) -> String {
    body.lines().find_map(|line| line.strip_prefix(&format!("{key} = \""))).unwrap().trim_end_matches('"').into()
}
fn evidence(path: &str) -> String { format!("\n[[evidence]]\npath = \"{path}\"\nsha256 = \"{}\"\n", hash(path)) }
fn prepare() {
    assert!(!Path::new(PACKAGE).join("artifacts.sha256").exists(), "sealed successor is immutable; create another successor for changed observations");
    assert_eq!(hash(format!("{PREDECESSOR}/artifacts.sha256")), PREDECESSOR_SHA);
    let output = Command::new("sha256sum").args(["-c", "artifacts.sha256"]).current_dir(PREDECESSOR).output().unwrap();
    write("predecessor-authentication.stdout", &output.stdout);
    write("predecessor-authentication.stderr", &output.stderr);
    assert!(output.status.success(), "predecessor authentication failed");
    assert_eq!(hash(LEDGER), LEDGER_SHA);
    assert_eq!(hash(BRIDGE), BRIDGE_SHA);
    for name in ["authority.toml", "ledger.toml"] {
        assert!(fs::symlink_metadata(Path::new(ROOT).join(".scratch/decomp-parallel").join(name)).is_err());
    }
    let target = Path::new(PACKAGE).join("target");
    let inspected = Command::new(target.join("debug/decomp-readiness-process-inspect")).output().unwrap();
    assert!(inspected.status.success());
    write("process-inspection-current.toml", &inspected.stdout);
    let process = String::from_utf8(inspected.stdout).unwrap();
    assert!(!process.lines().any(|line| line == "claim_path_match = true"), "inspect surviving legacy writers before preparing observations");
    let observed = field(&process, "observed_at");
    let observer = env::var("CODEX_THREAD_ID").unwrap();
    let common = evidence(&format!("{PACKAGE}/thread-inspection-current.json"))
        + &evidence(&format!("{PACKAGE}/process-inspection-current.toml"))
        + &evidence("/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/final-recovery-20261013/root-and-legacy-thread-inspection.json");
    assert!(!Path::new(PREDECESSOR).join("production-validation-1").exists(), "prior root may have executed readiness; inspect before replacing");
    write("prior-readiness-not-executed.toml", format!("schema = \"decomp-readiness-prior-execution-inspection-v1\"\nprevious_root = \"01a13500-1ef8-73d3-9a0d-3446b6050eba\"\nprevious_root_status = \"failed\"\nprevious_root_tool_status = \"systemError\"\nobserved_at = \"{observed}\"\noriginal_package_manifest_path = \"{PREDECESSOR}/artifacts.sha256\"\noriginal_package_manifest_sha256 = \"{PREDECESSOR_SHA}\"\nprevious_validation_output_absent = true\nprevious_validation_executed = false\nlimitations = [\"Exact documented output is absent and sealed package remains pending; no claim or generation release inferred\"]\n{common}"));
    for (name, id, status) in [
        ("replacement-live-observation.toml", ROOT_ID, "running"),
        ("previous-preparation-observation.toml", "01a13500-1ef8-73d3-9a0d-3446b6050eba", "failed"),
    ] {
        write(name, format!("schema = \"decomp-session-observation-v1\"\nsession_id = \"{id}\"\nstatus = \"{status}\"\nobserved_at = \"{observed}\"\n{common}"));
    }
    for name in ["compact-controller-observation.toml", "shared-game-host-observation.toml", "integrate-shared-game-host-observation.toml"] {
        let body = fs::read_to_string(Path::new(PREDECESSOR).join(name)).unwrap();
        let head = body.split("\n[[evidence]]").next().unwrap();
        let claim = body.split("\n[claim]\n").nth(1).unwrap();
        let head = head.lines().filter(|line| !line.starts_with("observed_at =")).collect::<Vec<_>>().join("\n");
        write(name, format!("{head}\nobserved_at = \"{observed}\"\n{common}\n[claim]\n{claim}"));
    }
    write("predecessor.toml", format!("schema = \"decomp-readiness-predecessor-v1\"\npath = \"{PREDECESSOR}/artifacts.sha256\"\nsha256 = \"{PREDECESSOR_SHA}\"\nlimitations = [\"Predecessor evidence is immutable and belongs to its original frozen source and drill epoch\"]\n"));
    let ownership = format!("schema = \"decomp-readiness-ownership-boundaries-v1\"\nobserved_at = \"{observed}\"\nobserver_thread = \"{observer}\"\nlegacy_integrator_thread = \"01a133eb-76e2-7073-9aef-d97c03a812e4\"\nlegacy_integrator_status = \"failed\"\nprevious_preparation_thread = \"01a134cd-dafb-7e50-8284-f285a905e05b\"\nprevious_preparation_status = \"failed\"\nreplacement_integrator_thread = \"{ROOT_ID}\"\nreplacement_integrator_status = \"running\"\nreplacement_integrator_generation = 1\nmain_git_owner_thread = \"{ROOT_ID}\"\nreporting_thread = \"01a13394-29b8-76a3-aab7-5ab8eff60694\"\nreporting_status = \"completed\"\nreporting_handoff_commit = \"9ed933daa151f52e3cf922e6559c7681330e0f85\"\nreporting_server_retained_pid = 420446\nreporting_server_start_ticks = 2249195\nreporting_server_boot_id = \"7119fcce-5593-4526-aa9b-07127767c4ef\"\nclaim_processes_observed = []\nclaim_states_unchanged = true\noriginal_claims_released = false\ngameplay_launched = false\nauthority_present = false\ncanonical_ledger_present = false\nlimitations = [\"Fresh process inspection found no retained Ceres path writer; submitted jobs and running integration remain held.\", \"Prospective generation1 transfer occurs only inside the private namespace; host production has never acquired it.\", \"Real thread/process observations remain a tool trust boundary, not platform attestation.\", \"Prior isolated-drill coordinator remains active; no ownership release is inferred from absence of matching processes.\"]\n{common}{}", evidence(&format!("{PREDECESSOR}/artifacts.sha256")));
    let ownership = ownership
        .replace("previous_preparation_thread = \"01a134cd-dafb-7e50-8284-f285a905e05b\"", "previous_preparation_thread = \"01a13500-1ef8-73d3-9a0d-3446b6050eba\"")
        .replace("Prior isolated-drill coordinator remains active; no ownership release is inferred from absence of matching processes.", "Prior isolated-drill coordinator and crashed retry harness are terminal; supplemental drill is monitored separately. No claim release is inferred.")
        + &evidence(&format!("{PACKAGE}/prior-readiness-not-executed.toml"));
    write("ownership-boundaries.toml", ownership);
    let old = fs::read_to_string(Path::new(PREDECESSOR).join("legacy-generation-zero-transfer.toml")).unwrap();
    let children = old.split("\n[[children]]").skip(1).map(|body| {
        let mut child = body.replace(PREDECESSOR, PACKAGE);
        for name in ["compact-controller-observation.toml", "shared-game-host-observation.toml", "integrate-shared-game-host-observation.toml"] {
            let old_hash = hash(Path::new(PREDECESSOR).join(name));
            child = child.replace(&old_hash, &hash(Path::new(PACKAGE).join(name)));
        }
        format!("\n[[children]]{child}")
    }).collect::<String>();
    let transfer = format!("schema = \"decomp-session-transfer-v1\"\ntoken = \"ceres-20261013\"\nsession_id = \"01a133eb-76e2-7073-9aef-d97c03a812e4\"\ngeneration = 0\nstatus = \"failed\"\nobserved_at = \"{observed}\"\nprocesses = []\n{common}{}{}{}{}\n[replacement]\npath = \"{PACKAGE}/replacement-live-observation.toml\"\nsha256 = \"{}\"\n{children}", evidence(&format!("{PACKAGE}/ownership-boundaries.toml")), evidence(LEDGER), evidence(BRIDGE), evidence(&format!("{PACKAGE}/previous-preparation-observation.toml")), hash(format!("{PACKAGE}/replacement-live-observation.toml")));
    write("legacy-generation-zero-transfer.toml", transfer);
    write("integrator-registration-observation.toml", format!("schema = \"decomp-session-observation-v1\"\nsession_id = \"{ROOT_ID}\"\nstatus = \"running\"\nobserved_at = \"{observed}\"\n{common}{}", evidence(&format!("{PACKAGE}/legacy-generation-zero-transfer.toml"))));
    let request = fs::read_to_string(Path::new(PREDECESSOR).join("prepare-request.toml")).unwrap()
        .replace(PREDECESSOR, PACKAGE)
        .replace("01a13500-1ef8-73d3-9a0d-3446b6050eba", ROOT_ID)
        .replace("/tmp/sm-decomp-preparation-final-source-20261013", SUBJECT)
        .replace("concurrency-drill-final", "concurrency-drill-retry")
        .replace(&hash(format!("{PREDECESSOR}/integrator-registration-observation.toml")), &hash(format!("{PACKAGE}/integrator-registration-observation.toml")));
    write("prepare-request.toml", request);
    fs::create_dir_all(Path::new(PACKAGE).join("bin")).unwrap();
    fs::copy(target.join("debug/decomp-readiness"), Path::new(PACKAGE).join("bin/decomp-readiness")).unwrap();
    fs::copy(target.join("debug/decomp-readiness-process-inspect"), Path::new(PACKAGE).join("bin/decomp-readiness-process-inspect")).unwrap();
    fs::copy(target.join("debug/package-readiness"), Path::new(PACKAGE).join("bin/package-readiness")).unwrap();
    let compiler = Command::new("rustc").arg("--version").output().unwrap();
    let build = format!("schema = \"decomp-readiness-helper-build-v1\"\ncompiler = \"{}\"\nsource_path = \"{PACKAGE}/main.rs\"\nsource_sha256 = \"{}\"\nmanifest_path = \"{PACKAGE}/Cargo.toml\"\nmanifest_sha256 = \"{}\"\nlock_path = \"{PACKAGE}/Cargo.lock\"\nlock_sha256 = \"{}\"\nbinary_path = \"{PACKAGE}/bin/decomp-readiness\"\nbinary_sha256 = \"{}\"\nbinary_bytes = {}\nprocess_inspector_source_path = \"{PACKAGE}/process-inspect.rs\"\nprocess_inspector_source_sha256 = \"{}\"\nprocess_inspector_binary_sha256 = \"{}\"\nprocess_inspector_binary_bytes = {}\nbuild_argv = [\"cargo\", \"build\", \"--locked\", \"--offline\", \"--manifest-path\", \"{PACKAGE}/Cargo.toml\", \"--target-dir\", \"{PACKAGE}/target\"]\nprospective_subject = \"{SUBJECT}\"\nactual_caller_required = \"{ROOT_ID}\"\nlimitations = [\"Local compiler/source/hash provenance; no platform attestation\", \"No actual readiness validation has run\", \"Helper changes fix successor subject/output paths; process inspector adds new drill paths and omits unrelated generic main-cwd processes\", \"Engine commit/fingerprint/build remain pending root exact frozen identity\"]\n", String::from_utf8(compiler.stdout).unwrap().trim(), hash(format!("{PACKAGE}/main.rs")), hash(format!("{PACKAGE}/Cargo.toml")), hash(format!("{PACKAGE}/Cargo.lock")), hash(format!("{PACKAGE}/bin/decomp-readiness")), fs::metadata(format!("{PACKAGE}/bin/decomp-readiness")).unwrap().len(), hash(format!("{PACKAGE}/process-inspect.rs")), hash(target.join("debug/decomp-readiness-process-inspect")), fs::metadata(target.join("debug/decomp-readiness-process-inspect")).unwrap().len());
    let build = build
        .replace("No actual readiness validation has run", "No actual readiness validation has run; exact final post-publication proof is bound by root invocation")
        .replace("Engine commit/fingerprint/build remain pending root exact frozen identity", "Exact frozen subject commit57f93c5ce4e6adac2dfe181b852d4aad6e0776bd and engine04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573 remain required")
        + &format!("packager_source_sha256 = \"{}\"\npackager_binary_sha256 = \"{}\"\n", hash(format!("{PACKAGE}/package-readiness.rs")), hash(format!("{PACKAGE}/bin/package-readiness")));
    write("helper-build.toml", build);
}
fn visit(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_name() == "target" || entry.file_name() == "artifacts.sha256" {continue;}
        if path.is_dir() { visit(&path, files); } else {files.push(path);}
    }
}
fn seal() {
    assert!(!Path::new(PACKAGE).join("artifacts.sha256").exists(), "sealed successor is immutable; do not overwrite the manifest");
    let mut files = Vec::new(); visit(Path::new(PACKAGE), &mut files); files.sort();
    write("artifacts.sha256", files.iter().map(|path| format!("{}  {}\n", hash(path), path.display())).collect::<String>());
}
fn cleanup() {
    let engine = Path::new(ROOT).join(".scratch/decomp-parallel/drill-retry/bin/sm-pipeline");
    for (name, dedup) in [("package-cleanup1.toml", false), ("package-cleanup2.toml", true)] {
        let mut cmd = Command::new(&engine);
        cmd.args(["cleanup", "--root", PACKAGE]);
        if dedup {cmd.arg("--deduplicate");}
        let result = cmd.arg("--apply").output().unwrap();
        write(name, result.stdout);
        write(&format!("{name}.stderr"), result.stderr);
        assert!(result.status.success(), "package cleanup failed");
    }
    let measured = Command::new("du").args(["-sb", &format!("{PACKAGE}/target")]).output().unwrap();
    assert!(measured.status.success());
    let bytes = String::from_utf8(measured.stdout).unwrap().split_whitespace().next().unwrap().parse::<u64>().unwrap();
    let clean = Command::new("cargo").args(["clean", "--manifest-path", &format!("{PACKAGE}/Cargo.toml"), "--target-dir", &format!("{PACKAGE}/target")]).output().unwrap();
    write("cargo-clean.stdout", clean.stdout);
    write("cargo-clean.stderr", clean.stderr);
    assert!(clean.status.success());
    let space = Command::new("df").args(["-B1", "--output=avail", PACKAGE]).output().unwrap();
    let available = String::from_utf8(space.stdout).unwrap().split_whitespace().last().unwrap().parse::<u64>().unwrap();
    write("package-cleanup.toml", format!("schema = \"decomp-readiness-package-cleanup-v1\"\nstatus = \"complete\"\nutility_passes = 2\nprivate_target_logical_bytes_before_clean = {bytes}\navailable_filesystem_bytes = {available}\nhelper_binary_retained = true\nnamespace_validation_run = false\nlimitations = [\"Both utility cleanup passes applied to this successor only; retained source/logs/seals/helper remain\", \"Private target removed through cargo clean after all build/process-inspection children finished\"]\n"));
}
fn controls() {
    assert!(!Path::new(PACKAGE).join("artifacts.sha256").exists(), "sealed successor is immutable");
    let helper = Path::new(PACKAGE).join("bin/decomp-readiness");
    assert_ne!(env::var("CODEX_THREAD_ID").unwrap(), ROOT_ID, "wrong-caller control must inherit the actual observer identity");
    let output = Path::new(PACKAGE).join("wrong-caller-output");
    assert!(!output.exists());
    let wrong = Command::new(&helper).args([
        "--engine", "/unavailable-engine", "--source-checkout", SUBJECT,
        "--source-commit", "57f93c5ce4e6adac2dfe181b852d4aad6e0776bd",
        "--source-fingerprint", "bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec",
        "--build", "/unavailable-build", "--prepare-request", "/unavailable-request",
        "--drill", "/unavailable-drill", "--post-publication-proof", "/unavailable-proof",
        "--output", output.to_str().unwrap(),
    ]).output().unwrap();
    write("wrong-caller.stdout", &wrong.stdout); write("wrong-caller.stderr", &wrong.stderr);
    assert_eq!(wrong.status.code(), Some(1));
    assert!(String::from_utf8(wrong.stderr).unwrap().contains("real root integrator session"));
    assert!(!output.exists());
    let inner = Command::new(&helper).args(["--inside", "/unavailable-engine", "/unavailable-prepare", "/unavailable-output", "/unavailable-validation"]).output().unwrap();
    write("direct-inner.stdout", &inner.stdout); write("direct-inner.stderr", &inner.stderr);
    assert_eq!(inner.status.code(), Some(1));
    assert!(String::from_utf8(inner.stderr).unwrap().contains("private mount namespace proof is missing"));
    write("helper-controls.toml", format!("schema = \"decomp-readiness-helper-controls-v1\"\nstatus = \"pass\"\nobserver_thread = \"{}\"\nwrong_caller_exit = 1\nwrong_caller_output_absent = true\ndirect_inner_exit = 1\nnamespace_executed = false\nidentity_override_performed = false\nfinal_proof_binding_unit_test_passed = true\nfinal_proof_binding_test_argv = [\"cargo\", \"test\", \"--locked\", \"--offline\", \"--manifest-path\", \"{PACKAGE}/Cargo.toml\", \"--target-dir\", \"{PACKAGE}/target\"]\nlimitations = [\"Unit regression rejects proof path/hash split across different evidence entries\", \"Actual readiness execution belongs to the root and remains pending\"]\n", env::var("CODEX_THREAD_ID").unwrap()));
}
fn main() {
    match env::args().nth(1).as_deref() {
        Some("prepare") => prepare(),
        Some("seal") => seal(),
        Some("cleanup") => cleanup(),
        Some("controls") => controls(),
        _ => panic!("usage: package-readiness prepare|seal|cleanup|controls"),
    }
}
