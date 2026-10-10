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
    let rom_path = option("--rom").map(PathBuf::from).unwrap_or_else(|| "rom/sm.sfc".into());
    let rom = std::fs::read(&rom_path)?;
    let movie = movie::load(&movie_path)?;
    snes::verify_movie_rom(&movie, &rom)?;
    let fallback = args.iter().any(|arg| arg == "--fallback-only");
    let all = if fallback { &[][..] } else { game::steps::STEP_TABLE };
    let disabled: Vec<serde_json::Value> = option("--disabled-keys")
        .map(|path| {
            std::fs::read(path).map_err(anyhow::Error::from).and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
        })
        .transpose()?
        .unwrap_or_default();
    let filtered: Vec<_> = all
        .iter()
        .copied()
        .filter(|&(address, mx, _, _)| {
            !disabled
                .iter()
                .any(|key| key["address"].as_u64() == Some(address.into()) && key["mx"].as_u64() == Some(mx.into()))
        })
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
        let keys: Vec<serde_json::Value> = serde_json::from_slice(&std::fs::read(keys_path)?)?;
        let keys: Vec<_> = keys
            .iter()
            .map(|key| {
                Ok((
                    key["address"].as_u64().context("key address")? as u32,
                    key["mx"].as_u64().context("key width")? as u8,
                ))
            })
            .collect::<Result<_>>()?;
        let frames = option("--frames").context("discovery --frames required")?.parse()?;
        let mut report = snes::controls::covered_intervals(&mut reference, &mut subject, &movie.frames, frames, &keys)?;
        report["keys"] = serde_json::to_value(
            keys.iter().map(|&(address, mx)| serde_json::json!({"address":address,"mx":mx})).collect::<Vec<_>>(),
        )?;
        report["binary_sha256"] = format!("{:x}", Sha256::digest(std::fs::read(std::env::current_exe()?)?)).into();
        report["identities"] = serde_json::json!({"rom_sha256":format!("{:x}",Sha256::digest(&rom)),"movie_sha256":format!("{:x}",Sha256::digest(std::fs::read(&movie_path)?)),"input_sha256":format!("{:x}",Sha256::digest(movie::write_inputs(&movie.frames).as_bytes()))});
        std::fs::create_dir(&output)?;
        std::fs::write(output.join("discovery.json"), serde_json::to_vec_pretty(&report)?)?;
        std::process::exit(if report["status"] == "pass" { 0 } else { 1 });
    }
    let mut report = if let Some(cache_path) = option("--cache") {
        let identity_path = option("--identities").context("--identities required for validated cache")?;
        let identities: serde_json::Value = serde_json::from_slice(&std::fs::read(identity_path)?)?;
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
        report["warm_replay_seconds"] = began.elapsed().as_secs_f64().into();
        report["cache"] = cache;
        report["cache_pair_sha256"] = pair.pair_sha256.into();
        report["cache_history_sha256"] = pair.history_sha256.into();
        report
    } else {
        snes::focused::compare_from_boundary(&mut reference, &mut subject, &movie.frames, start, skip, count, &output)?
    };
    report["identities"] = serde_json::json!({"rom_sha256":format!("{:x}",Sha256::digest(&rom)),
        "movie_sha256":format!("{:x}",Sha256::digest(std::fs::read(movie_path)?)),
        "input_sha256":format!("{:x}",Sha256::digest(movie::write_inputs(&movie.frames).as_bytes()))});
    report["disabled_keys"] = disabled.into();
    report["binary_sha256"] = format!("{:x}", Sha256::digest(std::fs::read(std::env::current_exe()?)?)).into();
    report["rerun"] = serde_json::to_value(std::env::args().collect::<Vec<_>>())?;
    std::fs::write(output.join("instruction.json"), serde_json::to_vec_pretty(&report)?)?;
    if report["status"] == "divergence" {
        std::process::exit(1);
    }
    Ok(())
}
