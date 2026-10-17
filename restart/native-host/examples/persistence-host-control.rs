//! Full-state actual-C SRAM codec pairs and protected, owned-cwd disk controls.
//! This executable is a verification producer; only the integration owner runs it.
use libloading::Library;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sm_native_host::{
    Game, PersistenceCounters, PersistenceObservation, PersistenceOutcome, PersistenceOutput, PersistencePhase,
    PersistenceServices, prepare,
};
use sm_persistence::{
    Entry, Output, Services,
    fixtures::{BodyFixture, body_fixtures},
};
use std::{
    cell::RefCell,
    ffi::c_void,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const WRAM: usize = 0x20000;
const SRAM: usize = 0x2000;
type Result<T> = std::result::Result<T, String>;
type RawHook = unsafe extern "C" fn(*mut c_void, u32, u16, *mut u8, *mut u8, u64, *mut u32) -> i32;
type RawObserver = unsafe extern "C" fn(*mut c_void, u32, u32, u16, u32, *const u8, *const u8) -> i32;
#[derive(Clone, Copy)]
struct Native {
    commit: unsafe extern "C" fn(*const u8, *const u8),
    invoke: unsafe extern "C" fn(u32, u16) -> u32,
    begin: unsafe extern "C" fn(),
    generation: unsafe extern "C" fn() -> u64,
    service: unsafe extern "C" fn(*mut u8, *mut u8, u64) -> i32,
    set_hook: unsafe extern "C" fn(Option<RawHook>, *mut c_void),
    set_observer: unsafe extern "C" fn(Option<RawObserver>, *mut c_void),
    camera: unsafe extern "C" fn() -> i32,
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        // SAFETY: Game authenticated the retained library recipe before these
        // dedicated fixture symbols are loaded with their frozen signatures.
        unsafe {
            Ok(Self {
                commit: *library.get(b"sm_native_commit_persistence\0").map_err(|e| e.to_string())?,
                invoke: *library.get(b"sm_native_invoke_persistence\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_persistence_generation\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_persistence_service\0").map_err(|e| e.to_string())?,
                set_hook: *library.get(b"sm_native_set_persistence_hook\0").map_err(|e| e.to_string())?,
                set_observer: *library.get(b"sm_native_set_persistence_observer\0").map_err(|e| e.to_string())?,
                camera: *library.get(b"sm_native_camera_dispatch\0").map_err(|e| e.to_string())?,
            })
        }
    }
    fn seed(self, ram: &[u8], sram: &[u8]) {
        assert_eq!(ram.len(), WRAM);
        assert_eq!(sram.len(), SRAM);
        unsafe {
            (self.commit)(ram.as_ptr(), sram.as_ptr());
        }
    }
}
struct Disk<'a>(&'a mut PersistenceServices);
impl Services for Disk<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn write_sram(&mut self, ram: &mut [u8], sram: &mut [u8]) -> Result<()> {
        self.0.write_sram(ram, sram)
    }
}
fn install(game: &mut Game) -> Result<()> {
    game.set_persistence_hook(|entry, argument, ram, sram, rom, services| {
        let entry = Entry::ALL[entry as usize];
        Ok(match sm_persistence::execute(entry, argument, ram, sram, rom, &mut Disk(services))? {
            Some(Output::Done) => PersistenceOutcome::Handled(PersistenceOutput::Done),
            Some(Output::Loaded(value)) => PersistenceOutcome::Handled(PersistenceOutput::Loaded(value)),
            None => PersistenceOutcome::Refused,
        })
    })
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    event: PersistenceObservation,
    ram: Vec<u8>,
    sram: Vec<u8>,
}
type Records = Rc<RefCell<Vec<Record>>>;
fn observe(game: &mut Game) -> Result<Records> {
    let records = Records::default();
    let output = records.clone();
    game.set_persistence_observer(move |event, ram, sram| {
        output.borrow_mut().push(Record { event, ram: ram.to_vec(), sram: sram.to_vec() });
        Ok(())
    })?;
    Ok(records)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}
fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))
}
fn json(path: &Path, value: &impl Serialize) -> Result<()> {
    write(path, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
}
fn compare(label: &str, left: &[u8], right: &[u8]) -> Result<()> {
    if left == right {
        return Ok(());
    }
    let mismatch = left.iter().zip(right).position(|(a, b)| a != b).unwrap_or(left.len().min(right.len()));
    Err(format!("{label} differs at {mismatch:#x}: {} versus {}", hash(left), hash(right)))
}
fn observations(records: &[Record]) -> Vec<u8> {
    let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
    for record in records {
        let output = match record.event.output {
            None => u32::MAX,
            Some(PersistenceOutput::Done) => 0,
            Some(PersistenceOutput::Loaded(value)) => 0x10000 | u32::from(value),
        };
        for value in [record.event.phase as u32, record.event.entry as u32, u32::from(record.event.argument), output] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&record.ram);
        bytes.extend_from_slice(&record.sram);
    }
    bytes
}
struct Cwd(PathBuf);
impl Cwd {
    fn enter(path: &Path) -> Result<Self> {
        let old = std::env::current_dir().map_err(|e| e.to_string())?;
        std::env::set_current_dir(path).map_err(|e| e.to_string())?;
        Ok(Self(old))
    }
}
impl Drop for Cwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Outcome {
    output: u32,
    wram_sha256: String,
    sram_sha256: String,
    effects_sha256: String,
    observations_sha256: String,
    disk_written: bool,
}
fn execute_case(
    game: &mut Game,
    native: Native,
    records: &Records,
    case: &BodyFixture,
    directory: &Path,
    rust: bool,
    disk_directory: bool,
) -> Result<Outcome> {
    fs::create_dir(directory).map_err(|e| format!("fresh fixture {}: {e}", directory.display()))?;
    let _cwd = Cwd::enter(directory)?;
    let old_save = vec![0xa5; SRAM];
    if disk_directory {
        fs::create_dir("saves").map_err(|e| e.to_string())?;
        write(Path::new("saves/sm.srm"), &old_save)?;
        write(Path::new("saves/sm.srm.bak"), vec![0x5a; SRAM])?;
    }
    write(Path::new("input.wram"), &case.wram)?;
    write(Path::new("input.sram"), &case.sram)?;
    if rust {
        install(game)?;
    } else {
        game.clear_persistence_hook();
    }
    native.seed(&case.wram, &case.sram);
    records.borrow_mut().clear();
    let before = game.persistence_counters();
    let generation = unsafe { (native.generation)() };
    unsafe {
        (native.begin)();
    }
    let output = unsafe { (native.invoke)(case.entry as u32, case.argument) };
    let after = game.persistence_counters();
    assert_eq!(unsafe { (native.generation)() }, generation + 2, "completed native invocation generation");
    let records = records.borrow();
    let bytes = observations(&records);
    write(Path::new("final.wram"), game.wram())?;
    write(Path::new("final.sram"), game.sram())?;
    write(Path::new("effects.bin"), game.effects())?;
    write(Path::new("observations.bin"), &bytes)?;
    json(Path::new("counters-before.json"), &before)?;
    json(Path::new("counters-after.json"), &after)?;
    let is_save = case.entry == Entry::Save;
    let phases: Vec<_> = records.iter().map(|r| r.event.phase).collect();
    let expected = if is_save {
        vec![PersistencePhase::Begin, PersistencePhase::DiskBegin, PersistencePhase::DiskEnd, PersistencePhase::End]
    } else {
        vec![PersistencePhase::Begin, PersistencePhase::End]
    };
    assert_eq!(phases, expected, "{} service order", case.name);
    assert!(records.iter().all(|r| r.event.entry as u32 == case.entry as u32 && r.event.argument == case.argument));
    assert_eq!(records[0].ram, case.wram);
    assert_eq!(records[0].sram, case.sram);
    assert_eq!(records.last().unwrap().ram, game.wram());
    assert_eq!(records.last().unwrap().sram, game.sram());
    if is_save {
        assert_eq!(records[1].ram, game.wram());
        assert_eq!(records[1].sram, game.sram());
        assert_eq!(records[1].ram, records[2].ram);
        assert_eq!(records[1].sram, records[2].sram);
    }
    check_counts(before, after, case.entry as usize, rust, is_save);
    if case.entry == Entry::Load {
        assert!(matches!(output, 0x10000 | 0x10001));
        let invalid = case.name.contains("neither") || case.name.contains("split");
        assert_eq!(output, if invalid { 0x10001 } else { 0x10000 });
    } else {
        assert_eq!(output, 0);
    }
    if is_save && disk_directory {
        compare("actual saved8192 bytes", game.sram(), &read(Path::new("saves/sm.srm"))?)?;
        compare("rename preserves previous save", &old_save, &read(Path::new("saves/sm.srm.bak"))?)?;
    } else if !disk_directory {
        assert!(!Path::new("saves").exists(), "failed open must not synthesize a directory");
    } else {
        compare("non-Save leaves existing save", &old_save, &read(Path::new("saves/sm.srm"))?)?;
        compare("non-Save leaves backup", &vec![0x5a; SRAM], &read(Path::new("saves/sm.srm.bak"))?)?;
    }
    let outcome = Outcome {
        output,
        wram_sha256: hash(game.wram()),
        sram_sha256: hash(game.sram()),
        effects_sha256: hash(game.effects()),
        observations_sha256: hash(&bytes),
        disk_written: is_save && disk_directory,
    };
    json(Path::new("result.json"), &outcome)?;
    Ok(outcome)
}
fn check_counts(before: PersistenceCounters, after: PersistenceCounters, entry: usize, rust: bool, save: bool) {
    for i in 0..4 {
        let called = u64::from(i == entry);
        assert_eq!(after.entries[i] - before.entries[i], called);
        assert_eq!(after.c_entries[i] - before.c_entries[i], if rust { 0 } else { called });
        assert_eq!(after.rust_entries[i] - before.rust_entries[i], if rust { called } else { 0 });
        assert_eq!(after.rust_completed[i] - before.rust_completed[i], if rust { called } else { 0 });
        assert_eq!(after.rust_refused[i], before.rust_refused[i]);
    }
    assert_eq!(after.write_requests - before.write_requests, u64::from(save));
    assert_eq!(after.write_completions - before.write_completions, u64::from(save));
}
fn pair(
    game: &mut Game,
    native: Native,
    records: &Records,
    case: &BodyFixture,
    root: &Path,
    disks: bool,
) -> Result<()> {
    let directory = root.join(&case.name);
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    let c = directory.join("c");
    let r = directory.join("rust");
    let left = execute_case(game, native, records, case, &c, false, disks)?;
    let right = execute_case(game, native, records, case, &r, true, disks)?;
    assert_eq!(left, right, "{} actual-C result", case.name);
    for file in ["input.wram", "input.sram", "final.wram", "final.sram", "effects.bin", "observations.bin"] {
        compare(&format!("{} {file}", case.name), &read(&c.join(file))?, &read(&r.join(file))?)?;
    }
    json(
        &directory.join("pair.json"),
        &serde_json::json!({"name": case.name, "actual_c": true,
        "full_wram": true, "full_sram": true, "ordered_effects": true, "service_images": true,
        "rust_completed": 1, "result": right}),
    )?;
    println!("pair={} entry={:?} full_wram=true full_sram=true actual_c=true rust_completed=1", case.name, case.entry);
    Ok(())
}

