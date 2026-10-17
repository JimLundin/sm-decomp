//! Root-owned eight-call composition check through production owner installers.
//! Usage: waver-runtime-control LIB ROM WAVER_INPUT PLAYER_INPUT PERSIST_REFERENCE OUT
//! Waver input: admitted main-positive-sine; player input: terrain-zero-clear.
//! Persistence reference: accepted24 save-slot-0-area-0/c directory. OUT is fresh.
//! Rust invocations keep all ten owners installed; each C comparator disables
//! only its target, leaving nine other production registrations alive.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_binary_evidence::ToEvidence;
use sm_native_host::{Game, PersistenceCounters, PlayerMotionCounters, WaverCounters};
use sm_restart::{gameplay, persistence_runtime};
use std::{
    fs,
    path::{Path, PathBuf},
};
type Result<T> = std::result::Result<T, String>;
const WRAM: usize = 0x20000;
const SRAM: usize = 0x2000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Waver,
    Player,
    Save,
    Load,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Output {
    Done,
    Loaded(u8),
}
#[derive(Clone, Copy)]
struct Native {
    commit: unsafe extern "C" fn(*const u8, *const u8),
    begin: unsafe extern "C" fn(),
    waver: unsafe extern "C" fn(u32),
    player: unsafe extern "C" fn(i32),
    save: unsafe extern "C" fn(u16),
    load: unsafe extern "C" fn(u16) -> u8,
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        // SAFETY: Game authenticated this retained process-lifetime library.
        // These signatures match the native AI/ordinary body/state exports.
        unsafe {
            Ok(Self {
                commit: *library.get(b"sm_native_commit_persistence\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                waver: *library.get(b"sm_native_waver_ai\0").map_err(|e| e.to_string())?,
                player: *library.get(b"Samus_MoveDown\0").map_err(|e| e.to_string())?,
                save: *library.get(b"SaveToSram\0").map_err(|e| e.to_string())?,
                load: *library.get(b"LoadFromSram\0").map_err(|e| e.to_string())?,
            })
        }
    }
    fn seed(self, ram: &[u8], sram: &[u8]) {
        assert_eq!(ram.len(), WRAM);
        assert_eq!(sram.len(), SRAM);
        // SAFETY: complete buffers, no live borrow/callback, synchronous copy.
        unsafe {
            (self.commit)(ram.as_ptr(), sram.as_ptr());
        }
    }
    fn invoke(self, target: Target) -> Output {
        // SAFETY: only declared retained fixture inputs are used, with admitted
        // Waver Main, downward amount0 and persistence slot0; owners are idle.
        unsafe {
            (self.begin)();
            match target {
                Target::Waver => {
                    (self.waver)(1);
                    Output::Done
                }
                Target::Player => {
                    (self.player)(0);
                    Output::Done
                }
                Target::Save => {
                    (self.save)(0);
                    Output::Done
                }
                Target::Load => Output::Loaded((self.load)(0)),
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Counters {
    waver: WaverCounters,
    player: PlayerMotionCounters,
    persistence: PersistenceCounters,
    others: String,
}
fn counters(game: &Game) -> Counters {
    Counters {
        waver: game.waver_counters(),
        player: game.player_motion_counters(),
        persistence: game.persistence_counters(),
        others: format!(
            "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",
            game.projectile_counters(),
            game.scroll_counters(),
            game.ceres_door_counters(),
            game.room_transition_counters(),
            game.door_scrolling_counters(),
            game.camera_counters(),
            game.item_instruction_counters()
        ),
    }
}
fn expected_counts(mut before: Counters, target: Target, rust: bool) -> Counters {
    match target {
        Target::Waver => {
            let c = &mut before.waver;
            c.entries[1] += 1;
            if rust {
                c.rust_entries[1] += 1;
                c.rust_completed[1] += 1;
            } else {
                c.c_entries[1] += 1;
            }
            for i in 0..3 {
                c.service_requests[i] += 1;
                c.service_completions[i] += 1;
            }
        }
        Target::Player => {
            let c = &mut before.player;
            c.entries[1] += 1;
            c.terrain_requests += 1;
            c.terrain_completions += 1;
            if rust {
                c.rust_entries[1] += 1;
                c.rust_completed[1] += 1;
                c.terrain_returns[1] += 1;
                c.feedback_returns[1] += 1;
            } else {
                c.c_entries[1] += 1;
            }
        }
        Target::Save | Target::Load => {
            let c = &mut before.persistence;
            let i = usize::from(target == Target::Load);
            c.entries[i] += 1;
            if rust {
                c.rust_entries[i] += 1;
                c.rust_completed[i] += 1;
            } else {
                c.c_entries[i] += 1;
            }
            if target == Target::Save {
                c.write_requests += 1;
                c.write_completions += 1;
            }
        }
    }
    before
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let p = path.as_ref();
    fs::read(p).map_err(|e| format!("{}: {e}", p.display()))
}
fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> Result<()> {
    let p = path.as_ref();
    fs::write(p, bytes).map_err(|e| format!("{}: {e}", p.display()))
}
fn write_counters(path: &Path, c: &Counters) -> Result<()> {
    let value = sm_binary_evidence::Value::Table(sm_binary_evidence::Map::from([
        ("waver".into(), c.waver.to_evidence()?),
        ("player".into(), c.player.to_evidence()?),
        ("persistence".into(), c.persistence.to_evidence()?),
    ]));
    write(path.with_extension("sme"), sm_binary_evidence::encode(&value)?)?;
    write(path.with_extension("siblings.txt"), &c.others)
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    output: Output,
    wram: Vec<u8>,
    sram: Vec<u8>,
    effects: Vec<u8>,
}
fn call(game: &mut Game, native: Native, target: Target, rust: bool, path: &Path) -> Result<Snapshot> {
    fs::create_dir(path).map_err(|e| e.to_string())?;
    write(path.join("input.wram"), game.wram())?;
    write(path.join("input.sram"), game.sram())?;
    let before = counters(game);
    write_counters(&path.join("counters-before.toml"), &before)?;
    let output = native.invoke(target);
    let result =
        Snapshot { output, wram: game.wram().to_vec(), sram: game.sram().to_vec(), effects: game.effects().to_vec() };
    write(path.join("final.wram"), &result.wram)?;
    write(path.join("final.sram"), &result.sram)?;
    write(path.join("effects.bin"), &result.effects)?;
    write(path.join("output.txt"), format!("{output:?}\n"))?;
    let after = counters(game);
    write_counters(&path.join("counters-after.toml"), &after)?;
    assert_eq!(
        after,
        expected_counts(before, target, rust),
        "target counters/service balance and unchanged other owners: {target:?}"
    );
    if matches!(target, Target::Save | Target::Load) {
        assert_eq!(read("saves/sm.srm")?, result.sram);
        assert_eq!(read("saves/sm.srm.bak")?, vec![0xa5; SRAM]);
        if target == Target::Load {
            assert_eq!(output, Output::Loaded(0));
        }
    } else {
        assert_eq!(read("saves/sm.srm")?, vec![0xa5; SRAM]);
        assert_eq!(read("saves/sm.srm.bak")?, vec![0x5a; SRAM]);
        assert_eq!(result.sram, read(path.join("input.sram"))?, "non-persistence owner preserves full SRAM");
    }
    Ok(result)
}
fn install_all(game: &mut Game) -> Result<()> {
    gameplay::install_waver(game)?;
    gameplay::install_player_motion(game)?;
    persistence_runtime::install(game)?;
    gameplay::install_projectiles(game, false)?;
    gameplay::install_scrolling(game, false)?;
    gameplay::install_ceres_doors(game, false)?;
    gameplay::install_room_transitions(game, false)?;
    gameplay::install_door_scrolling(game, false)?;
    gameplay::install_camera(game, false)?;
    gameplay::install_item_instructions(game)
}
fn select(game: &mut Game, target: Target, rust: bool) -> Result<()> {
    match (target, rust) {
        (Target::Waver, true) => gameplay::install_waver(game)?,
        (Target::Waver, false) => game.clear_waver_hook(),
        (Target::Player, true) => gameplay::install_player_motion(game)?,
        (Target::Player, false) => game.clear_player_motion_hook(),
        (_, true) => persistence_runtime::install(game)?,
        (_, false) => game.clear_persistence_hook(),
    }
    Ok(())
}
fn fresh_cwd(path: &Path) -> Result<()> {
    fs::create_dir(path).map_err(|e| e.to_string())?;
    std::env::set_current_dir(path).map_err(|e| e.to_string())?;
    fs::create_dir("saves").map_err(|e| e.to_string())?;
    write("saves/sm.srm", vec![0xa5; SRAM])?;
    write("saves/sm.srm.bak", vec![0x5a; SRAM])
}
fn existing(s: &str) -> Result<PathBuf> {
    fs::canonicalize(s).map_err(|e| format!("{s}: {e}"))
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err("usage: waver-runtime-control LIB ROM WAVER_INPUT PLAYER_INPUT PERSIST_REFERENCE OUT".into());
    }
    let library_path = existing(&args[0])?;
    let rom = existing(&args[1])?;
    let waver_path = existing(&args[2])?;
    let player_path = existing(&args[3])?;
    let reference = existing(&args[4])?;
    let old_cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let out = old_cwd.join(&args[5]);
    fs::create_dir(&out).map_err(|e| format!("fresh output {}: {e}", out.display()))?;
    let out = fs::canonicalize(out).map_err(|e| e.to_string())?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut manifest = String::new();
    for path in [&executable, &library_path, &rom, &waver_path, &player_path] {
        manifest.push_str(&format!("{}  {}\n", hash(&read(path)?), path.display()));
    }
    for file in ["input.wram", "input.sram", "final.wram", "final.sram", "effects.bin"] {
        let path = reference.join(file);
        manifest.push_str(&format!("{}  {}\n", hash(&read(&path)?), path.display()));
    }
    write(out.join("inputs.sha256"), manifest)?;
    write(out.join("argv.txt"), format!("{:?}\n", std::env::args().collect::<Vec<_>>()))?;
    let waver = read(&waver_path)?;
    let player = read(&player_path)?;
    let saved_ram = read(reference.join("input.wram"))?;
    let saved_sram = read(reference.join("input.sram"))?;
    assert_eq!(waver.len(), WRAM);
    assert_eq!(player.len(), WRAM);
    assert_eq!(saved_ram.len(), WRAM);
    assert_eq!(saved_sram.len(), SRAM);
    std::env::set_current_dir(&out).map_err(|e| e.to_string())?;
    let mut game = Game::boot(&library_path, &rom)?;
    // SAFETY: retain the second handle to the same authenticated library until
    // all synchronous diagnostic invocations have finished.
    let library = unsafe { Library::new(&library_path) }.map_err(|e| e.to_string())?;
    let native = unsafe { Native::load(&library)? };
    install_all(&mut game)?;
    let initial = counters(&game);
    write_counters(&out.join("initial-counters.toml"), &initial)?;
    for (target, name, input) in [(Target::Waver, "waver", &waver), (Target::Player, "player", &player)] {
        let mut results = Vec::new();
        for rust in [false, true] {
            fresh_cwd(&out.join(format!("{name}-{}", if rust { "rust" } else { "c" })))?;
            select(&mut game, target, rust)?;
            native.seed(input, &vec![0; SRAM]);
            results.push(call(&mut game, native, target, rust, Path::new("call"))?);
        }
        assert_eq!(results[0], results[1], "production {name} full-state/output/effect pair");
    }
    let mut results = Vec::new();
    for rust in [false, true] {
        fresh_cwd(&out.join(if rust { "persistence-rust" } else { "persistence-c" }))?;
        select(&mut game, Target::Save, rust)?;
        native.seed(&saved_ram, &saved_sram);
        let save = call(&mut game, native, Target::Save, rust, Path::new("save"))?;
        for (file, bytes) in [("final.wram", &save.wram), ("final.sram", &save.sram), ("effects.bin", &save.effects)] {
            assert_eq!(*bytes, read(reference.join(file))?, "accepted24 persistence {file}");
        }
        let mut modified = save.wram.clone();
        modified[0x9a2..0xa02].fill(0);
        modified[0x78b..0x78d].fill(0);
        modified[0x79f..0x7a1].fill(0);
        native.seed(&modified, &save.sram);
        let load = call(&mut game, native, Target::Load, rust, Path::new("load"))?;
        assert_eq!(&load.wram[0x9a2..0xa02], &saved_ram[0x9a2..0xa02]);
        assert_eq!(&load.wram[0x78b..0x78d], &saved_ram[0x78b..0x78d]);
        results.push((save, load));
    }
    assert_eq!(results[0], results[1], "production persistence full-state/output/effect pair");
    write_counters(&out.join("final-counters.toml"), &counters(&game))?;
    write(
        out.join("result.toml"),
        "success = true\nordinary_calls = 8\nfull_state_pairs = 4\nrust_production_owners_registered = 10\nc_comparator_other_owners_registered = 9\nwaver_main_completions = 1\nplayer_down_completions = 1\npersistence_save_completions = 1\npersistence_load_completions = 1\nunrelated_owner_counters_unchanged = true\nservices_balanced = true\nlive_gameplay = false\nfile_reopen_continue = false\n",
    )?;
    std::env::set_current_dir(old_cwd).map_err(|e| e.to_string())?;
    println!(
        "composition=pass ordinary_calls=8 full_state_pairs=4 production_owners=10 services_balanced=true live=false"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Waver runtime control: {error}");
        std::process::exit(1);
    }
}
