//! Frozen Ripper actual-C controls. Only root runs producers and cleanup.
use libloading::Library;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sm_native_host::{Game, RipperObservation, RipperPhase, RipperServices, WaverOutput, WaverServices, prepare};
use sm_ripper::{BodyFixture, Entry, Services, body_fixtures};
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
    invoke: unsafe extern "C" fn(u32),
    admitted: unsafe extern "C" fn(u32) -> i32,
    prepared: unsafe extern "C" fn() -> i32,
    service: unsafe extern "C" fn(*mut u8, usize, u32, u16, i32, *mut u8, u64) -> i32,
    generation: unsafe extern "C" fn() -> u64,
    ai: unsafe extern "C" fn(u32),
    waver: unsafe extern "C" fn(u32),
    waver_invoke: unsafe extern "C" fn(u32, u16, u16) -> u16,
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        unsafe {
            Ok(Self {
                seed: *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                invoke: *library.get(b"sm_native_invoke_ripper\0").map_err(|e| e.to_string())?,
                admitted: *library.get(b"sm_native_ripper_fixture_admitted\0").map_err(|e| e.to_string())?,
                prepared: *library.get(b"sm_native_ripper_prepared_tables\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_ripper_service\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_ripper_generation\0").map_err(|e| e.to_string())?,
                ai: *library.get(b"sm_native_ripper_ai\0").map_err(|e| e.to_string())?,
                waver: *library.get(b"sm_native_waver_ai\0").map_err(|e| e.to_string())?,
                waver_invoke: *library.get(b"sm_native_invoke_waver\0").map_err(|e| e.to_string())?,
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
    fn call(self, case: &BodyFixture) {
        assert_eq!(unsafe { (self.admitted)(case.entry as u32) }, 1, "unsafe fixture {}", case.name);
        unsafe { (self.invoke)(case.entry as u32) };
    }
}
struct Bridge<'a>(&'a mut RipperServices);
impl Services for Bridge<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn move_horizontal(&mut self, ram: &mut [u8], slot: u16, amount: i32) -> Result<u8> {
        self.0.move_horizontal(ram, slot, amount)
    }
}
struct Unavailable;
impl Services for Unavailable {
    fn available(&self) -> bool {
        false
    }
    fn move_horizontal(&mut self, _: &mut [u8], _: u16, _: i32) -> Result<u8> {
        Err("unexpected movement".into())
    }
}
struct InvalidCollision;
impl Services for InvalidCollision {
    fn available(&self) -> bool {
        true
    }
    fn move_horizontal(&mut self, _: &mut [u8], _: u16, _: i32) -> Result<u8> {
        Ok(2)
    }
}
fn install(game: &mut Game) -> Result<()> {
    game.set_ripper_hook(|entry, ram, rom, services| {
        sm_ripper::execute(Entry::try_from(entry as u32)?, ram, rom, &mut Bridge(services))
    })
}
struct WaverBridge<'a>(&'a mut WaverServices);
impl sm_waver::Services for WaverBridge<'_> {
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
fn install_waver(game: &mut Game) -> Result<()> {
    game.set_waver_hook(|entry, k, jp, ram, rom, services| {
        Ok(sm_waver::execute(sm_waver::Entry::try_from(entry as u32)?, k, jp, ram, rom, &mut WaverBridge(services))?
            .map(|o| match o {
                sm_waver::Output::Done => WaverOutput::Done,
                sm_waver::Output::Continue(jp) => WaverOutput::Continue(jp),
            }))
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    event: RipperObservation,
    ram: Vec<u8>,
}
type Records = Rc<RefCell<Vec<Record>>>;
fn observe(game: &mut Game) -> Result<Records> {
    let records = Records::default();
    let output = records.clone();
    game.set_ripper_observer(move |event, ram| {
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
fn put(ram: &mut [u8], at: usize, v: u16) {
    ram[at..at + 2].copy_from_slice(&v.to_le_bytes());
}
fn base(ram: &[u8]) -> usize {
    0xf78 + usize::from(word(ram, 0xe54))
}
fn amount(ram: &[u8]) -> i32 {
    let b = base(ram);
    ((u32::from(word(ram, b + 0x36)) << 16) | u32::from(word(ram, b + 0x34))) as i32
}
fn compare(name: &str, left: &[u8], right: &[u8]) -> Result<()> {
    if left == right {
        return Ok(());
    }
    let at = left.iter().zip(right).position(|(a, b)| a != b).unwrap_or(left.len().min(right.len()));
    Err(format!("{name} mismatch {at:#x}: {} vs {}", hash(left), hash(right)))
}
fn records_bytes(records: &[Record]) -> Vec<u8> {
    let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
    for r in records {
        for v in [
            r.event.phase as u32,
            r.event.entry as u32,
            u32::from(r.event.slot),
            r.event.amount as u32,
            u32::from(r.event.result),
        ] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend(&r.ram);
    }
    bytes
}
fn fixture(name: &str) -> BodyFixture {
    body_fixtures().into_iter().find(|f| f.name == name).unwrap()
}
fn pair(game: &mut Game, native: Native, records: &Records, case: &BodyFixture, root: &Path) -> Result<(Vec<u8>, u8)> {
    let directory = root.join(&case.name);
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    write(&directory.join("input.wram"), &case.wram)?;
    let mut executions = Vec::new();
    for rust in [false, true] {
        if rust {
            install(game)?;
        } else {
            game.clear_ripper_hook();
        }
        native.seed(&case.wram);
        records.borrow_mut().clear();
        let before = game.ripper_counters();
        let waver_before = game.waver_counters();
        native.call(case);
        let after = game.ripper_counters();
        assert_eq!(game.waver_counters(), waver_before, "Ripper must not count as Waver movement");
        let ram = game.wram().to_vec();
        let effects = game.effects().to_vec();
        let seen = records.borrow().clone();
        let label = if rust { "rust" } else { "c" };
        write(&directory.join(format!("{label}.wram")), &ram)?;
        write(&directory.join(format!("{label}.effects")), &effects)?;
        write(&directory.join(format!("{label}.observations")), records_bytes(&seen))?;
        json(
            &directory.join(format!("{label}.json")),
            &serde_json::json!({"before":before,"after":after,"waver_before":waver_before,"waver_after":game.waver_counters()}),
        )?;
        let i = case.entry as usize;
        let movement = u64::from(i == 1 || i == 3);
        assert_eq!(after.entries[i] - before.entries[i], 1);
        assert_eq!(after.rust_refused, before.rust_refused);
        assert_eq!(after.invalid_requests, before.invalid_requests);
        if rust {
            assert_eq!(after.rust_completed[i] - before.rust_completed[i], 1);
            assert_eq!(after.c_entries, before.c_entries);
        } else {
            assert_eq!(after.c_entries[i] - before.c_entries[i], 1);
            assert_eq!(after.rust_completed, before.rust_completed);
        }
        assert_eq!(after.service_requests - before.service_requests, movement);
        assert_eq!(after.service_completions - before.service_completions, movement);
        assert_eq!(seen.len(), 2 + 2 * movement as usize);
        assert_eq!(seen[0].event.phase, RipperPhase::RootBegin);
        assert_eq!(seen.last().unwrap().event.phase, RipperPhase::RootEnd);
        executions.push((ram, effects, seen));
    }
    let (c_ram, c_effects, c_seen) = &executions[0];
    let (ram, effects, seen) = &executions[1];
    compare(&case.name, c_ram, ram)?;
    compare("effects", c_effects, effects)?;
    assert_eq!(c_seen, seen);
    if matches!(case.entry, Entry::JetMain | Entry::RipperMain) {
        let b = base(&case.wram);
        assert_eq!(&ram[b + 6..b + 10], &case.wram[b + 6..b + 10]);
        if seen.last().unwrap().event.result == 0 {
            let start = (u32::from(word(&case.wram, b + 2)) << 16) | u32::from(word(&case.wram, b + 4));
            let end = (u32::from(word(ram, b + 2)) << 16) | u32::from(word(ram, b + 4));
            assert_eq!(end, start.wrapping_add(amount(&case.wram) as u32), "one native movement");
        }
    }
    if case.name.contains("spike-") {
        let events: Vec<_> = effects.as_chunks::<16>().0.iter().filter(|v| matches!(word(*v, 0), 3 | 4)).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(u32::from_le_bytes(events[0][4..8].try_into().unwrap()), 0xd094);
        assert_eq!(word(events[1], 0), 4);
        assert_eq!(u32::from_le_bytes(events[1][4..8].try_into().unwrap()), 0);
        assert_eq!(word(ram, 0x10002 + 104), if case.name.ends_with("allocate") { 0 } else { 0xa000 });
        assert_eq!(word(ram, 0x1c37 + 78), 0xd094);
    }
    json(
        &directory.join("pair.json"),
        &serde_json::json!({"actual_c":true,"full_wram":true,"ordered_effects":true,"actual_ai_dispatch":true,"waver_registered":true,"waver_counters_unchanged":true,"wram_sha256":hash(ram),"effects_sha256":hash(effects)}),
    )?;
    println!("pair={} actual_c=true full_wram=true", case.name);
    Ok((ram.clone(), seen.last().unwrap().event.result))
}
fn coexistence(game: &mut Game, native: Native, root: &Path) -> Result<()> {
    let case = sm_waver::body_fixtures().into_iter().find(|f| f.name == "main-positive-sine").unwrap();
    let directory = root.join("waver-coexistence");
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    write(&directory.join("input.wram"), &case.wram)?;
    install(game)?;
    let mut outputs = Vec::new();
    for rust in [false, true] {
        if rust {
            install_waver(game)?;
        } else {
            game.clear_waver_hook();
        }
        native.seed(&case.wram);
        let before = game.waver_counters();
        let ripper_before = game.ripper_counters();
        assert_eq!(unsafe { (native.waver_invoke)(case.entry as u32, case.k, case.jp) }, 0);
        assert_eq!(game.ripper_counters(), ripper_before, "Waver movement must not count as Ripper");
        let after = game.waver_counters();
        let b = base(&case.wram);
        let start = (u32::from(word(&case.wram, b + 2)) << 16) | u32::from(word(&case.wram, b + 4));
        let end = (u32::from(word(game.wram(), b + 2)) << 16) | u32::from(word(game.wram(), b + 4));
        let velocity = (u32::from(word(&case.wram, b + 0x32)) << 16) | u32::from(word(&case.wram, b + 0x30));
        assert_eq!(end, start.wrapping_add(velocity), "one Waver native horizontal movement");
        for i in 0..3 {
            assert_eq!(after.service_requests[i] - before.service_requests[i], 1);
            assert_eq!(after.service_completions[i] - before.service_completions[i], 1);
        }
        if rust {
            assert_eq!(after.rust_completed[1] - before.rust_completed[1], 1);
            assert_eq!(after.c_entries, before.c_entries);
        } else {
            assert_eq!(after.c_entries[1] - before.c_entries[1], 1);
        }
        let label = if rust { "rust" } else { "c" };
        write(&directory.join(format!("{label}.wram")), game.wram())?;
        write(&directory.join(format!("{label}.effects")), game.effects())?;
        json(
            &directory.join(format!("{label}.json")),
            &serde_json::json!({"waver_before":before,"waver_after":after,"ripper_before":ripper_before,"ripper_after":game.ripper_counters()}),
        )?;
        outputs.push((game.wram().to_vec(), game.effects().to_vec()));
    }
    compare("Waver coexistence WRAM", &outputs[0].0, &outputs[1].0)?;
    compare("Waver coexistence effects", &outputs[0].1, &outputs[1].1)?;
    json(
        &directory.join("pair.json"),
        &serde_json::json!({"actual_c":true,"both_hooks_registered":true,"one_horizontal_request":true,"ripper_counters_unchanged":true}),
    )?;
    Ok(())
}
fn admissions(game: &mut Game, native: Native, root: &Path) -> Result<usize> {
    let names = [
        "invalid-slot",
        "init-row65",
        "main-row65",
        "main-unaligned-row",
        "spike16",
        "extension-cycle",
        "extension-oob",
        "zero-width",
        "large-velocity",
        "geometry",
    ];
    for name in names {
        let mut f = fixture("jet-main-air-positive");
        let b = base(&f.wram);
        match name {
            "invalid-slot" => put(&mut f.wram, 0xe54, 1),
            "init-row65" => {
                f.entry = Entry::JetInit;
                put(&mut f.wram, b + 0x3c, 65);
            }
            "main-row65" => put(&mut f.wram, b + 0x38, 0x208),
            "main-unaligned-row" => put(&mut f.wram, b + 0x38, 7),
            "spike16" => {
                put(&mut f.wram, 0x10002 + 102, 0xa000);
                f.wram[0x16402 + 51] = 16;
            }
            "extension-cycle" => {
                put(&mut f.wram, 0x10002 + 102, 0x5000);
                f.wram[0x16402 + 51] = 1;
                put(&mut f.wram, 0x10002 + 104, 0x5000);
                f.wram[0x16402 + 52] = 255;
            }
            "extension-oob" => {
                put(&mut f.wram, 0x10002 + 102, 0xd000);
                f.wram[0x16402 + 51] = 128;
            }
            "zero-width" => put(&mut f.wram, 0x7a5, 0),
            "large-velocity" => put(&mut f.wram, b + 0x36, 17),
            "geometry" => put(&mut f.wram, b + 2, 1),
            _ => unreachable!(),
        }
        native.seed(&f.wram);
        let before = f.wram.clone();
        assert_eq!(unsafe { (native.admitted)(f.entry as u32) }, 0, "{name}");
        assert_eq!(sm_ripper::execute(f.entry, &mut f.wram, game.rom(), &mut Unavailable)?, None);
        compare("immutable private refusal", &before, &f.wram)?;
        compare("immutable canonical admission", &before, game.wram())?;
        let directory = root.join(format!("admission-{name}"));
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        write(&directory.join("input.wram"), &before)?;
        json(&directory.join("result.json"), &serde_json::json!({"unchanged":true,"refused_before_c":true}))?;
    }
    let mut f = fixture("jet-init-param0040-dir8000");
    let before = f.wram.clone();
    assert_eq!(sm_ripper::execute(f.entry, &mut f.wram, &game.rom()[..0x110187 + 519], &mut Unavailable)?, None);
    assert_eq!(f.wram, before);
    json(
        &root.join("short-rom.json"),
        &serde_json::json!({"unchanged":true,"available_table_bytes":519,"required":520}),
    )?;
    let mut short = f.wram[..WRAM - 1].to_vec();
    let before = short.clone();
    assert_eq!(sm_ripper::execute(f.entry, &mut short, game.rom(), &mut Unavailable)?, None);
    assert_eq!(short, before);
    json(&root.join("short-wram.json"), &serde_json::json!({"unchanged":true,"size":WRAM-1}))?;
    let f = fixture("jet-main-air-positive");
    game.clear_ripper_hook();
    native.seed(&f.wram);
    native.call(&f);
    let expected = game.wram().to_vec();
    let effects = game.effects().to_vec();
    game.set_ripper_hook(|_, _, _, _| Ok(None))?;
    native.seed(&f.wram);
    let before = game.ripper_counters();
    native.call(&f);
    let after = game.ripper_counters();
    compare("unchanged fallback", &expected, game.wram())?;
    compare("fallback effects", &effects, game.effects())?;
    assert_eq!(after.rust_refused[1] - before.rust_refused[1], 1);
    assert_eq!(after.c_entries[1] - before.c_entries[1], 1);
    assert_eq!(after.rust_completed, before.rust_completed);
    json(&root.join("unchanged-refusal.json"), &serde_json::json!({"before":before,"after":after,"c_fallback":true}))?;
    Ok(names.len() + 3)
}
const FATALS: &[(&str, &str)] = &[
    ("dirty-refusal", "Ripper refusal changed private WRAM"),
    ("request-refusal", "Ripper refusal after foreign request"),
    ("postrequest-error", "postrequest Ripper control"),
    ("postrequest-panic", "Rust Ripper panic"),
    ("wrong-buffer", "Ripper poisoned request"),
    ("short-buffer", "Ripper poisoned request"),
    ("wrong-generation", "Ripper poisoned request"),
    ("wrong-selector", "Ripper poisoned request"),
    ("wrong-slot", "Ripper poisoned request"),
    ("wrong-amount", "Ripper poisoned request"),
    ("changed-state", "Ripper poisoned request"),
    ("service-init", "Ripper poisoned request"),
    ("duplicate-service", "Ripper poisoned request"),
    ("invalid-collision", "invalid Ripper native collision result"),
    ("recursive-root", "recursive or cross-consumer Ripper root"),
    ("ripper-to-waver", "recursive or cross-consumer Waver root"),
    ("waver-to-ripper", "recursive or cross-consumer Ripper root"),
    ("recursive-observer", "recursive or cross-consumer Ripper root"),
    ("service-observer-error", "Ripper observer failed"),
];
fn fatal(game: &mut Game, native: Native, root: &Path, mode: &str) -> Result<()> {
    let f = fixture("jet-main-air-positive");
    native.seed(&f.wram);
    write(&root.join("input.wram"), &f.wram)?;
    json(
        &root.join("invocation.json"),
        &serde_json::json!({"mode":mode,"expected_exit":2,"invalid_collision_is_injected_trait_result":mode=="invalid-collision"}),
    )?;
    if mode == "recursive-observer" {
        game.set_ripper_observer(move |_, _| {
            unsafe { (native.ai)(0) };
            Ok(())
        })?;
    } else if mode == "service-observer-error" {
        install(game)?;
        game.set_ripper_observer(|e, _| {
            if e.phase == RipperPhase::ServiceEnd { Err("Ripper service-end observer control".into()) } else { Ok(()) }
        })?;
    } else if mode == "waver-to-ripper" {
        game.set_waver_hook(move |_, _, _, _, _, _| {
            unsafe { (native.ai)(0) };
            Ok(None)
        })?;
        unsafe { (native.waver)(0) };
        return Err("Waver-to-Ripper fatal unexpectedly returned".into());
    } else if mode == "invalid-collision" {
        game.set_ripper_hook(|entry, ram, rom, _| {
            sm_ripper::execute(Entry::try_from(entry as u32)?, ram, rom, &mut InvalidCollision)
        })?;
    } else {
        let mode = mode.to_owned();
        game.set_ripper_hook(move |_, ram, _, services| {
            let k = word(ram, 0xe54);
            let a = amount(ram);
            match mode.as_str() {
                "dirty-refusal" => ram[0] ^= 1,
                "request-refusal" => {
                    services.move_horizontal(ram, k, a)?;
                }
                "postrequest-error" => {
                    services.move_horizontal(ram, k, a)?;
                    return Err("postrequest Ripper control".into());
                }
                "postrequest-panic" => {
                    services.move_horizontal(ram, k, a)?;
                    panic!("postrequest Ripper control");
                }
                "wrong-buffer" => {
                    let _ = services.move_horizontal(&mut ram.to_vec(), k, a);
                }
                "short-buffer" => {
                    let _ = services.move_horizontal(&mut ram[..WRAM - 1], k, a);
                }
                "wrong-generation" | "wrong-selector" => {
                    let token = unsafe { (native.generation)() } + u64::from(mode == "wrong-generation");
                    let mut result = 0;
                    unsafe {
                        (native.service)(
                            ram.as_mut_ptr(),
                            ram.len(),
                            u32::from(mode == "wrong-selector"),
                            k,
                            a,
                            &mut result,
                            token,
                        );
                    }
                }
                "wrong-slot" => {
                    let _ = services.move_horizontal(ram, k + 64, a);
                }
                "wrong-amount" => {
                    let _ = services.move_horizontal(ram, k, a + 1);
                }
                "changed-state" => {
                    ram[0] ^= 1;
                    let _ = services.move_horizontal(ram, k, a);
                }
                "service-init" => {
                    let _ = services.move_horizontal(ram, k, a);
                }
                "duplicate-service" => {
                    services.move_horizontal(ram, k, a)?;
                    let _ = services.move_horizontal(ram, k, a);
                }
                "recursive-root" => unsafe { (native.ai)(0) },
                "ripper-to-waver" => unsafe { (native.waver)(0) },
                _ => return Err("unknown Ripper fatal mode".into()),
            }
            Ok(None)
        })?;
    }
    unsafe { (native.ai)(u32::from(mode != "service-init")) };
    Err("fatal Ripper control unexpectedly returned".into())
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
        return Err("usage: ripper-host-control ARCHIVE ROM OUTPUT MODE(prepare|all|bodies|admissions|coexistence|fatals|fatals-START|fatal-NAME)".into());
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
    if !matches!(mode.as_str(), "all" | "bodies" | "admissions" | "coexistence")
        && start.is_none()
        && !FATALS.iter().any(|(name, _)| mode == &format!("fatal-{name}"))
    {
        return Err("unknown Ripper mode".into());
    }
    let root = PathBuf::from(&args[3]).join(format!("ripper-{mode}"));
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
    assert_eq!(unsafe { (native.prepared)() }, 1, "prepared65row/PLM table binding differs");
    if let Some(mode) = mode.strip_prefix("fatal-") {
        return fatal(&mut game, native, &root, mode);
    }
    if mode == "coexistence" {
        coexistence(&mut game, native, &root)?;
        json(&root.join("result.json"), &serde_json::json!({"status":"passed","waver_coexistence_pairs":1}))?;
        return Ok(());
    }
    install_waver(&mut game)?;
    let records = observe(&mut game)?;
    let mut body_count = 0;
    let mut chain_count = 0;
    let mut coexistence_count = 0;
    if mode != "admissions" {
        let bodies = body_fixtures();
        assert_eq!(bodies.len(), 46);
        for case in &bodies {
            pair(&mut game, native, &records, case, &root)?;
        }
        body_count = bodies.len();
        for (variant, main) in [("jet", Entry::JetMain), ("ripper", Entry::RipperMain)] {
            let mut case = fixture(&format!("{variant}-init-param0040-dir8000"));
            case.name = format!("chain-{variant}-init");
            let b = base(&case.wram);
            put(&mut case.wram, b + 2, 0x38);
            for index in [52, 68] {
                put(&mut case.wram, 0x10002 + index * 2, 0x8000);
                case.wram[0x16402 + index] = 0;
            }
            let (ram, _) = pair(&mut game, native, &records, &case, &root)?;
            chain_count += 1;
            case = BodyFixture { name: format!("chain-{variant}-collision"), entry: main, wram: ram };
            let (ram, collision) = pair(&mut game, native, &records, &case, &root)?;
            assert_eq!(collision, 1);
            chain_count += 1;
            case = BodyFixture { name: format!("chain-{variant}-return-air"), entry: main, wram: ram };
            let (_, collision) = pair(&mut game, native, &records, &case, &root)?;
            assert_eq!(collision, 0);
            chain_count += 1;
        }
        coexistence(&mut game, native, &root)?;
        coexistence_count = 1;
    }
    let admission_count = admissions(&mut game, native, &root)?;
    let fatal_count = if mode == "all" { fatals(&args, &root, 0)? } else { 0 };
    json(
        &root.join("result.json"),
        &serde_json::json!({"status":"passed","shared_body_pairs":body_count,"chain_pairs":chain_count,"waver_coexistence_pairs":coexistence_count,"admission_controls":admission_count,"fatal_controls":fatal_count,"library_sha256":hash(&read(&prepared.library)?),"executable_sha256":hash(&read(&std::env::current_exe().map_err(|e|e.to_string())?)?),"runtime_table_sha256":hash(&game.rom()[0x110187..0x110187+520]),"original_rom_comparison":"not-tested","live_ownership":"not-tested"}),
    )?;
    println!(
        "ripper_body_pairs={body_count} chain_pairs={chain_count} coexistence={coexistence_count} admissions={admission_count} fatals={fatal_count}"
    );
    Ok(())
}
