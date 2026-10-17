//! Independent original-entry HandleProjectile fixtures. The 801-byte original
//! projection is fixed before execution; full C/Rust equality is a separate test.
use crate::fixture::{authenticate_capture, require, retain, text};
use crate::handle_calls::Call;
use crate::{Result, Table};
use sm_native_host::Game;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

const WRAM: usize = 0x20000;
const CALLS: usize = 34;
const WORDS: [usize; 7] = [0xb5e, 0xccc, 0xcce, 0xdc0, 0xdc4, 0xdde, 0x18ac];
const PROJECTION: &str = "schema = \"handle-projectile-semantic-projection-v1\"\nbytes = 801\nprojectile_range = [2916, 3276]\nsfx_range = [1603, 1670]\nwords = [2910, 3276, 3278, 3520, 3524, 3550, 6316]\ntrail_range = [54872, 55232]\nranges_are_half_open = true\nword_bytes = 2\norder = \"projectile range, listed words, SFX range, ten 18-slot trail arrays; increasing address within each selection\"\nsource = \"Pinned sm_90.c HandleProjectile and callees, sm_93.c projectile instructions, sm_9b.c trail placement, sm_80.c sound queues, variables.h\"\nexclusions = \"CPU registers/stack/DP, removed collision scratch, math MMIO, request ordering/audio and inter-call evolution; complete raw outputs retained\"\n";

fn addresses() -> impl Iterator<Item = usize> {
    (0xb64..0xccc).chain(WORDS.into_iter().flat_map(|at| at..at + 2)).chain(0x643..0x686).chain(0xd658..0xd7c0)
}
fn projection(ram: &[u8]) -> Vec<u8> {
    addresses().map(|at| ram[at]).collect()
}
fn difference(left: &[u8], right: &[u8], selected: bool) -> toml::Value {
    let at = if selected {
        addresses().find(|&at| left[at] != right[at])
    } else {
        (0..WRAM).find(|&at| left[at] != right[at])
    };
    let mut result = Table::new();
    result.insert("equal".into(), at.is_none().into());
    if let Some(at) = at {
        result.insert("wram_offset".into(), (at as i64).into());
        result.insert("left".into(), i64::from(left[at]).into());
        result.insert("right".into(), i64::from(right[at]).into());
    }
    result.into()
}

// Retain the source bytes actually compiled into this command. Native C and
// instrumentation are independently bound by the authenticated library manifest.
fn sources() -> Vec<(&'static str, &'static [u8])> {
    macro_rules! source {
        ($name:literal, $path:literal) => {
            ($name, include_bytes!($path).as_slice())
        };
    }
    vec![
        source!("Cargo.toml", "../../Cargo.toml"),
        source!("Cargo.lock", "../../Cargo.lock"),
        source!("original/Cargo.toml", "../Cargo.toml"),
        source!("original/src/main.rs", "main.rs"),
        source!("original/src/fixture.rs", "fixture.rs"),
        source!("original/src/handle_calls.rs", "handle_calls.rs"),
        source!("original/src/handle_fixture.rs", "handle_fixture.rs"),
        source!("original/src/validate.rs", "validate.rs"),
        source!("original/src/movie.rs", "movie.rs"),
        source!("original/src/observer.rs", "observer.rs"),
        source!("original/src/observer.lua", "observer.lua"),
        source!("projectile/Cargo.toml", "../../projectile/Cargo.toml"),
        source!("projectile/src/lib.rs", "../../projectile/src/lib.rs"),
        source!("projectile/src/audio.rs", "../../projectile/src/audio.rs"),
        source!("projectile/src/collision.rs", "../../projectile/src/collision.rs"),
        source!("projectile/src/core.rs", "../../projectile/src/core.rs"),
        source!("projectile/src/draw.rs", "../../projectile/src/draw.rs"),
        source!("projectile/src/memory.rs", "../../projectile/src/memory.rs"),
        source!("projectile/src/trails.rs", "../../projectile/src/trails.rs"),
        source!("native-host/Cargo.toml", "../../native-host/Cargo.toml"),
        source!("native-host/src/lib.rs", "../../native-host/src/lib.rs"),
        source!("native-host/src/hooks.rs", "../../native-host/src/hooks.rs"),
        source!("native-host/src/transaction.rs", "../../native-host/src/transaction.rs"),
        source!("native-host/src/prepare.rs", "../../native-host/src/prepare.rs"),
    ]
}
fn source_identity(out: &Path) -> Result<toml::Value> {
    let mut hashes = BTreeMap::new();
    for (name, bytes) in sources() {
        let destination = out.join("source").join(name);
        fs::create_dir_all(destination.parent().unwrap()).map_err(|e| e.to_string())?;
        crate::new_file(&destination, bytes)?;
        crate::seal(&destination)?;
        hashes.insert(name.to_owned(), crate::hash_bytes(bytes));
    }
    Ok(crate::file_map(hashes))
}
fn check_current_source() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().ok_or("original crate has no parent")?;
    for (name, bytes) in sources() {
        require(
            crate::hash(&root.join(name))? == crate::hash_bytes(bytes),
            &format!("compiled/current source differs: {name}"),
        )?;
    }
    Ok(())
}
fn artifacts(out: &Path) -> Result<toml::Value> {
    fn walk(root: &Path, directory: &Path, hashes: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path == root.join("result.toml") {
                continue;
            }
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                walk(root, &path, hashes)?;
            } else {
                require(kind.is_file(), "unexpected batch artifact type")?;
                hashes.insert(path.strip_prefix(root).unwrap().to_string_lossy().into_owned(), crate::hash(&path)?);
                crate::seal(&path)?;
            }
        }
        Ok(())
    }
    let mut hashes = BTreeMap::new();
    walk(out, out, &mut hashes)?;
    Ok(crate::file_map(hashes))
}

