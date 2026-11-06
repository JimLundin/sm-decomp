//! Lock format 2, the recording side (docs/oracle/RELOCK.md §2, step c).
//!
//! A [`Recorder`] folds one row's video frames into span marks: each stream's
//! digest of every 1000-frame span, folded from empty (Q1), the last span
//! partial. A row's digest per stream is the fold of its marks
//! ([`data::digest::fold_all`]); the engine's verifier computes the same.
//!
//! A cut is quiescent when it is the RTI of an NMI that found the main thread
//! at one of [`crate::game::WAIT_SITES`] ([`NmiKind::Requested`]). The memory,
//! `ppu`, `wram` and `apu` streams fold at quiescent cuts only; `video` and
//! `timing` fold every frame. The overrun and NMI-off schedules name the other
//! cuts, so a `Recorded` timing source folds the engine at the same frames.

use crate::record::{ApuAccessKind, Cut, FrameRecord, NmiKind, PpuRegisters};
use core::hash::{Hash, Hasher};
use data::digest::{Digest, fold};
use std::fmt::Write as _;
use std::path::Path;

/// The record format this module writes.
pub const FORMAT: u32 = 2;

/// Video frames per span mark.
pub const MARK_FRAMES: u64 = 1000;

/// A stream of the format-2 lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Vram,
    Cgram,
    Oam,
    Sram,
    Ppu,
    Apu,
    Video,
    Timing,
    Wram,
}

/// When a stream folds a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Folding {
    Quiescent,
    EveryFrame,
}

impl Stream {
    /// Every stream, in the order an entry lists them.
    pub const ALL: [Self; 9] = [
        Self::Vram,
        Self::Cgram,
        Self::Oam,
        Self::Sram,
        Self::Ppu,
        Self::Apu,
        Self::Video,
        Self::Timing,
        Self::Wram,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Vram => "vram",
            Self::Cgram => "cgram",
            Self::Oam => "oam",
            Self::Sram => "sram",
            Self::Ppu => "ppu",
            Self::Apu => "apu",
            Self::Video => "video",
            Self::Timing => "timing",
            Self::Wram => "wram",
        }
    }

    const fn folding(self) -> Folding {
        match self {
            Self::Vram
            | Self::Cgram
            | Self::Oam
            | Self::Sram
            | Self::Ppu
            | Self::Apu
            | Self::Wram => Folding::Quiescent,
            Self::Video | Self::Timing => Folding::EveryFrame,
        }
    }

    /// This stream's digest of `record`; `apu_busy` is the port log since
    /// the last quiescent cut, this frame's included. `None` where the record
    /// lacks the stream.
    fn digest(self, record: &FrameRecord, apu_busy: &[u8]) -> Option<u64> {
        match self {
            Self::Vram => <&[u16; 0x8000]>::try_from(&record.vram[..])
                .ok()
                .map(hashed),
            Self::Cgram => <&[u16; 0x100]>::try_from(&record.cgram[..])
                .ok()
                .map(hashed),
            Self::Oam => crate::streams::oam_digest(record),
            Self::Sram => Some(Digest::of(&record.sram)),
            Self::Ppu => Some(ppu_digest(&record.ppu)),
            Self::Apu => Some(Digest::of(apu_busy)),
            Self::Video => record
                .picture
                .as_ref()
                .map(|picture| Digest::of(picture.shown().as_flattened())),
            Self::Timing => Some(Digest::of(&timing(record))),
            Self::Wram => Some(Digest::of(&record.wram)),
        }
    }
}

fn hashed(value: &impl Hash) -> u64 {
    let mut digest = Digest::default();
    value.hash(&mut digest);
    digest.finish()
}

/// The architectural PPU registers without the five lsnes doesn't expose
/// (MEASUREMENTS (o); `nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel` are not
/// compared, RELOCK Q4), in declaration order.
fn ppu_digest(ppu: &PpuRegisters) -> u64 {
    let PpuRegisters {
        inidisp,
        obsel,
        bgmode,
        mosaic,
        bg_sc,
        bg12nba,
        bg34nba,
        bg_scroll,
        m7sel,
        m7,
        window_select,
        window_positions,
        wbglog,
        wobjlog,
        screens,
        cgwsel,
        cgadsub,
        fixed_color,
        setini,
        nmitimen: _,
        htime: _,
        vtime: _,
        hdmaen: _,
        memsel: _,
        dma,
    } = ppu;
    let mut digest = Digest::default();
    (inidisp, obsel, bgmode, mosaic, bg_sc, bg12nba, bg34nba).hash(&mut digest);
    (bg_scroll, m7sel, m7, window_select, window_positions).hash(&mut digest);
    (
        wbglog,
        wobjlog,
        screens,
        cgwsel,
        cgadsub,
        fixed_color,
        setini,
    )
        .hash(&mut digest);
    dma.hash(&mut digest);
    digest.finish()
}

