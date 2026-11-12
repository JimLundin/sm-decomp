//! The format-2 lock (docs/ORACLE.md, RELOCK §2 and §8): its reader and the comparison of
//! an engine run's span marks with it, per (row, stream), with the row's green status (D22).
//!
//! `oracle::relock` writes the text; this reads exactly that. `verify` doesn't depend on
//! `oracle`, so the two share a fixture instead of a type: `lock/format2.toml` is what the
//! writer's test writes byte for byte and what this module's tests read.
//!
//! The reader is strict: an unknown key, a duplicate key or row, a stream neither marked
//! nor named `absent`, or a mark count that doesn't fit the row's frames is an error.

mod compare;
mod text;

#[cfg(test)]
mod tests;

pub use compare::{EngineRow, FactOutcome, Outcome, RowReport, Side, SpanIndex, StreamReport};

use core::fmt;
use data::digest::Digest;
use std::collections::BTreeMap;
use text::{Line, Value};

/// The record format this reader reads.
pub const FORMAT: u64 = 2;

/// A stream of the format-2 lock (docs/ORACLE.md, RELOCK §2 "Streams").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stream {
    /// Video RAM at quiescent cuts.
    Vram,
    /// Palette RAM at quiescent cuts.
    Cgram,
    /// Sprite attribute memory at quiescent cuts.
    Oam,
    /// Cartridge save RAM at quiescent cuts.
    Sram,
    /// The architectural PPU registers lsnes exposes, at quiescent cuts.
    Ppu,
    /// The CPU's APU port log, folded at each busy span's closing quiescent cut.
    Apu,
    /// The picture as shown, every frame.
    Video,
    /// Each frame's cut kind, NMI kind and overrun flag, every frame.
    Timing,
    /// Work RAM, unmasked; recorded but not compared.
    Wram,
    /// The decoded State's input projection.
    Input,
    /// The decoded State's Samus projection.
    Samus,
    /// The decoded State's camera projection.
    Camera,
    /// The decoded State's room projection.
    Room,
    /// The decoded State's progress projection.
    Progress,
    /// The decoded State's actors projection.
    Actors,
    /// The decoded State's effects projection.
    Effects,
    /// The decoded State's random number projection.
    Rng,
}

impl Stream {
    /// The streams the oracle's record gives directly, in the order an entry lists them;
    /// every entry names each one, with marks or as `absent`.
    pub const MACHINE: [Self; 9] = [
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

    /// The projections of the decoded oracle State (D21); an entry has them only once a
    /// decoder recorded them.
    pub const TYPED: [Self; 8] = [
        Self::Input,
        Self::Samus,
        Self::Camera,
        Self::Room,
        Self::Progress,
        Self::Actors,
        Self::Effects,
        Self::Rng,
    ];

    /// The name an entry's `streams.NAME` key uses.
    #[must_use]
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
            Self::Input => "input",
            Self::Samus => "samus",
            Self::Camera => "camera",
            Self::Room => "room",
            Self::Progress => "progress",
            Self::Actors => "actors",
            Self::Effects => "effects",
            Self::Rng => "rng",
        }
    }

    /// Whether the stream decides green (D22): every stream but `wram`, which is recorded
    /// for oracle determinism and the unowned-RAM report.
    #[must_use]
    pub const fn compared(self) -> bool {
        match self {
            Self::Wram => false,
            Self::Vram
            | Self::Cgram
            | Self::Oam
            | Self::Sram
            | Self::Ppu
            | Self::Apu
            | Self::Video
            | Self::Timing
            | Self::Input
            | Self::Samus
            | Self::Camera
            | Self::Room
            | Self::Progress
            | Self::Actors
            | Self::Effects
            | Self::Rng => true,
        }
    }

    /// Every stream, machine streams first.
    pub fn all() -> impl Iterator<Item = Self> {
        Self::MACHINE.into_iter().chain(Self::TYPED)
    }
}

