//! The lockstep cache's reading side (docs/ORACLE.md, the record cache).
//!
//! `oracle corpus --cache DIR` writes each row's frame records to `DIR/<row>/`:
//! `records.bin`, the records in `oracle_record::codec`'s encoding one after another, and
//! `index.txt`, the row's lock pins and one `frame offset length cut` line per frame. A
//! [`Cache`] opens a row's directory, refuses it unless its pins are the lock row's, and
//! gives its records in frame order. [`fold`] folds them through the recorder's span rules,
//! so a cache can be checked against the lock it was taken for.

use crate::lock::{EngineRow, Frame, FrameCount, Row, Sha256, Stream};
use core::fmt;
use data::digest::Digest;
use data::span::{self, PortAccess, PortDirection, SpanFold};
use oracle_record::codec::{self, CodecError};
use oracle_record::{ApuAccessKind, Cut, FrameRecord, NmiKind, PpuRegisters};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The index's first line: the layout's name and version, as `oracle::cache` writes it.
pub const HEAD: &str = "# sm-decomp record cache 1";

/// How a cached frame's interval ended, as the index names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CutKind {
    /// The RTI of an NMI that found the main thread at a wait site (`R`).
    Requested,
    /// The RTI of an NMI that found it elsewhere, a lag frame (`O`).
    Overrun,
    /// Vblank with NMI disabled (`V`).
    VblankWithoutNmi,
    /// The end of vblank with the NMI still running (`S`).
    NmiStillRunning,
}

impl CutKind {
    /// The kind of `cut`.
    #[must_use]
    pub const fn of(cut: Cut) -> Self {
        match cut {
            Cut::NmiReturn(nmi) => match nmi.kind {
                NmiKind::Requested => Self::Requested,
                NmiKind::Overrun => Self::Overrun,
            },
            Cut::VblankWithoutNmi => Self::VblankWithoutNmi,
            Cut::NmiStillRunning { .. } => Self::NmiStillRunning,
        }
    }

    const fn from_letter(letter: &str) -> Option<Self> {
        match letter.as_bytes() {
            b"R" => Some(Self::Requested),
            b"O" => Some(Self::Overrun),
            b"V" => Some(Self::VblankWithoutNmi),
            b"S" => Some(Self::NmiStillRunning),
            _ => None,
        }
    }

    /// Whether the frame ends at a quiescent cut, where the memory streams fold.
    #[must_use]
    pub const fn quiescent(self) -> bool {
        matches!(self, Self::Requested)
    }
}

/// A lock pin a cache is checked against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    /// The native seed's state file.
    SeedRecord,
    /// The native seed's WRAM edits.
    SeedWords,
    /// The frame pad file.
    Pads,
    /// The row's video frames.
    Frames,
}

/// Why a row's cache can't be read.
#[derive(Debug)]
pub enum CacheError {
    /// A file couldn't be read.
    Io {
        /// The file.
        path: PathBuf,
        /// The error.
        error: std::io::Error,
    },
    /// The index isn't the layout [`HEAD`] names.
    Malformed {
        /// The index file.
        path: PathBuf,
        /// The 1-based line.
        line: usize,
    },
    /// The cache was taken for another row.
    WrongRow(String),
    /// A pin differs from the lock row's: the cache is stale.
    Stale(Pin),
    /// A frame's bytes lie outside `records.bin`.
    OutOfRange(Frame),
    /// A frame's bytes are not a record.
    Record {
        /// The frame.
        frame: Frame,
        /// Why.
        error: CodecError,
    },
    /// A record's index or cut isn't what the index says.
    Mismatch(Frame),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, error } => write!(f, "{}: {error}", path.display()),
            Self::Malformed { path, line } => write!(f, "{}:{line}: malformed", path.display()),
            Self::WrongRow(id) => write!(f, "the cache is row {id}'s"),
            Self::Stale(pin) => write!(f, "stale cache: {pin:?} differs from the lock"),
            Self::OutOfRange(Frame(frame)) => write!(f, "frame {frame}: outside the records"),
            Self::Record {
                frame: Frame(frame),
                error,
            } => write!(f, "frame {frame}: {error}"),
            Self::Mismatch(Frame(frame)) => write!(f, "frame {frame}: not the index's record"),
        }
    }
}

impl std::error::Error for CacheError {}

/// One index line: where a frame's record lies and how its interval ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The frame.
    pub frame: Frame,
    /// Its record's first byte in `records.bin`.
    pub offset: usize,
    /// Its record's length.
    pub length: usize,
    /// Its cut.
    pub cut: CutKind,
}

/// One row's cache, opened and checked.
#[derive(Debug)]
pub struct Cache {
    entries: Vec<Entry>,
    pictures: bool,
    records: Vec<u8>,
}

/// The pins and entries an index holds.
struct Index {
    row: String,
    seed_record: Option<Sha256>,
    seed_words: Option<Sha256>,
    pads: Sha256,
    frames: FrameCount,
    pictures: bool,
    entries: Vec<Entry>,
}

