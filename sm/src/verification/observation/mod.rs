//! Typed observations in the verifier (docs/SCENARIO_HARNESS.md): the
//! oracle's projection and the hooks of the paired frame (lockstep) and of
//! single-side runs (free run, determinism and the oracle digest lock).
//!
//! The model, the engine's typed projection, the comparisons and the reports
//! are `sm_tools`; this module decodes the oracle's RAM and hands both sides
//! over, beside the unchanged RAM comparison.

mod machine;
pub(crate) mod oracle;

use crate::game_state::Game;
use sm_tools::engine::{Domains, Leases, Projection};
use sm_tools::free_run::FreeRun;
use sm_tools::lockstep::{EngineSource, Lockstep};
use sm_tools::observation::Domain;
use sm_tools::trace::{Running, Tick, TraceWriter};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The side a single-side run steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Oracle,
    Engine,
}

/// What a single-side run does with each Tick.
pub struct Single {
    pub side: Side,
    /// Off, only the machine streams are taken: no typed projection runs.
    typed: bool,
    trace: Option<TraceWriter>,
    /// Where the run's lock entry (its running digests) is written.
    lock: Option<(PathBuf, Running)>,
    free_run: Option<FreeRun>,
}

/// A single-side run's options: free run, determinism and the digest lock.
#[derive(Default)]
pub struct SingleOptions<'a> {
    pub engine_only: bool,
    pub untyped: bool,
    pub trace_out: Option<&'a String>,
    pub lock_out: Option<&'a String>,
    pub free_run: Option<&'a String>,
}

impl SingleOptions<'_> {
    /// The run these options ask for; `None` for the paired lockstep.
    pub fn open(
        self,
        oracle_only: bool,
        report: Option<PathBuf>,
    ) -> Result<Option<Single>, String> {
        let side = match (oracle_only, self.engine_only) {
            (true, true) => return Err("--oracle-only and --engine-only exclude each other".into()),
            (true, false) => Side::Oracle,
            (false, true) => Side::Engine,
            (false, false) if self.untyped || self.trace_out.is_some() => {
                return Err("--untyped and --trace-out take a single-side run".into());
            }
            (false, false) if self.lock_out.is_some() || self.free_run.is_some() => {
                return Err("--lock-out and --free-run take a single-side run".into());
            }
            (false, false) => return Ok(None),
        };
        if report.is_some() && self.free_run.is_none() {
            return Err("a single-side divergence report requires --free-run".into());
        }
        Ok(Some(Single {
            side,
            typed: !self.untyped,
            trace: self
                .trace_out
                .map(|path| TraceWriter::create(Path::new(path)))
                .transpose()?,
            lock: self
                .lock_out
                .map(|path| (PathBuf::from(path), Running::default())),
            free_run: match self.free_run {
                Some(reference) => Some(FreeRun::new(
                    sm_tools::trace::read(Path::new(reference))?,
                    report,
                )),
                None => None,
            },
        }))
    }
}

enum Mode {
    Lockstep(Lockstep),
    Single(Box<Single>),
}

/// The verifier's observation state for one replay.
pub struct Observer {
    pub(crate) leases: Leases,
    mode: Mode,
    elapsed: Duration,
}

impl Observer {
    /// Compare typed observations beside RAM on every paired frame.
    pub fn lockstep(report: Option<PathBuf>) -> Self {
        Self {
            leases: Leases::default(),
            mode: Mode::Lockstep(Lockstep::new(report)),
            elapsed: Duration::ZERO,
        }
    }

    /// Record or compare every Tick of a single-side run.
    pub fn single(single: Single) -> Self {
        Self {
            leases: Leases::default(),
            mode: Mode::Single(Box::new(single)),
            elapsed: Duration::ZERO,
        }
    }

    /// The summary lines, which name no values, and for a free run the
    /// number of Ticks that failed it.
    pub fn finish(self) -> Result<(Vec<String>, Option<u64>), String> {
        match self.mode {
            Mode::Lockstep(mut lockstep) => {
                lockstep.elapsed += self.leases.elapsed;
                Ok((lockstep.finish()?, None))
            }
            Mode::Single(single) => {
                let Single {
                    side,
                    typed,
                    trace,
                    lock,
                    free_run,
                } = *single;
                if let Some(trace) = trace {
                    trace.finish()?;
                }
                if let Some((path, running)) = lock {
                    std::fs::write(&path, running.lock_toml())
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                }
                let mut lines = vec![format!(
                    "Single-side run: {side:?} only, typed observation {}; {:.2}s observing",
                    if typed { "on" } else { "off" },
                    (self.elapsed + self.leases.elapsed).as_secs_f64()
                )];
                let failing = match free_run {
                    Some(free_run) => {
                        let (summary, failing) = free_run.finish()?;
                        lines.extend(summary);
                        Some(failing)
                    }
                    None => None,
                };
                Ok((lines, failing))
            }
        }
    }
}

