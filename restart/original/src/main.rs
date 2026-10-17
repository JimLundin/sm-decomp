//! Authenticated, bounded original-ROM capture; no native-state transplantation.
mod fixture;
mod handle_calls;
mod handle_fixture;
mod movie;
mod observer;
mod validate;

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, String>;
type Table = toml::map::Map<String, toml::Value>;
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const MOVIE_SHA256: &str = "e53b1ce15b3f01516eb64c68a6db2911f91bd97197eb05960779561be5c7cbe9";
const LSNES_SHA256: &str = "c1226547e2a7d4efedd07819c20fb3ed9d778f49dce74b8da0c0ec5b38d85459";
const USAGE: &str = "Usage: sm-original capture --lsnes PATH --rom PATH --movie PATH --libraries DIR --out NEW_DIR [--prepare-only yes]\nAuthenticates pinned inputs; preserves original movie frames 1..8639, presses Right on frame 8650, releases, presses physical X on frame 8660, then releases through 8850. Captures from reset with a 120-second deadline. --prepare-only yes authors and verifies inputs without running lsnes. This is original capture evidence, not a C/Rust equivalence verdict.";
const COMPARE_USAGE: &str = "sm-original compare --capture DIR --library PATH --rom PATH --out NEW_DIR [--scope fire|handle] [--fault yes]\nRuns in a separate process with a 30-second deadline. Default fire compares one original entry on a fixed 441-byte projection; handle compares all 34 captured updates independently on 801 bytes. Both require separate full C/Rust WRAM equality; original hardware and live replay parity are not claimed.";

