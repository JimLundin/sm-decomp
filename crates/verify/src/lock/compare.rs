//! An engine run's marks against a row's entry, per stream, and the row's green status
//! (ARCHITECTURE D22).

use super::{FactResult, Row, RowId, Stream, StreamLock};
use data::digest::Digest;
use std::collections::BTreeMap;

/// What the engine gave for one row.
///
/// Each stream's span marks are folded as the recorder folds them (`data::digest`, one
/// mark per `mark_frames` frames, the last partial).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EngineRow {
    /// Each stream's marks; a stream the engine didn't fold is left out.
    pub streams: BTreeMap<Stream, Vec<Digest>>,
    /// Each fact's result on the engine's State.
    pub facts: BTreeMap<String, bool>,
}

/// The index of a span: frames `index * mark_frames` on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanIndex(pub usize);

/// The side that lacks a stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The lock names the stream `absent`.
    Lock,
    /// The engine gave no marks for it.
    Engine,
}

/// One (row, stream) pair's comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every span equal.
    Equal,
    /// The first span that differs. Where one side has fewer marks, it is the first span
    /// the shorter side lacks.
    Differs {
        /// The span.
        span: SpanIndex,
    },
    /// One side has no marks: the lock names it `absent`, or the engine didn't fold it.
    Absent(Side),
    /// The entry has no such stream (a typed stream not recorded yet); it doesn't decide
    /// green.
    NotLocked,
}

/// One stream's outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamReport {
    /// The stream.
    pub stream: Stream,
    /// Its comparison.
    pub outcome: Outcome,
}

impl StreamReport {
    /// Whether this pair keeps its row from green: a compared stream the lock has, not
    /// equal.
    #[must_use]
    pub const fn blocks(&self) -> bool {
        self.stream.compared() && !matches!(self.outcome, Outcome::Equal | Outcome::NotLocked)
    }

    /// Whether this is a green (row, stream) pair, the unit the gate ratchets.
    #[must_use]
    pub const fn green(&self) -> bool {
        matches!(self.outcome, Outcome::Equal)
    }
}

/// One fact's comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactOutcome {
    /// The engine's result is the oracle's.
    Agrees,
    /// The engine's result isn't the oracle's.
    Disagrees,
    /// The lock has no result to compare with. This doesn't block green yet: every fact
    /// is unevaluated until `tools relock` evaluates them on the decoded oracle State.
    Unevaluated,
    /// The lock has a result and the engine gave none.
    NotEvaluated,
}

impl FactOutcome {
    /// Whether this fact keeps its row from green.
    #[must_use]
    pub const fn blocks(self) -> bool {
        match self {
            Self::Agrees | Self::Unevaluated => false,
            Self::Disagrees | Self::NotEvaluated => true,
        }
    }
}

/// A row's comparison: every stream, machine streams first, and every fact of the entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowReport {
    /// The row.
    pub id: RowId,
    /// Every stream, machine streams first.
    pub streams: Vec<StreamReport>,
    /// Every fact of the entry, in its order.
    pub facts: Vec<(String, FactOutcome)>,
}

impl RowReport {
    /// D22's green: every compared stream equal and no fact blocking.
    pub fn green(&self) -> bool {
        !self.streams.iter().any(StreamReport::blocks)
            && !self.facts.iter().any(|(_, outcome)| outcome.blocks())
    }

    /// The green (row, stream) pairs' streams.
    pub fn green_streams(&self) -> impl Iterator<Item = Stream> + '_ {
        self.streams
            .iter()
            .filter(|report| report.green())
            .map(|report| report.stream)
    }

    /// The outcome for `stream`.
    #[must_use]
    pub fn outcome(&self, stream: Stream) -> Option<Outcome> {
        self.streams
            .iter()
            .find(|report| report.stream == stream)
            .map(|report| report.outcome)
    }
}

fn marks(locked: &[Digest], engine: &[Digest]) -> Outcome {
    let first = locked
        .iter()
        .zip(engine)
        .position(|(locked, engine)| locked != engine);
    match first {
        Some(span) => Outcome::Differs {
            span: SpanIndex(span),
        },
        None if locked.len() == engine.len() => Outcome::Equal,
        None => Outcome::Differs {
            span: SpanIndex(locked.len().min(engine.len())),
        },
    }
}

impl Row {
    /// Compares the engine's run of this row with its entry.
    #[must_use]
    pub fn compare(&self, engine: &EngineRow) -> RowReport {
        let streams = Stream::all()
            .map(|stream| {
                let outcome = match (self.stream(stream), engine.streams.get(&stream)) {
                    (None, _) => Outcome::NotLocked,
                    (Some(StreamLock::Absent), _) => Outcome::Absent(Side::Lock),
                    (Some(StreamLock::Marks(_)), None) => Outcome::Absent(Side::Engine),
                    (Some(StreamLock::Marks(locked)), Some(run)) => marks(locked, run),
                };
                StreamReport { stream, outcome }
            })
            .collect();
        let facts = self
            .facts
            .iter()
            .map(|(fact, result)| {
                let outcome = match (result, engine.facts.get(fact)) {
                    (FactResult::Unevaluated, _) => FactOutcome::Unevaluated,
                    (FactResult::Holds(_), None) => FactOutcome::NotEvaluated,
                    (FactResult::Holds(oracle), Some(engine)) if oracle == engine => {
                        FactOutcome::Agrees
                    }
                    (FactResult::Holds(_), Some(_)) => FactOutcome::Disagrees,
                };
                (fact.clone(), outcome)
            })
            .collect();
        RowReport {
            id: self.id.clone(),
            streams,
            facts,
        }
    }
}