impl TryFrom<&str> for Stream {
    type Error = ErrorKind;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        Self::all()
            .find(|stream| stream.name() == name)
            .ok_or_else(|| ErrorKind::UnknownStream(name.to_owned()))
    }
}

impl fmt::Display for Stream {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// A row's id, as the scenario files name it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RowId(pub String);

impl fmt::Display for RowId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A video frame of a row, counted from its first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame(pub u64);

/// A number of video frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrameCount(pub u64);

/// A number of requested NMIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ticks(pub u32);

/// A SHA-256, written as 64 lowercase hex digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Sha256(pub [u8; 32]);

const fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte.wrapping_sub(b'0')),
        b'a'..=b'f' => Some(byte.wrapping_sub(b'a').wrapping_add(10)),
        _ => None,
    }
}

impl Sha256 {
    /// The SHA-256 of `bytes`.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(data::rom::sha256(bytes))
    }
}

impl TryFrom<&str> for Sha256 {
    type Error = ErrorKind;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let bad = || ErrorKind::BadSha256(text.to_owned());
        let digits: Vec<u8> = text
            .bytes()
            .map(hex_digit)
            .collect::<Option<_>>()
            .ok_or_else(bad)?;
        if digits.len() != 64 {
            return Err(bad());
        }
        let mut bytes = [0; 32];
        for (byte, [high, low]) in bytes.iter_mut().zip(digits.as_chunks::<2>().0) {
            *byte = high.wrapping_shl(4) | low;
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
            .iter()
            .try_for_each(|byte| write!(formatter, "{byte:02x}"))
    }
}

/// A span mark: 16 lowercase hex digits.
fn mark(text: &str) -> Result<Digest, ErrorKind> {
    let bad = || ErrorKind::BadDigest(text.to_owned());
    if text.len() != 16 || !text.bytes().all(|byte| hex_digit(byte).is_some()) {
        return Err(bad());
    }
    u64::from_str_radix(text, 16).map(Digest).map_err(|_| bad())
}

/// Where a row starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Start {
    /// The console's power-on state.
    PowerOn,
    /// A native seed by its capture's name and index (`CAPTURE:INDEX`).
    Seed {
        /// The capture's name.
        capture: String,
        /// The seed's index in its capture.
        index: u32,
    },
}

impl TryFrom<&str> for Start {
    type Error = ErrorKind;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        if text == "power-on" {
            return Ok(Self::PowerOn);
        }
        let bad = || ErrorKind::BadStart(text.to_owned());
        let (capture, index) = text.split_once(':').ok_or_else(bad)?;
        if capture.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(bad());
        }
        Ok(Self::Seed {
            capture: capture.to_owned(),
            index: index.parse().map_err(|_| bad())?,
        })
    }
}

/// A fact's result on the oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FactResult {
    /// Evaluated on the decoded oracle State: whether it held.
    Holds(bool),
    /// Not evaluated on the oracle: `probe:` facts, and every fact until `tools relock`
    /// evaluates them on the decoded State (docs/ORACLE.md, RELOCK §6, what remains 1).
    Unevaluated,
}

/// One stream of an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamLock {
    /// One mark per span, the last partial.
    Marks(Vec<Digest>),
    /// Named in the entry's `absent`: some frame's record lacked the stream.
    Absent,
}

/// The oracle the lock was recorded by: part of every entry's key (D22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Oracle {
    /// The emulator, for example `lsnes-bsnes`.
    pub emulator: String,
    /// The emulator's pinned version.
    pub version: String,
    /// SHA-256 of the oracle's server script.
    pub server: Sha256,
    /// The named fixes applied to the emulator's sources.
    pub fixes: Vec<String>,
}

/// An encrypted ci-data bundle the lock pins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bundle {
    /// The bundle's file name in ci-data.
    pub name: String,
    /// SHA-256 of the bundle's ciphertext.
    pub sha256: Sha256,
}

