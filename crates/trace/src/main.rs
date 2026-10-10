//! trace merge <functions.toml> <misses.csv>...   add every missed (addr, variant)
//! trace names <functions.toml> <symbols.toml>      fill in names from a symbol map
//!
//! misses.csv comes from `Machine::misses_csv()` after a run with the interpreter
//! fallback enabled — every routine the game actually executed that has no
//! generated code yet. That is the discovery step of the loop.
//!
//! symbols.toml is `"808338" = "WaitForNMI"` lines. Build it once from snesrev/sm
//! function names and the bank logs at patrickjohnston.org/bank/.

use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use recomp::config::{self, Function, Functions};

fn save(path: &str, f: &Functions) -> Result<()> {
    let mut fs = f.functions.clone();
    fs.sort_by_key(|f| config::parse_addr(&f.addr).unwrap_or(0));
    for f in &mut fs {
        f.variants.sort();
        f.variants.dedup();
    }
    std::fs::write(
        path,
        toml::to_string_pretty(&Functions { functions: fs, excluded_steps: f.excluded_steps.clone() })?,
    )?;
    Ok(())
}

fn merge(fpath: &str, csvs: &[String]) -> Result<()> {
    let mut funcs: Functions = config::load(fpath.as_ref())?;
    let mut index: BTreeMap<u32, usize> = BTreeMap::new();
    for (i, f) in funcs.functions.iter().enumerate() {
        index.insert(config::parse_addr(&f.addr)?, i);
    }
    let mut added = 0;
    for csv in csvs {
        let text = std::fs::read_to_string(csv).with_context(|| format!("reading {csv}"))?;
        for line in text.lines().skip(1).filter(|l| !l.trim().is_empty()) {
            let mut cols = line.split(',');
            let addr = config::parse_addr(cols.next().context("addr column")?)?;
            let variant = cols.next().context("mx column")?.trim().to_string();
            config::parse_variant(&variant)?;
            match index.get(&addr) {
                Some(&i) => {
                    let f = &mut funcs.functions[i];
                    if !f.variants.contains(&variant) {
                        f.variants.push(variant);
                        added += 1;
                    }
                }
                None => {
                    index.insert(addr, funcs.functions.len());
                    funcs.functions.push(Function {
                        addr: format!("{addr:06X}"),
                        name: None,
                        variants: vec![variant],
                        override_: false,
                    });
                    added += 1;
                }
            }
        }
    }
    save(fpath, &funcs)?;
    eprintln!("trace: added {added} variants; {} functions total", funcs.functions.len());
    Ok(())
}

fn names(fpath: &str, sympath: &str) -> Result<()> {
    let mut funcs: Functions = config::load(fpath.as_ref())?;
    let syms: BTreeMap<String, String> = toml::from_str(&std::fs::read_to_string(sympath)?)?;
    let syms: BTreeMap<u32, String> =
        syms.into_iter().map(|(k, v)| Ok((config::parse_addr(&k)?, v))).collect::<Result<_>>()?;
    let mut n = 0;
    for f in &mut funcs.functions {
        if f.name.is_none()
            && let Some(s) = syms.get(&config::parse_addr(&f.addr)?)
        {
            f.name = Some(s.clone());
            n += 1;
        }
    }
    save(fpath, &funcs)?;
    eprintln!("trace: named {n} functions");
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("merge") if args.len() >= 3 => merge(&args[1], &args[2..]),
        Some("names") if args.len() == 3 => names(&args[1], &args[2]),
        _ => bail!("usage: trace merge <functions.toml> <misses.csv>... | trace names <functions.toml> <symbols.toml>"),
    }
}