#[derive(Default)]
struct NoPlm {
    requests: u64,
}
impl sm_projectile::Host for NoPlm {
    fn spawn_plm(&mut self, _: &mut [u8], _: u16) -> Result<u8> {
        self.requests += 1;
        Err("isolated HandleProjectile fixture forbids a PLM dependency".into())
    }
}

struct Native {
    _library: libloading::Library,
    commit: unsafe extern "C" fn(*const u8),
    begin: unsafe extern "C" fn(),
    handle: unsafe extern "C" fn(),
}
impl Native {
    fn load(path: &Path) -> Result<Self> {
        // SAFETY: caller already booted one Game with this authenticated canonical
        // path. This second handle pins the same DSO for these function pointers.
        unsafe {
            let library = libloading::Library::new(path).map_err(|e| e.to_string())?;
            let commit = *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?;
            let begin = *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?;
            let handle = *library.get(b"HandleProjectile\0").map_err(|e| e.to_string())?;
            Ok(Self { _library: library, commit, begin, handle })
        }
    }
}
fn call_metadata(call: &Call) -> Table {
    let mut result = Table::new();
    for (key, value) in [
        ("original_call", call.id),
        ("original_frame", call.frame),
        ("original_entry_event", call.entry_event),
        ("original_exit_event", call.exit_event),
        ("original_return_event", call.return_event),
        ("slot", call.slot as u64),
        ("original_hardware_events", call.hardware_events),
        ("original_sound_requests", call.sound_requests),
    ] {
        result.insert(key.into(), (value as i64).into());
    }
    result
}
fn execute_call(game: &Game, native: &Native, call: &Call, out: &Path, fault: bool) -> Result<Table> {
    fs::create_dir(out).map_err(|e| e.to_string())?;
    require(call.input.len() == WRAM && call.original.len() == WRAM, "original WRAM extent differs")?;
    for (name, bytes) in [
        ("original-entry.wram", call.input.as_slice()),
        ("original-return.wram", call.original.as_slice()),
        ("original-entry.cpu.toml", call.entry_cpu.as_slice()),
        ("original-return.cpu.toml", call.return_cpu.as_slice()),
        ("original-handle-events.csv", call.ledger.as_slice()),
    ] {
        retain(out, name, bytes)?;
    }
    let mut result = call_metadata(call);
    result.insert("schema".into(), "sm-original-handle-call-v1".into());
    let mut rust = call.input.clone();
    let mut host = NoPlm::default();
    let outcome = sm_projectile::run(sm_projectile::Entry::HandleProjectile, &mut rust, game.rom(), &mut host);
    // Retain a failed guard's state as well; unchecked C must never run for it.
    retain(out, "rust-before-fault.wram", &rust)?;
    require(
        outcome? == sm_projectile::Outcome::Handled && host.requests == 0,
        "Rust refused or requested PLM before C",
    )?;
    if fault {
        rust[0xb64 + 2 * call.slot] ^= 1;
    }
    retain(out, "rust.wram", &rust)?;
    let before = game.projectile_counters();
    let before_state = game.state();
    // SAFETY: select validated ordinary CPU/WRAM eligibility and the safe Rust
    // call above handled this entire 128KiB input without a foreign dependency.
    // Reset from THIS original entry, never from a predecessor's result. No
    // Game-backed slice is live across writes; no hook or host step is invoked.
    unsafe {
        (native.commit)(call.input.as_ptr());
        (native.begin)();
        (native.handle)();
    }
    let actual_c = game.wram().to_vec();
    let effects = game.effects().to_vec();
    let counts = game.projectile_counters();
    let after_state = game.state();
    retain(out, "actual-c.wram", &actual_c)?;
    retain(out, "actual-c.effects", &effects)?;
    require(effects.len().is_multiple_of(16), "native effect extent differs")?;
    let c_plm = effects
        .as_chunks::<16>()
        .0
        .iter()
        .filter(|event| u32::from_le_bytes(event[..4].try_into().unwrap()) == 3)
        .count();
    require(c_plm == 0 && counts.foreign_calls == before.foreign_calls, "actual C invoked PLM")?;
    require(
        (0..4).all(|entry| counts.c_entries[entry] == before.c_entries[entry] + u64::from(entry == 1))
            && counts.rust_entries == before.rust_entries
            && counts.rust_completed == before.rust_completed
            && counts.rust_refused == before.rust_refused
            && counts.ceres_high_bit_entries == before.ceres_high_bit_entries
            && after_state.host_steps == before_state.host_steps
            && after_state.polls == before_state.polls
            && after_state.updates == before_state.updates
            && after_state.waits == before_state.waits
            && after_state.c_gameplay_calls == before_state.c_gameplay_calls
            && after_state.cpu_fallback_calls == before_state.cpu_fallback_calls
            && after_state.cpu_instructions == before_state.cpu_instructions
            && after_state.nmis == before_state.nmis,
        "actual C did not execute exactly one isolated HandleProjectile invocation",
    )?;
    for (name, ram) in
        [("original.projection", &call.original), ("actual-c.projection", &actual_c), ("rust.projection", &rust)]
    {
        retain(out, name, &projection(ram))?;
    }
    let passed = actual_c == rust
        && projection(&actual_c) == projection(&call.original)
        && projection(&rust) == projection(&call.original);
    result.insert("status".into(), if passed { "passed" } else { "mismatch" }.into());
    result.insert("actual_c_rust_full_wram".into(), difference(&actual_c, &rust, false));
    result.insert("actual_c_original_projection".into(), difference(&actual_c, &call.original, true));
    result.insert("rust_original_projection".into(), difference(&rust, &call.original, true));
    result.insert("actual_c_handle_invocations".into(), 1.into());
    result.insert("actual_c_plm_calls".into(), (c_plm as i64).into());
    result.insert("rust_plm_requests".into(), (host.requests as i64).into());
    result.insert("projection_bytes".into(), 801.into());
    result.insert("deliberate_fault".into(), fault.into());
    result.insert("artifacts".into(), artifacts(out)?);
    crate::table_file(&out.join("result.toml"), &result)?;
    Ok(result)
}