/// The file's header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    /// Video frames per span mark.
    pub mark_frames: FrameCount,
    /// The oracle every entry was recorded by.
    pub oracle: Oracle,
    /// The decoder version, once typed streams are recorded.
    pub decoder: Option<String>,
    /// The ci-data bundles holding the seeds, pads and rungs.
    pub bundles: Vec<Bundle>,
}

/// One `[[row]]` entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The row's id.
    pub id: RowId,
    /// Where the row starts.
    pub start: Start,
    /// SHA-256 of the native seed's state file.
    pub seed_record: Option<Sha256>,
    /// SHA-256 of a derived seed's WRAM edits.
    pub seed_words: Option<Sha256>,
    /// SHA-256 of the per-frame pad file.
    pub pads: Sha256,
    /// The row's video frames.
    pub frames: FrameCount,
    /// The run's requested NMIs.
    pub ticks: Ticks,
    /// Frames whose NMI interrupted a non-wait site.
    pub overruns: Vec<Frame>,
    /// Frames cut at vblank without an NMI.
    pub nmi_off: Vec<Frame>,
    /// Frames cut with their NMI still running.
    pub still_running: Vec<Frame>,
    /// Frames with a native state in the bundle, for long rows.
    pub rungs: Vec<Frame>,
    /// The row's facts and their oracle results, in file order.
    pub facts: Vec<(String, FactResult)>,
    /// Every machine stream, and the typed streams recorded.
    pub streams: BTreeMap<Stream, StreamLock>,
}

impl Row {
    /// The stream's lock, `None` where the entry has none (a typed stream not recorded).
    #[must_use]
    pub fn stream(&self, stream: Stream) -> Option<&StreamLock> {
        self.streams.get(&stream)
    }

    /// The streams that decide the row's green (D22): those [`Stream::compared`] names that the
    /// lock holds marks for. `wram` is recorded but never compared, and a stream the lock
    /// lacks can't be equal, so neither counts toward a row's green.
    pub fn compared_streams(&self) -> impl Iterator<Item = Stream> + '_ {
        self.streams
            .iter()
            .filter(|(stream, lock)| stream.compared() && matches!(lock, StreamLock::Marks(_)))
            .map(|(&stream, _)| stream)
    }
}

/// A parsed format-2 lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock {
    /// The key the entries share.
    pub header: Header,
    /// The entries, in file order.
    pub rows: Vec<Row>,
}

