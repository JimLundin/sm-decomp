//! The movie frames of the recorded room seeds, which place the plan's milestone and the played
//! runs, and the rows that follow one another back to back by them.

use std::collections::BTreeMap;
use std::path::Path;

use crate::lock::{Lock, Row, Start};
use oracle_record::room_seeds::RoomSeeds;

/// The movie frame of each seed of `capture`, by its index, from `root`'s table of its seeds.
fn seed_frames(root: &Path, capture: &str) -> Result<BTreeMap<u32, u64>, String> {
    let table = RoomSeeds::read(&root.join(table_path(capture)))?;
    Ok(table
        .seed
        .iter()
        .map(|seed| (seed.room_index, seed.movie_frame))
        .collect())
}

fn table_path(capture: &str) -> String {
    format!("movies/room-seeds-{capture}.toml")
}

/// The movie frame of the seed `wanted` of `capture`.
pub(super) fn seed_frame(root: &Path, capture: &str, wanted: u32) -> Result<u64, String> {
    seed_frames(root, capture)?
        .get(&wanted)
        .copied()
        .ok_or_else(|| format!("{}: missing seed {wanted}", table_path(capture)))
}

/// The movie a capture records: the first word of its name. The lock holds no movie of its
/// own, so this is the naming the README's runs are counted by as well.
pub(super) fn movie(capture: &str) -> Option<&str> {
    capture.split('-').next()
}

/// The movie `row` plays: its seed's capture's or, from power-on, the first word of its id.
pub fn row_movie(row: &Row) -> Option<&str> {
    match &row.start {
        Start::PowerOn => row.id.0.split('-').next(),
        Start::Seed { capture, .. } => movie(capture),
    }
}

/// The movie frame of `row`'s cut 0: frame 0 from power-on, else its seed's.
///
/// # Errors
/// The seed's frame is absent or its table cannot be read.
pub fn first_frame(root: &Path, row: &Row) -> Result<u64, String> {
    match &row.start {
        Start::PowerOn => Ok(0),
        Start::Seed { capture, index } => seed_frame(root, capture, *index),
    }
}

/// The row that follows `row` back to back: the lock's row seeded in the same movie whose cut 0
/// is the frame after `row`'s last cut. It may come from another capture: the new game's
/// arrival ends one capture's seeds, and Ceres's first room begins another's. A row from
/// power-on is followed by the movie's seed at the frame after its last cut. None with no such
/// row.
///
/// # Errors
/// A table of seeds that can't be read, or more than one row that would follow.
pub fn following<'a>(root: &Path, lock: &'a Lock, row: &Row) -> Result<Option<&'a Row>, String> {
    let played = row_movie(row);
    let next = first_frame(root, row)?.saturating_add(row.frames.0);
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
        if movie(own) != played {
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
