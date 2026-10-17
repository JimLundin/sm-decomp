use crate::energy::Energy;
use crate::missile::Missile;
use crate::morph::Morph;
use crate::progression::Parlor;
use crate::route::Route;
use sha2::{Digest, Sha256};
use sm_binary_evidence::ToEvidence;
use sm_native_host::{Game, State};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, String>;
const PARLOR_PREFIX_STEPS: usize = 16991;
const PARLOR_TAIL_STEPS: usize = 4096;
const PARLOR_PREFIX_SHA256: &str = "39098a57ed3e51787b684da80c54561d81a46600b4931dc012d747c9e7c097f0";
const MORPH_PREFIX_STEPS: usize = 18072;
const MORPH_TAIL_STEPS: usize = 8192;
const MORPH_PREFIX_SHA256: &str = "125656a5c9d6bd040472d07ea23ac72f93c72ff6c54c092a61bb662b298b57bb";
const MISSILE_PREFIX_STEPS: usize = 23036;
const MISSILE_TAIL_STEPS: usize = 8192;
const MISSILE_PREFIX_SHA256: &str = "0a8ab6a50be742c1124aea5b802ae088a3b12cd785b7da54aea6cfc6143ae76a";
const ENERGY_PREFIX_STEPS: usize = 24832;
const ENERGY_TAIL_STEPS: usize = 16384;
const ENERGY_PREFIX_SHA256: &str = "43d67592cb3da2a9dd9c1568f609aa19b60bcb555a28b643453427677908be69";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TapeContract {
    scenario: &'static str,
    prefix: usize,
    tail: usize,
    sha256: &'static str,
}
fn tape_contract(scenario: &str) -> Option<TapeContract> {
    match scenario {
        "parlor" => Some(TapeContract {
            scenario: "parlor",
            prefix: PARLOR_PREFIX_STEPS,
            tail: PARLOR_TAIL_STEPS,
            sha256: PARLOR_PREFIX_SHA256,
        }),
        "morph" => Some(TapeContract {
            scenario: "morph",
            prefix: MORPH_PREFIX_STEPS,
            tail: MORPH_TAIL_STEPS,
            sha256: MORPH_PREFIX_SHA256,
        }),
        "missile" => Some(TapeContract {
            scenario: "missile",
            prefix: MISSILE_PREFIX_STEPS,
            tail: MISSILE_TAIL_STEPS,
            sha256: MISSILE_PREFIX_SHA256,
        }),
        "rio-return" => Some(TapeContract {
            scenario: "rio-return",
            prefix: ENERGY_PREFIX_STEPS,
            tail: 4096,
            sha256: ENERGY_PREFIX_SHA256,
        }),
        "energy" => Some(TapeContract {
            scenario: "energy",
            prefix: ENERGY_PREFIX_STEPS,
            tail: ENERGY_TAIL_STEPS,
            sha256: ENERGY_PREFIX_SHA256,
        }),
        _ => None,
    }
}

struct Options {
    command: String,
    values: BTreeMap<String, OsString>,
}
impl Options {
    fn parse(args: Vec<OsString>) -> Result<Self> {
        let mut args = args.into_iter();
        let command = args.next().ok_or("command required")?.into_string().map_err(|_| "invalid command")?;
        let mut values = BTreeMap::new();
        while let Some(key) = args.next() {
            let key = key.into_string().map_err(|_| "invalid option")?;
            if !matches!(
                key.as_str(),
                "--rom"
                    | "--reference-archive"
                    | "--out"
                    | "--library"
                    | "--steps"
                    | "--deadline-seconds"
                    | "--scenario"
                    | "--inputs"
                    | "--sdl-library"
                    | "--present"
                    | "--capture-every"
                    | "--gameplay"
                    | "--tail-policy"
                    | "--item-instructions"
                    | "--morph-instructions"
                    | "--tank-instructions"
                    | "--player-motion"
                    | "--persistence"
                    | "--waver"
                    | "--ripper"
                    | "--rio"
                    | "--rio-trace-start"
                    | "--rio-trace-steps"
                    | "--trace"
                    | "--trace-reference"
                    | "--trace-reference-sha256"
            ) {
                return Err(format!("unknown option {key}"));
            }
            let value = args.next().ok_or_else(|| format!("{key} needs a value"))?;
            if value.to_string_lossy().starts_with('-') {
                return Err(format!("{key} needs a value; prefix paths starting with '-' with './'"));
            }
            if values.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate {key}"));
            }
        }
        Ok(Self { command, values })
    }
    fn path(&self, key: &str) -> Result<PathBuf> {
        self.values.get(key).map(PathBuf::from).ok_or_else(|| format!("missing {key}"))
    }
    fn text<'a>(&'a self, key: &str, default: &'a str) -> Result<&'a str> {
        match self.values.get(key) {
            Some(s) => s.to_str().ok_or_else(|| format!("{key} is not UTF-8")),
            None => Ok(default),
        }
    }
    fn number(&self, key: &str, default: u64) -> Result<u64> {
        match self.values.get(key) {
            Some(s) => s
                .to_str()
                .and_then(|s| s.parse::<u64>().ok())
                .filter(|n| *n > 0)
                .ok_or_else(|| format!("invalid positive {key}")),
            None => Ok(default),
        }
    }
}

fn persistence_owner(options: &Options) -> Result<&str> {
    let owner = options.text("--persistence", options.text("--gameplay", "c")?)?;
    if matches!(owner, "c" | "rust") { Ok(owner) } else { Err("persistence must be c or rust".into()) }
}

pub fn validate(args: Vec<OsString>) -> Result<()> {
    let options = Options::parse(args)?;
    options.path("--out")?;
    if options.command == "prepare" {
        options.path("--reference-archive")?;
        if options.values.keys().any(|k| !matches!(k.as_str(), "--reference-archive" | "--out")) {
            return Err("prepare accepts only --reference-archive and --out".into());
        }
    } else {
        options.path("--rom")?;
        options.path("--library")?;
        if options.values.contains_key("--reference-archive") {
            return Err("--reference-archive is only valid for prepare".into());
        }
        if options.command == "play" && options.values.contains_key("--present") {
            return Err("play already selects interactive presentation".into());
        }
        for key in ["--steps", "--deadline-seconds", "--capture-every"] {
            options.number(key, 1)?;
        }
        if !matches!(
            options.text("--scenario", "boot")?,
            "boot" | "ceres" | "none" | "parlor" | "morph" | "missile" | "energy" | "rio-return"
        ) {
            return Err("scenario must be boot, ceres, parlor, morph, missile, energy, rio-return or none".into());
        }
        if !matches!(options.text("--gameplay", "c")?, "c" | "rust") {
            return Err("gameplay must be c or rust".into());
        }
        if !matches!(options.text("--item-instructions", "c")?, "c" | "rust") {
            return Err("item-instructions must be c or rust".into());
        }
        if !matches!(options.text("--morph-instructions", "c")?, "c" | "rust") {
            return Err("morph-instructions must be c or rust".into());
        }
        if !matches!(options.text("--tank-instructions", "c")?, "c" | "rust") {
            return Err("tank-instructions must be c or rust".into());
        }
        if !matches!(options.text("--player-motion", "c")?, "c" | "rust") {
            return Err("player-motion must be c or rust".into());
        }
        persistence_owner(&options)?;
        if !matches!(options.text("--waver", "c")?, "c" | "rust") {
            return Err("waver must be c or rust".into());
        }
        if !matches!(options.text("--ripper", "c")?, "c" | "rust") {
            return Err("ripper must be c or rust".into());
        }
        if !matches!(options.text("--rio", "c")?, "c" | "rust") {
            return Err("rio must be c or rust".into());
        }
        if !matches!(options.text("--present", "no")?, "no" | "interactive" | "replay") {
            return Err("present must be no, interactive or replay".into());
        }
        tape_mode(&options)?;
        rio_trace_interval(&options)?;
        trace_mode(&options)?;
    }
    Ok(())
}

fn trace_mode(options: &Options) -> Result<bool> {
    let enabled = match options.text("--trace", "no")? {
        "yes" => true,
        "no" => false,
        _ => return Err("trace must be yes or no".into()),
    };
    let reference = options.values.contains_key("--trace-reference");
    let seal = options.values.contains_key("--trace-reference-sha256");
    if reference != seal || (reference && !enabled) {
        return Err("trace comparison requires --trace yes and both reference path and SHA256".into());
    }
    if seal {
        let hash = options.text("--trace-reference-sha256", "")?;
        if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err("trace reference SHA256 must be 64 lowercase hexadecimal digits".into());
        }
    }
    if enabled {
        options.path("--inputs")?;
        options.number("--steps", 1)?;
        if !options.values.contains_key("--steps")
            || options.values.contains_key("--tail-policy")
            || options.command == "play"
            || options.text("--present", "no")? != "no"
        {
            return Err("trace requires explicit steps, recorded inputs, no authored tail and no presentation".into());
        }
    }
    Ok(enabled)
}

fn rio_trace_interval(options: &Options) -> Result<Option<(u64, u64)>> {
    match (options.values.contains_key("--rio-trace-start"), options.values.contains_key("--rio-trace-steps")) {
        (false, false) => Ok(None),
        (true, true) => {
            options.path("--inputs").map_err(|_| "Rio capture requires a recorded --inputs tape")?;
            if options.values.contains_key("--tail-policy") {
                return Err("Rio capture requires recorded replay without an authored tail".into());
            }
            if options.command == "play" || options.text("--present", "no")? == "interactive" {
                return Err("Rio capture requires noninteractive recorded replay".into());
            }
            let start = options.number("--rio-trace-start", 1)?;
            let count = options.number("--rio-trace-steps", 1)?;
            if count > 2048 || start.checked_add(count - 1).is_none() {
                return Err("Rio trace requires at most 2048 steps and a nonoverflowing interval".into());
            }
            Ok(Some((start, count)))
        }
        _ => Err("--rio-trace-start and --rio-trace-steps must be supplied together".into()),
    }
}

/// A finite recorded prefix plus either recorded replay or bounded authoring.
/// Input bytes are loaded and validated before Game exists, and replay EOF is
/// never permission to switch to a policy.
struct RecordedInputs {
    contract: TapeContract,
    pads: Vec<u16>,
    records: Vec<String>,
    source: PathBuf,
    sha256: String,
    author: bool,
    consumed: usize,
}
fn tape_mode(options: &Options) -> Result<Option<(TapeContract, bool)>> {
    let contract = tape_contract(options.text("--scenario", "boot")?);
    let author = options.values.contains_key("--tail-policy");
    if author && contract.map(|c| c.scenario) != Some(options.text("--tail-policy", "")?) {
        return Err("--tail-policy must match scenario parlor, morph, missile or energy or rio-return".into());
    }
    let Some(contract) = contract else {
        return Ok(None);
    };
    if options.command == "play" || options.text("--present", "no")? == "interactive" {
        return Err(format!("{} authoring/replay requires noninteractive presentation", contract.scenario));
    }
    options.path("--inputs")?;
    Ok(Some((contract, author)))
}
fn step_limit(options: &Options) -> Result<u64> {
    let contract = tape_contract(options.text("--scenario", "boot")?);
    let default = if let Some(contract) = contract {
        (contract.prefix + contract.tail) as u64
    } else if options.text("--present", "no")? == "interactive" {
        216000
    } else {
        20000
    };
    let requested = options.number("--steps", default)?;
    Ok(if let Some(contract) = contract { requested.min((contract.prefix + contract.tail) as u64) } else { requested })
}
fn deadline_seconds(options: &Options) -> Result<u64> {
    if matches!(options.text("--scenario", "boot")?, "energy" | "rio-return") {
        Ok(options.number("--deadline-seconds", 90)?.min(90))
    } else {
        options.number("--deadline-seconds", if options.command == "play" { 3600 } else { 60 })
    }
}
impl RecordedInputs {
    fn load(path: &Path, contract: TapeContract, author: bool) -> Result<Self> {
        use std::io::Read;
        let source = path.canonicalize().map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        File::open(&source)
            .map_err(|e| e.to_string())?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("recorded input exceeds 16 MiB".into());
        }
        let mut inputs = Self::parse(&bytes, contract, author)?;
        inputs.source = source;
        Ok(inputs)
    }
    fn parse(bytes: &[u8], contract: TapeContract, author: bool) -> Result<Self> {
        let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        let maximum = if author { contract.prefix } else { contract.prefix + contract.tail };
        if !(contract.prefix..=maximum).contains(&text.split_inclusive('\n').count()) {
            return Err(format!(
                "{} {} requires {}..={maximum} recorded pads",
                contract.scenario,
                if author { "authoring" } else { "replay" },
                contract.prefix
            ));
        }
        let records: Vec<_> = text.split_inclusive('\n').map(str::to_owned).collect();
        let prefix_bytes: usize = records[..contract.prefix].iter().map(String::len).sum();
        let prefix_hash = format!("{:x}", Sha256::digest(&bytes[..prefix_bytes]));
        if prefix_hash != contract.sha256 {
            return Err(format!(
                "{} input does not preserve the exact frozen {}-pad prefix",
                contract.scenario, contract.prefix
            ));
        }
        let pads = records.iter().map(|record| parse_pad(record)).collect::<Result<Vec<_>>>()?;
        Ok(Self {
            contract,
            pads,
            records,
            source: PathBuf::new(),
            sha256: format!("{:x}", Sha256::digest(bytes)),
            author,
            consumed: 0,
        })
    }
    fn next(&mut self) -> Result<Option<(u16, String)>> {
        if let Some(&pad) = self.pads.get(self.consumed) {
            let record = self.records[self.consumed].clone();
            self.consumed += 1;
            Ok(Some((pad, record)))
        } else if self.author {
            Ok(None)
        } else {
            Err(format!("input EOF before {} checkpoint; replay never invokes tail policy", self.contract.scenario))
        }
    }
    fn report(&self, result: &mut sm_binary_evidence::Map, completed: u64, limit: u64) {
        result.insert("input_source".into(), self.source.to_string_lossy().into_owned().into());
        result.insert("input_source_sha256".into(), self.sha256.clone().into());
        result.insert("input_source_steps".into(), (self.pads.len() as i64).into());
        result.insert(
            "input_authoring_mode".into(),
            if self.author {
                format!("frozen-prefix-plus-{}-policy", self.contract.scenario)
            } else {
                "recorded-replay-no-policy".into()
            }
            .into(),
        );
        result.insert("input_prefix_sha256".into(), self.contract.sha256.into());
        result.insert("input_prefix_steps".into(), (self.contract.prefix as i64).into());
        result
            .insert("input_prefix_completed_steps".into(), (completed.min(self.contract.prefix as u64) as i64).into());
        result.insert(
            "input_tail_completed_steps".into(),
            (completed.saturating_sub(self.contract.prefix as u64) as i64).into(),
        );
        result.insert("input_tail_limit".into(), (self.contract.tail as i64).into());
        result.insert("effective_step_limit".into(), (limit as i64).into());
    }
}

