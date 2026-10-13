//! Bounded Rust orchestration for this successor's preparation, seals and cleanup.
use std::{env, fs, path::{Path, PathBuf}, process::Command};

const ROOT: &str = "/home/jim/sm-decomp";
const PREDECESSOR: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-recovery-20261013";
const PACKAGE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-retry-20261013";
const SUBJECT: &str = "/tmp/sm-decomp-preparation-retry-source-20261013";
const ROOT_ID: &str = "01a13500-1ef8-73d3-9a0d-3446b6050eba";
const PREDECESSOR_SHA: &str = "96c408f3e5422c1cd5db953f669490c2b23026d558667ce11745a3702fc143f1";
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
        + &evidence(&format!("{PACKAGE}/process-inspection-current.toml"));
    for (name, id, status) in [
        ("replacement-live-observation.toml", ROOT_ID, "running"),
        ("previous-preparation-observation.toml", "01a134cd-dafb-7e50-8284-f285a905e05b", "failed"),
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
        .replace("/tmp/sm-decomp-preparation-final-source-20261013", SUBJECT)
        .replace("concurrency-drill-final", "concurrency-drill-retry")
        .replace(&hash(format!("{PREDECESSOR}/integrator-registration-observation.toml")), &hash(format!("{PACKAGE}/integrator-registration-observation.toml")));
    write("prepare-request.toml", request);
    fs::create_dir_all(Path::new(PACKAGE).join("bin")).unwrap();
    fs::copy(target.join("debug/decomp-readiness"), Path::new(PACKAGE).join("bin/decomp-readiness")).unwrap();
    let compiler = Command::new("rustc").arg("--version").output().unwrap();
    let build = format!("schema = \"decomp-readiness-helper-build-v1\"\ncompiler = \"{}\"\nsource_path = \"{PACKAGE}/main.rs\"\nsource_sha256 = \"{}\"\nmanifest_path = \"{PACKAGE}/Cargo.toml\"\nmanifest_sha256 = \"{}\"\nlock_path = \"{PACKAGE}/Cargo.lock\"\nlock_sha256 = \"{}\"\nbinary_path = \"{PACKAGE}/bin/decomp-readiness\"\nbinary_sha256 = \"{}\"\nbinary_bytes = {}\nprocess_inspector_source_path = \"{PACKAGE}/process-inspect.rs\"\nprocess_inspector_source_sha256 = \"{}\"\nprocess_inspector_binary_sha256 = \"{}\"\nprocess_inspector_binary_bytes = {}\nbuild_argv = [\"cargo\", \"build\", \"--locked\", \"--offline\", \"--manifest-path\", \"{PACKAGE}/Cargo.toml\", \"--target-dir\", \"{PACKAGE}/target\"]\nprospective_subject = \"{SUBJECT}\"\nactual_caller_required = \"{ROOT_ID}\"\nlimitations = [\"Local compiler/source/hash provenance; no platform attestation\", \"No actual readiness validation has run\", \"Helper changes fix successor subject/output paths; process inspector adds new drill paths and omits unrelated generic main-cwd processes\", \"Engine commit/fingerprint/build remain pending root exact frozen identity\"]\n", String::from_utf8(compiler.stdout).unwrap().trim(), hash(format!("{PACKAGE}/main.rs")), hash(format!("{PACKAGE}/Cargo.toml")), hash(format!("{PACKAGE}/Cargo.lock")), hash(format!("{PACKAGE}/bin/decomp-readiness")), fs::metadata(format!("{PACKAGE}/bin/decomp-readiness")).unwrap().len(), hash(format!("{PACKAGE}/process-inspect.rs")), hash(target.join("debug/decomp-readiness-process-inspect")), fs::metadata(target.join("debug/decomp-readiness-process-inspect")).unwrap().len());
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
    let mut files = Vec::new(); visit(Path::new(PACKAGE), &mut files); files.sort();
    write("artifacts.sha256", files.iter().map(|path| format!("{}  {}\n", hash(path), path.display())).collect::<String>());
}
fn cleanup() {
    let engine = Path::new(ROOT).join(".scratch/decomp-parallel/drill-final/bin/sm-pipeline");
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
fn main() {
    match env::args().nth(1).as_deref() {
        Some("prepare") => prepare(),
        Some("seal") => seal(),
        Some("cleanup") => cleanup(),
        _ => panic!("usage: package-readiness prepare|seal|cleanup"),
    }
}
