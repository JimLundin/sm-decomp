//! The lock rows a run plays through, and their records.

use crate::cache::Cache;
use crate::lock::{Lock, Row, RowId};
use crate::replay::cache_failure;
use crate::replay::readers::record;
use crate::replay::{data_failure, result::Outcome};
use crate::report::seeds;
use oracle_record::FrameRecord;
use std::path::Path;

/// The lock rows a run plays through: its own, then each row that continues it, reached as the
/// run reaches it. A row continues another when its seed's movie frame is the one after the
/// other's last cut, from whichever capture of the movie; the run's cuts count on through it.
struct Chain<'a> {
    lock: &'a Lock,
    /// The repository, whose tables of seeds give their movie frames.
    root: &'a Path,
    /// Each row reached so far, with the run's cut its own cut 0 is.
    reached: Vec<(u64, &'a Row)>,
}

impl<'a> Chain<'a> {
    fn new(lock: &'a Lock, root: &'a Path, row: &str) -> Result<Self, Outcome> {
        let row = lock
            .row(&RowId(row.to_owned()))
            .ok_or_else(|| data_failure(None, "played lock row"))?;
        Ok(Self {
            lock,
            root,
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
            let following = seeds::following(self.root, self.lock, row)
                .map_err(|_| data_failure(Some(cut), "played seed frames"))?
                .ok_or_else(|| data_failure(Some(cut), "played row continuing the last row"))?;
            self.reached.push((next, following));
        }
    }
}

/// The records of a run's chain of rows, each row's cache opened as the run reaches it.
pub(in crate::replay) struct Rows<'a> {
    chain: Chain<'a>,
    cache: &'a Path,
    /// The caches of the rows reached, in the chain's order.
    opened: Vec<Cache>,
    required_watch: Option<u32>,
    #[cfg(test)]
    pub(super) clock_cuts: Vec<u64>,
}

#[derive(Clone, Copy)]
pub(super) struct Turn<'a> {
    pub(super) controller: super::Controller,
    pub(super) previous: &'a FrameRecord,
    pub(super) current: &'a FrameRecord,
    pub(super) cut: u64,
    pub(super) place: (&'a str, u64),
}

impl Turn<'_> {
    pub(super) fn advance(
        &self,
        run: &mut super::Run<'_>,
    ) -> Result<
        (
            Vec<crate::replay::compare::Observation>,
            game::production::played::Outputs,
        ),
        Outcome,
    > {
        run.step_with_outputs(
            self.controller,
            self.previous,
            self.current,
            self.cut,
            self.place,
        )
    }

    pub(super) fn check_continuation(
        &self,
        restored: &mut super::Run<'_>,
        live: &super::Run<'_>,
        direct: &(
            Vec<crate::replay::compare::Observation>,
            game::production::played::Outputs,
        ),
    ) -> Result<(), Outcome> {
        let replayed = self.advance(restored)?;
        if replayed == *direct && Self::same_room(restored, live) {
            Ok(())
        } else {
            Err(Outcome::CheckpointFailure { cut: self.cut })
        }
    }

    pub(super) fn same_room(restored: &super::Run<'_>, live: &super::Run<'_>) -> bool {
        restored.state == live.state
            && restored.return_timing == live.return_timing
            && restored.warning_picture == live.warning_picture
    }

    pub(super) fn fields(
        &self,
        run: &mut super::Run<'_>,
    ) -> Result<Vec<crate::replay::compare::Observation>, Outcome> {
        run.step(
            self.controller,
            self.previous,
            self.current,
            self.cut,
            self.place,
        )
    }
}

impl<'a> Rows<'a> {
    pub(super) fn replay<S>(
        &mut self,
        mut previous: FrameRecord,
        mut controller: super::Controller,
        cuts: std::ops::RangeInclusive<u64>,
        mut state: S,
        mut step: impl FnMut(S, Turn<'_>, &mut Self) -> Result<S, Outcome>,
    ) -> Result<S, Outcome> {
        for cut in cuts {
            let current = self.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            let (row, local) = self.place(cut)?;
            state = step(
                state,
                Turn {
                    controller,
                    previous: &previous,
                    current: &current,
                    cut,
                    place: (&row, local),
                },
                self,
            )?;
            controller = controller.read(current.pad.0, super::polled(&current));
            previous = current;
        }
        Ok(state)
    }

    /// The rows from `row` on, their records in `cache` and their seeds' movie frames in the
    /// tables of `root`.
    pub(in crate::replay) fn open(
        lock: &'a Lock,
        root: &'a Path,
        cache: &'a Path,
        row: &str,
    ) -> Result<Self, Outcome> {
        Ok(Self {
            chain: Chain::new(lock, root, row)?,
            cache,
            opened: Vec::new(),
            required_watch: None,
            #[cfg(test)]
            clock_cuts: Vec::new(),
        })
    }

    pub(super) fn witnesses(&self) -> Result<super::witnesses::Witnesses, Outcome> {
        super::witnesses::read(self.chain.root)
    }

    /// Every subsequent record must distinguish an absent hit from an unwatched address.
    pub(in crate::replay) const fn require_watch(&mut self, pc: u32) {
        self.required_watch = Some(pc);
    }

    /// The row that holds the run's `cut`, and its own cut there.
    pub(in crate::replay) fn place(&mut self, cut: u64) -> Result<(String, u64), Outcome> {
        let (place, own) = self.chain.place(cut)?;
        let (_, row) = self
            .chain
            .reached
            .get(place)
            .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
        Ok((row.id.0.clone(), own))
    }

    /// Read the next native interval, including across a row boundary.
    pub(in crate::replay) fn successor(
        &mut self,
        cut: u64,
        previous: &FrameRecord,
    ) -> Result<FrameRecord, Outcome> {
        let current = self.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "disconnected native video frames"));
        }
        Ok(current)
    }

    /// The record at the run's `cut`.
    pub(in crate::replay) fn record(&mut self, cut: u64) -> Result<FrameRecord, Outcome> {
        let (place, own) = self.chain.place(cut)?;
        while self.opened.len() <= place {
            let (_, row) = self
                .chain
                .reached
                .get(self.opened.len())
                .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
            self.opened
                .push(Cache::open(self.cache, row).map_err(cache_failure(None))?);
        }
        let cache = self
            .opened
            .get(place)
            .ok_or_else(|| data_failure(Some(cut), "played rows"))?;
        if self
            .required_watch
            .is_some_and(|pc| !cache.watched().contains(&pc))
        {
            return Err(data_failure(
                Some(cut),
                "missing required instruction watch",
            ));
        }
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
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        let mut chain = super::Chain::new(&lock, root, "chain-any-0003-DFD7").unwrap();
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

    /// The new game's arrival ends one capture's seeds and Ceres's first room begins another's:
    /// the room's seed is the movie frame after the arrival's last cut, so the run goes on into it.
    #[test]
    fn a_run_goes_on_into_the_row_of_another_capture_seeded_at_the_next_movie_frame() {
        use crate::lock::{Lock, RowId};
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../scenarios/locks/lsnes.toml"
        );
        let lock = Lock::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        let arrival = lock
            .row(&RowId("any-ceres-arrival".into()))
            .unwrap()
            .frames
            .0;
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        let mut chain = super::Chain::new(&lock, root, "any-ceres-arrival").unwrap();
        assert_eq!(chain.place(arrival).unwrap(), (1, 0));
        assert_eq!(chain.reached[1].1.id.0, "chain-any-0001-DF45");
    }
}
