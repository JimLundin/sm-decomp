//! Explicit text inspection/import; verification only opens binary fixtures.
use anyhow::{Context, Result};
use offline_verification::fixtures::{Case, Manifest, read, write};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) fn run(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("inspect") if args.len() == 2 || args.len() == 3 => {
            let manifest = read(Path::new(&args[1]))?;
            let text = if let Some(name) = args.get(2) {
                let case = manifest
                    .cases
                    .iter()
                    .chain(manifest.routines.iter().flat_map(|r| &r.cases))
                    .find(|case| &case.name == name)
                    .context("unknown fixture case")?;
                toml::to_string_pretty(case)?
            } else {
                toml::to_string_pretty(&manifest)?
            };
            use std::io::Write as _;
            std::io::stdout().lock().write_all(text.as_bytes())?;
            Ok(())
        }
        Some("import") if args.len() == 3 => {
            let manifest: Manifest = toml::from_str(&fs::read_to_string(&args[1])?)?;
            write(Path::new(&args[2]), &manifest)
        }
        Some("diff") if args.len() == 3 => {
            let mut before = read(Path::new(&args[1]))?;
            let mut after = read(Path::new(&args[2]))?;
            let cases = |manifest: &Manifest| -> BTreeMap<String, Case> {
                manifest
                    .cases
                    .iter()
                    .map(|c| (format!("root/{}", c.name), c.clone()))
                    .chain(
                        manifest
                            .routines
                            .iter()
                            .flat_map(|r| r.cases.iter().map(|c| (format!("{}/{}", r.symbol, c.name), c.clone()))),
                    )
                    .collect()
            };
            let old = cases(&before);
            let new = cases(&after);
            let order = |manifest: &Manifest| -> Vec<_> {
                manifest
                    .cases
                    .iter()
                    .map(|case| format!("root/{}", case.name))
                    .chain(manifest.routines.iter().flat_map(|routine| {
                        routine.cases.iter().map(|case| format!("{}/{}", routine.symbol, case.name))
                    }))
                    .collect()
            };
            let order_changed = order(&before) != order(&after);
            for (name, case) in &old {
                match new.get(name) {
                    None => println!("removed {name}"),
                    Some(next) if next != case => println!("changed {name}"),
                    _ => (),
                }
            }
            for name in new.keys().filter(|name| !old.contains_key(*name)) {
                println!("added {name}");
            }
            for manifest in [&mut before, &mut after] {
                manifest.cases.clear();
                for routine in &mut manifest.routines {
                    routine.cases.clear();
                }
            }
            if before != after || order_changed {
                println!("changed family/routine metadata or case order");
            }
            Ok(())
        }
        _ => anyhow::bail!(
            "usage: fixture inspect FAMILY.smf [CASE] | fixture import EDIT.toml FAMILY.smf | fixture diff BEFORE.smf AFTER.smf"
        ),
    }
}
