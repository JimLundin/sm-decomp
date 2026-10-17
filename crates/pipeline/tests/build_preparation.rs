//! Public compiler recipes, cache authentication and bounded independent work.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-build-preparation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[derive(Clone, Serialize)]
struct CRequest {
    root: PathBuf,
    checkout: PathBuf,
    run: PathBuf,
    workers: usize,
    command: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
struct CacheResult {
    key: String,
    hit: bool,
}
#[derive(Deserialize)]
struct CRecords {
    objects: Vec<CacheResult>,
}
fn spawn<T: Serialize>(binary: &Path, command: &str, request: &T) -> Child {
    let mut child = Command::new(binary)
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(toml::to_string(request).unwrap().as_bytes()).unwrap();
    child
}
fn pipeline() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_sm-pipeline"))
}
fn success(child: Child) -> Output {
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    output
}
fn records(run: &Path) -> Vec<CacheResult> {
    toml::from_str::<CRecords>(&fs::read_to_string(run.join("c-cache.toml")).unwrap()).unwrap().objects
}
fn wait_until(mut ready: impl FnMut() -> bool) {
    let started = Instant::now();
    while !ready() {
        assert!(started.elapsed() < Duration::from_secs(15), "bounded observation timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}

// This adapter delegates every actual compiler invocation to cc. Its gate and
// counter make process progress observable without relying on timing ratios.
fn compiler(checkout: &Path) -> PathBuf {
    let source = checkout.join("compiler.rs");
    fs::write(&source, r#"
use std::{fs::{self, OpenOptions}, io::Write, path::Path, process::Command, time::{Duration, Instant}};
fn main() {
    let executable = std::env::current_exe().unwrap();
    let root = executable.parent().unwrap();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        writeln!(OpenOptions::new().create(true).append(true).open(root.join("versions")).unwrap(), "version").unwrap();
    }
    let source = args.iter().find(|a| a.ends_with(".c")).map(|a| Path::new(a).file_name().unwrap().to_string_lossy().into_owned());
    let compiling = args.iter().any(|a| a == "-c");
    if compiling {
        fs::write(root.join(format!("{}.started", source.as_ref().unwrap())), std::process::id().to_string()).unwrap();
        if source.as_deref() == Some("unit0.c") { fs::write(root.join("slow.started"), "started").unwrap(); }
        let started = Instant::now();
        while root.join("hold-all").exists() || (source.as_deref() == Some("unit0.c") && root.join("hold-slow").exists()) {
            assert!(started.elapsed() < Duration::from_secs(15));
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    let status = Command::new("cc").args(&args).status().unwrap();
    if compiling && status.success() { fs::write(root.join(format!("{}.done", source.unwrap())), "done").unwrap(); }
    std::process::exit(status.code().unwrap_or(1));
}
"#).unwrap();
    let path = checkout.join("compiler");
    assert!(Command::new("rustc").args(["--edition=2024", "-o"]).arg(&path).arg(&source).status().unwrap().success());
    path
}
fn c_request(
    checkout: &Path,
    root: &Path,
    run: &Path,
    count: usize,
    workers: usize,
    cc: &Path,
    nonce: &str,
) -> CRequest {
    let deps = checkout.join("deps");
    fs::create_dir_all(&deps).unwrap();
    fs::create_dir_all(run).unwrap();
    fs::write(deps.join("value.h"), format!("/* {nonce} */\n#define VALUE 3\n")).unwrap();
    let mut command =
        vec![cc.display().to_string(), "-O2".into(), "-DEXTRA=0".into(), "-iquote".into(), deps.display().to_string()];
    let mut main = String::new();
    for index in 0..count {
        let source = deps.join(format!("unit{index}.c"));
        fs::write(
            &source,
            format!("#include \"value.h\"\nint value{index}(void){{return VALUE + {index} + EXTRA;}}\n"),
        )
        .unwrap();
        command.push(source.display().to_string());
        main.push_str(&format!("int value{index}(void);\n"));
    }
    main.push_str(&format!(
        "int main(void){{return {};}}\n",
        (0..count).map(|i| format!("value{i}()")).collect::<Vec<_>>().join("+")
    ));
    fs::write(checkout.join("main.c"), main).unwrap();
    command.extend([
        checkout.join("main.c").display().to_string(),
        "-o".into(),
        run.join("program").display().to_string(),
    ]);
    CRequest { root: root.into(), checkout: checkout.into(), run: run.into(), workers, command }
}

#[test]
fn c_build_prepares_once_and_progresses_independent_units() {
    let directory = Directory::new();
    let root = &directory.0;
    let cc = compiler(root);
    let mut request = c_request(root, root, &root.join("run"), 3, 2, &cc, "progress");
    fs::write(root.join("hold-slow"), "hold").unwrap();
    let child = spawn(pipeline(), "c-build", &request);
    wait_until(|| root.join("slow.started").is_file() && root.join("unit1.c.done").is_file());
    assert!(!root.join("unit0.c.done").exists());
    assert_eq!(fs::read_to_string(root.join("versions")).unwrap().lines().count(), 1);
    fs::remove_file(root.join("hold-slow")).unwrap();
    success(child);
    assert!(records(&request.run).iter().all(|r| !r.hit));
    assert_eq!(Command::new(request.run.join("program")).status().unwrap().code(), Some(12));
    success(spawn(pipeline(), "c-build", &request));
    let warm = records(&request.run);
    assert!(warm.iter().all(|r| r.hit));
    assert_eq!(fs::read_to_string(root.join("versions")).unwrap().lines().count(), 2);
    fs::write(root.join("build-cache").join(&warm[1].key).join("artifact"), "corruption").unwrap();
    success(spawn(pipeline(), "c-build", &request));
    assert_eq!(records(&request.run).iter().map(|r| r.hit).collect::<Vec<_>>(), vec![true, false, true]);
    request.command[2] = "-DEXTRA=2".into();
    success(spawn(pipeline(), "c-build", &request));
    assert!(records(&request.run).iter().all(|r| !r.hit));
    assert_eq!(Command::new(request.run.join("program")).status().unwrap().code(), Some(18));
}

#[test]
fn c_build_rejects_input_mutation_during_the_cache_lock_wait() {
    let directory = Directory::new();
    let root = &directory.0;
    let request = c_request(root, root, &root.join("run"), 1, 1, Path::new("cc"), "wait");
    success(spawn(pipeline(), "c-build", &request));
    let original = records(&request.run);
    let lock_path = root.join("build-cache").join(format!("{}.lock", original[0].key));
    let held = File::open(&lock_path).unwrap();
    held.lock().unwrap();
    let child = spawn(pipeline(), "c-build", &request);
    // The opened cache lock proves the child has already frozen input hashes.
    let descriptors = PathBuf::from(format!("/proc/{}/fd", child.id()));
    wait_until(|| {
        fs::read_dir(&descriptors)
            .unwrap()
            .flatten()
            .any(|entry| fs::read_link(entry.path()).ok().as_ref() == Some(&lock_path))
    });
    fs::write(root.join("deps/value.h"), "#define VALUE 8\n").unwrap();
    drop(held);
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("compiler input changed while waiting"));
    success(spawn(pipeline(), "c-build", &request));
    assert!(!records(&request.run)[0].hit);
    assert_eq!(Command::new(request.run.join("program")).status().unwrap().code(), Some(8));
}

#[derive(Serialize)]
struct HelperRequest {
    checkout: PathBuf,
    root: PathBuf,
    run: PathBuf,
    workers: usize,
    helpers: Vec<String>,
}
#[derive(Deserialize, Serialize)]
struct HelperResult {
    artifacts: Vec<CacheResult>,
}
fn helper_request(checkout: &Path, root: &Path, run: &Path) -> HelperRequest {
    HelperRequest {
        checkout: checkout.into(),
        root: root.into(),
        run: run.into(),
        workers: 1,
        helpers: vec!["fixture-worker".into(), "random-library".into(), "random-cli".into()],
    }
}
fn helper_run(request: &HelperRequest) -> HelperResult {
    let output = success(spawn(pipeline(), "helper-build", request));
    toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
}
fn helper_checkout(checkout: &Path, nonce: &str) {
    fs::create_dir_all(checkout.join("crates/offline-verification/src")).unwrap();
    fs::create_dir_all(checkout.join("tools/decomp")).unwrap();
    fs::write(
        checkout.join("crates/offline-verification/src/fixture_worker.rs"),
        format!("{}\n// {nonce}\n", include_str!("../../offline-verification/src/fixture_worker.rs")),
    )
    .unwrap();
    fs::write(
        checkout.join("tools/decomp/next_random.rs"),
        format!("{}\n// {nonce}\ninclude!(\"extra.rs\");\n", include_str!("../../../tools/decomp/next_random.rs")),
    )
    .unwrap();
    fs::write(checkout.join("tools/decomp/extra.rs"), "#[allow(dead_code)] const EXTRA: u32 = 1;\n").unwrap();
}
#[test]
fn helper_artifacts_are_reused_and_authenticate_discovered_rust_inputs() {
    let directory = Directory::new();
    let root = &directory.0;
    helper_checkout(root, "helper");
    let cold = helper_request(root, root, &root.join("cold"));
    assert!(helper_run(&cold).artifacts.iter().all(|r| !r.hit));
    let warm = helper_request(root, root, &root.join("warm"));
    let restored = helper_run(&warm);
    assert!(restored.artifacts.iter().all(|r| r.hit));
    for name in ["libfixture_worker.a", "librng.a", "rng-cli"] {
        assert_eq!(hash(&cold.run.join(name)), hash(&warm.run.join(name)));
    }
    let exhaustive = |run: &Path| {
        let output = Command::new(run.join("rng-cli")).arg("exhaustive").output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), 65536 * 2);
        output.stdout
    };
    assert_eq!(exhaustive(&cold.run), exhaustive(&warm.run));
    fs::write(root.join("tools/decomp/extra.rs"), "#[allow(dead_code)] const EXTRA: u32 = 2;\n").unwrap();
    assert_eq!(helper_run(&warm).artifacts.iter().map(|r| r.hit).collect::<Vec<_>>(), vec![true, false, false]);
    fs::write(root.join("build-cache").join(&restored.artifacts[0].key).join("artifact"), "corruption").unwrap();
    assert_eq!(helper_run(&warm).artifacts.iter().map(|r| r.hit).collect::<Vec<_>>(), vec![false, true, true]);
    fs::write(root.join("tools/decomp/extra.rs"), "const _: Option<&str> = option_env!(\"SM_TEST_HELPER_INPUT\");\n")
        .unwrap();
    let rejected = spawn(pipeline(), "helper-build", &warm).wait_with_output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("unsupported build-time environment dependency"));
}

#[derive(Serialize)]
struct Sample {
    implementation: String,
    scenario: String,
    elapsed_ns: u64,
    compiler_identity_queries: usize,
    header_sha256: String,
    program_sha256: String,
    objects: Vec<CacheResult>,
}
#[test]
#[ignore = "bounded retained build comparison; SM_BUILD_PREPARATION_PROOF selects the evidence directory"]
fn retained_cold_warm_build_preparation_measurement() {
    let proof = PathBuf::from(std::env::var("SM_BUILD_PREPARATION_PROOF").unwrap());
    let shared = PathBuf::from(std::env::var("SM_PIPELINE_ROOT").unwrap());
    let workers: usize = std::env::var("SM_PIPELINE_WORKERS").unwrap().parse().unwrap();
    assert_eq!(workers, 4);
    fs::create_dir_all(&proof).unwrap();
    let checkout = proof.join("c-checkout");
    fs::create_dir_all(&checkout).unwrap();
    let cc = compiler(&checkout);
    let nonce = format!("{:?}", std::time::SystemTime::now());
    let request = c_request(&checkout, &shared, &proof.join("c-run"), 8, workers, &cc, &nonce);
    let count = || fs::read_to_string(checkout.join("versions")).unwrap_or_default().lines().count();
    let measure = |binary: &Path, implementation: &str, scenario: &str| {
        let before = count();
        let started = Instant::now();
        let output = success(spawn(binary, "c-build", &request));
        let elapsed_ns = started.elapsed().as_nanos() as u64;
        fs::write(proof.join(format!("{implementation}-{scenario}.stdout")), output.stdout).unwrap();
        fs::write(proof.join(format!("{implementation}-{scenario}.stderr")), output.stderr).unwrap();
        assert_eq!(Command::new(request.run.join("program")).status().unwrap().code(), Some(52));
        fs::copy(checkout.join("deps/value.h"), proof.join(format!("{implementation}-{scenario}.value.h"))).unwrap();
        fs::copy(request.run.join("program"), proof.join(format!("{implementation}-{scenario}.program"))).unwrap();
        fs::copy(request.run.join("c-cache.toml"), proof.join(format!("{implementation}-{scenario}.c-cache.toml")))
            .unwrap();
        if request.run.join("c-build.metrics.toml").exists() {
            fs::copy(
                request.run.join("c-build.metrics.toml"),
                proof.join(format!("{implementation}-{scenario}.metrics.toml")),
            )
            .unwrap();
        }
        Sample {
            implementation: implementation.into(),
            scenario: scenario.into(),
            elapsed_ns,
            compiler_identity_queries: count() - before,
            header_sha256: hash(&checkout.join("deps/value.h")),
            program_sha256: hash(&request.run.join("program")),
            objects: records(&request.run),
        }
    };
    let mut samples = Vec::new();
    if let Some(baseline) = std::env::var_os("SM_BUILD_BASELINE_PIPELINE") {
        let baseline = PathBuf::from(baseline);
        fs::write(proof.join("baseline-binary.sha256"), format!("{}  {}\n", hash(&baseline), baseline.display()))
            .unwrap();
        samples.push(measure(&baseline, "baseline", "cold"));
        samples.push(measure(&baseline, "baseline", "warm"));
        samples.push(measure(pipeline(), "prepared", "compatible-warm"));
        assert!(samples[2].objects.iter().all(|r| r.hit));
        assert_eq!(
            samples[0].objects.iter().map(|r| &r.key).collect::<Vec<_>>(),
            samples[2].objects.iter().map(|r| &r.key).collect::<Vec<_>>()
        );
        fs::write(checkout.join("deps/value.h"), format!("/* {nonce} prepared cold */\n#define VALUE 3\n")).unwrap();
    }
    samples.push(measure(pipeline(), "prepared", "cold"));
    samples.push(measure(pipeline(), "prepared", "warm"));
    assert!(!samples[samples.len() - 2].objects.iter().any(|r| r.hit));
    assert!(samples.last().unwrap().objects.iter().all(|r| r.hit));
    #[derive(Serialize)]
    struct Measurements {
        samples: Vec<Sample>,
        helper_cold_ns: u64,
        helper_warm_ns: u64,
        helper_cold: HelperResult,
        helper_warm: HelperResult,
        helper_hashes: std::collections::BTreeMap<String, String>,
    }
    let helpers = proof.join("helper-checkout");
    helper_checkout(&helpers, &nonce);
    let mut cold = helper_request(&helpers, &shared, &proof.join("helper-cold"));
    cold.workers = workers;
    let mut warm = helper_request(&helpers, &shared, &proof.join("helper-warm"));
    warm.workers = workers;
    let started = Instant::now();
    let helper_cold = helper_run(&cold);
    let helper_cold_ns = started.elapsed().as_nanos() as u64;
    let started = Instant::now();
    let helper_warm = helper_run(&warm);
    let helper_warm_ns = started.elapsed().as_nanos() as u64;
    assert!(helper_cold.artifacts.iter().all(|r| !r.hit));
    assert!(helper_warm.artifacts.iter().all(|r| r.hit));
    let mut helper_hashes = std::collections::BTreeMap::new();
    for name in ["libfixture_worker.a", "librng.a", "rng-cli"] {
        let digest = hash(&cold.run.join(name));
        assert_eq!(digest, hash(&warm.run.join(name)));
        helper_hashes.insert(name.into(), digest);
    }
    for request in [&cold, &warm] {
        let output = Command::new(request.run.join("rng-cli")).arg("exhaustive").output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout.len(), 131072);
        fs::write(request.run.join("rng-exhaustive.bin"), output.stdout).unwrap();
    }
    assert_eq!(hash(&cold.run.join("rng-exhaustive.bin")), hash(&warm.run.join("rng-exhaustive.bin")));
    fs::write(
        proof.join("measurements.toml"),
        toml::to_string_pretty(&Measurements {
            samples,
            helper_cold_ns,
            helper_warm_ns,
            helper_cold,
            helper_warm,
            helper_hashes,
        })
        .unwrap(),
    )
    .unwrap();
    fs::write(proof.join("candidate-binary.sha256"), format!("{}  {}\n", hash(pipeline()), pipeline().display()))
        .unwrap();
}