fn parse_index(path: &Path, text: &str) -> Result<Index, CacheError> {
    let mut lines = text.lines().enumerate();
    let mut next = |key: &str| -> Result<String, CacheError> {
        let (number, line) = lines.next().ok_or_else(|| CacheError::Malformed {
            path: path.to_owned(),
            line: 0,
        })?;
        let malformed = || CacheError::Malformed {
            path: path.to_owned(),
            line: number.saturating_add(1),
        };
        match key {
            "" => (line == HEAD).then(String::new).ok_or_else(malformed),
            key => line
                .strip_prefix(key)
                .and_then(|rest| rest.strip_prefix(' '))
                .map(str::to_owned)
                .ok_or_else(malformed),
        }
    };
    let malformed = |line: usize| CacheError::Malformed {
        path: path.to_owned(),
        line,
    };
    let sha = |text: &str, line: usize| match text {
        "-" => Ok(None),
        text => Sha256::try_from(text)
            .map(Some)
            .map_err(|_| malformed(line)),
    };
    next("")?;
    let row = next("row")?;
    let seed_record = sha(&next("seed_record")?, 3)?;
    let seed_words = sha(&next("seed_words")?, 4)?;
    let pads = sha(&next("pads")?, 5)?.ok_or_else(|| malformed(5))?;
    let frames = FrameCount(next("frames")?.parse().map_err(|_| malformed(6))?);
    let pictures = match next("pictures")?.as_str() {
        "yes" => true,
        "no" => false,
        _ => return Err(malformed(7)),
    };
    let entries = text
        .lines()
        .enumerate()
        .skip(7)
        .filter(|(_, line)| !line.starts_with('#'))
        .map(|(number, line)| {
            let bad = || malformed(number.saturating_add(1));
            let mut fields = line.split(' ');
            let mut field = || fields.next().ok_or_else(bad);
            let frame = Frame(field()?.parse().map_err(|_| bad())?);
            let offset = field()?.parse().map_err(|_| bad())?;
            let length = field()?.parse().map_err(|_| bad())?;
            let cut = CutKind::from_letter(field()?).ok_or_else(bad)?;
            Ok(Entry {
                frame,
                offset,
                length,
                cut,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Index {
        row,
        seed_record,
        seed_words,
        pads,
        frames,
        pictures,
        entries,
    })
}

fn read(path: PathBuf) -> Result<Vec<u8>, CacheError> {
    std::fs::read(&path).map_err(|error| CacheError::Io { path, error })
}

impl Cache {
    /// Row `row`'s cache under `root` (`root/<row>/`), checked against the row's pins.
    ///
    /// # Errors
    ///
    /// A [`CacheError`] when the files can't be read, the index is malformed or for
    /// another row, or a pin differs from the lock row's ([`CacheError::Stale`]).
    pub fn open(root: &Path, row: &Row) -> Result<Self, CacheError> {
        let directory = root.join(&row.id.0);
        let index_path = directory.join("index.txt");
        let text = read(index_path.clone())?;
        let text = String::from_utf8_lossy(&text);
        let index = parse_index(&index_path, &text)?;
        if index.row != row.id.0 {
            return Err(CacheError::WrongRow(index.row));
        }
        let pins = [
            (index.seed_record == row.seed_record, Pin::SeedRecord),
            (index.seed_words == row.seed_words, Pin::SeedWords),
            (index.pads == row.pads, Pin::Pads),
            (index.frames == row.frames, Pin::Frames),
            (
                u64::try_from(index.entries.len()).ok() == Some(row.frames.0),
                Pin::Frames,
            ),
        ];
        if let Some((_, pin)) = pins.iter().find(|(same, _)| !same) {
            return Err(CacheError::Stale(*pin));
        }
        let records = read(directory.join("records.bin"))?;
        Ok(Self {
            entries: index.entries,
            pictures: index.pictures,
            records,
        })
    }

    /// The cached frames' index lines, in frame order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Whether the records hold their pictures.
    #[must_use]
    pub const fn pictures(&self) -> bool {
        self.pictures
    }

    /// The bytes the records take.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.records.len()
    }

    /// The record of the frame `entry` indexes.
    ///
    /// # Errors
    ///
    /// A [`CacheError`] when its bytes are outside the records, not a record, or not the
    /// record the index names.
    pub fn record(&self, entry: &Entry) -> Result<FrameRecord, CacheError> {
        let bytes = entry
            .offset
            .checked_add(entry.length)
            .and_then(|end| self.records.get(entry.offset..end))
            .ok_or(CacheError::OutOfRange(entry.frame))?;
        let record = codec::decode(bytes).map_err(|error| CacheError::Record {
            frame: entry.frame,
            error,
        })?;
        if record.index != entry.frame.0 || CutKind::of(record.cut) != entry.cut {
            return Err(CacheError::Mismatch(entry.frame));
        }
        Ok(record)
    }

    /// Every frame's record, in frame order.
    pub fn records(&self) -> impl Iterator<Item = Result<(Frame, FrameRecord), CacheError>> + '_ {
        self.entries
            .iter()
            .map(|entry| self.record(entry).map(|record| (entry.frame, record)))
    }
}

/// A cached record as the span fold reads it, under the recorder's rules.
///
/// As `oracle::relock` reads a record: lsnes's bus hook sees no DMA side of the APU ports, so DMA and SPC
/// entries are dropped, and `ppu` leaves out the five registers lsnes doesn't expose.
pub struct RecordFrame<'a>(pub &'a FrameRecord);

