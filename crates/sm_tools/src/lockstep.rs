//! The lockstep observation comparison and its divergence report.
//!
//! Each paired frame, the verifier hands over the oracle's observation and,
//! where it has one, the engine's typed projection. Every domain the engine
//! projects is compared by hash, with a field diff only when the hashes differ.
//! Domains it does not project stay covered by the whole-RAM comparison, which
//! remains authoritative; the report counts both kinds per domain.
//!
//! The first divergence is written to an optional TOML report with both
//! values. Console text names only ticks, domains and fields, never values,
//! because values can be derived from the ROM.

use crate::engine::{self, Projection};
use crate::observation::{Difference, Domain, Observation};
use sm_game::session::Session;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Duration;

/// Differing fields the console names at the first divergence.
const NAMED_FIELDS: usize = 8;

/// The first Tick at which a projected domain differed.
struct Divergence {
    tick: i32,
    domain: Domain,
    differences: Vec<Difference>,
    /// Other projected domains that also differed at that Tick.
    also: Vec<Domain>,
}

#[derive(Default)]
struct DomainCounts {
    projected_ticks: u64,
    ram_compared_ticks: u64,
    divergent_ticks: u64,
    last_equal: Option<i32>,
    /// `last_equal` when this domain first differed.
    last_equal_before_first: Option<i32>,
    first_divergent: Option<i32>,
}

/// Where this Tick's engine projection came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineSource {
    /// The retained Session, current at the end of the Tick.
    Persistent,
    /// The Session of the Tick's last frame lease, as it was released.
    Lease,
}

/// Lockstep comparison state for one replay.
#[derive(Default)]
pub struct Lockstep {
    report: Option<PathBuf>,
    ticks: u64,
    domains: [DomainCounts; Domain::ALL.len()],
    first: Option<Divergence>,
    persistent_ticks: u64,
    lease_ticks: u64,
    unprojected_ticks: u64,
    both_passed: u64,
    both_failed: u64,
    ram_only: (u64, Option<i32>),
    observation_only: (u64, Option<i32>),
    report_error: Option<String>,
    capturing: bool,
    captured: Option<Projection>,
    /// Time spent projecting and comparing.
    pub elapsed: Duration,
}

impl Lockstep {
    /// Compare every paired frame; write the first divergence to `report`.
    pub fn new(report: Option<PathBuf>) -> Self {
        Self {
            report,
            ..Self::default()
        }
    }

    /// The engine's Tick starts: capture the leases it releases.
    pub fn begin_engine_tick(&mut self) {
        self.capturing = true;
        self.captured = None;
    }

    /// Project a frame lease's Session as it is released. Only the engine's
    /// Tick is captured; the last lease of the Tick is the one compared.
    pub fn capture_lease(&mut self, session: &Session) {
        if self.capturing {
            let start = std::time::Instant::now();
            self.captured = Some(engine::project(session));
            self.elapsed += start.elapsed();
        }
    }

    /// The engine's Tick ends; return the last lease's projection.
    pub fn end_engine_tick(&mut self) -> Option<Projection> {
        self.capturing = false;
        self.captured.take()
    }

    /// Compare one Tick; true when a projected domain differs.
    pub fn compare(
        &mut self,
        tick: i32,
        oracle: &Observation,
        engine: Option<(&Projection, EngineSource)>,
    ) -> bool {
        self.ticks += 1;
        match engine.map(|(_, source)| source) {
            Some(EngineSource::Persistent) => self.persistent_ticks += 1,
            Some(EngineSource::Lease) => self.lease_ticks += 1,
            None => self.unprojected_ticks += 1,
        }
        let mut differing = Vec::new();
        for (domain, counts) in Domain::ALL.into_iter().zip(&mut self.domains) {
            let Some((projection, _)) = engine.filter(|(p, _)| p.domains.contains(domain)) else {
                counts.ram_compared_ticks += 1;
                continue;
            };
            counts.projected_ticks += 1;
            if oracle.hash(domain) == projection.observation.hash(domain) {
                counts.last_equal = Some(tick);
                continue;
            }
            differing.push(domain);
            counts.divergent_ticks += 1;
            if counts.first_divergent.is_none() {
                counts.first_divergent = Some(tick);
                counts.last_equal_before_first = counts.last_equal;
            }
        }
        let (Some(&domain), Some((projection, _))) = (differing.first(), engine) else {
            return false;
        };
        if self.first.is_none() {
            let differences = oracle.differences(&projection.observation, domain);
            // A changed list can differ in every entry; name the first few.
            let mut fields: Vec<String> = differences
                .iter()
                .take(NAMED_FIELDS)
                .map(|difference| difference.path.to_string())
                .collect();
            if differences.len() > NAMED_FIELDS {
                fields.push(format!("{} more", differences.len() - NAMED_FIELDS));
            }
            let also = differing[1..].to_vec();
            let also_names: Vec<&str> = also.iter().map(|domain| domain.name()).collect();
            eprintln!(
                "@{tick}: Observation divergence in {}: {}{}",
                domain.name(),
                fields.join(", "),
                if also.is_empty() {
                    String::new()
                } else {
                    format!("; also differs: {}", also_names.join(", "))
                }
            );
            self.first = Some(Divergence {
                tick,
                domain,
                differences,
                also,
            });
            self.write_report();
        }
        true
    }

