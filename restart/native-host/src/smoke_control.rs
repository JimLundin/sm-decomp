//! Real native smoke assertions shared by the example and dedicated DLL child.
use crate::{Game, prepare};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Run once in a dedicated process; native Game has process-lifetime state.
pub fn run(archive: &Path, rom: &Path, out: &Path) -> Result<(), String> {
    let archive = std::fs::canonicalize(archive).map_err(|error| error.to_string())?;
    let rom = std::fs::canonicalize(rom).map_err(|error| error.to_string())?;
    let out = if out.is_absolute() {
        out.to_path_buf()
    } else {
        std::env::current_dir().map_err(|error| error.to_string())?.join(out)
    };
    let prepared = prepare(&archive, &out)?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    // This control owns a dedicated process; keep native save output inside its
    // retained run rather than relying on a saves directory in the caller CWD.
    let run_dir = out.join(format!("smoke-{}", std::process::id()));
    std::fs::create_dir(&run_dir).map_err(|error| error.to_string())?;
    std::fs::create_dir(run_dir.join("saves")).map_err(|error| error.to_string())?;
    std::env::set_current_dir(&run_dir).map_err(|error| error.to_string())?;
    let mut game = Game::boot(&prepared.library, &rom)?;
    println!("boot_rom={}", game.patched_rom_sha256());
    // Reset-logo waits are distinct from consumed controller polls and updates.
    for _ in 0..600 {
        let state = game.host_step(0)?;
        if state.host_steps % 100 == 0 {
            println!("{state:?}");
        }
    }
    println!("current_rom={}", game.patched_rom_sha256());
    assert!(game.state().polls > 0);
    assert!(game.state().updates > 0);
    assert_eq!(game.audio().len(), 1068);
    assert_eq!(game.pixels().len(), 256 * 240);
    assert!(game.pixels().iter().any(|pixel| pixel & 0xffffff != 0));
    println!("visible_pixels={}", game.pixels().iter().filter(|pixel| **pixel & 0xffffff != 0).count());
    match Game::boot(&prepared.library, &rom) {
        Err(error) => assert_eq!(error, "only one native Game may boot per process"),
        Ok(_) => panic!("a second native Game unexpectedly booted"),
    }
    // Literal masks distinguish fresh presses, held inputs and releases.
    for (buttons, expected_new) in [(1_u16, 0x8000), (1, 0), (0, 0), (2, 0x4000), (0, 0), (0x30, 0), (0xc0, 0), (0, 0)]
    {
        let polls_before = game.state().polls;
        let mut observed = game.host_step(buttons)?;
        for _ in 0..10 {
            if observed.polls > polls_before {
                break;
            }
            observed = game.host_step(buttons)?;
        }
        assert_eq!(observed.polls, polls_before + 1, "exactly the first consumed poll is required");
        let effective = if buttons == 0x30 || buttons == 0xc0 { 0 } else { buttons };
        assert_eq!(observed.effective_buttons, effective);
        assert_eq!(observed.joy1_current, effective.reverse_bits());
        assert_eq!(observed.joy1_new, expected_new, "fresh edge for host buttons {buttons:#06x}");
        println!("control {observed:?}");
    }
    assert!(game.observation_bytes().starts_with(b"SMNOBS02"));
    assert_eq!(game.effects().len() % 16, 0);
    println!(
        "observation_bytes={} sha256={:x}",
        game.observation_bytes().len(),
        Sha256::digest(game.observation_bytes())
    );
    println!("effects_bytes={} sha256={:x}", game.effects().len(), Sha256::digest(game.effects()));
    println!("projectile_counters={:?}", game.projectile_counters());
    let state = game.state();
    let wram = game.wram().to_vec();
    let effects = game.effects().to_vec();
    let audio = game.audio().to_vec();
    let pixels = game.pixels().to_vec();
    let observation = game.observation_bytes().to_vec();
    for invalid in [0x1000, 0x8000, 0xffff] {
        assert_eq!(game.host_step(invalid).unwrap_err(), "native buttons use only bits 0..11");
        assert_eq!(game.state(), state);
        assert_eq!(game.wram(), wram);
        assert_eq!(game.effects(), effects);
        assert_eq!(game.audio(), audio);
        assert_eq!(game.pixels(), pixels);
        assert_eq!(game.observation_bytes(), observation);
    }
    std::fs::write(run_dir.join("final.wram"), &wram).map_err(|error| error.to_string())?;
    std::fs::write(run_dir.join("effects.bin"), &effects).map_err(|error| error.to_string())?;
    std::fs::write(run_dir.join("observations.bin"), &observation).map_err(|error| error.to_string())?;
    std::fs::write(run_dir.join("audio.bin"), audio.iter().flat_map(|sample| sample.to_le_bytes()).collect::<Vec<_>>())
        .map_err(|error| error.to_string())?;
    std::fs::write(run_dir.join("pixels.bin"), pixels.iter().flat_map(|pixel| pixel.to_le_bytes()).collect::<Vec<_>>())
        .map_err(|error| error.to_string())?;
    Ok(())
}
