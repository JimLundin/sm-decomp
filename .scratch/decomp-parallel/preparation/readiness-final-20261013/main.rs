//! Production preflight in a private mount namespace. Host authority stays absent.
use std::{
    collections::BTreeMap,
    env, fs,
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

const ROOT: &str = "/home/jim/sm-decomp";
const SUBJECT: &str = "/tmp/sm-decomp-preparation-retry-source-20261013";
const INTEGRATOR: &str = "01a13554-bc0d-7662-9378-28eecfa6122f";
const PARENT: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel";
const READINESS: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013";
const BRIDGE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml";
const BRIDGE_SHA: &str = "cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9";
const LEGACY_LEDGER: &str = "/home/jim/sm-decomp/.scratch/ceres-continuation/ledger.toml";
const LEGACY_SHA: &str = "77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89";
const SOURCE_COMMIT: &str = "57f93c5ce4e6adac2dfe181b852d4aad6e0776bd";
const SOURCE_FINGERPRINT: &str = "bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec";
const ENGINE_SHA: &str = "04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573";

fn ensure(condition: bool, message: &str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message.into()) }
}
fn q(value: &str) -> String {
    // TOML basic strings; argv goes directly to Command, never through a shell.
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r"))
}
fn hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}
fn write(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|error| format!("{}: {error}", path.display()))
}
fn hash(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum").arg(path).output().map_err(|error| error.to_string())?;
    ensure(output.status.success(), "SHA-256 inspection failed")?;
    let result = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let result = result.split_whitespace().next().ok_or("SHA-256 output is empty")?.to_owned();
    ensure(hex(&result, 64), "SHA-256 output is malformed")?;
    Ok(result)
}
fn head(path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .current_dir(path)
        .output()
        .map_err(|error| error.to_string())?;
    ensure(output.status.success(), "source Git HEAD inspection failed")?;
    Ok(String::from_utf8(output.stdout).map_err(|error| error.to_string())?.trim().into())
}
fn absent_policy() -> Result<(), String> {
    for name in ["authority.toml", "ledger.toml"] {
        let path = Path::new(PARENT).join(name);
        ensure(
            matches!(fs::symlink_metadata(&path), Err(error) if error.kind() == io::ErrorKind::NotFound),
            "host canonical authority/ledger must be absent, including symlinks",
        )?;
    }
    Ok(())
}
fn toml_string_field(value: &str, key: &str) -> Result<String, String> {
    value
        .lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(name, field)| {
            (name.trim() == key).then(|| field.trim().strip_prefix('"')?.strip_suffix('"').map(str::to_owned)).flatten()
        })
        .ok_or_else(|| format!("build metadata lacks {key}"))
}
fn retains_evidence(value: &str, path: &Path, sha256: &str) -> bool {
    value.split("\n[[evidence]]\n").skip(1).any(|section| {
        let fields = section.split("\n[").next().unwrap_or(section);
        toml_string_field(fields, "path").ok().as_deref() == Some(path.to_string_lossy().as_ref())
            && toml_string_field(fields, "sha256").ok().as_deref() == Some(sha256)
    })
}
fn ledgers() -> Result<BTreeMap<PathBuf, String>, String> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(Path::new(ROOT).join(".scratch")).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path().join("ledger.toml");
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
            Ok(metadata) => {
                ensure(!metadata.file_type().is_symlink() && metadata.is_file(), "ledger must be a regular file")?;
                result.insert(path.clone(), hash(&path)?);
            }
        }
    }
    Ok(result)
}
fn seals(paths: &BTreeMap<PathBuf, String>) -> String {
    paths.iter().map(|(path, digest)| format!("{digest}  {}\n", path.display())).collect()
}
fn run(binary: &Path, request: &Path, output: &Path, name: &str) -> Result<(), String> {
    let child = Command::new(binary)
        .args(["decomp-authority", "--request"])
        .arg(request)
        .current_dir(SUBJECT)
        .stdout(File::create(output.join(format!("{name}.stdout.toml"))).map_err(|error| error.to_string())?)
        .stderr(File::create(output.join(format!("{name}.stderr.txt"))).map_err(|error| error.to_string())?)
        .status()
        .map_err(|error| error.to_string())?;
    write(&output.join(format!("{name}.exit.txt")), &format!("{}\n", child.code().unwrap_or(-1)))?;
    ensure(child.success(), &format!("{name} rejected; retained stderr explains why"))?;
    let receipt = text(&output.join(format!("{name}.stdout.toml")))?;
    ensure(receipt.lines().any(|line| line == "status = \"prepared\""), "preflight receipt must remain prepared")
}

