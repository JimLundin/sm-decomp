//! Frozen Waver actual-C and transport controls; integration owner runs producers.
use libloading::Library;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sm_native_host::{
    Game, HookOutcome, WaverObservation, WaverOutput, WaverPhase, WaverService, WaverServices, prepare,
};
use sm_waver::{BodyFixture, Entry, Output, Services, body_fixtures};
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, String>;
const WRAM: usize = 0x20000;
#[derive(Clone, Copy)]
struct Native {
    seed: unsafe extern "C" fn(*const u8),
    begin: unsafe extern "C" fn(),
    invoke: unsafe extern "C" fn(u32, u16, u16) -> u16,
    admitted: unsafe extern "C" fn(u32, u16, u16) -> i32,
    prepared: unsafe extern "C" fn() -> i32,
    service: unsafe extern "C" fn(*mut u8, usize, u32, u16, i32, u16, *mut u16, u64) -> i32,
    generation: unsafe extern "C" fn() -> u64,
    ai: unsafe extern "C" fn(u32),
    instruction: unsafe extern "C" fn(),
    player_down: unsafe extern "C" fn(i32),
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        unsafe {
            Ok(Self {
                seed: *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                invoke: *library.get(b"sm_native_invoke_waver\0").map_err(|e| e.to_string())?,
                admitted: *library.get(b"sm_native_waver_fixture_admitted\0").map_err(|e| e.to_string())?,
                prepared: *library.get(b"sm_native_waver_prepared_tables\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_waver_service\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_waver_generation\0").map_err(|e| e.to_string())?,
                ai: *library.get(b"sm_native_waver_ai\0").map_err(|e| e.to_string())?,
                instruction: *library.get(b"ProcessEnemyInstructions\0").map_err(|e| e.to_string())?,
                player_down: *library.get(b"Samus_MoveDown\0").map_err(|e| e.to_string())?,
            })
        }
    }
    fn seed(self, ram: &[u8]) {
        assert_eq!(ram.len(), WRAM);
        unsafe {
            (self.seed)(ram.as_ptr());
            (self.begin)();
        }
    }
    fn call(self, case: &BodyFixture, interpreter: bool) -> u16 {
        assert_eq!(
            unsafe { (self.admitted)(case.entry as u32, case.k, case.jp) },
            1,
            "unsafe fixture never reaches C: {}",
            case.name
        );
        unsafe {
            if interpreter {
                (self.instruction)();
                0
            } else {
                (self.invoke)(case.entry as u32, case.k, case.jp)
            }
        }
    }
}
struct Bridge<'a>(&'a mut WaverServices);
impl Services for Bridge<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8> {
        self.0.move_horizontal(ram, k, amount)
    }
    fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16> {
        self.0.sine(ram, angle, amplitude)
    }
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8> {
        self.0.move_vertical(ram, k, amount)
    }
}
struct Unavailable;
impl Services for Unavailable {
    fn available(&self) -> bool {
        false
    }
    fn move_horizontal(&mut self, _: &mut [u8], _: u16, _: i32) -> Result<u8> {
        Err("unexpected service".into())
    }
    fn sine(&mut self, _: &mut [u8], _: u16, _: u16) -> Result<u16> {
        Err("unexpected service".into())
    }
    fn move_vertical(&mut self, _: &mut [u8], _: u16, _: i32) -> Result<u8> {
        Err("unexpected service".into())
    }
}
fn install(game: &mut Game) -> Result<()> {
    game.set_waver_hook(|entry, k, jp, ram, rom, services| {
        let entry = Entry::try_from(entry as u32)?;
        Ok(sm_waver::execute(entry, k, jp, ram, rom, &mut Bridge(services))?.map(|v| match v {
            Output::Done => WaverOutput::Done,
            Output::Continue(jp) => WaverOutput::Continue(jp),
        }))
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    event: WaverObservation,
    ram: Vec<u8>,
}
type Records = Rc<RefCell<Vec<Record>>>;
fn observe(game: &mut Game) -> Result<Records> {
    let records = Records::default();
    let output = records.clone();
    game.set_waver_observer(move |event, ram| {
        output.borrow_mut().push(Record { event, ram: ram.to_vec() });
        Ok(())
    })?;
    Ok(records)
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    fs::write(path, bytes).map_err(|e| e.to_string())
}
fn read(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|e| e.to_string())
}
fn json(path: &Path, value: &impl Serialize) -> Result<()> {
    write(path, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
}
fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn base(ram: &[u8]) -> usize {
    0xf78 + usize::from(word(ram, 0xe54))
}
fn amount(ram: &[u8]) -> i32 {
    let b = base(ram);
    ((u32::from(word(ram, b + 0x32)) << 16) | u32::from(word(ram, b + 0x30))) as i32
}
fn compare(name: &str, left: &[u8], right: &[u8]) -> Result<()> {
    if left == right {
        return Ok(());
    }
    let at = left.iter().zip(right).position(|(a, b)| a != b).unwrap_or(left.len().min(right.len()));
    Err(format!("{name} mismatch at {at:#x}: {} vs {}", hash(left), hash(right)))
}
fn record_bytes(records: &[Record]) -> Vec<u8> {
    let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
    for r in records {
        for v in [
            r.event.phase as u32,
            r.event.entry as u32,
            u32::from(r.event.k),
            u32::from(r.event.jp),
            r.event.service.map_or(3, |s| s as u32),
            r.event.argument as u32,
            u32::from(r.event.auxiliary),
            u32::from(r.event.result),
        ] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend(&r.ram);
    }
    bytes
}
fn pair(
    game: &mut Game,
    native: Native,
    records: &Records,
    case: &BodyFixture,
    root: &Path,
    interpreter: bool,
) -> Result<Vec<u8>> {
    let directory = root.join(&case.name);
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    write(&directory.join("input.wram"), &case.wram)?;
    let mut executions = Vec::new();
    for rust in [false, true] {
        if rust {
            install(game)?;
        } else {
            game.clear_waver_hook();
        }
        native.seed(&case.wram);
        records.borrow_mut().clear();
        let before = game.waver_counters();
        let output = native.call(case, interpreter);
        let after = game.waver_counters();
        let ram = game.wram().to_vec();
        let effects = game.effects().to_vec();
        let seen = records.borrow().clone();
        let label = if rust { "rust" } else { "c" };
        write(&directory.join(format!("{label}.wram")), &ram)?;
        write(&directory.join(format!("{label}.effects")), &effects)?;
        write(&directory.join(format!("{label}.observations")), record_bytes(&seen))?;
        json(
            &directory.join(format!("{label}.json")),
            &serde_json::json!({"output":output,"before":before,"after":after}),
        )?;
        let entry = case.entry as usize;
        assert_eq!(after.entries[entry] - before.entries[entry], 1);
        if rust {
            assert_eq!(after.rust_completed[entry] - before.rust_completed[entry], 1);
            assert_eq!(after.c_entries, before.c_entries);
        } else {
            assert_eq!(after.c_entries[entry] - before.c_entries[entry], 1);
            assert_eq!(after.rust_completed, before.rust_completed);
        }
        assert_eq!(after.rust_refused, before.rust_refused);
        assert_eq!(after.invalid_requests, before.invalid_requests);
        let services: Vec<_> =
            seen.iter().filter(|r| r.event.phase == WaverPhase::ServiceEnd).map(|r| r.event.service.unwrap()).collect();
        if case.entry == Entry::Main {
            assert!(
                services == [WaverService::MoveHorizontal]
                    || services == [WaverService::MoveHorizontal, WaverService::Sine, WaverService::MoveVertical]
            );
        } else {
            assert!(services.is_empty());
        }
        for (i, service) in WaverService::ALL.into_iter().enumerate() {
            let n = services.iter().filter(|&&s| s == service).count() as u64;
            assert_eq!(after.service_requests[i] - before.service_requests[i], n);
            assert_eq!(after.service_completions[i] - before.service_completions[i], n);
        }
        assert_eq!(seen.len(), 2 + services.len() * 2);
        assert_eq!(seen[0].event.phase, WaverPhase::RootBegin);
        assert_eq!(seen.last().unwrap().event.phase, WaverPhase::RootEnd);
        if case.entry == Entry::Instruction {
            assert_eq!(seen.last().unwrap().event.result, case.jp);
        }
        executions.push((output, ram, effects, seen));
    }
    let (c_output, c_ram, c_effects, c_seen) = &executions[0];
    let (output, ram, effects, seen) = &executions[1];
    assert_eq!(c_output, output);
    compare(&case.name, c_ram, ram)?;
    compare("effects", c_effects, effects)?;
    assert_eq!(c_seen, seen);
    if interpreter {
        assert_eq!(word(ram, base(ram) + 0x1a), case.jp);
        assert_eq!(word(ram, base(ram) + 0x38), 1);
    }
    if case.name.starts_with("main-spike-") {
        let events: Vec<_> = effects.as_chunks::<16>().0.iter().filter(|v| matches!(word(*v, 0), 3 | 4)).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(u32::from_le_bytes(events[0][4..8].try_into().unwrap()), 0xd094);
        assert_eq!(word(events[1], 0), 4);
        assert_eq!(u32::from_le_bytes(events[1][4..8].try_into().unwrap()), 0);
        assert_eq!(word(ram, 0x10002 + 104), if case.name == "main-spike-allocate" { 0 } else { 0xa000 });
        assert_eq!(word(ram, 0x1c37 + 78), 0xd094);
    }
    json(
        &directory.join("pair.json"),
        &serde_json::json!({"name":case.name,"actual_c":true,"full_wram":true,"ordered_services":true,"scheduler":case.entry!=Entry::SelectAnimation,"generic_interpreter":interpreter,"wram_sha256":hash(ram),"effects_sha256":hash(effects)}),
    )?;
    println!("pair={} entry={:?} actual_c=true full_wram=true", case.name, case.entry);
    Ok(ram.clone())
}
fn fixture(name: &str) -> BodyFixture {
    body_fixtures().into_iter().find(|f| f.name == name).unwrap()
}
fn admissions(game: &mut Game, native: Native, root: &Path) -> Result<usize> {
    let names = [
        "invalid-slot",
        "main-selector",
        "helper-selector",
        "instruction-k",
        "instruction-low-jp",
        "instruction-end-jp",
        "extension-cycle",
        "extension-out-of-range",
        "spike-index16",
        "zero-width",
        "large-amount",
        "geometry",
    ];
    for name in names {
        let mut f = fixture("main-zero-sine");
        let b = base(&f.wram);
        match name {
            "invalid-slot" => put(&mut f.wram, 0xe54, 1),
            "main-selector" => put(&mut f.wram, b + 0x3a, 4),
            "helper-selector" => {
                f.entry = Entry::SelectAnimation;
                put(&mut f.wram, b + 0x3a, 4);
            }
            "instruction-k" => {
                f.entry = Entry::Instruction;
                f.k = 1;
                f.jp = 0x86c5;
            }
            "instruction-low-jp" => {
                f.entry = Entry::Instruction;
                f.jp = 0x7fff;
            }
            "instruction-end-jp" => {
                f.entry = Entry::Instruction;
                f.jp = 0xffff;
            }
            "extension-cycle" => {
                put(&mut f.wram, 0x10002 + 102, 0x5000);
                f.wram[0x16402 + 51] = 1;
                put(&mut f.wram, 0x10002 + 104, 0x5000);
                f.wram[0x16402 + 52] = 255;
            }
            "extension-out-of-range" => {
                put(&mut f.wram, 0x10002 + 102, 0xd000);
                f.wram[0x16402 + 51] = 128;
            }
            "spike-index16" => {
                put(&mut f.wram, 0x10002 + 102, 0xa000);
                f.wram[0x16402 + 51] = 16;
            }
            "zero-width" => put(&mut f.wram, 0x7a5, 0),
            "large-amount" => put(&mut f.wram, b + 0x32, 17),
            "geometry" => put(&mut f.wram, b + 2, 1),
            _ => unreachable!(),
        }
        native.seed(&f.wram);
        let before = f.wram.clone();
        assert_eq!(unsafe { (native.admitted)(f.entry as u32, f.k, f.jp) }, 0, "{name}");
        assert!(
            matches!(
                sm_waver::execute(f.entry, f.k, f.jp, &mut f.wram, game.rom(), &mut Unavailable),
                Ok(None) | Err(_)
            ),
            "{name}"
        );
        compare("immutable Rust refusal", &before, &f.wram)?;
        compare("immutable native admission", &before, game.wram())?;
        let directory = root.join(format!("admission-{name}"));
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        write(&directory.join("input.wram"), &before)?;
        json(&directory.join("result.json"), &serde_json::json!({"refused_before_c":true,"unchanged":true}))?;
    }
    let mut f = fixture("helper-select-1");
    let before = f.wram.clone();
    assert!(sm_waver::execute(f.entry, 0, 0, &mut f.wram, &[], &mut Unavailable).is_err());
    assert_eq!(f.wram, before);
    json(&root.join("short-rom.json"), &serde_json::json!({"refused_before_c":true,"unchanged":true,"rom_bytes":0}))?;
    let f = fixture("main-zero-sine");
    game.clear_waver_hook();
    native.seed(&f.wram);
    native.call(&f, false);
    let expected = game.wram().to_vec();
    let effects = game.effects().to_vec();
    game.set_waver_hook(|_, _, _, _, _, _| Ok(None))?;
    native.seed(&f.wram);
    let before = game.waver_counters();
    native.call(&f, false);
    let after = game.waver_counters();
    compare("unchanged refusal C fallback", &expected, game.wram())?;
    compare("refusal effects", &effects, game.effects())?;
    assert_eq!(after.rust_refused[1] - before.rust_refused[1], 1);
    assert_eq!(after.c_entries[1] - before.c_entries[1], 1);
    assert_eq!(after.rust_completed, before.rust_completed);
    json(&root.join("unchanged-refusal.json"), &serde_json::json!({"c_fallback":true,"before":before,"after":after}))?;
    Ok(names.len() + 2)
}
const FATALS: &[(&str, &str)] = &[
    ("dirty-refusal", "Waver refusal changed private WRAM"),
    ("request-refusal", "Waver refusal after foreign request"),
    ("postrequest-error", "postrequest Waver control"),
    ("postrequest-panic", "Rust Waver panic"),
    ("wrong-buffer", "Waver poisoned request"),
    ("short-buffer", "Waver poisoned request"),
    ("wrong-generation", "Waver poisoned request"),
    ("wrong-selector", "Waver poisoned request"),
    ("changed-state", "Waver poisoned request"),
    ("wrong-argument", "Waver poisoned request"),
    ("wrong-order", "Waver poisoned request"),
    ("service-outside-main", "Waver poisoned request"),
    ("wrong-output", "invalid Waver typed result"),
    ("wrong-continuation", "invalid Waver typed result"),
    ("recursive-root", "recursive or cross-consumer Waver root"),
    ("sibling-player", "recursive or cross-consumer player-motion root"),
    ("player-to-waver", "recursive or cross-consumer Waver root"),
    ("recursive-observer", "recursive or cross-consumer Waver root"),
    ("service-observer-error", "Waver observer failed"),
];
fn fatal(game: &mut Game, native: Native, root: &Path, mode: &str) -> Result<()> {
    let f = fixture("main-zero-sine");
    native.seed(&f.wram);
    write(&root.join("input.wram"), &f.wram)?;
    json(&root.join("invocation.json"), &serde_json::json!({"mode":mode,"expected_exit":2}))?;
    if mode == "recursive-observer" {
        game.set_waver_observer(move |_, _| {
            unsafe { (native.ai)(0) };
            Ok(())
        })?;
    } else if mode == "service-observer-error" {
        install(game)?;
        game.set_waver_observer(|e, _| {
            if e.phase == WaverPhase::ServiceEnd { Err("Waver service-end observer control".into()) } else { Ok(()) }
        })?;
    } else if mode == "player-to-waver" {
        game.set_player_motion_hook(move |_, _, _, _| {
            unsafe { (native.ai)(0) };
            Ok(HookOutcome::Refused)
        })?;
        unsafe { (native.player_down)(0) };
        return Err("player-to-Waver fatal unexpectedly returned".into());
    } else {
        let mode = mode.to_owned();
        game.set_waver_hook(move |_, _, jp, ram, _, services| {
            let k = word(ram, 0xe54);
            let a = amount(ram);
            match mode.as_str() {
                "dirty-refusal" => ram[0] ^= 1,
                "request-refusal" => {
                    services.move_horizontal(ram, k, a)?;
                }
                "postrequest-error" => {
                    services.move_horizontal(ram, k, a)?;
                    return Err("postrequest Waver control".into());
                }
                "postrequest-panic" => {
                    services.move_horizontal(ram, k, a)?;
                    panic!("postrequest Waver control");
                }
                "wrong-buffer" => {
                    let _ = services.move_horizontal(&mut ram.to_vec(), k, a);
                }
                "short-buffer" => {
                    let _ = services.move_horizontal(&mut ram[..WRAM - 1], k, a);
                }
                "wrong-generation" | "wrong-selector" => {
                    let generation = unsafe { (native.generation)() } + u64::from(mode == "wrong-generation");
                    let mut result = 0;
                    unsafe {
                        (native.service)(
                            ram.as_mut_ptr(),
                            ram.len(),
                            if mode == "wrong-selector" { 3 } else { 0 },
                            k,
                            a,
                            0,
                            &mut result,
                            generation,
                        );
                    }
                }
                "changed-state" => {
                    ram[0] ^= 1;
                    let _ = services.move_horizontal(ram, k, a);
                }
                "wrong-argument" => {
                    let _ = services.move_horizontal(ram, k, a + 1);
                }
                "wrong-order" => {
                    let _ = services.sine(ram, 0, 4);
                }
                "service-outside-main" => {
                    let _ = services.move_horizontal(ram, k, a);
                }
                "wrong-output" => return Ok(Some(WaverOutput::Continue(0))),
                "wrong-continuation" => return Ok(Some(WaverOutput::Continue(jp.wrapping_add(2)))),
                "recursive-root" => unsafe { (native.ai)(0) },
                "sibling-player" => unsafe { (native.player_down)(0) },
                _ => return Err("unknown Waver fatal mode".into()),
            }
            Ok(None)
        })?;
    }
    if mode == "wrong-continuation" {
        let f = fixture("instruction-current");
        native.call(&f, false);
    } else {
        unsafe { (native.ai)(u32::from(mode != "service-outside-main")) };
    }
    Err("fatal Waver control unexpectedly returned".into())
}
fn fatals(args: &[String], root: &Path, start: usize) -> Result<usize> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    for &(mode, expected) in &FATALS[start..] {
        let directory = root.join(format!("process-{mode}"));
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let stdout = fs::File::create(directory.join("stdout.txt")).map_err(|e| e.to_string())?;
        let stderr = fs::File::create(directory.join("stderr.txt")).map_err(|e| e.to_string())?;
        let mut child = Command::new(&exe)
            .args(&args[1..4])
            .arg(format!("fatal-{mode}"))
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .map_err(|e| e.to_string())?;
        let started = Instant::now();
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
        let stderr = read(&directory.join("stderr.txt"))?;
        let verified = !timed_out && status.code() == Some(2) && String::from_utf8_lossy(&stderr).contains(expected);
        json(
            &directory.join("process.json"),
            &serde_json::json!({"mode":mode,"expected_error":expected,"exit_code":status.code(),"expected_exit_code":2,"timed_out":timed_out,"verified":verified,"elapsed_ms":started.elapsed().as_millis(),"stderr_sha256":hash(&stderr),"stdout_sha256":hash(&read(&directory.join("stdout.txt"))?),"argv":[exe.to_string_lossy().as_ref(),&args[1],&args[2],&args[3],&format!("fatal-{mode}")]}),
        )?;
        if !verified {
            return Err(format!("fatal {mode} failed: {}", directory.display()));
        }
    }
    Ok(FATALS.len() - start)
}
fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: waver-host-control ARCHIVE ROM OUTPUT MODE(prepare|all|bodies|admissions|fatals|fatals-START|fatal-NAME)".into());
    }
    for i in [1, 2] {
        args[i] = Path::new(&args[i]).canonicalize().map_err(|e| e.to_string())?.to_string_lossy().into_owned();
    }
    fs::create_dir_all(&args[3]).map_err(|e| e.to_string())?;
    args[3] = Path::new(&args[3]).canonicalize().map_err(|e| e.to_string())?.to_string_lossy().into_owned();
    let prepared = prepare(Path::new(&args[1]), Path::new(&args[3]))?;
    if args[4] == "prepare" {
        println!("library={}", prepared.library.display());
        return Ok(());
    }
    let mode = &args[4];
    let start = if mode == "fatals" {
        Some(0)
    } else if let Some(s) = mode.strip_prefix("fatals-") {
        Some(s.parse::<usize>().ok().filter(|&i| i < FATALS.len()).ok_or("invalid fatal start")?)
    } else {
        None
    };
    if !matches!(mode.as_str(), "all" | "bodies" | "admissions")
        && start.is_none()
        && !FATALS.iter().any(|(name, _)| mode == &format!("fatal-{name}"))
    {
        return Err("unknown Waver mode".into());
    }
    let root = PathBuf::from(&args[3]).join(format!("waver-{mode}"));
    fs::create_dir(&root).map_err(|e| format!("fresh output: {e}"))?;
    if let Some(start) = start {
        let count = fatals(&args, &root, start)?;
        json(
            &root.join("result.json"),
            &serde_json::json!({"status":"passed","fatal_controls":count,"skipped":start,"body_pairs":0}),
        )?;
        return Ok(());
    }
    let mut game = Game::boot(&prepared.library, Path::new(&args[2]))?;
    let library = unsafe { Library::new(&prepared.library).map_err(|e| e.to_string())? };
    let native = unsafe { Native::load(&library)? };
    assert_eq!(unsafe { (native.prepared)() }, 1, "prepared spike ROM mapping differs");
    if let Some(mode) = mode.strip_prefix("fatal-") {
        return fatal(&mut game, native, &root, mode);
    }
    let records = observe(&mut game)?;
    let mut body_count = 0;
    let mut chain_count = 0;
    let mut interpreter_count = 0;
    if mode != "admissions" {
        let bodies = body_fixtures();
        assert_eq!(bodies.len(), 24);
        let mut continuations = Vec::new();
        for case in &bodies {
            let ram = pair(&mut game, native, &records, case, &root, false)?;
            if matches!(case.name.as_str(), "init-right" | "instruction-current") {
                continuations.push(BodyFixture {
                    name: format!("chain-{}-main", case.name),
                    entry: Entry::Main,
                    k: 0,
                    jp: 0,
                    wram: ram,
                });
            }
        }
        body_count = bodies.len();
        for case in &continuations {
            pair(&mut game, native, &records, case, &root, false)?;
            chain_count += 1;
        }
        for name in ["instruction-current", "instruction-distinct-k"] {
            let mut case = fixture(name);
            case.name = format!("interpreter-{name}");
            case.k = word(&case.wram, 0xe54);
            let b = base(&case.wram);
            assert_eq!(word(game.rom(), 0x110000 + usize::from(case.jp) - 2), 0x86e3);
            assert_eq!(word(game.rom(), 0x110000 + usize::from(case.jp)), 0x812f);
            put(&mut case.wram, b + 0x12, 0);
            put(&mut case.wram, b + 0x1c, 1);
            put(&mut case.wram, b + 0x1a, case.jp - 2);
            case.wram[b + 0x2e] = 0xa3;
            pair(&mut game, native, &records, &case, &root, true)?;
            interpreter_count += 1;
        }
    }
    let admission_count = admissions(&mut game, native, &root)?;
    let fatal_count = if mode == "all" { fatals(&args, &root, 0)? } else { 0 };
    json(
        &root.join("result.json"),
        &serde_json::json!({"status":"passed","shared_body_pairs":body_count,"chain_pairs":chain_count,"interpreter_pairs":interpreter_count,"admission_controls":admission_count,"fatal_controls":fatal_count,"library_sha256":hash(&read(&prepared.library)?),"executable_sha256":hash(&read(&std::env::current_exe().map_err(|e|e.to_string())?)?),"original_rom_comparison":"not-tested","live_ownership":"not-tested"}),
    )?;
    println!(
        "waver_body_pairs={body_count} chain_pairs={chain_count} interpreter_pairs={interpreter_count} admissions={admission_count} fatals={fatal_count}"
    );
    Ok(())
}