#[test]
fn helper_rejects_a_changed_transitive_closure_before_cache_publication() {
    let directory = Directory::new();
    let root = &directory.0;
    helper_checkout(root, "transitive-race");
    let output = Command::new("rustc").args(["--print", "sysroot"]).output().unwrap();
    assert!(output.status.success());
    let real = Path::new(std::str::from_utf8(&output.stdout).unwrap().trim()).join("bin/rustc");
    let bin = root.join("toolchain/bin");
    fs::create_dir_all(&bin).unwrap();
    let compiler_source = root.join("rustc.rs");
    fs::write(
        &compiler_source,
        r#"
use std::{fs, path::Path, process::Command, time::{Duration, Instant}};
const REAL: &str = __RUSTC__;
fn main() {
    let executable = std::env::current_exe().unwrap();
    let sysroot = executable.parent().unwrap().parent().unwrap();
    let root = sysroot.parent().unwrap();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--print", "sysroot"] { println!("{}", sysroot.display()); return; }
    let status = Command::new(REAL).args(&args).status().unwrap();
    if args.iter().any(|a| a.starts_with("--emit=dep-info=")) && root.join("hold-scan").is_file() {
        fs::write(root.join("scan-finished"), "done").unwrap();
        let start = Instant::now();
        while root.join("hold-scan").is_file() {
            assert!(start.elapsed() < Duration::from_secs(15));
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    if args.iter().any(|a| a.starts_with("--emit=link,")) {
        assert!(Path::new(&std::env::args().next().unwrap()).is_absolute(), "toolchain not pinned");
    }
    std::process::exit(status.code().unwrap_or(1));
}
"#
        .replace("__RUSTC__", &format!("{:?}", real.to_str().unwrap())),
    )
    .unwrap();
    assert!(Command::new(&real).arg(&compiler_source).arg("-o").arg(bin.join("rustc")).status().unwrap().success());
    let path =
        std::env::join_paths(std::iter::once(bin).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())))
            .unwrap();
    let mut request = helper_request(root, root, &root.join("run"));
    request.helpers = vec!["random-cli".into()];
    let launch = || {
        let mut child = Command::new(pipeline())
            .arg("helper-build")
            .env("PATH", &path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
        child
    };
    fs::write(root.join("hold-scan"), "hold").unwrap();
    let child = launch();
    wait_until(|| root.join("scan-finished").exists());
    fs::write(root.join("tools/decomp/extra.rs"), "include!(\"new.rs\");\n").unwrap();
    fs::write(root.join("tools/decomp/new.rs"), "#[allow(dead_code)] const EXTRA: u32 = 1;\n").unwrap();
    fs::remove_file(root.join("hold-scan")).unwrap();
    let rejected = child.wait_with_output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("compiler dependency closure changed during build"));
    assert!(
        !fs::read_dir(root.join("build-cache")).unwrap().flatten().any(|entry| entry.path().join("seal.toml").exists())
    );
    let first: HelperResult = toml::from_str(std::str::from_utf8(&success(launch()).stdout).unwrap()).unwrap();
    assert!(!first.artifacts[0].hit);
    fs::write(root.join("tools/decomp/new.rs"), "#[allow(dead_code)] const EXTRA: u32 = 2;\n").unwrap();
    let changed: HelperResult = toml::from_str(std::str::from_utf8(&success(launch()).stdout).unwrap()).unwrap();
    assert!(!changed.artifacts[0].hit);
    assert_ne!(first.artifacts[0].key, changed.artifacts[0].key);
}

