//! Actual compiler controls for invocation-scoped preparation and cache reuse.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Barrier},
    time::{Duration, Instant},
};
use tool_execution::{build_cache::CacheResult, builds, metrics};

struct Directory {
    path: PathBuf,
    retained: bool,
}

impl Directory {
    fn new(name: &str) -> Result<Self> {
        let retained = std::env::var_os("SM_SHARED_BUILD_PROOF").is_some();
        let parent = std::env::var_os("SM_SHARED_BUILD_PROOF").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let path = parent.join(format!("{name}-{}-{}", std::process::id(), tool_execution::unique_id()));
        fs::create_dir_all(&path)?;
        Ok(Self { path, retained })
    }

    fn capacity(&self) -> PathBuf {
        std::env::var_os("SM_PIPELINE_ROOT").map(PathBuf::from).unwrap_or_else(|| self.path.clone())
    }

    fn workers(&self) -> usize {
        std::env::var("SM_PIPELINE_WORKERS").ok().and_then(|value| value.parse().ok()).unwrap_or(4)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        if !self.retained {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn compiler(directory: &Directory) -> Result<PathBuf> {
    let source = directory.path.join("compiler.rs");
    fs::write(
        &source,
        r#"
use std::{fs::OpenOptions, io::Write, process::Command};
fn main() {
    let executable = std::env::current_exe().unwrap();
    let root = executable.parent().unwrap();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let record = if args == ["--version"] { Some("version".to_string()) }
        else if args.iter().any(|arg| arg == "-c") {
            args.iter().find(|arg| arg.ends_with(".c")).cloned()
        } else { None };
    if let Some(record) = record {
        OpenOptions::new().append(true).create(true).open(root.join("compiler-calls")).unwrap().write_all(format!("{record}\n").as_bytes()).unwrap();
    }
    let status = Command::new("cc").args(args).status().unwrap();
    std::process::exit(status.code().unwrap_or(1));
}
"#,
    )?;
    let compiler = directory.path.join("compiler");
    builds::execute(
        &[
            "rustc".into(),
            "--edition=2024".into(),
            source.display().to_string(),
            "-o".into(),
            compiler.display().to_string(),
        ],
        &directory.path.join("adapter-build"),
        "adapter-build",
        &directory.capacity(),
        directory.workers(),
    )?;
    Ok(compiler)
}

fn request(directory: &Directory, compiler: &Path, family: &str) -> Result<builds::CBuildRequest> {
    let run = directory.path.join(family);
    fs::create_dir_all(&run)?;
    fs::write(run.join("main.c"), "int value(void); int main(void) { return value(); }\n")?;
    Ok(builds::CBuildRequest {
        command: vec![
            compiler.display().to_string(),
            "-O2".into(),
            "-iquote".into(),
            directory.path.join("deps").display().to_string(),
            directory.path.join("deps/unit.c").display().to_string(),
            run.join("main.c").display().to_string(),
            "-o".into(),
            run.join("program").display().to_string(),
        ],
        run,
        checkout: directory.path.clone(),
        root: directory.capacity(),
        workers: directory.workers(),
    })
}

fn inputs(directory: &Directory) -> Result<()> {
    fs::create_dir(directory.path.join("deps"))?;
    fs::write(directory.path.join("deps/value.h"), "#define VALUE 7\n")?;
    fs::write(directory.path.join("deps/unit.c"), "#include \"value.h\"\nint value(void) { return VALUE; }\n")?;
    Ok(())
}

fn records(run: &Path) -> Result<Vec<CacheResult>> {
    #[derive(Deserialize)]
    struct Records {
        objects: Vec<CacheResult>,
    }
    Ok(toml::from_str::<Records>(&fs::read_to_string(run.join("c-cache.toml"))?)?.objects)
}

#[test]
fn compatible_families_prepare_compiler_once_and_coalesce_the_actual_object() -> Result<()> {
    let directory = Directory::new("compatible-families")?;
    inputs(&directory)?;
    let compiler = compiler(&directory)?;
    let requests = [request(&directory, &compiler, "a")?, request(&directory, &compiler, "b")?];
    let mut parent = metrics::Run::start(directory.path.join("families.metrics.toml"), "family-builds")?;
    let context = metrics::active().context("test metrics missing")?;
    let builds = Arc::new(builds::BuildContext::default());
    let ready = Arc::new(Barrier::new(requests.len()));
    std::thread::scope(|scope| -> Result<()> {
        let handles: Vec<_> = requests
            .into_iter()
            .map(|request| {
                let context = context.clone();
                let ready = ready.clone();
                let builds = builds.clone();
                scope.spawn(move || {
                    let _entered = context.enter();
                    ready.wait();
                    builds::c_build_in(&builds, request, &|_| Ok(()))
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("family thread panicked")?;
        }
        Ok(())
    })?;
    parent.finish("success", None)?;
    let calls = fs::read_to_string(directory.path.join("compiler-calls"))?;
    let versions = calls.lines().filter(|line| *line == "version").count();
    let unit = directory.path.join("deps/unit.c").display().to_string();
    assert_eq!(calls.lines().filter(|line| *line == unit).count(), 1, "equal-key objects must compile once");
    let a = records(&directory.path.join("a"))?;
    let b = records(&directory.path.join("b"))?;
    assert_eq!(a[0].key, b[0].key);
    assert_ne!(a[0].hit, b[0].hit);
    for family in ["a", "b"] {
        ensure!(
            Command::new(directory.path.join(family).join("program")).status()?.code() == Some(7),
            "actual C result changed"
        );
    }
    assert_eq!(versions, 1, "compatible families repeated compiler identity preparation");
    Ok(())
}

fn rust_request(directory: &Directory, family: &str) -> builds::StandaloneBuildRequest {
    builds::StandaloneBuildRequest {
        checkout: directory.path.clone(),
        root: directory.capacity(),
        run: directory.path.join(family),
        workers: directory.workers(),
        artifacts: vec![builds::StandaloneArtifact {
            source: "helper.rs".into(),
            name: "program".into(),
            label: "helper-build".into(),
            flags: vec![],
        }],
    }
}

#[test]
fn rust_toolchain_is_prepared_once_and_stays_bound_to_checkout_configuration() -> Result<()> {
    let directory = Directory::new("rust-families")?;
    fs::write(directory.path.join("helper.rs"), "fn main() { println!(\"{}\", include_str!(\"value.txt\")); }\n")?;
    fs::write(directory.path.join("value.txt"), "seven")?;
    for family in ["a", "b"] {
        let run = directory.path.join(family);
        fs::create_dir(&run)?;
        fs::write(run.join("rust-toolchain.toml"), "[toolchain]\nchannel = \"unavailable-run-toolchain\"\n")?;
    }
    let context = builds::BuildContext::default();
    let mut metrics = metrics::Run::start(directory.path.join("families.metrics.toml"), "rust-family-builds")?;
    let cold = builds::standalone_builds_in(&context, rust_request(&directory, "a"))?;
    let warm = builds::standalone_builds_in(&context, rust_request(&directory, "b"))?;
    assert!(!cold[0].hit);
    assert!(warm[0].hit);
    assert_eq!(cold[0].key, warm[0].key);
    for family in ["a", "b"] {
        let actual = Command::new(directory.path.join(family).join("program")).output()?;
        assert!(actual.status.success());
        assert_eq!(actual.stdout, b"seven\n");
    }
    // Adding a previously absent checkout override must invalidate the shared
    // discovery, even though the selected compiler executable still exists.
    fs::write(directory.path.join("rust-toolchain.toml"), "[toolchain]\nchannel = \"changed-toolchain\"\n")?;
    let error = builds::standalone_builds_in(&context, rust_request(&directory, "changed")).unwrap_err();
    assert!(error.to_string().contains("Rust toolchain configuration changed"), "{error:#}");
    metrics.finish("success", None)?;
    let receipt: pipeline_model::metrics::Receipt =
        pipeline_model::read(&directory.path.join("families.metrics.toml"))?;
    for role in ["helper-toolchain", "cache-compiler-identity", "cache-rustc-identity", "helper-build"] {
        assert_eq!(receipt.processes.iter().filter(|process| process.role == role).count(), 1, "repeated {role}");
    }
    assert_eq!(receipt.processes.iter().filter(|process| process.role == "helper-build-dependencies").count(), 2);
    Ok(())
}

#[test]
fn unmapped_source_paths_remain_semantic_build_inputs() -> Result<()> {
    let directory = Directory::new("physical-paths")?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    for family in ["first", "second"] {
        let checkout = Directory { path: directory.path.join(family), retained: true };
        fs::create_dir(&checkout.path)?;
        inputs(&checkout)?;
        let request = request(&checkout, &compiler, "run")?;
        fs::write(checkout.path.join("deps/unit.c"), "const char *value(void) { return __FILE__; }\n")?;
        fs::write(
            request.run.join("main.c"),
            "#include <stdio.h>\nconst char *value(void); int main(void) { puts(value()); }\n",
        )?;
        builds::c_build_in(&context, request, &|_| Ok(()))?;
        let actual = Command::new(checkout.path.join("run/program")).output()?;
        assert!(actual.status.success());
        assert_eq!(String::from_utf8(actual.stdout)?.trim(), checkout.path.join("deps/unit.c").display().to_string());
    }
    Ok(())
}

#[test]
fn explicit_file_maps_share_objects_and_preserve_logical_targets_and_flags() -> Result<()> {
    let directory = Directory::new("mapped-paths")?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    let mut previous = None;
    for (family, logical, optimization) in [
        ("first", "logical", "-O2"),
        ("second", "logical", "-O2"),
        ("new-target", "other-logical", "-O2"),
        ("new-flags", "logical", "-O0"),
    ] {
        let checkout = Directory { path: directory.path.join(family), retained: true };
        fs::create_dir(&checkout.path)?;
        inputs(&checkout)?;
        let mut request = request(&checkout, &compiler, "run")?;
        let target = directory.path.join(logical);
        request.command.push(format!("-ffile-prefix-map={}={}", checkout.path.display(), target.display()));
        request.command[1] = optimization.into();
        fs::write(checkout.path.join("deps/value.h"), "static const char *header(void) { return __FILE__; }\n")?;
        fs::write(
            checkout.path.join("deps/unit.c"),
            "#include \"value.h\"\n#include <stdio.h>\nint value(void) { puts(__FILE__); puts(__BASE_FILE__); puts(header()); return 0; }\n",
        )?;
        builds::c_build_in(&context, request, &|_| Ok(()))?;
        let actual = Command::new(checkout.path.join("run/program")).output()?;
        assert!(actual.status.success());
        assert_eq!(
            String::from_utf8(actual.stdout)?,
            format!(
                "{}\n{}\n{}\n",
                target.join("deps/unit.c").display(),
                target.join("deps/unit.c").display(),
                target.join("deps/value.h").display()
            )
        );
        let records = records(&checkout.path.join("run"))?;
        if family == "second" {
            assert!(records[0].hit);
            assert_eq!(previous.as_ref(), Some(&records[0].key));
        } else {
            assert!(!records[0].hit);
            if family == "first" {
                previous = Some(records[0].key.clone());
            } else {
                assert_ne!(previous.as_ref(), Some(&records[0].key));
            }
        }
    }
    Ok(())
}

#[test]
fn file_maps_keep_artificial_line_paths_in_the_computation() -> Result<()> {
    let directory = Directory::new("artificial-line-paths")?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    let artificial = directory.path.join("first/virtual.c");
    let logical = directory.path.join("logical");
    for family in ["first", "second"] {
        let checkout = Directory { path: directory.path.join(family), retained: true };
        fs::create_dir(&checkout.path)?;
        inputs(&checkout)?;
        let mut request = request(&checkout, &compiler, "run")?;
        request.command.push(format!("-ffile-prefix-map={}={}", checkout.path.display(), logical.display()));
        fs::write(
            checkout.path.join("deps/unit.c"),
            format!("#line 1 {:?}\nconst char *value(void) {{ return __FILE__; }}\n", artificial.to_str().unwrap()),
        )?;
        fs::write(
            request.run.join("main.c"),
            "#include <stdio.h>\nconst char *value(void); int main(void) { puts(value()); }\n",
        )?;
        builds::c_build_in(&context, request, &|_| Ok(()))?;
        let actual = Command::new(checkout.path.join("run/program")).output()?;
        assert!(actual.status.success());
        let expected = if family == "first" { logical.join("virtual.c") } else { artificial.clone() };
        assert_eq!(String::from_utf8(actual.stdout)?.trim(), expected.display().to_string());
    }
    Ok(())
}

#[test]
fn volatile_c_units_stay_fresh_through_headers_and_diagnostic_pragmas() -> Result<()> {
    let directory = Directory::new("volatile-inputs")?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    for (case, header) in [
        ("date", "#define CLOCK_TEXT __DATE__\n"),
        ("time", "#define INNER_CLOCK __TIME__\n#define CLOCK_TEXT INNER_CLOCK\n"),
        ("timestamp", "#define CLOCK_TEXT __TIMESTAMP__\n"),
        ("system-time", "#define CLOCK_TEXT __TIME__\n"),
        ("pragma", "#pragma GCC diagnostic ignored \"-Wdate-time\"\n#define CLOCK_TEXT __TIME__\n"),
        ("system-pragma", "#pragma GCC diagnostic ignored \"-Wdate-time\"\n#define CLOCK_TEXT __TIME__\n"),
        (
            "macro-pragma",
            "#define SILENCE _Pragma(\"GCC diagnostic ignored \\\"-Wdate-time\\\"\")\nSILENCE\n#define CLOCK_TEXT __TIMESTAMP__\n",
        ),
    ] {
        let checkout = Directory { path: directory.path.join(case), retained: true };
        fs::create_dir(&checkout.path)?;
        inputs(&checkout)?;
        fs::write(checkout.path.join("deps/value.h"), header)?;
        fs::write(
            checkout.path.join("deps/unit.c"),
            if case.starts_with("system-") {
                "#include <value.h>\nconst char *value(void) { return CLOCK_TEXT; }\n"
            } else {
                "#include \"value.h\"\nconst char *value(void) { return CLOCK_TEXT; }\n"
            },
        )?;
        for family in ["a", "b"] {
            let mut request = request(&checkout, &compiler, family)?;
            if case.starts_with("system-") {
                request.command[2] = "-isystem".into();
            }
            fs::write(
                request.run.join("main.c"),
                "#include <stdio.h>\nconst char *value(void); int main(void) { puts(value()); }\n",
            )?;
            let run = request.run.clone();
            builds::c_build_in(&context, request, &|_| Ok(()))?;
            assert!(records(&run)?.is_empty(), "{case} incorrectly became cache eligible");
            let metadata: toml::Value = toml::from_str(&fs::read_to_string(run.join("c-cache.toml"))?)?;
            assert_eq!(metadata["units"][0]["reason"].as_str(), Some("volatile-or-unsupported-preprocessing"));
            let actual = Command::new(run.join("program")).output()?;
            assert!(actual.status.success());
            assert!(!actual.stdout.trim_ascii().is_empty());
        }
        let unit = checkout.path.join("deps/unit.c").display().to_string();
        let calls = fs::read_to_string(directory.path.join("compiler-calls"))?;
        assert_eq!(calls.lines().filter(|line| *line == unit).count(), 2, "{case} did not execute both real compilers");
    }
    Ok(())
}

#[test]
fn failed_family_does_not_cancel_an_independent_family_in_the_same_context() -> Result<()> {
    let directory = Directory::new("independent-cancellation")?;
    inputs(&directory)?;
    let compiler = compiler(&directory)?;
    let requests = [request(&directory, &compiler, "good")?, request(&directory, &compiler, "bad")?];
    fs::write(requests[1].run.join("main.c"), "invalid syntax;\n")?;
    let builds = Arc::new(builds::BuildContext::default());
    let mut parent = metrics::Run::start(directory.path.join("families.metrics.toml"), "family-builds")?;
    let context = metrics::active().context("test metrics missing")?;
    let ready = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let handles: Vec<_> = requests
            .into_iter()
            .map(|request| {
                let context = context.clone();
                let builds = builds.clone();
                let ready = ready.clone();
                scope.spawn(move || {
                    let _entered = context.enter();
                    ready.wait();
                    builds::c_build_in(&builds, request, &|_| Ok(()))
                })
            })
            .collect();
        handles.into_iter().map(|handle| handle.join().expect("family panicked")).collect::<Vec<_>>()
    });
    assert!(results[0].is_ok(), "independent family was cancelled: {:?}", results[0]);
    assert!(results[1].as_ref().unwrap_err().to_string().contains("c-object-001 failed"));
    assert_eq!(Command::new(directory.path.join("good/program")).status()?.code(), Some(7));
    assert!(directory.path.join("bad/c-object-001.stderr").is_file());
    assert!(!directory.path.join("bad/c-cache.toml").exists());
    parent.finish("success", None)?;
    Ok(())
}

fn wait_until(mut ready: impl FnMut() -> bool) -> Result<()> {
    let started = Instant::now();
    while !ready() {
        ensure!(started.elapsed() < Duration::from_secs(20), "bounded cache-wait observation timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn mutate_while_waiting(
    directory: &Directory,
    context: &builds::BuildContext,
    request: builds::CBuildRequest,
    key: &str,
    mutation: impl FnOnce() -> Result<()>,
) -> Result<anyhow::Error> {
    let lock = directory.capacity().join("build-cache").join(format!("{key}.lock"));
    let held = File::open(&lock)?;
    held.lock()?;
    let mut parent = metrics::Run::start(request.run.join("wait.metrics.toml"), "cache-wait-control")?;
    let metrics = metrics::active().context("test metrics missing")?;
    let result = std::thread::scope(|scope| -> Result<_> {
        let child = scope.spawn(|| {
            let _entered = metrics.enter();
            builds::c_build_in(context, request, &|_| Ok(()))
        });
        let waiting = wait_until(|| {
            fs::read_dir("/proc/self/fd")
                .unwrap()
                .flatten()
                .filter(|entry| fs::read_link(entry.path()).ok().as_ref() == Some(&lock))
                .count()
                >= 2
        });
        let changed = if waiting.is_ok() { mutation() } else { Ok(()) };
        drop(held);
        let result = child.join().expect("waiting family panicked");
        waiting?;
        changed?;
        Ok(result)
    })?;
    parent.finish("success", None)?;
    match result {
        Ok(_) => Err(anyhow::anyhow!("changed build was incorrectly admitted")),
        Err(error) => Ok(error),
    }
}

#[test]
fn shared_context_rejects_header_mutation_during_an_artifact_wait() -> Result<()> {
    let directory = Directory::new("header-wait")?;
    inputs(&directory)?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    let first = request(&directory, &compiler, "seed")?;
    let seed = first.run.clone();
    builds::c_build_in(&context, first, &|_| Ok(()))?;
    let key = records(&seed)?[0].key.clone();
    let error = mutate_while_waiting(&directory, &context, request(&directory, &compiler, "waiting")?, &key, || {
        Ok(fs::write(directory.path.join("deps/value.h"), "#define VALUE 9\n")?)
    })?;
    assert!(format!("{error:#}").contains("compiler input changed while waiting"), "{error:#}");
    assert!(!directory.path.join("waiting/c-cache.toml").exists());
    let stale = builds::c_build_in(&context, request(&directory, &compiler, "stale-context")?, &|_| Ok(()));
    assert!(format!("{:#}", stale.unwrap_err()).contains("compiler input changed"));
    builds::c_build_in(&builds::BuildContext::default(), request(&directory, &compiler, "fresh")?, &|_| Ok(()))?;
    assert!(!records(&directory.path.join("fresh"))?[0].hit);
    assert_eq!(Command::new(directory.path.join("fresh/program")).status()?.code(), Some(9));
    Ok(())
}

#[test]
fn shared_context_rejects_new_transitive_include_during_an_artifact_wait() -> Result<()> {
    let directory = Directory::new("include-wait")?;
    inputs(&directory)?;
    fs::write(
        directory.path.join("deps/unit.c"),
        "#if __has_include(\"later.h\")\n#include \"later.h\"\n#else\n#define VALUE 7\n#endif\nint value(void) { return VALUE; }\n",
    )?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    builds::c_build_in(&context, request(&directory, &compiler, "seed")?, &|_| Ok(()))?;
    let key = records(&directory.path.join("seed"))?[0].key.clone();
    let error = mutate_while_waiting(&directory, &context, request(&directory, &compiler, "waiting")?, &key, || {
        Ok(fs::write(directory.path.join("deps/later.h"), "#define VALUE 11\n")?)
    })?;
    assert!(format!("{error:#}").contains("compiler dependency closure changed"), "{error:#}");
    assert!(!directory.path.join("waiting/c-cache.toml").exists());
    builds::c_build_in(&builds::BuildContext::default(), request(&directory, &compiler, "fresh")?, &|_| Ok(()))?;
    assert_eq!(Command::new(directory.path.join("fresh/program")).status()?.code(), Some(11));
    Ok(())
}

#[test]
fn shared_context_rechecks_compiler_bytes_after_artifact_waits() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let directory = Directory::new("compiler-wait")?;
    inputs(&directory)?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    builds::c_build_in(&context, request(&directory, &compiler, "seed")?, &|_| Ok(()))?;
    let key = records(&directory.path.join("seed"))?[0].key.clone();
    let error = mutate_while_waiting(&directory, &context, request(&directory, &compiler, "waiting")?, &key, || {
        let mut bytes = fs::read(&compiler)?;
        bytes.extend_from_slice(b"changed-compiler-identity");
        let replacement = compiler.with_extension("replacement");
        fs::write(&replacement, bytes)?;
        fs::set_permissions(&replacement, fs::Permissions::from_mode(0o755))?;
        Ok(fs::rename(replacement, &compiler)?)
    })?;
    assert!(format!("{error:#}").contains("prepared compiler changed"), "{error:#}");
    assert!(!directory.path.join("waiting/c-cache.toml").exists());
    builds::c_build_in(&builds::BuildContext::default(), request(&directory, &compiler, "fresh")?, &|_| Ok(()))?;
    assert!(!records(&directory.path.join("fresh"))?[0].hit);
    assert_eq!(Command::new(directory.path.join("fresh/program")).status()?.code(), Some(7));
    Ok(())
}

#[test]
fn corrupted_shared_artifact_is_rebuilt_and_authenticated() -> Result<()> {
    let directory = Directory::new("artifact-corruption")?;
    inputs(&directory)?;
    let compiler = compiler(&directory)?;
    let context = builds::BuildContext::default();
    builds::c_build_in(&context, request(&directory, &compiler, "seed")?, &|_| Ok(()))?;
    let key = records(&directory.path.join("seed"))?[0].key.clone();
    fs::write(directory.capacity().join("build-cache").join(&key).join("artifact"), "corrupt object")?;
    builds::c_build_in(&context, request(&directory, &compiler, "repaired")?, &|_| Ok(()))?;
    assert!(!records(&directory.path.join("repaired"))?[0].hit);
    builds::c_build_in(&context, request(&directory, &compiler, "restored")?, &|_| Ok(()))?;
    assert!(records(&directory.path.join("restored"))?[0].hit);
    assert_eq!(Command::new(directory.path.join("restored/program")).status()?.code(), Some(7));
    let calls = fs::read_to_string(directory.path.join("compiler-calls"))?;
    let source = directory.path.join("deps/unit.c").display().to_string();
    assert_eq!(calls.lines().filter(|line| *line == source).count(), 2);
    assert_eq!(calls.lines().filter(|line| *line == "version").count(), 1);
    Ok(())
}

#[test]
fn preprocessor_budget_exhaustion_is_a_hard_failure_without_cache_publication() -> Result<()> {
    let directory = Directory::new("preprocessor-budget")?;
    inputs(&directory)?;
    let compiler = compiler(&directory)?;
    let mut request = request(&directory, &compiler, "limited")?;
    request.command.retain(|argument| !argument.ends_with("/main.c"));
    fs::write(
        directory.path.join("deps/unit.c"),
        (0..32).map(|index| format!("int value{index}(void) {{ return {index}; }}\n")).collect::<String>(),
    )?;
    let error = tool_execution::bounded::until(tool_execution::bounded::now_unix_ms()? + 30_000, 512, || {
        builds::c_build_in(&builds::BuildContext::default(), request, &|_| Ok(()))
    })
    .unwrap_err();
    assert!(format!("{error:#}").contains("budget"), "{error:#}");
    assert!(!directory.path.join("limited/c-cache.toml").exists());
    let calls = fs::read_to_string(directory.path.join("compiler-calls"))?;
    assert!(calls.lines().all(|line| line == "version"), "budget failure incorrectly launched an object compiler");
    Ok(())
}

#[test]
fn compiler_environment_is_bound_even_when_declared_inputs_stay_equal() -> Result<()> {
    // Set environment only on new processes. Mutating this multithreaded test
    // process's environment would be unsafe and could contaminate other gates.
    if let Some(path) = std::env::var_os("SM_SHARED_BUILD_ENVIRONMENT_DIRECTORY") {
        let directory = Directory { path: path.into(), retained: true };
        let family = std::env::var("SM_SHARED_BUILD_ENVIRONMENT_FAMILY")?;
        builds::c_build_in(
            &builds::BuildContext::default(),
            request(&directory, &directory.path.join("compiler"), &family)?,
            &|_| Ok(()),
        )?;
        assert_eq!(Command::new(directory.path.join(family).join("program")).status()?.code(), Some(7));
        return Ok(());
    }
    let directory = Directory::new("environment-identity")?;
    inputs(&directory)?;
    compiler(&directory)?;
    let mut first = None;
    for (family, environment, hit) in [("seed", "1", false), ("same", "1", true), ("changed", "2", false)] {
        let output = Command::new(std::env::current_exe()?)
            .args(["--exact", "compiler_environment_is_bound_even_when_declared_inputs_stay_equal", "--nocapture"])
            .env("SM_SHARED_BUILD_ENVIRONMENT_DIRECTORY", &directory.path)
            .env("SM_SHARED_BUILD_ENVIRONMENT_FAMILY", family)
            .env("CFLAGS", format!("-DENVIRONMENT_IDENTITY_CONTROL={environment}"))
            .output()?;
        fs::write(directory.path.join(format!("{family}.stdout")), &output.stdout)?;
        fs::write(directory.path.join(format!("{family}.stderr")), &output.stderr)?;
        ensure!(
            output.status.success(),
            "environment subprocess {family} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let record = records(&directory.path.join(family))?.remove(0);
        assert_eq!(record.hit, hit);
        let mut seal: toml::Value = toml::from_str(&fs::read_to_string(
            directory.capacity().join("build-cache").join(&record.key).join("seal.toml"),
        )?)?;
        let inputs = seal.get_mut("inputs").and_then(toml::Value::as_table_mut).context("cache inputs missing")?;
        let environment_hash = inputs.remove("environment_sha256").context("cache environment missing")?;
        if let Some((first_key, first_inputs, first_environment)) = &first {
            assert_eq!(inputs, first_inputs, "environment control changed another cache input");
            if family == "same" {
                assert_eq!(&record.key, first_key);
                assert_eq!(&environment_hash, first_environment);
            } else {
                assert_ne!(&record.key, first_key);
                assert_ne!(&environment_hash, first_environment);
            }
        } else {
            first = Some((record.key, inputs.clone(), environment_hash));
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct PreparationMeasurement {
    round: usize,
    mode: String,
    seed: String,
    elapsed_ns: u64,
    checkout: PathBuf,
    metrics: PathBuf,
    key: String,
    cache_hits: usize,
    compiler_processes: BTreeMap<String, usize>,
}

fn measure_preparation(
    directory: &Directory,
    compiler: &Path,
    round: usize,
    shared: bool,
    warm: bool,
) -> Result<PreparationMeasurement> {
    let seed = if warm { "warm" } else { "cold" };
    let mode = if shared { "shared" } else { "private" };
    let requests =
        [request(directory, compiler, &format!("{seed}-a"))?, request(directory, compiler, &format!("{seed}-b"))?];
    let metrics_path = directory.path.join(format!("{seed}.metrics.toml"));
    let started = Instant::now();
    let mut parent = metrics::Run::start(metrics_path.clone(), "matched-preparation-measurement")?;
    let metrics = metrics::active().context("measurement metrics missing")?;
    let context = Arc::new(builds::BuildContext::default());
    let ready = Arc::new(Barrier::new(requests.len()));
    std::thread::scope(|scope| -> Result<()> {
        let handles: Vec<_> = requests
            .into_iter()
            .map(|request| {
                let metrics = metrics.clone();
                let context = if shared { context.clone() } else { Arc::new(builds::BuildContext::default()) };
                let ready = ready.clone();
                scope.spawn(move || {
                    let _entered = metrics.enter();
                    ready.wait();
                    builds::c_build_in(&context, request, &|_| Ok(()))
                })
            })
            .collect();
        for handle in handles {
            handle.join().expect("measurement family panicked")?;
        }
        Ok(())
    })?;
    parent.finish("success", None)?;
    let mut cache = Vec::new();
    for family in [format!("{seed}-a"), format!("{seed}-b")] {
        assert_eq!(Command::new(directory.path.join(&family).join("program")).status()?.code(), Some(7));
        cache.extend(records(&directory.path.join(family))?);
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos())?;
    assert_eq!(cache.len(), 2);
    assert_eq!(cache[0].key, cache[1].key);
    let cache_hits = cache.iter().filter(|result| result.hit).count();
    assert_eq!(cache_hits, if warm { 2 } else { 1 });
    let receipt: pipeline_model::metrics::Receipt = pipeline_model::read(&metrics_path)?;
    let mut compiler_processes = BTreeMap::new();
    for process in receipt.processes {
        *compiler_processes.entry(process.role).or_default() += 1;
    }
    assert_eq!(compiler_processes.get("cache-compiler-identity"), Some(&if shared { 1 } else { 2 }));
    assert_eq!(compiler_processes.get("c-object-000").copied().unwrap_or_default(), if warm { 0 } else { 1 });
    assert_eq!(compiler_processes.get("c-object-001"), Some(&2));
    assert_eq!(compiler_processes.get("c-link"), Some(&2));
    Ok(PreparationMeasurement {
        round,
        mode: mode.into(),
        seed: seed.into(),
        elapsed_ns,
        checkout: directory.path.clone(),
        metrics: metrics_path,
        key: cache.remove(0).key,
        cache_hits,
        compiler_processes,
    })
}

#[test]
#[ignore = "opt-in matched cold/warm preparation measurement"]
fn preparation_matched_cold_warm_measurements() -> Result<()> {
    let directory = Directory::new("matched-measurements")?;
    let compiler = compiler(&directory)?;
    let mut observations = Vec::new();
    // Separate physical checkouts provide empty cache seeds without clearing
    // the shared cache. Their C bytes and flags are equal and use no file/cwd
    // macros; the typed absolute source/include operands are the only variation.
    // Alternate ordering to expose rather than hide host/cache warming noise.
    for round in 0..3 {
        for shared in if round % 2 == 0 { [false, true] } else { [true, false] } {
            let mode = if shared { "shared" } else { "private" };
            let checkout = Directory { path: directory.path.join(format!("{round}-{mode}")), retained: true };
            fs::create_dir(&checkout.path)?;
            inputs(&checkout)?;
            for warm in [false, true] {
                let sample = measure_preparation(&checkout, &compiler, round, shared, warm)?;
                println!(
                    "round={} mode={} seed={} wall_ns={} processes={:?}",
                    sample.round, sample.mode, sample.seed, sample.elapsed_ns, sample.compiler_processes
                );
                observations.push(sample);
            }
        }
    }
    #[derive(Serialize)]
    struct Measurements {
        format: &'static str,
        compiler: PathBuf,
        compiler_sha256: String,
        source_sha256: String,
        header_sha256: String,
        capacity_root: PathBuf,
        workers: usize,
        observations: Vec<PreparationMeasurement>,
    }
    let source = directory.path.join("0-private/deps/unit.c");
    let header = directory.path.join("0-private/deps/value.h");
    let source_sha256 = tool_execution::digest(&source)?;
    let header_sha256 = tool_execution::digest(&header)?;
    for sample in &observations {
        assert_eq!(tool_execution::digest(&sample.checkout.join("deps/unit.c"))?, source_sha256);
        assert_eq!(tool_execution::digest(&sample.checkout.join("deps/value.h"))?, header_sha256);
    }
    fs::write(
        directory.path.join("measurements.toml"),
        toml::to_string_pretty(&Measurements {
            format: "shared-preparation-measurements-v1",
            compiler_sha256: tool_execution::digest(&compiler)?,
            compiler,
            source_sha256,
            header_sha256,
            capacity_root: directory.capacity(),
            workers: directory.workers(),
            observations,
        })?,
    )?;
    Ok(())
}