fn retain_sources(out: &Path) -> Result<()> {
    fs::create_dir(out.join("source")).map_err(|e| e.to_string())?;
    for (name, source) in [
        ("binary-evidence-lib.rs", include_str!("../evidence/src/lib.rs")),
        ("binary-evidence-Cargo.toml", include_str!("../evidence/Cargo.toml")),
        ("trace-lib.rs", include_str!("../trace/src/lib.rs")),
        ("trace-Cargo.toml", include_str!("../trace/Cargo.toml")),
        ("native-evidence.rs", include_str!("../native-host/src/evidence.rs")),
        ("route.rs", include_str!("route.rs")),
        ("progression.rs", include_str!("progression.rs")),
        ("morph.rs", include_str!("morph.rs")),
        ("missile.rs", include_str!("missile.rs")),
        ("rio_return.rs", include_str!("rio_return.rs")),
        ("energy.rs", include_str!("energy.rs")),
        ("runtime_cli.rs", include_str!("runtime_cli.rs")),
        ("replay_trace.rs", include_str!("replay_trace.rs")),
        ("rio_capture.rs", include_str!("rio_capture.rs")),
        ("rio_observation.rs", include_str!("rio_observation.rs")),
        ("gameplay.rs", include_str!("gameplay.rs")),
        ("persistence_runtime.rs", include_str!("persistence_runtime.rs")),
        ("persistence-lib.rs", include_str!("../persistence/src/lib.rs")),
        ("persistence-tables.rs", include_str!("../persistence/src/tables.rs")),
        ("player-motion-lib.rs", include_str!("../player-motion/src/lib.rs")),
        ("player-motion-memory.rs", include_str!("../player-motion/src/memory.rs")),
        ("player-motion-vertical.rs", include_str!("../player-motion/src/vertical.rs")),
        ("player-motion-solid-enemy.rs", include_str!("../player-motion/src/solid_enemy.rs")),
        ("player-motion-feedback.rs", include_str!("../player-motion/src/feedback.rs")),
        ("waver-lib.rs", include_str!("../waver/src/lib.rs")),
        ("waver-memory.rs", include_str!("../waver/src/memory.rs")),
        ("waver-actor.rs", include_str!("../waver/src/actor.rs")),
        ("ripper-lib.rs", include_str!("../ripper/src/lib.rs")),
        ("ripper-memory.rs", include_str!("../ripper/src/memory.rs")),
        ("ripper-patrol.rs", include_str!("../ripper/src/patrol.rs")),
        ("rio-lib.rs", include_str!("../rio/src/lib.rs")),
        ("rio-memory.rs", include_str!("../rio/src/memory.rs")),
        ("rio-actor.rs", include_str!("../rio/src/actor.rs")),
        ("rio-sine.rs", include_str!("../rio/src/sine.rs")),
        ("rio-service-writes.rs", include_str!("../rio/src/service_writes.rs")),
        ("rio-service-protocol.rs", include_str!("../rio/src/service_protocol.rs")),
        ("item-instructions-lib.rs", include_str!("../item-instructions/src/lib.rs")),
        ("item-instructions-memory.rs", include_str!("../item-instructions/src/memory.rs")),
        ("item-instructions-morph.rs", include_str!("../item-instructions/src/morph.rs")),
        ("item-instructions-tanks.rs", include_str!("../item-instructions/src/tanks.rs")),
        ("main.rs", include_str!("main.rs")),
    ] {
        fs::write(out.join("source").join(name), source).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Synchronous entry for a dedicated externally supervised process.
/// Supports preparation or the same worker used by the CLI. Worker output must
/// be fresh; all input paths must be absolute. The caller owns process lifetime,
/// stdout/stderr retention, outer deadline and actual DLL/launcher identities.
/// Native state and working-directory changes prohibit reuse of the process.
pub fn run_external(args: Vec<OsString>) -> Result<()> {
    #[cfg(feature = "native-preparation")]
    if args.first().is_some_and(|command| command == "room-transition-control") {
        if args.len() != 6 {
            return Err("room-transition-control requires ARCHIVE ROM OUTPUT FIXTURE_WRAM MODE".into());
        }
        let paths: Vec<_> = args[1..5].iter().map(PathBuf::from).collect();
        if paths.iter().any(|path| !path.is_absolute()) {
            return Err("room-transition-control paths must be absolute".into());
        }
        return sm_native_host::room_transition_control::run(
            &paths[0],
            &paths[1],
            &paths[2],
            &paths[3],
            args[5].to_str().ok_or("room-transition mode must be UTF-8")?,
        );
    }
    #[cfg(feature = "native-preparation")]
    if args.first().is_some_and(|command| command == "door-scrolling-control") {
        if args.len() != 6 {
            return Err("door-scrolling-control requires ARCHIVE ROM OUTPUT FIXTURE_WRAM MODE".into());
        }
        let paths: Vec<_> = args[1..5].iter().map(PathBuf::from).collect();
        if paths.iter().any(|path| !path.is_absolute()) {
            return Err("door-scrolling-control paths must be absolute".into());
        }
        return sm_native_host::door_scrolling_control::run(
            &paths[0],
            &paths[1],
            &paths[2],
            &paths[3],
            args[5].to_str().ok_or("door-scrolling mode must be UTF-8")?,
        );
    }
    #[cfg(feature = "native-preparation")]
    if args.first().is_some_and(|command| command == "camera-control") {
        if args.len() != 6 {
            return Err("camera-control requires ARCHIVE ROM OUTPUT FIXTURE_WRAM MODE".into());
        }
        let paths: Vec<_> = args[1..5].iter().map(PathBuf::from).collect();
        if paths.iter().any(|path| !path.is_absolute()) {
            return Err("camera-control paths must be absolute".into());
        }
        return sm_native_host::camera_control::run(
            &paths[0],
            &paths[1],
            &paths[2],
            &paths[3],
            args[5].to_str().ok_or("camera mode must be UTF-8")?,
        );
    }
    #[cfg(feature = "native-preparation")]
    if args.first().is_some_and(|command| command == "native-smoke") {
        if args.len() != 4 {
            return Err("native-smoke requires ARCHIVE ROM OUTPUT".into());
        }
        let paths: Vec<_> = args[1..].iter().map(PathBuf::from).collect();
        if paths.iter().any(|path| !path.is_absolute()) {
            return Err("native-smoke paths must be absolute".into());
        }
        return sm_native_host::smoke_control::run(&paths[0], &paths[1], &paths[2]);
    }
    #[cfg(feature = "native-preparation")]
    if args.first().is_some_and(|command| command == "prepare-control") {
        if args.len() != 5 {
            return Err("prepare-control requires ARCHIVE FRESH_OUTPUT RUNTIME PRODUCER_DLL".into());
        }
        let paths: Vec<_> = args[1..].iter().map(PathBuf::from).collect();
        if paths.iter().any(|path| !path.is_absolute()) {
            return Err("prepare-control paths must be absolute".into());
        }
        return sm_native_host::preparation_control::run(&paths[0], &paths[1], &paths[2], &paths[3]);
    }
    #[cfg(feature = "private-checks")]
    if args.first().is_some_and(|command| command == "private-checks") {
        return crate::private_checks::run(&args);
    }
    validate(args.clone())?;
    let options = Options::parse(args.clone())?;
    for key in ["--out", "--rom", "--library", "--reference-archive", "--inputs", "--sdl-library", "--trace-reference"]
    {
        if options.values.contains_key(key) && !options.path(key)?.is_absolute() {
            return Err(format!("external entry requires absolute {key}"));
        }
    }
    if options.command == "prepare" {
        return run(args);
    }
    if options.command != "run-worker" {
        return Err("external entry accepts prepare or run-worker only".into());
    }
    let out = options.path("--out")?;
    fs::create_dir(&out).map_err(|e| format!("create fresh run {}: {e}", out.display()))?;
    retain_sources(&out)?;
    worker(&options)
}
pub fn run(args: Vec<OsString>) -> Result<()> {
    let options = Options::parse(args.clone())?;
    if options.command == "prepare" {
        let prepared = sm_native_host::prepare(&options.path("--reference-archive")?, &options.path("--out")?)?;
        println!("Prepared native library: {}", prepared.library.display());
        return Ok(());
    }
    if options.command == "run-worker" {
        return worker(&options);
    }
    let scenario_inputs = tape_mode(&options)?
        .map(|(contract, author)| RecordedInputs::load(&options.path("--inputs")?, contract, author))
        .transpose()?;
    let out = options.path("--out")?;
    options.path("--rom")?;
    options.path("--library")?;
    let seconds = deadline_seconds(&options)?;
    fs::create_dir(&out).map_err(|e| format!("create fresh run {}: {e}", out.display()))?;
    retain_sources(&out)?;
    let mut child_args = args;
    child_args[0] = "run-worker".into();
    if options.command == "play" {
        child_args.extend(["--present".into(), "interactive".into()]);
    }
    let mut manifest = sm_binary_evidence::Map::new();
    manifest.insert("schema".into(), "sm-native-run-v1".into());
    manifest.insert("status".into(), "running".into());
    manifest.insert("original_rom_comparison".into(), "not-tested".into());
    manifest.insert("deadline_seconds".into(), sm_binary_evidence::Value::Integer(seconds as i64));
    manifest.insert(
        "argv".into(),
        sm_binary_evidence::Value::Array(
            child_args.iter().map(|s| sm_binary_evidence::Value::String(s.to_string_lossy().into_owned())).collect(),
        ),
    );
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    manifest.insert("executable_sha256".into(), hash_file(&exe)?.into());
    manifest.insert("library_sha256".into(), hash_file(&options.path("--library")?)?.into());
    manifest.insert("rom_sha256".into(), hash_file(&options.path("--rom")?)?.into());
    if options.values.contains_key("--inputs") {
        manifest.insert("raw_input_sha256".into(), hash_file(&options.path("--inputs")?)?.into());
    }
    if let Some(inputs) = &scenario_inputs {
        inputs.report(&mut manifest, 0, step_limit(&options)?);
    }
    write_evidence(&out.join("invocation.sme"), &manifest)?;
    let stdout = File::create(out.join("stdout.log")).map_err(|e| e.to_string())?;
    let stderr = File::create(out.join("stderr.log")).map_err(|e| e.to_string())?;
    let mut child = Command::new(exe)
        .args(child_args)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| e.to_string())?;
    let start = Instant::now();
    let result = supervise(&mut child, Duration::from_secs(seconds))
        .map_err(|error| format!("{error}; inspect retained observations and stderr.log in {}", out.display()));
    manifest.insert("status".into(), if result.is_ok() { "completed" } else { "failed" }.into());
    manifest.insert("elapsed_seconds".into(), sm_binary_evidence::Value::Float(start.elapsed().as_secs_f64()));
    if let Err(error) = &result {
        manifest.insert("error".into(), error.clone().into());
    }
    write_evidence(&out.join("invocation.sme"), &manifest)?;
    if out.join("result.sme").exists() {
        match sm_binary_evidence::read(out.join("result.sme")) {
            Ok(summary) => println!("{summary:?}"),
            Err(error) => {
                return Err(match result {
                    Ok(()) => format!("result evidence: {error}"),
                    Err(primary) => format!("{primary}; result evidence: {error}"),
                });
            }
        }
    }
    result
}

fn supervise(child: &mut Child, deadline: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() { Ok(()) } else { Err(format!("native child failed: {status}")) };
            }
            Ok(None) if start.elapsed() < deadline => std::thread::sleep(Duration::from_millis(25)),
            result => {
                let _ = child.kill();
                let reaped = child.wait();
                let error = match result {
                    Err(error) => format!("native child wait failed: {error}"),
                    _ => format!("native child exceeded {}s", deadline.as_secs_f64()),
                };
                return Err(match reaped {
                    Ok(_) => error,
                    Err(error_reaping) => format!("{error}; reaping failed: {error_reaping}"),
                });
            }
        }
    }
}

fn hash_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn write_evidence(path: &Path, value: &sm_binary_evidence::Map) -> Result<()> {
    fs::write(path, sm_binary_evidence::encode(&sm_binary_evidence::Value::Table(value.clone()))?)
        .map_err(|e| e.to_string())
}
#[cfg(feature = "png-output")]
fn screenshot(path: &Path, pixels: &[u32], width: usize, height: usize) -> Result<()> {
    if pixels.len() != width * height {
        return Err("pixel extent differs".into());
    }
    let out = BufWriter::new(File::create(path).map_err(|e| e.to_string())?);
    let mut encoder = png::Encoder::new(out, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = pixels.iter().flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8]).collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
#[cfg(not(feature = "png-output"))]
fn screenshot(path: &Path, pixels: &[u32], width: usize, height: usize) -> Result<()> {
    validate_ppm_extent(pixels, width, height)?;
    let out = BufWriter::new(File::create(path.with_extension("ppm")).map_err(|e| e.to_string())?);
    write_ppm(out, pixels, width, height)
}

#[cfg(not(feature = "png-output"))]
fn validate_ppm_extent(pixels: &[u32], width: usize, height: usize) -> Result<()> {
    if width == 0 || height == 0 || width.checked_mul(height) != Some(pixels.len()) {
        return Err("pixel extent differs".into());
    }
    Ok(())
}

#[cfg(not(feature = "png-output"))]
fn write_ppm(mut out: impl Write, pixels: &[u32], width: usize, height: usize) -> Result<()> {
    validate_ppm_extent(pixels, width, height)?;
    writeln!(out, "P6\n{width} {height}\n255").map_err(|e| e.to_string())?;
    for pixel in pixels {
        out.write_all(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8]).map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

#[cfg(all(not(feature = "png-output"), any(test, feature = "private-checks")))]
#[cfg_attr(test, test)]
pub(crate) fn ppm_writer_checks_rgb_extent_and_io_failures() {
    let pixels = [0xaa123456, 0xff789abc];
    let mut bytes = Vec::new();
    write_ppm(&mut bytes, &pixels, 2, 1).unwrap();
    assert_eq!(bytes, b"P6\n2 1\n255\n\x12\x34\x56\x78\x9a\xbc");
    for (width, height) in [(0, 2), (2, 0), (1, 1), (3, 1), (usize::MAX, 2)] {
        let mut unchanged = b"untouched".to_vec();
        assert_eq!(write_ppm(&mut unchanged, &pixels, width, height).unwrap_err(), "pixel extent differs");
        assert_eq!(unchanged, b"untouched");
    }
    for (width, height) in [(0, 2), (2, 0), (0, 0)] {
        let mut sentinel = b"unchanged".to_vec();
        assert!(write_ppm(&mut sentinel, &[], width, height).is_err());
        assert_eq!(sentinel, b"unchanged");
    }
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let directory = std::env::temp_dir().join(format!("sm-ppm-control-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let file_outcome = std::panic::catch_unwind(|| {
        let target = directory.join("existing.ppm");
        std::fs::write(&target, b"preserve").unwrap();
        assert!(screenshot(&target, &pixels, 1, 1).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"preserve");
        assert!(screenshot(&target, &[], 0, 1).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"preserve");
        assert!(screenshot(&directory.join("absent/child.png"), &pixels, 2, 1).is_err());
        screenshot(&directory.join("valid.png"), &pixels, 2, 1).unwrap();
        assert_eq!(std::fs::read(directory.join("valid.ppm")).unwrap(), bytes);
        assert!(!directory.join("valid.png").exists());
    });
    std::fs::remove_dir_all(&directory).unwrap_or_else(|error| panic!("remove {}: {error}", directory.display()));
    if let Err(payload) = file_outcome {
        std::panic::resume_unwind(payload);
    }
    struct Failure {
        remaining: usize,
        fail_flush: bool,
    }
    impl Write for Failure {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.remaining == 0 {
                return Err(std::io::Error::other("injected write failure"));
            }
            let count = bytes.len().min(self.remaining);
            self.remaining -= count;
            Ok(count)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            if self.fail_flush { Err(std::io::Error::other("injected flush failure")) } else { Ok(()) }
        }
    }
    for remaining in [0, 11, 14] {
        assert_eq!(
            write_ppm(Failure { remaining, fail_flush: false }, &pixels, 2, 1).unwrap_err(),
            "injected write failure"
        );
    }
    assert_eq!(
        write_ppm(Failure { remaining: usize::MAX, fail_flush: true }, &pixels, 2, 1).unwrap_err(),
        "injected flush failure"
    );
}
fn record(writer: &mut impl Write, s: &State, ram: &[u8]) -> Result<()> {
    let word = |o| u16::from_le_bytes([ram[o], ram[o + 1]]);
    writeln!(writer, "{},{:04x},{:04x},{},{:04x},{},{},{},{},{},{},{},{},{},{:03x},{:03x},{:04x},{:04x},{},{:04x},{},{},{},{},{},{},{:04x}", s.host_steps, s.game_state, s.loading_game_state, s.area, s.room, s.x, s.y, s.timer, s.polls, s.updates, s.nmis, s.waits, s.cpu_fallback_calls, s.cpu_instructions, s.requested_buttons, s.effective_buttons, s.joy1_current, s.joy1_new, word(0x9c2),word(0x93f),word(0xb2e),word(0xb2c),word(0xb36),word(0xb00),word(0xf7a),word(0xf7e),word(0xfa8)).map_err(|e| e.to_string())
}