fn inside(args: &[String]) -> Result<(), String> {
    ensure(args.len() == 4, "invalid private-namespace invocation")?;
    let original_namespace =
        env::var("SM_READINESS_OUTER_MOUNT_NS").map_err(|_| "private mount namespace proof is missing")?;
    let current_namespace = fs::read_link("/proc/self/ns/mnt").map_err(|error| error.to_string())?;
    ensure(
        current_namespace.to_string_lossy() != original_namespace,
        "refusing preparation outside private mount namespace",
    )?;
    let mountinfo = text(Path::new("/proc/self/mountinfo"))?;
    ensure(
        mountinfo.lines().any(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            fields.get(4) == Some(&PARENT) && line.split(" - ").nth(1).is_some_and(|tail| tail.starts_with("tmpfs "))
        }),
        "authority parent must be a private tmpfs mount",
    )?;
    ensure(env::var("CODEX_THREAD_ID").ok().as_deref() == Some(INTEGRATOR), "calling real integrator session differs")?;
    let binary = Path::new(&args[0]);
    let preparation = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    let validation = Path::new(&args[3]);
    run(binary, preparation, output, "prepare")?;
    let policy_path = Path::new(PARENT).join("authority.toml");
    let policy = fs::read(&policy_path).map_err(|error| error.to_string())?;
    fs::write(output.join("prepared-policy.toml"), &policy).map_err(|error| error.to_string())?;
    run(binary, validation, output, "validate-activation")?;
    ensure(
        fs::read(&policy_path).map_err(|error| error.to_string())? == policy,
        "validate-activation changed the prepared namespace policy",
    )?;
    let status = output.join("status-request.toml");
    write(&status, &format!("operation = \"status\"\ncheckout = {}\n", q(SUBJECT)))?;
    run(binary, &status, output, "status")?;
    ensure(
        !Path::new(PARENT).join("ledger.toml").exists(),
        "canonical ledger unexpectedly created in private namespace",
    )?;
    write(
        &output.join("namespace-completion.toml"),
        "schema = \"decomp-readiness-namespace-v1\"\nstatus = \"complete\"\npolicy_unchanged_by_validation = true\npolicy_state = \"prepared\"\ncanonical_ledger_present = false\nactivation_performed = false\n",
    )
}