#[test]
fn failed_c_unit_cancels_sibling_cache_wait_and_preserves_the_compiler_error() {
    let directory = Directory::new();
    let root = &directory.0;
    let request = c_request(root, root, &root.join("run"), 2, 2, Path::new("cc"), "failed-sibling");
    success(spawn(pipeline(), "c-build", &request));
    let key = &records(&request.run)[0].key;
    let held = File::open(root.join("build-cache").join(format!("{key}.lock"))).unwrap();
    held.lock().unwrap();
    fs::write(root.join("deps/unit1.c"), "invalid syntax;\n").unwrap();
    let mut child = spawn(pipeline(), "c-build", &request);
    wait_until(|| child.try_wait().unwrap().is_some());
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("c-object-001 failed"));
    drop(held);
}

#[test]
fn sigterm_reaps_all_parallel_c_compilers() {
    let directory = Directory::new();
    let root = &directory.0;
    let cc = compiler(root);
    let request = c_request(root, root, &root.join("run"), 2, 2, &cc, "signal");
    fs::write(root.join("hold-all"), "hold").unwrap();
    let mut child = spawn(pipeline(), "c-build", &request);
    wait_until(|| root.join("unit0.c.started").exists() && root.join("unit1.c.started").exists());
    let compilers: Vec<u32> = (0..2)
        .map(|index| fs::read_to_string(root.join(format!("unit{index}.c.started"))).unwrap().parse().unwrap())
        .collect();
    // SAFETY: this is the live child created by this test; SIGTERM exercises
    // the public command's cancellation and owned process-group teardown.
    assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
    wait_until(|| child.try_wait().unwrap().is_some());
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("interrupted by signal"));
    for compiler in compilers {
        assert!(!Path::new(&format!("/proc/{compiler}")).exists());
    }
    assert!(!request.run.join("c-cache.toml").exists());
}

