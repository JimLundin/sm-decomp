//! Explicit gameplay experiments sharing one comparison loop.
use sha2::{Digest, Sha256};
use sm_native_host::Game;
use sm_trace::{Identity, Reference, Writer};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode};
use std::time::{Duration, Instant};

#[path = "sm-pilot/camera.rs"]
mod camera;
#[path = "sm-pilot/ceres_door.rs"]
mod ceres_door;
#[path = "sm-pilot/door_scrolling.rs"]
mod door_scrolling;
#[path = "sm-pilot/escape.rs"]
mod escape;
#[path = "sm-pilot/room_transition.rs"]
mod room_transition;
#[path = "sm-pilot/scrolling.rs"]
mod scrolling;

type Result<T> = std::result::Result<T, String>;
const PROJECTILE_SCENARIO: &str = "ceres-platform-ordinary-shots-v2";
const SCROLL_SCENARIO: &str = "ceres-room-scrolling-v1";
const DOOR_SCENARIO: &str = "ceres-door-lifecycle-v1";
const TRANSITION_SCENARIO: &str = "ceres-room-transitions-v1";
const DIRECTIONAL_SCENARIO: &str = "ceres-directional-door-scrolling-v1";
const CAMERA_SCENARIO: &str = "ceres-normal-camera-v1";
const ESCAPE_SCENARIO: &str = "ceres-projectile-update-escape-v1";
const USAGE: &str = "Usage: sm-pilot reference --library PATH --rom PATH --prefix INPUTS --out NEW_DIR
       sm-pilot check --library PATH --rom PATH --reference DIR --out NEW_DIR
       sm-pilot dev --library PATH --rom PATH --reference DIR --out NEW_DIR
Options: --scenario projectiles|scrolling|ceres-doors|room-transition|door-scrolling|camera|projectile-escape (default projectiles),
         --fault yes (deliberate gameplay fault), --mode c (fresh C control),
         --capture-step N (retain exact state for mismatch diagnosis).
Projectile reference appends shots; scrolling reference uses the saved route.
Dev builds the candidate incrementally and reuses unchanged passing results.
All gameplay starts from reset; original-ROM comparison is not claimed.";

