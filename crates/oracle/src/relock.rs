//! Lock format 2, the recording side (docs/ORACLE.md, RELOCK §2 and step c).
//!
//! A [`Recorder`] folds one row's video frames into span marks: each stream's
//! digest of every 1000-frame span, folded from empty (Q1), the last span
//! partial. A row's digest per stream is the fold of its marks
//! ([`data::digest::fold_all`]). The rules live in `data::span::SpanFold`, which the
//! engine's verifier folds through too, so both sides compute marks the same way.
//!
//! A cut is quiescent when it is the RTI of an NMI that found the main thread
//! at one of [`crate::game::WAIT_SITES`] ([`NmiKind::Requested`]). The memory,
//! `ppu`, `wram` and `apu` streams fold at quiescent cuts only; `video` and
//! `timing` fold every frame. The overrun and NMI-off schedules name the other
//! cuts, so a `Recorded` timing source folds the engine at the same frames.

use crate::record::{ApuAccessKind, Cut, FrameRecord, NmiKind, PpuRegisters};
use core::hash::{Hash, Hasher};
use data::digest::Digest;
use data::span::{self, Folded, Marks, PortAccess, PortDirection, SpanFold};
use std::fmt::Write as _;
use std::path::Path;

pub use data::span::{MARK_FRAMES, Stream};

/// The record format this module writes.
pub const FORMAT: u32 = 2;

/// A frame record as the shared span fold reads it.
struct RecordFrame<'a>(&'a FrameRecord);

impl span::Frame for RecordFrame<'_> {
    fn cut(&self) -> span::Cut {
        match self.0.cut {
            Cut::NmiReturn(nmi) => match nmi.kind {
                NmiKind::Requested => span::Cut::Requested,
                NmiKind::Overrun => span::Cut::Overrun,
            },
            Cut::VblankWithoutNmi => span::Cut::VblankWithoutNmi,
            Cut::NmiStillRunning { .. } => span::Cut::NmiStillRunning,
        }
    }

    fn memory(&self, stream: Stream) -> Option<u64> {
        let record = self.0;
        match stream {
            Stream::Vram => (record.vram.len() == 0x8000).then(|| span::vram_digest(&record.vram)),
            Stream::Cgram => {
                (record.cgram.len() == 0x100).then(|| span::cgram_digest(&record.cgram))
            }
            Stream::Oam => crate::streams::oam_digest(record),
            Stream::Sram => Some(span::bytes_digest(&record.sram)),
            Stream::Ppu => Some(ppu_digest(&record.ppu)),
            Stream::Wram => Some(span::bytes_digest(&record.wram)),
            Stream::Apu | Stream::Video | Stream::Timing => None,
        }
    }

    /// lsnes's bus hook sees no DMA side, so DMA entries are dropped on both sides.
    fn ports(&self) -> impl Iterator<Item = PortAccess> {
        self.0.apu_log.iter().filter_map(|access| {
            let direction = match access.kind {
                ApuAccessKind::CpuWrite => PortDirection::Write,
                ApuAccessKind::CpuRead => PortDirection::Read,
                ApuAccessKind::DmaWrite | ApuAccessKind::DmaRead | ApuAccessKind::SpcWrite => {
                    return None;
                }
            };
            Some(PortAccess {
                direction,
                port: access.port,
                value: access.value,
            })
        })
    }

    fn video(&self) -> Option<u64> {
        self.0
            .picture
            .as_ref()
            .map(|picture| span::bytes_digest(picture.shown().as_flattened()))
    }
}

/// The architectural PPU registers without the five lsnes doesn't expose
/// (docs/ORACLE.md, MEASUREMENTS (o); `nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel` are not
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

/// One row's fold, fed every frame record in order: the shared span fold
/// (`data::span`) plus the schedules, the typed streams and the sidecar.
#[derive(Default)]
pub struct Recorder {
    spans: SpanFold,
    typed: Vec<(Box<dyn TypedStream>, Marks)>,
    overruns: Vec<u64>,
    nmi_off: Vec<u64>,
    still_running: Vec<u64>,
    /// The row's sidecar text when asked for ([`Recorder::logging`]).
    frame_log: Option<String>,
}

/// The sidecar's first line: what each frame's line holds.
pub fn sidecar_head() -> String {
    let names: Vec<&str> = Stream::ALL.iter().map(|stream| stream.name()).collect();
    format!("# frame lsnes_frame cut {}\n", names.join(" "))
}

/// A cut's kind as the frame pads' `.cuts` files write it.
pub fn cut_letter(record: &FrameRecord) -> char {
    match record.cut {
        Cut::NmiReturn(nmi) => match nmi.kind {
            NmiKind::Requested => 'R',
            NmiKind::Overrun => 'O',
        },
        Cut::VblankWithoutNmi => 'V',
        Cut::NmiStillRunning { .. } => 'S',
    }
}

