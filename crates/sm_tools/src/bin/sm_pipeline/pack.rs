//! Pack replay rows into Verify jobs by their measured seconds
//! (`.github/scripts/pack.py`, whose docstring gives the measurements behind
//! each constant).
//!
//! The replay job splits one job's rows into `workers` verifier processes;
//! rows with an input prefix share the first process. The longest row in a
//! plan sets its wall time, so it is the budget: every other job's processes
//! are packed to finish well within it, with as few runners as it takes.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use crate::route::lightest;
use crate::scenario::Row;
use crate::toml;

/// ci.yml runs at most 18 replay jobs at once; the short lockstep route jobs
/// and rom-import fill in after the first to finish.
const RUNNERS: f64 = 17.0;
/// A process of several rows is packed to this share of the budget.
const FILL: f64 = 0.8;
/// A row within this share of the plan's longest row is a floor row.
const FIRST: f64 = 0.9;
/// Below this a job spends a large share of its time on setup.
const SHORTEST: f64 = 300.0;
/// A margin under ci.yml's 60-minute job timeouts for estimate error.
const LONGEST: f64 = 1800.0;
/// Frames a second assumed for a row without a measurement, in either mode.
const FPS: f64 = 5.0;

/// A matrix mode: what a job's verifier processes do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mode {
    /// Engine-only replay against the oracle digest lock.
    Lock,
    /// Lockstep against the oracle.
    Lockstep,
    /// Re-record faithful oracle locks privately and compare them.
    Record,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lock => "lock",
            Self::Lockstep => "lockstep",
            Self::Record => "record",
        }
    }

    /// Whether row_seconds.toml measures it as a lock check.
    fn is_lock(self) -> bool {
        self == Self::Lock
    }

    /// Processes per job: one per vCPU for lock checks, one per physical core for lockstep.
    pub fn workers(self) -> usize {
        if self.is_lock() { 4 } else { 2 }
    }
}

/// CPython's `sum` of floats (3.12 and later): Neumaier's compensated sum.
pub fn py_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let (mut total, mut compensation) = (0.0_f64, 0.0_f64);
    for value in values {
        let next = total + value;
        compensation += if total.abs() >= value.abs() {
            (total - next) + value
        } else {
            (value - next) + total
        };
        total = next;
    }
    if compensation != 0.0 && compensation.is_finite() {
        total + compensation
    } else {
        total
    }
}

/// Each row's measured replay seconds per verifier mode (row_seconds.toml).
#[derive(Debug, Clone, Default)]
pub struct Measured {
    lock: HashMap<String, f64>,
    lockstep: HashMap<String, f64>,
}

impl Measured {
    pub fn read(root: &Path) -> Result<Self, String> {
        let data = toml::read(&root.join(".github/scripts/row_seconds.toml"))?;
        let table = |name: &str| -> Result<HashMap<String, f64>, String> {
            let Some(value) = data.get(name) else {
                return Ok(HashMap::new());
            };
            let table = value
                .as_table()
                .ok_or_else(|| format!("row_seconds.toml: {name} must be a table"))?;
            table
                .iter()
                .map(|(id, seconds)| match seconds {
                    toml::Value::Integer(whole) => Ok((id.to_owned(), *whole as f64)),
                    toml::Value::Float(seconds) => Ok((id.to_owned(), *seconds)),
                    _ => Err(format!("row_seconds.toml: {name}.{id} must be a number")),
                })
                .collect()
        };
        Ok(Self {
            lock: table("lock")?,
            lockstep: table("lockstep")?,
        })
    }
}

/// The rows a plan packs, by id, with their measurements.
pub struct Rows<'a> {
    by_id: HashMap<&'a str, &'a Row>,
    measured: &'a Measured,
}

impl<'a> Rows<'a> {
    pub fn new(rows: &'a [Row], measured: &'a Measured) -> Self {
        Self {
            by_id: rows.iter().map(|row| (row.id.as_str(), row)).collect(),
            measured,
        }
    }

