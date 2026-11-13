//! CI's complete corpus preflight. Local harness runs may use a partial cache.
//!
//! Opening every committed row checks its pins and files; byte ranges reject truncation.
//! The replay checks record decoding and behaviour after this preflight succeeds.

use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::Path;

use crate::cache::Cache;
use crate::lock::{Lock, RowId};

fn required_rows<'a>(floors: &'a str, free_floors: &'a str) -> Result<BTreeSet<&'a str>, String> {
    let mut rows = BTreeSet::new();
    for (text, domain_fields) in [(floors, 3), (free_floors, 9)] {
        let mut file_rows = BTreeSet::new();
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<_> = line.split_whitespace().collect();
            let expected = if domain_fields == 9 && fields.first() == Some(&"uncompared") {
                3
            } else {
                domain_fields
            };
            if fields.len() != expected
                || fields[2..]
                    .iter()
                    .any(|field| field.parse::<u64>().is_err())
            {
                return Err(format!("malformed floor line {}", number.saturating_add(1)));
            }
            file_rows.insert(fields[1]);
        }
        if file_rows.is_empty() {
            return Err("floor file has no required rows".to_owned());
        }
        rows.extend(file_rows);
    }
    Ok(rows)
}

fn preflight(cache: &Path, lock: &Lock, required: &BTreeSet<&str>) -> Result<usize, String> {
    if required.is_empty() {
        return Err("corpus has no required rows".to_owned());
    }
    for &id in required {
        let row = lock
            .row(&RowId(id.to_owned()))
            .ok_or_else(|| format!("{id}: required row is not in the lock"))?;
        let opened = Cache::open(cache, row).map_err(|error| format!("{id}: {error}"))?;
        for entry in opened.entries() {
            if entry
                .offset
                .checked_add(entry.length)
                .is_none_or(|end| end > opened.bytes())
            {
                return Err(format!("{id}: frame {} outside the records", entry.frame.0));
            }
        }
    }
    Ok(required.len())
}

#[test]
fn committed_corpus_is_complete_before_replay() {
    let Some(cache) = std::env::var_os("SM_CACHE") else {
        let _ = writeln!(
            std::io::stdout(),
            "skipped: corpus preflight SM_CACHE unset"
        );
        return;
    };
    let required = required_rows(include_str!("floors.txt"), include_str!("free_floors.txt"))
        .unwrap_or_else(|error| panic!("{error}"));
    let lock = Lock::parse(include_str!("../../../../scenarios/locks/lsnes.toml"))
        .unwrap_or_else(|error| panic!("{error:?}"));
    let count = preflight(Path::new(&cache), &lock, &required)
        .unwrap_or_else(|error| panic!("corpus preflight: {error}"));
    let _ = writeln!(
        std::io::stdout(),
        "corpus preflight holds: {count} required rows"
    );
}

#[test]
fn required_rows_include_both_floor_files_and_uncompared_rows() {
    assert_eq!(
        required_rows(
            "clock a 1\nclock b 2\n",
            "clock b 1 1 1 0 0 0 0\nuncompared c 0\n"
        ),
        Ok(BTreeSet::from(["a", "b", "c"]))
    );
    for (floors, free) in [
        ("# empty", "uncompared c 0"),
        ("clock a 1", "# empty"),
        ("clock a", "uncompared c 0"),
        ("clock a nope", "uncompared c 0"),
        ("clock a 1", "clock c 1 2"),
    ] {
        assert!(required_rows(floors, free).is_err());
    }
}

#[test]
fn required_rows_must_open_and_their_record_ranges_must_fit() {
    let mut lock = Lock::parse(include_str!("../../../../scenarios/locks/lsnes.toml"))
        .unwrap_or_else(|error| panic!("{error:?}"));
    let row = &mut lock.rows[0];
    row.frames = crate::lock::FrameCount(1);
    row.seed_record = None;
    row.seed_words = None;
    let id = row.id.0.clone();
    let root = std::env::temp_dir().join(format!("sm-corpus-preflight-{}", std::process::id()));
    let directory = root.join(&id);
    std::fs::create_dir_all(&directory).unwrap_or_else(|error| panic!("{error}"));
    let required = BTreeSet::from([id.as_str()]);
    assert!(preflight(&root, &lock, &required).is_err());
    let mut records = Vec::new();
    let record = super::tests::record(0, 0, [0; 4]);
    oracle_record::codec::encode(
        &record,
        oracle_record::codec::PictureMode::Keep,
        &mut records,
    );
    let index = format!(
        "{}\nrow {id}\nseed_record -\nseed_words -\npads {}\nframes 1\npictures yes\n# frame offset length cut\n0 0 {} R\n",
        crate::cache::HEAD,
        lock.rows[0].pads,
        records.len()
    );
    std::fs::write(directory.join("index.txt"), &index).unwrap_or_else(|error| panic!("{error}"));
    assert!(preflight(&root, &lock, &required).is_err());
    std::fs::write(directory.join("records.bin"), &records)
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(preflight(&root, &lock, &required), Ok(1));
    std::fs::write(directory.join("records.bin"), []).unwrap_or_else(|error| panic!("{error}"));
    assert!(preflight(&root, &lock, &required).is_err());
    std::fs::write(directory.join("index.txt"), "invalid index\n")
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(preflight(&root, &lock, &required).is_err());
    assert!(preflight(&root, &lock, &BTreeSet::new()).is_err());
    std::fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("{error}"));
}
