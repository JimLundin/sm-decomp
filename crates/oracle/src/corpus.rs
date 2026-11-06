//! Running the lock's corpus rows on an oracle and comparing their streams.
//!
//! The existing lock (`oracle = "faithful"`) was recorded by the old frame
//! stepper, whose Tick is one run of the game's main loop. Its rows' inputs
//! hold one pad word per Tick, and the NMI ending Tick k latched word k+1.
//! On a real machine a Tick is a requested frame, so the runner presents word
//! k+1 until the k-th requested NMI has read it, and folds the existing lock's
//! streams on requested frames only. The oracle contract's streams fold on
//! every video frame.
//!
//! The new lock gives a pad per video frame instead (ARCHITECTURE D20,
//! docs/oracle/RELOCK.md step b). A run under the Tick rule records the pad
//! it presented at each frame ([`RowRun::pads`]) once; a run from those pads
//! ([`Pads::Frames`]) knows every next pad, so it previews it and lsnes cuts
//! every frame at its NMI's RTI, also where the NMI outlasts the frame.

use crate::lock::{Entry, Pin};
use crate::record::{Cut, FrameRecord, NmiKind, Pad};
use crate::seed::Capture;
use crate::streams::{self, NAMES, Running};
use crate::{Backend, OracleError, Start, Tiers};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// One row's run.
#[derive(Clone, Debug)]
pub struct RowRun {
    /// Requested frames: the old stepper's Ticks.
    pub ticks: u32,
    /// Video frames.
    pub frames: u64,
    /// Frames whose NMI interrupted the main thread outside a wait site: lag frames.
    pub overrun: u64,
    /// Frames without an NMI (NMI disabled).
    pub without_nmi: u64,
    /// Frames cut at lsnes's boundary with their NMI still running: the
    /// steps a run without the next pad cannot cut at the RTI.
    pub still_running: u64,
    /// The pad presented at each video frame, then the pad of the frame
    /// after the row, which the last frame's NMI reads if it outlasts it.
    pub pads: Vec<Pad>,
    /// How often an overrun NMI interrupted each address.
    pub interrupted: std::collections::BTreeMap<u32, u32>,
    /// The frames (from 0) whose NMI overran.
    pub overrun_frames: Vec<u64>,
    /// The existing lock's streams, folded per Tick.
    pub legacy: Running,
    /// The oracle contract's streams, folded per video frame.
    pub v1: Running,
    pub seconds: f64,
}

/// Why a row has no run.
#[derive(Debug)]
pub enum RowError {
    /// Its data or start could not be set up, so it never ran.
    NotRun(String),
    /// The game stopped requesting frames.
    Stalled(String),
    /// The oracle failed under it; the rest of the corpus still runs.
    Oracle(OracleError),
}

impl core::fmt::Display for RowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotRun(reason) => write!(f, "not run: {reason}"),
            Self::Stalled(reason) => write!(f, "stalled: {reason}"),
            Self::Oracle(error) => write!(f, "failed: {error}"),
        }
    }
}

impl From<String> for RowError {
    fn from(reason: String) -> Self {
        Self::NotRun(reason)
    }
}

impl From<OracleError> for RowError {
    fn from(error: OracleError) -> Self {
        Self::Oracle(error)
    }
}

/// Where a row's private data lives under `data`: one directory per
/// ci-data bundle, named as the asset without `.tar.gz.enc`.
pub fn bundle_directory(data: &Path, pin: &Pin) -> PathBuf {
    data.join(pin.bundle.trim_end_matches(".tar.gz.enc"))
}

/// Run `entry`'s row with `pin`'s data from `data`.
pub fn run_row(rom: &[u8], pin: &Pin, entry: &Entry, data: &Path) -> Result<RowRun, RowError> {
    run_row_with(rom, pin, entry, data, &Observe::default(), &mut |_| ())
}

/// Where a row's pads come from.
#[derive(Clone, Copy, Debug, Default)]
pub enum Pads<'a> {
    /// The lock's inputs: one word per Tick, word k+1 presented until the
    /// k-th requested NMI has read it.
    #[default]
    Ticks,
    /// One pad per video frame and the pad of the frame after the row
    /// ([`RowRun::pads`]); the run is one frame shorter than the list.
    Frames(&'a [Pad]),
}

