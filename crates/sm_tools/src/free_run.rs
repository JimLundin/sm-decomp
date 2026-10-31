//! The free-run comparison: a single-side run against a reference trace.
//!
//! Neither side adopts the other's state, so a difference is not wiped out:
//! the report gives each stream's first divergence and how far it spreads
//! (how many Ticks, from when to when). The reference is the oracle's run for
//! free run, or the engine's own earlier run for determinism.
//!
//! Console text names ticks, streams and fields only; the optional TOML
//! report also holds the first differing values of each typed domain.

use crate::observation::{Difference, Domain};
use crate::trace::{MACHINE, STREAMS, Tick, stream_name};
use std::fmt::Write as _;
use std::path::PathBuf;

/// Differing fields the console names per stream.
const NAMED_FIELDS: usize = 4;

#[derive(Clone, Default)]
struct Counts {
    compared: u64,
    divergent: u64,
    last_equal: Option<i32>,
    /// `last_equal` when the stream first differed.
    last_equal_before_first: Option<i32>,
    first_divergent: Option<i32>,
    last_divergent: Option<i32>,
    /// A typed domain's differing values at its first divergence.
    first_differences: Vec<Difference>,
}

/// The reference's record of input frame `index`.
fn reference_tick(reference: &[Tick], index: u32) -> Option<&Tick> {
    let position = usize::try_from(index.checked_sub(reference.first()?.index)?).ok()?;
    reference.get(position).filter(|tick| tick.index == index)
}

/// Free-run comparison state for one run.
pub struct FreeRun {
    reference: Vec<Tick>,
    report: Option<PathBuf>,
    ticks: u64,
    /// Ticks the reference has no record of.
    unmatched: u64,
    divergent_ticks: u64,
    streams: Vec<Counts>,
    /// The first divergent Tick and every stream that differed at it.
    first: Option<(i32, Vec<usize>)>,
}

impl FreeRun {
    /// Compare against `reference`; write the report to `report` at the end.
    pub fn new(reference: Vec<Tick>, report: Option<PathBuf>) -> Self {
        Self {
            reference,
            report,
            ticks: 0,
            unmatched: 0,
            divergent_ticks: 0,
            streams: vec![Counts::default(); STREAMS],
            first: None,
        }
    }

    /// Compare one Tick by stream digest; true when any compared stream differs.
    pub fn compare(&mut self, tick: &Tick) -> bool {
        self.ticks += 1;
        let Some(reference) = reference_tick(&self.reference, tick.index) else {
            self.unmatched += 1;
            return true;
        };
        let at = reference.tick;
        let mut differing = Vec::new();
        for (stream, counts) in self.streams.iter_mut().enumerate() {
            let (Some(theirs), Some(mine)) = (reference.digest(stream), tick.digest(stream)) else {
                continue;
            };
            counts.compared += 1;
            if theirs == mine {
                counts.last_equal = Some(at);
                continue;
            }
            differing.push(stream);
            counts.divergent += 1;
            counts.last_divergent = Some(at);
            if counts.first_divergent.is_none() {
                counts.first_divergent = Some(at);
                counts.last_equal_before_first = counts.last_equal;
                if let (Some(domain), Some(left), Some(right)) = (
                    stream.checked_sub(MACHINE.len()).map(|d| Domain::ALL[d]),
                    &reference.typed,
                    &tick.typed,
                ) {
                    counts.first_differences =
                        left.observation.differences(&right.observation, domain);
                }
            }
        }
        if differing.is_empty() {
            return false;
        }
        self.divergent_ticks += 1;
        if self.first.is_none() {
            let names: Vec<&str> = differing.iter().map(|&s| stream_name(s)).collect();
            eprintln!(
                "@{at}: Free-run divergence in {}{}",
                names.join(", "),
                self.fields(&differing)
            );
            self.first = Some((at, differing));
        }
        true
    }

    /// The first differing field names of the typed streams in `streams`.
    fn fields(&self, streams: &[usize]) -> String {
        let mut names = Vec::new();
        for &stream in streams {
            let differences = &self.streams[stream].first_differences;
            names.extend(
                differences
                    .iter()
                    .take(NAMED_FIELDS)
                    .map(|d| d.path.to_string()),
            );
            if differences.len() > NAMED_FIELDS {
                names.push(format!("{} more", differences.len() - NAMED_FIELDS));
            }
        }
        if names.is_empty() {
            String::new()
        } else {
            format!(" ({})", names.join(", "))
        }
    }