const ITEM_HEADER: &str = "step,state,loading,area,room,x,y,requested,effective,held,new,polls,updates,nmis,waits,equipped_09a2,collected_09a4,item_bits_d873,queued_message,message_index,message_coroutine,frozen,health";
fn record_item(writer: &mut impl Write, s: &State, ram: &[u8]) -> Result<()> {
    let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
    writeln!(writer, "{},{:04x},{:04x},{},{:04x},{},{},{:03x},{:03x},{:04x},{:04x},{},{},{},{},{:04x},{:04x},{:02x},{:04x},{:04x},{:02x},{:04x},{}",
        s.host_steps, s.game_state, s.loading_game_state, s.area, s.room, s.x, s.y,
        s.requested_buttons, s.effective_buttons, s.joy1_current, s.joy1_new, s.polls, s.updates, s.nmis, s.waits,
        word(0x9a2), word(0x9a4), ram[0xd873], word(0xdc8), word(0x1c1f), ram[0x614], word(0xa78), word(0x9c2))
        .map_err(|e| e.to_string())
}
fn morph_snapshots(
    out: &Path,
    step: u64,
    ram: &[u8],
    prefix_retained: &mut bool,
    acquired_step: &mut Option<u64>,
) -> Result<()> {
    if step == MORPH_PREFIX_STEPS as u64 && !*prefix_retained {
        fs::write(out.join("morph-prefix.wram"), ram).map_err(|e| e.to_string())?;
        *prefix_retained = true;
    }
    // These are raw observations. Morph owns the stronger clear-to-set and
    // resumed-response predicate; a snapshot alone cannot grant the checkpoint.
    if step >= MORPH_PREFIX_STEPS as u64 && acquired_step.is_none() && ram[0x9a2] & ram[0x9a4] & ram[0xd873] & 4 != 0 {
        fs::write(out.join("morph-acquired.wram"), ram).map_err(|e| e.to_string())?;
        *acquired_step = Some(step);
    }
    Ok(())
}

const MISSILE_HEADER: &str = "step,state,loading,area,room,x,y,requested,effective,held,new,polls,updates,nmis,waits,pose,movement_type,equipped_09a2,collected_09a4,ammo_09c6,capacity_09c8,item_bits_d874,queued_message,message_index,message_coroutine,frozen,health,revealed,acquired,plm_slot,plm_header,plm_argument,plm_block_index,plm_preinstruction,plm_instruction,plm_timer,plm_instruction_timer,plm_link,plm_draw,plm_gfx,plm_block_valid,plm_block_word,plm_bts";
fn observe_missile(missile: &mut Missile, state: &State, ram: &[u8]) {
    // The observer's first state is its authenticated prefix boundary. Earlier
    // recorded inputs still execute normally and remain in the general CSV.
    if state.host_steps >= MISSILE_PREFIX_STEPS as u64 {
        missile.observe(state, ram);
    }
}
fn record_missile(writer: &mut impl Write, s: &State, ram: &[u8], revealed: bool, acquired: bool) -> Result<()> {
    let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
    // Preserve ambiguity explicitly rather than selecting an arbitrary item.
    let mut slots = (0..40).filter(|i| word(0x1c37 + 2 * i) == 0xef2f && word(0x1dc7 + 2 * i) == 0x22);
    let first = slots.next();
    let slot = if slots.next().is_some() { -2 } else { first.map_or(-1, |i| i as i32) };
    let field = |base| if slot >= 0 { word(base + 2 * slot as usize) } else { 0 };
    let block = usize::from(field(0x1c87));
    let valid =
        slot >= 0 && block.is_multiple_of(2) && 0x10002 + block + 1 < ram.len() && 0x16402 + block / 2 < ram.len();
    writeln!(writer, "{},{:04x},{:04x},{},{:04x},{},{},{:03x},{:03x},{:04x},{:04x},{},{},{},{},{:04x},{:02x},{:04x},{:04x},{},{},{:02x},{:04x},{:04x},{:02x},{:04x},{},{},{},{},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{:04x},{},{:04x},{:02x}",
        s.host_steps, s.game_state, s.loading_game_state, s.area, s.room, s.x, s.y,
        s.requested_buttons, s.effective_buttons, s.joy1_current, s.joy1_new, s.polls, s.updates, s.nmis, s.waits,
        word(0xa1c), ram[0xa1f], word(0x9a2), word(0x9a4), word(0x9c6), word(0x9c8), ram[0xd874],
        word(0xdc8), word(0x1c1f), ram[0x614], word(0xa78), word(0x9c2), revealed, acquired, slot,
        field(0x1c37), field(0x1dc7), field(0x1c87), field(0x1cd7), field(0x1d27), field(0x1d77),
        field(0xde1c), field(0xdebc), field(0xde6c), field(0xdf0c), valid,
        if valid { word(0x10002 + block) } else { 0 }, if valid { ram[0x16402 + block / 2] } else { 0 })
        .map_err(|e| e.to_string())
}

#[derive(Default)]
struct MissileSnapshots {
    prefix: bool,
    revealed: Option<u64>,
    acquired: Option<u64>,
}
impl MissileSnapshots {
    fn observe(&mut self, out: &Path, step: u64, ram: &[u8], revealed: bool, acquired: bool) -> Result<()> {
        if step < MISSILE_PREFIX_STEPS as u64 {
            return Ok(());
        }
        if step == MISSILE_PREFIX_STEPS as u64 && !self.prefix {
            fs::write(out.join("missile-prefix.wram"), ram).map_err(|e| e.to_string())?;
            self.prefix = true;
        }
        for (name, observed, first) in [
            ("missile-revealed.wram", revealed, &mut self.revealed),
            ("missile-acquired.wram", acquired, &mut self.acquired),
        ] {
            if observed && first.is_none() {
                fs::write(out.join(name), ram).map_err(|e| e.to_string())?;
                *first = Some(step);
            }
        }
        Ok(())
    }
    fn report(&self, out: &Path, result: &mut sm_binary_evidence::Map) -> Result<()> {
        result.insert("missile_prefix_wram_retained".into(), self.prefix.into());
        if self.prefix {
            result.insert("missile_prefix_wram_sha256".into(), hash_file(&out.join("missile-prefix.wram"))?.into());
        }
        for (label, step) in [("revealed", self.revealed), ("acquired", self.acquired)] {
            if let Some(step) = step {
                result.insert(format!("missile_{label}_wram_step"), (step as i64).into());
                result.insert(
                    format!("missile_{label}_wram_sha256"),
                    hash_file(&out.join(format!("missile-{label}.wram")))?.into(),
                );
            }
        }
        Ok(())
    }
}

const ENERGY_HEADER: &str = "step,state,loading,area,room,x,y,requested,effective,held,new,polls,updates,nmis,waits,pose,movement_type,y_speed,y_subspeed,y_direction,y_radius,x_speed,x_subspeed,equipped_09a2,collected_09a4,health,maximum,missiles,max_missiles,supers,max_supers,power_bombs,max_power_bombs,reserves,max_reserves,item_bits_d871,queued_message,message_index,message_coroutine,frozen,present,acquired,plm_slot,plm_header,plm_argument,plm_block_index,plm_preinstruction,plm_instruction,plm_timer,plm_instruction_timer,plm_link,plm_draw,plm_gfx,plm_block_valid,plm_block_word,plm_bts,target_block_index,target_block_word,target_bts";
fn record_energy(writer: &mut impl Write, s: &State, ram: &[u8], present: bool, acquired: bool) -> Result<()> {
    if ram.len() != 0x20000 {
        return Err("energy observation requires full WRAM".into());
    }
    let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
    let mut slots = (0..40).filter(|i| word(0x1c37 + 2 * i) == 0xeed7 && word(0x1dc7 + 2 * i) == 8);
    let first = slots.next();
    let slot = if slots.next().is_some() { -2 } else { first.map_or(-1, |i| i as i32) };
    let field = |base| if slot >= 0 { word(base + 2 * slot as usize) } else { 0 };
    let block = usize::from(field(0x1c87));
    let valid =
        slot >= 0 && block.is_multiple_of(2) && 0x10002 + block + 1 < ram.len() && 0x16402 + block / 2 < ram.len();
    let mut fields = vec![
        s.host_steps.to_string(),
        format!("{:04x}", s.game_state),
        format!("{:04x}", s.loading_game_state),
        s.area.to_string(),
        format!("{:04x}", s.room),
        s.x.to_string(),
        s.y.to_string(),
        format!("{:03x}", s.requested_buttons),
        format!("{:03x}", s.effective_buttons),
        format!("{:04x}", s.joy1_current),
        format!("{:04x}", s.joy1_new),
        s.polls.to_string(),
        s.updates.to_string(),
        s.nmis.to_string(),
        s.waits.to_string(),
        format!("{:04x}", word(0xa1c)),
        format!("{:02x}", ram[0xa1f]),
    ];
    fields.extend([0xb2e, 0xb2c, 0xb36, 0xb00, 0xb46, 0xb48, 0x9a2, 0x9a4].map(|at| format!("{:04x}", word(at))));
    fields
        .extend([0x9c2, 0x9c4, 0x9c6, 0x9c8, 0x9ca, 0x9cc, 0x9ce, 0x9d0, 0x9d6, 0x9d4].map(|at| word(at).to_string()));
    fields.extend([
        format!("{:02x}", ram[0xd871]),
        format!("{:04x}", word(0xdc8)),
        format!("{:04x}", word(0x1c1f)),
        format!("{:02x}", ram[0x614]),
        format!("{:04x}", word(0xa78)),
        present.to_string(),
        acquired.to_string(),
        slot.to_string(),
    ]);
    fields.extend(
        [0x1c37, 0x1dc7, 0x1c87, 0x1cd7, 0x1d27, 0x1d77, 0xde1c, 0xdebc, 0xde6c, 0xdf0c]
            .map(|base| format!("{:04x}", field(base))),
    );
    fields.extend([
        valid.to_string(),
        format!("{:04x}", if valid { word(0x10002 + block) } else { 0 }),
        format!("{:02x}", if valid { ram[0x16402 + block / 2] } else { 0 }),
        "1f8e".into(),
        format!("{:04x}", word(0x11f90)),
        format!("{:02x}", ram[0x173c9]),
    ]);
    writeln!(writer, "{}", fields.join(",")).map_err(|e| e.to_string())
}

#[derive(Default)]
struct EnergySnapshots {
    prefix: bool,
    present: Option<u64>,
    acquired: Option<u64>,
}
impl EnergySnapshots {
    fn observe(&mut self, out: &Path, step: u64, ram: &[u8], present: bool, acquired: bool) -> Result<()> {
        if step < ENERGY_PREFIX_STEPS as u64 {
            return Ok(());
        }
        if ram.len() != 0x20000 {
            return Err("energy snapshot requires full WRAM".into());
        }
        if step == ENERGY_PREFIX_STEPS as u64 && !self.prefix {
            fs::write(out.join("energy-prefix.wram"), ram).map_err(|e| e.to_string())?;
            self.prefix = true;
        }
        for (name, observed, first) in [
            ("energy-present.wram", present, &mut self.present),
            ("energy-acquired.wram", acquired, &mut self.acquired),
        ] {
            if observed && first.is_none() {
                fs::write(out.join(name), ram).map_err(|e| e.to_string())?;
                *first = Some(step);
            }
        }
        Ok(())
    }
    fn report(&self, out: &Path, result: &mut sm_binary_evidence::Map) -> Result<()> {
        result.insert("energy_prefix_wram_retained".into(), self.prefix.into());
        if self.prefix {
            result.insert("energy_prefix_wram_sha256".into(), hash_file(&out.join("energy-prefix.wram"))?.into());
        }
        for (label, step) in [("present", self.present), ("acquired", self.acquired)] {
            if let Some(step) = step {
                result.insert(format!("energy_{label}_wram_step"), (step as i64).into());
                result.insert(
                    format!("energy_{label}_wram_sha256"),
                    hash_file(&out.join(format!("energy-{label}.wram")))?.into(),
                );
            }
        }
        Ok(())
    }
}

// Freeze earlier lifecycle observers at the exact accepted tape endpoints.
fn morph_observation_step(scenario: &str, step: u64) -> bool {
    scenario == "morph" || (matches!(scenario, "missile" | "energy") && step <= MISSILE_PREFIX_STEPS as u64)
}
fn missile_observation_step(scenario: &str, step: u64) -> bool {
    step >= MISSILE_PREFIX_STEPS as u64
        && (scenario == "missile" || (scenario == "energy" && step <= ENERGY_PREFIX_STEPS as u64))
}
fn observe_energy(energy: &mut Energy, state: &State, ram: &[u8]) {
    if state.host_steps >= ENERGY_PREFIX_STEPS as u64 {
        energy.observe(state, ram);
    }
}

fn item_ownership_complete(counts: sm_native_host::ItemInstructionCounters) -> bool {
    let groups = [counts.scope_standalone, counts.scope_service52];
    (0..13).all(|i| groups.iter().map(|g| g.rust_completed[i]).sum::<u64>() > 0)
        && groups.iter().all(|g| {
            g.c_entries.iter().all(|&n| n == 0)
                && g.rust_refused.iter().all(|&n| n == 0)
                && g.entries == g.rust_entries
                && g.rust_entries == g.rust_completed
        })
        && counts.scope_service52.rust_completed.iter().any(|n| *n > 0)
        && counts.service_requests == counts.service_completions
        && counts.service_requests[..3].iter().all(|n| *n > 0)
}