fn chains(game: &mut Game, native: Native, records: &Records, root: &Path) -> Result<usize> {
    let cases = body_fixtures();
    for slot in 0..3 {
        let mut case = cases.iter().find(|case| case.name == format!("save-slot-{slot}-area-{slot}")).unwrap().clone();
        case.name = format!("chain-slot-{slot}");
        let directory = root.join(&case.name);
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        for rust in [false, true] {
            let owner = directory.join(if rust { "rust" } else { "c" });
            fs::create_dir(&owner).map_err(|e| e.to_string())?;
            let saved = execute_case(game, native, records, &case, &owner.join("save"), rust, true)?;
            let mut load = BodyFixture {
                name: format!("chain-load-{slot}"),
                entry: Entry::Load,
                argument: case.argument,
                wram: game.wram().to_vec(),
                sram: game.sram().to_vec(),
            };
            // Declared fixture perturbation between codec invocations; SRAM is
            // the canonical output of Save, without disk reopen or gameplay.
            load.wram[0x9a2..0xa02].fill(0x3c);
            load.wram[0xcd52..0xd454].fill(0x7e);
            load.wram[0x78b..0x78d].fill(0);
            load.wram[0x79f..0x7a1].fill(0);
            let loaded = execute_case(game, native, records, &load, &owner.join("load"), rust, true)?;
            assert_eq!(loaded.output, 0x10000);
            compare("chain restores player96", &case.wram[0x9a2..0xa02], &game.wram()[0x9a2..0xa02])?;
            compare("chain restores station", &case.wram[0x78b..0x78d], &game.wram()[0x78b..0x78d])?;
            compare("chain restores area", &case.wram[0x79f..0x7a1], &game.wram()[0x79f..0x7a1])?;
            compare("Load preserves valid canonical SRAM", &load.sram, game.sram())?;
            results.push((saved, loaded));
        }
        assert_eq!(results[0], results[1], "save/load chain {slot}");
        for phase in ["save", "load"] {
            for file in ["input.wram", "input.sram", "final.wram", "final.sram", "effects.bin", "observations.bin"] {
                compare(
                    &format!("chain{slot}/{phase}/{file}"),
                    &read(&directory.join("c").join(phase).join(file))?,
                    &read(&directory.join("rust").join(phase).join(file))?,
                )?;
            }
        }
        json(
            &directory.join("pair.json"),
            &serde_json::json!({"slot": slot, "actual_c": true,
            "canonical_save_load_chain": true, "process_reopen": false, "live_continue": false}),
        )?;
    }
    Ok(3)
}

