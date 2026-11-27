//! Reader for the format-2 oracle lock (docs/ORACLE.md, the lock).
//!
//! Capture and replay share this reader. It is strict: an unknown key, a duplicate key
//! or a duplicate row is an error.

mod text;

use core::fmt;
use text::{Line, Value};

/// The record format this reader reads.
pub const FORMAT: u64 = 2;

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
        Self(crate::rom::sha256(bytes))
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

impl fmt::Display for Start {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PowerOn => formatter.write_str("power-on"),
            Self::Seed { capture, index } => write!(formatter, "{capture}:{index}"),
        }
    }
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

/// The oracle that plays the lock's rows (D22).
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
    /// The oracle that plays the rows.
    pub oracle: Oracle,
    /// The ci-data bundles holding the seeds and pads.
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
    /// Not 64 lowercase hex digits.
    BadSha256(String),
    /// A start that is neither `power-on` nor `CAPTURE:INDEX`.
    BadStart(String),
    /// A row entered twice.
    DuplicateRow(RowId),
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
            Self::BadSha256(text) => write!(formatter, "not a SHA-256: `{text}`"),
            Self::BadStart(text) => write!(formatter, "not a start: `{text}`"),
            Self::DuplicateRow(id) => write!(formatter, "row `{id}` entered twice"),
        }
    }
}

/// A lock error and the line it was found on (1-based; the section head's line for a
/// key missing from it).
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
        other @ (Value::Int(_) | Value::Array(_) | Value::Table(_)) => {
            Err(wrong(key, "string", &other))
        }
    }
}

fn integer(key: &str, value: Value) -> Result<u64, ErrorKind> {
    match value {
        Value::Int(number) => Ok(number),
        other @ (Value::Str(_) | Value::Array(_) | Value::Table(_)) => {
            Err(wrong(key, "integer", &other))
        }
    }
}

fn array(key: &str, value: Value) -> Result<Vec<Value>, ErrorKind> {
    match value {
        Value::Array(items) => Ok(items),
        other @ (Value::Str(_) | Value::Int(_) | Value::Table(_)) => {
            Err(wrong(key, "array", &other))
        }
    }
}

fn table(key: &str, value: Value) -> Result<Vec<(String, Value)>, ErrorKind> {
    match value {
        Value::Table(pairs) => Ok(pairs),
        other @ (Value::Str(_) | Value::Int(_) | Value::Array(_)) => {
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

fn header(mut section: Section) -> Result<Header, LockError> {
    let format = section.required("format", integer)?;
    if format != FORMAT {
        return Err(section.error(ErrorKind::Format(format)));
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
    Ok(Header { oracle, bundles })
}

fn row(mut section: Section) -> Result<Row, LockError> {
    let row = Row {
        id: RowId(section.required("id", string)?),
        start: section.required("start", |key, value| {
            Start::try_from(string(key, value)?.as_str())
        })?,
        seed_record: section.optional("seed_record", sha256)?,
        seed_words: section.optional("seed_words", sha256)?,
        pads: section.required("pads", sha256)?,
        frames: FrameCount(section.required("frames", integer)?),
    };
    section.finish()?;
    Ok(row)
}

impl Lock {
    /// Parses format-2 lock text.
    ///
    /// # Errors
    ///
    /// The first thing in the text the format doesn't allow, with its line.
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
            let row = row(section)?;
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

    /// Row `id`'s entry.
    #[must_use]
    pub fn row(&self, id: &RowId) -> Option<&Row> {
        self.rows.iter().find(|row| row.id == *id)
    }
}