const MORPH_ENTRIES: [usize; 12] = [0, 1, 3, 5, 6, 8, 11, 12, 13, 14, 15, 16];

fn morph_ownership_complete(counts: sm_native_host::ItemInstructionCounters) -> bool {
    let groups = [counts.morph_standalone, counts.morph_service52];
    MORPH_ENTRIES.iter().all(|&i| groups.iter().map(|g| g.rust_completed[i]).sum::<u64>() > 0)
        && groups.iter().all(|g| {
            g.c_entries.iter().all(|&n| n == 0)
                && g.rust_refused.iter().all(|&n| n == 0)
                && g.entries == g.rust_entries
                && g.rust_entries == g.rust_completed
        })
        && counts.morph_service52.rust_completed.iter().any(|&n| n > 0)
        && counts.service_requests == counts.service_completions
        && counts.service_requests[1..6].iter().all(|&n| n > 0)
}

fn tank_observed_entries_owned(counts: sm_native_host::ItemInstructionCounters) -> bool {
    let groups = [counts.tank_standalone, counts.tank_service52];
    groups.iter().any(|g| g.entries.iter().any(|&n| n > 0))
        && groups.iter().all(|g| {
            g.c_entries.iter().all(|&n| n == 0)
                && g.rust_refused.iter().all(|&n| n == 0)
                && g.entries == g.rust_entries
                && g.rust_entries == g.rust_completed
        })
        && counts.service_requests == counts.service_completions
}

fn energy_ownership_complete(counts: sm_native_host::ItemInstructionCounters) -> bool {
    tank_observed_entries_owned(counts)
        && counts.tank_standalone.rust_completed[17] + counts.tank_service52.rust_completed[17] > 0
}

fn acceptance_counts(
    scenario: &str,
    prefix: Option<sm_native_host::ItemInstructionCounters>,
    final_counts: sm_native_host::ItemInstructionCounters,
) -> sm_native_host::ItemInstructionCounters {
    if scenario == "energy" { prefix.unwrap_or_default() } else { final_counts }
}

fn energy_navigation_stop(author: bool, blocked: bool, state: &State) -> Option<String> {
    (author && blocked).then(|| {
        format!(
            "energy authoring navigation blocked at step {} room {:04x} x={} y={}",
            state.host_steps, state.room, state.x, state.y
        )
    })
}

fn ownership_failure_reason(scenario: &str, existing: String, failure: &str) -> String {
    // A collector cannot complete after navigation stops. Retain the cause of
    // that stop while the separate ownership report still rejects acceptance.
    if scenario == "energy" && !existing.is_empty() { existing } else { failure.into() }
}