/// What is wrong with a lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Text outside the lock's TOML subset.
    Syntax(&'static str),
    /// A format other than [`FORMAT`].
    Format(u64),
    /// A key the format doesn't have.
    UnknownKey(String),
    /// A key given twice in one section or table.
    DuplicateKey(String),
    /// A required key not given.
    MissingKey(&'static str),
    /// A value of the wrong type.
    WrongType {
        /// The key.
        key: String,
        /// The type the format gives the key.
        expected: &'static str,
        /// The type written.
        found: &'static str,
    },
    /// A section head other than `[[row]]`.
    UnknownSection(String),
    /// A stream name the format doesn't have.
    UnknownStream(String),
    /// Not 64 lowercase hex digits.
    BadSha256(String),
    /// A span mark that isn't 16 lowercase hex digits.
    BadDigest(String),
    /// A start that is neither `power-on` nor `CAPTURE:INDEX`.
    BadStart(String),
    /// A fact result that is neither a boolean nor `"unevaluated"`.
    BadFact(String),
    /// A schedule's frame range that isn't ascending, overlaps or lies past the row.
    BadSchedule(String),
    /// A machine stream neither marked nor `absent`.
    MissingStream(Stream),
    /// A stream both marked and `absent`, or named twice.
    DuplicateStream(Stream),
    /// A stream whose mark count isn't the row's span count.
    SpanCount {
        /// The stream.
        stream: Stream,
        /// The row's spans: its frames over `mark_frames`, rounded up.
        expected: u64,
        /// The marks written.
        found: usize,
    },
    /// A row entered twice.
    DuplicateRow(RowId),
    /// `mark_frames = 0`.
    ZeroMarkFrames,
    /// The lock's oracle isn't the one asked for.
    OracleMismatch,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(what) => write!(formatter, "syntax: {what}"),
            Self::Format(format) => write!(formatter, "format {format}, expected {FORMAT}"),
            Self::UnknownKey(key) => write!(formatter, "unknown key `{key}`"),
            Self::DuplicateKey(key) => write!(formatter, "duplicate key `{key}`"),
            Self::MissingKey(key) => write!(formatter, "missing key `{key}`"),
            Self::WrongType {
                key,
                expected,
                found,
            } => write!(formatter, "`{key}` is a {found}, expected a {expected}"),
            Self::UnknownSection(name) => write!(formatter, "unknown section `[[{name}]]`"),
            Self::UnknownStream(name) => write!(formatter, "unknown stream `{name}`"),
            Self::BadSha256(text) => write!(formatter, "not a SHA-256: `{text}`"),
            Self::BadDigest(text) => write!(formatter, "not a span mark: `{text}`"),
            Self::BadStart(text) => write!(formatter, "not a start: `{text}`"),
            Self::BadFact(fact) => write!(
                formatter,
                "fact `{fact}` is neither a boolean nor \"unevaluated\""
            ),
            Self::BadSchedule(text) => write!(formatter, "bad schedule entry `{text}`"),
            Self::MissingStream(stream) => {
                write!(formatter, "stream `{stream}` neither marked nor absent")
            }
            Self::DuplicateStream(stream) => write!(formatter, "stream `{stream}` named twice"),
            Self::SpanCount {
                stream,
                expected,
                found,
            } => write!(
                formatter,
                "stream `{stream}` has {found} marks, expected {expected}"
            ),
            Self::DuplicateRow(id) => write!(formatter, "row `{id}` entered twice"),
            Self::ZeroMarkFrames => formatter.write_str("mark_frames is 0"),
            Self::OracleMismatch => formatter.write_str("the lock was recorded by another oracle"),
        }
    }
}

/// A lock error and the line it was found on (1-based; the section head's line for a
/// key missing from it, 0 for the file as a whole).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockError {
    /// The line, 1-based.
    pub line: usize,
    /// What is wrong.
    pub kind: ErrorKind,
}

impl fmt::Display for LockError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "lock line {}: {}", self.line, self.kind)
    }
}

impl std::error::Error for LockError {}

/// One section's pairs, taken out by key; whatever is left is unknown.
struct Section {
    line: usize,
    pairs: Vec<(usize, String, Value)>,
}

impl Section {
    const fn new(line: usize) -> Self {
        Self {
            line,
            pairs: Vec::new(),
        }
    }

    fn push(&mut self, line: usize, key: String, value: Value) -> Result<(), LockError> {
        if self.pairs.iter().any(|(_, known, _)| *known == key) {
            return Err(LockError {
                line,
                kind: ErrorKind::DuplicateKey(key),
            });
        }
        self.pairs.push((line, key, value));
        Ok(())
    }

    const fn error(&self, kind: ErrorKind) -> LockError {
        LockError {
            line: self.line,
            kind,
        }
    }

    fn take(&mut self, key: &str) -> Option<(usize, Value)> {
        let found = self.pairs.iter().position(|(_, known, _)| known == key)?;
        let (line, _, value) = self.pairs.remove(found);
        Some((line, value))
    }

    /// The value at `key`, converted; `None` if the key is absent.
    fn optional<T>(
        &mut self,
        key: &str,
        convert: impl FnOnce(&str, Value) -> Result<T, ErrorKind>,
    ) -> Result<Option<T>, LockError> {
        self.take(key)
            .map(|(line, value)| convert(key, value).map_err(|kind| LockError { line, kind }))
            .transpose()
    }

    fn required<T>(
        &mut self,
        key: &'static str,
        convert: impl FnOnce(&str, Value) -> Result<T, ErrorKind>,
    ) -> Result<T, LockError> {
        self.optional(key, convert)?
            .ok_or_else(|| self.error(ErrorKind::MissingKey(key)))
    }