struct Admission {
    available: bool,
    calls: usize,
}
impl Services for Admission {
    fn available(&self) -> bool {
        self.available
    }
    fn write_sram(&mut self, _: &mut [u8], _: &mut [u8]) -> Result<()> {
        self.calls += 1;
        Err("admission must not issue disk service".into())
    }
}
fn refusals(game: &Game, root: &Path) -> Result<usize> {
    let base = body_fixtures().into_iter().find(|case| case.entry == Entry::Save).unwrap();
    let mut controls = 0;
    for (label, entry, argument, area, available, short_ram, short_sram, changed_rom, expect_error) in [
        ("slot3-save", Entry::Save, 3, 0, true, false, false, false, false),
        ("slot3-load", Entry::Load, 0xffff, 0, true, false, false, false, false),
        ("area6", Entry::Save, 0, 6, true, false, false, false, false),
        ("area-ffff", Entry::Save, 0, 0xffff, true, false, false, false, false),
        ("missing-service", Entry::Save, 0, 0, false, false, false, false, true),
        ("short-wram", Entry::Save, 0, 0, true, true, false, false, true),
        ("short-sram", Entry::Load, 0, 0, true, false, true, false, true),
        ("changed-table", Entry::Pack, 0, 0, true, false, false, true, true),
        ("helper-argument", Entry::Unpack, 1, 0, true, false, false, false, true),
    ] {
        let mut ram = base.wram.clone();
        let mut sram = base.sram.clone();
        let mut rom = game.rom().to_vec();
        ram[0x79f..0x7a1].copy_from_slice(&(area as u16).to_le_bytes());
        if short_ram {
            ram.pop();
        }
        if short_sram {
            sram.pop();
        }
        if changed_rom {
            rom[0x8131] ^= 1;
        }
        let before = (ram.clone(), sram.clone());
        let mut service = Admission { available, calls: 0 };
        let result = sm_persistence::execute(entry, argument, &mut ram, &mut sram, &rom, &mut service);
        if expect_error {
            assert!(result.is_err(), "{label}");
        } else {
            assert_eq!(result?, None, "{label}");
        }
        assert_eq!((ram, sram), before, "{label} atomic refusal");
        assert_eq!(service.calls, 0, "{label}");
        controls += 1;
    }
    json(
        &root.join("admission.json"),
        &serde_json::json!({"controls": controls, "unchanged": true,
        "native_unsafe_inputs_executed": false, "disk_requests": 0}),
    )?;
    Ok(controls)
}

