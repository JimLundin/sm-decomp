//! Local process leases and authenticated cache publication for decomp workers.
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

mod builds;
mod inputs;

fn digest(path: &Path) -> Result<String> {
    let mut hash = Sha256::new();
    let mut input = File::open(path)?;
    let mut buffer = [0u8; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    file.lock()?;
    Ok(file)
}

fn emit(value: &Value) -> Result<()> {
    println!("{value}");
    std::io::stdout().flush()?;
    Ok(())
}

fn line() -> Result<String> {
    let mut value = String::new();
    ensure!(std::io::stdin().lock().read_line(&mut value)? > 0, "client closed without commit");
    Ok(value)
}

fn acquire_lease(root: &Path, requested: usize) -> Result<(File, f64)> {
    ensure!((1..=64).contains(&requested), "worker count must be 1..64");
    let started = Instant::now();
    let root = root.join("workers");
    fs::create_dir_all(&root)?;
    let guard = lock(&root.join("config.lock"))?;
    let config = root.join("capacity.json");
    let capacity = if config.exists() {
        let doc: Value = serde_json::from_reader(File::open(config)?)?;
        doc["workers"].as_u64().context("missing worker capacity")? as usize
    } else {
        atomic_json(&config, &json!({"workers":requested}))?;
        requested
    };
    ensure!(capacity == requested, "shared worker budget already configured to {capacity}");
    drop(guard);
    let held = loop {
        let mut selected = None;
        for slot in 0..capacity {
            let file = OpenOptions::new().create(true).append(true).open(root.join(format!("{slot}.lock")))?;
            if file.try_lock().is_ok() {
                selected = Some(file);
                break;
            }
        }
        if let Some(file) = selected {
            break file;
        }
        ensure!(started.elapsed() < Duration::from_secs(3600), "worker wait exceeded one hour");
        std::thread::sleep(Duration::from_millis(50));
    };
    Ok((held, started.elapsed().as_secs_f64()))
}

fn lease(root: &Path, requested: usize) -> Result<()> {
    let (held, wait) = acquire_lease(root, requested)?;
    emit(&json!({"worker_wait_seconds":wait}))?;
    // EOF releases a lease too, so killed clients cannot strand worker capacity.
    let _ = line();
    drop(held);
    Ok(())
}

fn atomic_json(path: &Path, value: &Value) -> Result<()> {
    let pending = path.with_extension(format!("pending-{}", std::process::id()));
    let mut out = OpenOptions::new().create_new(true).write(true).open(&pending)?;
    writeln!(out, "{value}")?;
    out.sync_all()?;
    fs::rename(&pending, path)?;
    File::open(path.parent().context("path parent missing")?)?.sync_all()?;
    Ok(())
}

fn executable(name: &str) -> Result<PathBuf> {
    if name.contains('/') {
        return Ok(PathBuf::from(name));
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").context("PATH missing")?) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("compiler not found: {name}")
}

fn cached_artifact(request: &Value, build: impl FnOnce(&Path) -> Result<()>) -> Result<Value> {
    let root = PathBuf::from(request["root"].as_str().context("cache root missing")?);
    let checkout = PathBuf::from(request["checkout"].as_str().context("checkout missing")?);
    let common = Command::new("git").args(["rev-parse", "--git-common-dir"]).current_dir(&checkout).output()?;
    let repository = if common.status.success() {
        let common = PathBuf::from(String::from_utf8(common.stdout)?.trim());
        let common = fs::canonicalize(checkout.join(common))?;
        common.parent().context("git common parent missing")?.to_path_buf()
    } else {
        checkout.clone()
    };
    let inputs = request["inputs"].as_array().context("inputs missing")?;
    let command = request["command"].as_array().context("command missing")?;
    let compiler_name = command.first().and_then(Value::as_str).context("compiler missing")?;
    let compiler = executable(compiler_name)?;
    let version = Command::new(&compiler).arg("--version").current_dir(&checkout).output()?;
    ensure!(version.status.success(), "compiler version failed");
    let mut sources = serde_json::Map::new();
    let mut originals = Vec::new();
    for name in inputs {
        let path = fs::canonicalize(name.as_str().context("invalid input path")?)?;
        let key = path
            .strip_prefix(&checkout)
            .or_else(|_| path.strip_prefix(&repository))
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let hash = digest(&path)?;
        originals.push((path, hash.clone()));
        sources.insert(key, hash.into());
    }
    let environment: std::collections::BTreeMap<_, _> = std::env::vars()
        .filter(|(name, _)| {
            name.starts_with("CARGO_")
                || name.starts_with("RUST")
                || name.starts_with("CC_")
                || [
                    "CC",
                    "AR",
                    "CFLAGS",
                    "CPPFLAGS",
                    "LDFLAGS",
                    "PATH",
                    "CPATH",
                    "C_INCLUDE_PATH",
                    "CPLUS_INCLUDE_PATH",
                    "LIBRARY_PATH",
                    "SOURCE_DATE_EPOCH",
                    "GCC_EXEC_PREFIX",
                    "COMPILER_PATH",
                    "SDKROOT",
                    "MACOSX_DEPLOYMENT_TARGET",
                ]
                .contains(&name.as_str())
        })
        .collect();
    let env_hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&environment)?));
    let canonical_command: Vec<_> = command
        .iter()
        .map(|arg| {
            Ok(arg
                .as_str()
                .context("invalid command argument")?
                .replace(checkout.to_string_lossy().as_ref(), "{checkout}")
                .replace(repository.to_string_lossy().as_ref(), "{checkout}"))
        })
        .collect::<Result<_>>()?;
    let mut identity = json!({"version":3,"command":canonical_command,"sources":sources,
        "compiler":{"sha256":digest(&compiler)?,"version":String::from_utf8_lossy(&version.stdout)},
        "environment_sha256":env_hash,"platform":[std::env::consts::ARCH,std::env::consts::OS]});
    if ["cargo", "rustc"].contains(&Path::new(compiler_name).file_name().and_then(|v| v.to_str()).unwrap_or_default()) {
        let info = Command::new("rustc").arg("-vV").current_dir(&checkout).output()?;
        ensure!(info.status.success(), "rustc version failed");
        identity["rustc"] = String::from_utf8_lossy(&info.stdout).into_owned().into();
    }
    let key = format!("{:x}", Sha256::digest(serde_json::to_vec(&identity)?));
    let root = root.join("build-cache");
    fs::create_dir_all(&root)?;
    let _held = lock(&root.join(format!("{key}.lock")))?;
    let entry = root.join(&key);
    for (path, expected) in &originals {
        ensure!(digest(path)? == *expected, "compiler input changed while waiting for cache");
    }
    let hit = (|| -> Result<bool> {
        ensure!(!entry.symlink_metadata()?.file_type().is_symlink(), "cache symlink");
        let seal: Value = serde_json::from_reader(File::open(entry.join("seal.json"))?)?;
        ensure!(!entry.join("artifact").symlink_metadata()?.file_type().is_symlink(), "artifact symlink");
        Ok(seal["inputs"] == identity && seal["sha256"].as_str() == Some(&digest(&entry.join("artifact"))?))
    })()
    .unwrap_or(false);
    let stage = root.join(format!("{key}-{}.pending", std::process::id()));
    if !hit {
        if stage.exists() {
            fs::remove_dir_all(&stage)?;
        }
        fs::create_dir(&stage)?;
    }
    if !hit {
        let result = (|| -> Result<()> {
            build(&stage.join("artifact"))?;
            for (path, expected) in originals {
                ensure!(digest(&path)? == expected, "compiler input changed during build");
            }
            ensure!(!stage.join("artifact").symlink_metadata()?.file_type().is_symlink(), "build artifact symlink");
            let hash = digest(&stage.join("artifact"))?;
            File::open(stage.join("artifact"))?.sync_all()?;
            atomic_json(&stage.join("seal.json"), &json!({"inputs":identity,"sha256":hash}))?;
            if entry.exists() {
                fs::remove_dir_all(&entry)?;
            }
            fs::rename(&stage, &entry)?;
            File::open(&root)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&stage);
        }
        result?;
    }
    let destination = PathBuf::from(request["destination"].as_str().context("destination missing")?);
    fs::create_dir_all(destination.parent().context("destination parent missing")?)?;
    fs::copy(entry.join("artifact"), &destination)?;
    ensure!(digest(&destination)? == digest(&entry.join("artifact"))?, "copied artifact differs");
    Ok(json!({"key":key,"hit":hit}))
}

fn cache(request: Value) -> Result<()> {
    let mut missed = false;
    let result = cached_artifact(&request, |artifact| {
        missed = true;
        emit(&json!({"hit":false,"artifact":artifact}))?;
        ensure!(line()?.trim() == "commit", "cache client aborted");
        Ok(())
    })?;
    if !missed {
        emit(&json!({"hit":true}))?;
    }
    emit(&result)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("lease") if args.len() == 3 => lease(Path::new(&args[1]), args[2].parse()?),
        Some("cache") if args.len() == 1 => cache(serde_json::from_str(&line()?)?),
        Some("wrapper") if args.len() == 1 => emit(&inputs::wrapper(serde_json::from_str(&line()?)?)?),
        Some("dependencies") if args.len() == 1 => emit(&inputs::dependencies(serde_json::from_str(&line()?)?)?),
        Some("c-build") if args.len() == 1 => emit(&builds::c_build(serde_json::from_str(&line()?)?)?),
        _ => bail!(
            "usage: sm-pipeline lease ROOT WORKERS | cache | wrapper | dependencies | c-build (JSON request on stdin)"
        ),
    }
}
