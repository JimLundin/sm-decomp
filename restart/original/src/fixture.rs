//! One observed Fire call, compared with actual C and Rust under a fixed projection.
//! This is an isolated routine invocation, never a resumed original game.
use crate::{Result, Table};
use sm_native_host::Game;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

const WRAM: usize = 0x20000;
const WORDS: [usize; 7] = [0xb5e, 0xccc, 0xcce, 0xdc0, 0xdc4, 0xdde, 0x18ac];
const PROJECTION: &str = "schema = \"first-fire-semantic-projection-v1\"\nbytes = 441\nprojectile_range = [2916, 3276]\nsfx_range = [1603, 1670]\nwords = [2910, 3276, 3278, 3520, 3524, 3550, 6316]\nranges_are_half_open = true\nword_bytes = 2\norder = \"projectile range, listed words, SFX range; increasing address within each selection\"\nsource = \"Pinned sm_90.c FireUnchargedBeam and callees, sm_93.c InitializeProjectile, sm_80.c sound queues, variables.h\"\nexclusions = \"CPU registers, CPU stack, DP scratch, removed collision temporaries; full originals remain retained\"\n";

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn addresses() -> impl Iterator<Item = usize> {
    (0xb64..0xccc).chain(WORDS.into_iter().flat_map(|at| at..at + 2)).chain(0x643..0x686)
}
fn projection(ram: &[u8]) -> Vec<u8> {
    addresses().map(|at| ram[at]).collect()
}
pub(crate) fn read_table(path: &Path) -> Result<Table> {
    toml::from_str(&fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?)
        .map_err(|e| format!("{}: {e}", path.display()))
}
pub(crate) fn text<'a>(table: &'a Table, key: &str) -> Result<&'a str> {
    table.get(key).and_then(toml::Value::as_str).ok_or_else(|| format!("missing string {key}"))
}
fn number(table: &Table, key: &str) -> Result<i64> {
    table.get(key).and_then(toml::Value::as_integer).ok_or_else(|| format!("missing integer {key}"))
}
pub(crate) fn require(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn artifact_path(root: &Path, name: &str) -> Result<PathBuf> {
    require(!name.is_empty(), "empty artifact path")?;
    let mut path = root.to_owned();
    for component in Path::new(name).components() {
        let Component::Normal(part) = component else { return Err("unsafe artifact path".into()) };
        path.push(part);
        require(!fs::symlink_metadata(&path).map_err(|e| e.to_string())?.file_type().is_symlink(), "artifact symlink")?;
    }
    require(path.is_file(), "artifact is not a regular file")?;
    Ok(path)
}
pub(crate) fn retain(out: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    let path = out.join(name);
    crate::new_file(&path, bytes)?;
    crate::seal(&path)
}
fn artifacts(out: &Path) -> Result<toml::Value> {
    let mut values = BTreeMap::new();
    for entry in fs::read_dir(out).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name() == "result.toml" {
            continue;
        }
        require(entry.file_type().map_err(|e| e.to_string())?.is_file(), "unexpected fixture output")?;
        values.insert(entry.file_name().to_string_lossy().into_owned(), crate::hash(&entry.path())?);
        crate::seal(&entry.path())?;
    }
    Ok(crate::file_map(values))
}