struct Options {
    action: String,
    values: BTreeMap<String, String>,
}
impl Options {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let action = args.next().ok_or(USAGE)?.to_owned();
        if !matches!(action.as_str(), "reference" | "check" | "dev" | "worker") {
            return Err(USAGE.into());
        }
        let mut values = BTreeMap::new();
        while let Some(key) = args.next() {
            if !matches!(
                key.as_str(),
                "--library"
                    | "--rom"
                    | "--prefix"
                    | "--out"
                    | "--reference"
                    | "--mode"
                    | "--fault"
                    | "--capture-step"
                    | "--scenario"
            ) {
                return Err(format!("unknown {key}\n{USAGE}"));
            }
            let value =
                args.next().filter(|s| !s.starts_with("--")).ok_or_else(|| format!("missing value for {key}"))?;
            if values.insert(key.clone(), value).is_some() {
                return Err(format!("duplicate {key}"));
            }
        }
        let options = Self { action, values };
        if !matches!(
            options.value("--scenario", "projectiles"),
            "projectiles"
                | "scrolling"
                | "ceres-doors"
                | "room-transition"
                | "door-scrolling"
                | "camera"
                | "projectile-escape"
        ) {
            return Err(
                "scenario must be projectiles, scrolling, ceres-doors, room-transition, door-scrolling, camera or projectile-escape"
                    .into(),
            );
        }
        for key in ["--library", "--rom", "--out"] {
            options.path(key)?;
        }
        if !matches!(options.value("--mode", "rust"), "rust" | "c") {
            return Err("mode must be rust or c".into());
        }
        if !matches!(options.value("--fault", "no"), "yes" | "no") {
            return Err("fault must be yes or no".into());
        }
        if options.value("--capture-step", "0").parse::<usize>().is_err() {
            return Err("capture-step must be an integer".into());
        }
        if options.action == "reference" {
            options.path("--prefix")?;
        } else {
            options.path("--reference")?;
        }
        Ok(options)
    }
    fn path(&self, key: &str) -> Result<PathBuf> {
        self.values.get(key).map(PathBuf::from).ok_or_else(|| format!("missing {key}"))
    }
    fn value<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.values.get(key).map(String::as_str).unwrap_or(default)
    }
    fn scrolling(&self) -> bool {
        matches!(
            self.value("--scenario", "projectiles"),
            "scrolling" | "ceres-doors" | "room-transition" | "door-scrolling" | "camera" | "projectile-escape"
        )
    }
    fn doors(&self) -> bool {
        matches!(
            self.value("--scenario", "projectiles"),
            "ceres-doors" | "room-transition" | "door-scrolling" | "camera" | "projectile-escape"
        )
    }
    fn transitions(&self) -> bool {
        matches!(
            self.value("--scenario", "projectiles"),
            "room-transition" | "door-scrolling" | "camera" | "projectile-escape"
        )
    }
    fn directional(&self) -> bool {
        matches!(self.value("--scenario", "projectiles"), "door-scrolling" | "camera" | "projectile-escape")
    }
    fn camera(&self) -> bool {
        matches!(self.value("--scenario", "projectiles"), "camera" | "projectile-escape")
    }
    fn escape(&self) -> bool {
        self.value("--scenario", "projectiles") == "projectile-escape"
    }
    fn scenario(&self) -> &'static str {
        if self.escape() {
            ESCAPE_SCENARIO
        } else if self.camera() {
            CAMERA_SCENARIO
        } else if self.directional() {
            DIRECTIONAL_SCENARIO
        } else if self.transitions() {
            TRANSITION_SCENARIO
        } else if self.doors() {
            DOOR_SCENARIO
        } else if self.scrolling() {
            SCROLL_SCENARIO
        } else {
            PROJECTILE_SCENARIO
        }
    }
}
fn main() -> ExitCode {
    if matches!(std::env::args().nth(1).as_deref(), Some("--help" | "-h")) {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let options = match Options::parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    match if options.action == "worker" { worker(&options) } else { supervise(&options) } {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
fn hash(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)))
}
fn read_inputs(path: &Path) -> Result<Vec<u16>> {
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if data.len() > 200_000 {
        return Err("input file exceeds pilot bound".into());
    }
    let inputs: Vec<u16> = data
        .lines()
        .map(|line| {
            u16::from_str_radix(line.trim(), 16).ok().filter(|v| *v <= 0xfff).ok_or("invalid native pad".into())
        })
        .collect::<Result<_>>()?;
    if inputs.is_empty() || inputs.len() > 40_000 {
        return Err("pilot input count outside 1..40000".into());
    }
    Ok(inputs)
}
fn write_report(path: &Path, report: &sm_binary_evidence::Map) -> Result<()> {
    fs::write(path, sm_binary_evidence::encode(&sm_binary_evidence::Value::Table(report.clone()))?)
        .map_err(|e| e.to_string())
}
fn identity(library: &Path, rom: &Path, inputs: &Path, scenario: &str) -> Result<Identity> {
    // The prepared manifest binds the fixed Rust host recipe as well as C.
    // Rust-only host changes can leave the compiled C library byte-identical.
    // Candidate gameplay remains outside this reference identity.
    let manifest = library.parent().ok_or("native library has no parent")?.join("library.sme");
    let host = format!("{}:{}", hash(library)?, hash(&manifest)?);
    Ok(Identity {
        reference: sm_native_host::SOURCE_REVISION.into(),
        rom: hash(rom)?,
        host: format!("{:x}", Sha256::digest(host)),
        input: hash(inputs)?,
        observation: "native-pilot-wram-external-effects-v1".into(),
        scenario: scenario.into(),
    })
}
fn wait(child: &mut Child, limit: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return if status.success() { Ok(()) } else { Err(format!("child failed: {status}")) },
            Ok(None) if start.elapsed() < limit => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(e) => e.to_string(),
                    _ => "pilot deadline exceeded".into(),
                });
            }
        }
    }
}

