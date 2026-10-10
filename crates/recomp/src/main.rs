//! `recomp --rom rom/sm.sfc` → out/generated/game.rs
//!
//! Reads config/functions.toml, builds a CFG per (entry, variant), emits Rust.
//! Functions marked `override = true` are skipped (hand-written in game/src/overrides).

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use recomp::{cfg, config, emit, rom::Rom, step};

struct Args {
    rom: PathBuf,
    functions: PathBuf,
    out: PathBuf,
    steps_out: Option<PathBuf>,
    prospective_pattern: Option<String>,
    selection_profile: Option<PathBuf>,
    selection_entries: Option<PathBuf>,
    selection_inventory: Option<PathBuf>,
    selection_out: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut a = Args {
        rom: "rom/sm.sfc".into(),
        functions: "config/functions.toml".into(),
        out: "out/generated/game.rs".into(),
        steps_out: None,
        prospective_pattern: None,
        selection_profile: None,
        selection_entries: None,
        selection_inventory: None,
        selection_out: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        let v = it.next().with_context(|| format!("{k} needs a value"))?;
        match k.as_str() {
            "--rom" => a.rom = v.into(),
            "--functions" => a.functions = v.into(),
            "--out" => a.out = v.into(),
            "--steps-out" => a.steps_out = Some(v.into()),
            "--prospective-pattern" => a.prospective_pattern = Some(v),
            "--selection-profile" => a.selection_profile = Some(v.into()),
            "--selection-entries" => a.selection_entries = Some(v.into()),
            "--selection-inventory" => a.selection_inventory = Some(v.into()),
            "--selection-out" => a.selection_out = Some(v.into()),
            _ => anyhow::bail!("unknown flag {k}; flags: --rom --functions --out --steps-out"),
        }
    }
    Ok(a)
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let rom = Rom::load(&args.rom)?;
    let funcs: config::Functions = config::load(&args.functions)?;

    if let Some(output) = args.selection_out {
        let report = recomp::selection::select(
            &rom,
            &funcs,
            &args.selection_profile.context("--selection-profile required")?,
            &args.selection_entries.context("--selection-entries required")?,
            &args.selection_inventory.context("--selection-inventory required")?,
        )?;
        std::fs::write(output, serde_json::to_string_pretty(&report)? + "\n")?;
        return Ok(());
    }

    let entries: HashSet<u32> = funcs.functions.iter().map(|f| config::parse_addr(&f.addr)).collect::<Result<_>>()?;

    let read = |a: u32| rom.byte(a);
    let mut built = Vec::new();
    let mut stops = 0usize;
    for f in &funcs.functions {
        if f.override_ {
            continue;
        }
        let entry = config::parse_addr(&f.addr)?;
        for v in &f.variants {
            let (m8, x8) = config::parse_variant(v)?;
            let g = cfg::build(&read, entry, m8, x8, &entries).with_context(|| format!("building ${entry:06X} {v}"))?;
            stops += g.blocks.iter().filter(|b| matches!(b.exit, cfg::Exit::Stop(_))).count();
            built.push((g, f.name.clone()));
        }
    }

    let src = emit::emit_crate(&built)?;
    if let Some(path) = args.steps_out {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        use sha2::{Digest, Sha256};
        let exclusions = funcs
            .excluded_steps
            .iter()
            .map(|row| {
                let (m8, x8) = config::parse_variant(&row.variant)?;
                Ok((config::parse_addr(&row.addr)?, emit::mx_key(m8, x8)))
            })
            .collect::<Result<HashSet<_>>>()?;
        let mut source = step::emit_table_for_proof(&built, &read, &exclusions, args.prospective_pattern.as_deref())?;

        source.push_str(&format!("pub const ROM_SHA256: Option<&str> = Some(\"{:x}\");\n", Sha256::digest(&rom.data)));
        std::fs::write(path, source)?;
    }
    if let Some(dir) = args.out.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&args.out, src)?;
    let blocks: usize = built.iter().map(|(g, _)| g.blocks.len()).sum();
    eprintln!(
        "recomp: {} variants, {} blocks, {} unsupported stops → {}",
        built.len(),
        blocks,
        stops,
        args.out.display()
    );
    Ok(())
}
