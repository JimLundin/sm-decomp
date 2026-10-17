//! Actual-C exposed-tank fixtures. Scheduler pairs use fresh native processes
//! because a real pending message changes device state outside copied WRAM.
#[allow(dead_code)]
#[path = "item-instruction-host-control.rs"]
mod item_control;

use item_control::{
    Native, Preflight, Records, SEED_SHA, compare, compare_records, hash, install, observe, read, word,
};
use libloading::Library;
use serde::{Deserialize, Serialize};
use sm_item_instructions::{Entry, fixtures};
use sm_native_host::{Game, ItemInstructionOutcome as Outcome, ItemInstructionPhase as Phase, prepare};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const BODY_CASES: usize = 36;
const REFUSAL_CASES: usize = 21;
const SLOT: u16 = 68;
const HEADERS: [u16; 5] = [0xeed7, 0xeedb, 0xeedf, 0xeee3, 0xef27];

fn body_pairs(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let cases = fixtures::tank_body_fixtures(seed, game.rom())?;
    assert_eq!(cases.len(), BODY_CASES);
    for case in cases {
        let entry = case.entry.expect("tank body entry");
        game.clear_item_instruction_hook();
        native.seed(&case.ram);
        records.borrow_mut().clear();
        let c_result = native.invoke(game, entry, case.slot, case.operand);
        let c_ram = game.wram().to_vec();
        let c_effects = game.effects().to_vec();
        let c_records = records.borrow().clone();
        install(game)?;
        native.seed(&case.ram);
        records.borrow_mut().clear();
        let before = game.item_instruction_counters();
        let result = native.invoke(game, entry, case.slot, case.operand);
        assert_eq!(result, c_result, "{} actual-C return", case.id);
        assert_eq!(case.expected, fixtures::Expectation::Handled(result), "{} expected return", case.id);
        compare(case.id, &c_ram, game.wram());
        compare(case.id, &c_effects, game.effects());
        compare_records(case.id, &c_records, &records.borrow());
        let seen = records.borrow();
        assert!(seen.iter().all(|r| r.observation.scope_id == 3 && r.observation.scope_identity));
        let calls: Vec<_> = seen
            .iter()
            .filter(|r| r.observation.phase == Phase::ServiceBegin)
            .map(|r| (r.observation.value >> 16, r.observation.value as u16))
            .collect();
        let expected: Vec<_> = case.service_calls.iter().map(|(service, arg)| (*service as u32, *arg)).collect();
        assert_eq!(calls, expected, "{} native service order", case.id);
        assert_eq!(seen.len(), 2 + 2 * calls.len(), "{} complete service images", case.id);
        let after = game.item_instruction_counters();
        assert_eq!(
            after.tank_standalone.rust_completed[entry as usize]
                - before.tank_standalone.rust_completed[entry as usize],
            1
        );
        assert_eq!(after.tank_standalone.c_entries, before.tank_standalone.c_entries);
        assert_eq!(after.tank_standalone.rust_refused, before.tank_standalone.rust_refused);
        assert_eq!(after.scope_standalone, before.scope_standalone);
        assert_eq!(after.morph_standalone, before.morph_standalone);
        for id in 0..8 {
            let expected = calls.iter().filter(|(service, _)| *service == id as u32).count() as u64;
            assert_eq!(after.service_requests[id] - before.service_requests[id], expected);
            assert_eq!(after.service_completions[id] - before.service_completions[id], expected);
        }
        println!(
            "body={} scope=tank result={result:?} wram={} observer_records={} actual_c=true rust_completed=1",
            case.id,
            hash(game.wram()),
            seen.len()
        );
    }
    Ok(())
}

