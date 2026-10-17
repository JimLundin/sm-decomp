//! Cold replay with complete per-instruction CPU/pending-state/effect comparison.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() >= 2,
        "usage: focused-replay movie --out directory [--start-input N --instructions N --rom path --fallback-only]"
    );
    let movie_path = PathBuf::from(&args[0]);
    let option = |name: &str| args.iter().position(|arg| arg == name).and_then(|index| args.get(index + 1));
    let output = PathBuf::from(option("--out").context("--out required")?);
    let start = option("--start-input").map(|v| v.parse()).transpose()?.unwrap_or(0);
    let count = option("--instructions").map(|v| v.parse()).transpose()?.unwrap_or(100);
    let skip = option("--start-instruction").map(|v| v.parse()).transpose()?.unwrap_or(0);
    let until_frame_end = args.iter().any(|arg| arg == "--until-frame-end");
    ensure!(
        !until_frame_end || (skip == 0 && option("--cache").is_none()),
        "failed-frame localization requires a cold frame boundary without skipped instructions"
    );
    let rom_path = option("--rom").map(PathBuf::from).unwrap_or_else(|| "rom/sm.sfc".into());
    let rom = std::fs::read(&rom_path)?;
    let movie = movie::load(&movie_path)?;
    snes::verify_movie_rom(&movie, &rom)?;
    let fallback = args.iter().any(|arg| arg == "--fallback-only");
    let all = if fallback { &[][..] } else { game::steps::STEP_TABLE };
    #[derive(serde::Deserialize)]
    struct Keys {
        keys: Vec<snes::focused::Key>,
    }
    let disabled = option("--disabled-keys")
        .map(|path| -> Result<_> {
            let keys: Keys = toml::from_str(&std::fs::read_to_string(path)?)?;
            Ok(keys.keys)
        })
        .transpose()?
        .unwrap_or_default();
    let filtered: Vec<_> = all
        .iter()
        .copied()
        .filter(|&(address, mx, _, _)| !disabled.iter().any(|key| key.address == address && key.mx == mx))
        .collect();
    let table = filtered.as_slice();
    if !fallback {
        ensure!(!all.is_empty(), "focused compiled replay requires nonempty table");
        let identity = movie::Movie { frames: Vec::new(), rom_sha256: game::steps::ROM_SHA256.map(str::to_owned) };
        ensure!(identity.rom_sha256.is_some(), "compiled table has no ROM identity");
        snes::verify_movie_rom(&identity, &rom)?;
    }
    let mut reference = snes::Reference::new(&rom, 0)?;
    let mut subject = snes::Subject::with_steps(&rom, 0, table)?;
    if let Some(keys_path) = option("--discover-keys") {
        let keys: Keys = toml::from_str(&std::fs::read_to_string(keys_path)?)?;
        let keys = keys.keys.iter().map(|key| (key.address, key.mx)).collect::<Vec<_>>();
        let frames = option("--frames").context("discovery --frames required")?.parse()?;
        let mut report = snes::controls::covered_intervals(&mut reference, &mut subject, &movie.frames, frames, &keys)?;
        report.keys = keys.iter().map(|&(address, mx)| snes::focused::Key { address, mx }).collect();
        report.binary_sha256 = Some(format!("{:x}", Sha256::digest(std::fs::read(std::env::current_exe()?)?)));
        report.identities = Some(snes::focused::Identities {
            rom_sha256: format!("{:x}", Sha256::digest(&rom)),
            movie_sha256: format!("{:x}", Sha256::digest(std::fs::read(&movie_path)?)),
            input_sha256: format!("{:x}", Sha256::digest(movie::write_inputs(&movie.frames).as_bytes())),
        });
        std::fs::create_dir(&output)?;
        std::fs::write(output.join("discovery.toml"), toml::to_string_pretty(&report)?)?;
        std::process::exit(if report.status == "pass" { 0 } else { 1 });
    }
    let mut report = if until_frame_end {
        snes::focused::compare_frame_from_boundary(&mut reference, &mut subject, &movie.frames, start, &output)?
    } else if let Some(cache_path) = option("--cache") {
        let identity_path = option("--identities").context("--identities required for validated cache")?;
        let identities: pipeline_model::CheckpointIdentities = pipeline_model::read(&PathBuf::from(identity_path))?;
        let (pair, cache) = snes::paired::prepare_cache(
            &rom,
            0,
            table,
            &movie.frames,
            (start, skip),
            identities.clone(),
            &PathBuf::from(cache_path),
        )?;
        snes::paired::restore(&pair, &pair.identities, &mut reference, &mut subject)?;
        let began = std::time::Instant::now();
        let mut report = snes::focused::compare_restored(
            &mut reference,
            &mut subject,
            &movie.frames,
            (pair.input_index, start, skip),
            count,
            &output,
            "warm",
        )?;
        report.warm_replay_seconds = Some(began.elapsed().as_secs_f64());
        report.cache = Some(cache);
        report.cache_pair_sha256 = Some(pair.pair_sha256);
        report.cache_history_sha256 = Some(pair.history_sha256);
        report
    } else {
        snes::focused::compare_from_boundary(&mut reference, &mut subject, &movie.frames, start, skip, count, &output)?
    };
    report.identities = Some(snes::focused::Identities {
        rom_sha256: format!("{:x}", Sha256::digest(&rom)),
        movie_sha256: format!("{:x}", Sha256::digest(std::fs::read(movie_path)?)),
        input_sha256: format!("{:x}", Sha256::digest(movie::write_inputs(&movie.frames).as_bytes())),
    });
    report.disabled_keys = disabled;
    report.binary_sha256 = Some(format!("{:x}", Sha256::digest(std::fs::read(std::env::current_exe()?)?)));
    report.rerun = std::env::args().collect();
    pipeline_model::write(&output.join("instruction.toml"), &report)?;
    if report.status == "divergence" {
        std::process::exit(1);
    }
    if report.status != "pass" || !report.complete {
        std::process::exit(2);
    }
    Ok(())
}