    /// Record whether the whole-RAM comparison agreed with this one.
    pub fn agree(&mut self, tick: i32, observation_failed: bool, ram_failed: bool) {
        let (count, first, side) = match (observation_failed, ram_failed) {
            (false, false) => {
                self.both_passed += 1;
                return;
            }
            (true, true) => {
                self.both_failed += 1;
                return;
            }
            (false, true) => (&mut self.ram_only.0, &mut self.ram_only.1, "RAM"),
            (true, false) => (
                &mut self.observation_only.0,
                &mut self.observation_only.1,
                "observation",
            ),
        };
        *count += 1;
        if first.is_none() {
            *first = Some(tick);
            eprintln!("@{tick}: Observation/RAM disagreement: only the {side} comparison failed");
        }
    }

    fn write_report(&mut self) {
        let Some(path) = &self.report else {
            return;
        };
        if let Err(error) = std::fs::write(path, self.report_toml()) {
            self.report_error = Some(format!("{}: {error}", path.display()));
        }
    }

    fn report_toml(&self) -> String {
        let mut text = String::new();
        macro_rules! line {
            ($($argument:tt)*) => {
                writeln!(text, $($argument)*).expect("String write")
            };
        }
        line!("# Observation divergence report (docs/SCENARIO_HARNESS.md).");
        line!("schema = 1");
        line!("ticks = {}", self.ticks);
        line!("diverged = {}", self.first.is_some());
        if let Some(first) = &self.first {
            let index = Domain::ALL
                .iter()
                .position(|&domain| domain == first.domain)
                .expect("listed domain");
            line!();
            line!("[divergence]");
            line!("tick = {}", first.tick);
            line!("domain = \"{}\"", first.domain.name());
            if let Some(difference) = first.differences.first() {
                line!("field = \"{}\"", difference.path);
                value_lines(&mut text, difference);
            }
            match self.domains[index].last_equal_before_first {
                Some(tick) => line!("last_equal_tick = {tick}"),
                None => line!("# No earlier Tick of this replay had an equal domain."),
            }
            let also: Vec<String> = first
                .also
                .iter()
                .map(|domain| format!("\"{}\"", domain.name()))
                .collect();
            line!("also_differs = [{}]", also.join(", "));
            for difference in &first.differences {
                line!();
                line!("[[divergence.fields]]");
                line!("field = \"{}\"", difference.path);
                value_lines(&mut text, difference);
            }
        }
        for (domain, counts) in Domain::ALL.iter().zip(&self.domains) {
            line!();
            line!("[domains.{}]", domain.name());
            line!("projected_ticks = {}", counts.projected_ticks);
            line!("ram_compared_ticks = {}", counts.ram_compared_ticks);
            line!("divergent_ticks = {}", counts.divergent_ticks);
            if let Some(tick) = counts.first_divergent {
                line!("first_divergent_tick = {tick}");
            }
            if let Some(tick) = counts.last_equal {
                line!("last_equal_tick = {tick}");
            }
        }
        line!();
        line!("[engine]");
        line!("persistent_ticks = {}", self.persistent_ticks);
        line!("lease_ticks = {}", self.lease_ticks);
        line!("unprojected_ticks = {}", self.unprojected_ticks);
        line!();
        line!("[agreement]");
        line!("both_passed = {}", self.both_passed);
        line!("both_failed = {}", self.both_failed);
        line!("ram_only_failed = {}", self.ram_only.0);
        line!("observation_only_failed = {}", self.observation_only.0);
        text
    }