fn reusable_result(cache: &Path, key: &str) -> Option<String> {
    let previous = sm_binary_evidence::read(cache).ok()?;
    if previous.get("key")?.as_str()? != key {
        return None;
    }
    let path = previous.get("result")?.as_str()?;
    let digest = previous.get("sha256")?.as_str()?;
    (hash(Path::new(path)).ok()?.as_str() == digest).then(|| path.to_owned())
}
fn supervise(options: &Options) -> Result<()> {
    let start = Instant::now();
    let out = options.path("--out")?;
    fs::create_dir(&out).map_err(|e| format!("fresh output {}: {e}", out.display()))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;
    let library = options.path("--library")?.canonicalize().map_err(|e| e.to_string())?;
    let rom = options.path("--rom")?.canonicalize().map_err(|e| e.to_string())?;
    let mut exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut report = sm_binary_evidence::Map::new();
    report.insert("schema".into(), "sm-pilot-invocation-v1".into());
    report.insert("action".into(), options.action.clone().into());
    report.insert("scenario".into(), options.scenario().into());
    report.insert("original_rom_comparison".into(), "not-tested".into());
    let build_start = Instant::now();
    if options.action == "dev" {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut build = Command::new("cargo")
            .current_dir(workspace)
            .args(["build", "--locked", "--offline", "-j1", "--bin", "sm-pilot", "--target-dir"])
            .arg(workspace.join("target"))
            .stdout(File::create(out.join("build.stdout.log")).map_err(|e| e.to_string())?)
            .stderr(File::create(out.join("build.stderr.log")).map_err(|e| e.to_string())?)
            .spawn()
            .map_err(|e| e.to_string())?;
        wait(&mut build, Duration::from_secs(120))?;
        exe = workspace.join("target/debug/sm-pilot");
    }
    report.insert("build_seconds".into(), build_start.elapsed().as_secs_f64().into());
    let validation = Instant::now();
    sm_native_host::authenticate_library(&library)?;
    let reference = if options.action == "reference" {
        let mut inputs = read_inputs(&options.path("--prefix")?)?;
        if options.escape() {
            inputs = escape::author_inputs(inputs)?;
        } else if options.scrolling() {
            inputs = scrolling::author_inputs(inputs)?;
        } else {
            // A released interval makes the firing edge explicit. The left turn
            // lasts one step; all shots complete at the initial Ceres platform.
            inputs.extend([0; 10]);
            inputs.push(0x200);
            inputs.extend([0; 120]);
            inputs.push(0x040);
            inputs.extend([0; 16]);
            inputs.push(0x200);
            inputs.extend([0; 120]);
            inputs.extend([0x010; 8]);
            inputs.push(0x210);
            inputs.extend([0; 120]);
        }
        let text: String = inputs.iter().map(|v| format!("{v:03x}\n")).collect();
        fs::write(out.join("inputs.txt"), text).map_err(|e| e.to_string())?;
        out.clone()
    } else {
        options.path("--reference")?.canonicalize().map_err(|e| e.to_string())?
    };
    let trace = reference.join("trace.bin");
    let inputs = reference.join("inputs.txt");
    if options.action != "reference" {
        Reference::load(&trace, &identity(&library, &rom, &inputs, options.scenario())?).map_err(|e| e.to_string())?;
    }
    let mut key = format!(
        "{}:{}:{}:{}:{}:{}",
        hash(&exe)?,
        hash(&library)?,
        hash(&rom)?,
        hash(&inputs)?,
        options.value("--mode", "rust"),
        options.value("--fault", "no")
    );
    if options.action != "reference" {
        key.push_str(&hash(&trace)?);
    }
    let key = format!("{:x}", Sha256::digest(key));
    let cache_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("out/pilot-dev-cache.toml");
    report.insert("validation_seconds".into(), validation.elapsed().as_secs_f64().into());
    report.insert("candidate_key".into(), key.clone().into());
    report.insert("executable_sha256".into(), hash(&exe)?.into());
    report.insert("native_sha256".into(), hash(&library)?.into());
    report.insert("reference".into(), reference.to_string_lossy().into_owned().into());
    if options.action == "dev"
        && !options.values.contains_key("--capture-step")
        && let Some(path) = reusable_result(&cache_path, &key)
    {
        report.insert("status".into(), "reused".into());
        report.insert("reused_result".into(), path.clone().into());
        report.insert("game_processes".into(), 0.into());
        report.insert("elapsed_seconds".into(), start.elapsed().as_secs_f64().into());
        write_report(&out.join("invocation.sme"), &report)?;
        println!("unchanged candidate: reused {path}; no game process ({:.3}s)", start.elapsed().as_secs_f64());
        return Ok(());
    }
    let mode = if options.action == "reference" { "reference" } else { options.value("--mode", "rust") };
    let mut args: Vec<OsString> = vec![
        "worker".into(),
        "--library".into(),
        library.into(),
        "--rom".into(),
        rom.into(),
        "--out".into(),
        out.clone().into(),
        "--reference".into(),
        reference.into(),
        "--mode".into(),
        if mode == "reference" { "c" } else { mode }.into(),
    ];
    args.extend(["--scenario".into(), options.value("--scenario", "projectiles").into()]);
    if options.action == "reference" {
        args.extend(["--prefix".into(), "create-reference".into()]);
    }
    if options.value("--fault", "no") == "yes" {
        args.extend(["--fault".into(), "yes".into()]);
    }
    if let Some(step) = options.values.get("--capture-step") {
        args.extend(["--capture-step".into(), step.into()]);
    }
    report.insert(
        "argv".into(),
        sm_binary_evidence::Value::Array(args.iter().map(|v| v.to_string_lossy().into_owned().into()).collect()),
    );
    write_report(&out.join("invocation.sme"), &report)?;
    let run = Instant::now();
    let mut child = Command::new(&exe)
        .args(args)
        .stdout(File::create(out.join("stdout.log")).map_err(|e| e.to_string())?)
        .stderr(File::create(out.join("stderr.log")).map_err(|e| e.to_string())?)
        .spawn()
        .map_err(|e| e.to_string())?;
    let result = wait(&mut child, Duration::from_secs(90));
    report.insert("run_seconds".into(), run.elapsed().as_secs_f64().into());
    report.insert("game_processes".into(), 1.into());
    report.insert("status".into(), if result.is_ok() { "passed" } else { "failed" }.into());
    report.insert("elapsed_seconds".into(), start.elapsed().as_secs_f64().into());
    write_report(&out.join("invocation.sme"), &report)?;
    if result.is_ok() && options.action == "dev" {
        let mut cache = sm_binary_evidence::Map::new();
        cache.insert("key".into(), key.into());
        cache.insert("result".into(), out.join("result.sme").to_string_lossy().into_owned().into());
        cache.insert("sha256".into(), hash(&out.join("result.sme"))?.into());
        write_report(&cache_path, &cache)?;
    }
    println!(
        "{}: {:.3}s; {}",
        if result.is_ok() { "passed" } else { "failed" },
        start.elapsed().as_secs_f64(),
        out.display()
    );
    result.map_err(|e| format!("{e}; inspect {}/stderr.log", out.display()))
}