struct Options {
    lsnes: PathBuf,
    rom: PathBuf,
    movie: PathBuf,
    libraries: PathBuf,
    out: PathBuf,
    prepare_only: bool,
}
impl Options {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        if args.next().as_deref() != Some("capture") {
            return Err(USAGE.into());
        }
        let mut values = BTreeMap::new();
        while let Some(key) = args.next() {
            if !["--lsnes", "--rom", "--movie", "--libraries", "--out", "--prepare-only"].contains(&key.as_str()) {
                return Err(format!("unknown option {key}\n{USAGE}"));
            }
            let value =
                args.next().filter(|v| !v.starts_with("--")).ok_or_else(|| format!("missing value for {key}"))?;
            if values.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate option {key}"));
            }
        }
        let path = |key: &str| values.get(key).map(PathBuf::from).ok_or_else(|| format!("missing {key}"));
        let prepare_only = match values.get("--prepare-only").map(String::as_str).unwrap_or("no") {
            "yes" => true,
            "no" => false,
            _ => return Err("prepare-only must be yes or no".into()),
        };
        Ok(Self {
            lsnes: path("--lsnes")?,
            rom: path("--rom")?,
            movie: path("--movie")?,
            libraries: path("--libraries")?,
            out: path("--out")?,
            prepare_only,
        })
    }
}
fn main() -> ExitCode {
    if matches!(std::env::args().nth(1).as_deref(), Some("--help" | "-h")) {
        println!("{USAGE}\n\n{COMPARE_USAGE}");
        return ExitCode::SUCCESS;
    }
    let result = match std::env::args().nth(1).as_deref() {
        Some("compare") => supervise_compare(),
        Some("__compare") => compare(),
        _ => Options::parse().and_then(run),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
fn compare() -> Result<()> {
    let mut args = std::env::args().skip(2);
    let mut values = BTreeMap::new();
    while let Some(key) = args.next() {
        if !["--capture", "--library", "--rom", "--out", "--fault", "--scope"].contains(&key.as_str()) {
            return Err(format!("unknown option {key}\n{COMPARE_USAGE}"));
        }
        let value = args.next().filter(|v| !v.starts_with("--")).ok_or_else(|| format!("missing {key}"))?;
        if values.insert(key.clone(), value).is_some() {
            return Err(format!("duplicate option {key}"));
        }
    }
    let path = |key: &str| values.get(key).map(Path::new).ok_or_else(|| format!("missing {key}"));
    let fault = match values.get("--fault").map(String::as_str).unwrap_or("no") {
        "yes" => true,
        "no" => false,
        _ => return Err("fault must be yes or no".into()),
    };
    let compare = match values.get("--scope").map(String::as_str).unwrap_or("fire") {
        "fire" => fixture::compare,
        "handle" => handle_fixture::compare,
        _ => return Err("scope must be fire or handle".into()),
    };
    compare(path("--capture")?, path("--library")?, path("--rom")?, path("--out")?, fault)
}
fn supervise_compare() -> Result<()> {
    let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .arg("__compare")
        .args(std::env::args_os().skip(2))
        .stdin(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|e| e.to_string())?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() { Ok(()) } else { Err(format!("fixture failed: {status}")) };
            }
            Ok(None) if start.elapsed() < Duration::from_secs(30) => std::thread::sleep(Duration::from_millis(20)),
            other => {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                // SAFETY: child leads a dedicated process group created above.
                unsafe {
                    kill(-(child.id() as i32), 9);
                }
                let _ = child.wait();
                return Err(match other {
                    Err(error) => error.to_string(),
                    _ => "fixture exceeded 30-second deadline".into(),
                });
            }
        }
    }
}
fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hash(path: &Path) -> Result<String> {
    let mut input = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut digest = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let count = input.read(&mut bytes).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&bytes[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn authenticate(path: &Path, expected: &str) -> Result<PathBuf> {
    let path = path.canonicalize().map_err(|e| format!("{}: {e}", path.display()))?;
    if !path.is_file() || hash(&path)? != expected {
        return Err(format!("pinned identity mismatch: {}", path.display()));
    }
    Ok(path)
}
fn new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut output = File::options().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
    output.write_all(bytes).map_err(|e| e.to_string())?;
    output.sync_all().map_err(|e| e.to_string())
}
fn seal(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).map_err(|e| e.to_string())
}
fn table_file(path: &Path, table: &Table) -> Result<()> {
    new_file(path, toml::to_string_pretty(table).map_err(|e| e.to_string())?.as_bytes())?;
    seal(path)
}
fn file_map(values: BTreeMap<String, String>) -> toml::Value {
    toml::Value::Table(values.into_iter().map(|(k, v)| (k, v.into())).collect())
}
fn retained_artifacts(out: &Path) -> Result<BTreeMap<String, String>> {
    let mut artifacts = BTreeMap::new();
    for name in ["events.csv", "completion.txt", "stdout.log", "stderr.log", "failure.txt"] {
        let path = out.join(name);
        if path.exists() {
            artifacts.insert(name.into(), hash(&path)?);
            seal(&path)?;
        }
    }
    for directory in ["states", "diagnostics"] {
        if !out.join(directory).exists() {
            continue;
        }
        for entry in fs::read_dir(out.join(directory)).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
                return Err("unexpected non-file in snapshot inventory".into());
            }
            let path = entry.path();
            artifacts.insert(format!("{directory}/{}", path.file_name().unwrap().to_string_lossy()), hash(&path)?);
            seal(&path)?;
        }
    }
    Ok(artifacts)
}
fn library_hashes(directory: &Path) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.file_name().unwrap().to_string_lossy().contains(".so") && path.is_file() {
            result.insert(path.file_name().unwrap().to_string_lossy().into_owned(), hash(&path)?);
        }
    }
    if result.is_empty() {
        return Err("empty shared-library directory".into());
    }
    Ok(result)
}
fn run(options: Options) -> Result<()> {
    let lsnes = authenticate(&options.lsnes, LSNES_SHA256)?;
    let rom = authenticate(&options.rom, ROM_SHA256)?;
    let movie = authenticate(&options.movie, MOVIE_SHA256)?;
    let libraries = options.libraries.canonicalize().map_err(|e| e.to_string())?;
    let dependencies = library_hashes(&libraries)?;
    fs::create_dir(&options.out).map_err(|e| format!("output must be fresh: {}: {e}", options.out.display()))?;
    let out = options.out.canonicalize().map_err(|e| e.to_string())?;
    let execution_started = Instant::now();
    let result = execute(&out, &lsnes, &rom, &movie, &libraries, &dependencies, options.prepare_only);
    if let Err(error) = &result
        && !out.join("result.toml").exists()
    {
        let mut failure = Table::new();
        failure.insert("schema".into(), "sm-original-result-v1".into());
        failure.insert("status".into(), "failed".into());
        failure.insert("error".into(), error.clone().into());
        failure.insert("elapsed_seconds".into(), execution_started.elapsed().as_secs_f64().into());
        failure.insert("elapsed_scope".into(), "preparation, emulator execution and validation".into());
        failure.insert("original_equivalence".into(), "not-tested".into());
        if let Ok(digest) = hash(&out.join("request.toml")) {
            failure.insert("request_sha256".into(), digest.into());
        }
        if let Ok(artifacts) = retained_artifacts(&out) {
            failure.insert("artifacts".into(), file_map(artifacts));
        }
        let _ = table_file(&out.join("result.toml"), &failure);
    }
    result
}

