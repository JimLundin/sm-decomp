//! Bounded synthetic active-shot controls, stopping at each update return.
//! Only this diagnostic uses private symbols to inject its fixed fixture. No
//! draw entry or host step runs, and this is not live-from-reset evidence.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_native_host::{Game, HookOutcome, ProjectileEntry, ProjectileServices};
use sm_projectile::{Entry, Host, Outcome};
use std::path::Path;

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(ram[at..at + 2].try_into().unwrap())
}
fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_word(rom: &[u8], address: usize) -> u16 {
    word(rom, ((address >> 16) & 127) * 0x8000 + (address & 0x7fff))
}
struct NoService;
impl Host for NoService {
    fn spawn_plm(&mut self, _: &mut [u8], _: u16) -> Result<u8, String> {
        Err("fixed air/solid fixture unexpectedly requested SpawnPLM".into())
    }
}
struct Services<'a>(&'a mut ProjectileServices);
impl Host for Services<'_> {
    fn spawn_plm(&mut self, ram: &mut [u8], header: u16) -> Result<u8, String> {
        self.0.spawn_plm(ram, header)
    }
}

fn seed(rom: &[u8], matrix: [u16; 6]) -> Vec<u8> {
    let mut ram = vec![0; sm_projectile::WRAM_SIZE];
    let definition = rom_word(rom, 0x9383c1);
    let instruction = rom_word(rom, 0x930000 | usize::from(definition + 6)); // direction 2
    let radius = rom_word(rom, 0x930000 | usize::from(instruction + 4));
    // One ordinary rightward shot in a 16x16 air room with a solid column.
    // Actual ROM instruction/trail records are used without modification.
    for (at, value) in [
        (0x7a5, 16),
        (0x7a7, 16),
        (0x7a9, 1),
        (0x7ab, 1),
        (0x7b9, 512),
        (0x93f, 0x8000),
        (0xb64, 100),
        (0xb78, 100),
        (0xbb4, radius & 255),
        (0xbc8, radius >> 8),
        (0xbdc, 0x400),
        (0xc04, 2),
        (0xc18, 0x8000),
        (0xc2c, rom_word(rom, 0x930000 | usize::from(definition))),
        (0xc40, instruction),
        (0xc54, 1),
        (0xc68, 0xaef3),
        (0xc90, 1),
        (0xcce, 1),
    ] {
        put(&mut ram, at, value);
    }
    for (index, value) in matrix.into_iter().enumerate() {
        put(&mut ram, 0x78 + index * 2, value);
    }
    for row in 0..16 {
        put(&mut ram, 0x10002 + 2 * (row * 16 + 9), 0x8000);
    }
    ram
}