/// Run inside the existing 30-second comparison child. One Game for the batch.
pub fn compare(capture: &Path, library: &Path, rom: &Path, out: &Path, fault: bool) -> Result<()> {
    let start = Instant::now();
    fs::create_dir(out).map_err(|e| format!("batch output must be fresh: {e}"))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;
    let mut result = Table::new();
    result.insert("schema".into(), "sm-original-handle-batch-v1".into());
    result.insert("expected_calls".into(), (CALLS as i64).into());
    result.insert("completed_calls".into(), 0.into());
    result.insert("passed_calls".into(), 0.into());
    result.insert("projection_bytes".into(), 801.into());
    result.insert("deliberate_fault".into(), fault.into());
    let outcome = execute(capture, library, rom, &out, fault, &mut result);
    let status = if outcome.is_ok() {
        "passed"
    } else if result.get("mismatch_call").is_some() {
        "mismatch"
    } else {
        "failed"
    };
    result.insert("status".into(), status.into());
    if let Err(error) = &outcome {
        result.insert("error".into(), error.clone().into());
    }
    result.insert("elapsed_seconds".into(), start.elapsed().as_secs_f64().into());
    result.insert(
        "original_equivalence".into(),
        if outcome.is_ok() {
            "34 independent original-entry HandleProjectile calls; fixed 801-byte semantic output projection"
        } else {
            "not-established"
        }
        .into(),
    );
    for (key, value) in [
        ("hardware_effect_equivalence", "not-tested; original math-MMIO and CPU context differ in abstraction"),
        (
            "sound_effect_scope",
            "original request markers retained; sound queue storage projected; native request ordering/audio not compared",
        ),
        ("full_original_wram_equivalence", "not-tested"),
        ("original_live_replay_equivalence", "not-tested"),
        ("inter_call_evolution", "not-tested; every call resets from its own original entry"),
    ] {
        result.insert(key.into(), value.into());
    }
    if out.join("request.toml").exists() {
        result.insert("request_sha256".into(), crate::hash(&out.join("request.toml"))?.into());
    }
    result.insert("artifacts".into(), artifacts(&out)?);
    crate::table_file(&out.join("result.toml"), &result)?;
    outcome
}

