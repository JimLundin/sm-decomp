//! Actual native C/Rust Rio controls. Run only with an authenticated prepared
//! library; retained success covers these synthetic terrain fixtures, not live.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_native_host::Game;
use sm_rio::Entry;
use std::{
    ffi::c_void,
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, String>;
type Seed = unsafe extern "C" fn(*const u8);
type Begin = unsafe extern "C" fn();
type Invoke = unsafe extern "C" fn(u32, u16, u32) -> u16;
type Admit = unsafe extern "C" fn(u32, u16, u32) -> i32;
type Callback = unsafe extern "C" fn(*mut c_void, u32, u16, u32, *mut u16) -> i32;
type Set = unsafe extern "C" fn(Option<Callback>, *mut c_void);
type Bind = unsafe extern "C" fn(*const u8, usize, u64) -> i32;
type Commit = Bind;
type Service = unsafe extern "C" fn(*mut u8, usize, u32, u16, i32, *mut u16, u64) -> i32;
struct Fault {
    mode: String,
    ram: Vec<u8>,
    bind: Bind,
    commit: Commit,
    service: Service,
    generation: unsafe extern "C" fn() -> u64,
    invoke: Invoke,
    sibling: unsafe extern "C" fn(u32),
    horizontal: unsafe extern "C" fn(u16, i32) -> u8,
}
const FATALS: &[(&str, &str)] = &[
    ("wrong-buffer", "Rio poisoned invocation"),
    ("short-buffer", "Rio poisoned invocation"),
    ("wrong-generation", "Rio poisoned invocation"),
    ("null-result", "Rio poisoned invocation"),
    ("wrong-selector", "Rio poisoned invocation"),
    ("wrong-order", "Rio poisoned invocation"),
    ("wrong-slot", "Rio poisoned invocation"),
    ("wrong-amount", "Rio poisoned invocation"),
    ("early-commit", "Rio poisoned invocation"),
    ("duplicate-bind", "Rio poisoned invocation"),
    ("recursive-root", "unsafe Rio fixture rejected"),
    ("sibling-root", "recursive or cross-consumer Waver root"),
    ("uncommitted-success", "Rio incomplete committed invocation"),
    ("postservice-refusal", "Rust Rio terminal failure"),
    ("terminal-error", "Rust Rio terminal failure"),
    ("unissued-service", "recursive or unissued Rio native service"),
    ("duplicate-commit", "Rio poisoned invocation"),
    ("commit-wrong-buffer", "Rio poisoned invocation"),
    ("commit-wrong-generation", "Rio poisoned invocation"),
];
// Deliberately malformed requests use valid owned allocations; no control
// dereferences an invalid pointer. C must reject before publishing scratch.
unsafe extern "C" fn fault_callback(context: *mut c_void, _: u32, _: u16, _: u32, _: *mut u16) -> i32 {
    let fault = unsafe { &mut *context.cast::<Fault>() };
    let token = unsafe { (fault.generation)() };
    let ptr = fault.ram.as_mut_ptr();
    let len = fault.ram.len();
    if unsafe { (fault.bind)(ptr, len, token) } != 0 {
        return -1;
    }
    let mut result = 0;
    let mut alternate = fault.ram.clone();
    match fault.mode.as_str() {
        "early-commit" => {
            unsafe { (fault.commit)(ptr, len, token) };
        }
        "duplicate-bind" => {
            unsafe { (fault.bind)(ptr, len, token) };
        }
        "recursive-root" => {
            unsafe { (fault.invoke)(Entry::State3 as u32, 0x40, 0) };
        }
        "sibling-root" => {
            unsafe { (fault.sibling)(0) };
        }
        "unissued-service" => {
            unsafe { (fault.horizontal)(0x40, 0x20000) };
        }
        "duplicate-commit" | "commit-wrong-buffer" | "commit-wrong-generation" => {
            // Init has no services, so the first commit is valid. These controls
            // isolate the allocation, token and duplicate publication guards.
            if fault.mode == "duplicate-commit" {
                unsafe { (fault.commit)(ptr, len, token) };
            }
            unsafe {
                (fault.commit)(
                    if fault.mode == "commit-wrong-buffer" { alternate.as_ptr() } else { ptr },
                    len,
                    token.wrapping_add(u64::from(fault.mode == "commit-wrong-generation")),
                )
            };
        }
        "uncommitted-success" => return 1,
        "terminal-error" => return -1,
        mode => {
            unsafe {
                (fault.service)(
                    if mode == "wrong-buffer" { alternate.as_mut_ptr() } else { ptr },
                    len - usize::from(mode == "short-buffer"),
                    if mode == "wrong-selector" { 3 } else { u32::from(mode == "wrong-order") },
                    if mode == "wrong-slot" { 0x80 } else { 0x40 },
                    0x20000 + i32::from(mode == "wrong-amount"),
                    if mode == "null-result" { std::ptr::null_mut() } else { &mut result },
                    token.wrapping_add(u64::from(mode == "wrong-generation")),
                )
            };
        }
    }
    0
}
fn fatal(game: &mut Game, library: &Library, mode: &str, root: &Path) -> Result<()> {
    let entry = if mode.starts_with("commit-") || mode == "duplicate-commit" { Entry::Init } else { Entry::State3 };
    let (ram, _, _) = fixture(entry, false);
    write(root, "input.wram", &ram)?;
    // SAFETY: authenticated library, exclusive Game and stable boxed callback.
    unsafe {
        let seed: Seed = *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?;
        let set: Set = *library.get(b"sm_native_set_rio_hook\0").map_err(|e| e.to_string())?;
        let mut fault = Box::new(Fault {
            mode: mode.into(),
            ram,
            bind: *library.get(b"sm_native_rio_bind\0").map_err(|e| e.to_string())?,
            commit: *library.get(b"sm_native_rio_commit\0").map_err(|e| e.to_string())?,
            service: *library.get(b"sm_native_rio_service\0").map_err(|e| e.to_string())?,
            generation: *library.get(b"sm_native_rio_generation\0").map_err(|e| e.to_string())?,
            invoke: *library.get(b"sm_native_invoke_rio\0").map_err(|e| e.to_string())?,
            sibling: *library.get(b"sm_native_waver_ai\0").map_err(|e| e.to_string())?,
            horizontal: *library.get(b"Enemy_MoveRight_IgnoreSlopes\0").map_err(|e| e.to_string())?,
        });
        game.clear_rio();
        seed(fault.ram.as_ptr());
        set(Some(fault_callback), (&mut *fault as *mut Fault).cast());
        (fault.invoke)(entry as u32, 0x40, 0);
        set(None, std::ptr::null_mut());
    }
    Err(format!("fatal {mode} unexpectedly returned"))
}
fn fatals(args: &[std::ffi::OsString], root: &Path) -> Result<usize> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    for &(mode, expected) in FATALS {
        let directory = root.join(format!("process-{mode}"));
        let child_output = directory.join("case");
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let mut child = Command::new(&exe)
            .args(&args[..2])
            .arg(&child_output)
            .arg(format!("fatal-{mode}"))
            .stdout(Stdio::from(fs::File::create(directory.join("stdout.txt")).map_err(|e| e.to_string())?))
            .stderr(Stdio::from(fs::File::create(directory.join("stderr.txt")).map_err(|e| e.to_string())?))
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
                outcome => {
                    let _ = child.kill();
                    let status = child.wait().map_err(|e| e.to_string())?;
                    if let Err(e) = outcome {
                        return Err(e.to_string());
                    }
                    timed_out = true;
                    break status;
                }
            }
        };
        let stderr = fs::read(directory.join("stderr.txt")).map_err(|e| e.to_string())?;
        let stdout = fs::read(directory.join("stdout.txt")).map_err(|e| e.to_string())?;
        let verified = !timed_out && status.code() == Some(2) && String::from_utf8_lossy(&stderr).contains(expected);
        write(&directory, "process.txt", format!("verified={verified} exit_code={:?} timed_out={timed_out} elapsed_ms={} stderr_sha256={} stdout_sha256={} expected={expected}\n", status.code(), started.elapsed().as_millis(), hash(&stderr), hash(&stdout)).as_bytes())?;
        if !verified {
            return Err(format!("fatal {mode} failed: {}", directory.display()));
        }
    }
    Ok(FATALS.len())
}
fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn write(root: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    fs::write(root.join(name), bytes).map_err(|e| e.to_string())
}
fn compare(name: &str, left: &[u8], right: &[u8]) -> Result<()> {
    if left == right {
        return Ok(());
    }
    let first = left.iter().zip(right).position(|(a, b)| a != b).unwrap_or(left.len().min(right.len()));
    Err(format!("{name} mismatch at {first:#x}"))
}
fn fixture(entry: Entry, wall: bool) -> (Vec<u8>, u16, u32) {
    let mut ram = vec![0; 0x20000];
    let k = 0x40;
    let base = 0xf78 + usize::from(k);
    for (at, value) in [
        (0xe54, k),
        (0x7a5, 16),
        (0x7a7, 16),
        (0xaf6, 144),
        (0xafa, 144),
        (0x998, 8),
        (0x5e5, 0x1234),
        (base + 2, 128),
        (base + 6, 128),
        (base + 10, 8),
        (base + 12, 8),
        (base + 0x34, 0x100),
        (base + 0x36, 0x200),
        (base + 0x38, 1),
    ] {
        put(&mut ram, at, value);
    }
    let state = match entry {
        Entry::State1 => 0xbbed,
        Entry::State2 => 0xbc32,
        Entry::State4 => 0xbcb7,
        Entry::State5 => 0xbcff,
        _ => 0xbc48,
    };
    put(&mut ram, base + 0x32, state);
    if wall {
        // A vertical solid wall immediately beside the actor. Terrain is real
        // native block collision; scripts are not used by this control.
        for row in 0..16 {
            put(&mut ram, 0x10002 + 2 * (row * 16 + 8), 0x8000);
        }
    }
    // Sentinel outside all declared actor/terrain effects.
    ram[0x1ffff] = 0xab;
    let arg = match entry {
        Entry::Instruction => 0xbb7f,
        Entry::SelectAnimation => 0xbb97,
        Entry::CallRioFunc => 0xa2bc48,
        _ => 0,
    };
    (ram, k, arg)
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(3..=4).contains(&args.len()) {
        return Err("usage: rio-host-control PREPARED_LIBRARY ROM NEW_OUTPUT [all|fatal-NAME]".into());
    }
    let mode = args.get(3).map(|s| s.to_str().ok_or("mode must be UTF-8")).transpose()?.unwrap_or("all");
    if mode != "all" && !FATALS.iter().any(|(name, _)| mode == format!("fatal-{name}")) {
        return Err("unknown Rio control mode".into());
    }
    let library_path = Path::new(&args[0]);
    let root = Path::new(&args[2]);
    sm_native_host::authenticate_library(library_path)?;
    fs::create_dir(root).map_err(|e| e.to_string())?;
    write(root, "status.txt", b"running; no passing verdict\n")?;
    write(root, "runner.rs", include_bytes!("rio-host-control.rs"))?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    write(
        root,
        "subject.txt",
        format!(
            "library_sha256={} executable_sha256={} runner_sha256={}\n",
            hash(&fs::read(library_path).map_err(|e| e.to_string())?),
            hash(&fs::read(executable).map_err(|e| e.to_string())?),
            hash(include_bytes!("rio-host-control.rs"))
        )
        .as_bytes(),
    )?;
    let mut game = Game::boot(library_path, Path::new(&args[1]))?;
    // SAFETY: Game authenticated this same process-lifetime native library.
    let library = unsafe { Library::new(library_path) }.map_err(|e| e.to_string())?;
    if let Some(mode) = mode.strip_prefix("fatal-") {
        return fatal(&mut game, &library, mode, root);
    }
    let (seed, begin, invoke, admit): (Seed, Begin, Invoke, Admit) = unsafe {
        (
            *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
            *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
            *library.get(b"sm_native_invoke_rio\0").map_err(|e| e.to_string())?,
            *library.get(b"sm_native_rio_fixture_admitted\0").map_err(|e| e.to_string())?,
        )
    };
    let mut pairs = 0;
    for entry in Entry::ALL {
        for terrain in
            ["air", "wall", "horizontal-extension", "vertical-extension", "spike-allocate", "spike-full-pool"]
        {
            // Terrain controls require movement; the first two still cover all roots.
            if terrain != "air" && terrain != "wall" && entry != Entry::State3 {
                continue;
            }
            let directory = root.join(format!("{entry:?}-{terrain}"));
            fs::create_dir(&directory).map_err(|e| e.to_string())?;
            let (mut ram, k, arg) = fixture(entry, terrain == "wall");
            if terrain != "air" && terrain != "wall" {
                let index = 7 * 16 + 8;
                let (kind, bts) = match terrain {
                    "horizontal-extension" => (0x5000, 1),
                    "vertical-extension" => (0xd000, 1),
                    _ => (0xa000, 15),
                };
                put(&mut ram, 0x10002 + 2 * index, kind);
                ram[0x16402 + index] = bts;
                if terrain.ends_with("extension") {
                    put(&mut ram, 0x10002 + 2 * (index + if kind == 0x5000 { 1 } else { 16 }), 0x8000);
                }
                if terrain == "spike-full-pool" {
                    for offset in (0..80).step_by(2) {
                        put(&mut ram, 0x1c37 + offset, 0xd094);
                    }
                }
            }
            write(&directory, "input.wram", &ram)?;
            let mut results = vec![];
            for rust in [false, true] {
                if rust {
                    game.install_rio();
                } else {
                    game.clear_rio();
                }
                // SAFETY: exclusive Game, full WRAM, no active callbacks.
                unsafe {
                    seed(ram.as_ptr());
                    begin();
                }
                if unsafe { admit(entry as u32, k, arg) } != 1 {
                    return Err(format!("fixture refused: {entry:?}"));
                }
                let before = game.rio_counters();
                let sram = game.sram().to_vec();
                let output = unsafe { invoke(entry as u32, k, arg) };
                let after = game.rio_counters();
                let index = entry as usize;
                let mut expected = before;
                expected[index] += 1;
                if rust {
                    expected[20 + index] += 1;
                    expected[40 + index] += 1;
                } else {
                    expected[10 + index] += 1;
                }
                let horizontal =
                    matches!(entry, Entry::Main | Entry::State3 | Entry::State4 | Entry::State5 | Entry::CallRioFunc);
                let short_circuit = matches!(entry, Entry::Main | Entry::State3 | Entry::CallRioFunc)
                    && (terrain == "wall" || terrain.ends_with("extension"));
                let services =
                    [u64::from(horizontal), u64::from(horizontal && !short_circuit), u64::from(entry == Entry::State5)];
                for (selector, count) in services.into_iter().enumerate() {
                    expected[60 + selector] += count;
                    expected[63 + selector] += count;
                }
                if after != expected {
                    return Err(format!("complete ownership/service counter delta differs: {entry:?} {terrain}"));
                }
                compare("SRAM sentinel", &sram, game.sram())?;
                if game.wram()[0x1ffff] != 0xab {
                    return Err("unrelated WRAM sentinel changed".into());
                }
                if terrain.starts_with("spike-") {
                    let events = game
                        .effects()
                        .as_chunks::<16>()
                        .0
                        .iter()
                        .filter(|e| u32::from_le_bytes(e[..4].try_into().unwrap()) == 3)
                        .count();
                    if events == 0 {
                        return Err("spike fixture never reached native allocation".into());
                    }
                    let final_tile =
                        u16::from_le_bytes(game.wram()[0x10002 + 2 * (7 * 16 + 8)..][..2].try_into().unwrap());
                    let expected = if terrain == "spike-allocate" { 0 } else { 0xa000 };
                    if final_tile != expected {
                        return Err("spike allocation/pool postcondition differs".into());
                    }
                    if u16::from_le_bytes(game.wram()[0x1c37 + 78..][..2].try_into().unwrap()) != 0xd094 {
                        return Err("spike PLM slot postcondition differs".into());
                    }
                }
                if terrain.ends_with("extension") {
                    let state = u16::from_le_bytes(game.wram()[0xf78 + 0x40 + 0x32..][..2].try_into().unwrap());
                    if state != 0xbcb7 {
                        return Err("extension fixture did not collide and enter State4".into());
                    }
                }
                let label = if rust { "rust" } else { "c" };
                write(&directory, &format!("{label}.wram"), game.wram())?;
                write(&directory, &format!("{label}.effects"), game.effects())?;
                let counters = after.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>();
                write(&directory, &format!("{label}.counters"), &counters)?;
                results.push((output, game.wram().to_vec(), game.effects().to_vec()));
            }
            if results[0].0 != results[1].0 {
                return Err("typed output mismatch".into());
            }
            compare("full WRAM", &results[0].1, &results[1].1)?;
            compare("ordered effects", &results[0].2, &results[1].2)?;
            write(
                &directory,
                "pair.txt",
                format!(
                    "passed output={} wram_sha256={} effects_sha256={}\n",
                    results[0].0,
                    hash(&results[0].1),
                    hash(&results[0].2)
                )
                .as_bytes(),
            )?;
            pairs += 1;
        }
    }
    game.clear_rio();
    let mut admissions = 0;
    for fault in 0..12 {
        let (mut ram, mut k, mut arg) = fixture(Entry::State3, false);
        let entry = match fault {
            0 => {
                k = 1;
                Entry::State3
            }
            1 => {
                put(&mut ram, 0xe54, 1);
                Entry::State3
            }
            2 => {
                put(&mut ram, 0x7a5, 0);
                Entry::State3
            }
            3 => {
                put(&mut ram, 0xf78 + 0x40 + 10, 0);
                Entry::State3
            }
            4 => {
                arg = 0xa3bc48;
                Entry::CallRioFunc
            }
            5 => {
                arg = 0x7fff;
                Entry::Instruction
            }
            6 => {
                put(&mut ram, 0xf78 + 0x40 + 0x32, 0x1234);
                Entry::Main
            }
            7..=10 => {
                let index = 7 * 16 + 8;
                put(&mut ram, 0x10002 + 2 * index, if fault == 9 { 0xa000 } else { 0xd000 });
                ram[0x16402 + index] = match fault {
                    7 => 1,
                    8 => 128,
                    9 => 16,
                    _ => 1,
                };
                if fault == 7 || fault == 10 {
                    put(&mut ram, 0x10002 + 2 * (index + 16), 0xd000);
                    ram[0x16402 + index + 16] = if fault == 7 { 255 } else { 1 };
                }
                Entry::State3
            }
            11 => {
                // Cached explicit actor is valid, but native slope helpers read
                // a different current actor whose geometry is outside the domain.
                put(&mut ram, 0xe54, 0x80);
                put(&mut ram, 0xf78 + 0x80 + 2, 1);
                Entry::State3
            }
            _ => unreachable!(),
        };
        unsafe {
            seed(ram.as_ptr());
            begin();
        }
        let before = game.wram().to_vec();
        let counters = game.rio_counters();
        if unsafe { admit(entry as u32, k, arg) } != 0 {
            return Err(format!("invalid admission {fault} accepted"));
        }
        compare("admission changed WRAM", &before, game.wram())?;
        if counters != game.rio_counters() {
            return Err("admission changed counters".into());
        }
        write(root, &format!("admission-{fault}.wram"), &before)?;
        admissions += 1;
    }
    // Valid native air movement can exceed the deliberately narrow Rust domain.
    // Use ordinary AI dispatch to prove installed refusal leaves the C fallback
    // equivalent; malformed diagnostic admissions above are never invoked.
    let ai: unsafe extern "C" fn(u32) = unsafe { *library.get(b"CallEnemyAi\0").map_err(|e| e.to_string())? };
    let mut refusals = 0;
    for field in [0x34, 0x36] {
        let (mut ram, _, _) = fixture(Entry::Main, false);
        put(&mut ram, 0xf78 + 0x40 + field, 4097);
        let mut results = vec![];
        for rust in [false, true] {
            if rust {
                game.install_rio();
            } else {
                game.clear_rio();
            }
            unsafe {
                seed(ram.as_ptr());
                begin();
            }
            if unsafe { admit(Entry::Main as u32, 0, 0) } != 0 {
                return Err("refusal fixture entered Rust domain".into());
            }
            let before = game.rio_counters();
            let sram = game.sram().to_vec();
            unsafe {
                ai(0xa2bbe3);
            }
            let after = game.rio_counters();
            let mut expected = before;
            expected[1] += 1;
            expected[11] += 1;
            expected[51] += 1;
            if rust {
                expected[21] += 1;
                expected[31] += 1;
            }
            for selector in 0..2 {
                expected[60 + selector] += 1;
                expected[63 + selector] += 1;
            }
            if expected != after {
                return Err("runtime refusal counter delta differs".into());
            }
            compare("refusal SRAM", &sram, game.sram())?;
            let label = if rust { "rust" } else { "c" };
            write(root, &format!("refusal-{field:x}-{label}.wram"), game.wram())?;
            write(root, &format!("refusal-{field:x}-{label}.effects"), game.effects())?;
            results.push((game.wram().to_vec(), game.effects().to_vec()));
        }
        compare("runtime refusal full WRAM", &results[0].0, &results[1].0)?;
        compare("runtime refusal ordered effects", &results[0].1, &results[1].1)?;
        refusals += 1;
    }
    // Fatal children do not spawn descendants. The outer caller still supervises
    // this process and the native library, including any unexpected descendants.
    drop(library);
    drop(game);
    let fatal_count = fatals(&args, root)?;
    write(root,"verdict.txt",format!("passed pairs={pairs} admissions={admissions} runtime_refusal_pairs={refusals} fatal_controls={fatal_count} full_wram=true ordered_effects=true real_native_services=true live=false original_rom_parity=false\n").as_bytes())?;
    write(root, "status.txt", b"passed\n")?;
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Rio native control failed: {e}");
        std::process::exit(1);
    }
}