/// The `timing` stream's frame: the cut kind, the NMI kind and the overrun flag.
fn timing(record: &FrameRecord) -> [u8; 3] {
    let cut = match record.cut {
        Cut::NmiReturn(_) => 0,
        Cut::VblankWithoutNmi => 1,
        Cut::NmiStillRunning { .. } => 2,
    };
    let kind = record.nmi().map(|nmi| nmi.kind);
    let nmi = match kind {
        None => 0,
        Some(NmiKind::Requested) => 1,
        Some(NmiKind::Overrun) => 2,
    };
    [cut, nmi, u8::from(kind == Some(NmiKind::Overrun))]
}

/// The CPU's `$2140-$2143` accesses of a frame as `(kind, port, value)`.
/// lsnes's bus hook sees no DMA side, so DMA entries are dropped on both sides.
fn port_log(record: &FrameRecord) -> impl Iterator<Item = u8> + '_ {
    record
        .apu_log
        .iter()
        .filter_map(|access| {
            let kind = match access.kind {
                ApuAccessKind::CpuWrite => 0,
                ApuAccessKind::CpuRead => 1,
                ApuAccessKind::DmaWrite | ApuAccessKind::DmaRead | ApuAccessKind::SpcWrite => {
                    return None;
                }
            };
            Some([kind, access.port, access.value])
        })
        .flatten()
}

/// Whether `record`'s cut is quiescent: the RTI of an NMI that interrupted a wait site.
pub fn is_quiescent(record: &FrameRecord) -> bool {
    record
        .nmi()
        .is_some_and(|nmi| nmi.kind == NmiKind::Requested)
}

/// A stream of the decoded oracle State (D21): the hook for `input`, `samus`,
/// `camera` and the others, which `verify::decode` will project. None is
/// implemented yet; a recorder without them writes none.
pub trait TypedStream {
    fn name(&self) -> &'static str;
    /// The digest of the State at a quiescent cut, or `None` while the
    /// decoder can't decode it (the stream is then absent, Q3).
    fn digest(&mut self, record: &FrameRecord) -> Option<u64>;
}

/// One stream's marks.
#[derive(Clone, Debug)]
struct Spans {
    current: u64,
    marks: Vec<u64>,
    absent: bool,
}

impl Default for Spans {
    fn default() -> Self {
        Self {
            current: Digest::default().finish(),
            marks: Vec::new(),
            absent: false,
        }
    }
}

impl Spans {
    fn fold(&mut self, digest: Option<u64>) {
        match digest {
            Some(digest) => self.current = fold(self.current, digest),
            None => self.absent = true,
        }
    }

    fn close(&mut self) {
        self.marks.push(core::mem::replace(
            &mut self.current,
            Digest::default().finish(),
        ));
    }

    fn marks(&self) -> Option<&[u64]> {
        (!self.absent).then_some(&self.marks)
    }
}

/// One row's fold, fed every frame record in order.
#[derive(Default)]
pub struct Recorder {
    frames: u64,
    spans: [Spans; 9],
    typed: Vec<(Box<dyn TypedStream>, Spans)>,
    apu_busy: Vec<u8>,
    overruns: Vec<u64>,
    nmi_off: Vec<u64>,
    still_running: Vec<u64>,
}

impl Recorder {
    /// A recorder that also folds `typed` streams.
    pub fn with_typed(typed: Vec<Box<dyn TypedStream>>) -> Self {
        Self {
            typed: typed
                .into_iter()
                .map(|stream| (stream, Spans::default()))
                .collect(),
            ..Self::default()
        }
    }

    pub fn fold(&mut self, record: &FrameRecord) {
        let frame = self.frames;
        match record.cut {
            Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Overrun => self.overruns.push(frame),
            Cut::NmiReturn(_) => {}
            Cut::VblankWithoutNmi => self.nmi_off.push(frame),
            Cut::NmiStillRunning { .. } => self.still_running.push(frame),
        }
        self.apu_busy.extend(port_log(record));
        let quiescent = is_quiescent(record);
        for (stream, spans) in Stream::ALL.iter().zip(&mut self.spans) {
            if quiescent || stream.folding() == Folding::EveryFrame {
                spans.fold(stream.digest(record, &self.apu_busy));
            }
        }
        if quiescent {
            self.apu_busy.clear();
            for (stream, spans) in &mut self.typed {
                spans.fold(stream.digest(record));
            }
        }
        self.frames += 1;
        if self.frames.is_multiple_of(MARK_FRAMES) {
            self.close_spans();
        }
    }

