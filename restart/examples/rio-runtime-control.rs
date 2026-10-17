//! Eight-call affected composition through the public production installers.
//! CLI: LIB ROM JET_INPUT RIPPER_INPUT WAVER_INPUT RIO_INPUT FRESH_OUT
//! Inputs: retained28 jet-main-air-positive/ripper-main-air-positive input.wram,
//! retained27 main-positive-sine/input.wram, Rio Main-air/input.wram from the
//! current native control. No build, game step or Save call.
//! All twelve owners are registered for Rust. C disables only the target family.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_binary_evidence::ToEvidence;
use sm_native_host::{Game, PersistenceCounters, PlayerMotionCounters, RipperCounters, WaverCounters};
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
    Jet,
    Ripper,
    Waver,
    Rio,
}
impl Target {
    fn address(self) -> u32 {
        match self {
            Self::Jet => 0xa2e353,
            Self::Ripper => 0xa2e4da,
            Self::Waver => 0xa3874c,
            Self::Rio => 0xa2bbe3,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Output {
    Done,
}
#[derive(Clone, Copy)]
struct Native {
    commit: unsafe extern "C" fn(*const u8, *const u8),
    begin: unsafe extern "C" fn(),
    ai: unsafe extern "C" fn(u32),
    rio_admitted: unsafe extern "C" fn(u32, u16, u32) -> i32,
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        // SAFETY: Game authenticated this library. Exact original AI and retained
        // complete-state diagnostic signatures are bound to its process lifetime.
        unsafe {
            Ok(Self {
                commit: *library.get(b"sm_native_commit_persistence\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                ai: *library.get(b"CallEnemyAi\0").map_err(|e| e.to_string())?,
                rio_admitted: *library.get(b"sm_native_rio_fixture_admitted\0").map_err(|e| e.to_string())?,
            })
        }
    }
    fn seed(self, ram: &[u8], sram: &[u8]) {
        assert_eq!(ram.len(), WRAM);
        assert_eq!(sram.len(), SRAM);
        // SAFETY: synchronous copy of complete buffers, every owner idle.
        unsafe {
            (self.commit)(ram.as_ptr(), sram.as_ptr());
        }
    }
    fn invoke(self, target: Target) -> Output {
        // SAFETY: retained admitted air fixtures and these exact three AI cases;
        // no Game borrow or callback is active before the synchronous invocation.
        unsafe {
            if target == Target::Rio {
                assert_eq!((self.rio_admitted)(1, 0, 0), 1, "unsafe Rio composition fixture");
            }
            (self.begin)();
            (self.ai)(target.address());
        }
        Output::Done
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Counters {
    ripper: RipperCounters,
    waver: WaverCounters,
    player: PlayerMotionCounters,
    persistence: PersistenceCounters,
    rio: [u64; 67],
    others: String,
}
fn counters(game: &Game) -> Counters {
    Counters {
        ripper: game.ripper_counters(),
        waver: game.waver_counters(),
        player: game.player_motion_counters(),
        persistence: game.persistence_counters(),
        rio: game.rio_counters(),
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
        Target::Jet | Target::Ripper => {
            let i = if target == Target::Jet { 1 } else { 3 };
            let c = &mut before.ripper;
            c.entries[i] += 1;
            if rust {
                c.rust_entries[i] += 1;
                c.rust_completed[i] += 1;
            } else {
                c.c_entries[i] += 1;
            }
            c.service_requests += 1;
            c.service_completions += 1;
        }
        Target::Rio => {
            let c = &mut before.rio;
            c[1] += 1;
            if rust {
                c[21] += 1;
                c[41] += 1;
            } else {
                c[11] += 1;
            }
            for i in 0..2 {
                c[60 + i] += 1;
                c[63 + i] += 1;
            }
        }
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
    }
    before
}
fn difference<const N: usize>(after: [u64; N], before: [u64; N]) -> [u64; N] {
    std::array::from_fn(|i| after[i] - before[i])
}
fn ripper_delta(a: RipperCounters, b: RipperCounters) -> RipperCounters {
    RipperCounters {
        entries: difference(a.entries, b.entries),
        c_entries: difference(a.c_entries, b.c_entries),
        rust_entries: difference(a.rust_entries, b.rust_entries),
        rust_refused: difference(a.rust_refused, b.rust_refused),
        rust_completed: difference(a.rust_completed, b.rust_completed),
        terrain_unavailable: difference(a.terrain_unavailable, b.terrain_unavailable),
        service_requests: a.service_requests - b.service_requests,
        service_completions: a.service_completions - b.service_completions,
        invalid_requests: a.invalid_requests - b.invalid_requests,
    }
}
fn waver_delta(a: WaverCounters, b: WaverCounters) -> WaverCounters {
    WaverCounters {
        entries: difference(a.entries, b.entries),
        c_entries: difference(a.c_entries, b.c_entries),
        rust_entries: difference(a.rust_entries, b.rust_entries),
        rust_refused: difference(a.rust_refused, b.rust_refused),
        rust_completed: difference(a.rust_completed, b.rust_completed),
        terrain_unavailable: difference(a.terrain_unavailable, b.terrain_unavailable),
        service_requests: difference(a.service_requests, b.service_requests),
        service_completions: difference(a.service_completions, b.service_completions),
        invalid_requests: a.invalid_requests - b.invalid_requests,
    }
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
        ("ripper".into(), c.ripper.to_evidence()?),
        ("waver".into(), c.waver.to_evidence()?),
        ("player".into(), c.player.to_evidence()?),
        ("persistence".into(), c.persistence.to_evidence()?),
        ("rio".into(), c.rio.to_evidence()?),
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
    let delta = if target == Target::Rio {
        difference(after.rio, before.rio).to_evidence()
    } else if target == Target::Waver {
        (waver_delta(after.waver, before.waver)).to_evidence()
    } else {
        (ripper_delta(after.ripper, before.ripper)).to_evidence()
    }
    .map_err(|e| e.to_string())?;
    write(path.join("target-delta.sme"), sm_binary_evidence::encode(&delta)?)?;
    assert_eq!(
        after,
        expected_counts(before, target, rust),
        "exacttargetdelta/servicebalance and unchanged other owners: {target:?}"
    );
    assert_eq!(result.sram, read(path.join("input.sram"))?, "actor preserves complete seeded SRAM");
    assert!(!Path::new("saves").exists(), "composition contains no native disk Save request");
    Ok(result)
}
fn install_all(game: &mut Game) -> Result<()> {
    game.install_rio();
    gameplay::install_ripper(game)?;
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
        (Target::Rio, true) => game.install_rio(),
        (Target::Rio, false) => game.clear_rio(),
        (Target::Waver, true) => gameplay::install_waver(game)?,
        (Target::Waver, false) => game.clear_waver_hook(),
        (_, true) => gameplay::install_ripper(game)?,
        (_, false) => game.clear_ripper_hook(),
    }
    Ok(())
}
fn existing(s: &str) -> Result<PathBuf> {
    fs::canonicalize(s).map_err(|e| format!("{s}: {e}"))
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 7 {
        return Err("usage: rio-runtime-control LIB ROM JET_INPUT RIPPER_INPUT WAVER_INPUT RIO_INPUT FRESH_OUT".into());
    }
    let library_path = existing(&args[0])?;
    let rom = existing(&args[1])?;
    let jet_path = existing(&args[2])?;
    let ripper_path = existing(&args[3])?;
    let waver_path = existing(&args[4])?;
    let rio_path = existing(&args[5])?;
    let old_cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let out = old_cwd.join(&args[6]);
    fs::create_dir(&out).map_err(|e| format!("fresh output {}: {e}", out.display()))?;
    let out = fs::canonicalize(out).map_err(|e| e.to_string())?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut manifest = String::new();
    for path in [&executable, &library_path, &rom, &jet_path, &ripper_path, &waver_path, &rio_path] {
        manifest.push_str(&format!("{}  {}\n", hash(&read(path)?), path.display()));
    }
    write(out.join("inputs.sha256"), manifest)?;
    write(out.join("runner.rs"), include_bytes!("rio-runtime-control.rs"))?;
    write(out.join("argv.txt"), format!("{:?}\n", std::env::args().collect::<Vec<_>>()))?;
    let jet = read(&jet_path)?;
    let ripper = read(&ripper_path)?;
    let waver = read(&waver_path)?;
    let rio = read(&rio_path)?;
    for ram in [&jet, &ripper, &waver, &rio] {
        assert_eq!(ram.len(), WRAM);
    }
    let sram: Vec<u8> = (0..SRAM).map(|i| (i * 31 + i / 251 + 17) as u8).collect();
    std::env::set_current_dir(&out).map_err(|e| e.to_string())?;
    let mut game = Game::boot(&library_path, &rom)?;
    // SAFETY: keep the second handle to the same authenticated library alive for
    // every diagnostic invocation; no native reset/reopen lifecycle is tested.
    let library = unsafe { Library::new(&library_path) }.map_err(|e| e.to_string())?;
    let native = unsafe { Native::load(&library)? };
    install_all(&mut game)?;
    write_counters(&out.join("initial-counters.toml"), &counters(&game))?;
    for (target, name, input) in [
        (Target::Jet, "jet", &jet),
        (Target::Ripper, "ripper", &ripper),
        (Target::Waver, "waver", &waver),
        (Target::Rio, "rio", &rio),
    ] {
        let mut results = Vec::new();
        for rust in [false, true] {
            let path = out.join(format!("{name}-{}", if rust { "rust" } else { "c" }));
            fs::create_dir(&path).map_err(|e| e.to_string())?;
            std::env::set_current_dir(path).map_err(|e| e.to_string())?;
            select(&mut game, target, rust)?;
            native.seed(input, &sram);
            results.push(call(&mut game, native, target, rust, Path::new("call"))?);
        }
        assert_eq!(results[0], results[1], "production {name} full WRAM/SRAM/effect/output pair");
    }
    write_counters(&out.join("final-counters.toml"), &counters(&game))?;
    write(
        out.join("result.toml"),
        "success = true\nordinary_calls = 8\nfull_state_pairs = 4\nrust_production_owners_registered = 12\nc_comparator_other_owners_registered = 11\njet_main_completions = 1\nripper_main_completions = 1\nwaver_main_completions = 1\nrio_main_completions = 1\nunrelated_owner_counters_unchanged = true\nservices_balanced = true\nsram_unchanged = true\nplayer_or_persistence_invocations = 0\nlive_gameplay = false\n",
    )?;
    std::env::set_current_dir(old_cwd).map_err(|e| e.to_string())?;
    println!(
        "composition=pass ordinary_calls=8 full_state_pairs=4 production_owners=12 services_balanced=true sram_unchanged=true live=false"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Rio runtime control: {error}");
        std::process::exit(1);
    }
}
