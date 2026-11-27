//! Running format-2 rows from their pinned frame pads and native seeds.

use crate::lock::{Row, Sha256, Start as LockStart};
use crate::lsnes::{Lsnes, NativeSeed};
use crate::record::{Cut, FrameRecord, NmiKind, Pad};
use crate::{OracleError, Start, Streams};
use std::path::Path;
use std::time::Instant;

/// Counts and timing for a completed row.
#[derive(Clone, Debug, Default)]
pub struct RowRun {
    pub ticks: u32,
    pub frames: u64,
    pub overrun: u64,
    pub without_nmi: u64,
    pub still_running: u64,
    pub seconds: f64,
}

/// Why a row could not be played.
#[derive(Debug)]
pub enum RowError {
    Source(String),
    Oracle(OracleError),
}

impl core::fmt::Display for RowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Source(reason) => write!(f, "not run: {reason}"),
            Self::Oracle(error) => write!(f, "failed: {error}"),
        }
    }
}

impl From<String> for RowError {
    fn from(reason: String) -> Self {
        Self::Source(reason)
    }
}

impl From<OracleError> for RowError {
    fn from(error: OracleError) -> Self {
        Self::Oracle(error)
    }
}

/// ROM-derived inputs for one row, validated against the format-2 lock.
pub struct Sources {
    pads: Vec<Pad>,
    seed: Option<NativeSeed>,
}

impl Sources {
    /// Read only the active row's pad file and, for a seeded row, its native
    /// state and optional WRAM edits. The last pad previews the next frame.
    pub fn read(bundle: &Path, row: &Row) -> Result<Self, RowError> {
        let path = bundle.join("pads").join(format!("{}.pads", row.id));
        let bytes = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        if Sha256::of(&bytes) != row.pads {
            return Err(format!("{}: pad pin differs from the lock", path.display()).into());
        }
        let text =
            std::str::from_utf8(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
        let pads = crate::inputs::parse(&path, text)?;
        let frames = pads.len().saturating_sub(1);
        if pads.is_empty() || u64::try_from(frames).ok() != Some(row.frames.0) {
            return Err(format!(
                "{}: expected {} frames and a preview pad, got {} pads",
                path.display(),
                row.frames.0,
                pads.len()
            )
            .into());
        }
        let seed = match &row.start {
            LockStart::Seed { capture, index } => Some(NativeSeed::read_pinned(
                &bundle
                    .join("seeds")
                    .join(capture)
                    .join(format!("{index:04}.lss")),
                row.seed_record
                    .ok_or_else(|| "native seed has no pin".to_owned())?,
                row.seed_words,
            )?),
            LockStart::PowerOn if row.seed_record.is_none() && row.seed_words.is_none() => None,
            LockStart::PowerOn => return Err("power-on row has a seed pin".to_owned().into()),
        };
        Ok(Self { pads, seed })
    }
}

/// Optional streams and watched addresses.
#[derive(Clone, Copy, Debug, Default)]
pub struct Observe<'a> {
    pub streams: Option<Streams>,
    pub watch: &'a [u32],
    /// Four bytes of WRAM to sample at each watched instruction.
    pub peek: Option<u32>,
    /// Omit memory records when only cuts are needed.
    pub lean: bool,
}

/// Play a validated row and deliver each frame record to `observe`.
pub fn run_row_with(
    rom: &[u8],
    sources: Sources,
    observing: &Observe<'_>,
    observe: &mut dyn FnMut(&FrameRecord),
) -> Result<RowRun, RowError> {
    let start = sources
        .seed
        .as_ref()
        .map_or(Start::PowerOn, |seed| Start::State(seed.state.clone()));
    let mut oracle = match observing.peek {
        Some(peek) => Lsnes::open_measuring(rom, start, observing.watch, peek),
        None => Lsnes::open_watching(rom, start, observing.watch),
    }?;
    if let Some(seed) = &sources.seed {
        seed.apply(&mut oracle)?;
    }
    oracle.memories = !observing.lean;
    if let Some(streams) = observing.streams {
        oracle.record_streams(streams)?;
    }
    let started = Instant::now();
    let mut run = RowRun::default();
    for [pad, next] in sources.pads.array_windows() {
        oracle.preview(*next);
        let record = oracle.step(*pad)?;
        run.frames += 1;
        observe(record);
        match record.cut {
            Cut::NmiReturn(nmi) => match nmi.kind {
                NmiKind::Requested => run.ticks += 1,
                NmiKind::Overrun => run.overrun += 1,
            },
            Cut::NmiStillRunning { .. } => run.still_running += 1,
            Cut::VblankWithoutNmi => run.without_nmi += 1,
        }
    }
    run.seconds = started.elapsed().as_secs_f64();
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::{FrameCount, Lock};

    #[test]
    fn a_row_reads_the_pinned_native_seed_and_pads_without_movies() {
        let mut lock = Lock::parse(include_str!("../../verify/src/lock/format2.toml"))
            .expect("the fixture parses");
        let row = &mut lock.rows[0];
        let bundle =
            std::env::temp_dir().join(format!("oracle-corpus-sources-{}", std::process::id()));
        let seed_dir = bundle.join("seeds/capture-a");
        let pads_dir = bundle.join("pads");
        std::fs::create_dir_all(&seed_dir).unwrap();
        std::fs::create_dir_all(&pads_dir).unwrap();
        let state = b"synthetic native state";
        let words = b"0012=3400\n";
        let pads = b"0000\n0001\n";
        let pad_path = pads_dir.join("row-alpha.pads");
        let words_path = seed_dir.join("0003.words");
        std::fs::write(seed_dir.join("0003.lss"), state).unwrap();
        std::fs::write(&words_path, words).unwrap();
        std::fs::write(&pad_path, pads).unwrap();
        row.seed_record = Some(Sha256::of(state));
        row.seed_words = Some(Sha256::of(words));
        row.pads = Sha256::of(pads);
        row.frames = FrameCount(1);

        let sources = Sources::read(&bundle, row).unwrap();
        assert_eq!(sources.pads, [Pad(0), Pad(1)]);
        assert_eq!(sources.seed.unwrap().words[0].offset, 0x12);

        row.frames = FrameCount(2);
        assert!(matches!(
            Sources::read(&bundle, row),
            Err(RowError::Source(_))
        ));
        row.frames = FrameCount(1);
        std::fs::write(&pad_path, b"0000\n0002\n").unwrap();
        assert!(matches!(
            Sources::read(&bundle, row),
            Err(RowError::Source(_))
        ));
        std::fs::write(&pad_path, pads).unwrap();
        std::fs::remove_file(&words_path).unwrap();
        assert!(matches!(
            Sources::read(&bundle, row),
            Err(RowError::Source(_))
        ));
        std::fs::remove_dir_all(bundle).unwrap();
    }
}