#[derive(Default, Clone, Copy)]
struct Shot {
    moved: bool,
    impacted: bool,
    drawn: bool,
    explosion_drawn: bool,
}
#[derive(Default)]
struct Lifecycle {
    previous: [(u16, u16, u16, u16); 5],
    shots: [Shot; 5],
    drawn: [u64; 4],
    spawned: u64,
    moved: u64,
    impacted: u64,
    removed: u64,
    complete_shots: u64,
    overlapping_shots: bool,
}
impl Lifecycle {
    fn observe(&mut self, ram: &[u8], drawn: [u64; 4]) {
        let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
        self.overlapping_shots |= (0..5).filter(|i| word(0xc40 + i * 2) != 0).count() > 1;
        for i in 0..5 {
            let now = (word(0xc40 + i * 2), word(0xc18 + i * 2), word(0xb64 + i * 2), word(0xb78 + i * 2));
            let before = self.previous[i];
            if before.0 == 0 && now.0 != 0 && now.1 == 0x8000 {
                self.spawned += 1;
                self.shots[i] = Shot::default();
            }
            if before.0 != 0
                && now.0 != 0
                && before.1 == 0x8000
                && now.1 == 0x8000
                && (before.2, before.3) != (now.2, now.3)
            {
                self.moved += 1;
                self.shots[i].moved = true;
            }
            if before.1 == 0x8000 && now.1 == 0x8700 {
                self.impacted += 1;
                self.shots[i].impacted = true;
            }
            // This authored scenario has at most one active shot. The draw
            // counters measure actual OAM emission around each scheduled entry.
            if now.0 != 0 && now.1 == 0x8000 && drawn[2] > self.drawn[2] {
                self.shots[i].drawn = true;
            }
            if now.0 != 0 && now.1 == 0x8700 && drawn[3] > self.drawn[3] {
                self.shots[i].explosion_drawn = true;
            }
            if before.0 != 0 && now.0 == 0 {
                self.removed += 1;
                let shot = self.shots[i];
                if shot.moved && shot.impacted && shot.drawn && shot.explosion_drawn {
                    self.complete_shots += 1;
                }
            }
            self.previous[i] = now;
        }
        self.drawn = drawn;
    }
    fn complete(&self) -> bool {
        !self.overlapping_shots
            && self.spawned >= 3
            && self.complete_shots >= 2
            && self.removed >= 3
            && self.previous.iter().all(|v| v.0 == 0)
    }
}
fn screenshot(path: &Path, pixels: &[u32]) -> Result<()> {
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path).map_err(|e| e.to_string())?), 256, 240);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = pixels.iter().flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8]).collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
fn worker(options: &Options) -> Result<()> {
    let library = options.path("--library")?;
    let rom = options.path("--rom")?;
    let out = options.path("--out")?;
    let reference = options.path("--reference")?;
    let inputs_path = reference.join("inputs.txt");
    let inputs = read_inputs(&inputs_path)?;
    let id = identity(&library, &rom, &inputs_path, options.scenario())?;
    let recording = options.values.contains_key("--prefix");
    let reference = if recording {
        None
    } else {
        Some(Reference::load(reference.join("trace.bin"), &id).map_err(|e| e.to_string())?)
    };
    let mut compare = reference.as_ref().map(|r| r.comparison());
    let mut writer =
        if recording { Some(Writer::create(out.join("trace.bin"), &id).map_err(|e| e.to_string())?) } else { None };
    fs::create_dir(out.join("saves")).map_err(|e| e.to_string())?;
    std::env::set_current_dir(&out).map_err(|e| e.to_string())?;
    let mut game = Game::boot(&library, &rom)?;
    let rust = options.value("--mode", "rust") == "rust";
    if rust {
        sm_restart::gameplay::install_scrolling(
            &mut game,
            options.scrolling() && !options.doors() && options.value("--fault", "no") == "yes",
        )?;
        sm_restart::gameplay::install_ceres_doors(
            &mut game,
            options.doors() && !options.transitions() && options.value("--fault", "no") == "yes",
        )?;
        sm_restart::gameplay::install_room_transitions(
            &mut game,
            options.transitions() && !options.directional() && options.value("--fault", "no") == "yes",
        )?;
        sm_restart::gameplay::install_door_scrolling(
            &mut game,
            options.directional() && !options.camera() && options.value("--fault", "no") == "yes",
        )?;
        sm_restart::gameplay::install_camera(
            &mut game,
            options.camera() && !options.escape() && options.value("--fault", "no") == "yes",
        )?;
    }
    let mut rust_started = false;
    let mut prelude_c_entries = [0u64; 4];
    let mut activation_step = 0;
    let mut life = Lifecycle::default();
    let mut scroll = scrolling::Material::default();
    let mut doors = ceres_door::Material::default();
    let mut transitions = room_transition::Material::default();
    let mut directional = door_scrolling::Material::default();
    let mut camera = camera::Material::default();
    let mut escape = escape::Material::default();
    let mut camera_observations = BufWriter::new(File::create(out.join("camera.csv")).map_err(|e| e.to_string())?);
    writeln!(camera_observations, "step,state,room,before_x,before_y,after_x,after_y,previous_published")
        .map_err(|e| e.to_string())?;
    let mut directional_observations =
        BufWriter::new(File::create(out.join("directional.csv")).map_err(|e| e.to_string())?);
    writeln!(directional_observations, "step,entry,direction,counter_before,counter_after,flag,scroll_children")
        .map_err(|e| e.to_string())?;
    let mut transition_observations =
        BufWriter::new(File::create(out.join("transitions.csv")).map_err(|e| e.to_string())?);
    writeln!(transition_observations, "step,state,room,phase,continuation,door_flag,pending,resumed")
        .map_err(|e| e.to_string())?;
    let mut door_observations = BufWriter::new(File::create(out.join("doors.csv")).map_err(|e| e.to_string())?);
    writeln!(door_observations, "step,room,index,parameter,x,y,instruction,properties,spritemap,proximity_success,closed_step,animation_step,open_step").map_err(|e| e.to_string())?;
    let mut observations = BufWriter::new(File::create(out.join("lifecycle.csv")).map_err(|e| e.to_string())?);
    writeln!(observations, "step,state,room,x,y,spawned,moved,impacted,removed,camera_x,camera_y")
        .map_err(|e| e.to_string())?;
    let mut compare_seconds = 0.0;
    let mut game_seconds = 0.0;
    for (i, pad) in inputs.iter().enumerate() {
        if options.scrolling() && !options.escape() && i == 8379 {
            let state = game.state();
            let ram = game.wram();
            let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
            if (state.game_state, state.area, state.room, state.x, state.y) != (8, 6, 0xdf45, 140, 75)
                || *pad != 0x281
                || ram[0x0a1f] != 1
                || word(0x0b2e) != 0
                || word(0x0b2c) != 0
                || word(0x0b36) != 0
                || word(0x0b00) != 21
            {
                return Err("scrolling scenario did not reach the authored grounded shot context".into());
            }
        }
        if rust
            && !rust_started
            && game.state().game_state == 8
            && game.state().area == 6
            && game.state().room == 0xdf45
        {
            prelude_c_entries = game.projectile_counters().c_entries;
            activation_step = i + 1;
            if options.escape() {
                sm_restart::gameplay::install_escape_projectiles(&mut game, options.value("--fault", "no") == "yes")?;
            } else {
                sm_restart::gameplay::install_projectiles(
                    &mut game,
                    !options.scrolling() && options.value("--fault", "no") == "yes",
                )?;
            }
            rust_started = true;
        }
        let t = Instant::now();
        let state = game.host_step(*pad)?;
        game_seconds += t.elapsed().as_secs_f64();
        if rust
            && rust_started
            && if options.escape() {
                !escape::ownership_valid(game.projectile_counters(), prelude_c_entries)
            } else {
                game.projectile_counters().rust_refused.iter().any(|n| *n != 0)
            }
        {
            fs::write(out.join("refused.wram"), game.wram()).map_err(|e| e.to_string())?;
            return Err(format!("required Rust projectile entry refused at step {}", i + 1));
        }
        if rust && game.scroll_counters().rust_refused.iter().any(|n| *n != 0) {
            fs::write(out.join("refused.wram"), game.wram()).map_err(|e| e.to_string())?;
            return Err(format!("required Rust scrolling entry refused at step {}", i + 1));
        }
        if rust && game.ceres_door_counters().rust_refused.iter().any(|n| *n != 0) {
            fs::write(out.join("refused.wram"), game.wram()).map_err(|e| e.to_string())?;
            return Err(format!("required Rust Ceres door entry refused at step {}", i + 1));
        }
        if rust && game.room_transition_counters().rust_refused.iter().any(|n| *n != 0) {
            fs::write(out.join("refused.wram"), game.wram()).map_err(|e| e.to_string())?;
            return Err(format!("required Rust room-transition entry refused at step {}", i + 1));
        }
        if rust && game.door_scrolling_counters().rust_refused.iter().any(|n| *n != 0) {
            fs::write(out.join("refused.wram"), game.wram()).map_err(|e| e.to_string())?;
            return Err(format!("required Rust directional scrolling entry refused at step {}", i + 1));
        }
        if rust && game.camera_counters().rust_refused != 0 {
            fs::write(out.join("refused.wram"), game.wram()).map_err(|e| e.to_string())?;
            return Err(format!("required Rust camera entry refused at step {}", i + 1));
        }
        let before = (life.spawned, life.impacted, life.removed);
        if options.escape() {
            escape.observe((i + 1) as u64, state, game.wram(), game.projectile_counters())?;
        }
        if state.game_state == 8 && state.area == 6 && state.room == 0xdf45 {
            life.observe(game.wram(), game.projectile_counters().drawn_oam_entries);
        }
        scroll.observe(&state, game.wram(), game.scroll_counters());
        match camera.observe((i + 1) as u64, game.effects()) {
            Ok(rows) => camera_observations.write_all(rows.as_bytes()).map_err(|e| e.to_string())?,
            Err(error) => {
                fs::write(out.join("camera-error.effects"), game.effects()).map_err(|e| e.to_string())?;
                fs::write(out.join("camera-error.wram"), game.wram()).map_err(|e| e.to_string())?;
                fs::write(out.join("camera-error.txt"), format!("step {}: {error}", i + 1))
                    .map_err(|e| e.to_string())?;
                return Err(format!("camera observation at step {}: {error}", i + 1));
            }
        }
        match directional.observe((i + 1) as u64, state, game.wram(), game.effects()) {
            Ok(rows) => directional_observations.write_all(rows.as_bytes()).map_err(|e| e.to_string())?,
            Err(error) => {
                fs::write(out.join("directional-error.effects"), game.effects()).map_err(|e| e.to_string())?;
                fs::write(out.join("directional-error.wram"), game.wram()).map_err(|e| e.to_string())?;
                fs::write(out.join("directional-error.txt"), format!("step {}: {error}", i + 1))
                    .map_err(|e| e.to_string())?;
                return Err(format!("directional observation at step {}: {error}", i + 1));
            }
        }
        for row in doors.observe((i + 1) as u64, state, game.wram(), game.ceres_door_counters()) {
            door_observations.write_all(row.as_bytes()).map_err(|e| e.to_string())?;
        }
        match transitions.observe((i + 1) as u64, state, game.wram(), game.room_transition_counters(), game.effects()) {
            Ok(Some(row)) => transition_observations.write_all(row.as_bytes()).map_err(|e| e.to_string())?,
            Ok(None) => {}
            Err(error) => {
                fs::write(out.join("observer-error.effects"), game.effects()).map_err(|e| e.to_string())?;
                fs::write(out.join("observer-error.wram"), game.wram()).map_err(|e| e.to_string())?;
                fs::write(out.join("observer-error.txt"), format!("step {}: {error}", i + 1))
                    .map_err(|e| e.to_string())?;
                return Err(format!("transition observation at step {}: {error}", i + 1));
            }
        }
        writeln!(
            observations,
            "{},{},{:04x},{},{},{},{},{},{},{},{}",
            i + 1,
            state.game_state,
            state.room,
            state.x,
            state.y,
            life.spawned,
            life.moved,
            life.impacted,
            life.removed,
            scroll.camera_x,
            scroll.camera_y
        )
        .map_err(|e| e.to_string())?;
        observations.flush().map_err(|e| e.to_string())?;
        if before != (life.spawned, life.impacted, life.removed) {
            screenshot(&out.join(format!("step-{:06}.png", i + 1)), game.pixels())?;
        }
        let t = Instant::now();
        let external = game.observation_bytes();
        if options.value("--capture-step", "0").parse::<usize>().unwrap() == i + 1 {
            fs::write(out.join("capture.wram"), game.wram()).map_err(|e| e.to_string())?;
            fs::write(out.join("capture.external"), external).map_err(|e| e.to_string())?;
            fs::write(out.join("capture.effects"), game.effects()).map_err(|e| e.to_string())?;
            screenshot(&out.join("capture.png"), game.pixels())?;
        }
        let checked = if let Some(writer) = writer.as_mut() {
            writer.record((i + 1) as u64, game.wram(), external, game.effects())
        } else {
            compare.as_mut().unwrap().check((i + 1) as u64, game.wram(), external, game.effects())
        };
        compare_seconds += t.elapsed().as_secs_f64();
        if let Err(error) = checked {
            fs::write(out.join("mismatch.wram"), game.wram()).map_err(|e| e.to_string())?;
            fs::write(out.join("mismatch.external"), external).map_err(|e| e.to_string())?;
            fs::write(out.join("mismatch.effects"), game.effects()).map_err(|e| e.to_string())?;
            screenshot(&out.join("mismatch.png"), game.pixels())?;
            fs::write(out.join("mismatch.txt"), error.to_string()).map_err(|e| e.to_string())?;
            return Err(error.to_string());
        }
    }
    if let Some(writer) = writer {
        writer.finish().map_err(|e| e.to_string())?;
    }
    if let Some(compare) = compare {
        compare.finish().map_err(|e| e.to_string())?;
    }
    let counters = game.projectile_counters();
    let ownership_ok = !rust
        || (rust_started
            && if options.escape() {
                escape::ownership_complete(counters, prelude_c_entries)
            } else {
                counters.rust_completed.iter().all(|n| *n > 0)
                    && counters.rust_refused.iter().all(|n| *n == 0)
                    && rust_started
                    && counters.c_entries == prelude_c_entries
                    && counters.rust_entries == counters.rust_completed
            });
    let scrolling = game.scroll_counters();
    let scroll_ownership_ok = !rust
        || (scrolling.c_entries == [0; 3]
            && scrolling.rust_entries == scrolling.rust_completed
            && scrolling.rust_refused == [0; 3]
            && (!options.scrolling() || scrolling.rust_completed.iter().all(|n| *n > 0)));
    let material_ok = if options.escape() {
        escape.complete(&game)
            && escape::consumers_complete(
                &scroll.report(&game),
                &doors.report(&game),
                &transitions.report(game.room_transition_counters()),
                &directional.report(game.door_scrolling_counters()),
                &camera.report(game.camera_counters()),
            )
    } else if options.scrolling() {
        scroll.complete(&game)
            && !life.overlapping_shots
            && life.spawned >= 1
            && life.complete_shots >= 1
            && (0..5).all(|i| game.wram()[0xc40 + i * 2..0xc42 + i * 2] == [0, 0])
    } else {
        life.complete()
    };
    let door_counts = game.ceres_door_counters();
    let door_ownership_ok = !rust
        || (door_counts.c_entries == [0; 13]
            && door_counts.rust_entries == door_counts.rust_completed
            && door_counts.rust_refused == [0; 13]
            && (!options.doors() || (door_counts.rust_completed[0] > 0 && door_counts.rust_completed[1] > 0)));
    let success = material_ok
        && ownership_ok
        && scroll_ownership_ok
        && door_ownership_ok
        && (!options.doors() || options.escape() || doors.complete(&game))
        && (!rust || room_transition::ownership_complete(game.room_transition_counters()))
        && (!options.transitions() || options.escape() || transitions.complete())
        && (!rust || door_scrolling::ownership_complete(game.door_scrolling_counters()))
        && (!options.directional() || options.escape() || directional.complete(game.door_scrolling_counters()))
        && (!rust || camera::ownership_complete(game.camera_counters()))
        && (!options.camera() || camera.complete(game.camera_counters()))
        && game.state().cpu_fallback_calls == 0;
    door_observations.flush().map_err(|e| e.to_string())?;
    transition_observations.flush().map_err(|e| e.to_string())?;
    directional_observations.flush().map_err(|e| e.to_string())?;
    camera_observations.flush().map_err(|e| e.to_string())?;
    fs::write(out.join("final.wram"), game.wram()).map_err(|e| e.to_string())?;
    screenshot(&out.join("final.png"), game.pixels())?;
    let mut result = sm_binary_evidence::Map::new();
    result.insert("schema".into(), "sm-gameplay-pilot-v2".into());
    result.insert("status".into(), if success { "passed" } else { "milestone-not-reached" }.into());
    result.insert("scenario".into(), options.scenario().into());
    result.insert("overlapping_shots".into(), life.overlapping_shots.into());
    result.insert("rust_activation_step".into(), (activation_step as i64).into());
    result.insert(
        "prelude_c_entries".into(),
        sm_binary_evidence::Value::Array(
            prelude_c_entries.iter().map(|n| sm_binary_evidence::Value::Integer(*n as i64)).collect(),
        ),
    );
    result.insert("original_rom_comparison".into(), "not-tested".into());
    result.insert("scrolling".into(), scroll.report(&game));
    result.insert("ceres_doors".into(), doors.report(&game));
    result.insert("room_transitions".into(), transitions.report(game.room_transition_counters()));
    result.insert("door_scrolling".into(), directional.report(game.door_scrolling_counters()));
    result.insert("camera".into(), camera.report(game.camera_counters()));
    if options.escape() {
        result.insert("projectile_escape".into(), escape.report(&game));
    }
    result.insert("game_seconds".into(), game_seconds.into());
    result.insert("comparison_seconds".into(), compare_seconds.into());
    result.insert("patched_rom_sha256".into(), game.patched_rom_sha256().into());
    for (key, n) in [
        ("steps", inputs.len() as u64),
        ("spawned", life.spawned),
        ("moved", life.moved),
        ("impacted", life.impacted),
        ("removed", life.removed),
        ("complete_shots", life.complete_shots),
        ("foreign_calls", counters.foreign_calls),
    ] {
        result.insert(key.into(), (n as i64).into());
    }
    for (key, counts) in [
        ("c_entries", counters.c_entries),
        ("rust_entries", counters.rust_entries),
        ("rust_completed", counters.rust_completed),
        ("rust_refused", counters.rust_refused),
    ] {
        result.insert(
            key.into(),
            sm_binary_evidence::Value::Array(
                counts.iter().map(|n| sm_binary_evidence::Value::Integer(*n as i64)).collect(),
            ),
        );
    }
    result.insert(
        "drawn_oam_entries".into(),
        sm_binary_evidence::Value::Array(
            counters.drawn_oam_entries.iter().map(|n| sm_binary_evidence::Value::Integer(*n as i64)).collect(),
        ),
    );
    write_report(&out.join("result.sme"), &result)?;
    if success { Ok(()) } else { Err("required lifecycle or Rust ownership was not observed".into()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word(ram: &mut [u8], at: usize, n: u16) {
        ram[at..at + 2].copy_from_slice(&n.to_le_bytes());
    }

    #[test]
    fn offscreen_deletion_cannot_stand_in_for_impact_and_explosion() {
        let mut life = Lifecycle::default();
        let mut ram = vec![0; 0x20000];
        word(&mut ram, 0xc40, 0x8000);
        word(&mut ram, 0xc18, 0x8000);
        life.observe(&ram, [0, 0, 1, 0]);
        word(&mut ram, 0xb64, 12);
        life.observe(&ram, [0, 0, 2, 0]);
        word(&mut ram, 0xc40, 0);
        life.observe(&ram, [0, 0, 2, 0]);
        assert_eq!(life.removed, 1);
        assert_eq!(life.complete_shots, 0);
    }

    #[test]
    fn completed_hit_requires_movement_and_both_draws_for_same_slot() {
        let mut life = Lifecycle::default();
        let mut ram = vec![0; 0x20000];
        word(&mut ram, 0xc40, 0x8000);
        word(&mut ram, 0xc18, 0x8000);
        life.observe(&ram, [0, 0, 1, 0]);
        word(&mut ram, 0xb64, 12);
        life.observe(&ram, [0, 0, 2, 0]);
        word(&mut ram, 0xc18, 0x8700);
        life.observe(&ram, [0, 0, 2, 1]);
        word(&mut ram, 0xc40, 0);
        life.observe(&ram, [0, 0, 2, 1]);
        assert_eq!(life.complete_shots, 1);
        word(&mut ram, 0xc40, 0x8000);
        word(&mut ram, 0xc42, 0x8000);
        life.observe(&ram, [0, 0, 3, 1]);
        assert!(life.overlapping_shots);
    }
}
