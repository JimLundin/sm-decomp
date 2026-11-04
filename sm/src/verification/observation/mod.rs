//! Typed observations in the verifier (docs/SCENARIO_HARNESS.md): the
//! oracle's projection and the hooks of the paired frame (lockstep) and of
//! single-side runs (free run, determinism and the oracle digest lock).
//!
//! The model, the engine's typed projection, the comparisons and the reports
//! are `sm_tools`; this module decodes the oracle's RAM and hands both sides
//! over, beside the unchanged RAM comparison.

mod coverage;
mod machine;
pub(crate) mod oracle;
mod session;

use crate::game_state::Game;
use sm_tools::engine::{Domains, Leases, Projection};
use sm_tools::free_run::FreeRun;
use sm_tools::lockstep::{EngineSource, Lockstep};
use sm_tools::observation::Domain;
use sm_tools::session_compare::{SessionCompare, SessionStreams, Skip};
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
    /// The Session streams' state, when the run takes them: always on the
    /// oracle, with `--session-streams` on the engine.
    session: Option<SessionStreams>,
}

/// A single-side run's options: free run, determinism and the digest lock.
#[derive(Default)]
pub struct SingleOptions<'a> {
    pub engine_only: bool,
    pub untyped: bool,
    pub trace_out: Option<&'a String>,
    pub lock_out: Option<&'a String>,
    pub free_run: Option<&'a String>,
    /// The engine also takes the Session streams, as the oracle always does.
    pub session_streams: bool,
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
            (false, false)
                if self.lock_out.is_some() || self.free_run.is_some() || self.session_streams =>
            {
                return Err(
                    "--lock-out, --free-run and --session-streams take a single-side run".into(),
                );
            }
            (false, false) => return Ok(None),
        };
        if report.is_some() && self.free_run.is_none() {
            return Err("a single-side divergence report requires --free-run".into());
        }
        Ok(Some(Single {
            side,
            session: (side == Side::Oracle || self.session_streams).then(SessionStreams::default),
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
    /// The report-only Session-space comparison beside the lockstep's.
    session: Option<SessionCompare>,
    elapsed: Duration,
}

impl Observer {
    /// Compare typed observations beside RAM on every paired frame.
    /// The Session-space comparison runs beside it unless
    /// `SM_VERIFY_SESSION_COMPARE=0`.
    pub fn lockstep(report: Option<PathBuf>) -> Self {
        let mut leases = Leases::default();
        let session = (std::env::var_os("SM_VERIFY_SESSION_COMPARE").as_deref()
            != Some("0".as_ref()))
        .then(|| {
            leases.keep_sessions();
            SessionCompare::default()
        });
        Self {
            leases,
            mode: Mode::Lockstep(Lockstep::new(report)),
            session,
            elapsed: Duration::ZERO,
        }
    }

    /// Record or compare every Tick of a single-side run.
    pub fn single(single: Single) -> Self {
        Self {
            leases: Leases::default(),
            mode: Mode::Single(Box::new(single)),
            session: None,
            elapsed: Duration::ZERO,
        }
    }

    /// Whether the run reads its rendered frames: the paired frame compares
    /// them, and a single-side run's `video` stream reaches its trace, lock
    /// entry or free run. A single-side run with none of those reads nothing.
    pub fn observes_video(&self) -> bool {
        match &self.mode {
            Mode::Lockstep(_) => true,
            Mode::Single(single) => {
                single.trace.is_some() || single.lock.is_some() || single.free_run.is_some()
            }
        }
    }

    /// The summary lines, which name no values, and for a free run the
    /// number of Ticks that failed it.
    pub fn finish(self) -> Result<(Vec<String>, Option<u64>), String> {
        match self.mode {
            Mode::Lockstep(mut lockstep) => {
                lockstep.elapsed += self.leases.elapsed;
                let mut lines = lockstep.finish()?;
                if let Some(mut session) = self.session {
                    session.elapsed += self.leases.session_elapsed;
                    lines.extend(session.finish());
                    lines.extend(
                        session
                            .coverage
                            .finish(&coverage::unowned(), &coverage::held()),
                    );
                    lines.push(coverage::unowned_summary());
                    lines.push(coverage::shared_summary(
                        &session
                            .coverage
                            .classes(&coverage::unowned(), &coverage::held()),
                    ));
                }
                Ok((lines, None))
            }
            Mode::Single(single) => {
                let Single {
                    side,
                    typed,
                    trace,
                    lock,
                    free_run,
                    session: _,
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

/// Compare the paired frame's Sessions at the frame's end: the engine's, the
/// Session its next lease takes up, and the oracle's result decoded from its
/// RAM. Report-only; the RAM comparison decides.
pub(crate) fn observe_session(game: &mut Game) {
    let Some(compare) = game
        .observation
        .as_mut()
        .filter(|observer| matches!(observer.mode, Mode::Lockstep(_)))
        .and_then(|observer| observer.session.as_mut())
    else {
        return;
    };
    let mut coverage = std::mem::take(&mut compare.coverage);
    let start = std::time::Instant::now();
    let last_lease = game
        .observation
        .as_deref()
        .and_then(|observer| observer.leases.last_session());
    let frame = match game.observed_session() {
        Some(retained) => session::Frame::Retained(retained),
        None => match (
            last_lease,
            crate::game::session_runtime::kept_session_writes(game),
        ) {
            (Some(kept), Some(written)) if same_world(kept, game) => {
                session::Frame::Kept(kept, written)
            }
            (last_lease, _) => session::Frame::Imported(last_lease),
        },
    };
    let source = frame.source();
    let world = game.world();
    let sessions = session::oracle(&game.g_snapshot_theirs.ram, world).zip(session::engine(
        &frame,
        &game.g_snapshot_mine.ram,
        &game.g_snapshot_theirs.ram,
        world,
        &mut coverage,
    ));
    let tick = game.snes_frame_counter;
    let Some(compare) = game
        .observation
        .as_mut()
        .and_then(|observer| observer.session.as_mut())
    else {
        return;
    };
    compare.coverage = coverage;
    compare.source(source);
    match sessions {
        None => compare.skip(Skip::Undecodable),
        Some((oracle, engine)) => {
            if let Some(last_lease) = &engine.last_lease {
                compare.written_after_lease(tick, last_lease, &engine.frame_end);
            }
            compare.compare(tick, &engine.frame_end, &oracle);
        }
    }
    compare.elapsed += start.elapsed();
}

/// Whether `session` binds the world Game holds, as a kept Session's reuse requires.
fn same_world(session: &sm_game::session::Session, game: &Game) -> bool {
    std::sync::Arc::ptr_eq(&session.world, game.world())
        && session.world.revision() == game.world().revision()
}

/// Count the paired frame's agreement between the two comparisons.
pub(crate) fn agree(game: &mut Game, observation_failed: Option<bool>, ram_failed: bool) {
    let tick = game.snes_frame_counter;
    let Some(observer) = game.observation.as_mut() else {
        return;
    };
    if let (Mode::Lockstep(lockstep), Some(failed)) = (&mut observer.mode, observation_failed) {
        lockstep.agree(tick, failed, ram_failed);
    }
    if let Some(session) = observer.session.as_mut() {
        session.agree(tick, ram_failed);
    }
}

/// Take a single-side run's Tick after input frame `index`: trace it, fold it
/// into the lock, compare it with the reference. True when it diverged.
pub fn single_tick(game: &mut Game, index: usize) -> Result<bool, String> {
    let Some(Mode::Single(single)) = game.observation.as_deref().map(|o| &o.mode) else {
        return Ok(false);
    };
    let (side, typed) = (single.side, single.typed);
    let streams = single.session.is_some();
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
    let mut tick = Tick {
        index: u32::try_from(index).map_err(|_| "input frame index overflow")?,
        // The lockstep names a Tick by the frame counter before it advanced.
        tick: game.snes_frame_counter - 1,
        machine: machine::result_digests(game),
        typed,
        session: None,
    };
    // Both sides decode their frame result, with the normalized RAM masked as
    // the wram stream masks it, until the engine's own Session is compared
    // (docs/handoff/session-compare.md, slice 5).
    let decoded = streams
        .then(|| session::oracle(&game.g_snapshot_mine.ram, game.world()))
        .flatten();
    let Some(observer) = game.observation.as_mut() else {
        return Ok(false);
    };
    let Mode::Single(single) = &mut observer.mode else {
        return Ok(false);
    };
    if let (Some(state), Some(decoded)) = (single.session.as_mut(), decoded) {
        tick.session = Some(state.digests(decoded));
    }
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