    fn close_spans(&mut self) {
        self.spans.iter_mut().for_each(Spans::close);
        self.typed.iter_mut().for_each(|(_, spans)| spans.close());
    }

    /// The row's marks per stream, closing the partial last span.
    pub fn finish(mut self) -> Recorded {
        if !self.frames.is_multiple_of(MARK_FRAMES) {
            self.close_spans();
        }
        let streams = Stream::ALL
            .iter()
            .zip(&self.spans)
            .map(|(stream, spans)| (stream.name(), spans.marks().map(<[u64]>::to_vec)))
            .chain(
                self.typed
                    .iter()
                    .map(|(stream, spans)| (stream.name(), spans.marks().map(<[u64]>::to_vec))),
            )
            .collect();
        Recorded {
            frames: self.frames,
            overruns: self.overruns,
            nmi_off: self.nmi_off,
            still_running: self.still_running,
            streams,
        }
    }
}

/// What a row's run gave the recorder.
#[derive(Clone, Debug)]
pub struct Recorded {
    pub frames: u64,
    pub overruns: Vec<u64>,
    pub nmi_off: Vec<u64>,
    /// Frames cut with their NMI still running: none in a run from frame pads.
    pub still_running: Vec<u64>,
    /// Each stream's marks, `None` where some frame lacked it.
    pub streams: Vec<(&'static str, Option<Vec<u64>>)>,
}

/// A fact's result on the oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactResult {
    Holds(bool),
    /// Not evaluated: `probe:` facts are old-engine instrumentation, and the
    /// others wait for `verify::decode` (step c, second part).
    Unevaluated,
}

/// One format-2 `[[row]]` entry.
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    /// The native seed's capture name (`CAPTURE:INDEX`), or `power-on`.
    pub start: String,
    /// SHA-256 of the native seed's state file, and of its WRAM edits if it has any.
    pub seed_record: Option<String>,
    pub seed_words: Option<String>,
    /// SHA-256 of the per-frame pad file.
    pub pads: String,
    pub ticks: u32,
    pub facts: Vec<(String, FactResult)>,
    pub recorded: Recorded,
}

/// Frame numbers as ranges: `["118-121", "200"]`.
fn ranges(frames: &[u64]) -> String {
    let mut runs: Vec<(u64, u64)> = Vec::new();
    for &frame in frames {
        match runs.last_mut() {
            Some((_, last)) if *last + 1 == frame => *last = frame,
            _ => runs.push((frame, frame)),
        }
    }
    let quoted: Vec<String> = runs
        .iter()
        .map(|&(first, last)| {
            if first == last {
                format!("\"{first}\"")
            } else {
                format!("\"{first}-{last}\"")
            }
        })
        .collect();
    format!("[{}]", quoted.join(", "))
}

fn digests(marks: &[u64]) -> String {
    let quoted: Vec<String> = marks
        .iter()
        .map(|mark| format!("\"{mark:016x}\""))
        .collect();
    format!("[{}]", quoted.join(", "))
}

impl Entry {
    /// The entry as lock text: TOML, one key per line.
    pub fn to_toml(&self) -> String {
        let mut text = String::new();
        let recorded = &self.recorded;
        let _ = writeln!(text, "[[row]]");
        let _ = writeln!(text, "id = \"{}\"", self.id);
        let _ = writeln!(text, "start = \"{}\"", self.start);
        if let Some(sha) = &self.seed_record {
            let _ = writeln!(text, "seed_record = \"{sha}\"");
        }
        if let Some(sha) = &self.seed_words {
            let _ = writeln!(text, "seed_words = \"{sha}\"");
        }
        let _ = writeln!(text, "pads = \"{}\"", self.pads);
        let _ = writeln!(text, "frames = {}", recorded.frames);
        let _ = writeln!(text, "ticks = {}", self.ticks);
        let _ = writeln!(text, "overruns = {}", ranges(&recorded.overruns));
        let _ = writeln!(text, "nmi_off = {}", ranges(&recorded.nmi_off));
        if !recorded.still_running.is_empty() {
            let _ = writeln!(text, "still_running = {}", ranges(&recorded.still_running));
        }
        let facts: Vec<String> = self
            .facts
            .iter()
            .map(|(fact, result)| match result {
                FactResult::Holds(holds) => format!("\"{fact}\" = {holds}"),
                FactResult::Unevaluated => format!("\"{fact}\" = \"unevaluated\""),
            })
            .collect();
        let _ = writeln!(text, "facts = {{ {} }}", facts.join(", "));
        let absent: Vec<String> = recorded
            .streams
            .iter()
            .filter(|(_, marks)| marks.is_none())
            .map(|(name, _)| format!("\"{name}\""))
            .collect();
        if !absent.is_empty() {
            let _ = writeln!(text, "absent = [{}]", absent.join(", "));
        }
        for (name, marks) in &recorded.streams {
            if let Some(marks) = marks {
                let _ = writeln!(text, "streams.{name} = {}", digests(marks));
            }
        }
        text
    }
}

