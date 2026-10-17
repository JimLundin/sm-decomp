//! Frozen actual-C vertical-motion controls. Only the integration owner runs it.
use libloading::Library;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sm_native_host::{
    Game, HookOutcome, PlayerMotionEntry, PlayerMotionObservation, PlayerMotionPhase, PlayerMotionServices, prepare,
};
use sm_player_motion::{BodyFixture, Entry, Output, Terrain, body_fixtures};
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
const TERRAIN_CASES: usize = 31;
#[derive(Clone, Copy)]
struct Native {
    seed: unsafe extern "C" fn(*const u8),
    begin: unsafe extern "C" fn(),
    invoke: unsafe extern "C" fn(u32, i32, *mut u16, *mut i32),
    admitted: unsafe extern "C" fn(u32, i32) -> i32,
    prepared: unsafe extern "C" fn() -> i32,
    clear: unsafe extern "C" fn() -> i32,
    generation: unsafe extern "C" fn() -> u64,
    service: unsafe extern "C" fn(*mut u8, usize, u32, i32, *mut i32, u64) -> i32,
    up: unsafe extern "C" fn(i32),
    down: unsafe extern "C" fn(i32),
    camera: unsafe extern "C" fn() -> i32,
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        unsafe {
            Ok(Self {
                seed: *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                invoke: *library.get(b"sm_native_invoke_player_motion\0").map_err(|e| e.to_string())?,
                admitted: *library.get(b"sm_native_player_motion_fixture_admitted\0").map_err(|e| e.to_string())?,
                prepared: *library.get(b"sm_native_player_motion_prepared_tables\0").map_err(|e| e.to_string())?,
                clear: *library.get(b"sm_native_player_motion_pointer_clear\0").map_err(|e| e.to_string())?,
                generation: *library.get(b"sm_native_player_motion_generation\0").map_err(|e| e.to_string())?,
                service: *library.get(b"sm_native_player_motion_terrain\0").map_err(|e| e.to_string())?,
                up: *library.get(b"Samus_MoveUp\0").map_err(|e| e.to_string())?,
                down: *library.get(b"Samus_MoveDown\0").map_err(|e| e.to_string())?,
                camera: *library.get(b"sm_native_camera_dispatch\0").map_err(|e| e.to_string())?,
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
    fn call(self, entry: Entry) -> (u16, i32) {
        let (id, amount) = selector(entry);
        assert_eq!(unsafe { (self.admitted)(id, amount) }, 1, "unsafe fixture never reaches C");
        let (mut collision, mut applied) = (0, 0);
        unsafe {
            (self.invoke)(id, amount, &mut collision, &mut applied);
        }
        assert_eq!(unsafe { (self.clear)() }, 1);
        (collision, applied)
    }
}
fn selector(entry: Entry) -> (u32, i32) {
    match entry {
        Entry::MoveUp(a) => (0, a),
        Entry::MoveDown(a) => (1, a),
        Entry::CheckSolidEnemy(a) => (2, a),
        Entry::MoveUpNoColl(a) => (3, a),
        Entry::MoveDownNoColl(a) => (4, a),
        Entry::UpFeedback => (5, 0),
        Entry::DownFeedback => (6, 0),
    }
}
fn raw(output: Output) -> (u16, i32) {
    match output {
        Output::Done => (0, 0),
        Output::EnemyCollision { collision, amount } => (collision, amount),
    }
}
struct Service<'a>(&'a mut PlayerMotionServices);
impl Terrain for Service<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn move_vertical(&mut self, ram: &mut [u8], amount: i32) -> Result<i32> {
        self.0.move_vertical(ram, amount)
    }
}
struct Unavailable;
impl Terrain for Unavailable {
    fn available(&self) -> bool {
        false
    }
    fn move_vertical(&mut self, _: &mut [u8], _: i32) -> Result<i32> {
        Err("unexpected terrain request".into())
    }
}
fn install(game: &mut Game) -> Result<()> {
    game.set_player_motion_hook(|entry, amount, ram, services| {
        let entry = match entry {
            PlayerMotionEntry::Up => Entry::MoveUp(amount),
            PlayerMotionEntry::Down => Entry::MoveDown(amount),
        };
        Ok(match sm_player_motion::execute(entry, ram, &mut Service(services))? {
            Some(Output::Done) => HookOutcome::Handled,
            None => HookOutcome::Refused,
            Some(_) => return Err("composed motion root returned checker output".into()),
        })
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    event: PlayerMotionObservation,
    ram: Vec<u8>,
}
type Records = Rc<RefCell<Vec<Record>>>;
fn observe(game: &mut Game) -> Result<Records> {
    let records = Records::default();
    let output = records.clone();
    game.set_player_motion_observer(move |event, ram| {
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
fn block(ram: &mut [u8], index: usize, kind: u16, bts: u8) {
    put(ram, 0x10002 + 2 * index, kind << 12);
    ram[0x16402 + index] = bts;
}
fn records_bytes(records: &[Record]) -> Vec<u8> {
    let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
    for r in records {
        for v in [r.event.phase as u32, r.event.entry as u32, r.event.amount as u32, r.event.result as u32] {
            bytes.extend(v.to_le_bytes());
        }
        bytes.extend(&r.ram);
    }
    bytes
}
fn compare(name: &str, left: &[u8], right: &[u8]) -> Result<()> {
    if left == right {
        return Ok(());
    }
    let at = left.iter().zip(right).position(|(a, b)| a != b).unwrap_or(left.len().min(right.len()));
    Err(format!("{name} mismatch at {at:#x}: {} vs {}", hash(left), hash(right)))
}
fn pair(game: &mut Game, native: Native, records: &Records, case: &BodyFixture, root: &Path) -> Result<()> {
    let directory = root.join(&case.name);
    fs::create_dir(&directory).map_err(|e| e.to_string())?;
    write(&directory.join("input.wram"), &case.wram)?;
    game.clear_player_motion_hook();
    native.seed(&case.wram);
    records.borrow_mut().clear();
    let c_before = game.player_motion_counters();
    let c_output = native.call(case.entry);
    let c_ram = game.wram().to_vec();
    let c_effects = game.effects().to_vec();
    let c_records = records.borrow().clone();
    json(&directory.join("c-counters-before.json"), &c_before)?;
    json(&directory.join("c-counters-after.json"), &game.player_motion_counters())?;
    write(&directory.join("c.wram"), &c_ram)?;
    write(&directory.join("c.effects"), &c_effects)?;
    write(&directory.join("c.observations"), records_bytes(&c_records))?;
    let (id, _) = selector(case.entry);
    install(game)?;
    native.seed(&case.wram);
    records.borrow_mut().clear();
    let before = game.player_motion_counters();
    let (output, ram, effects, seen) = if id < 2 {
        let output = native.call(case.entry);
        (output, game.wram().to_vec(), game.effects().to_vec(), records.borrow().clone())
    } else {
        let mut ram = case.wram.clone();
        let output =
            sm_player_motion::execute(case.entry, &mut ram, &mut Unavailable)?.ok_or("valid direct helper refused")?;
        (raw(output), ram, Vec::new(), Vec::new())
    };
    write(&directory.join("rust.wram"), &ram)?;
    write(&directory.join("rust.effects"), &effects)?;
    write(&directory.join("rust.observations"), records_bytes(&seen))?;
    let after = game.player_motion_counters();
    json(&directory.join("rust-counters-before.json"), &before)?;
    json(&directory.join("rust-counters-after.json"), &after)?;
    assert_eq!(output, c_output, "{} compound return", case.name);
    compare(&case.name, &c_ram, &ram)?;
    compare("effects", &c_effects, &effects)?;
    assert_eq!(c_records, seen, "{} service snapshots", case.name);
    if id < 2 {
        let i = id as usize;
        assert_eq!(after.rust_completed[i] - before.rust_completed[i], 1);
        assert_eq!(after.c_entries, before.c_entries);
        assert_eq!(after.rust_refused, before.rust_refused);
        assert_eq!(after.feedback_returns[i] - before.feedback_returns[i], 1);
        let calls = after.terrain_completions - before.terrain_completions;
        assert!(calls <= 1);
        assert_eq!(after.terrain_requests - before.terrain_requests, calls);
        assert_eq!(after.terrain_invalid_requests, before.terrain_invalid_requests);
        assert_eq!(after.terrain_returns[i] - before.terrain_returns[i], calls);
        assert_eq!(after.enemy_hit_returns[i] - before.enemy_hit_returns[i], 1 - calls);
        assert_eq!(after.no_collision_move_returns[i] - before.no_collision_move_returns[i], 1 - calls);
        assert_eq!(seen.len(), 2 + 2 * calls as usize);
        assert_eq!(seen[0].event.phase, PlayerMotionPhase::RootBegin);
        assert_eq!(seen.last().unwrap().event.phase, PlayerMotionPhase::RootEnd);
    } else {
        assert_eq!(after, before, "direct helper does not create scheduled roots");
    }
    terrain_expectations(case, &ram, &effects, &seen);
    json(
        &directory.join("pair.json"),
        &serde_json::json!({"name":case.name,"entry":id,"collision":output.0,"amount":output.1,
        "actual_c":true,"full_wram":true,"ordered_effects":true,"observer_records":seen.len(),"scheduled_rust_root":id<2,
        "wram_sha256":hash(&ram),"effects_sha256":hash(&effects)}),
    )?;
    println!("pair={} entry={id} actual_c=true full_wram=true", case.name);
    Ok(())
}
fn terrain_base() -> Vec<u8> {
    body_fixtures().into_iter().find(|c| matches!(c.entry, Entry::MoveDown(0))).unwrap().wram
}
fn terrain_cases(rom: &[u8]) -> Vec<BodyFixture> {
    let names = [
        "zero-clear",
        "zero-sand",
        "air-reset",
        "floor",
        "ceiling",
        "square",
        "nonsquare-orientation",
        "nonsquare-column",
        "horizontal-extension",
        "vertical-extension",
        "parity-even",
        "parity-odd",
        "spike-air",
        "spike-fresh",
        "spike-invincible",
        "spike-boss-off",
        "spike-boss-on",
        "door-room",
        "door-nonroom",
        "quicksand-clamp",
        "quicksand-contact",
        "quicksand-full",
        "quicksand-block-contact",
        "crumble-down",
        "crumble-up",
        "speed-pass",
        "speed-stop",
        "speed-pose",
        "special-air-carry",
        "bomb-air",
        "bomb-solid",
    ];
    names
        .into_iter()
        .map(|name| {
            let mut ram = terrain_base();
            let mut entry = Entry::MoveDown(0x10000);
            match name {
                "zero-clear" => entry = Entry::MoveDown(0),
                "zero-sand" => {
                    entry = Entry::MoveDown(0);
                    put(&mut ram, 0x1e71, 1);
                    put(&mut ram, 0xdba, 7);
                }
                "air-reset" => {
                    put(&mut ram, 0x1e71, 9);
                    put(&mut ram, 0xdba, 7);
                }
                "floor" => {
                    entry = Entry::MoveDown(0x18000);
                    put(&mut ram, 0xafc, 0x8000);
                    block(&mut ram, 50, 8, 0);
                }
                "ceiling" => {
                    entry = Entry::MoveUp(-0x18000);
                    put(&mut ram, 0xafc, 0x8000);
                    block(&mut ram, 18, 8, 0);
                }
                "square" => block(&mut ram, 50, 1, 4),
                "nonsquare-orientation" => block(&mut ram, 50, 1, 0x85),
                "nonsquare-column" => {
                    put(&mut ram, 0xaf6, 48);
                    block(&mut ram, 50, 1, 5);
                }
                "horizontal-extension" => {
                    block(&mut ram, 50, 5, 1);
                    block(&mut ram, 51, 8, 0);
                }
                "vertical-extension" => {
                    block(&mut ram, 50, 13, 1);
                    block(&mut ram, 66, 8, 0);
                }
                "parity-even" | "parity-odd" => {
                    put(&mut ram, 0xaf6, 48);
                    put(&mut ram, 0x79f, 4);
                    put(&mut ram, 0x5b6, u16::from(name == "parity-odd"));
                    block(&mut ram, 50, 3, 0x80);
                    block(&mut ram, 51, 11, 0);
                }
                "spike-air" => block(&mut ram, 50, 2, 1),
                "spike-fresh" | "spike-invincible" => {
                    block(&mut ram, 50, 10, 1);
                    if name == "spike-invincible" {
                        put(&mut ram, 0x18a8, 5);
                    }
                }
                "spike-boss-off" | "spike-boss-on" => {
                    block(&mut ram, 50, 10, 0);
                    put(&mut ram, 0x79f, 3);
                    ram[0xd82b] = u8::from(name == "spike-boss-on");
                }
                "door-room" | "door-nonroom" => {
                    // Deterministic bounded ROM-read fixture, not an asserted live room.
                    let want_room = name == "door-room";
                    let address = (0x8000..=0xfffe)
                        .find(|&at| {
                            let pointer = word(rom, 0x78000 + (at & 0x7fff));
                            (0x8000..=0xfff4).contains(&pointer)
                                && (word(rom, 0x18000 + usize::from(pointer & 0x7fff)) & 0x8000 != 0) == want_room
                        })
                        .unwrap();
                    put(&mut ram, 0x7b5, address as u16);
                    block(&mut ram, 50, 9, 0);
                }
                "quicksand-clamp" | "quicksand-contact" | "quicksand-full" | "quicksand-block-contact" => {
                    put(&mut ram, 0x79f, 4);
                    block(&mut ram, 50, if name == "quicksand-block-contact" { 11 } else { 3 }, 0x80);
                    entry = Entry::MoveDown(0x100ab);
                    if name.contains("contact") {
                        put(&mut ram, 0xa6e, 1);
                    }
                    if name == "quicksand-full" {
                        for slot in 0..40 {
                            put(&mut ram, 0x1c37 + 2 * slot, 1);
                        }
                    }
                }
                "crumble-down" | "crumble-up" => {
                    block(&mut ram, 50, 11, 0);
                    if name == "crumble-up" {
                        entry = Entry::MoveUp(0x10000);
                    }
                }
                "speed-pass" | "speed-stop" | "speed-pose" => {
                    block(&mut ram, 50, 11, 0x62);
                    if name == "speed-pass" {
                        put(&mut ram, 0xb3e, 0x400);
                    }
                    if name == "speed-pose" {
                        put(&mut ram, 0xa1c, 0xc9);
                    }
                }
                "special-air-carry" => block(&mut ram, 50, 3, 0),
                "bomb-air" => block(&mut ram, 50, 7, 0x80),
                "bomb-solid" => block(&mut ram, 50, 15, 0x80),
                _ => unreachable!(),
            }
            BodyFixture { name: format!("terrain-{name}"), entry, wram: ram }
        })
        .collect()
}
fn terrain_expectations(case: &BodyFixture, ram: &[u8], effects: &[u8], records: &[Record]) {
    let name = case.name.as_str();
    if !name.starts_with("terrain-") {
        return;
    }
    let applied = records.iter().find(|r| r.event.phase == PlayerMotionPhase::TerrainEnd).unwrap().event.result;
    let spawns: Vec<_> = effects
        .as_chunks::<16>()
        .0
        .iter()
        .filter(|v| word(*v, 0) == 3)
        .map(|v| u32::from_le_bytes(v[4..8].try_into().unwrap()))
        .collect();
    match name {
        "terrain-quicksand-clamp" => {
            assert_eq!(applied, 0x30ab);
            assert_eq!(word(ram, 0x1e71), 1);
            assert_eq!(word(ram, 0x1c37 + 78), 0xb72b);
        }
        "terrain-quicksand-contact" => {
            assert_eq!(applied, 0);
            assert_eq!(word(ram, 0xdd0), 1);
        }
        "terrain-quicksand-full" => {
            assert_eq!(applied, 0x100ab);
            assert_eq!(word(ram, 0x1e71), 0);
            assert_eq!(spawns, vec![0xb72b]);
        }
        "terrain-parity-even" => assert_eq!(spawns, vec![0xb72b, 0xd044]),
        "terrain-parity-odd" => assert_eq!(spawns, vec![0xd044]),
        "terrain-spike-air" | "terrain-spike-invincible" | "terrain-spike-boss-off" => assert_eq!(word(ram, 0xa50), 0),
        "terrain-spike-fresh" => assert_eq!(word(ram, 0xa50), 16),
        "terrain-spike-boss-on" => assert_eq!(word(ram, 0xa50), 60),
        "terrain-floor" => assert_eq!(word(ram, 0xafc), 0xffff),
        "terrain-ceiling" => assert_eq!(word(ram, 0xafc), 0),
        "terrain-square" => assert_eq!(word(ram, 0xdba), 1),
        "terrain-door-room" => assert_eq!(word(ram, 0x998), 9),
        "terrain-door-nonroom" => assert_eq!(word(ram, 0xe16), 1),
        "terrain-crumble-down" => assert_eq!(word(ram, 0xde1c + 78), 4),
        "terrain-speed-pass" | "terrain-speed-pose" => assert_eq!(word(ram, 0x10066), 0xb6),
        "terrain-speed-stop" => assert_eq!(word(ram, 0x1c37 + 78), 0),
        "terrain-special-air-carry" => assert_eq!(word(ram, 0xdd0), 0),
        _ => {}
    }
}

fn admissions(game: &mut Game, native: Native, root: &Path) -> Result<usize> {
    let names = [
        "extension-cycle",
        "extension-out-of-range",
        "spike16",
        "unsealed-plm",
        "shared-extension-target",
        "zero-width",
        "large-amount",
        "invalid-door",
        "up-min",
        "unterminated-list",
        "invalid-feedback",
    ];
    for name in names {
        let mut ram = terrain_base();
        let mut entry = Entry::MoveDown(0x10000);
        match name {
            "extension-cycle" => {
                block(&mut ram, 50, 5, 1);
                block(&mut ram, 51, 5, 0xff);
            }
            "extension-out-of-range" => block(&mut ram, 50, 13, 0x80),
            "spike16" => block(&mut ram, 50, 10, 16),
            "unsealed-plm" => block(&mut ram, 50, 11, 1),
            "shared-extension-target" => {
                put(&mut ram, 0xaf6, 48);
                block(&mut ram, 50, 5, 1);
            }
            "zero-width" => put(&mut ram, 0x7a5, 0),
            "large-amount" => entry = Entry::MoveDown(0x110000),
            "invalid-door" => {
                block(&mut ram, 50, 9, 0);
                put(&mut ram, 0x7b5, 1);
            }
            "up-min" => entry = Entry::MoveUp(i32::MIN),
            "unterminated-list" => {
                put(&mut ram, 0x17a6, 1);
                ram[0x17ec..0x182c].fill(0);
            }
            "invalid-feedback" => ram[0xa1f] = 0xff,
            _ => unreachable!(),
        }
        native.seed(&ram);
        let (id, amount) = selector(entry);
        assert_eq!(unsafe { (native.admitted)(id, amount) }, 0, "{name}");
        let before = ram.clone();
        let result = sm_player_motion::execute(entry, &mut ram, &mut Unavailable);
        assert!(matches!(result, Ok(None) | Err(_)), "{name}");
        assert_eq!(ram, before, "{name} unchanged");
        compare("read-only admission canonical WRAM", &before, game.wram())?;
        let directory = root.join(format!("admission-{name}"));
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        write(&directory.join("input.wram"), &before)?;
        json(
            &directory.join("result.json"),
            &serde_json::json!({"name":name,"refused_before_c":true,"unchanged":true}),
        )?;
    }
    // A legal unchanged callback refusal does fall through to the preserved C
    // body. Its branch counters must not imply any Rust helper reached return.
    let ram = terrain_base();
    game.clear_player_motion_hook();
    native.seed(&ram);
    native.call(Entry::MoveDown(0x10000));
    let expected = game.wram().to_vec();
    game.set_player_motion_hook(|_, _, _, _| Ok(HookOutcome::Refused))?;
    native.seed(&ram);
    let before = game.player_motion_counters();
    native.call(Entry::MoveDown(0x10000));
    let after = game.player_motion_counters();
    compare("unchanged refusal C fallback", &expected, game.wram())?;
    assert_eq!(after.rust_refused[1] - before.rust_refused[1], 1);
    assert_eq!(after.c_entries[1] - before.c_entries[1], 1);
    assert_eq!(after.rust_completed, before.rust_completed);
    assert_eq!(after.feedback_returns, before.feedback_returns);
    assert_eq!(after.enemy_hit_returns, before.enemy_hit_returns);
    assert_eq!(after.terrain_returns, before.terrain_returns);
    assert_eq!(after.no_collision_move_returns, before.no_collision_move_returns);
    json(
        &root.join("unchanged-refusal.json"),
        &serde_json::json!({"c_fallback":true,"rust_refused":1,"rust_completed":0}),
    )?;
    Ok(names.len() + 1)
}
const FATALS: &[(&str, &str)] = &[
    ("dirty-refusal", "refusal changed private WRAM"),
    ("request-refusal", "refusal after foreign request"),
    ("postrequest-error", "postrequest failure control"),
    ("postrequest-panic", "Rust player-motion panic"),
    ("wrong-buffer", "player-motion poisoned request"),
    ("short-buffer", "player-motion poisoned request"),
    ("wrong-selector", "player-motion poisoned request"),
    ("wrong-generation", "player-motion poisoned request"),
    ("changed-geometry", "player-motion poisoned request"),
    ("recursive-root", "recursive or cross-consumer player-motion root"),
    ("sibling-entry", "recursive or cross-consumer camera root"),
    ("recursive-observer", "recursive or cross-consumer player-motion root"),
    ("service-observer-error", "player-motion observer failed"),
];
fn fatal(game: &mut Game, native: Native, root: &Path, mode: &str) -> Result<()> {
    let ram = terrain_base();
    write(&root.join("input.wram"), &ram)?;
    native.seed(&ram);
    json(&root.join("invocation.json"), &serde_json::json!({"mode":mode,"entry":1,"amount":65536,"expected_exit":2}))?;
    if mode == "recursive-observer" {
        game.set_player_motion_observer(move |_, _| {
            unsafe { (native.up)(0x10000) };
            Ok(())
        })?;
    } else if mode == "service-observer-error" {
        install(game)?;
        game.set_player_motion_observer(|event, _| {
            if event.phase == PlayerMotionPhase::TerrainEnd {
                Err("service-end observer failure control".into())
            } else {
                Ok(())
            }
        })?;
    } else {
        let mode = mode.to_owned();
        game.set_player_motion_hook(move |_, amount, ram, services| {
            if mode == "dirty-refusal" {
                ram[0] ^= 1;
                return Ok(HookOutcome::Refused);
            }
            put(ram, 0xb02, 3);
            put(ram, 0xdd0, 0);
            match mode.as_str() {
                "request-refusal" => {
                    services.move_vertical(ram, amount)?;
                    return Ok(HookOutcome::Refused);
                }
                "postrequest-error" => {
                    services.move_vertical(ram, amount)?;
                    return Err("postrequest failure control".into());
                }
                "postrequest-panic" => {
                    services.move_vertical(ram, amount)?;
                    panic!("postrequest panic control");
                }
                "wrong-buffer" => {
                    let _ = services.move_vertical(&mut ram.to_vec(), amount);
                }
                "short-buffer" => {
                    let _ = services.move_vertical(&mut ram[..WRAM - 1], amount);
                }
                "wrong-selector" | "wrong-generation" => {
                    let mut result = 0;
                    let generation = unsafe { (native.generation)() } + u64::from(mode == "wrong-generation");
                    unsafe {
                        (native.service)(
                            ram.as_mut_ptr(),
                            ram.len(),
                            u32::from(mode == "wrong-selector"),
                            amount,
                            &mut result,
                            generation,
                        );
                    }
                }
                "changed-geometry" => {
                    put(ram, 0x7a5, 0);
                    let _ = services.move_vertical(ram, amount);
                }
                "recursive-root" => unsafe { (native.down)(amount) },
                "sibling-entry" => unsafe {
                    (native.camera)();
                },
                _ => return Err("unknown fatal player-motion mode".into()),
            }
            Ok(HookOutcome::Refused)
        })?;
    }
    unsafe { (native.down)(0x10000) };
    Err("fatal player-motion control unexpectedly returned".into())
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
            &serde_json::json!({"mode":mode,"expected_error":expected,"exit_code":status.code(),"expected_exit_code":2,
            "timed_out":timed_out,"verified":verified,"elapsed_ms":started.elapsed().as_millis(),"stderr_sha256":hash(&stderr),
            "stdout_sha256":hash(&read(&directory.join("stdout.txt"))?),"argv":[exe.to_string_lossy().as_ref(),&args[1],&args[2],&args[3],&format!("fatal-{mode}")]}),
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
        return Err("usage: player-motion-host-control ARCHIVE ROM OUTPUT MODE(prepare|all|bodies|admissions|fatals|fatals-START|fatal-NAME)".into());
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
        return Err("unknown player-motion control mode".into());
    }
    let root = PathBuf::from(&args[3]).join(format!("player-motion-{mode}"));
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
    assert_eq!(unsafe { (native.prepared)() }, 1, "prepared ROM slices differ from frozen terrain headers");
    if let Some(mode) = mode.strip_prefix("fatal-") {
        return fatal(&mut game, native, &root, mode);
    }
    let records = observe(&mut game)?;
    let mut body_count = 0;
    let mut terrain_count = 0;
    if mode != "admissions" {
        let bodies = body_fixtures();
        assert_eq!(bodies.len(), 192);
        let terrains = terrain_cases(game.rom());
        assert_eq!(terrains.len(), TERRAIN_CASES);
        for case in &bodies {
            pair(&mut game, native, &records, case, &root)?;
        }
        body_count = bodies.len();
        for case in &terrains {
            pair(&mut game, native, &records, case, &root)?;
        }
        terrain_count = terrains.len();
    }
    let admission_count = admissions(&mut game, native, &root)?;
    let fatal_count = if mode == "all" { fatals(&args, &root, 0)? } else { 0 };
    json(
        &root.join("result.json"),
        &serde_json::json!({"status":"passed","shared_body_pairs":body_count,"terrain_pairs":terrain_count,
        "admission_controls":admission_count,"fatal_controls":fatal_count,"library_sha256":hash(&read(&prepared.library)?),
        "executable_sha256":hash(&read(&std::env::current_exe().map_err(|e|e.to_string())?)?),"original_rom_comparison":"not-tested"}),
    )?;
    println!(
        "player_motion_body_pairs={body_count} terrain_pairs={terrain_count} admission_controls={admission_count} fatal_controls={fatal_count}"
    );
    Ok(())
}