    /// Errors on the first pair no one took.
    fn finish(self) -> Result<(), LockError> {
        match self.pairs.into_iter().next() {
            Some((line, key, _)) => Err(LockError {
                line,
                kind: ErrorKind::UnknownKey(key),
            }),
            None => Ok(()),
        }
    }
}

fn wrong(key: &str, expected: &'static str, value: &Value) -> ErrorKind {
    ErrorKind::WrongType {
        key: key.to_owned(),
        expected,
        found: value.kind(),
    }
}

fn string(key: &str, value: Value) -> Result<String, ErrorKind> {
    match value {
        Value::Str(text) => Ok(text),
        other @ (Value::Int(_) | Value::Bool(_) | Value::Array(_) | Value::Table(_)) => {
            Err(wrong(key, "string", &other))
        }
    }
}

fn integer(key: &str, value: Value) -> Result<u64, ErrorKind> {
    match value {
        Value::Int(number) => Ok(number),
        other @ (Value::Str(_) | Value::Bool(_) | Value::Array(_) | Value::Table(_)) => {
            Err(wrong(key, "integer", &other))
        }
    }
}

fn array(key: &str, value: Value) -> Result<Vec<Value>, ErrorKind> {
    match value {
        Value::Array(items) => Ok(items),
        other @ (Value::Str(_) | Value::Int(_) | Value::Bool(_) | Value::Table(_)) => {
            Err(wrong(key, "array", &other))
        }
    }
}

fn table(key: &str, value: Value) -> Result<Vec<(String, Value)>, ErrorKind> {
    match value {
        Value::Table(pairs) => Ok(pairs),
        other @ (Value::Str(_) | Value::Int(_) | Value::Bool(_) | Value::Array(_)) => {
            Err(wrong(key, "table", &other))
        }
    }
}

fn strings(key: &str, value: Value) -> Result<Vec<String>, ErrorKind> {
    array(key, value)?
        .into_iter()
        .map(|item| string(key, item))
        .collect()
}

fn sha256(key: &str, value: Value) -> Result<Sha256, ErrorKind> {
    Sha256::try_from(string(key, value)?.as_str())
}

/// An inline table read as a [`Section`], so its keys are checked as strictly.
fn inline_section(line: usize, key: &str, value: Value) -> Result<Section, LockError> {
    let mut section = Section::new(line);
    let pairs = table(key, value).map_err(|kind| LockError { line, kind })?;
    for (key, value) in pairs {
        section.push(line, key, value)?;
    }
    Ok(section)
}

/// A schedule: `["118-121", "200"]`, strictly ascending runs, as the writer collapses them.
fn schedule(key: &str, value: Value) -> Result<Vec<Frame>, ErrorKind> {
    let mut frames: Vec<Frame> = Vec::new();
    for text in strings(key, value)? {
        let bad = || ErrorKind::BadSchedule(text.clone());
        let number = |part: &str| -> Result<u64, ErrorKind> {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(bad());
            }
            part.parse().map_err(|_| bad())
        };
        let (first, last) = match text.split_once('-') {
            Some((first, last)) => (number(first)?, number(last)?),
            None => (number(&text)?, number(&text)?),
        };
        // A run follows the last one with a gap, or the writer would have joined them.
        let after_gap = frames
            .last()
            .is_none_or(|previous| previous.0.checked_add(1).is_some_and(|next| next < first));
        let single = text.contains('-') == (first < last);
        if !after_gap || first > last || !single {
            return Err(bad());
        }
        frames.extend((first..=last).map(Frame));
    }
    Ok(frames)
}

fn frame_list(key: &str, value: Value) -> Result<Vec<Frame>, ErrorKind> {
    let frames: Vec<Frame> = array(key, value)?
        .into_iter()
        .map(|item| integer(key, item).map(Frame))
        .collect::<Result<_, _>>()?;
    if frames
        .windows(2)
        .all(|pair| matches!(pair, [a, b] if a < b))
    {
        Ok(frames)
    } else {
        Err(ErrorKind::BadSchedule(key.to_owned()))
    }
}