const FATALS: &[(&str, &str)] = &[
    ("dirty-wram", "persistence dirty refusal"),
    ("dirty-sram", "persistence dirty refusal"),
    ("wrong-wram", "persistence invocation poisoned by service failure or changed generation"),
    ("wrong-sram", "persistence invocation poisoned by service failure or changed generation"),
    ("wrong-generation", "persistence invocation poisoned"),
    ("native-wrong-wram", "persistence invocation poisoned"),
    ("native-wrong-sram", "persistence invocation poisoned"),
    ("service-refuse", "persistence dirty refusal or refusal after service"),
    ("post-service-error", "post-service failure control"),
    ("recursive-callback", "invalid or nested persistence entry"),
    ("recursive-observer", "invalid or nested persistence entry"),
    ("sibling-entry", "recursive or cross-consumer camera root"),
    ("slot3-save", "unsafe persistence argument"),
    ("slot3-load", "unsafe persistence argument"),
    ("malformed-output", "invalid persistence Load byte"),
];

struct RawFault {
    native: Native,
    mode: String,
}
unsafe extern "C" fn raw_service_fault(
    context: *mut c_void,
    _: u32,
    _: u16,
    ram: *mut u8,
    sram: *mut u8,
    generation: u64,
    _: *mut u32,
) -> i32 {
    let fault = unsafe { &*context.cast::<RawFault>() };
    // Deliberately swallow the raw error: C must retain poison and forbid retry.
    let mut wrong_ram = vec![0; WRAM];
    let mut wrong_sram = vec![0; SRAM];
    let (ram, sram, generation) = match fault.mode.as_str() {
        "native-wrong-wram" => (wrong_ram.as_mut_ptr(), sram, generation),
        "native-wrong-sram" => (ram, wrong_sram.as_mut_ptr(), generation),
        _ => (ram, sram, generation.wrapping_add(1)),
    };
    let _ = unsafe { (fault.native.service)(ram, sram, generation) };
    0
}
unsafe extern "C" fn recursive_observer(
    context: *mut c_void,
    _: u32,
    _: u32,
    _: u16,
    _: u32,
    _: *const u8,
    _: *const u8,
) -> i32 {
    let native = unsafe { &*context.cast::<Native>() };
    unsafe {
        (native.invoke)(1, 0);
    }
    0
}
fn fatal(game: &mut Game, native: Native, root: &Path, mode: &str) -> Result<()> {
    let _cwd = Cwd::enter(root)?;
    fs::create_dir("saves").map_err(|e| e.to_string())?;
    let case = body_fixtures().into_iter().find(|case| case.entry == Entry::Save).unwrap();
    write(Path::new("input.wram"), &case.wram)?;
    write(Path::new("input.sram"), &case.sram)?;
    json(
        Path::new("invocation.json"),
        &serde_json::json!({"mode": mode, "entry": "Save", "argument": 0,
        "slot3_override": mode.starts_with("slot3"), "unsafe_c_body_permitted": false}),
    )?;
    native.seed(&case.wram, &case.sram);
    unsafe {
        (native.begin)();
    }
    if mode == "slot3-save" || mode == "slot3-load" {
        unsafe {
            (native.invoke)(u32::from(mode == "slot3-load"), 0xffff);
        }
    } else if matches!(mode, "wrong-generation" | "native-wrong-wram" | "native-wrong-sram") {
        let mut context = RawFault { native, mode: mode.into() };
        unsafe {
            (native.set_hook)(Some(raw_service_fault), (&mut context as *mut RawFault).cast());
            (native.invoke)(0, 0);
        }
    } else if mode == "recursive-observer" {
        let mut context = native;
        unsafe {
            (native.set_observer)(Some(recursive_observer), (&mut context as *mut Native).cast());
            (native.invoke)(0, 0);
        }
    } else {
        let mode = mode.to_owned();
        game.set_persistence_hook(move |_, _, ram, sram, _, services| {
            match mode.as_str() {
                "dirty-wram" => ram[0] ^= 1,
                "dirty-sram" => sram[0] ^= 1,
                "wrong-wram" => {
                    let _ = services.write_sram(&mut ram.to_vec(), sram);
                }
                "wrong-sram" => {
                    let _ = services.write_sram(ram, &mut sram.to_vec());
                }
                "service-refuse" => services.write_sram(ram, sram)?,
                "post-service-error" => {
                    services.write_sram(ram, sram)?;
                    return Err("post-service failure control".into());
                }
                "recursive-callback" => unsafe {
                    (native.invoke)(1, 0);
                },
                "sibling-entry" => unsafe {
                    (native.camera)();
                },
                "malformed-output" => return Ok(PersistenceOutcome::Handled(PersistenceOutput::Loaded(2))),
                _ => return Err("unknown fatal persistence mode".into()),
            }
            Ok(PersistenceOutcome::Refused)
        })?;
        unsafe {
            (native.invoke)(0, 0);
        }
    }
    // Returning normally is an unsuccessful control, never a fatal proof.
    Err("fatal persistence control unexpectedly returned".into())
}

