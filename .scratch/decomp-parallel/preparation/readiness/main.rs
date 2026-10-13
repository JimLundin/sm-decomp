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
const SUBJECT: &str = "/tmp/sm-decomp-preparation-integration-20261013";
const SUBJECT_HEAD: &str = "c8799f326ffe198b652301f08866f1b7a7bd9ebc";
const SUBJECT_FINGERPRINT: &str = "4d87e457e895a03f82f5a7ccdd891f7769b83d25d48caf2fd3a55432e5929338";
const SUBJECT_BINARY: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill/bin/sm-pipeline";
const SUBJECT_BINARY_SHA: &str = "89e3c46596ea556011cfcbb679cace5b96277bbf56abb8a7a2a61db7048268d9";
const INTEGRATOR: &str = "01a13460-4aa9-7fe0-865e-a8f54ffca310";
const PARENT: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel";
const READINESS: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness";
const BRIDGE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml";
const BRIDGE_SHA: &str = "cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9";
const LEGACY_LEDGER: &str = "/home/jim/sm-decomp/.scratch/ceres-continuation/ledger.toml";
const LEGACY_SHA: &str = "77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89";

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
        args.len() == 16,
        "usage: readiness --validator PATH --validator-source-checkout PATH --validator-source-commit FULL --validator-source-fingerprint SHA256 --validator-build PATH --prepare-request PATH --drill PATH --output NEW_DIRECTORY",
    )?;
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        ensure(options.insert(pair[0].as_str(), pair[1].as_str()).is_none(), "duplicate option")?;
    }
    for name in [
        "--validator",
        "--validator-source-checkout",
        "--validator-source-commit",
        "--validator-source-fingerprint",
        "--validator-build",
        "--prepare-request",
        "--drill",
        "--output",
    ] {
        ensure(options.contains_key(name), "missing or unknown option")?;
    }
    ensure(
        env::var("CODEX_THREAD_ID").ok().as_deref() == Some(INTEGRATOR),
        "run from the real root integrator session; do not override CODEX_THREAD_ID",
    )?;
    ensure(
        hex(options["--validator-source-commit"], 40) && hex(options["--validator-source-fingerprint"], 64),
        "validator version metadata is malformed",
    )?;
    let mut paths = BTreeMap::new();
    for key in ["--validator", "--validator-source-checkout", "--validator-build", "--prepare-request", "--drill"] {
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
    ensure(head(Path::new(SUBJECT))? == SUBJECT_HEAD, "frozen engine source HEAD changed")?;
    ensure(
        head(&paths["--validator-source-checkout"])? == options["--validator-source-commit"],
        "validator source HEAD differs from recorded version",
    )?;
    ensure(
        hash(Path::new(SUBJECT_BINARY))? == SUBJECT_BINARY_SHA
            && hash(Path::new(BRIDGE))? == BRIDGE_SHA
            && hash(Path::new(LEGACY_LEDGER))? == LEGACY_SHA,
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
    let validator_sha = hash(&paths["--validator"])?;
    let validator_build = text(&paths["--validator-build"])?;
    ensure(
        toml_string_field(&validator_build, "source_checkout")?
            == paths["--validator-source-checkout"].display().to_string()
            && toml_string_field(&validator_build, "source_commit")? == options["--validator-source-commit"]
            && toml_string_field(&validator_build, "source_fingerprint")? == options["--validator-source-fingerprint"]
            && toml_string_field(&validator_build, "binary_path")? == paths["--validator"].display().to_string()
            && toml_string_field(&validator_build, "binary_sha256")? == validator_sha,
        "validator source/binary arguments differ from retained build provenance",
    )?;
    let drill_sha = hash(&paths["--drill"])?;
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
        "schema = \"decomp-readiness-version-map-v1\"\nsubject_engine_checkout = {}\nsubject_engine_commit = {}\nsubject_engine_fingerprint_expected = {}\nsubject_engine_binary = {}\nsubject_engine_binary_sha256 = {}\nvalidator_source_checkout = {}\nvalidator_source_commit = {}\nvalidator_source_fingerprint_recorded = {}\nvalidator_binary = {}\nvalidator_binary_sha256 = {}\nvalidator_build = {}\nvalidator_build_sha256 = {}\ndrill = {}\ndrill_sha256 = {}\nprepare_request = {}\nprepare_request_sha256 = {}\nvalidator_authorship_is_separate_from_executed_engine = true\n",
        q(SUBJECT),
        q(SUBJECT_HEAD),
        q(SUBJECT_FINGERPRINT),
        q(SUBJECT_BINARY),
        q(SUBJECT_BINARY_SHA),
        q(&paths["--validator-source-checkout"].display().to_string()),
        q(options["--validator-source-commit"]),
        q(options["--validator-source-fingerprint"]),
        q(&paths["--validator"].display().to_string()),
        q(&validator_sha),
        q(&paths["--validator-build"].display().to_string()),
        q(&hash(&paths["--validator-build"])?),
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
        .arg(&paths["--validator"])
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
            && head(Path::new(SUBJECT))? == SUBJECT_HEAD
            && hash(&paths["--validator"])? == validator_sha
            && hash(&paths["--drill"])? == drill_sha,
        "host claims, frozen source, validator or drill changed during preflight",
    )?;
    write(
        &output.join("host-preservation.toml"),
        &format!(
            "schema = \"decomp-readiness-host-preservation-v1\"\nstatus = \"complete\"\nauthority_present = false\ncanonical_ledger_present = false\nall_existing_ledger_hashes_unchanged = true\nlegacy_bridge_unchanged = true\nfrozen_engine_head_unchanged = true\nnamespace_exit_code = {}\nactivation_performed = false\n",
            child.code().unwrap_or(-1)
        ),
    )?;
    ensure(
        child.success(),
        "private namespace validation rejected; host policy and claims remain unchanged; see retained namespace/operation diagnostics",
    )?;
    let mut artifacts = BTreeMap::new();
    for entry in fs::read_dir(&output).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_file() {
            artifacts.insert(path.clone(), hash(&path)?);
        }
    }
    write(&output.join("artifacts.sha256"), &seals(&artifacts))?;
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