fn outer(args: &[String]) -> Result<(), String> {
    ensure(
        args.len() == 18,
        "usage: readiness --engine PATH --source-checkout PATH --source-commit FULL --source-fingerprint SHA256 --build PATH --prepare-request PATH --drill PATH --post-publication-proof PATH --output NEW_DIRECTORY",
    )?;
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        ensure(options.insert(pair[0].as_str(), pair[1].as_str()).is_none(), "duplicate option")?;
    }
    for name in [
        "--engine",
        "--source-checkout",
        "--source-commit",
        "--source-fingerprint",
        "--build",
        "--prepare-request",
        "--drill",
        "--post-publication-proof",
        "--output",
    ] {
        ensure(options.contains_key(name), "missing or unknown option")?;
    }
    ensure(
        env::var("CODEX_THREAD_ID").ok().as_deref() == Some(INTEGRATOR),
        "run from the real root integrator session; do not override CODEX_THREAD_ID",
    )?;
    ensure(
        hex(options["--source-commit"], 40) && hex(options["--source-fingerprint"], 64),
        "engine version metadata is malformed",
    )?;
    ensure(options["--source-checkout"] == SUBJECT, "the reviewed frozen subject checkout is required")?;
    ensure(
        options["--source-commit"] == SOURCE_COMMIT && options["--source-fingerprint"] == SOURCE_FINGERPRINT,
        "the reviewed source commit and fingerprint are required",
    )?;
    let mut paths = BTreeMap::new();
    for key in ["--engine", "--source-checkout", "--build", "--prepare-request", "--drill", "--post-publication-proof"] {
        let raw = Path::new(options[key]);
        ensure(raw.is_absolute(), "all paths must be absolute")?;
        paths.insert(key, fs::canonicalize(raw).map_err(|error| error.to_string())?);
    }
    let output = PathBuf::from(options["--output"]);
    ensure(
        output.is_absolute() && output.starts_with(READINESS) && !output.exists(),
        "output must be a new child directory under owned readiness",
    )?;
    // Hold the real lock without writing bytes. Inside gets a distinct lock inode.
    let host_lock = OpenOptions::new()
        .write(true)
        .open(Path::new(ROOT).join(".scratch/assignment.lock"))
        .map_err(|error| error.to_string())?;
    host_lock.lock().map_err(|error| error.to_string())?;
    absent_policy()?;
    ensure(head(Path::new(SUBJECT))? == options["--source-commit"], "frozen engine source HEAD changed")?;
    ensure(
        head(&paths["--source-checkout"])? == options["--source-commit"],
        "engine source HEAD differs from recorded version",
    )?;
    ensure(
        hash(Path::new(BRIDGE))? == BRIDGE_SHA && hash(Path::new(LEGACY_LEDGER))? == LEGACY_SHA,
        "subject engine or retained legacy seals changed",
    )?;
    let before = ledgers()?;
    fs::create_dir(&output).map_err(|error| error.to_string())?;
    let output = fs::canonicalize(output).map_err(|error| error.to_string())?;
    ensure(
        output.starts_with(fs::canonicalize(READINESS).map_err(|error| error.to_string())?),
        "output path escaped readiness",
    )?;
    write(&output.join("host-ledgers-before.sha256"), &seals(&before))?;
    let engine_sha = hash(&paths["--engine"])?;
    ensure(engine_sha == ENGINE_SHA, "the reviewed frozen engine SHA-256 is required")?;
    let engine_build = text(&paths["--build"])?;
    ensure(
        toml_string_field(&engine_build, "source_checkout")? == paths["--source-checkout"].display().to_string()
            && toml_string_field(&engine_build, "source_commit")? == options["--source-commit"]
            && toml_string_field(&engine_build, "source_fingerprint")? == options["--source-fingerprint"]
            && toml_string_field(&engine_build, "binary_path")? == paths["--engine"].display().to_string()
            && toml_string_field(&engine_build, "binary_sha256")? == engine_sha,
        "engine source/binary arguments differ from retained build provenance",
    )?;
    let drill_sha = hash(&paths["--drill"])?;
    let post_publication_sha = hash(&paths["--post-publication-proof"])?;
    let drill = text(&paths["--drill"])?;
    ensure(
        retains_evidence(&drill, &paths["--post-publication-proof"], &post_publication_sha),
        "the final drill must directly retain the exact post-publication interruption proof",
    )?;
    write(
        &output.join("final-drill-binding.toml"),
        &format!(
            "schema = \"decomp-readiness-final-drill-binding-v1\"\ndrill = {}\ndrill_sha256 = {}\npost_publication_proof = {}\npost_publication_proof_sha256 = {}\nbound_at_actual_root_execution = true\n",
            q(&paths["--drill"].display().to_string()), q(&drill_sha),
            q(&paths["--post-publication-proof"].display().to_string()), q(&post_publication_sha),
        ),
    )?;
    let validation = output.join("validate-activation-request.toml");
    write(
        &validation,
        &format!(
            "operation = \"validate-activation\"\ncheckout = {}\ncoordinator = \"ceres-20261013\"\n[authority]\nsession_id = {}\ngeneration = 1\n[drill]\npath = {}\nsha256 = {}\n",
            q(SUBJECT),
            q(INTEGRATOR),
            q(&paths["--drill"].display().to_string()),
            q(&drill_sha)
        ),
    )?;
    let version = format!(
        "schema = \"decomp-readiness-version-map-v1\"\nsubject_engine_checkout = {}\nsubject_engine_commit = {}\nsubject_engine_fingerprint_expected = {}\nsubject_engine_binary = {}\nsubject_engine_binary_sha256 = {}\ninspection_source_checkout = {}\ninspection_source_commit = {}\ninspection_source_fingerprint_recorded = {}\ninspection_binary = {}\ninspection_binary_sha256 = {}\nengine_build = {}\nengine_build_sha256 = {}\ndrill = {}\ndrill_sha256 = {}\nprepare_request = {}\nprepare_request_sha256 = {}\nvalidator_authorship_is_separate_from_executed_engine = false\n",
        q(SUBJECT),
        q(options["--source-commit"]),
        q(options["--source-fingerprint"]),
        q(&paths["--engine"].display().to_string()),
        q(&engine_sha),
        q(&paths["--source-checkout"].display().to_string()),
        q(options["--source-commit"]),
        q(options["--source-fingerprint"]),
        q(&paths["--engine"].display().to_string()),
        q(&engine_sha),
        q(&paths["--build"].display().to_string()),
        q(&hash(&paths["--build"])?),
        q(&paths["--drill"].display().to_string()),
        q(&drill_sha),
        q(&paths["--prepare-request"].display().to_string()),
        q(&hash(&paths["--prepare-request"])?)
    );
    write(&output.join("source-version-map.toml"), &version)?;
    let backing_lock = output.join("namespace-assignment.lock");
    File::create(&backing_lock).map_err(|error| error.to_string())?;
    let namespace = fs::read_link("/proc/self/ns/mnt").map_err(|error| error.to_string())?.display().to_string();
    let mut command = Command::new("bwrap");
    command.args(["--unshare-user", "--die-with-parent", "--ro-bind", "/", "/", "--tmpfs", PARENT]);
    for entry in fs::read_dir(PARENT).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        command.arg("--ro-bind").arg(&path).arg(&path);
    }
    command
        .arg("--bind")
        .arg(&output)
        .arg(&output)
        .arg("--bind")
        .arg(&backing_lock)
        .arg(Path::new(ROOT).join(".scratch/assignment.lock"));
    command.args(["--setenv", "SM_READINESS_OUTER_MOUNT_NS", &namespace, "--chdir", SUBJECT, "--"]);
    command
        .arg(env::current_exe().map_err(|error| error.to_string())?)
        .arg("--inside")
        .arg(&paths["--engine"])
        .arg(&paths["--prepare-request"])
        .arg(&output)
        .arg(&validation);
    let argv: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
    write(
        &output.join("namespace-command.toml"),
        &format!("program = \"bwrap\"\nargv = [{}]\n", argv.iter().map(|arg| q(arg)).collect::<Vec<_>>().join(", ")),
    )?;
    let child = command
        .stdout(File::create(output.join("namespace.stdout.txt")).map_err(|error| error.to_string())?)
        .stderr(File::create(output.join("namespace.stderr.txt")).map_err(|error| error.to_string())?)
        .status()
        .map_err(|error| error.to_string())?;
    // Even rejected evidence gets host preservation checks and retained diagnostics.
    absent_policy()?;
    let after = ledgers()?;
    write(&output.join("host-ledgers-after.sha256"), &seals(&after))?;
    ensure(
        before == after
            && hash(Path::new(BRIDGE))? == BRIDGE_SHA
            && head(Path::new(SUBJECT))? == options["--source-commit"]
            && hash(&paths["--engine"])? == engine_sha
            && hash(&paths["--drill"])? == drill_sha,
        "host claims, frozen source, engine or drill changed during preflight",
    )?;
    ensure(
        hash(&paths["--post-publication-proof"])? == post_publication_sha,
        "post-publication interruption proof changed during preflight",
    )?;
    write(
        &output.join("host-preservation.toml"),
        &format!(
            "schema = \"decomp-readiness-host-preservation-v1\"\nstatus = \"complete\"\nauthority_present = false\ncanonical_ledger_present = false\nall_existing_ledger_hashes_unchanged = true\nlegacy_bridge_unchanged = true\nfrozen_engine_head_unchanged = true\nnamespace_exit_code = {}\nactivation_performed = false\n",
            child.code().unwrap_or(-1)
        ),
    )?;
    let mut artifacts = BTreeMap::new();
    for entry in fs::read_dir(&output).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_file() {
            artifacts.insert(path.clone(), hash(&path)?);
        }
    }
    write(&output.join("artifacts.sha256"), &seals(&artifacts))?;
    ensure(
        child.success(),
        "private namespace validation rejected; host policy and claims remain unchanged; see retained namespace/operation diagnostics",
    )?;
    println!(
        "Readiness validated in a private namespace; host authority and canonical ledger remain absent. Receipts: {}",
        output.display()
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let result = if args.first().map(String::as_str) == Some("--inside") { inside(&args[1..]) } else { outer(&args) };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::retains_evidence;
    use std::path::Path;

    #[test]
    fn final_proof_must_be_the_same_sealed_evidence_entry() {
        let proof = Path::new("/retained/post-publication-interruption.toml");
        let exact = "schema = \"drill\"\n\n[[evidence]]\npath = \"/retained/post-publication-interruption.toml\"\nsha256 = \"proof-hash\"\n\n[execution]\npath = \"/retained/execution.toml\"\nsha256 = \"execution-hash\"\n";
        assert!(retains_evidence(exact, proof, "proof-hash"));
        assert!(!retains_evidence(exact, proof, "execution-hash"));
        let split = "\n[[evidence]]\npath = \"/retained/post-publication-interruption.toml\"\nsha256 = \"wrong\"\n\n[[evidence]]\npath = \"/different\"\nsha256 = \"proof-hash\"\n";
        assert!(!retains_evidence(split, proof, "proof-hash"));
        assert!(!retains_evidence(exact, Path::new("/different"), "proof-hash"));
    }
}
