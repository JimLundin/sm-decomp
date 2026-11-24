//! The lock rows a run plays through, and their records.

use crate::cache::Cache;
use crate::lock::{Lock, Row, RowId, Start};
use crate::replay::readers::record;
use crate::replay::{data_failure, result::Outcome};
use oracle_record::FrameRecord;
use std::path::Path;

/// The lock rows a run plays through: its own, then each row that continues it, reached as the
/// run reaches it. A row continues another when both start from one capture's seeds, its own
/// the next, so its cut 0 is the video frame after the other's last cut; the run's cuts count
/// on through it.
struct Chain<'a> {
    lock: &'a Lock,
    /// Each row reached so far, with the run's cut its own cut 0 is.
    reached: Vec<(u64, &'a Row)>,
}

impl<'a> Chain<'a> {
    fn new(lock: &'a Lock, row: &str) -> Result<Self, Outcome> {
        let row = lock
            .row(&RowId(row.to_owned()))
            .ok_or_else(|| data_failure(None, "played lock row"))?;
        Ok(Self {
            lock,
            reached: vec![(0, row)],
        })
    }

    /// Which reached row holds the run's `cut`, by its place in the chain, and its own cut
    /// there, reaching the rows that continue the run up to it.
    fn place(&mut self, cut: u64) -> Result<(usize, u64), Outcome> {
        loop {
            let place = self.reached.len().saturating_sub(1);
            let &(first, row) = self
                .reached
                .last()
                .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
            let next = first.saturating_add(row.frames.0);
            if cut < next {
                let own = cut
                    .checked_sub(first)
                    .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
                return Ok((place, own));
            }
            let following = self
                .following(row)
                .ok_or_else(|| data_failure(Some(cut), "played row continuing the last row"))?;
            self.reached.push((next, following));
        }
    }

    /// The row whose seed is the next of `row`'s capture.
    fn following(&self, row: &Row) -> Option<&'a Row> {
        let Start::Seed { capture, index } = &row.start else {
            return None;
        };
        let next = index.checked_add(1)?;
        self.lock.rows.iter().find(|candidate| {
            matches!(&candidate.start, Start::Seed { capture: own, index: at } if own == capture && *at == next)
        })
    }
}

/// The records of a run's chain of rows, each row's cache opened as the run reaches it.
pub(super) struct Rows<'a> {
    chain: Chain<'a>,
    cache: &'a Path,
    /// The caches of the rows reached, in the chain's order.
    opened: Vec<Cache>,
}

impl<'a> Rows<'a> {
    pub(super) fn open(lock: &'a Lock, cache: &'a Path, row: &str) -> Result<Self, Outcome> {
        Ok(Self {
            chain: Chain::new(lock, row)?,
            cache,
            opened: Vec::new(),
        })
    }

    /// The record at the run's `cut`.
    pub(super) fn record(&mut self, cut: u64) -> Result<FrameRecord, Outcome> {
        let (place, own) = self.chain.place(cut)?;
        while self.opened.len() <= place {
            let (_, row) = self
                .chain
                .reached
                .get(self.opened.len())
                .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
            self.opened.push(
                Cache::open(self.cache, row)
                    .map_err(|_| data_failure(None, "record cache pins"))?,
            );
        }
        let cache = self
            .opened
            .get(place)
            .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
        record(cache, own)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_run_reaches_the_rows_that_continue_it_and_counts_its_cuts_on_through_them() {
        use crate::lock::{Lock, RowId};
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scenarios/locks/lsnes.toml"
        );
        let lock = Lock::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let frames = |id: &str| lock.row(&RowId(id.into())).unwrap().frames.0;
        let first = frames("chain-any-0003-DFD7");
        let second = frames("chain-any-0004-E021");
        let mut chain = super::Chain::new(&lock, "chain-any-0003-DFD7").unwrap();
        assert_eq!(chain.place(0).unwrap(), (0, 0));
        assert_eq!(chain.place(first - 1).unwrap(), (0, first - 1));
        assert_eq!(
            chain.reached.len(),
            1,
            "a row is reached only when the run reaches it"
        );
        assert_eq!(chain.place(first).unwrap(), (1, 0));
        assert_eq!(chain.place(first + second + 2).unwrap(), (2, 2));
        let ids: Vec<_> = chain
            .reached
            .iter()
            .map(|(at, row)| (*at, row.id.0.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                (0, "chain-any-0003-DFD7"),
                (first, "chain-any-0004-E021"),
                (first + second, "chain-any-0005-E06B"),
            ]
        );
        assert!(
            chain.place(u64::MAX - 1).is_err(),
            "no row continues the capture's last"
        );
    }
}