/// What a run records beyond its streams, and on which oracle.
#[derive(Clone, Copy, Debug, Default)]
pub struct Observe<'a> {
    /// The backend `Backend::open` names (LakeSnes when empty).
    pub backend: &'a str,
    /// Harness-native seeds (`movie --native-seeds`): a row seeded from
    /// `CAPTURE:INDEX` resumes the oracle's own state `DIR/CAPTURE/INDEX.lss`
    /// instead of loading the migration capture.
    pub native_seeds: Option<&'a Path>,
    pub tiers: Tiers,
    /// Watched instruction addresses (`Oracle::set_watch`).
    pub watch: &'a [u32],
    /// The WRAM address a watch hit peeks.
    pub peek: u32,
    pub pads: Pads<'a>,
    /// Records without the memories (lsnes), for runs that need only the
    /// pads and the cuts.
    pub lean: bool,
}

/// `run_row` with chosen tiers and watched addresses, showing `observe` every frame.
pub fn run_row_with(
    rom: &[u8],
    pin: &Pin,
    entry: &Entry,
    data: &Path,
    observing: &Observe<'_>,
    observe: &mut dyn FnMut(&FrameRecord),
) -> Result<RowRun, RowError> {
    let root = bundle_directory(data, pin);
    let inputs = crate::inputs::read(&root.join("movies").join(&pin.inputs))?;
    let mut native_seed = None;
    let start = match (&pin.seed, &pin.checkpoint) {
        (_, Some(checkpoint)) => {
            return Err(RowError::NotRun(format!(
                "starts from the old verifier's checkpoint {checkpoint}"
            )));
        }
        (Some(seed), None) => {
            let (capture, index) = seed
                .split_once(':')
                .ok_or_else(|| format!("seed {seed} is not CAPTURE:INDEX"))?;
            let index: u32 = index
                .parse()
                .map_err(|_| format!("seed {seed}: bad index"))?;
            if let Some(native) = observing.native_seeds {
                let seed = crate::lsnes::NativeSeed::read(
                    &native.join(capture).join(format!("{index:04}.lss")),
                )?;
                let start = Start::State(seed.state.clone());
                native_seed = Some(seed);
                start
            } else {
                let directory = root.join("target").join("seeds").join(capture);
                Start::Capture(Box::new(Capture::read(&directory, index)?))
            }
        }
        (None, None) => Start::PowerOn { wram_fill: 0x55 },
    };
    let mut oracle = Backend::open(observing.backend, rom, start)?;
    match (&mut oracle, &native_seed) {
        (Backend::Lsnes(lsnes), Some(seed)) => seed.apply(lsnes)?,
        (Backend::LakeSnes(_), Some(_)) => {
            return Err(RowError::NotRun("native seeds are lsnes states".into()));
        }
        (_, None) => {}
    }
    oracle.set_tiers(observing.tiers);
    oracle.set_watch(observing.watch, observing.peek);
    if let Backend::Lsnes(lsnes) = &mut oracle {
        lsnes.memories = !observing.lean;
    }
    let started = Instant::now();
    let mut run = RowRun {
        ticks: 0,
        frames: 0,
        overrun: 0,
        without_nmi: 0,
        still_running: 0,
        pads: Vec::new(),
        interrupted: std::collections::BTreeMap::new(),
        overrun_frames: Vec::new(),
        legacy: Running::default(),
        v1: Running::default(),
        seconds: 0.0,
    };
    let mut scratch = Vec::with_capacity(0x2_0000);
    // A stall (the game never requesting a frame) ends the row.
    let limit = u64::from(entry.ticks) * 4 + 600;
    let index = |count: u64| usize::try_from(count).unwrap_or(usize::MAX);
    // Word k+1 until the k-th requested NMI has read it.
    let tick_pad = |ticks: u32| {
        let tick = u64::from(ticks);
        inputs
            .get(index(tick + 1))
            .or_else(|| inputs.get(index(tick)))
            .copied()
            .unwrap_or(Pad(0))
    };
    loop {
        let pad = match observing.pads {
            Pads::Ticks if run.ticks >= entry.ticks => {
                run.pads.push(tick_pad(run.ticks));
                break;
            }
            Pads::Ticks => tick_pad(run.ticks),
            Pads::Frames(pads) => {
                let (Some(&pad), Some(&next)) =
                    (pads.get(index(run.frames)), pads.get(index(run.frames + 1)))
                else {
                    run.pads.extend(pads.last());
                    break;
                };
                oracle.preview(next);
                pad
            }
        };
        run.pads.push(pad);
        let record = oracle.step(pad)?;
        run.frames += 1;
        observe(record);
        run.v1.fold(&streams::v1(record));
        match record.nmi().map(|nmi| nmi.kind) {
            Some(NmiKind::Requested) => {
                run.legacy.fold(&streams::legacy(record, &mut scratch));
                run.ticks += 1;
            }
            Some(NmiKind::Overrun) => {
                run.overrun += 1;
                run.overrun_frames.push(run.frames - 1);
                if let Some(nmi) = record.nmi() {
                    *run.interrupted.entry(nmi.interrupted_pc).or_default() += 1;
                }
            }
            None if matches!(record.cut, Cut::NmiStillRunning { .. }) => run.still_running += 1,
            None => run.without_nmi += 1,
        }
        if matches!(observing.pads, Pads::Ticks) && run.frames > limit {
            let mut common: Vec<(u32, u32)> = run
                .interrupted
                .iter()
                .map(|(&pc, &count)| (count, pc))
                .collect();
            common.sort_unstable_by(|a, b| b.cmp(a));
            let common: Vec<String> = common
                .iter()
                .take(3)
                .map(|(count, pc)| format!("${pc:06X}x{count}"))
                .collect();
            return Err(RowError::Stalled(format!(
                "{} Ticks in {} frames; overruns interrupted {}",
                run.ticks,
                run.frames,
                common.join(" ")
            )));
        }
    }
    run.seconds = started.elapsed().as_secs_f64();
    Ok(run)
}