fn worker(options: &Options) -> Result<()> {
    let out = options.path("--out")?.canonicalize().map_err(|e| e.to_string())?;
    let library = options.path("--library")?.canonicalize().map_err(|e| e.to_string())?;
    let rom = options.path("--rom")?.canonicalize().map_err(|e| e.to_string())?;
    let sdl_library = options
        .values
        .get("--sdl-library")
        .map(PathBuf::from)
        .map(|p| p.canonicalize().map_err(|e| e.to_string()))
        .transpose()?;
    let scenario = options.text("--scenario", "boot")?;
    if !matches!(scenario, "boot" | "ceres" | "none" | "parlor" | "morph" | "missile" | "energy" | "rio-return") {
        return Err("scenario must be boot, ceres, parlor, morph, missile, energy, rio-return or none".into());
    }
    let present = options.text("--present", "no")?;
    if !matches!(present, "no" | "interactive" | "replay") {
        return Err("present must be no, interactive or replay".into());
    }
    let tape_mode = tape_mode(options)?;
    let mut scenario_inputs = tape_mode
        .map(|(contract, author)| RecordedInputs::load(&options.path("--inputs")?, contract, author))
        .transpose()?;
    let limit = step_limit(options)?;
    let capture_every = options.number("--capture-every", 1000)?;
    let raw_input_hash =
        if options.values.contains_key("--inputs") { Some(hash_file(&options.path("--inputs")?)?) } else { None };
    let trace_enabled = trace_mode(options)?;
    if trace_enabled && options.number("--steps", 1)? != limit {
        return Err("trace steps must not exceed the scenario's effective limit".into());
    }
    let trace_identity = if trace_enabled {
        use std::io::Read;
        let mut bytes = Vec::new();
        File::open(options.path("--inputs")?)
            .map_err(|e| e.to_string())?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("trace input exceeds 16 MiB".into());
        }
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let pads = text.lines().map(parse_pad).collect::<Result<Vec<_>>>()?;
        if pads.len() as u64 != limit || raw_input_hash.as_ref() != Some(&format!("{:x}", Sha256::digest(&bytes))) {
            return Err("trace input must match the exact effective step count and authenticated bytes".into());
        }
        Some(sm_trace::Identity {
            reference: sm_native_host::SOURCE_REVISION.into(),
            rom: hash_file(&rom)?,
            host: format!(
                "library={};manifest={}",
                hash_file(&library)?,
                hash_file(&library.with_file_name("library.sme"))?
            ),
            input: raw_input_hash.clone().ok_or("trace requires inputs")?,
            observation: "128KiB-WRAM;SMNOBS02-external-including-SRAM;raw-ordered-native-effects;host-step-ordinal-v1"
                .into(),
            scenario: format!("{scenario};steps={limit}"),
        })
    } else {
        None
    };
    let trace_reference = if options.values.contains_key("--trace-reference") {
        let reference = crate::replay_trace::reference(
            &options.path("--trace-reference")?,
            options.text("--trace-reference-sha256", "")?,
            trace_identity.as_ref().ok_or("trace comparison requires identity")?,
        )?;
        if reference.summary().records != limit {
            return Err("trace reference step count differs from requested replay".into());
        }
        Some(reference)
    } else {
        None
    };
    let mut inputs = if tape_mode.is_none() && options.values.contains_key("--inputs") {
        use std::io::Read;
        let mut bytes = Vec::new();
        File::open(options.path("--inputs")?)
            .map_err(|e| e.to_string())?
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("recorded input exceeds the 16 MiB tape limit".into());
        }
        if raw_input_hash.as_ref() != Some(&format!("{:x}", Sha256::digest(&bytes))) {
            return Err("recorded input changed before replay load".into());
        }
        Some(BufReader::new(std::io::Cursor::new(bytes)).lines())
    } else {
        None
    };
    if let Some(inputs) = &scenario_inputs
        && raw_input_hash.as_ref() != Some(&inputs.sha256)
    {
        return Err("scenario input changed before replay load".into());
    }
    fs::create_dir(out.join("saves")).map_err(|e| e.to_string())?;
    let mut full_trace = trace_identity
        .as_ref()
        .map(|identity| crate::replay_trace::Session::create(&out, identity, trace_reference.as_ref()))
        .transpose()?;
    std::env::set_current_dir(&out).map_err(|e| e.to_string())?;
    let mut game = Game::boot(&library, &rom)?;
    let persistence_owner = persistence_owner(options)?;
    if persistence_owner == "rust" {
        crate::persistence_runtime::install(&mut game)?;
    }
    let item_owner = options.text("--item-instructions", options.text("--gameplay", "c")?)?;
    if !matches!(item_owner, "c" | "rust") {
        return Err("item-instructions must be c or rust".into());
    }
    let morph_owner = options.text("--morph-instructions", item_owner)?;
    if !matches!(morph_owner, "c" | "rust") {
        return Err("morph-instructions must be c or rust".into());
    }
    let tank_owner = options.text("--tank-instructions", item_owner)?;
    if !matches!(tank_owner, "c" | "rust") {
        return Err("tank-instructions must be c or rust".into());
    }
    let player_owner = options.text("--player-motion", "c")?;
    if !matches!(player_owner, "c" | "rust") {
        return Err("player-motion must be c or rust".into());
    }
    if player_owner == "rust" {
        crate::gameplay::install_player_motion(&mut game)?;
    }
    let waver_owner = options.text("--waver", "c")?;
    if !matches!(waver_owner, "c" | "rust") {
        return Err("waver must be c or rust".into());
    }
    if waver_owner == "rust" {
        crate::gameplay::install_waver(&mut game)?;
    }
    let ripper_owner = options.text("--ripper", "c")?;
    if !matches!(ripper_owner, "c" | "rust") {
        return Err("ripper must be c or rust".into());
    }
    if ripper_owner == "rust" {
        crate::gameplay::install_ripper(&mut game)?;
    }
    let rio_owner = options.text("--rio", "c")?;
    if !matches!(rio_owner, "c" | "rust") {
        return Err("rio must be c or rust".into());
    }
    if rio_owner == "rust" {
        game.install_rio();
    }
    if options.text("--gameplay", "c")? == "rust" {
        crate::gameplay::install_scrolling(&mut game, false)?;
        crate::gameplay::install_ceres_doors(&mut game, false)?;
        crate::gameplay::install_room_transitions(&mut game, false)?;
        crate::gameplay::install_door_scrolling(&mut game, false)?;
        crate::gameplay::install_camera(&mut game, false)?;
    }
    if item_owner == "rust" || morph_owner == "rust" || tank_owner == "rust" {
        crate::gameplay::install_selected_item_instructions(
            &mut game,
            item_owner == "rust",
            morph_owner == "rust",
            tank_owner == "rust",
        )?;
    }
    let callback_step = Rc::new(Cell::new(1u64));
    let ripper_rows = Rc::new(Cell::new(0u64));
    let ripper_observer = if ripper_owner == "rust" {
        let writer = Rc::new(RefCell::new(BufWriter::new(
            File::create(out.join("ripper-callbacks.csv")).map_err(|e| e.to_string())?,
        )));
        writeln!(writer.borrow_mut(), "step,phase,entry,slot,amount,result,x,x_sub,y,y_sub,speed_fraction,speed_integer,row,timer,instruction,instruction_timer")
            .map_err(|e| e.to_string())?;
        let output = writer.clone();
        let step = callback_step.clone();
        let rows = ripper_rows.clone();
        game.set_ripper_observer(move |event, ram| {
            let field = |offset: usize| {
                if event.slot < 0x800 && event.slot % 0x40 == 0 {
                    let at = 0xf78 + usize::from(event.slot) + offset;
                    u16::from_le_bytes([ram[at], ram[at + 1]]).to_string()
                } else {
                    String::new()
                }
            };
            writeln!(
                output.borrow_mut(),
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                step.get(),
                event.phase as u32,
                event.entry as u32,
                event.slot,
                event.amount,
                event.result,
                field(2),
                field(4),
                field(6),
                field(8),
                field(0x34),
                field(0x36),
                field(0x38),
                field(0x18),
                field(0x1a),
                field(0x1c)
            )
            .map_err(|e| e.to_string())?;
            rows.set(rows.get() + 1);
            Ok(())
        })?;
        Some(writer)
    } else {
        None
    };
    let waver_rows = Rc::new(Cell::new(0u64));
    let waver_observer = if waver_owner == "rust" {
        let writer = Rc::new(RefCell::new(BufWriter::new(
            File::create(out.join("waver-callbacks.csv")).map_err(|e| e.to_string())?,
        )));
        writeln!(writer.borrow_mut(), "step,phase,entry,k,jp,service,argument,auxiliary,result,current_slot,x,x_sub,y,y_sub,animation,phase_counter,event,desired_animation")
            .map_err(|e| e.to_string())?;
        let output = writer.clone();
        let step = callback_step.clone();
        let rows = waver_rows.clone();
        game.set_waver_observer(move |event, ram| {
            let word = |at: usize| u16::from_le_bytes([ram[at], ram[at + 1]]);
            let slot = word(0xe54);
            let field = |offset: usize| {
                if slot < 0x800 && slot % 0x40 == 0 {
                    word(0xf78 + usize::from(slot) + offset).to_string()
                } else {
                    String::new()
                }
            };
            writeln!(
                output.borrow_mut(),
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                step.get(),
                event.phase as u32,
                event.entry as u32,
                event.k,
                event.jp,
                event.service.map(|s| (s as u32).to_string()).unwrap_or_default(),
                event.argument,
                event.auxiliary,
                event.result,
                slot,
                field(2),
                field(4),
                field(6),
                field(8),
                field(0x34),
                field(0x36),
                field(0x38),
                field(0x3a)
            )
            .map_err(|e| e.to_string())?;
            rows.set(rows.get() + 1);
            Ok(())
        })?;
        Some(writer)
    } else {
        None
    };
    let player_rows = Rc::new(Cell::new(0u64));
    let player_observer = if player_owner == "rust" {
        let writer = Rc::new(RefCell::new(BufWriter::new(
            File::create(out.join("player-motion-callbacks.csv")).map_err(|e| e.to_string())?,
        )));
        writeln!(
            writer.borrow_mut(),
            "step,phase,entry,amount,result,x,x_sub,y,y_sub,direction,collision,pose_input,movement_type"
        )
        .map_err(|e| e.to_string())?;
        let output = writer.clone();
        let step = callback_step.clone();
        let rows = player_rows.clone();
        game.set_player_motion_observer(move |event, ram| {
            let word = |at: usize| u16::from_le_bytes([ram[at], ram[at + 1]]);
            writeln!(
                output.borrow_mut(),
                "{},{},{},{},{},{},{},{},{},{},{},{},{}",
                step.get(),
                event.phase as u32,
                event.entry as u32,
                event.amount,
                event.result,
                word(0xaf6),
                word(0xaf8),
                word(0xafa),
                word(0xafc),
                word(0xb02),
                word(0xdd0),
                word(0xdc6),
                ram[0xa1f]
            )
            .map_err(|e| e.to_string())?;
            rows.set(rows.get() + 1);
            Ok(())
        })?;
        Some(writer)
    } else {
        None
    };
    let callback_rows = [Rc::new(Cell::new(0u64)), Rc::new(Cell::new(0u64)), Rc::new(Cell::new(0u64))];
    let item_observer = if matches!(scenario, "missile" | "morph" | "energy") {
        let mut writers = Vec::new();
        for name in ["item-callbacks.csv", "morph-callbacks.csv", "tank-callbacks.csv"] {
            let writer =
                Rc::new(RefCell::new(BufWriter::new(File::create(out.join(name)).map_err(|e| e.to_string())?)));
            writeln!(writer.borrow_mut(), "step,phase,entry,slot,value,context,wram_sha256")
                .map_err(|e| e.to_string())?;
            writers.push(writer);
        }
        let output = writers.clone();
        let step = callback_step.clone();
        let rows = callback_rows.clone();
        let energy_run = scenario == "energy";
        game.set_item_instruction_observer(move |event, ram| {
            if energy_run && event.scope_id <= 2 && step.get() > ENERGY_PREFIX_STEPS as u64 {
                return Ok(());
            }
            if matches!(event.scope_id, 1..=3) {
                let index = event.scope_id as usize - 1;
                let entry = event.entry.ok_or("scope item observation without entry")?;
                let mut output = output[index].borrow_mut();
                writeln!(
                    output,
                    "{},{},{},{:04x},{:08x},{},{:x}",
                    step.get(),
                    event.phase as u32,
                    entry as u32,
                    event.slot,
                    event.value,
                    event.context,
                    Sha256::digest(ram)
                )
                .map_err(|e| e.to_string())?;
                output.flush().map_err(|e| e.to_string())?;
                rows[index].set(rows[index].get() + 1);
            }
            Ok(())
        })?;
        Some(writers)
    } else {
        None
    };
    let mut projectiles_installed = false;
    let (width, height) = game.dimensions();
    let mut window = if present != "no" {
        Some(
            sm_presentation::Window::new(width as u32, height as u32, 3, sdl_library.as_deref())
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    let mut observations = BufWriter::new(File::create(out.join("observations.csv")).map_err(|e| e.to_string())?);
    writeln!(observations,"step,state,loading,area,room,x,y,timer,polls,updates,nmis,waits,cpu_calls,cpu_instructions,requested,effective,held,new,health,ceres_status,y_speed,y_subspeed,y_direction,y_radius,enemy_x,enemy_y,enemy_ai").map_err(|e| e.to_string())?;
    let mut input_log = BufWriter::new(File::create(out.join("inputs.txt")).map_err(|e| e.to_string())?);
    let mut item_observations = if matches!(scenario, "morph" | "missile" | "energy") {
        let mut writer = BufWriter::new(File::create(out.join("item-observations.csv")).map_err(|e| e.to_string())?);
        writeln!(writer, "{ITEM_HEADER}").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        Some(writer)
    } else {
        None
    };
    let mut morph_prefix_retained = false;
    let mut morph_acquired_step = None;
    let mut item_rows = 0u64;
    let mut missile_observations = if matches!(scenario, "missile" | "energy") {
        let mut writer = BufWriter::new(File::create(out.join("missile-observations.csv")).map_err(|e| e.to_string())?);
        writeln!(writer, "{MISSILE_HEADER}").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        Some(writer)
    } else {
        None
    };
    let mut missile_snapshots = MissileSnapshots::default();
    let mut missile_rows = 0u64;
    let mut energy_observations = if scenario == "energy" {
        let mut writer = BufWriter::new(File::create(out.join("energy-observations.csv")).map_err(|e| e.to_string())?);
        writeln!(writer, "{ENERGY_HEADER}").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        Some(writer)
    } else {
        None
    };
    let mut energy_snapshots = EnergySnapshots::default();
    let mut energy_rows = 0u64;
    let mut morph_prefix_counts = None;
    let mut missile_prefix_counts = None;
    let mut route = Route::default();
    let mut parlor = Parlor::default();
    let mut morph = Morph::default();
    let mut missile = Missile::default();
    let mut energy = Energy::default();
    let mut rio_return = crate::rio_return::Return::default();
    let mut rio_capture =
        rio_trace_interval(options)?
            .map(|(start, count)| {
                let rio_identity = sm_trace::Identity {
                    reference: sm_native_host::SOURCE_REVISION.into(),
                    rom: hash_file(&rom)?,
                    host: format!(
                        "library={};manifest={}",
                        hash_file(&library)?,
                        hash_file(&library.with_file_name("library.sme"))?
                    ),
                    input: raw_input_hash.clone().ok_or("Rio capture requires an authenticated recorded input tape")?,
                    observation:
                        "128KiB-WRAM;SMNOBS02-external;raw-ordered-native-effects;interval-ordinal-start-in-scenario"
                            .into(),
                    scenario: format!("{scenario};rio-interval={start},{count}"),
                };
                crate::rio_capture::Capture::create(&out, start, count, game.rio_counters(), &rio_identity)
            })
            .transpose()?;
    let (first_buttons, first_record) = if let Some(inputs) = scenario_inputs.as_mut() {
        inputs.next()?.ok_or("scenario prefix missing first pad")?
    } else {
        let pad = if let Some(lines) = inputs.as_mut() { read_pad(lines)? } else { 0 };
        (pad, format!("{pad:03x}\n"))
    };
    write!(input_log, "{first_record}").map_err(|e| e.to_string())?;
    input_log.flush().map_err(|e| e.to_string())?;
    let mut state = game.host_step(first_buttons)?;
    let mut previous = (u16::MAX, u16::MAX);
    let mut success = false;
    let mut closed = false;
    let mut steps = 1;
    let start = Instant::now();
    let mut audio_nonzero = 0u64;
    let mut stop_reason = String::new();
    loop {
        if let Some(trace) = &mut full_trace
            && let Err(error) = trace.record(state.host_steps, game.wram(), game.observation_bytes(), game.effects())
        {
            record(&mut observations, &state, game.wram())?;
            observations.flush().map_err(|e| e.to_string())?;
            screenshot(&out.join("mismatch.png"), game.pixels(), width, height)?;
            return Err(error);
        }
        route.observe(&state);
        if scenario == "rio-return" && state.host_steps >= crate::rio_return::PREFIX {
            rio_return.observe(&state, game.wram());
        }
        if scenario == "parlor" {
            parlor.observe(&state);
        }
        let ram = game.wram();
        if let Some(capture) = rio_capture.as_mut() {
            capture.record(&state, ram, game.observation_bytes(), game.effects(), game.rio_counters())?;
        }
        if morph_observation_step(scenario, state.host_steps) {
            morph.observe(&state, ram);
        }
        if missile_observation_step(scenario, state.host_steps) {
            observe_missile(&mut missile, &state, ram);
        }
        if scenario == "energy" {
            if state.host_steps == MISSILE_PREFIX_STEPS as u64 {
                morph_prefix_counts = Some(game.item_instruction_counters());
            }
            if state.host_steps == ENERGY_PREFIX_STEPS as u64 {
                missile_prefix_counts = Some(game.item_instruction_counters());
            }
            observe_energy(&mut energy, &state, ram);
        }
        if let Some(items) = energy_observations.as_mut()
            && state.host_steps >= ENERGY_PREFIX_STEPS as u64
        {
            record_energy(items, &state, ram, energy.present(), energy.acquired())?;
            items.flush().map_err(|e| e.to_string())?;
            energy_snapshots.observe(&out, state.host_steps, ram, energy.present(), energy.acquired())?;
            energy_rows += 1;
        }
        if let Some(items) = missile_observations.as_mut()
            && missile_observation_step(scenario, state.host_steps)
        {
            record_missile(items, &state, ram, missile.revealed(), missile.acquired())?;
            items.flush().map_err(|e| e.to_string())?;
            missile_snapshots.observe(&out, state.host_steps, ram, missile.revealed(), missile.acquired())?;
            missile_rows += 1;
        }
        if let Some(items) = item_observations.as_mut()
            && state.host_steps >= MORPH_PREFIX_STEPS as u64
            && morph_observation_step(scenario, state.host_steps)
        {
            record_item(items, &state, ram)?;
            items.flush().map_err(|e| e.to_string())?;
            morph_snapshots(&out, state.host_steps, ram, &mut morph_prefix_retained, &mut morph_acquired_step)?;
            item_rows += 1;
        }
        let health = u16::from_le_bytes([ram[0x9c2], ram[0x9c3]]);
        record(&mut observations, &state, ram)?;
        audio_nonzero += game.audio().iter().filter(|v| **v != 0).count() as u64;
        let changed = previous != (state.game_state, state.room);
        // Preserve the exact last completed observation before any foreign call.
        observations.flush().map_err(|e| e.to_string())?;
        if changed || state.host_steps % capture_every == 0 {
            println!(
                "step={} state={:x} area={} room={:04x} x={} y={} hp={} polls={} updates={} escape={}",
                state.host_steps,
                state.game_state,
                state.area,
                state.room,
                state.x,
                state.y,
                health,
                state.polls,
                state.updates,
                route.escape_stage
            );
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            screenshot(&out.join(format!("step-{:06}.png", state.host_steps)), game.pixels(), width, height)?;
        }
        previous = (state.game_state, state.room);
        if let Some(window) = window.as_mut() {
            window.present(game.pixels(), game.audio()).map_err(|e| e.to_string())?;
        }
        if present != "interactive"
            && ((scenario == "boot" && route.controller_response)
                || (scenario == "ceres" && route.arrived)
                || (scenario == "parlor" && route.arrived && parlor.complete())
                || (scenario == "morph" && route.arrived && morph.complete())
                || (scenario == "missile" && route.arrived && missile.complete())
                || (scenario == "energy"
                    && route.arrived
                    && morph.complete()
                    && missile.complete()
                    && energy.complete()))
        {
            if trace_enabled && steps != limit {
                fs::write(out.join("early-checkpoint.wram"), game.wram()).map_err(|e| e.to_string())?;
                fs::write(out.join("early-checkpoint.external"), game.observation_bytes())
                    .map_err(|e| e.to_string())?;
                fs::write(out.join("early-checkpoint.effects"), game.effects()).map_err(|e| e.to_string())?;
                return Err(format!("trace checkpoint reached at step {steps} before exact limit {limit}"));
            }
            success = true;
            break;
        }
        if inputs.is_none()
            && scenario == "ceres"
            && state.game_state == 8
            && state.room != 0xe0b5
            && route.stationary_steps > 300
        {
            eprintln!("controller stalled in room {:04x} at {},{}", state.room, state.x, state.y);
            break;
        }
        if matches!(scenario, "ceres" | "parlor" | "morph" | "missile" | "energy" | "rio-return")
            && matches!(state.game_state, 0x19 | 0x1a | 0x23 | 0x24 | 0x25)
        {
            stop_reason = format!("{scenario} route entered failure state {:x}", state.game_state);
            eprintln!("{stop_reason}");
            break;
        }
        if scenario == "energy"
            && let Some(reason) = energy_navigation_stop(
                scenario_inputs.as_ref().is_some_and(|inputs| inputs.author),
                energy.blocked(),
                &state,
            )
        {
            stop_reason = reason;
            eprintln!("{stop_reason}");
            break;
        }
        if scenario == "rio-return"
            && let Some(reason) = rio_return.failure()
        {
            stop_reason = reason.into();
            break;
        }
        if steps >= limit {
            success = scenario == "none";
            stop_reason = "step budget exhausted before checkpoint".into();
            break;
        }
        let held = if let Some(window) = window.as_mut() {
            match window.poll().map_err(|e| e.to_string())? {
                None => {
                    closed = true;
                    stop_reason = "presentation closed before checkpoint".into();
                    break;
                }
                Some(held) if present == "interactive" => Some(held),
                Some(_) => None,
            }
        } else {
            None
        };
        let (buttons, input_record) = if let Some(inputs) = scenario_inputs.as_mut() {
            match inputs.next() {
                Ok(Some(record)) => record,
                Ok(None) => {
                    let pad = if scenario == "rio-return" {
                        rio_return.buttons(&state, game.wram())
                    } else if scenario == "energy" {
                        energy.buttons(&state, game.wram())
                    } else if scenario == "missile" {
                        missile.buttons(&state, game.wram())
                    } else if scenario == "morph" {
                        morph.buttons(&state, game.wram())
                    } else {
                        parlor.buttons(&state, game.wram())
                    };
                    (pad, format!("{pad:03x}\n"))
                }
                Err(error) => {
                    stop_reason = error;
                    break;
                }
            }
        } else if let Some(lines) = inputs.as_mut() {
            let pad = held.map(Ok).unwrap_or_else(|| read_pad(lines))?;
            (pad, format!("{pad:03x}\n"))
        } else {
            let pad = held.unwrap_or_else(|| route.buttons(&state, game.wram(), scenario));
            (pad, format!("{pad:03x}\n"))
        };
        if present != "no" {
            let due = Duration::from_secs_f64(steps as f64 * 534.0 / 32000.0);
            if let Some(delay) = due.checked_sub(start.elapsed()) {
                std::thread::sleep(delay);
            }
        }
        write!(input_log, "{input_record}").map_err(|e| e.to_string())?;
        input_log.flush().map_err(|e| e.to_string())?;
        if !projectiles_installed
            && options.text("--gameplay", "c")? == "rust"
            && state.game_state == 8
            && state.area == 6
            && state.room == 0xdf45
        {
            crate::gameplay::install_projectiles(&mut game, false)?;
            projectiles_installed = true;
        }
        callback_step.set(steps + 1);
        state = game.host_step(buttons)?;
        steps += 1;
    }
    let item_counts = game.item_instruction_counters();
    let item_acceptance_counts = acceptance_counts(scenario, missile_prefix_counts, item_counts);
    let morph_acceptance_counts = acceptance_counts(scenario, morph_prefix_counts, item_counts);
    if matches!(scenario, "missile" | "energy")
        && item_owner == "rust"
        && !item_ownership_complete(item_acceptance_counts)
    {
        success = false;
        stop_reason = ownership_failure_reason(scenario, stop_reason, "item instruction material ownership incomplete");
    }
    let mut morph_report = sm_binary_evidence::Map::new();
    if matches!(scenario, "morph" | "missile" | "energy") {
        morph.report(&mut morph_report);
        if morph_owner == "rust"
            && (!morph_ownership_complete(morph_acceptance_counts)
                || !morph.complete()
                || morph_report
                    .get("morph_message_waits")
                    .and_then(sm_binary_evidence::Value::as_integer)
                    .unwrap_or(i64::MAX)
                    > 512)
        {
            success = false;
            stop_reason =
                ownership_failure_reason(scenario, stop_reason, "Morph instruction ownership or lifecycle incomplete");
        }
    }
    if scenario == "energy"
        && (!morph.complete()
            || !missile.complete()
            || morph_report
                .get("morph_message_waits")
                .and_then(sm_binary_evidence::Value::as_integer)
                .unwrap_or(i64::MAX)
                > 512)
    {
        success = false;
        stop_reason =
            ownership_failure_reason(scenario, stop_reason, "frozen Morph or missile prefix lifecycle incomplete");
    }
    if tank_owner == "rust"
        && [item_counts.tank_standalone, item_counts.tank_service52].iter().any(|g| g.entries.iter().any(|&n| n > 0))
        && !tank_observed_entries_owned(item_counts)
    {
        success = false;
        stop_reason =
            ownership_failure_reason(scenario, stop_reason, "observed exposed tank entries were not fully Rust-owned");
    }
    if scenario == "energy" && tank_owner == "rust" && !energy_ownership_complete(item_counts) {
        success = false;
        stop_reason = ownership_failure_reason(
            scenario,
            stop_reason,
            "energy collector entry17 or tank scope Rust ownership incomplete",
        );
    }
    observations.flush().map_err(|e| e.to_string())?;
    input_log.flush().map_err(|e| e.to_string())?;
    fs::write(out.join("final.wram"), game.wram()).map_err(|e| e.to_string())?;
    if trace_enabled {
        fs::write(out.join("final.external"), game.observation_bytes()).map_err(|e| e.to_string())?;
        fs::write(out.join("final.effects"), game.effects()).map_err(|e| e.to_string())?;
    }
    screenshot(&out.join("final.png"), game.pixels(), width, height)?;
    let mut result = sm_binary_evidence::Map::new();
    if let Some(capture) = rio_capture {
        let (rows, complete) = capture.finish()?;
        if !complete {
            success = false;
            stop_reason = ownership_failure_reason(scenario, stop_reason, "requested Rio capture interval incomplete");
        }
        result.insert("rio_trace_rows".into(), (rows as i64).into());
        result.insert("rio_trace_interval_complete".into(), complete.into());
        result.insert("rio_trace_sha256".into(), hash_file(&out.join("rio-trace.bin"))?.into());
        result.insert("rio_facts_sha256".into(), hash_file(&out.join("rio-facts.csv"))?.into());
        result.insert("rio_consumed_inputs_sha256".into(), hash_file(&out.join("inputs.txt"))?.into());
        result.insert("rio_samples_inventory_sha256".into(), hash_file(&out.join("rio-samples.sha256"))?.into());
        if !complete {
            result.insert("incomplete_reason".into(), stop_reason.clone().into());
        }
    }
    if let Some(trace) = full_trace {
        if !success {
            fs::write(out.join("trace-failure.txt"), &stop_reason).map_err(|e| e.to_string())?;
            result.insert("trace_acceptance_failed".into(), true.into());
        } else {
            let summary = trace.finish(limit, success)?;
            result.insert("trace_records".into(), (summary.records as i64).into());
            result.insert("trace_sha256".into(), hash_file(&out.join("trace.bin"))?.into());
            result.insert("trace_comparison_complete".into(), trace_reference.is_some().into());
        }
    }
    result.insert("schema".into(), "sm-native-result-v2".into());
    result.insert("screenshot_format".into(), if cfg!(feature = "png-output") { "png" } else { "ppm" }.into());
    result.insert("scenario".into(), scenario.into());
    result.insert(
        "status".into(),
        if closed {
            "window-closed"
        } else if success {
            "passed"
        } else {
            "milestone-not-reached"
        }
        .into(),
    );
    result.insert("controller_response".into(), route.controller_response.into());
    result.insert("ceres_escape".into(), (route.escape_stage == 3).into());
    result.insert("area_zero_arrival".into(), route.arrived.into());
    if let Some(inputs) = &scenario_inputs {
        inputs.report(&mut result, state.host_steps, limit);
        let complete = if scenario == "rio-return" {
            rio_return.report(&mut result);
            false
        } else if scenario == "energy" {
            missile.report(&mut result);
            energy.report(&mut result);
            morph.complete() && missile.complete() && energy.complete()
        } else if scenario == "missile" {
            missile.report(&mut result);
            missile.complete()
        } else if scenario == "morph" {
            morph.report(&mut result);
            morph.complete()
        } else {
            parlor.report(&mut result);
            parlor.complete()
        };
        result.insert("checkpoint_complete".into(), (route.arrived && complete).into());
        result.insert("input_log_sha256".into(), hash_file(&out.join("inputs.txt"))?.into());
        if !success {
            result.insert("incomplete_reason".into(), stop_reason.clone().into());
        }
    }
    if matches!(scenario, "morph" | "missile" | "energy") {
        result.extend(morph_report);
        result.insert("item_observation_rows".into(), (item_rows as i64).into());
        result.insert("item_observations_sha256".into(), hash_file(&out.join("item-observations.csv"))?.into());
        result.insert("morph_prefix_wram_retained".into(), morph_prefix_retained.into());
        if morph_prefix_retained {
            result.insert("morph_prefix_wram_sha256".into(), hash_file(&out.join("morph-prefix.wram"))?.into());
        }
        if let Some(step) = morph_acquired_step {
            result.insert("morph_acquired_wram_step".into(), (step as i64).into());
            result.insert("morph_acquired_wram_sha256".into(), hash_file(&out.join("morph-acquired.wram"))?.into());
        }
    }
    if matches!(scenario, "missile" | "energy") {
        result.insert("missile_observation_rows".into(), (missile_rows as i64).into());
        result.insert("missile_observations_sha256".into(), hash_file(&out.join("missile-observations.csv"))?.into());
        missile_snapshots.report(&out, &mut result)?;
    }
    if scenario == "energy" {
        result.insert("energy_observation_rows".into(), (energy_rows as i64).into());
        result.insert("energy_observations_sha256".into(), hash_file(&out.join("energy-observations.csv"))?.into());
        energy_snapshots.report(&out, &mut result)?;
        result.insert("energy_instruction_ownership_complete".into(), energy_ownership_complete(item_counts).into());
        for (name, step, counts) in [
            ("morph", MISSILE_PREFIX_STEPS, morph_prefix_counts),
            ("missile", ENERGY_PREFIX_STEPS, missile_prefix_counts),
        ] {
            result.insert(format!("{name}_acceptance_counters_retained"), counts.is_some().into());
            if let Some(counts) = counts {
                result.insert(format!("{name}_acceptance_counters_step"), (step as i64).into());
                result.insert(format!("{name}_acceptance_counters"), counts.to_evidence()?);
            }
        }
    }
    result.insert("original_rom_comparison".into(), "not-tested".into());
    result.insert("item_instruction_owner".into(), item_owner.into());
    result.insert("morph_instruction_owner".into(), morph_owner.into());
    result.insert("tank_instruction_owner".into(), tank_owner.into());
    result.insert("tank_observed_entries_owned".into(), tank_observed_entries_owned(item_counts).into());
    result.insert(
        "tank_new_collectors_completed".into(),
        [17, 18, 19, 20]
            .iter()
            .map(|&i| {
                sm_binary_evidence::Value::Integer(
                    (item_counts.tank_standalone.rust_completed[i] + item_counts.tank_service52.rust_completed[i])
                        as i64,
                )
            })
            .collect::<Vec<_>>()
            .into(),
    );
    result.insert(
        "morph_instruction_ownership_complete".into(),
        morph_ownership_complete(morph_acceptance_counts).into(),
    );
    result.insert("item_instruction_ownership_complete".into(), item_ownership_complete(item_acceptance_counts).into());
    result.insert("item_instruction_counters".into(), game.item_instruction_counters().to_evidence()?);
    if let Some(writers) = item_observer {
        for (i, name) in ["item", "morph", "tank"].iter().enumerate() {
            writers[i].borrow_mut().flush().map_err(|e| e.to_string())?;
            result.insert(format!("{name}_callback_rows"), (callback_rows[i].get() as i64).into());
            result.insert(
                format!("{name}_callbacks_sha256"),
                hash_file(&out.join(format!("{name}-callbacks.csv")))?.into(),
            );
        }
    }
    let mut owners = Vec::new();
    result.insert("ripper_owner".into(), ripper_owner.into());
    result.insert("rio_owner".into(), rio_owner.into());
    result.insert("rio_hook_installed".into(), (rio_owner == "rust").into());
    let rio = game.rio_counters();
    let mut rio_result = sm_binary_evidence::Map::new();
    for (name, range) in [
        ("entries", 0..10),
        ("c_entries", 10..20),
        ("rust_entries", 20..30),
        ("rust_refused", 30..40),
        ("rust_completed", 40..50),
        ("terrain_unavailable", 50..60),
        ("service_requests", 60..63),
        ("service_completions", 63..66),
    ] {
        rio_result.insert(name.into(), rio[range].to_evidence()?);
    }
    rio_result.insert("invalid_requests".into(), rio[66].to_evidence()?);
    result.insert("rio_counters".into(), sm_binary_evidence::Value::Table(rio_result));
    result.insert("ripper_hook_installed".into(), (ripper_owner == "rust").into());
    result.insert("ripper_counters".into(), game.ripper_counters().to_evidence()?);
    if let Some(writer) = ripper_observer {
        writer.borrow_mut().flush().map_err(|e| e.to_string())?;
        result.insert("ripper_callback_rows".into(), (ripper_rows.get() as i64).into());
        result.insert("ripper_callbacks_sha256".into(), hash_file(&out.join("ripper-callbacks.csv"))?.into());
    }
    result.insert("waver_owner".into(), waver_owner.into());
    result.insert("waver_hook_installed".into(), (waver_owner == "rust").into());
    result.insert("waver_counters".into(), game.waver_counters().to_evidence()?);
    if let Some(writer) = waver_observer {
        writer.borrow_mut().flush().map_err(|e| e.to_string())?;
        result.insert("waver_callback_rows".into(), (waver_rows.get() as i64).into());
        result.insert("waver_callbacks_sha256".into(), hash_file(&out.join("waver-callbacks.csv"))?.into());
    }
    result.insert("persistence_owner".into(), persistence_owner.into());
    result.insert("persistence_hook_installed".into(), (persistence_owner == "rust").into());
    result.insert("persistence_counters".into(), game.persistence_counters().to_evidence()?);
    result.insert("final_sram_sha256".into(), format!("{:x}", Sha256::digest(game.sram())).into());
    result.insert("player_motion_owner".into(), player_owner.into());
    result.insert("player_motion_counters".into(), game.player_motion_counters().to_evidence()?);
    if let Some(writer) = player_observer {
        writer.borrow_mut().flush().map_err(|e| e.to_string())?;
        result.insert("player_motion_callback_rows".into(), (player_rows.get() as i64).into());
        result.insert(
            "player_motion_callbacks_sha256".into(),
            hash_file(&out.join("player-motion-callbacks.csv"))?.into(),
        );
    }
    if options.text("--gameplay", "c")? == "rust" {
        owners.extend(["projectiles", "scrolling", "ceres-doors", "room-transitions", "directional-motion", "camera"]);
    }
    for (owner, name) in [
        (item_owner, "missile-instructions"),
        (morph_owner, "Morph-instructions"),
        (tank_owner, "exposed-tank-instructions"),
        (player_owner, "player-vertical-motion"),
        (persistence_owner, "persistence-codec"),
        (waver_owner, "waver-actor"),
        (ripper_owner, "ripper-patrol"),
        (rio_owner, "rio-actor"),
    ] {
        if owner == "rust" {
            owners.push(name);
        }
    }
    result.insert(
        "gameplay_owner".into(),
        if owners.is_empty() { "recovered-C".to_owned() } else { format!("mixed-{}-Rust", owners.join("-")) }.into(),
    );
    result.insert("patched_rom_sha256".into(), game.patched_rom_sha256().into());
    let projectile = game.projectile_counters();
    for (name, counts) in [
        ("projectile_c_entries", projectile.c_entries),
        ("projectile_rust_completed", projectile.rust_completed),
        ("projectile_rust_refused", projectile.rust_refused),
    ] {
        result.insert(
            name.into(),
            sm_binary_evidence::Value::Array(
                counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect(),
            ),
        );
    }
    result
        .insert("projectile_foreign_calls".into(), sm_binary_evidence::Value::Integer(projectile.foreign_calls as i64));
    let scrolling = game.scroll_counters();
    for (name, counts) in [
        ("scroll_c_entries", scrolling.c_entries),
        ("scroll_rust_completed", scrolling.rust_completed),
        ("scroll_rust_refused", scrolling.rust_refused),
    ] {
        result.insert(
            name.into(),
            sm_binary_evidence::Value::Array(counts.into_iter().map(|n| (n as i64).into()).collect()),
        );
    }
    let doors = game.ceres_door_counters();
    for (name, counts) in [
        ("ceres_door_c_entries", doors.c_entries),
        ("ceres_door_rust_completed", doors.rust_completed),
        ("ceres_door_rust_refused", doors.rust_refused),
    ] {
        result.insert(
            name.into(),
            sm_binary_evidence::Value::Array(counts.into_iter().map(|n| (n as i64).into()).collect()),
        );
    }
    result.insert("ceres_door_foreign_requests".into(), (doors.foreign_requests as i64).into());
    result.insert("ceres_door_foreign_completions".into(), (doors.foreign_completions as i64).into());
    let transition = game.room_transition_counters();
    for (name, counts) in [
        ("room_transition_c_entries", transition.c_entries),
        ("room_transition_rust_entries", transition.rust_entries),
        ("room_transition_rust_completed", transition.rust_completed),
        ("room_transition_rust_refused", transition.rust_refused),
        ("room_transition_pending", transition.root_pending),
        ("room_transition_resumed", transition.root_resumed),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    for (name, counts) in [
        ("transition_routine_c_entries", transition.routine_c_entries),
        ("transition_routine_rust_entries", transition.routine_rust_entries),
        ("transition_routine_rust_completed", transition.routine_rust_completed),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    for (name, counts) in [
        ("transition_service_requests", transition.service_requests),
        ("transition_service_completions", transition.service_completions),
        ("transition_foreign_requests", transition.foreign_requests),
        ("transition_foreign_completions", transition.foreign_completions),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    let directional = game.door_scrolling_counters();
    for (name, counts) in [
        ("door_scrolling_c_entries", directional.c_entries),
        ("door_scrolling_rust_entries", directional.rust_entries),
        ("door_scrolling_rust_completed", directional.rust_completed),
        ("door_scrolling_rust_refused", directional.rust_refused),
        ("door_scrolling_coordinator_entries", directional.coordinator_entries),
        ("door_scrolling_standalone_entries", directional.standalone_entries),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    for (name, counts) in [
        ("door_scrolling_routine_c_entries", directional.routine_c_entries),
        ("door_scrolling_routine_rust_entries", directional.routine_rust_entries),
        ("door_scrolling_routine_rust_completed", directional.routine_rust_completed),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    for (name, counts) in [
        ("door_scrolling_service_requests", directional.service_requests),
        ("door_scrolling_service_completions", directional.service_completions),
        ("door_scrolling_foreign_requests", directional.foreign_requests),
        ("door_scrolling_foreign_completions", directional.foreign_completions),
        ("door_scrolling_child_entries", directional.child_entries),
        ("door_scrolling_child_completions", directional.child_completions),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    let camera = game.camera_counters();
    for (name, counts) in [
        ("camera_routine_c_entries", camera.routine_c_entries),
        ("camera_routine_c_completed", camera.routine_c_completed),
        ("camera_routine_rust_entries", camera.routine_rust_entries),
        ("camera_routine_rust_completed", camera.routine_rust_completed),
    ] {
        result.insert(
            name.into(),
            counts.into_iter().map(|n| sm_binary_evidence::Value::Integer(n as i64)).collect::<Vec<_>>().into(),
        );
    }
    for (name, value) in [
        ("camera_c_entries", camera.c_entries),
        ("camera_rust_entries", camera.rust_entries),
        ("camera_rust_completed", camera.rust_completed),
        ("camera_rust_refused", camera.rust_refused),
        ("steps", state.host_steps),
        ("polls", state.polls),
        ("completed_updates", state.updates),
        ("cpu_fallback_calls", state.cpu_fallback_calls),
        ("cpu_instructions", state.cpu_instructions),
        ("audio_nonzero_samples", audio_nonzero),
        ("game_state", state.game_state as u64),
        ("area", state.area as u64),
        ("room", state.room as u64),
    ] {
        result.insert(name.into(), sm_binary_evidence::Value::Integer(value as i64));
    }
    if scenario == "rio-return" {
        rio_return.report(&mut result);
    }
    write_evidence(&out.join("result.sme"), &result)?;
    if success || (closed && tape_mode.is_none() && rio_trace_interval(options)?.is_none()) {
        Ok(())
    } else if tape_mode.is_some() {
        Err(format!("{scenario} milestone not reached in {steps} host steps: {stop_reason}"))
    } else {
        Err(format!("{scenario} milestone not reached in {steps} host steps"))
    }
}

fn read_pad(lines: &mut impl Iterator<Item = std::io::Result<String>>) -> Result<u16> {
    let line = lines.next().ok_or("input EOF before named milestone/step limit")?.map_err(|e| e.to_string())?;
    parse_pad(&line)
}
fn parse_pad(line: &str) -> Result<u16> {
    u16::from_str_radix(line.trim(), 16)
        .ok()
        .filter(|v| *v <= 0xfff)
        .ok_or_else(|| "input must be 12-bit native hexadecimal pad".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_trace_requires_exact_recorded_replay_and_a_sealed_reference() {
        assert!(trace_mode(&options(&["--trace", "yes"])).is_err());
        assert!(trace_mode(&options(&["--trace", "yes", "--inputs", "tape", "--steps", "1"])).unwrap());
        assert!(
            trace_mode(&options(&["--trace", "yes", "--inputs", "tape", "--steps", "1", "--tail-policy", "parlor"]))
                .is_err()
        );
        assert!(
            trace_mode(&options(&["--trace", "yes", "--inputs", "tape", "--steps", "1", "--present", "replay"]))
                .is_err()
        );
        assert!(trace_mode(&options(&["--trace-reference", "reference"])).is_err());
        assert!(
            trace_mode(&options(&[
                "--trace",
                "yes",
                "--inputs",
                "tape",
                "--steps",
                "1",
                "--trace-reference",
                "reference",
                "--trace-reference-sha256",
                "invalid"
            ]))
            .is_err()
        );
    }

    #[test]
    fn external_entry_refuses_relative_paths_and_supervisor_commands_before_mutation() {
        let args = |command: &str| {
            [command, "--out", "relative", "--rom", "relative", "--library", "relative"]
                .into_iter()
                .map(OsString::from)
                .collect()
        };
        assert_eq!(run_external(args("run-worker")).unwrap_err(), "external entry requires absolute --out");
        let mut options = Options::parse(args("run")).unwrap();
        let absolute = std::env::current_dir().unwrap();
        for value in options.values.values_mut() {
            *value = absolute.as_os_str().to_owned();
        }
        let mut arguments = vec![OsString::from("run")];
        for (key, value) in options.values {
            arguments.extend([key.into(), value]);
        }
        assert_eq!(run_external(arguments).unwrap_err(), "external entry accepts prepare or run-worker only");
    }

    #[test]
    fn rio_capture_requires_a_finite_recorded_interval_without_authoring() {
        let parse =
            |arguments: &[&str]| Options::parse(arguments.iter().map(|s| OsString::from(*s)).collect()).unwrap();
        assert!(rio_trace_interval(&parse(&["run", "--rio-trace-start", "1"])).is_err());
        assert!(rio_trace_interval(&parse(&["run", "--rio-trace-start", "1", "--rio-trace-steps", "2"])).is_err());
        assert!(
            rio_trace_interval(&parse(&[
                "run",
                "--inputs",
                "tape",
                "--rio-trace-start",
                "1",
                "--rio-trace-steps",
                "2049"
            ]))
            .is_err()
        );
        assert!(
            rio_trace_interval(&parse(&[
                "run",
                "--inputs",
                "tape",
                "--rio-trace-start",
                "1",
                "--rio-trace-steps",
                "2",
                "--tail-policy",
                "parlor"
            ]))
            .is_err()
        );
        assert_eq!(
            rio_trace_interval(&parse(&[
                "run",
                "--inputs",
                "tape",
                "--rio-trace-start",
                "4",
                "--rio-trace-steps",
                "2"
            ]))
            .unwrap(),
            Some((4, 2))
        );
    }

    #[test]
    fn item_ownership_requires_every_body_and_real_room_load_completions() {
        let mut counts = sm_native_host::ItemInstructionCounters::default();
        assert!(!item_ownership_complete(counts));
        counts.scope_standalone.entries[..13].fill(1);
        counts.scope_standalone.rust_entries[..13].fill(1);
        counts.scope_standalone.rust_completed[..13].fill(1);
        counts.service_requests[..3].fill(1);
        counts.service_completions[..3].fill(1);
        assert!(!item_ownership_complete(counts));
        counts.scope_service52.entries[9] = 1;
        counts.scope_service52.rust_entries[9] = 1;
        counts.scope_service52.rust_completed[9] = 1;
        assert!(item_ownership_complete(counts));
        let mut wrong = counts;
        wrong.scope_standalone.c_entries[7] = 1;
        assert!(!item_ownership_complete(wrong));
        wrong = counts;
        wrong.scope_standalone.rust_completed[0] = 0;
        assert!(!item_ownership_complete(wrong));
        wrong = counts;
        wrong.service_completions[2] = 0;
        assert!(!item_ownership_complete(wrong));
        wrong = counts;
        wrong.scope_standalone.c_entries[16] = 1;
        assert!(!item_ownership_complete(wrong));
    }

    #[test]
    fn morph_ownership_requires_both_animation_roots_room_load_and_draw_services() {
        let mut counts = sm_native_host::ItemInstructionCounters::default();
        assert!(!morph_ownership_complete(counts));
        for i in MORPH_ENTRIES {
            counts.morph_standalone.entries[i] = 1;
            counts.morph_standalone.rust_entries[i] = 1;
            counts.morph_standalone.rust_completed[i] = 1;
        }
        counts.service_requests[1..6].fill(1);
        counts.service_completions[1..6].fill(1);
        assert!(!morph_ownership_complete(counts));
        counts.morph_service52.entries[13] = 1;
        counts.morph_service52.rust_entries[13] = 1;
        counts.morph_service52.rust_completed[13] = 1;
        assert!(morph_ownership_complete(counts));
        for i in [14, 15] {
            let mut wrong = counts;
            wrong.morph_standalone.rust_completed[i] = 0;
            assert!(!morph_ownership_complete(wrong));
        }
        let mut wrong = counts;
        wrong.morph_standalone.c_entries[16] = 1;
        assert!(!morph_ownership_complete(wrong));
        let mut wrong = counts;
        wrong.morph_service52.rust_refused[13] = 1;
        assert!(!morph_ownership_complete(wrong));
        let mut wrong = counts;
        wrong.service_completions[5] = 0;
        assert!(!morph_ownership_complete(wrong));
    }

    #[test]
    fn tank_observation_ownership_rejects_empty_partial_and_fallback_counts() {
        let mut counts = sm_native_host::ItemInstructionCounters::default();
        assert!(!tank_observed_entries_owned(counts));
        counts.tank_service52.entries[5] = 1;
        counts.tank_service52.rust_entries[5] = 1;
        assert!(!tank_observed_entries_owned(counts));
        counts.tank_service52.rust_completed[5] = 1;
        assert!(tank_observed_entries_owned(counts));
        // A shared initialization root does not imply any collector ran.
        assert!(counts.tank_service52.rust_completed[17..].iter().all(|&n| n == 0));
        let mut wrong = counts;
        wrong.tank_standalone.c_entries[19] = 1;
        assert!(!tank_observed_entries_owned(wrong));
        wrong = counts;
        wrong.tank_standalone.rust_refused[20] = 1;
        assert!(!tank_observed_entries_owned(wrong));
        wrong = counts;
        wrong.service_requests[6] = 1;
        assert!(!tank_observed_entries_owned(wrong));
    }

    #[test]
    fn deadline_kills_and_reaps_a_hung_child() {
        #[cfg(unix)]
        let mut child = Command::new("/bin/sleep").arg("60").spawn().unwrap();
        #[cfg(windows)]
        let mut child = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", "Start-Sleep -Seconds 60"])
            .spawn()
            .unwrap();
        let error = supervise(&mut child, Duration::from_millis(20)).unwrap_err();
        assert!(error.contains("exceeded"));
        assert!(!child.try_wait().unwrap().unwrap().success());
    }

    fn options(extra: &[&str]) -> Options {
        let mut args: Vec<OsString> =
            ["run", "--out", "unused", "--rom", "unused", "--library", "unused"].into_iter().map(Into::into).collect();
        args.extend(extra.iter().map(OsString::from));
        Options::parse(args).unwrap()
    }

    #[test]
    fn actor_selection_is_explicit_and_validated_before_run() {
        for option in ["--waver", "--ripper", "--rio"] {
            assert_eq!(options(&[]).text(option, "c").unwrap(), "c");
            assert_eq!(options(&["--gameplay", "rust"]).text(option, "c").unwrap(), "c");
            for owner in ["c", "rust"] {
                let run = ["run", "--rom", "unused", "--library", "unused", "--out", "unused", option, owner];
                assert!(validate(run.into_iter().map(Into::into).collect()).is_ok());
            }
            let run = ["run", "--rom", "unused", "--library", "unused", "--out", "unused", option, "invalid"];
            let expected = format!("{} must be", &option[2..]);
            assert!(validate(run.into_iter().map(Into::into).collect()).unwrap_err().contains(&expected));
            let prepare = ["prepare", "--reference-archive", "unused", "--out", "unused", option, "rust"];
            assert!(
                validate(prepare.into_iter().map(Into::into).collect()).unwrap_err().contains("prepare accepts only")
            );
        }
    }

    #[test]
    fn persistence_selection_inherits_gameplay_and_preserves_explicit_overrides() {
        for (args, expected) in [
            (vec![], "c"),
            (vec!["--gameplay", "rust"], "rust"),
            (vec!["--gameplay", "rust", "--persistence", "c"], "c"),
            (vec!["--gameplay", "c", "--persistence", "rust"], "rust"),
            (vec!["--persistence", "rust"], "rust"),
        ] {
            assert_eq!(persistence_owner(&options(&args)).unwrap(), expected, "{args:?}");
        }
        assert!(persistence_owner(&options(&["--persistence", "invalid"])).is_err());
        let prepare = ["prepare", "--reference-archive", "unused", "--out", "unused", "--persistence", "rust"];
        assert!(validate(prepare.into_iter().map(Into::into).collect()).unwrap_err().contains("prepare accepts only"));
        let run = ["run", "--rom", "unused", "--library", "unused", "--out", "unused", "--persistence", "invalid"];
        assert!(validate(run.into_iter().map(Into::into).collect()).unwrap_err().contains("persistence must be"));
    }

    #[test]
    fn parlor_policy_rejects_wrong_scenario_interactive_and_missing_tape() {
        for args in [
            vec!["--tail-policy", "parlor"],
            vec!["--scenario", "ceres", "--tail-policy", "parlor"],
            vec!["--scenario", "parlor", "--tail-policy", "other", "--inputs", "unused"],
            vec!["--scenario", "parlor", "--tail-policy", "parlor"],
            vec!["--scenario", "parlor", "--tail-policy", "parlor", "--inputs", "unused", "--present", "interactive"],
        ] {
            assert!(tape_mode(&options(&args)).is_err(), "{args:?}");
        }
        assert_eq!(
            tape_mode(&options(&["--scenario", "parlor", "--inputs", "unused"])).unwrap(),
            Some((tape_contract("parlor").unwrap(), false))
        );
        assert_eq!(
            tape_mode(&options(&["--scenario", "parlor", "--inputs", "unused", "--tail-policy", "parlor"])).unwrap(),
            Some((tape_contract("parlor").unwrap(), true))
        );
        assert_eq!(tape_mode(&options(&["--scenario", "ceres"])).unwrap(), None);
        assert_eq!(step_limit(&options(&["--scenario", "parlor", "--steps", "999999"])).unwrap(), 21087);
        assert_eq!(step_limit(&options(&["--scenario", "parlor", "--steps", "17000"])).unwrap(), 17000);
        assert_eq!(step_limit(&options(&["--scenario", "ceres"])).unwrap(), 20000);
    }

    #[test]
    fn authoring_rejects_extra_or_changed_prefix_before_game() {
        for name in ["parlor", "morph", "missile", "energy"] {
            let contract = tape_contract(name).unwrap();
            assert!(RecordedInputs::parse(b"000\n", contract, true).err().unwrap().contains("recorded pads"));
            let changed = "000\n".repeat(contract.prefix);
            assert!(RecordedInputs::parse(changed.as_bytes(), contract, true).err().unwrap().contains("exact frozen"));
            assert!(RecordedInputs::parse(changed.as_bytes(), contract, false).err().unwrap().contains("exact frozen"));
            let too_long = "000\n".repeat(contract.prefix + 1);
            assert!(
                RecordedInputs::parse(too_long.as_bytes(), contract, true).err().unwrap().contains("recorded pads")
            );
            let too_long_replay = "000\n".repeat(contract.prefix + contract.tail + 1);
            assert!(
                RecordedInputs::parse(too_long_replay.as_bytes(), contract, false)
                    .err()
                    .unwrap()
                    .contains("recorded pads")
            );
        }
    }

    #[test]
    fn replay_eof_never_becomes_policy_and_record_bytes_are_preserved() {
        let mut replay = RecordedInputs {
            contract: tape_contract("morph").unwrap(),
            pads: vec![0x20],
            records: vec!["020\n".into()],
            source: PathBuf::new(),
            sha256: String::new(),
            author: false,
            consumed: 0,
        };
        assert_eq!(replay.next().unwrap(), Some((0x20, "020\n".into())));
        assert!(replay.next().unwrap_err().contains("never invokes tail policy"));
        let mut author = RecordedInputs { author: true, ..replay };
        assert_eq!(author.next().unwrap(), None);
    }

    #[test]
    fn morph_contract_rejects_other_tail_and_preserves_parlor_limits() {
        for (scenario, policy) in [("morph", "parlor"), ("parlor", "morph")] {
            assert!(
                tape_mode(&options(&["--scenario", scenario, "--inputs", "unused", "--tail-policy", policy])).is_err()
            );
        }
        let contract = tape_contract("morph").unwrap();
        assert_eq!(
            tape_mode(&options(&["--scenario", "morph", "--inputs", "unused", "--tail-policy", "morph"])).unwrap(),
            Some((contract, true))
        );
        assert_eq!(
            (contract.prefix, contract.tail, contract.sha256),
            (18072, 8192, "125656a5c9d6bd040472d07ea23ac72f93c72ff6c54c092a61bb662b298b57bb")
        );
        assert_eq!(step_limit(&options(&["--scenario", "morph"])).unwrap(), 26264);
        assert_eq!(step_limit(&options(&["--scenario", "morph", "--steps", "999999"])).unwrap(), 26264);
        assert_eq!(step_limit(&options(&["--scenario", "morph", "--steps", "18073"])).unwrap(), 18073);
        assert_eq!(step_limit(&options(&["--scenario", "parlor"])).unwrap(), 21087);
    }

    #[test]
    fn missile_contract_is_bounded_and_rejects_cross_scenario_policy() {
        let contract = tape_contract("missile").unwrap();
        assert_eq!(
            (contract.prefix, contract.tail, contract.sha256),
            (23036, 8192, "0a8ab6a50be742c1124aea5b802ae088a3b12cd785b7da54aea6cfc6143ae76a")
        );
        assert_eq!(step_limit(&options(&["--scenario", "missile"])).unwrap(), 31228);
        assert_eq!(step_limit(&options(&["--scenario", "missile", "--steps", "999999"])).unwrap(), 31228);
        assert_eq!(step_limit(&options(&["--scenario", "missile", "--steps", "23037"])).unwrap(), 23037);
        assert_eq!(
            tape_mode(&options(&["--scenario", "missile", "--inputs", "unused"])).unwrap(),
            Some((contract, false))
        );
        assert_eq!(
            tape_mode(&options(&["--scenario", "missile", "--inputs", "unused", "--tail-policy", "missile"])).unwrap(),
            Some((contract, true))
        );
        for (scenario, policy) in [("missile", "morph"), ("morph", "missile"), ("parlor", "missile")] {
            assert!(
                tape_mode(&options(&["--scenario", scenario, "--inputs", "unused", "--tail-policy", policy])).is_err()
            );
        }
    }

    #[test]
    fn energy_navigation_stop_is_author_only_and_survives_ownership_rejection() {
        let state = State { host_steps: 28000, room: 0x9f11, x: 130, y: 409, ..State::default() };
        assert!(energy_navigation_stop(false, true, &state).is_none());
        assert!(energy_navigation_stop(true, false, &state).is_none());
        assert!(energy_navigation_stop(false, false, &state).is_none());
        let reason = energy_navigation_stop(true, true, &state).unwrap();
        assert_eq!(reason, "energy authoring navigation blocked at step 28000 room 9f11 x=130 y=409");
        assert_eq!(ownership_failure_reason("energy", reason.clone(), "collector incomplete"), reason);
        assert_eq!(ownership_failure_reason("energy", String::new(), "collector incomplete"), "collector incomplete");
        assert_eq!(
            ownership_failure_reason("missile", "previous".into(), "collector incomplete"),
            "collector incomplete"
        );
    }

    #[test]
    fn energy_contract_bounds_tape_deadline_and_policy() {
        let contract = tape_contract("energy").unwrap();
        assert_eq!(
            (contract.prefix, contract.tail, contract.sha256),
            (24832, 16384, "43d67592cb3da2a9dd9c1568f609aa19b60bcb555a28b643453427677908be69")
        );
        assert_eq!(step_limit(&options(&["--scenario", "energy"])).unwrap(), 41216);
        assert_eq!(step_limit(&options(&["--scenario", "energy", "--steps", "999999"])).unwrap(), 41216);
        assert_eq!(step_limit(&options(&["--scenario", "energy", "--steps", "24833"])).unwrap(), 24833);
        assert_eq!(deadline_seconds(&options(&["--scenario", "energy"])).unwrap(), 90);
        assert_eq!(deadline_seconds(&options(&["--scenario", "energy", "--deadline-seconds", "900"])).unwrap(), 90);
        assert_eq!(deadline_seconds(&options(&["--scenario", "energy", "--deadline-seconds", "30"])).unwrap(), 30);
        assert_eq!(deadline_seconds(&options(&["--scenario", "missile"])).unwrap(), 60);
        assert_eq!(
            tape_mode(&options(&["--scenario", "energy", "--inputs", "unused"])).unwrap(),
            Some((contract, false))
        );
        assert_eq!(
            tape_mode(&options(&["--scenario", "energy", "--inputs", "unused", "--tail-policy", "energy"])).unwrap(),
            Some((contract, true))
        );
        for args in [
            vec!["--scenario", "energy"],
            vec!["--scenario", "energy", "--inputs", "unused", "--tail-policy", "missile"],
            vec!["--scenario", "missile", "--inputs", "unused", "--tail-policy", "energy"],
            vec!["--scenario", "energy", "--inputs", "unused", "--present", "interactive"],
        ] {
            assert!(tape_mode(&options(&args)).is_err(), "{args:?}");
        }
        let mut play = options(&["--scenario", "energy", "--inputs", "unused"]);
        play.command = "play".into();
        assert!(tape_mode(&play).is_err());
    }

    #[test]
    fn energy_observer_starts_at_boundary_and_prefix_windows_stop() {
        let mut energy = Energy::default();
        let mut before = sm_binary_evidence::Map::new();
        energy.report(&mut before);
        for step in [1, 23036, 24831] {
            observe_energy(&mut energy, &State { host_steps: step, ..State::default() }, &[]);
        }
        let mut after = sm_binary_evidence::Map::new();
        energy.report(&mut after);
        assert_eq!(before, after);
        observe_energy(&mut energy, &State { host_steps: 24832, ..State::default() }, &[0; 0x20000]);
        energy.report(&mut after);
        assert_ne!(before, after); // Boundary reaches policy and rejects this synthetic endpoint.
        for (step, morph, missile) in [
            (23035, true, false),
            (23036, true, true),
            (23037, false, true),
            (24832, false, true),
            (24833, false, false),
        ] {
            assert_eq!(morph_observation_step("energy", step), morph);
            assert_eq!(missile_observation_step("energy", step), missile);
        }
        assert!(missile_observation_step("missile", 24833));
    }

    #[test]
    fn energy_ownership_requires_collector17_and_every_observed_callback() {
        let mut counts = sm_native_host::ItemInstructionCounters::default();
        assert!(!energy_ownership_complete(counts));
        for entry in [5, 18] {
            counts.tank_service52.entries[entry] = 1;
            counts.tank_service52.rust_entries[entry] = 1;
            counts.tank_service52.rust_completed[entry] = 1;
            assert!(tank_observed_entries_owned(counts));
            assert!(!energy_ownership_complete(counts));
        }
        counts.tank_standalone.entries[17] = 1;
        counts.tank_standalone.rust_entries[17] = 1;
        assert!(!energy_ownership_complete(counts));
        counts.tank_standalone.rust_completed[17] = 1;
        assert!(energy_ownership_complete(counts));
        let mut wrong = counts;
        wrong.tank_service52.c_entries[5] = 1;
        assert!(!energy_ownership_complete(wrong));
        wrong = counts;
        wrong.tank_service52.rust_refused[5] = 1;
        assert!(!energy_ownership_complete(wrong));
        wrong = counts;
        wrong.service_requests[6] = 1;
        assert!(!energy_ownership_complete(wrong));
    }

    #[test]
    fn energy_tail_cannot_rescue_missing_or_incomplete_prefix_ownership() {
        let prefix = sm_native_host::ItemInstructionCounters::default();
        let mut final_counts = prefix;
        for i in 0..13 {
            final_counts.scope_standalone.entries[i] = 1;
            final_counts.scope_standalone.rust_entries[i] = 1;
            final_counts.scope_standalone.rust_completed[i] = 1;
        }
        final_counts.scope_service52.entries[9] = 1;
        final_counts.scope_service52.rust_entries[9] = 1;
        final_counts.scope_service52.rust_completed[9] = 1;
        final_counts.service_requests[..3].fill(1);
        final_counts.service_completions[..3].fill(1);
        assert!(item_ownership_complete(final_counts));
        for captured in [None, Some(prefix)] {
            assert!(!item_ownership_complete(acceptance_counts("energy", captured, final_counts)));
        }
        assert!(item_ownership_complete(acceptance_counts("energy", Some(final_counts), prefix)));
        assert!(item_ownership_complete(acceptance_counts("missile", None, final_counts)));
    }

    #[test]
    fn energy_snapshots_and_csv_retain_exact_slot_and_upper_bank_terrain() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!("energy-runtime-{}-{nonce}", std::process::id())));
        let out = &scratch.0;
        fs::create_dir(out).unwrap();
        let mut snapshots = EnergySnapshots::default();
        let mut ram = vec![0; 0x20000];
        snapshots.observe(out, 24831, &ram, true, true).unwrap();
        assert!(!snapshots.prefix && snapshots.present.is_none() && snapshots.acquired.is_none());
        assert!(snapshots.observe(out, 24832, &ram[..0x10000], false, false).is_err());
        snapshots.observe(out, 24832, &ram, false, false).unwrap();
        let prefix = ram.clone();
        ram[0] = 1;
        snapshots.observe(out, 24833, &ram, true, false).unwrap();
        let present = ram.clone();
        ram[0] = 2;
        snapshots.observe(out, 24834, &ram, true, true).unwrap();
        let acquired = ram.clone();
        ram[0] = 3;
        snapshots.observe(out, 24835, &ram, true, true).unwrap();
        snapshots.observe(out, 24832, &ram, false, false).unwrap();
        for (name, expected) in [("prefix", prefix), ("present", present), ("acquired", acquired)] {
            assert_eq!(fs::read(out.join(format!("energy-{name}.wram"))).unwrap(), expected);
        }
        let mut report = sm_binary_evidence::Map::new();
        snapshots.report(out, &mut report).unwrap();
        assert_eq!(report["energy_present_wram_step"].as_integer(), Some(24833));
        assert_eq!(report["energy_acquired_wram_step"].as_integer(), Some(24834));
        for (at, word) in
            [(0x1c37, 0xeed7u16), (0x1dc7, 8), (0x1c87, 0x1f8e), (0x11f90, 0x1234), (0x9c2, 65), (0x9c4, 99)]
        {
            ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        ram[0x173c9] = 0x56;
        ram[0x73c9] = 0x99;
        let row = |ram: &[u8]| {
            let mut bytes = Vec::new();
            record_energy(&mut bytes, &State::default(), ram, true, false).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            let fields: Vec<_> = text.trim().split(',').map(str::to_owned).collect();
            assert_eq!(fields.len(), ENERGY_HEADER.split(',').count());
            ENERGY_HEADER.split(',').zip(fields).collect::<BTreeMap<_, _>>()
        };
        let fields = row(&ram);
        for (name, value) in [
            ("health", "65"),
            ("maximum", "99"),
            ("plm_slot", "0"),
            ("plm_header", "eed7"),
            ("plm_argument", "0008"),
            ("plm_block_index", "1f8e"),
            ("plm_block_valid", "true"),
            ("plm_block_word", "1234"),
            ("plm_bts", "56"),
            ("target_bts", "56"),
        ] {
            assert_eq!(fields[name], value, "{name}");
        }
        ram[0x1c87] = 0x8f;
        assert_eq!(row(&ram)["plm_block_valid"], "false");
        for (at, word) in [(0x1c39, 0xeed7u16), (0x1dc9, 8)] {
            ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        assert_eq!(row(&ram)["plm_slot"], "-2");
        ram[0x1dc7] = 9;
        ram[0x1dc9] = 9;
        assert_eq!(row(&ram)["plm_slot"], "-1");
        assert!(record_energy(&mut Vec::new(), &State::default(), &ram[..0x10000], false, false).is_err());
    }

    #[test]
    fn missile_observer_first_receives_its_prefix_boundary() {
        let mut missile = Missile::default();
        let mut ram = vec![0; 0x20000];
        observe_missile(&mut missile, &State { host_steps: 1, ..State::default() }, &ram);
        observe_missile(&mut missile, &State { host_steps: 23035, ..State::default() }, &ram);
        for (at, value) in [(0x9c2, 99u16), (0x9a2, 4), (0x9a4, 4)] {
            ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        let state = State {
            host_steps: 23036,
            game_state: 8,
            loading_game_state: 5,
            area: 1,
            room: 0x9e9f,
            x: 1108,
            y: 651,
            polls: 21426,
            updates: 21414,
            nmis: 23036,
            waits: 1622,
            requested_buttons: 0x41,
            effective_buttons: 0x41,
            joy1_current: 0x8200,
            ..State::default()
        };
        observe_missile(&mut missile, &state, &ram);
        let mut report = sm_binary_evidence::Map::new();
        missile.report(&mut report);
        assert_eq!(report["missile_prefix_verified"].as_bool(), Some(true));
        assert_eq!(report["missile_invalid"].as_bool(), Some(false));
    }

    #[test]
    fn missile_snapshots_preserve_first_observer_events_and_plm_evidence() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!("missile-runtime-{}-{nonce}", std::process::id())));
        let out = &scratch.0;
        fs::create_dir(out).unwrap();
        let mut ram = vec![0; 0x20000];
        let mut snapshots = MissileSnapshots::default();
        snapshots.observe(out, 23035, &ram, false, false).unwrap();
        assert!(!snapshots.prefix);
        snapshots.observe(out, 23036, &ram, false, false).unwrap();
        assert_eq!(fs::read(out.join("missile-prefix.wram")).unwrap(), ram);
        ram[0] = 1;
        snapshots.observe(out, 23037, &ram, true, false).unwrap();
        let revealed = ram.clone();
        ram[0] = 2;
        snapshots.observe(out, 23038, &ram, true, true).unwrap();
        let acquired = ram.clone();
        ram[0] = 3;
        snapshots.observe(out, 23039, &ram, true, true).unwrap();
        assert_eq!((snapshots.revealed, snapshots.acquired), (Some(23037), Some(23038)));
        assert_eq!(fs::read(out.join("missile-revealed.wram")).unwrap(), revealed);
        assert_eq!(fs::read(out.join("missile-acquired.wram")).unwrap(), acquired);
        for (at, word) in
            [(0x1c37, 0xef2fu16), (0x1dc7, 0x22), (0x1c87, 0xe8), (0x100ea, 0x1234), (0x9c6, 5), (0x9c8, 5)]
        {
            ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        ram[0x16476] = 0x56;
        ram[0x6476] = 0x99; // A lower-bank alias is not the BTS array.
        let mut row = Vec::new();
        record_missile(&mut row, &State::default(), &ram, true, false).unwrap();
        let row = String::from_utf8(row).unwrap();
        let fields: Vec<_> = row.trim().split(',').collect();
        assert_eq!(fields.len(), MISSILE_HEADER.split(',').count());
        assert_eq!(&fields[19..21], &["5", "5"]);
        assert_eq!(&fields[27..33], &["true", "false", "0", "ef2f", "0022", "00e8"]);
        assert_eq!(&fields[40..], &["true", "1234", "56"]);
        for (at, word) in [(0x1c39, 0xef2fu16), (0x1dc9, 0x22)] {
            ram[at..at + 2].copy_from_slice(&word.to_le_bytes());
        }
        let mut row = Vec::new();
        record_missile(&mut row, &State::default(), &ram, false, false).unwrap();
        let row = String::from_utf8(row).unwrap();
        let fields: Vec<_> = row.trim().split(',').collect();
        assert_eq!(fields[29], "-2");
        assert_eq!(fields[40], "false");
    }

    #[test]
    fn item_snapshots_keep_boundary_and_first_complete_three_bit_observation() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!("morph-runtime-{}-{nonce}", std::process::id())));
        let out = &scratch.0;
        fs::create_dir(out).unwrap();
        let mut ram = vec![0; 0x20000];
        let mut prefix = false;
        let mut acquired = None;
        morph_snapshots(out, 18071, &ram, &mut prefix, &mut acquired).unwrap();
        assert!(!prefix && acquired.is_none());
        morph_snapshots(out, 18072, &ram, &mut prefix, &mut acquired).unwrap();
        assert_eq!(fs::read(out.join("morph-prefix.wram")).unwrap(), ram);
        ram[0x9a2] = 4;
        ram[0x9a4] = 4;
        morph_snapshots(out, 18073, &ram, &mut prefix, &mut acquired).unwrap();
        assert!(acquired.is_none());
        ram[0xd873] = 4;
        morph_snapshots(out, 18074, &ram, &mut prefix, &mut acquired).unwrap();
        assert_eq!(acquired, Some(18074));
        let first = ram.clone();
        ram[0x9a3] = 0x80;
        morph_snapshots(out, 18075, &ram, &mut prefix, &mut acquired).unwrap();
        assert_eq!(fs::read(out.join("morph-acquired.wram")).unwrap(), first);
        let mut row = Vec::new();
        record_item(&mut row, &State { host_steps: 18075, ..State::default() }, &ram).unwrap();
        let row = String::from_utf8(row).unwrap();
        let fields: Vec<_> = row.trim().split(',').collect();
        assert_eq!(fields.len(), ITEM_HEADER.split(',').count());
        assert_eq!(&fields[15..18], &["8004", "0004", "04"]);
    }
}