/// The engine's Tick starts: capture the frame leases it releases.
pub fn begin_engine_tick(game: &mut Game) {
    if let Some(observer) = game.observation.as_mut() {
        observer.leases.begin_tick();
    }
}

/// The engine's typed state at the end of its Tick: the retained Session
/// when it is current, otherwise the Session of the Tick's last frame lease.
fn engine_projection(game: &mut Game) -> Option<(Projection, EngineSource)> {
    let lease = game.observation.as_mut()?.leases.end_tick();
    match game.observed_session() {
        Some(session) => Some((sm_tools::engine::project(session), EngineSource::Persistent)),
        None => lease.map(|projection| (projection, EngineSource::Lease)),
    }
}

/// Project both results of the paired frame just run and compare them. The
/// oracle's is its result snapshot, before comparison normalization. `None`
/// when the lockstep comparison is off.
pub(crate) fn observe(game: &mut Game) -> Option<bool> {
    if !matches!(game.observation.as_deref()?.mode, Mode::Lockstep(_)) {
        return None;
    }
    let start = std::time::Instant::now();
    let engine = engine_projection(game);
    let oracle = oracle::project(&game.g_snapshot_theirs.ram);
    let tick = game.snes_frame_counter;
    let observer = game.observation.as_mut()?;
    let Mode::Lockstep(lockstep) = &mut observer.mode else {
        return None;
    };
    let diverged = lockstep.compare(
        tick,
        &oracle,
        engine
            .as_ref()
            .map(|(projection, source)| (projection, *source)),
    );
    observer.elapsed += start.elapsed();
    Some(diverged)
}

/// Count the paired frame's agreement between the two comparisons.
pub(crate) fn agree(game: &mut Game, observation_failed: Option<bool>, ram_failed: bool) {
    let tick = game.snes_frame_counter;
    if let (Some(observer), Some(failed)) = (game.observation.as_mut(), observation_failed) {
        if let Mode::Lockstep(lockstep) = &mut observer.mode {
            lockstep.agree(tick, failed, ram_failed);
        }
    }
}

/// Take a single-side run's Tick after input frame `index`: trace it, fold it
/// into the lock, compare it with the reference. True when it diverged.
pub fn single_tick(game: &mut Game, index: usize) -> Result<bool, String> {
    let Some(Mode::Single(single)) = game.observation.as_deref().map(|o| &o.mode) else {
        return Ok(false);
    };
    let (side, typed) = (single.side, single.typed);
    let start = std::time::Instant::now();
    let engine = match side {
        Side::Engine if typed => engine_projection(game).map(|(projection, _)| projection),
        _ => None,
    };
    let result = machine::capture_result(game);
    let typed = match side {
        Side::Oracle if typed => {
            let mut domains = Domains::default();
            for domain in Domain::ALL {
                domains.insert(domain);
            }
            Some(Projection {
                observation: oracle::project(&result.ram),
                domains,
            })
        }
        _ => engine,
    };
    let tick = Tick {
        index: u32::try_from(index).map_err(|_| "input frame index overflow")?,
        // The lockstep names a Tick by the frame counter before it advanced.
        tick: game.snes_frame_counter - 1,
        machine: machine::result_digests(game),
        typed,
    };
    let Some(observer) = game.observation.as_mut() else {
        return Ok(false);
    };
    let Mode::Single(single) = &mut observer.mode else {
        return Ok(false);
    };
    if let Some(trace) = single.trace.as_mut() {
        trace.write(&tick)?;
    }
    if let Some((_, running)) = single.lock.as_mut() {
        running.fold(&tick);
    }
    let diverged = single
        .free_run
        .as_mut()
        .is_some_and(|free_run| free_run.compare(&tick));
    observer.elapsed += start.elapsed();
    Ok(diverged)
}