fn fatal_children(args: &[String], root: &Path, start: usize) -> Result<usize> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    for &(mode, expected) in &FATALS[start..] {
        let logs = root.join(format!("fatal-{mode}-process"));
        fs::create_dir(&logs).map_err(|e| e.to_string())?;
        let stdout = fs::File::create(logs.join("stdout.txt")).map_err(|e| e.to_string())?;
        let stderr = fs::File::create(logs.join("stderr.txt")).map_err(|e| e.to_string())?;
        let started_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
        let started = Instant::now();
        let mut child = Command::new(&exe)
            .args(&args[1..4])
            .arg(format!("fatal-{mode}"))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut timed_out = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() < Duration::from_secs(20) => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Ok(None) => {
                    timed_out = true;
                    let _ = child.kill();
                    break child.wait().map_err(|e| e.to_string())?;
                }
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(error.to_string());
                }
            }
        };
        let stderr = read(&logs.join("stderr.txt"))?;
        let verified = !timed_out && status.code() == Some(2) && String::from_utf8_lossy(&stderr).contains(expected);
        json(
            &logs.join("process.json"),
            &serde_json::json!({"mode": mode, "expected_error": expected,
            "argv": [exe.to_string_lossy().as_ref(), &args[1], &args[2], &args[3], &format!("fatal-{mode}")],
            "cwd": std::env::current_dir().map_err(|e| e.to_string())?,
            "started_unix_ms": started_ms, "elapsed_ms": started.elapsed().as_millis(),
            "exit_code": status.code(), "expected_exit_code": 2, "timed_out": timed_out, "verified": verified,
            "stderr_sha256": hash(&stderr), "stdout_sha256": hash(&read(&logs.join("stdout.txt"))?)}),
        )?;
        if !verified {
            return Err(format!("fatal control {mode} failed; see {}", logs.display()));
        }
    }
    Ok(FATALS.len() - start)
}

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: persistence-host-control ARCHIVE ROM OUTPUT MODE(prepare|all|bodies|fatals|fatals-START|fatal-NAME)".into());
    }
    for index in [1, 2] {
        args[index] = Path::new(&args[index]).canonicalize().map_err(|e| e.to_string())?.to_string_lossy().into_owned();
    }
    fs::create_dir_all(&args[3]).map_err(|e| e.to_string())?;
    args[3] = Path::new(&args[3]).canonicalize().map_err(|e| e.to_string())?.to_string_lossy().into_owned();
    let prepared = prepare(Path::new(&args[1]), Path::new(&args[3]))?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if args[4] == "prepare" {
        return Ok(());
    }
    let mode = &args[4];
    // START is a zero-based index in the frozen FATALS order. Skipped cases are
    // reported, never counted as newly verified by this invocation.
    let fatal_start = if mode == "fatals" {
        Some(0)
    } else if let Some(index) = mode.strip_prefix("fatals-") {
        Some(index.parse::<usize>().ok().filter(|&index| index < FATALS.len()).ok_or("invalid fatal start index")?)
    } else {
        None
    };
    if !matches!(mode.as_str(), "all" | "bodies")
        && fatal_start.is_none()
        && !FATALS.iter().any(|(name, _)| mode == &format!("fatal-{name}"))
    {
        return Err("unknown persistence control mode".into());
    }
    let root = Path::new(&args[3]).join(format!("persistence-control-{mode}"));
    fs::create_dir(&root).map_err(|e| format!("fresh control root {}: {e}", root.display()))?;
    let _cwd = Cwd::enter(&root)?;
    if let Some(start) = fatal_start {
        let completed = fatal_children(&args, &root, start)?;
        json(
            &root.join("result.json"),
            &serde_json::json!({"status": "passed", "fatal_controls": completed,
            "fatal_start_index": start, "skipped_fatal_controls": start, "body_pairs": 0,
            "library_sha256": hash(&read(&prepared.library)?),
            "executable_sha256": hash(&read(&std::env::current_exe().map_err(|e| e.to_string())?)?)}),
        )?;
        println!("persistence_fatal_controls={completed} start_index={start} body_pairs=0");
        return Ok(());
    }
    let mut game = Game::boot(&prepared.library, Path::new(&args[2]))?;
    let library = unsafe { Library::new(&prepared.library).map_err(|e| e.to_string())? };
    let native = unsafe { Native::load(&library)? };
    if let Some(mode) = mode.strip_prefix("fatal-") {
        return fatal(&mut game, native, &root, mode);
    }
    let records = observe(&mut game)?;
    let cases = body_fixtures();
    assert_eq!(cases.len(), 30, "frozen shared behavior matrix changed");
    for case in &cases {
        pair(&mut game, native, &records, case, &root, true)?;
    }
    let mut absent = cases.iter().find(|case| case.entry == Entry::Save).unwrap().clone();
    absent.name = "save-without-directory".into();
    pair(&mut game, native, &records, &absent, &root, false)?;
    let chains = chains(&mut game, native, &records, &root)?;
    let refusals = refusals(&game, &root)?;
    let fatals = if mode == "all" { fatal_children(&args, &root, 0)? } else { 0 };
    json(
        &root.join("result.json"),
        &serde_json::json!({"status": "passed", "body_pairs": cases.len(),
        "absent_directory_pairs": 1, "save_load_chains": chains, "atomic_admission_controls": refusals,
        "fatal_controls": fatals, "library_sha256": hash(&read(&prepared.library)?),
        "executable_sha256": hash(&read(&std::env::current_exe().map_err(|e| e.to_string())?)?),
        "rom_sha256": sm_native_host::ROM_SHA256, "process_reopen": false, "live_continue": false}),
    )?;
    println!(
        "persistence_body_pairs={} disk_failure_pairs=1 save_load_chains={chains} admission_controls={refusals} fatal_controls={fatals} live_continue=false",
        cases.len()
    );
    Ok(())
}