#[test]
fn helper_pins_checkout_toolchain_across_run_overrides_and_cache_waits() {
    let directory = Directory::new();
    let root = &directory.0;
    helper_checkout(root, "toolchain");
    let output = Command::new("rustc").args(["--print", "sysroot"]).output().unwrap();
    assert!(output.status.success());
    let sysroot = PathBuf::from(std::str::from_utf8(&output.stdout).unwrap().trim());
    let toolchain = sysroot.file_name().unwrap().to_str().unwrap();
    fs::write(root.join("rust-toolchain.toml"), format!("[toolchain]\nchannel = {toolchain:?}\n")).unwrap();
    let mut cold = helper_request(root, root, &root.join("cold"));
    cold.helpers = vec!["random-cli".into()];
    let mut warm = helper_request(root, root, &root.join("warm"));
    warm.helpers = cold.helpers.clone();
    for run in [&cold.run, &warm.run] {
        fs::create_dir(run).unwrap();
        fs::write(run.join("rust-toolchain.toml"), "[toolchain]\nchannel = \"unavailable-run-toolchain\"\n").unwrap();
    }
    let launch = |request: &HelperRequest| {
        let mut child = Command::new(pipeline())
            .arg("helper-build")
            .env_remove("RUSTUP_TOOLCHAIN")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(toml::to_string(request).unwrap().as_bytes()).unwrap();
        child
    };
    let first: HelperResult = toml::from_str(std::str::from_utf8(&success(launch(&cold)).stdout).unwrap()).unwrap();
    assert!(!first.artifacts[0].hit);
    let lock_path = root.join("build-cache").join(format!("{}.lock", first.artifacts[0].key));
    let held = File::open(&lock_path).unwrap();
    held.lock().unwrap();
    let child = launch(&warm);
    let descriptors = PathBuf::from(format!("/proc/{}/fd", child.id()));
    wait_until(|| {
        fs::read_dir(&descriptors)
            .unwrap()
            .flatten()
            .any(|entry| fs::read_link(entry.path()).ok().as_ref() == Some(&lock_path))
    });
    fs::write(warm.run.join("rust-toolchain.toml"), "[toolchain]\nchannel = \"another-unavailable-run-toolchain\"\n")
        .unwrap();
    drop(held);
    let restored: HelperResult = toml::from_str(std::str::from_utf8(&success(child).stdout).unwrap()).unwrap();
    assert!(restored.artifacts[0].hit);
    assert_eq!(hash(&cold.run.join("rng-cli")), hash(&warm.run.join("rng-cli")));
}