fn refusals(game: &Game, seed: &[u8]) -> Result<(), String> {
    let cases = fixtures::tank_refusal_fixtures(seed, game.rom())?;
    assert_eq!(cases.len(), REFUSAL_CASES);
    for mut case in cases {
        let before = case.ram.clone();
        let mut services = Preflight { available: case.services_available, calls: 0 };
        let result = case.run(&mut services);
        match case.expected {
            fixtures::Expectation::Refused => assert_eq!(result?, None, "{}", case.id),
            fixtures::Expectation::Error => assert!(result.is_err(), "{} must fail preflight", case.id),
            fixtures::Expectation::Handled(_) => panic!("tank refusal marked handled"),
        }
        compare(case.id, &before, &case.ram);
        assert_eq!(services.calls, 0);
        println!("preflight={} scope=tank unchanged=true services=0 unchecked_c=false", case.id);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Domain {
    start: u16,
    collector: Entry,
    message: u16,
    continuation: u16,
}
struct ChainSelection {
    rust: bool,
    header: u16,
    collected: bool,
}
fn domain(header: u16) -> Result<Domain, String> {
    let (start, collector, message, continuation) = match header {
        0xeed7 => (0xe099, Entry::CollectHealthEnergyTank, 1, 0xe0ba),
        0xeedb => (0xe0be, Entry::CollectAmmoMissileTank, 2, 0xe0df),
        0xeedf => (0xe0e3, Entry::CollectAmmoSuperMissileTank, 3, 0xe104),
        0xeee3 => (0xe108, Entry::CollectAmmoPowerBombTank, 4, 0xe129),
        0xef27 => (0xe41d, Entry::CollectHealthReserveTank, 0x19, 0xe446),
        _ => return Err("unknown tank chain header".into()),
    };
    Ok(Domain { start, collector, message, continuation })
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct StepReport {
    label: String,
    result: u8,
    wram_sha256: String,
    effects_sha256: String,
    observations_sha256: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ChainReport {
    header: u16,
    argument: u16,
    collected: bool,
    scheduler_calls: usize,
    pending_message: Option<u16>,
    presentation_completed: bool,
    steps: Vec<StepReport>,
}

fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), String> {
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}
fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    write(path, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
}
fn phase_file(directory: &Path, index: usize, suffix: &str) -> PathBuf {
    directory.join(format!("phase-{index:02}.{suffix}"))
}

fn save_step(
    game: &Game,
    records: &Records,
    directory: &Path,
    index: usize,
    label: &str,
    result: u8,
) -> Result<StepReport, String> {
    let records = records.borrow();
    // Fixed record encoding: count:u32 then phase/entry/slot/value/context/scope
    // as six little-endian u32 values, followed by complete128KiB WRAM each.
    let mut observations = Vec::new();
    observations.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for record in records.iter() {
        let at = record.observation;
        for value in [
            at.phase as u32,
            at.entry.map_or(u32::MAX, |entry| entry as u32),
            u32::from(at.slot),
            at.value,
            at.context,
            at.scope_id,
        ] {
            observations.extend_from_slice(&value.to_le_bytes());
        }
        observations.extend_from_slice(&record.ram);
    }
    write(&phase_file(directory, index, "wram"), game.wram())?;
    write(&phase_file(directory, index, "effects"), game.effects())?;
    write(&phase_file(directory, index, "observations"), &observations)?;
    Ok(StepReport {
        label: label.into(),
        result,
        wram_sha256: hash(game.wram()),
        effects_sha256: hash(game.effects()),
        observations_sha256: hash(&observations),
    })
}

struct ChainRun<'a> {
    native: Native,
    game: &'a Game,
    records: &'a Records,
    directory: &'a Path,
    steps: Vec<StepReport>,
    scheduler_calls: usize,
}
impl ChainRun<'_> {
    fn step(&mut self, label: &str, scheduler: bool, action: impl FnOnce() -> u8) -> Result<u8, String> {
        if scheduler {
            self.scheduler_calls += 1;
            if self.scheduler_calls > 16 {
                return Err("tank chain exceeded16scheduler resumptions".into());
            }
        }
        unsafe {
            (self.native.begin)();
        }
        self.records.borrow_mut().clear();
        let result = action();
        assert_eq!(unsafe { (self.native.item_active)() }, 0, "item borrow survived scheduler return");
        self.steps.push(save_step(self.game, self.records, self.directory, self.steps.len(), label, result)?);
        Ok(result)
    }
}

fn chain_child(
    args: &[String],
    game: &mut Game,
    native: Native,
    library: &Library,
    seed: &[u8],
    records: &Records,
    selection: ChainSelection,
) -> Result<(), String> {
    let ChainSelection { rust, header, collected } = selection;
    let spec = domain(header)?;
    let cases = fixtures::tank_body_fixtures(seed, game.rom())?;
    let fixture = cases
        .into_iter()
        .find(|case| read(&case.ram, 0x1c37 + usize::from(case.slot)) == header)
        .ok_or("no shared tank seed for header")?;
    assert_eq!(fixture.slot, SLOT);
    let mut ram = fixture.ram;
    let argument = read(&ram, 0x1dc7 + usize::from(SLOT));
    assert!(argument <= 0x1ff);
    for slot in (0..80).step_by(2) {
        word(&mut ram, 0x1c37 + slot, 0);
    }
    ram[0x611..=0x615].fill(0);
    for (at, value) in [
        (0x1c23, 0x8000),
        (0x1c37 + usize::from(SLOT), header),
        (0x1cd7 + usize::from(SLOT), 0x86d0),
        (0x1d27 + usize::from(SLOT), spec.start),
        (0xde1c + usize::from(SLOT), 1),
        (0xdc8, 0),
        (0x998, 11),
        (0x99c, 0xe4a9),
        (0x943, 0),
        (0x330, 0),
        (0x1c25, 0),
        (0x1c2d, 0),
        (0xa78, 0),
    ] {
        word(&mut ram, at, value);
    }
    ram[0x612] = 2; // Real native LoadMoreThings resumption at service52.
    let bit_at = 0xd870 + usize::from(argument >> 3);
    let mask = 1u8 << (argument & 7);
    if collected {
        ram[bit_at] |= mask;
    } else {
        ram[bit_at] &= !mask;
    }
    let inventory = ram[0x9c0..0x9d8].to_vec();
    if rust {
        install(game)?;
    } else {
        game.clear_item_instruction_hook();
    }
    native.seed(&ram);
    let directory = Path::new(&args[3]).join("tank-chains").join(&args[5]);
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    write(&directory.join("input.wram"), &ram)?;
    let mut run = ChainRun { native, game, records, directory: &directory, steps: Vec::new(), scheduler_calls: 0 };
    assert_eq!(run.step("native-room-transition-service52", true, || unsafe { (native.root11)() })?, 0);
    if collected {
        assert!(
            records.borrow().iter().all(|record| record.observation.phase != Phase::ServiceBegin),
            "collected initialization must omit pickup/music/frame services"
        );
    }
    if header == 0xef27 {
        assert_eq!(read(run.game.wram(), 0x1c2d), 2, "reserve gfx loaded even on collected path");
        assert_eq!(read(run.game.wram(), 0x1c2f), 0xe41f);
        assert_eq!(&run.game.wram()[0xd0..0xd7], &[0, 1, 0, 0x90, 0x89, 0, 0x3e]);
    }
    // Explicit fixture context transition; no native game-state transition or
    // ordinary-input gameplay claim is made by this direct scheduler fixture.
    let mut normal_context = run.game.wram().to_vec();
    word(&mut normal_context, 0x998, 8);
    native.seed(&normal_context);
    write(&directory.join("standalone-context.wram"), &normal_context)?;
    if collected {
        assert_eq!(read(run.game.wram(), 0x1d27 + usize::from(SLOT)), 0xdfad);
        assert_eq!(run.step("native-collected-cleanup", true, || unsafe { (native.plm)() })?, 0);
        assert_eq!(read(run.game.wram(), 0x1c37 + usize::from(SLOT)), 0);
        assert_eq!(read(run.game.wram(), 0xdc8), 0);
        compare("collected tank inventory", &inventory, &run.game.wram()[0x9c0..0x9d8]);
    } else {
        for index in 0..8 {
            assert_eq!(run.step(&format!("native-animation-{index}"), true, || unsafe { (native.plm)() })?, 0);
            assert_eq!(run.game.wram()[bit_at] & mask, 0);
        }
        if run.game.rom().get(0x26ed3..0x26ed7) != Some(&[0xab, 0xee, 0xe3, 0xaa][..]) {
            return Err("unsealed EED3 native trigger header".into());
        }
        let spawn: unsafe extern "C" fn(u16) -> u8 = unsafe { *library.get(b"SpawnPLM\0").map_err(|e| e.to_string())? };
        let mut trigger_input = run.game.wram().to_vec();
        let block = read(&trigger_input, 0x1c87 + usize::from(SLOT));
        word(&mut trigger_input, 0xdc4, block / 2);
        native.seed(&trigger_input);
        write(&directory.join("native-trigger-input.wram"), &trigger_input)?;
        assert_eq!(run.step("native-SpawnPLM-EED3-contact", false, || unsafe { spawn(0xeed3) })?, 0);
        assert_eq!(read(run.game.wram(), 0x1d77 + usize::from(SLOT)), 255);
        assert_eq!(run.step("native-pickup-first-message-pending", true, || unsafe { (native.plm)() })?, 1);
        assert_ne!(run.game.wram()[0x613], 0);
        assert_ne!(run.game.wram()[0x614], 0);
        assert_eq!(read(run.game.wram(), 0xdc8), spec.message);
        assert_eq!(read(run.game.wram(), 0x1d27 + usize::from(SLOT)), spec.continuation);
        assert_ne!(run.game.wram()[bit_at] & mask, 0);
        // Stop here. Clearing coroutine/message state would manufacture resume.
    }
    let counts = run.game.item_instruction_counters();
    if collected {
        for group in [counts.tank_standalone, counts.tank_service52] {
            assert_eq!(group.entries[spec.collector as usize], 0, "collected path executed collector");
        }
        assert_eq!(counts.service_requests, [0; 8]);
    }
    if rust {
        for group in [counts.tank_standalone, counts.tank_service52] {
            assert!(group.c_entries.iter().all(|&n| n == 0));
            assert!(group.rust_refused.iter().all(|&n| n == 0));
        }
        assert!(counts.tank_service52.rust_completed.iter().sum::<u64>() > 0);
        if !collected {
            assert_eq!(counts.tank_standalone.rust_completed[spec.collector as usize], 1);
        }
    } else {
        assert_eq!(counts.tank_standalone.rust_entries, [0; 21]);
        assert_eq!(counts.tank_service52.rust_entries, [0; 21]);
        if !collected {
            assert_eq!(counts.tank_standalone.c_entries[spec.collector as usize], 1);
        }
    }
    write_json(&directory.join("ownership.json"), &counts)?;
    let report = ChainReport {
        header,
        argument,
        collected,
        scheduler_calls: run.scheduler_calls,
        pending_message: (!collected).then_some(spec.message),
        presentation_completed: false,
        steps: run.steps,
    };
    assert_eq!(report.scheduler_calls, if collected { 2 } else { 10 });
    write_json(&directory.join("chain.json"), &report)?;
    println!(
        "chain={header:04X} collected={collected} scheduler_calls={} pending={:?} presentation_completed=false",
        report.scheduler_calls, report.pending_message
    );
    Ok(())
}

fn run_child(args: &[String], mode: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let directory = Path::new(&args[3]).join("tank-chains").join(mode);
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let stdout = fs::File::create(directory.join("stdout.txt")).map_err(|e| e.to_string())?;
    let stderr = fs::File::create(directory.join("stderr.txt")).map_err(|e| e.to_string())?;
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let argv: Vec<String> = std::iter::once(exe.to_string_lossy().into_owned())
        .chain(args[1..5].iter().cloned())
        .chain(std::iter::once(mode.to_owned()))
        .collect();
    let started_unix_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
    let started = Instant::now();
    let mut child = Command::new(&exe)
        .args(&args[1..5])
        .arg(mode)
        .current_dir(&cwd)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < Duration::from_secs(20) => std::thread::sleep(Duration::from_millis(10)),
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
    write_json(
        &directory.join("process.json"),
        &serde_json::json!({
            "argv": argv, "cwd": cwd, "started_unix_ms": started_unix_ms,
            "elapsed_ms": started.elapsed().as_millis(), "exit_code": status.code(), "timed_out": timed_out,
            "stdout_sha256": hash(&read_file(&directory.join("stdout.txt"))?),
            "stderr_sha256": hash(&read_file(&directory.join("stderr.txt"))?),
        }),
    )?;
    if !status.success() || timed_out {
        return Err(format!("tank child {mode} failed; see {}", directory.join("process.json").display()));
    }
    Ok(directory)
}

fn chain_pairs(args: &[String]) -> Result<(), String> {
    for collected in [false, true] {
        let class = if collected { "collected" } else { "normal" };
        for header in HEADERS {
            let c = run_child(args, &format!("chain-c-{header:04X}-{class}"))?;
            let r = run_child(args, &format!("chain-rust-{header:04X}-{class}"))?;
            let left: ChainReport =
                serde_json::from_slice(&read_file(&c.join("chain.json"))?).map_err(|e| e.to_string())?;
            let right: ChainReport =
                serde_json::from_slice(&read_file(&r.join("chain.json"))?).map_err(|e| e.to_string())?;
            assert_eq!(left, right, "tank {header:04X} {class} phase reports");
            for name in ["input.wram", "standalone-context.wram"] {
                compare(name, &read_file(&c.join(name))?, &read_file(&r.join(name))?);
            }
            if !collected {
                compare(
                    "native trigger input",
                    &read_file(&c.join("native-trigger-input.wram"))?,
                    &read_file(&r.join("native-trigger-input.wram"))?,
                );
            }
            for index in 0..left.steps.len() {
                for suffix in ["wram", "effects", "observations"] {
                    compare(
                        suffix,
                        &read_file(&phase_file(&c, index, suffix))?,
                        &read_file(&phase_file(&r, index, suffix))?,
                    );
                }
            }
            write_json(
                &Path::new(&args[3]).join("tank-chains").join(format!("pair-{header:04X}-{class}.json")),
                &serde_json::json!({"header": header, "collected": collected, "compared": true,
                    "scheduler_calls_per_child": left.scheduler_calls, "pending_message": left.pending_message,
                    "presentation_completed": false, "c": c, "rust": r}),
            )?;
            println!(
                "pair={header:04X}-{class} full_wram=true effects=true full_service_images=true pending={:?} scheduler_calls={}",
                left.pending_message, left.scheduler_calls
            );
        }
    }
    Ok(())
}

fn fatal(game: &mut Game, native: Native, seed: &[u8], mode: &str) -> Result<(), String> {
    let power = mode.starts_with("power-");
    let cases = fixtures::tank_body_fixtures(seed, game.rom())?;
    let case = cases.into_iter().find(|c| c.id == if power { "T11" } else { "T08" }).ok_or("missing HUD fixture")?;
    native.seed(&case.ram);
    let mode = mode.to_owned();
    game.set_item_instruction_hook(move |_, _, _, ram, _, services| {
        if mode.ends_with("wrong-ram") {
            let mut wrong = ram.to_vec();
            if power {
                let _ = services.add_power_bombs_to_hud(&mut wrong);
            } else {
                let _ = services.add_super_missiles_to_hud(&mut wrong);
            }
            return Ok(Outcome::Refused);
        }
        if power {
            services.add_power_bombs_to_hud(ram)?;
        } else {
            services.add_super_missiles_to_hud(ram)?;
        }
        if mode.ends_with("after-request-error") {
            if power {
                services.add_super_missiles_to_hud(ram)?;
            } else {
                services.add_power_bombs_to_hud(ram)?;
            }
            return Err("wrong collector HUD unexpectedly accepted".into());
        }
        Ok(Outcome::Refused)
    })?;
    native.invoke(game, case.entry.unwrap(), case.slot, case.operand);
    Err("fatal tank HUD control unexpectedly returned".into())
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: tank-instruction-host-control ARCHIVE ROM OUTPUT SEED_WRAM MODE".into());
    }
    let prepared = prepare(Path::new(&args[1]), Path::new(&args[3]))?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if args[5] == "prepare" {
        return Ok(());
    }
    if args[5] == "chains" {
        return chain_pairs(&args);
    }
    let seed = read_file(Path::new(&args[4]))?;
    if hash(&seed) != SEED_SHA {
        return Err("unsealed shared item fixture seed".into());
    }
    let mut game = Game::boot(&prepared.library, Path::new(&args[2]))?;
    let library = unsafe { Library::new(&prepared.library).map_err(|e| e.to_string())? };
    let native = unsafe { Native::load(&library)? };
    let records = observe(&mut game)?;
    if args[5].starts_with("chain-") {
        let parts: Vec<_> = args[5].split('-').collect();
        if parts.len() != 4 || !["c", "rust"].contains(&parts[1]) || !["normal", "collected"].contains(&parts[3]) {
            return Err("invalid tank chain mode".into());
        }
        let header = u16::from_str_radix(parts[2], 16).map_err(|e| e.to_string())?;
        return chain_child(
            &args,
            &mut game,
            native,
            &library,
            &seed,
            &records,
            ChainSelection { rust: parts[1] == "rust", header, collected: parts[3] == "collected" },
        );
    }
    match args[5].as_str() {
        "all" => {
            body_pairs(&mut game, native, &seed, &records)?;
            refusals(&game, &seed)?;
            chain_pairs(&args)?;
            println!(
                "tank_body_pairs={BODY_CASES} preflight_controls={REFUSAL_CASES} scheduler_pairs=10 presentation_completed=false"
            );
            Ok(())
        }
        "bodies" => body_pairs(&mut game, native, &seed, &records),
        "refusals" => refusals(&game, &seed),
        mode @ ("super-service-refuse"
        | "power-service-refuse"
        | "super-wrong-ram"
        | "power-wrong-ram"
        | "super-after-request-error"
        | "power-after-request-error") => fatal(&mut game, native, &seed, mode),
        _ => Err("unknown tank control mode".into()),
    }
}
