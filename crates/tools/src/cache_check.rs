//! Verify that a lean capture can be reused with its pinned inputs and requested watches.

use std::path::Path;

use verify::cache::Cache;
use verify::lock::{Lock, RowId, Sha256, Start};

pub const USAGE: &str = "tools cache-check LOCK [BUNDLE CACHE ROW [[--carry-watches] WATCH_PCS]]";

fn read_lock(path: &str) -> Result<Lock, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    Lock::parse(&text).map_err(|error| format!("{path}: {error:?}"))
}

fn require_pin(path: &Path, expected: Sha256) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if Sha256::of(&bytes) != expected {
        return Err(format!("{}: does not match the lock", path.display()));
    }
    Ok(())
}

pub fn run(args: &[String]) -> Result<String, String> {
    if let [lock_path] = args {
        return read_lock(lock_path).map(|lock| format!("{}\n", lock.rows.len()));
    }
    let [lock_path, bundle, root, id, options @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let (requested, carry_watches) = match options {
        [] => ("", false),
        [list] => (list.as_str(), false),
        [option, list] if option == "--carry-watches" => (list.as_str(), true),
        _ => return Err(USAGE.to_owned()),
    };
    let lock = read_lock(lock_path)?;
    let row = lock
        .row(&RowId(id.clone()))
        .ok_or_else(|| format!("{id}: not in the lock"))?;
    let bundle = Path::new(bundle);
    require_pin(&bundle.join("pads").join(format!("{id}.pads")), row.pads)?;
    if let Start::Seed { capture, index } = &row.start {
        let state = bundle
            .join("seeds")
            .join(capture)
            .join(format!("{index:04}.lss"));
        let record = row
            .seed_record
            .ok_or_else(|| format!("{id}: seeded start has no seed_record pin"))?;
        require_pin(&state, record)?;
        let words = state.with_extension("words");
        if let Some(pin) = row.seed_words {
            require_pin(&words, pin)?;
        } else if words.exists() {
            return Err(format!("{}: missing seed_words pin", words.display()));
        }
    }

    let root = Path::new(root);
    if !root.join(id).join("lean.txt").is_file() {
        return Err(format!("{id}: no lean cache index"));
    }
    let invalid = |error| format!("{id}: {error}");
    let cache = Cache::open(root, row).map_err(invalid)?;
    if cache.pictures() {
        return Err(format!("{id}: expected a lean cache"));
    }
    let mut captured = cache.watched().to_vec();
    if !requested.is_empty() {
        for address in requested.split(',') {
            let pc = u32::from_str_radix(
                address.trim_start_matches('$').replace(':', "").as_str(),
                16,
            )
            .map_err(|_| format!("invalid watch address: {address}"))?;
            if !captured.contains(&pc) {
                if !carry_watches {
                    return Err(format!("{id}: cache did not watch {pc:06x}"));
                }
                captured.push(pc);
            }
        }
    }
    let mut end = 0;
    for (index, entry) in cache.entries().iter().enumerate() {
        if u64::try_from(index).ok() != Some(entry.frame.0) || entry.offset != end {
            return Err(format!("{id}: out of order cache index"));
        }
        end = entry
            .offset
            .checked_add(entry.length)
            .filter(|&end| end <= cache.bytes())
            .ok_or_else(|| format!("{id}: incomplete lean cache"))?;
    }
    if end != cache.bytes() {
        return Err(format!("{id}: unindexed cache bytes"));
    }
    for record in cache.records_without_pictures() {
        record.map_err(invalid)?;
    }
    if carry_watches {
        let addresses: Vec<_> = captured.iter().map(|pc| format!("{pc:06x}")).collect();
        return Ok(format!("{}\n", addresses.join(",")));
    }
    Ok(format!("{}\n", row.frames.0))
}
