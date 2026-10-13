use std::{env, fs, path::{Path, PathBuf}, process::Command};

const OLD: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013";
const NEW: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013";
const ROOT_ID: &str = "01a13559-e517-72d1-88b5-e554fed61da4";
const OLD_ID: &str = "01a13554-bc0d-7662-9378-28eecfa6122f";
const OLD_SEAL: &str = "8dc634efb7b35ba6f5de9d4eb39252f3e75b22ff9c64ac273133a29e9df45b97";
const ROOT_OBSERVATION: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/final-recovery-20261013/current-root-inspection.json";
const ENGINE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/bin/sm-pipeline";
const BRIDGE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml";
const LEDGER: &str = "/home/jim/sm-decomp/.scratch/ceres-continuation/ledger.toml";

fn hash(path: impl AsRef<Path>) -> String {
    let output = Command::new("sha256sum").arg(path.as_ref()).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}
fn write(name: &str, value: impl AsRef<[u8]>) { fs::write(Path::new(NEW).join(name), value).unwrap(); }
fn read(name: &str) -> String { fs::read_to_string(Path::new(OLD).join(name)).unwrap() }
fn evidence(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    format!("\n[[evidence]]\npath = \"{}\"\nsha256 = \"{}\"\n", path.display(), hash(path))
}
fn prepare() {
    assert_eq!(hash(format!("{OLD}/artifacts.sha256")), OLD_SEAL);
    let result = Command::new("sha256sum").args(["-c", "artifacts.sha256"]).current_dir(OLD).output().unwrap();
    write("predecessor-authentication.stdout", result.stdout);
    write("predecessor-authentication.stderr", result.stderr);
    assert!(result.status.success());
    assert!(!Path::new(OLD).join("production-validation-1").exists());
    for name in ["authority.toml", "ledger.toml"] {
        assert!(fs::symlink_metadata(Path::new("/home/jim/sm-decomp/.scratch/decomp-parallel").join(name)).is_err());
    }
    assert_eq!(hash(BRIDGE), "cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9");
    assert_eq!(hash(LEDGER), "77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89");
    fs::copy(ROOT_OBSERVATION, Path::new(NEW).join("current-root-inspection.json")).unwrap();
    let result = Command::new(format!("{OLD}/bin/decomp-readiness-process-inspect")).output().unwrap();
    assert!(result.status.success());
    write("process-inspection-current.toml", &result.stdout);
    let process = String::from_utf8(result.stdout).unwrap();
    assert!(!process.lines().any(|line| line == "claim_path_match = true"), "retained Ceres writer requires inspection");
    let observed = process.lines().find_map(|line| line.strip_prefix("observed_at = ")).unwrap();
    let common = evidence(format!("{NEW}/current-root-inspection.json"))
        + &evidence(format!("{NEW}/process-inspection-current.toml"))
        + &evidence(format!("{OLD}/artifacts.sha256"));
    write("predecessor.toml", format!("schema = \"decomp-readiness-predecessor-v1\"\npath = \"{OLD}/artifacts.sha256\"\nsha256 = \"{OLD_SEAL}\"\nvalidation_output_absent = true\nprevious_root = \"{OLD_ID}\"\nprevious_root_turn_status = \"failed\"\nprevious_root_tool_status = \"notLoaded\"\nlimitations = [\"Old package and all prior controls stay immutable; current root active/inProgress and predecessor failed turn come from the exact retained tool observation\", \"No readiness execution is inferred from an absent documented output\"]\n{common}"));
    write("replacement-live-observation.toml", format!("schema = \"decomp-session-observation-v1\"\nsession_id = \"{ROOT_ID}\"\nstatus = \"running\"\nobserved_at = {observed}\n{common}"));
    write("ownership-boundaries.toml", format!("schema = \"decomp-readiness-ownership-boundaries-v1\"\nobserved_at = {observed}\nobserver_thread = \"{}\"\nprevious_preparation_thread = \"{OLD_ID}\"\nprevious_preparation_status = \"failed\"\nreplacement_integrator_thread = \"{ROOT_ID}\"\nreplacement_integrator_status = \"running\"\nreplacement_integrator_generation = 1\nmain_git_owner_thread = \"{ROOT_ID}\"\nclaim_processes_observed = []\noriginal_claims_released = false\ngameplay_launched = false\nauthority_present = false\ncanonical_ledger_present = false\nlimitations = [\"Generation one is prospective and only acquired in the private namespace\", \"Retained legacy child observations remain unchanged; submitted jobs and running integration stay held\", \"No matching Ceres process was observed; absence does not release ownership\", \"Exact thread/process observations are a tool trust boundary, not platform attestation\"]\n{common}{}{}", env::var("CODEX_THREAD_ID").unwrap(), evidence(BRIDGE), evidence(LEDGER)));
    let old_transfer = read("legacy-generation-zero-transfer.toml");
    let children = old_transfer.split("\n[[children]]").skip(1).map(|part| format!("\n[[children]]{part}")).collect::<String>();
    let transfer = format!("schema = \"decomp-session-transfer-v1\"\ntoken = \"ceres-20261013\"\nsession_id = \"01a133eb-76e2-7073-9aef-d97c03a812e4\"\ngeneration = 0\nstatus = \"failed\"\nobserved_at = {observed}\nprocesses = []\n{common}{}{}{}\n[replacement]\npath = \"{NEW}/replacement-live-observation.toml\"\nsha256 = \"{}\"\n{children}", evidence(format!("{OLD}/legacy-generation-zero-transfer.toml")), evidence(format!("{NEW}/ownership-boundaries.toml")), evidence(LEDGER), hash(format!("{NEW}/replacement-live-observation.toml")));
    write("legacy-generation-zero-transfer.toml", transfer);
    write("integrator-registration-observation.toml", format!("schema = \"decomp-session-observation-v1\"\nsession_id = \"{ROOT_ID}\"\nstatus = \"running\"\nobserved_at = {observed}\n{common}{}", evidence(format!("{NEW}/legacy-generation-zero-transfer.toml"))));
    write("prepare-request.toml", read("prepare-request.toml").replace(OLD_ID, ROOT_ID).replace(&format!("{OLD}/integrator-registration-observation.toml"), &format!("{NEW}/integrator-registration-observation.toml")).replace(&hash(format!("{OLD}/integrator-registration-observation.toml")), &hash(format!("{NEW}/integrator-registration-observation.toml"))));
    write("main.rs", read("main.rs").replace(&format!("const INTEGRATOR: &str = \"{OLD_ID}\";"), &format!("const INTEGRATOR: &str = \"{ROOT_ID}\";")).replace(&format!("const READINESS: &str = \"{OLD}\";"), &format!("const READINESS: &str = \"{NEW}\";")));
    write("Cargo.toml", "[package]\nname = \"decomp-readiness-current-root\"\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n\n[workspace]\n\n[[bin]]\nname = \"decomp-readiness\"\npath = \"main.rs\"\n");
    write("Cargo.lock", "# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\nversion = 4\n\n[[package]]\nname = \"decomp-readiness-current-root\"\nversion = \"0.0.0\"\n");
}
fn finalize() {
    fs::create_dir(Path::new(NEW).join("bin")).unwrap();
    fs::copy(Path::new(NEW).join("target/debug/decomp-readiness"), Path::new(NEW).join("bin/decomp-readiness")).unwrap();
    write("helper-build.toml", format!("schema = \"decomp-readiness-helper-build-v1\"\nsource_path = \"{NEW}/main.rs\"\nsource_sha256 = \"{}\"\nmanifest_sha256 = \"{}\"\nlock_sha256 = \"{}\"\nbinary_path = \"{NEW}/bin/decomp-readiness\"\nbinary_sha256 = \"{}\"\nactual_caller_required = \"{ROOT_ID}\"\npredecessor_controls_retained = true\nnamespace_validation_run = false\nlimitations = [\"Only required caller and owned output root constants differ from predecessor main.rs; completed controls were not rerun\", \"Frozen source/engine and direct sealed post-publication proof requirements remain identical\"]\n{}", hash(format!("{NEW}/main.rs")), hash(format!("{NEW}/Cargo.toml")), hash(format!("{NEW}/Cargo.lock")), hash(format!("{NEW}/bin/decomp-readiness")), evidence(format!("{OLD}/helper-controls.toml"))));
    let command = format!("{NEW}/bin/decomp-readiness \\\n+  --engine {ENGINE} \\\n+  --source-checkout /tmp/sm-decomp-preparation-retry-source-20261013 \\\n+  --source-commit 57f93c5ce4e6adac2dfe181b852d4aad6e0776bd \\\n+  --source-fingerprint bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec \\\n+  --build /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/engine-build.toml \\\n+  --prepare-request {NEW}/prepare-request.toml \\\n+  --drill /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/concurrency-drill.toml \\\n+  --post-publication-proof /home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/post-publication-interruption.toml \\\n+  --output {NEW}/production-validation-1\n");
    write("run-command.sh", &command);
    write("README.md", format!("# Current-root readiness successor\n\nRequired actual caller: `{ROOT_ID}`. Execute `run-command.sh` with inherited real CODEX_THREAD_ID after the final drill seal is stable. The predecessor manifest `{OLD_SEAL}` and all previous controls remain immutable; only caller/output-root constants changed in the Rust helper. Exact root active/inProgress and predecessor failed-turn observations are retained, as is fresh read-only process inspection. Host production authority and canonical ledger remain absent.\n\nAuthenticate `artifacts.sha256` before execution. The helper seals its new validation output. Root must run both project cleanup passes on that finished output and authenticate the output seal and retained inputs afterward. Prepared policy exists only inside the private namespace; gameplay launch remains deferred.\n"));
}
fn cleanup() {
    for (name, dedup) in [("cleanup1.toml", false), ("cleanup2.toml", true)] {
        let mut command = Command::new(ENGINE);
        command.args(["cleanup", "--root", NEW]);
        if dedup { command.arg("--deduplicate"); }
        let result = command.arg("--apply").output().unwrap();
        write(name, result.stdout); write(&format!("{name}.stderr"), result.stderr);
        assert!(result.status.success());
    }
    let measured = Command::new("du").args(["-sb", &format!("{NEW}/target")]).output().unwrap();
    assert!(measured.status.success());
    let measured = String::from_utf8(measured.stdout).unwrap().split_whitespace().next().unwrap().to_owned();
    let result = Command::new("cargo").args(["clean", "--manifest-path", &format!("{NEW}/Cargo.toml"), "--target-dir", &format!("{NEW}/target")]).output().unwrap();
    write("cargo-clean.stdout", result.stdout); write("cargo-clean.stderr", result.stderr); assert!(result.status.success());
    let space = Command::new("df").args(["-B1", "--output=avail", NEW]).output().unwrap();
    let available = String::from_utf8(space.stdout).unwrap().split_whitespace().last().unwrap().to_owned();
    write("cleanup.toml", format!("schema = \"decomp-readiness-package-cleanup-v1\"\nstatus = \"complete\"\nprivate_target_logical_bytes_before_clean = {measured}\navailable_filesystem_bytes = {available}\nhelper_binary_retained = true\nnamespace_validation_run = false\nlimitations = [\"Scoped utility cleanup and deduplication succeeded; cargo target removed after builder/compiler children exited; all predecessor and retained evidence dependencies remain\"]\n"));
}
fn visit(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "target" || entry.file_name() == "artifacts.sha256" || entry.file_name() == "successor" { continue; }
        let path = entry.path();
        if path.is_dir() { visit(&path, files); } else { files.push(path); }
    }
}
fn seal() {
    assert!(!Path::new(NEW).join("artifacts.sha256").exists());
    let mut files = Vec::new(); visit(Path::new(NEW), &mut files); files.sort();
    write("artifacts.sha256", files.iter().map(|path| format!("{}  {}\n", hash(path), path.display())).collect::<String>());
}
fn main() {
    assert!(!Path::new(NEW).join("artifacts.sha256").exists(), "sealed successor is immutable");
    match env::args().nth(1).as_deref() {
        Some("prepare") => prepare(), Some("finalize") => finalize(), Some("cleanup") => cleanup(), Some("seal") => seal(), _ => panic!("prepare, finalize, cleanup or seal required"),
    }
}