    /// Finish the replay: rewrite the report with the final counts and
    /// return the summary lines, which name no values.
    pub fn finish(&mut self) -> Result<Vec<String>, String> {
        self.write_report();
        if let Some(error) = self.report_error.take() {
            return Err(format!("divergence report: {error}"));
        }
        let first = match &self.first {
            Some(first) => format!(
                "first divergence at tick {} in {} ({})",
                first.tick,
                first.domain.name(),
                first
                    .differences
                    .first()
                    .map_or_else(String::new, |difference| difference.path.to_string())
            ),
            None => "no divergence".to_owned(),
        };
        let divergent: Vec<String> = Domain::ALL
            .iter()
            .zip(&self.domains)
            .filter(|(_, counts)| counts.divergent_ticks != 0)
            .map(|(domain, counts)| format!("{} {}", domain.name(), counts.divergent_ticks))
            .collect();
        let coverage: Vec<String> = Domain::ALL
            .iter()
            .zip(&self.domains)
            .map(
                |(domain, counts)| match (counts.projected_ticks, counts.ram_compared_ticks) {
                    (_, 0) => format!("{} projected", domain.name()),
                    (0, _) => format!("{} RAM", domain.name()),
                    (projected, ram) => {
                        format!("{} projected {projected}/RAM {ram}", domain.name())
                    }
                },
            )
            .collect();
        let first_tick = |first: Option<i32>| {
            first.map_or_else(String::new, |tick| format!(", first tick {tick}"))
        };
        Ok(vec![
            format!(
                "Observations: {} ticks; {first}{}",
                self.ticks,
                if divergent.is_empty() {
                    String::new()
                } else {
                    format!("; divergent ticks: {}", divergent.join(", "))
                }
            ),
            format!(
                "Observation coverage: {}; engine Session from the frame lease {} ticks, persistent {}, none {}; {:.2}s observing",
                coverage.join(", "),
                self.lease_ticks,
                self.persistent_ticks,
                self.unprojected_ticks,
                self.elapsed.as_secs_f64()
            ),
            format!(
                "Observation/RAM agreement: {} both passed, {} both failed, {} RAM-only failures{}, {} observation-only failures{}",
                self.both_passed,
                self.both_failed,
                self.ram_only.0,
                first_tick(self.ram_only.1),
                self.observation_only.0,
                first_tick(self.observation_only.1)
            ),
        ])
    }
}

fn value_lines(text: &mut String, difference: &Difference) {
    for (side, value) in [("oracle", difference.oracle), ("engine", difference.engine)] {
        match value {
            Some(value) => writeln!(text, "{side} = {}", difference.path.display(value)),
            None => writeln!(text, "{side}_missing = true"),
        }
        .expect("String write");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Domains;

    fn projection(observation: Observation, domains: &[Domain]) -> Projection {
        let mut set = Domains::default();
        for &domain in domains {
            set.insert(domain);
        }
        Projection {
            observation,
            domains: set,
        }
    }

    #[test]
    fn only_projected_domains_are_compared() {
        let oracle = Observation::default();
        let mut engine = oracle.clone();
        engine.input.held = 1;
        let mut lockstep = Lockstep::new(None);
        let engine = projection(engine, &[Domain::Samus]);
        assert!(!lockstep.compare(0, &oracle, Some((&engine, EngineSource::Lease))));
        assert!(lockstep.first.is_none());
    }

    #[test]
    fn the_first_divergence_keeps_its_tick_domain_and_last_equal_tick() {
        let oracle = Observation::default();
        let mut lockstep = Lockstep::new(None);
        let equal = projection(oracle.clone(), &[Domain::Samus, Domain::Camera]);
        assert!(!lockstep.compare(4, &oracle, Some((&equal, EngineSource::Lease))));
        let mut engine = oracle.clone();
        engine.samus.charge = 3;
        let engine = projection(engine, &[Domain::Samus, Domain::Camera]);
        assert!(lockstep.compare(5, &oracle, Some((&engine, EngineSource::Lease))));
        let first = lockstep.first.as_ref().expect("divergence");
        assert_eq!((first.tick, first.domain), (5, Domain::Samus));
        assert_eq!(first.differences[0].path.to_string(), "samus.charge");
        let report = lockstep.report_toml();
        assert!(report.contains("field = \"samus.charge\""));
        assert!(report.contains("last_equal_tick = 4"));
    }

    #[test]
    fn disagreement_counts_each_side() {
        let mut lockstep = Lockstep::new(None);
        lockstep.agree(1, true, false);
        lockstep.agree(2, false, true);
        lockstep.agree(3, false, false);
        assert_eq!(lockstep.observation_only, (1, Some(1)));
        assert_eq!(lockstep.ram_only, (1, Some(2)));
        assert_eq!(lockstep.both_passed, 1);
    }
}