    pub fn get(&self, id: &str) -> Result<&'a Row, String> {
        self.by_id
            .get(id)
            .copied()
            .ok_or_else(|| format!("no row {id}"))
    }

    fn row(&self, id: &str) -> &'a Row {
        self.by_id[id]
    }

    fn prefixed(&self, id: &str) -> bool {
        self.row(id).start_frame != 0
    }

    /// A row's replay seconds in one mode: measured in it, else measured in the
    /// other (about as long), else estimated from its frames.
    pub fn seconds(&self, id: &str, mode: Mode) -> f64 {
        let (own, other) = if mode.is_lock() {
            (&self.measured.lock, &self.measured.lockstep)
        } else {
            (&self.measured.lockstep, &self.measured.lock)
        };
        if let Some(&seconds) = own.get(id).or_else(|| other.get(id)) {
            return seconds;
        }
        let row = self.row(id);
        let prefix = row.start_frame;
        let frames = row.frames
            + if row.checkpoint.is_some() {
                prefix.div_euclid(5)
            } else {
                prefix
            };
        (frames as f64 / FPS).max(1.0)
    }

    fn by_cost(&self, ids: &[String], mode: Mode) -> Vec<String> {
        let mut order = ids.to_vec();
        order.sort_by(|a, b| {
            self.seconds(b, mode)
                .total_cmp(&self.seconds(a, mode))
                .then_with(|| a.cmp(b))
        });
        order
    }

    /// Split one job's rows into `count` balanced processes, keeping prefix rows together.
    pub fn partition(&self, ids: &[String], mode: Mode, count: usize) -> Vec<Vec<String>> {
        let prefixed: Vec<String> = ids.iter().filter(|id| self.prefixed(id)).cloned().collect();
        let mut groups: Vec<Vec<String>> = vec![Vec::new(); count.min(ids.len()).max(1)];
        let mut loads = vec![0.0; groups.len()];
        if !prefixed.is_empty() {
            loads[0] = py_sum(prefixed.iter().map(|id| self.seconds(id, mode)));
            groups[0].extend(prefixed.iter().cloned());
        }
        let rest: Vec<String> = ids
            .iter()
            .filter(|id| !self.prefixed(id))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        for id in self.by_cost(&rest, mode) {
            let index = lightest(&loads, |&load| load);
            loads[index] += self.seconds(&id, mode);
            groups[index].push(id);
        }
        groups.retain(|group| !group.is_empty());
        groups
    }

    /// A job's estimated replay seconds: its slowest process.
    pub fn estimate(&self, ids: &[String], mode: Mode, count: usize) -> f64 {
        self.partition(ids, mode, count)
            .iter()
            .map(|group| py_sum(group.iter().map(|id| self.seconds(id, mode))))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// One plan's job budget over its (row ids, mode) work, and its longest row.
    pub fn budget<'w>(&self, work: impl IntoIterator<Item = (&'w [String], Mode)>) -> (f64, f64) {
        let mut costs: Vec<(f64, f64)> = work
            .into_iter()
            .flat_map(|(ids, mode)| ids.iter().map(move |id| (id, mode)))
            .map(|(id, mode)| (self.seconds(id, mode), mode.workers() as f64))
            .collect();
        if costs.is_empty() {
            costs.push((0.0, 1.0));
        }
        let longest = costs
            .iter()
            .map(|&(cost, _)| cost)
            .fold(f64::NEG_INFINITY, f64::max);
        let spread = py_sum(costs.iter().map(|&(cost, count)| cost / count)) / (FILL * RUNNERS);
        let wanted = longest.max(spread).max(SHORTEST);
        (longest.max(wanted.min(LONGEST)), longest)
    }

    /// Two-process jobs for the floor rows among ids, and the ids left.
    ///
    /// Beside each floor row runs one process of the longest other rows that fit
    /// within FILL of it; prefix rows stay out, since they would share its process.
    pub fn floor(
        &self,
        ids: &[String],
        mode: Mode,
        longest: f64,
    ) -> (Vec<Vec<String>>, Vec<String>) {
        let order = self.by_cost(ids, mode);
        let (floors, mut left): (Vec<String>, Vec<String>) = order
            .into_iter()
            .partition(|id| self.seconds(id, mode) >= FIRST * longest);
        let mut jobs = Vec::new();
        for id in floors {
            let cost = self.seconds(&id, mode);
            let mut job = vec![id];
            let mut load = 0.0;
            left.retain(|other| {
                let fits = !self.prefixed(other) && load + self.seconds(other, mode) <= FILL * cost;
                if fits {
                    load += self.seconds(other, mode);
                    job.push(other.clone());
                }
                !fits
            });
            jobs.push(job);
        }
        (jobs, left)
    }

    /// The fewest jobs whose processes each fit the budget, longest rows first.
    ///
    /// A process of one row may take the whole budget; one of several rows keeps
    /// to FILL of it, and never past LONGEST. Each job is checked with
    /// `partition`, as it will run.
    pub fn pack(&self, ids: &[String], mode: Mode, limit: f64) -> Vec<Vec<String>> {
        let count = mode.workers();
        let cap = FILL * limit.min(LONGEST);
        let order = self.by_cost(ids, mode);
        let fits = |job: &Vec<String>| {
            self.partition(job, mode, count).iter().all(|group| {
                group.len() == 1 || py_sum(group.iter().map(|id| self.seconds(id, mode))) <= cap
            })
        };
        let total_seconds = py_sum(order.iter().map(|id| self.seconds(id, mode)));
        // A job count, far inside usize.
        let mut total = (total_seconds / (count as f64 * cap)).ceil().max(1.0) as usize;
        loop {
            let mut jobs: Vec<Vec<String>> = vec![Vec::new(); total];
            let mut loads = vec![vec![0.0; count]; total];
            for id in &order {
                // Prefix rows run in the first process of their job.
                let cores = if self.prefixed(id) { 1 } else { count };
                let places: Vec<(usize, usize)> = (0..total)
                    .flat_map(|job| (0..cores).map(move |core| (job, core)))
                    .collect();
                let (job, core) = places[lightest(&places, |&(job, core)| loads[job][core])];
                jobs[job].push(id.clone());
                loads[job][core] += self.seconds(id, mode);
            }
            jobs.retain(|job| !job.is_empty());
            if total >= order.len() || jobs.iter().all(fits) {
                return jobs;
            }
            total += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_as_cpython_does() {
        // Plain addition gives 0.30000000000000004 and 0.0; CPython's sum gives these.
        assert_eq!(py_sum([0.1, 0.1, 0.1]), 0.30000000000000004);
        assert_eq!(py_sum([1e100, 1.0, -1e100]), 1.0);
        assert_eq!(py_sum([0.1; 10]), 1.0);
        assert_eq!(py_sum([]), 0.0);
    }

    fn row(id: &str, frames: i64, start_frame: i64) -> Row {
        Row {
            id: id.to_owned(),
            frames,
            start_frame,
            ..Row::default()
        }
    }

    #[test]
    fn packs_longest_first_and_keeps_prefix_rows_together() {
        let rows = [
            row("a", 5000, 0),
            row("b", 3000, 0),
            row("c", 1000, 0),
            row("p", 500, 100),
            row("q", 500, 100),
        ];
        let measured = Measured::default();
        let packed = Rows::new(&rows, &measured);
        let ids: Vec<String> = ["a", "b", "c", "p", "q"].map(String::from).to_vec();
        assert_eq!(packed.seconds("a", Mode::Lock), 1000.0);
        assert_eq!(packed.seconds("p", Mode::Lock), 120.0);
        let groups = packed.partition(&ids, Mode::Lockstep, 2);
        assert_eq!(groups, [vec!["p", "q", "b", "c"], vec!["a"]]);
        let (floors, left) = packed.floor(&ids, Mode::Lockstep, 1000.0);
        assert_eq!(floors, [vec!["a", "b", "c"]]);
        assert_eq!(left, ["p", "q"]);
    }
}