fn facts(key: &str, value: Value) -> Result<Vec<(String, FactResult)>, ErrorKind> {
    let pairs = table(key, value)?;
    let mut facts: Vec<(String, FactResult)> = Vec::with_capacity(pairs.len());
    for (fact, result) in pairs {
        if facts.iter().any(|(known, _)| *known == fact) {
            return Err(ErrorKind::DuplicateKey(fact));
        }
        let result = match result {
            Value::Bool(holds) => FactResult::Holds(holds),
            Value::Str(text) if text == "unevaluated" => FactResult::Unevaluated,
            Value::Str(_) | Value::Int(_) | Value::Array(_) | Value::Table(_) => {
                return Err(ErrorKind::BadFact(fact));
            }
        };
        facts.push((fact, result));
    }
    Ok(facts)
}

fn header(mut section: Section) -> Result<Header, LockError> {
    let format = section.required("format", integer)?;
    if format != FORMAT {
        return Err(section.error(ErrorKind::Format(format)));
    }
    let mark_frames = FrameCount(section.required("mark_frames", integer)?);
    if mark_frames.0 == 0 {
        return Err(section.error(ErrorKind::ZeroMarkFrames));
    }
    let (line, value) = section
        .take("oracle")
        .ok_or_else(|| section.error(ErrorKind::MissingKey("oracle")))?;
    let mut fields = inline_section(line, "oracle", value)?;
    let oracle = Oracle {
        emulator: fields.required("emulator", string)?,
        version: fields.required("version", string)?,
        server: fields.required("server", sha256)?,
        fixes: fields.required("fixes", strings)?,
    };
    fields.finish()?;
    let decoder = section.optional("decoder", string)?;
    let bundles = match section.take("bundles") {
        None => Vec::new(),
        Some((line, value)) => array("bundles", value)
            .map_err(|kind| LockError { line, kind })?
            .into_iter()
            .map(|item| {
                let mut fields = inline_section(line, "bundles", item)?;
                let bundle = Bundle {
                    name: fields.required("name", string)?,
                    sha256: fields.required("sha256", sha256)?,
                };
                fields.finish().map(|()| bundle)
            })
            .collect::<Result<_, _>>()?,
    };
    section.finish()?;
    Ok(Header {
        mark_frames,
        oracle,
        decoder,
        bundles,
    })
}

