//! The movie frames of the recorded room seeds, which place the plan's milestone and the played
//! runs, and the rows that follow one another back to back by them.

use std::collections::BTreeMap;
use std::path::Path;

use crate::lock::{Lock, Row, Start};
use oracle_record::room_seeds::RoomSeeds;

fn table_path(capture: &str) -> String {
    format!("movies/room-seeds-{capture}.toml")
}

/// The movie frames of the seeds under a repository root, each capture's table read the first
/// time one of its seeds is asked for.
pub struct Seeds<'a> {
    root: &'a Path,
    /// The movie frame of each seed read so far, by capture and then by index.
    frames: BTreeMap<String, BTreeMap<u32, u64>>,
}

impl<'a> Seeds<'a> {
    pub const fn new(root: &'a Path) -> Self {
        Self {
            root,
            frames: BTreeMap::new(),
        }
    }

    /// The movie frame of each seed of `capture`, by its index.
    fn of(&mut self, capture: &str) -> Result<&BTreeMap<u32, u64>, String> {
        if !self.frames.contains_key(capture) {
            let table = RoomSeeds::read(&self.root.join(table_path(capture)))?;
            let frames = table
                .seed
                .iter()
                .map(|seed| (seed.room_index, seed.movie_frame))
                .collect();
            self.frames.insert(capture.to_owned(), frames);
        }
        self.frames
            .get(capture)
            .ok_or_else(|| format!("{}: not read", table_path(capture)))
    }

    /// The movie frame of the seed `wanted` of `capture`.
    ///
    /// # Errors
    /// The seed is absent or its table cannot be read.
    pub fn frame(&mut self, capture: &str, wanted: u32) -> Result<u64, String> {
        self.of(capture)?
            .get(&wanted)
            .copied()
            .ok_or_else(|| format!("{}: missing seed {wanted}", table_path(capture)))
    }

    /// The movie frame of `row`'s cut 0: frame 0 from power-on, else its seed's.
    ///
    /// # Errors
    /// The seed's frame is absent or its table cannot be read.
    pub fn first_frame(&mut self, row: &Row) -> Result<u64, String> {
        match &row.start {
            Start::PowerOn => Ok(0),
            Start::Seed { capture, index } => self.frame(capture, *index),
        }
    }

    /// The row that follows `row` back to back: the lock's row seeded in the same movie whose
    /// cut 0 is the frame after `row`'s last cut. It may come from another capture: the new
    /// game's arrival ends one capture's seeds, and Ceres's first room begins another's. A row
    /// from power-on is followed by the movie's seed at the frame after its last cut. None with
    /// no such row.
    ///
    /// # Errors
    /// A table of seeds that can't be read, or more than one row that would follow.
    pub fn following<'l>(&mut self, lock: &'l Lock, row: &Row) -> Result<Option<&'l Row>, String> {
        let played = row_movie(row);
        let next = self.first_frame(row)?.saturating_add(row.frames.0);
        let mut found: Option<&'l Row> = None;
        for candidate in &lock.rows {
            let Start::Seed {
                capture: own,
                index,
            } = &candidate.start
            else {
                continue;
            };
            if movie(own) != played || self.of(own)?.get(index) != Some(&next) {
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