fn execute(capture: &Path, library: &Path, rom: &Path, out: &Path, fault: bool, result: &mut Table) -> Result<()> {
    retain(out, "projection.toml", PROJECTION.as_bytes())?;
    let source_map = source_identity(out)?;
    check_current_source()?;
    let capture = capture.canonicalize().map_err(|e| e.to_string())?;
    let proof = authenticate_capture(&capture)?;
    require(text(&proof, "capture_status")? == "captured", "Handle batch requires a complete original capture")?;
    let calls = crate::handle_calls::select(&capture)?;
    require(calls.len() == CALLS, "Handle batch requires exactly 34 calls")?;
    let library = library.canonicalize().map_err(|e| e.to_string())?;
    let rom = rom.canonicalize().map_err(|e| e.to_string())?;
    let manifest = library.parent().ok_or("native library has no parent")?.join("library.sme");
    // boot authenticates native sources/library and pristine ROM once. No second
    // initial native rescan and no authentication in the per-call loop.
    let game = Game::boot(&library, &rom)?;
    let native = Native::load(&library)?;
    let library_hash = crate::hash(&library)?;
    let binary = std::env::current_exe().map_err(|e| e.to_string())?;
    retain(out, "fixture-command", &fs::read(&binary).map_err(|e| e.to_string())?)?;
    for (name, path) in [
        ("original-result.toml", capture.join("result.toml")),
        ("original-request.toml", capture.join("request.toml")),
        ("native-library.sme", manifest.clone()),
    ] {
        retain(out, name, &fs::read(path).map_err(|e| e.to_string())?)?;
    }
    let mut request = Table::new();
    for (key, value) in [
        ("schema", "sm-original-handle-batch-request-v1".to_owned()),
        ("capture", capture.to_string_lossy().into_owned()),
        ("capture_result_sha256", crate::hash(&capture.join("result.toml"))?),
        ("capture_request_sha256", crate::hash(&capture.join("request.toml"))?),
        ("library", library.to_string_lossy().into_owned()),
        ("library_sha256", library_hash.clone()),
        ("library_manifest_sha256", crate::hash(&manifest)?),
        ("rom_sha256", crate::ROM_SHA256.to_owned()),
        ("command", binary.to_string_lossy().into_owned()),
        ("command_sha256", crate::hash(&binary)?),
        ("projection_sha256", crate::hash_bytes(PROJECTION.as_bytes())),
        ("native_rom_view_sha256", crate::hash_bytes(game.rom())),
    ] {
        request.insert(key.into(), value.into());
    }
    request.insert("sources".into(), source_map);
    request.insert("original_capture".into(), proof.clone().into());
    request.insert("deliberate_fault".into(), fault.into());
    request.insert(
        "selection".into(),
        calls.iter().map(|call| toml::Value::Table(call_metadata(call))).collect::<Vec<_>>().into(),
    );
    crate::table_file(&out.join("request.toml"), &request)?;
    result.insert("original_capture".into(), proof.clone().into());
    let mut reports = Vec::new();
    let mut failure = None;
    for (index, call) in calls.iter().enumerate() {
        let directory = out.join(format!("call-{:03}", call.id));
        let call_result = execute_call(&game, &native, call, &directory, fault && index == 0);
        match call_result {
            Ok(report) => {
                let passed = text(&report, "status")? == "passed";
                result.insert("completed_calls".into(), ((index + 1) as i64).into());
                let mut summary = call_metadata(call);
                summary.insert("status".into(), if passed { "passed" } else { "mismatch" }.into());
                summary.insert("result_sha256".into(), crate::hash(&directory.join("result.toml"))?.into());
                reports.push(toml::Value::Table(summary));
                if !passed {
                    result.insert("mismatch_call".into(), (call.id as i64).into());
                    failure = Some(format!(
                        "Handle call {} mismatch; inspect retained raw outputs and first differences",
                        call.id
                    ));
                    break;
                }
                result.insert("passed_calls".into(), ((index + 1) as i64).into());
            }
            Err(error) => {
                let mut report = call_metadata(call);
                report.insert("schema".into(), "sm-original-handle-call-v1".into());
                report.insert("status".into(), "failed".into());
                report.insert("error".into(), error.clone().into());
                if directory.exists() && !directory.join("result.toml").exists() {
                    report.insert("artifacts".into(), artifacts(&directory)?);
                    crate::table_file(&directory.join("result.toml"), &report)?;
                }
                result.insert("failed_call".into(), (call.id as i64).into());
                failure = Some(error);
                break;
            }
        }
    }
    result.insert("calls".into(), reports.into());
    // Reauthenticate each large dependency once at the batch boundary, including
    // mismatch runs. No preceding call can lend its state to a successor.
    require(
        sm_native_host::authenticate_library(&library)? == library_hash
            && crate::hash(&manifest)? == text(&request, "library_manifest_sha256")?
            && crate::hash(&binary)? == text(&request, "command_sha256")?
            && crate::hash(&rom)? == crate::ROM_SHA256
            && crate::hash_bytes(game.rom()) == text(&request, "native_rom_view_sha256")?,
        "batch dependency changed",
    )?;
    require(
        authenticate_capture(&capture)? == proof
            && crate::hash(&capture.join("result.toml"))? == text(&request, "capture_result_sha256")?,
        "original capture changed during batch",
    )?;
    check_current_source()?;
    result.insert("final_dependencies_authenticated".into(), true.into());
    if let Some(error) = failure {
        return Err(error);
    }
    require(reports_len(result) == CALLS, "incomplete Handle batch")
}
fn reports_len(result: &Table) -> usize {
    result.get("calls").and_then(toml::Value::as_array).map_or(0, Vec::len)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn projection_is_exactly_the_frozen_801_addresses() {
        let all: Vec<_> = addresses().collect();
        assert_eq!(all.len(), 801);
        let unique: std::collections::BTreeSet<_> = all.iter().copied().collect();
        assert_eq!(unique.len(), 801);
        assert_eq!(&all[441..], &(0xd658..0xd7c0).collect::<Vec<_>>());
        let mut changed = vec![0; WRAM];
        let original = changed.clone();
        for at in [0x14, 0x1ff0, 0xdd2, 0xd657, 0xd7c0] {
            changed[at] = 1;
        }
        assert_eq!(projection(&changed), projection(&original));
        for at in [0xb64, 0xccc, 0x18ad, 0x643, 0xd658, 0xd7bf] {
            let mut selected = original.clone();
            selected[at] = 1;
            assert_ne!(projection(&selected), projection(&original));
            assert_eq!(difference(&selected, &original, true)["wram_offset"].as_integer(), Some(at as i64));
        }
    }
    #[test]
    fn original_projection_does_not_hide_full_c_rust_difference() {
        let left = vec![0; WRAM];
        let mut right = left.clone();
        right[0x14] = 7;
        assert_eq!(difference(&left, &right, true)["equal"].as_bool(), Some(true));
        assert_eq!(difference(&left, &right, false)["wram_offset"].as_integer(), Some(0x14));
        right[0xd7bf] = 3;
        assert_eq!(difference(&left, &right, true)["wram_offset"].as_integer(), Some(0xd7bf));
    }
}