/// The file header: the key every entry below shares.
pub fn header(oracle: crate::OracleId, server_sha256: &str) -> String {
    let fixes: Vec<String> = oracle
        .fixes
        .iter()
        .map(|fix| format!("\"{fix}\""))
        .collect();
    format!(
        "format = {FORMAT}\nmark_frames = {MARK_FRAMES}\noracle = {{ emulator = \"{}\", version = \"{}\", server = \"{server_sha256}\", fixes = [{}] }}\n",
        oracle.emulator,
        oracle.version,
        fixes.join(", ")
    )
}

/// SHA-256 of a file's bytes, `None` if it doesn't exist.
pub fn file_sha256(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(crate::ladder::hex(&crate::ladder::sha256(&bytes)))
}

/// Row `id`'s facts from the scenario files in `directory` (every `.toml`
/// but the lock), in file order: from a `facts = [...]` on the line naming
/// `id = "ID"` (route files' inline rows), or on a later line of its
/// `[[scenario]]` block.
pub fn scenario_facts(directory: &Path, id: &str) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut paths: Vec<_> = listing
        .filter_map(|item| item.ok().map(|item| item.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "toml")
        })
        .filter(|path| path.file_name().is_some_and(|name| name != "lock.toml"))
        .collect();
    paths.sort();
    let named = format!("id = \"{id}\"");
    let facts_of = |line: &str| -> Option<Vec<String>> {
        let list = line.split_once("facts = [")?.1.split_once(']')?.0;
        Some(
            list.split(',')
                .map(|fact| fact.trim().trim_matches('"').to_owned())
                .filter(|fact| !fact.is_empty())
                .collect(),
        )
    };
    for path in paths {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut lines = text.lines();
        while let Some(line) = lines.next() {
            if !line.contains(&named) {
                continue;
            }
            if let Some(facts) = facts_of(line) {
                return facts;
            }
            return lines
                .take_while(|line| !line.trim_start().starts_with("[["))
                .find_map(facts_of)
                .unwrap_or_default();
        }
    }
    Vec::new()
}

/// Where a row's format-2 entry takes its pins from.
#[derive(Clone, Copy, Debug)]
pub struct Sources<'a> {
    /// The per-frame pad file the row played.
    pub pads: &'a Path,
    /// The native seeds' directory (`DIR/CAPTURE/NNNN.lss`).
    pub native_seeds: Option<&'a Path>,
    /// The scenario files' directory, for the row's facts.
    pub scenarios: &'a Path,
}

/// Row `entry`'s format-2 entry from its run's `recorded` marks.
pub fn entry(
    pin: &crate::lock::Pin,
    entry: &crate::lock::Entry,
    ticks: u32,
    recorded: Recorded,
    sources: Sources<'_>,
) -> Entry {
    let seed = pin.seed.as_deref().and_then(|seed| {
        let (capture, index) = seed.split_once(':')?;
        let index: u32 = index.parse().ok()?;
        let state = sources
            .native_seeds?
            .join(capture)
            .join(format!("{index:04}.lss"));
        Some((
            file_sha256(&state),
            file_sha256(&state.with_extension("words")),
        ))
    });
    let (seed_record, seed_words) = seed.unwrap_or_default();
    let facts = scenario_facts(sources.scenarios, &entry.id)
        .into_iter()
        .map(|fact| (fact, FactResult::Unevaluated))
        .collect();
    Entry {
        id: entry.id.clone(),
        start: pin.seed.clone().unwrap_or_else(|| "power-on".to_owned()),
        seed_record,
        seed_words,
        pads: file_sha256(sources.pads).unwrap_or_default(),
        ticks,
        facts,
        recorded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_collapse_runs() {
        assert_eq!(ranges(&[118, 119, 120, 121, 200]), "[\"118-121\", \"200\"]");
        assert_eq!(ranges(&[]), "[]");
    }
}