impl Recorder {
    /// The same recorder, also writing the per-frame sidecar: one line per
    /// frame with its index, lsnes's frame counter, the cut kind and the
    /// digest each stream folded at it (`-` where the stream didn't fold,
    /// `absent` where the record lacked it), so two runs diff frame by frame.
    #[must_use]
    pub fn logging(mut self) -> Self {
        self.frame_log = Some(sidecar_head());
        self
    }

    /// A recorder that also folds `typed` streams.
    pub fn with_typed(typed: Vec<Box<dyn TypedStream>>) -> Self {
        Self {
            typed: typed
                .into_iter()
                .map(|stream| (stream, Marks::default()))
                .collect(),
            ..Self::default()
        }
    }

    pub fn fold(&mut self, record: &FrameRecord) {
        let frame = self.spans.frames();
        match record.cut {
            Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Overrun => self.overruns.push(frame),
            Cut::NmiReturn(_) => {}
            Cut::VblankWithoutNmi => self.nmi_off.push(frame),
            Cut::NmiStillRunning { .. } => self.still_running.push(frame),
        }
        let folded = self.spans.fold(&RecordFrame(record));
        if let Some(log) = self.frame_log.as_mut() {
            let _ = write!(log, "{frame} {} {}", record.video_frame, cut_letter(record));
            for stream in folded.streams {
                match stream {
                    Folded::Digest(digest) => {
                        let _ = write!(log, " {digest:016x}");
                    }
                    Folded::Absent => log.push_str(" absent"),
                    Folded::Skipped => log.push_str(" -"),
                }
            }
            log.push('\n');
        }
        if folded.quiescent {
            for (stream, marks) in &mut self.typed {
                marks.fold(stream.digest(record));
            }
        }
        if folded.closed {
            self.typed.iter_mut().for_each(|(_, marks)| marks.close());
        }
    }

    /// The row's marks per stream, closing the partial last span.
    pub fn finish(mut self) -> Recorded {
        if self.spans.partial() {
            self.typed.iter_mut().for_each(|(_, marks)| marks.close());
        }
        let frames = self.spans.frames();
        let streams = self
            .spans
            .finish()
            .into_iter()
            .map(|(stream, marks)| (stream.name(), marks))
            .chain(
                self.typed
                    .iter()
                    .map(|(stream, marks)| (stream.name(), marks.marks().map(<[u64]>::to_vec))),
            )
            .collect();
        Recorded {
            frames,
            overruns: self.overruns,
            nmi_off: self.nmi_off,
            still_running: self.still_running,
            streams,
            frame_log: self.frame_log,
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
    /// The per-frame sidecar, when the recorder was [`Recorder::logging`].
    pub frame_log: Option<String>,
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

    fn marks(base: u64, count: u64) -> Option<Vec<u64>> {
        Some((1..=count).map(|span| base + span).collect())
    }

    /// The writer writes `verify`'s reader fixture byte for byte, so the reader's tests
    /// read exactly what this module writes (docs/ORACLE.md, RELOCK step f). `oracle` can't
    /// depend on `verify`, so the two share this file rather than a type.
    #[test]
    fn the_writer_writes_the_readers_fixture() {
        const ORACLE: crate::OracleId = crate::OracleId {
            emulator: "synthetic",
            version: "test 1",
            patches: &[],
            fixes: &["fix-a"],
        };
        let sha = |digit: &str| digit.repeat(64);
        let streams = |base: u64, count: u64, absent: Option<Stream>| {
            Stream::ALL
                .iter()
                .zip(1..)
                .map(|(stream, index)| {
                    let marks =
                        (absent != Some(*stream)).then(|| marks(base + index * 0x100, count));
                    (stream.name(), marks.flatten())
                })
                .collect()
        };
        let alpha = Entry {
            id: "row-alpha".to_owned(),
            start: "capture-a:3".to_owned(),
            seed_record: Some(sha("1")),
            seed_words: Some(sha("2")),
            pads: sha("3"),
            ticks: 1490,
            facts: vec![
                ("room:91F8".to_owned(), FactResult::Holds(true)),
                ("probe:example".to_owned(), FactResult::Unevaluated),
            ],
            recorded: Recorded {
                frames: 1500,
                overruns: vec![254, 256, 257],
                nmi_off: Vec::new(),
                still_running: Vec::new(),
                streams: streams(0, 2, None),
                frame_log: None,
            },
        };
        let beta = Entry {
            id: "row-beta".to_owned(),
            start: "power-on".to_owned(),
            seed_record: None,
            seed_words: None,
            pads: sha("4"),
            ticks: 1000,
            facts: Vec::new(),
            recorded: Recorded {
                frames: 1000,
                overruns: Vec::new(),
                nmi_off: vec![5, 6, 7, 8, 9],
                still_running: vec![12],
                streams: streams(0xa_0000, 1, Some(Stream::Video)),
                frame_log: None,
            },
        };
        let server = "00112233445566778899aabbccddeeff".repeat(2);
        let written = format!(
            "{}\n{}\n{}",
            header(ORACLE, &server),
            alpha.to_toml(),
            beta.to_toml()
        );
        assert_eq!(written, include_str!("../../verify/src/lock/format2.toml"));
    }
}
