//! The movie frames of the recorded room seeds, which place the plan's milestone and rungs, and
//! the rows that follow one another back to back by them.

use std::collections::BTreeMap;
use std::path::Path;

use crate::lock::{Lock, Row, Start};

/// The movie frame of each seed of `capture`, by its index, from `root`'s table of its seeds.
fn seed_frames(root: &Path, capture: &str) -> Result<BTreeMap<u32, u64>, String> {
    let path = format!("movies/room-seeds-{capture}.csv");
    let text =
        std::fs::read_to_string(root.join(&path)).map_err(|_| format!("{path}: unavailable"))?;
    let mut lines = text.lines();
    let header: Vec<_> = lines.next().unwrap_or_default().split(',').collect();
    let column = |name: &str| {
        header
            .iter()
            .position(|field| field.trim() == name)
            .ok_or_else(|| format!("{path}: missing column {name}"))
    };
    let (id_column, frame_column) = (column("room_index")?, column("movie_frame")?);
    let mut frames = BTreeMap::new();
    for (line_number, line) in lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
    {
        let fields: Vec<_> = line.split(',').map(str::trim).collect();
        let invalid = || {
            format!(
                "{path}: invalid seed metadata at row {}",
                line_number.saturating_add(2)
            )
        };
        let id: u32 = fields
            .get(id_column)
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?;
        let frame: u64 = fields
            .get(frame_column)
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?;
        if frames.insert(id, frame).is_some() {
            return Err(format!("{path}: duplicate seed {id}"));
        }
    }
    Ok(frames)
}

/// The movie frame of the seed `wanted` of `capture`.
pub(super) fn seed_frame(root: &Path, capture: &str, wanted: u32) -> Result<u64, String> {
    seed_frames(root, capture)?
        .get(&wanted)
        .copied()
        .ok_or_else(|| format!("movies/room-seeds-{capture}.csv: missing seed {wanted}"))
}

/// The movie a capture records: the first word of its name. The lock holds no movie of its
/// own, so this is the naming the README's runs are counted by as well.
pub(super) fn movie(capture: &str) -> Option<&str> {
    capture.split('-').next()
}

/// The row that follows `row` back to back: the lock's row seeded in the same movie whose cut 0
/// is the frame after `row`'s last cut. It may come from another capture: the new game's
/// arrival ends one capture's seeds, and Ceres's first room begins another's. None for a row
/// from power-on or with no such row.
///
/// # Errors
/// A table of seeds that can't be read, or more than one row that would follow.
pub fn following<'a>(root: &Path, lock: &'a Lock, row: &Row) -> Result<Option<&'a Row>, String> {
    let Start::Seed { capture, index } = &row.start else {
        return Ok(None);
    };
    let next = seed_frame(root, capture, *index)?.saturating_add(row.frames.0);
    let mut tables = BTreeMap::new();
    let mut found: Option<&'a Row> = None;
    for candidate in &lock.rows {
        let Start::Seed {
            capture: own,
            index,
        } = &candidate.start
        else {
            continue;
        };
        if movie(own) != movie(capture) {
            continue;
        }
        if !tables.contains_key(own.as_str()) {
            tables.insert(own.as_str(), seed_frames(root, own)?);
        }
        if tables
            .get(own.as_str())
            .and_then(|frames| frames.get(index))
            != Some(&next)
        {
            continue;
        }
        if let Some(first) = found.replace(candidate) {
            return Err(format!(
                "two rows follow {}: {} and {}",
                row.id, first.id, candidate.id
            ));
        }
    }
    Ok(found)
}