    fn report_toml(&self) -> String {
        let mut text = String::new();
        macro_rules! line {
            ($($argument:tt)*) => {
                writeln!(text, $($argument)*).expect("String write")
            };
        }
        line!("# Free-run divergence report (docs/SCENARIO_HARNESS.md).");
        line!("schema = 1");
        line!("ticks = {}", self.ticks);
        line!("unmatched_ticks = {}", self.unmatched);
        line!("divergent_ticks = {}", self.divergent_ticks);
        line!("diverged = {}", self.first.is_some() || self.unmatched != 0);
        if let Some((tick, streams)) = &self.first {
            let names: Vec<String> = streams
                .iter()
                .map(|&s| format!("\"{}\"", stream_name(s)))
                .collect();
            line!();
            line!("[first]");
            line!("tick = {tick}");
            line!("streams = [{}]", names.join(", "));
        }
        for (stream, counts) in self.streams.iter().enumerate() {
            line!();
            line!("[streams.{}]", stream_name(stream));
            line!("compared_ticks = {}", counts.compared);
            line!("divergent_ticks = {}", counts.divergent);
            for (key, tick) in [
                ("first_divergent_tick", counts.first_divergent),
                ("last_equal_tick_before", counts.last_equal_before_first),
                ("last_divergent_tick", counts.last_divergent),
            ] {
                if let Some(tick) = tick {
                    line!("{key} = {tick}");
                }
            }
            for difference in &counts.first_differences {
                line!();
                line!("[[streams.{}.fields]]", stream_name(stream));
                line!("field = \"{}\"", difference.path);
                for (side, value) in [("reference", difference.oracle), ("run", difference.engine)]
                {
                    match value {
                        Some(value) => line!("{side} = {}", difference.path.display(value)),
                        None => line!("{side}_missing = true"),
                    }
                }
            }
        }
        text
    }

    /// Finish the run: write the report and return the summary lines, which
    /// name no values, with the number of failing Ticks.
    pub fn finish(&self) -> Result<(Vec<String>, u64), String> {
        if let Some(path) = &self.report {
            std::fs::write(path, self.report_toml())
                .map_err(|error| format!("{}: {error}", path.display()))?;
        }
        let first = match &self.first {
            Some((tick, streams)) => {
                let names: Vec<&str> = streams.iter().map(|&s| stream_name(s)).collect();
                format!(
                    "first divergence at tick {tick} in {}{}",
                    names.join(", "),
                    self.fields(streams)
                )
            }
            None => "no divergence".to_owned(),
        };
        let spread: Vec<String> = self
            .streams
            .iter()
            .enumerate()
            .filter_map(|(stream, counts)| {
                Some(format!(
                    "{} {} ticks ({}..{})",
                    stream_name(stream),
                    counts.divergent,
                    counts.first_divergent?,
                    counts.last_divergent?
                ))
            })
            .collect();
        let (compared, uncompared): (Vec<usize>, Vec<usize>) =
            (0..STREAMS).partition(|&stream| self.streams[stream].compared != 0);
        let names = |streams: &[usize]| {
            streams
                .iter()
                .map(|&s| stream_name(s))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut lines = vec![format!(
            "Free run: {} ticks, {} divergent, {} outside the reference; {first}",
            self.ticks, self.divergent_ticks, self.unmatched
        )];
        if !spread.is_empty() {
            lines.push(format!(
                "Free-run spread: {} of {STREAMS} streams: {}",
                spread.len(),
                spread.join(", ")
            ));
        }
        lines.push(format!(
            "Free-run coverage: compared {}; not compared {}",
            names(&compared),
            if uncompared.is_empty() {
                "none".to_owned()
            } else {
                names(&uncompared)
            }
        ));
        Ok((lines, self.divergent_ticks + self.unmatched))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Domains, Projection};
    use crate::observation::Observation;
    use crate::trace::typed_stream;

    fn tick(index: u32, wram: u64, charge: u16) -> Tick {
        let mut domains = Domains::default();
        domains.insert(Domain::Samus);
        let mut observation = Observation::default();
        observation.samus.charge = charge;
        Tick {
            index,
            tick: i32::try_from(index).expect("small") + 100,
            machine: [wram, 0, 0, 0, 0, 0, 0, 0],
            typed: Some(Projection {
                observation,
                domains,
            }),
        }
    }

    #[test]
    fn each_stream_keeps_its_first_divergence_and_spread() {
        let reference: Vec<Tick> = (0..4).map(|index| tick(index, 1, 0)).collect();
        let mut run = FreeRun::new(reference, None);
        assert!(!run.compare(&tick(0, 1, 0)));
        assert!(run.compare(&tick(1, 1, 5)));
        assert!(run.compare(&tick(2, 2, 5)));
        assert!(!run.compare(&tick(3, 1, 0)));
        let samus = &run.streams[typed_stream(Domain::Samus)];
        assert_eq!(
            (
                samus.first_divergent,
                samus.last_equal_before_first,
                samus.divergent
            ),
            (Some(101), Some(100), 2)
        );
        assert_eq!(samus.first_differences[0].path.to_string(), "samus.charge");
        assert_eq!(run.streams[0].first_divergent, Some(102));
        assert_eq!(run.first.as_ref().map(|(tick, _)| *tick), Some(101));
        let (lines, failing) = run.finish().expect("no report file");
        assert_eq!(failing, 2);
        assert!(lines[1].contains("samus 2 ticks (101..102)"));
        assert!(run.report_toml().contains("field = \"samus.charge\""));
    }

    #[test]
    fn a_tick_past_the_reference_fails() {
        let mut run = FreeRun::new(vec![tick(5, 1, 0)], None);
        assert!(!run.compare(&tick(5, 1, 0)));
        assert!(run.compare(&tick(6, 1, 0)));
        assert_eq!(run.finish().expect("no report file").1, 1);
    }
}
