//! Actual compiler controls for invocation-scoped preparation and cache reuse.
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Barrier},
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