impl span::Frame for RecordFrame<'_> {
    fn cut(&self) -> span::Cut {
        match CutKind::of(self.0.cut) {
            CutKind::Requested => span::Cut::Requested,
            CutKind::Overrun => span::Cut::Overrun,
            CutKind::VblankWithoutNmi => span::Cut::VblankWithoutNmi,
            CutKind::NmiStillRunning => span::Cut::NmiStillRunning,
        }
    }

    fn memory(&self, stream: span::Stream) -> Option<u64> {
        let record = self.0;
        match stream {
            span::Stream::Vram => {
                (record.vram.len() == 0x8000).then(|| span::vram_digest(&record.vram))
            }
            span::Stream::Cgram => {
                (record.cgram.len() == 0x100).then(|| span::cgram_digest(&record.cgram))
            }
            span::Stream::Oam => oam_digest(&record.oam),
            span::Stream::Sram => Some(span::bytes_digest(&record.sram)),
            span::Stream::Ppu => Some(ppu_digest(&record.ppu)),
            span::Stream::Wram => Some(span::bytes_digest(&record.wram)),
            span::Stream::Apu | span::Stream::Video | span::Stream::Timing => None,
        }
    }

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

/// OAM as the record holds it: 256 little-endian words of the low table, then the high
/// table's 32 bytes.
fn oam_digest(oam: &[u8]) -> Option<u64> {
    let (low, high) = oam.split_at_checked(0x200)?;
    let mut words = [0; 256];
    for (word, pair) in words.iter_mut().zip(low.as_chunks::<2>().0) {
        *word = u16::from_le_bytes(*pair);
    }
    Some(span::oam_digest(&words, high.try_into().ok()?))
}

/// The `ppu` stream's digest, the recorder's rule: the architectural registers in
/// declaration order without `nmitimen`, `htime`, `vtime`, `hdmaen` and `memsel`.
#[must_use]
pub fn ppu_digest(ppu: &PpuRegisters) -> u64 {
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
    span::hashed(&(
        (inidisp, obsel, bgmode, mosaic, bg_sc, bg12nba, bg34nba),
        (bg_scroll, m7sel, m7, window_select, window_positions),
        (
            wbglog,
            wobjlog,
            screens,
            cgwsel,
            cgadsub,
            fixed_color,
            setini,
        ),
        dma,
    ))
}

/// The row a cache's records fold to under the recorder's span rules: every machine
/// stream's marks, a stream some frame lacked left out. Facts are not evaluated.
///
/// # Errors
///
/// The first record that can't be read.
pub fn fold(cache: &Cache) -> Result<EngineRow, CacheError> {
    let mut spans = SpanFold::default();
    for item in cache.records() {
        let (_, record) = item?;
        spans.fold(&RecordFrame(&record));
    }
    let streams: BTreeMap<Stream, Vec<Digest>> = spans
        .finish()
        .into_iter()
        .filter_map(|(stream, marks)| {
            Some((
                Stream::from(stream),
                marks?.into_iter().map(Digest).collect(),
            ))
        })
        .collect();
    Ok(EngineRow {
        streams,
        facts: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "36f9967524b6959d84129fe8fc770ce83127c08b51bfaf8dd066208e9383a172";

    fn index(pictures: &str) -> String {
        format!(
            "{HEAD}\nrow area-brinstar\nseed_record {SHA}\nseed_words -\npads {SHA}\nframes 2\npictures {pictures}\n# frame offset length cut\n0 0 10 R\n1 10 12 O\n"
        )
    }

    #[test]
    fn an_index_gives_its_pins_and_frames() {
        let parsed = parse_index(Path::new("index.txt"), &index("no"));
        let Ok(parsed) = parsed else {
            return assert!(parsed.is_ok());
        };
        assert_eq!(parsed.row, "area-brinstar");
        assert_eq!(parsed.seed_record, Sha256::try_from(SHA).ok());
        assert_eq!(parsed.seed_words, None);
        assert_eq!(parsed.frames, FrameCount(2));
        assert!(!parsed.pictures);
        assert_eq!(
            parsed.entries.last(),
            Some(&Entry {
                frame: Frame(1),
                offset: 10,
                length: 12,
                cut: CutKind::Overrun,
            })
        );
    }

    #[test]
    fn a_malformed_index_names_its_line() {
        let parsed = parse_index(Path::new("index.txt"), &index("maybe"));
        assert!(matches!(parsed, Err(CacheError::Malformed { line: 7, .. })));
    }
}