pub(crate) fn authenticate_capture(capture: &Path) -> Result<Table> {
    let result = read_table(&artifact_path(capture, "result.toml")?)?;
    require(
        text(&result, "schema")? == "sm-original-result-v1"
            && matches!(text(&result, "status")?, "captured" | "failed"),
        "original capture has no completed result record",
    )?;
    let request_path = artifact_path(capture, "request.toml")?;
    require(crate::hash(&request_path)? == text(&result, "request_sha256")?, "original request hash differs")?;
    let request = read_table(&request_path)?;
    require(
        text(&request, "schema")? == "sm-original-request-v1"
            && text(&request, "scenario")? == crate::movie::SCENARIO
            && number(&request, "requested_frames")? == crate::movie::FRAMES as i64,
        "original request scenario differs",
    )?;
    for (path_key, hash_key, expected) in [
        ("rom", "rom_sha256", crate::ROM_SHA256),
        ("lsnes", "lsnes_sha256", crate::LSNES_SHA256),
        ("source_movie", "source_movie_sha256", crate::MOVIE_SHA256),
    ] {
        require(text(&request, hash_key)? == expected, "original pinned identity differs")?;
        crate::authenticate(Path::new(text(&request, path_key)?), expected)?;
    }
    for (file, key) in [("scenario.lsmv", "authored_movie_sha256"), ("observe.lua", "observer_sha256")] {
        require(
            crate::hash(&artifact_path(capture, file)?)? == text(&request, key)?,
            "original authored input hash differs",
        )?;
    }
    require(
        crate::hash_bytes(crate::observer::generate(capture)?.as_bytes()) == text(&request, "observer_sha256")?,
        "capture observer is not this command's reviewed contract",
    )?;
    require(
        request.get("library_hashes")
            == Some(&crate::file_map(crate::library_hashes(Path::new(text(&request, "library_directory")?))?)),
        "original shared libraries changed",
    )?;
    let hashes =
        result.get("artifacts").and_then(toml::Value::as_table).ok_or("original result has no artifact hashes")?;
    for required in ["events.csv", "completion.txt", "stdout.log", "stderr.log"] {
        require(hashes.contains_key(required), "original artifact hash missing")?;
    }
    for (name, digest) in hashes {
        require(
            crate::hash(&artifact_path(capture, name)?)? == digest.as_str().ok_or("invalid artifact hash")?,
            "original artifact hash differs",
        )?;
    }
    for entry in fs::read_dir(capture.join("states")).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        require(
            hashes.contains_key(&format!("states/{}", entry.file_name().to_string_lossy())),
            "unhashed original snapshot",
        )?;
    }
    if text(&result, "status")? == "captured" {
        let observed = crate::validate::capture(capture)?;
        for (key, value) in observed {
            require(result.get(&key) == Some(&value), "original result disagrees with revalidated observations")?;
        }
    }
    let prefix = crate::validate::first_fire_prefix(capture)?;
    let mut proof = Table::new();
    proof.insert("capture_status".into(), text(&result, "status")?.into());
    if text(&result, "status")? == "failed" {
        proof.insert("capture_error".into(), text(&result, "error")?.into());
    }
    proof.insert("first_fire_prefix".into(), prefix.into());
    Ok(proof)
}