pub fn run(game: &mut Game, library_path: &Path) -> Result<(), String> {
    // SAFETY: retain a second handle to Game's exact already-loaded library.
    // These symbols are used only synchronously below, outside every callback.
    let library = unsafe { Library::new(library_path) }.map_err(|e| e.to_string())?;
    let (commit, update, begin): (unsafe extern "C" fn(*const u8), unsafe extern "C" fn(), unsafe extern "C" fn()) = unsafe {
        (
            *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
            *library.get(b"HandleProjectile\0").map_err(|e| e.to_string())?,
            *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
        )
    };
    let rom = game.rom().to_vec();
    let matrices = [
        ("identity", [0x100, 0, 0, 0x7777, 60, 80]),
        ("signed-fraction", [0xff81, 0x101, 0xfe83, 0x1234, 61, 83]),
        ("wrapping", [0x7fff, 0x8000, 0x7ffe, 0x4321, 0x8001, 0x7fff]),
        ("negative-products", [0x8000, 0xffff, 0x8001, 0, 0xfffd, 0xfffb]),
    ];
    let mut total_returns = 0;
    for (name, matrix) in matrices {
        let mut input = seed(&rom, matrix);
        let mut saw_trail = false;
        let mut saw_flight = false;
        let mut saw_instruction = false;
        let mut saw_impact = false;
        let mut saw_deleted = false;
        for tick in 1..=64 {
            // Validate every inherited fixture through the safe Rust boundary
            // before invoking actual C. This also rules out foreign services.
            let mut checked = input.clone();
            if sm_projectile::run(Entry::HandleProjectile, &mut checked, &rom, &mut NoService)? != Outcome::Handled {
                return Err(format!("{name}: fixture update refused before C invocation"));
            }
            game.clear_projectile_hook();
            // SAFETY: the fixed/inherited full WRAM fixture is validated above;
            // no native callback is active. Rendering is never invoked.
            unsafe {
                commit(input.as_ptr());
                begin();
                update();
            }
            let reference = game.wram().to_vec();
            let effects = game.effects().to_vec();
            game.set_projectile_hook(|entry, ram, rom, services| {
                if entry != ProjectileEntry::HandleProjectile {
                    return Err("unexpected root in update-return fixture".into());
                }
                Ok(match sm_projectile::run(Entry::HandleProjectile, ram, rom, &mut Services(services))? {
                    Outcome::Handled => HookOutcome::Handled,
                    Outcome::Refused => HookOutcome::Refused,
                })
            })?;
            unsafe {
                commit(input.as_ptr());
                begin();
                update();
            }
            if let Some(at) = reference.iter().zip(game.wram()).position(|(c, r)| c != r) {
                return Err(format!(
                    "{name} tick={tick} WRAM {at:05x}: C={:02x} Rust={:02x}",
                    reference[at],
                    game.wram()[at]
                ));
            }
            if game.wram() != checked || game.effects() != effects {
                return Err(format!("{name} tick={tick}: transaction/effect mismatch"));
            }
            saw_flight |= word(&reference, 0xb64) != word(&input, 0xb64) && word(&reference, 0xc18) == 0x8000;
            saw_instruction |= word(&reference, 0xc40) != word(&input, 0xc40) && word(&reference, 0xc40) != 0;
            saw_impact |= word(&reference, 0xc18) == 0x8700;
            saw_deleted |= word(&reference, 0xc40) == 0;
            if tick == 1 {
                let slot = 34;
                if word(&reference, 0xd658 + slot) != 1
                    || word(&reference, 0xd67c + slot) != 1
                    || word(&reference, 0xd6a0 + slot) == 0
                    || word(&reference, 0xd6c4 + slot) == 0
                {
                    return Err(format!("{name}: first tick did not allocate both trail sides"));
                }
                saw_trail = true;
                println!(
                    "fixture={name} trail_tick=1 left=({:04x},{:04x}) right=({:04x},{:04x}) lists=({:04x},{:04x})",
                    word(&reference, 0xd730 + slot),
                    word(&reference, 0xd778 + slot),
                    word(&reference, 0xd754 + slot),
                    word(&reference, 0xd79c + slot),
                    word(&reference, 0xd6a0 + slot),
                    word(&reference, 0xd6c4 + slot)
                );
            }
            println!(
                "fixture={name} tick={tick} full_wram_sha256={:x} effects_sha256={:x} x={} type={:04x} instruction={:04x} timer={}",
                Sha256::digest(&reference),
                Sha256::digest(&effects),
                word(&reference, 0xb64),
                word(&reference, 0xc18),
                word(&reference, 0xc40),
                word(&reference, 0xc54)
            );
            total_returns += 1;
            input = reference;
            if saw_deleted {
                break;
            }
        }
        if !(saw_trail && saw_flight && saw_instruction && saw_impact && saw_deleted) {
            return Err(format!(
                "{name}: incomplete lifetime trail={saw_trail} flight={saw_flight} instruction={saw_instruction} impact={saw_impact} deleted={saw_deleted}"
            ));
        }
    }
    println!(
        "escape-update: four synthetic fixtures passed; update_returns={total_returns}; full_wram_bytes=131072; rendering_calls=0; live_from_reset=false"
    );
    let counters = game.projectile_counters();
    assert_eq!(counters.c_entries, [0, total_returns, 0, 0]);
    assert_eq!(counters.rust_completed, [0, total_returns, 0, 0]);
    assert_eq!(counters.ceres_high_bit_entries, [0, 2 * total_returns, 0, 0]);
    assert_eq!(counters.ceres_high_bit_completed, [0, total_returns, 0, 0]);
    assert_eq!(counters.rust_refused, [0; 4]);
    assert_eq!(counters.ceres_high_bit_refused, [0; 4]);
    assert_eq!(counters.other_refused, [0; 4]);
    assert_eq!(counters.foreign_calls, 0);
    println!("counters={counters:?}");
    Ok(())
}