fn execute(
    out: &Path,
    lsnes: &Path,
    rom: &Path,
    source_movie: &Path,
    libraries: &Path,
    dependencies: &BTreeMap<String, String>,
    prepare_only: bool,
) -> Result<()> {
    let authored = out.join("scenario.lsmv");
    let members = movie::author(source_movie, &authored)?;
    seal(&authored)?;
    let observer_path = out.join("observe.lua");
    new_file(&observer_path, observer::generate(out)?.as_bytes())?;
    seal(&observer_path)?;
    fs::create_dir(out.join("states")).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("diagnostics")).map_err(|e| e.to_string())?;
    fs::create_dir(out.join("lsnes-config")).map_err(|e| e.to_string())?;
    let arguments = vec![
        "--dumper=INTERNAL-NULL".to_owned(),
        format!("--rom-a={}", rom.display()),
        format!("--length={}", movie::FRAMES),
        format!("--lua={}", observer_path.display()),
        authored.to_string_lossy().into_owned(),
    ];
    let mut request = Table::new();
    request.insert("schema".into(), "sm-original-request-v1".into());
    request.insert("scenario".into(), movie::SCENARIO.into());
    request.insert("observation".into(), "reset-ledger-first-shot-cpu-wram-hardware-v2".into());
    request.insert(
        "diagnostic_snapshots".into(),
        "frame 8648 before Right and first actual consumed X; separate from call proof".into(),
    );
    request.insert("turn_button".into(), "Right".into());
    request.insert("turn_frame".into(), (movie::TURN as i64).into());
    request.insert(
        "fire_context".into(),
        "pose 01, direction 08, standing movement 00, X 130, Y 75, radius 21, zero vertical velocity".into(),
    );
    request.insert("requested_frames".into(), (movie::FRAMES as i64).into());
    request.insert("preserved_input_frames".into(), (movie::PREFIX as i64).into());
    request.insert("physical_button".into(), movie::BUTTON.into());
    request.insert("button_frame".into(), (movie::FIRE as i64).into());
    request.insert("hardware_button_word".into(), 64_i64.into());
    request.insert("deadline_seconds".into(), 120_i64.into());
    request.insert("snapshot_limit".into(), 512_i64.into());
    request.insert("lsnes".into(), lsnes.to_string_lossy().into_owned().into());
    request.insert("lsnes_sha256".into(), LSNES_SHA256.into());
    request.insert("rom".into(), rom.to_string_lossy().into_owned().into());
    request.insert("rom_sha256".into(), ROM_SHA256.into());
    request.insert("source_movie".into(), source_movie.to_string_lossy().into_owned().into());
    request.insert("source_movie_sha256".into(), MOVIE_SHA256.into());
    request.insert("authored_movie_sha256".into(), hash(&authored)?.into());
    request.insert("observer_sha256".into(), hash(&observer_path)?.into());
    request.insert("command_sha256".into(), hash(&std::env::current_exe().map_err(|e| e.to_string())?)?.into());
    request.insert("argv".into(), arguments.iter().cloned().map(toml::Value::String).collect::<Vec<_>>().into());
    request.insert("library_directory".into(), libraries.to_string_lossy().into_owned().into());
    request.insert("library_hashes".into(), file_map(dependencies.clone()));
    request.insert("movie_member_hashes".into(), file_map(members));
    request.insert(
        "reset_contract".into(),
        "No savestate, SRAM or volatile RAM members; preserved movie RTC 1000000000:0".into(),
    );
    table_file(&out.join("request.toml"), &request)?;
    if prepare_only {
        let mut result = Table::new();
        result.insert("schema".into(), "sm-original-result-v1".into());
        result.insert("status".into(), "prepared-only".into());
        result.insert("request_sha256".into(), hash(&out.join("request.toml"))?.into());
        result.insert("original_equivalence".into(), "not-tested".into());
        table_file(&out.join("result.toml"), &result)?;
        println!("prepared {}", out.display());
        return Ok(());
    }
    let start = Instant::now();
    let mut child = Command::new(lsnes)
        .args(&arguments)
        .current_dir(out)
        .env("LD_LIBRARY_PATH", libraries)
        .env("XDG_CONFIG_HOME", out.join("lsnes-config"))
        .env_remove("LD_PRELOAD")
        .env_remove("LUA_PATH")
        .env_remove("LUA_CPATH")
        .stdin(Stdio::null())
        .stdout(File::options().write(true).create_new(true).open(out.join("stdout.log")).map_err(|e| e.to_string())?)
        .stderr(File::options().write(true).create_new(true).open(out.join("stderr.log")).map_err(|e| e.to_string())?)
        .process_group(0)
        .spawn()
        .map_err(|e| e.to_string())?;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < Duration::from_secs(120) => std::thread::sleep(Duration::from_millis(20)),
            other => {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                // SAFETY: child was created as leader of a dedicated process group.
                unsafe {
                    kill(-(child.id() as i32), 9);
                }
                let _ = child.wait();
                return Err(match other {
                    Err(e) => e.to_string(),
                    _ => "original emulator exceeded 120-second deadline".into(),
                });
            }
        }
    };
    if !status.success() {
        return Err(format!("original emulator failed: {status}; see retained logs"));
    }
    authenticate(lsnes, LSNES_SHA256)?;
    authenticate(rom, ROM_SHA256)?;
    authenticate(source_movie, MOVIE_SHA256)?;
    if dependencies != &library_hashes(libraries)?
        || request["authored_movie_sha256"].as_str() != Some(hash(&authored)?.as_str())
        || request["observer_sha256"].as_str() != Some(hash(&observer_path)?.as_str())
    {
        return Err("capture inputs changed during execution".into());
    }
    let mut result = validate::capture(out)?;
    result.insert("schema".into(), "sm-original-result-v1".into());
    result.insert("status".into(), "captured".into());
    result.insert("original_equivalence".into(), "not-tested".into());
    result.insert(
        "scope".into(),
        "Observed original first-shot invocation states and effects; no C/Rust fixture or live equivalence verdict"
            .into(),
    );
    result.insert("elapsed_seconds".into(), start.elapsed().as_secs_f64().into());
    result.insert("elapsed_scope".into(), "emulator execution and validation".into());
    result.insert("request_sha256".into(), hash(&out.join("request.toml"))?.into());
    result.insert("artifacts".into(), file_map(retained_artifacts(out)?));
    table_file(&out.join("result.toml"), &result)?;
    println!("captured {}", out.display());
    Ok(())
}