struct Call {
    entry_event: u64,
    return_event: u64,
    id: u64,
    entry_cpu: Vec<u8>,
    return_cpu: Vec<u8>,
    input: Vec<u8>,
    original: Vec<u8>,
    ledger: Vec<u8>,
    slot: usize,
    hardware_events: u64,
}
fn select(capture: &Path) -> Result<Call> {
    let ledger = fs::read_to_string(capture.join("events.csv")).map_err(|e| e.to_string())?;
    let mut first = None;
    let mut returned = None;
    let mut fire_ledger = String::from(ledger.lines().next().ok_or("empty original ledger")?);
    fire_ledger.push('\n');
    let mut sfx1 = 0;
    let mut hardware_events = 0;
    let mut status_address = None;
    for line in ledger.lines().skip(1) {
        let p: Vec<_> = line.split(',').collect();
        require(p.len() == 17, "original ledger extent differs")?;
        let n = |i: usize| p[i].parse::<u64>().map_err(|_| "invalid original ledger number".to_owned());
        if p[2] == "fire-entry" {
            require(first.is_none(), "more than one original Fire entry")?;
            first = Some((n(1)?, n(5)?));
        }
        if first.is_none() {
            continue;
        }
        fire_ledger.push_str(line);
        fire_ledger.push('\n');
        require(
            !matches!(p[2], "nmi-entry" | "irq-entry" | "plm-entry"),
            "first original Fire has an interrupt or PLM dependency",
        )?;
        if p[2].starts_with("hardware-") {
            hardware_events += 1;
        }
        if p[2] == "sfx1-max15-entry" {
            require(
                n(3)? == 0x809021 && n(4)? & 255 == 11 && n(5)? == first.unwrap().1,
                "unexpected original first-Fire sound request",
            )?;
            require(status_address.is_none(), "missing original sound status")?;
            sfx1 += 1;
            status_address = Some(n(3)?);
        } else if p[2] == "sfx2-max6-entry" {
            return Err("first original Fire requested impact sound".into());
        } else if p[2] == "semantic-status" {
            require(
                status_address.take() == Some(n(3)?) && n(5)? == first.unwrap().1 && n(4)? & 0x30 == 0,
                "original semantic effect CPU width differs",
            )?;
        }
        if p[2] == "fire-return" {
            require(n(5)? == first.unwrap().1 && n(16)? == 0, "original Fire return identity/interrupt differs")?;
            returned = Some(n(1)?);
            break;
        }
    }
    let (entry_event, id) = first.ok_or("no original Fire entry")?;
    let return_event = returned.ok_or("no original Fire return")?;
    require(sfx1 == 1 && status_address.is_none(), "first original Fire lacks exactly one validated sound request")?;
    let state = |event, extension| {
        fs::read(capture.join(format!("states/state-{event}.{extension}"))).map_err(|e| e.to_string())
    };
    let input = state(entry_event, "wram")?;
    let original = state(return_event, "wram")?;
    require(input.len() == WRAM && original.len() == WRAM, "original WRAM extent differs")?;
    let entry_cpu = state(entry_event, "cpu.toml")?;
    let return_cpu = state(return_event, "cpu.toml")?;
    let e: Table =
        toml::from_str(std::str::from_utf8(&entry_cpu).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let r: Table =
        toml::from_str(std::str::from_utf8(&return_cpu).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let s = usize::try_from(number(&e, "s")?).map_err(|_| "negative original stack")?;
    require((0x100..0x1ffd).contains(&s), "original Fire stack outside WRAM mirror")?;
    let continuation = 0x900000 | u64::from(word(&input, s + 2).wrapping_add(1));
    require(
        number(&e, "boundary_pc")? == 0x90b887
            && number(&e, "pb")? == 0x90
            && number(&e, "pc")? == 0xb888
            && number(&e, "e")? == 0
            && number(&e, "d")? == 0
            && number(&e, "p")? & 0x30 == 0
            && number(&e, "call")? == id as i64,
        "original Fire entry CPU context differs",
    )?;
    for key in ["entry_s", "expected_return_s", "continuation_pc", "saved_status"] {
        require(number(&e, key)? == number(&r, key)?, "original Fire entry/return CPU linkage differs")?;
    }
    require(
        number(&e, "saved_status")? == i64::from(input[s + 1])
            && number(&e, "entry_s")? == s as i64
            && number(&e, "expected_return_s")? == (s + 3) as i64
            && number(&e, "continuation_pc")? == continuation as i64
            && number(&r, "boundary_pc")? == continuation as i64
            && number(&r, "s")? == (s + 3) as i64
            && number(&r, "interrupts")? == 0
            && number(&r, "call")? == id as i64
            && [0x90b89d, 0x90b8a9, 0x90b96d].contains(&number(&r, "exit_pc")?),
        "original Fire inherited status/RTS frame differs",
    )?;
    require(
        word(&input, 0x998) == 8
            && word(&input, 0x79f) == 6
            && word(&input, 0x79b) == 0xdf45
            && word(&input, 0x9a6) == 0
            && word(&input, 0xa76) == 0
            && word(&input, 0x93f) & 0x8000 == 0
            && word(&input, 0xac4) == 0
            && word(&input, 0xac6) == 0
            && word(&input, 0xcce) == 0
            && input[0xccc] == 0
            && word(&input, 0x9b2) == 64
            && word(&input, 0x8b) & 64 != 0
            && word(&input, 0x8f) & 64 != 0,
        "first original Fire is outside ordinary fresh-shot eligibility",
    )?;
    require(
        word(&input, 0xaf6) == 130
            && word(&input, 0xafa) == 75
            && word(&input, 0xa1c) == 1
            && input[0xa1e] == 8
            && input[0xa1f] == 0
            && word(&input, 0xb00) == 21
            && word(&input, 0xb2c) == 0
            && word(&input, 0xb2e) == 0,
        "first original Fire is outside the fixed standing-after-turn context",
    )?;
    require(
        (0..10).all(|i| word(&input, 0xc40 + 2 * i) == 0 && word(&input, 0xc2c + 2 * i) == 0),
        "first original Fire pool is not empty",
    )?;
    let slots: Vec<_> = (0..10).filter(|i| word(&original, 0xc40 + 2 * i) != 0).collect();
    require(
        slots.len() == 1 && slots[0] < 5 && word(&original, 0xcce) == 1,
        "first original Fire did not create exactly one ordinary slot",
    )?;
    let slot = slots[0];
    require(
        word(&original, 0xc18 + 2 * slot) == 0x8000 && word(&original, 0xc68 + 2 * slot) == 0xaef3,
        "first original Fire impacted or left the ordinary-beam domain",
    )?;
    Ok(Call {
        entry_event,
        return_event,
        id,
        entry_cpu,
        return_cpu,
        input,
        original,
        ledger: fire_ledger.into_bytes(),
        slot,
        hardware_events,
    })
}

#[derive(Default)]
struct NoPlm {
    requests: u64,
}
impl sm_projectile::Host for NoPlm {
    fn spawn_plm(&mut self, _ram: &mut [u8], _header: u16) -> Result<u8> {
        self.requests += 1;
        Err("isolated first-Fire fixture forbids a PLM dependency".into())
    }
}
fn difference(left: &[u8], right: &[u8], selected: bool) -> toml::Value {
    let found = if selected {
        addresses().find(|&at| left[at] != right[at])
    } else {
        (0..WRAM).find(|&at| left[at] != right[at])
    };
    let mut result = Table::new();
    result.insert("equal".into(), found.is_none().into());
    if let Some(at) = found {
        result.insert("wram_offset".into(), (at as i64).into());
        result.insert("left".into(), i64::from(left[at]).into());
        result.insert("right".into(), i64::from(right[at]).into());
    }
    result.into()
}

/// Must run in a fresh supervised process: the native host is a process singleton.
pub fn compare(capture: &Path, library: &Path, rom: &Path, out: &Path, deliberate_fault: bool) -> Result<()> {
    fs::create_dir(out).map_err(|e| format!("fixture output must be fresh: {e}"))?;
    let out = out.canonicalize().map_err(|e| e.to_string())?;
    let result = execute(capture, library, rom, &out, deliberate_fault);
    if let Err(error) = &result
        && !out.join("result.toml").exists()
    {
        let mut failure = Table::new();
        failure.insert("schema".into(), "sm-original-fire-fixture-v1".into());
        failure.insert("status".into(), "failed".into());
        failure.insert("error".into(), error.clone().into());
        failure.insert("original_equivalence".into(), "not-established".into());
        let request_path = out.join("request.toml");
        if request_path.exists() {
            let request = read_table(&request_path)?;
            failure.insert("request_sha256".into(), crate::hash(&request_path)?.into());
            if let Some(proof) = request.get("original_capture") {
                failure.insert("original_capture".into(), proof.clone());
            }
        }
        failure.insert("artifacts".into(), artifacts(&out)?);
        crate::table_file(&out.join("result.toml"), &failure)?;
    }
    result
}

fn execute(capture: &Path, library: &Path, rom: &Path, out: &Path, deliberate_fault: bool) -> Result<()> {
    let capture = capture.canonicalize().map_err(|e| e.to_string())?;
    let capture_proof = authenticate_capture(&capture)?;
    let call = select(&capture)?;
    let library = library.canonicalize().map_err(|e| e.to_string())?;
    let library_hash = sm_native_host::authenticate_library(&library)?;
    let manifest = library.parent().ok_or("native library has no parent")?.join("library.sme");
    let rom = crate::authenticate(rom, crate::ROM_SHA256)?;
    let binary = std::env::current_exe().map_err(|e| e.to_string())?;
    retain(out, "fixture-command", &fs::read(&binary).map_err(|e| e.to_string())?)?;
    for (name, path) in [
        ("original-result.toml", capture.join("result.toml")),
        ("original-request.toml", capture.join("request.toml")),
        ("native-library.sme", manifest.clone()),
    ] {
        retain(out, name, &fs::read(path).map_err(|e| e.to_string())?)?;
    }
    for (name, bytes) in [
        ("original-entry.wram", call.input.as_slice()),
        ("original-return.wram", call.original.as_slice()),
        ("original-entry.cpu.toml", call.entry_cpu.as_slice()),
        ("original-return.cpu.toml", call.return_cpu.as_slice()),
        ("original-fire-events.csv", call.ledger.as_slice()),
        ("projection.toml", PROJECTION.as_bytes()),
    ] {
        retain(out, name, bytes)?;
    }
    let mut request = Table::new();
    for (key, value) in [
        ("schema", "sm-original-fire-fixture-request-v1".to_owned()),
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
        ("input_sha256", crate::hash_bytes(&call.input)),
        ("original_return_sha256", crate::hash_bytes(&call.original)),
    ] {
        request.insert(key.into(), value.into());
    }
    request.insert("original_entry_event".into(), (call.entry_event as i64).into());
    request.insert("original_return_event".into(), (call.return_event as i64).into());
    request.insert("original_call".into(), (call.id as i64).into());
    request.insert("deliberate_fault".into(), deliberate_fault.into());
    request.insert("original_capture".into(), capture_proof.clone().into());
    crate::table_file(&out.join("request.toml"), &request)?;

    let game = Game::boot(&library, &rom)?;
    let before = game.projectile_counters();
    let before_state = game.state();
    // SAFETY: Game::boot authenticated this exact library against its complete
    // source recipe. dlopen of the same canonical path references the same DSO.
    // No Game-backed slices are alive during these writes. The input is exactly
    // 128 KiB; both no-argument exports have the signatures in pilot.c/sm_90.c.
    unsafe {
        let direct = libloading::Library::new(&library).map_err(|e| e.to_string())?;
        let commit: libloading::Symbol<'_, unsafe extern "C" fn(*const u8)> =
            direct.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?;
        let begin: libloading::Symbol<'_, unsafe extern "C" fn()> =
            direct.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?;
        let fire: libloading::Symbol<'_, unsafe extern "C" fn()> =
            direct.get(b"FireUnchargedBeam\0").map_err(|e| e.to_string())?;
        commit(call.input.as_ptr());
        begin();
        fire();
    }
    let actual_c = game.wram().to_vec();
    let native_effects = game.effects().to_vec();
    let counts = game.projectile_counters();
    let after_state = game.state();
    retain(out, "actual-c.wram", &actual_c)?;
    retain(out, "actual-c.effects", &native_effects)?;
    require(native_effects.len().is_multiple_of(16), "native effects extent differs")?;
    let c_plm = native_effects
        .as_chunks::<16>()
        .0
        .iter()
        .filter(|event| u32::from_le_bytes(event[..4].try_into().unwrap()) == 3)
        .count();
    require(c_plm == 0 && counts.foreign_calls == before.foreign_calls, "actual C invoked a PLM dependency")?;
    require(
        counts.c_entries[0] == before.c_entries[0] + 1
            && counts.c_entries[1..] == before.c_entries[1..]
            && counts.rust_entries == before.rust_entries
            && counts.rust_completed == before.rust_completed
            && counts.rust_refused == before.rust_refused
            && after_state.host_steps == before_state.host_steps
            && after_state.cpu_fallback_calls == before_state.cpu_fallback_calls
            && after_state.cpu_instructions == before_state.cpu_instructions
            && after_state.nmis == before_state.nmis,
        "actual C did not execute exactly one isolated Fire invocation",
    )?;
    let mut rust = call.input.clone();
    let mut host = NoPlm::default();
    let outcome = sm_projectile::run(sm_projectile::Entry::FireUnchargedBeam, &mut rust, game.rom(), &mut host);
    if deliberate_fault && outcome == Ok(sm_projectile::Outcome::Handled) {
        let at = 0xb64 + 2 * call.slot;
        let changed = word(&rust, at).wrapping_add(1);
        rust[at..at + 2].copy_from_slice(&changed.to_le_bytes());
    }
    retain(out, "rust.wram", &rust)?;
    require(
        outcome? == sm_projectile::Outcome::Handled && host.requests == 0,
        "Rust refused or invoked a PLM dependency",
    )?;
    for (name, ram) in
        [("original.projection", &call.original), ("actual-c.projection", &actual_c), ("rust.projection", &rust)]
    {
        retain(out, name, &projection(ram))?;
    }
    // Authenticate retained dependencies again before claiming any comparison.
    require(
        sm_native_host::authenticate_library(&library)? == library_hash
            && crate::hash(&manifest)? == text(&request, "library_manifest_sha256")?
            && crate::hash(&binary)? == text(&request, "command_sha256")?
            && crate::hash(&rom)? == crate::ROM_SHA256,
        "fixture dependency changed during execution",
    )?;
    require(authenticate_capture(&capture)? == capture_proof, "original validated prefix changed during fixture")?;
    require(
        crate::hash(&capture.join("result.toml"))? == text(&request, "capture_result_sha256")?,
        "original capture result changed during fixture",
    )?;
    let c_rust = actual_c == rust;
    let c_original = projection(&actual_c) == projection(&call.original);
    let rust_original = projection(&rust) == projection(&call.original);
    let passed = c_rust && c_original && rust_original;
    let mut result = Table::new();
    result.insert("schema".into(), "sm-original-fire-fixture-v1".into());
    result.insert("status".into(), if passed { "passed" } else { "mismatch" }.into());
    result.insert("request_sha256".into(), crate::hash(&out.join("request.toml"))?.into());
    result.insert("actual_c_rust_full_wram".into(), difference(&actual_c, &rust, false));
    result.insert("actual_c_original_projection".into(), difference(&actual_c, &call.original, true));
    result.insert("rust_original_projection".into(), difference(&rust, &call.original, true));
    result.insert("projection_bytes".into(), 441.into());
    result.insert("actual_c_fire_invocations".into(), 1.into());
    result.insert("original_plm_calls".into(), 0.into());
    result.insert("actual_c_plm_calls".into(), (c_plm as i64).into());
    result.insert("rust_plm_requests".into(), (host.requests as i64).into());
    result.insert("original_sfx1_max15_sound11_requests".into(), 1.into());
    result.insert("original_first_fire_hardware_events".into(), (call.hardware_events as i64).into());
    result.insert("native_rom_view_sha256".into(), crate::hash_bytes(game.rom()).into());
    result.insert(
        "original_equivalence".into(),
        if passed {
            "one observed Fire invocation; fixed 441-byte semantic output projection"
        } else {
            "not-established"
        }
        .into(),
    );
    result.insert(
        "hardware_effect_equivalence".into(),
        "not-tested; original CPU stack/math MMIO differs in abstraction from recovered C".into(),
    );
    result.insert("sound_effect_scope".into(), "Original sound-request marker plus compared sound-queue storage; native request-call order and audible playback not compared".into());
    result.insert("full_original_wram_equivalence".into(), "not-tested".into());
    result.insert("original_live_replay_equivalence".into(), "not-tested".into());
    result.insert("deliberate_fault".into(), deliberate_fault.into());
    result.insert("original_capture".into(), capture_proof.into());
    result.insert("artifacts".into(), artifacts(out)?);
    crate::table_file(&out.join("result.toml"), &result)?;
    require(passed, "first Fire fixture mismatch; inspect retained result and WRAM/projection artifacts")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retained_capture() -> PathBuf {
        std::env::var_os("SM_ORIGINAL_CAPTURE_PREFIX")
            .map(PathBuf::from)
            .expect("set SM_ORIGINAL_CAPTURE_PREFIX to the retained failed original capture")
    }

    #[test]
    #[ignore = "requires a retained original capture"]
    fn failed_capture_can_authenticate_only_its_first_fire_prefix() {
        let capture = retained_capture();
        let proof = authenticate_capture(&capture).unwrap();
        assert_eq!(text(&proof, "capture_status").unwrap(), "failed");
        assert!(!text(&proof, "capture_error").unwrap().is_empty());
        let prefix = proof["first_fire_prefix"].as_table().unwrap();
        let call = select(&capture).unwrap();
        assert_eq!(number(prefix, "entry_event").unwrap(), call.entry_event as i64);
        assert_eq!(number(prefix, "through_event").unwrap(), call.return_event as i64);
        assert_eq!(text(prefix, "scope").unwrap(), "reset through first Fire return only; no later lifecycle claim");
        assert!(crate::validate::capture(&capture).is_err());
    }

    #[test]
    #[ignore = "requires a retained original capture"]
    fn first_fire_prefix_rejects_missing_input_exit_and_cpu_linkage() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {
            fn drop(&mut self) {
                fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let capture = retained_capture();
        let call = select(&capture).unwrap();
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let scratch = Scratch(std::env::temp_dir().join(format!("sm-original-prefix-{}-{nonce}", std::process::id())));
        fs::create_dir(&scratch.0).unwrap();
        fs::create_dir(scratch.0.join("states")).unwrap();
        for event in [call.entry_event, call.return_event] {
            for extension in ["wram", "cpu.toml"] {
                let name = format!("states/state-{event}.{extension}");
                fs::write(scratch.0.join(&name), fs::read(capture.join(name)).unwrap()).unwrap();
            }
        }
        let ledger_path = scratch.0.join("events.csv");
        let mut ledger = fs::read_to_string(capture.join("events.csv")).unwrap();
        ledger.push_str("deliberately invalid later data\n");
        fs::write(&ledger_path, &ledger).unwrap();
        fs::write(scratch.0.join("failure.txt"), "later Draw failed\n").unwrap();
        let prefix = crate::validate::first_fire_prefix(&scratch.0).unwrap();
        assert_eq!(select(&scratch.0).unwrap().return_event, call.return_event);
        assert!(crate::validate::capture(&scratch.0).is_err());

        fs::write(&ledger_path, ledger.replace(",fire-exit,", ",missing-exit,")).unwrap();
        assert!(crate::validate::first_fire_prefix(&scratch.0).unwrap_err().contains("witnessed call"));

        let frame = number(&prefix, "through_frame").unwrap().to_string();
        let mut changed_reads = 0;
        let missing_input: String = ledger
            .lines()
            .map(|line| {
                let mut fields: Vec<_> = line.split(',').collect();
                if fields.len() == 17 && fields[0] == frame && fields[2] == "controller-read" && fields[4] == "64" {
                    fields[4] = "0";
                    changed_reads += 1;
                }
                fields.join(",") + "\n"
            })
            .collect();
        assert_eq!(changed_reads, 1);
        fs::write(&ledger_path, missing_input).unwrap();
        assert!(crate::validate::first_fire_prefix(&scratch.0).unwrap_err().contains("consumed X"));
        fs::write(&ledger_path, &ledger).unwrap();

        let cpu_path = scratch.0.join(format!("states/state-{}.cpu.toml", call.return_event));
        let mut cpu = read_table(&cpu_path).unwrap();
        let saved_status = number(&cpu, "saved_status").unwrap();
        cpu.insert("saved_status".into(), (saved_status ^ 1).into());
        fs::write(&cpu_path, toml::to_string(&cpu).unwrap()).unwrap();
        assert!(crate::validate::first_fire_prefix(&scratch.0).unwrap_err().contains("entry and witnessed events"));
    }

    #[test]
    fn projection_keeps_projectile_change_and_excludes_cpu_scratch() {
        let original = vec![0; WRAM];
        let mut changed = original.clone();
        changed[0x14] = 7;
        changed[0x1ff0] = 8;
        changed[0xdd2] = 9;
        assert_eq!(projection(&original), projection(&changed));
        assert_eq!(projection(&original).len(), 441);
        changed[0xb64] = 1;
        assert_ne!(projection(&original), projection(&changed));
    }
}