fn row(mut section: Section, mark_frames: FrameCount) -> Result<Row, LockError> {
    let id = RowId(section.required("id", string)?);
    let start = section.required("start", |key, value| {
        Start::try_from(string(key, value)?.as_str())
    })?;
    let seed_record = section.optional("seed_record", sha256)?;
    let seed_words = section.optional("seed_words", sha256)?;
    let pads = section.required("pads", sha256)?;
    let frames = FrameCount(section.required("frames", integer)?);
    let ticks = section.required("ticks", |key, value| {
        let ticks = integer(key, value)?;
        u32::try_from(ticks)
            .map(Ticks)
            .map_err(|_| ErrorKind::Syntax("ticks out of range"))
    })?;
    let overruns = section.required("overruns", schedule)?;
    let nmi_off = section.required("nmi_off", schedule)?;
    let still_running = section
        .optional("still_running", schedule)?
        .unwrap_or_default();
    let rungs = section.optional("rungs", frame_list)?.unwrap_or_default();
    for (name, list) in [
        ("overruns", &overruns),
        ("nmi_off", &nmi_off),
        ("still_running", &still_running),
        ("rungs", &rungs),
    ] {
        if list.last().is_some_and(|last| last.0 >= frames.0) {
            return Err(section.error(ErrorKind::BadSchedule(name.to_owned())));
        }
    }
    let facts = section.required("facts", facts)?;
    let mut streams = BTreeMap::new();
    let absent = section.optional("absent", strings)?.unwrap_or_default();
    for name in absent {
        let stream = Stream::try_from(name.as_str()).map_err(|kind| section.error(kind))?;
        if streams.insert(stream, StreamLock::Absent).is_some() {
            return Err(section.error(ErrorKind::DuplicateStream(stream)));
        }
    }
    let expected = frames.0.div_ceil(mark_frames.0);
    for stream in Stream::all() {
        let key = format!("streams.{}", stream.name());
        let Some((line, value)) = section.take(&key) else {
            continue;
        };
        let at = |kind| LockError { line, kind };
        let marks: Vec<Digest> = strings(&key, value)
            .map_err(at)?
            .iter()
            .map(|text| mark(text))
            .collect::<Result<_, _>>()
            .map_err(at)?;
        if u64::try_from(marks.len()).ok() != Some(expected) {
            return Err(at(ErrorKind::SpanCount {
                stream,
                expected,
                found: marks.len(),
            }));
        }
        if streams.insert(stream, StreamLock::Marks(marks)).is_some() {
            return Err(at(ErrorKind::DuplicateStream(stream)));
        }
    }
    // A `streams.NAME` left here names no stream.
    if let Some((line, key, _)) = section
        .pairs
        .iter()
        .find(|(_, key, _)| key.starts_with("streams."))
    {
        let name = key.trim_start_matches("streams.").to_owned();
        return Err(LockError {
            line: *line,
            kind: ErrorKind::UnknownStream(name),
        });
    }
    if let Some(stream) = Stream::MACHINE
        .into_iter()
        .find(|stream| !streams.contains_key(stream))
    {
        return Err(section.error(ErrorKind::MissingStream(stream)));
    }
    section.finish()?;
    Ok(Row {
        id,
        start,
        seed_record,
        seed_words,
        pads,
        frames,
        ticks,
        overruns,
        nmi_off,
        still_running,
        rungs,
        facts,
        streams,
    })
}

impl Lock {
    /// Parses format-2 lock text.
    ///
    /// # Errors
    ///
    /// The first thing in the text the writer wouldn't have written, with its line.
    pub fn parse(text: &str) -> Result<Self, LockError> {
        let mut head = Section::new(0);
        let mut rows: Vec<Section> = Vec::new();
        for (index, source) in text.lines().enumerate() {
            let number = index.saturating_add(1);
            let parsed = text::line(source).map_err(|kind| LockError { line: number, kind })?;
            match parsed {
                None => {}
                Some(Line::ArrayHead(name)) if name == "row" => rows.push(Section::new(number)),
                Some(Line::ArrayHead(name)) => {
                    return Err(LockError {
                        line: number,
                        kind: ErrorKind::UnknownSection(name),
                    });
                }
                Some(Line::Pair { key, value }) => {
                    rows.last_mut()
                        .unwrap_or(&mut head)
                        .push(number, key, value)?;
                }
            }
        }
        let header = header(head)?;
        let mut parsed: Vec<Row> = Vec::with_capacity(rows.len());
        for section in rows {
            let line = section.line;
            let row = row(section, header.mark_frames)?;
            if parsed.iter().any(|known| known.id == row.id) {
                return Err(LockError {
                    line,
                    kind: ErrorKind::DuplicateRow(row.id),
                });
            }
            parsed.push(row);
        }
        Ok(Self {
            header,
            rows: parsed,
        })
    }

    /// Parses `text` and checks it was recorded by `oracle` (D22: a lock is keyed by
    /// row, oracle and oracle version).
    ///
    /// # Errors
    ///
    /// As [`Lock::parse`], and [`ErrorKind::OracleMismatch`] for another oracle.
    pub fn parse_for(text: &str, oracle: &Oracle) -> Result<Self, LockError> {
        let lock = Self::parse(text)?;
        if lock.header.oracle == *oracle {
            Ok(lock)
        } else {
            Err(LockError {
                line: 0,
                kind: ErrorKind::OracleMismatch,
            })
        }
    }

    /// Row `id`'s entry.
    #[must_use]
    pub fn row(&self, id: &RowId) -> Option<&Row> {
        self.rows.iter().find(|row| row.id == *id)
    }
}