/// Which of the existing lock's streams the run matches: `Some(true)` or
/// `Some(false)` per stream, `None` where the harness cannot compute it.
pub fn compare(entry: &Entry, run: &RowRun) -> [Option<bool>; 8] {
    core::array::from_fn(|stream| {
        let mine = run.legacy.digest(stream)?;
        let locked = entry.digests.get(NAMES[stream])?;
        Some(format!("{mine:016x}") == *locked)
    })
}

/// The lock entry the harness records for a run: the v1 streams per video
/// frame, the existing lock's definitions per Tick beside them, and the
/// frames whose NMI overran.
pub fn lock_entry(entry: &Entry, oracle: crate::OracleId, run: &RowRun) -> String {
    use std::fmt::Write as _;
    let digests = |running: &Running| {
        NAMES
            .iter()
            .enumerate()
            .filter_map(|(stream, name)| {
                running
                    .digest(stream)
                    .map(|digest| format!("\"{name}\" = \"{digest:016x}\""))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let marks = run
        .v1
        .marks
        .iter()
        .map(|(frames, digest)| format!("\"{frames}\" = \"{digest:016x}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let overruns = run
        .overrun_frames
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let mut text = String::new();
    let _ = writeln!(text, "[[scenario]]");
    let _ = writeln!(text, "id = \"{}\"", entry.id);
    let _ = writeln!(text, "oracle = \"{}\"", oracle.emulator);
    let _ = writeln!(text, "version = \"{}\"", oracle.version);
    let fixes: Vec<String> = oracle
        .fixes
        .iter()
        .map(|fix| format!("\"{fix}\""))
        .collect();
    let _ = writeln!(text, "fixes = [{}]", fixes.join(", "));
    let _ = writeln!(text, "start = \"{}\"", entry.start);
    let _ = writeln!(text, "inputs = \"{}\"", entry.inputs);
    let _ = writeln!(text, "format = 1");
    let _ = writeln!(text, "frames = {}", run.frames);
    let _ = writeln!(text, "ticks = {}", run.ticks);
    let _ = writeln!(text, "combined = \"{:016x}\"", run.v1.combined);
    let _ = writeln!(text, "digests = {{ {} }}", digests(&run.v1));
    let _ = writeln!(text, "legacy = {{ {} }}", digests(&run.legacy));
    let _ = writeln!(text, "marks = {{ {marks} }}");
    let _ = writeln!(text, "overruns = [{overruns}]");
    text
}
