//! The lockstep cache's reading side (docs/ORACLE.md, the record cache).
//!
//! `oracle corpus --cache DIR` writes each row's frame records to `DIR/<row>/`:
//! `records.bin`, the records in `oracle_record::codec`'s encoding one after another, and
//! `index.txt`, the row's lock pins and one `frame offset length cut` line per frame. A
//! [`Cache`] opens a row's directory, refuses it unless its pins are the lock row's, and
//! gives its records in frame order, read from the file a record at a time: a row's records
//! run to hundreds of megabytes. [`fold`] folds them through the recorder's span rules,
//! so a cache can be checked against the lock it was taken for.
//!
//! A row recorded with watched instruction addresses also has `hits.txt` (a `# watched`
//! line, then `frame pc line dot peek` per hit); [`Cache::record`] puts each frame's hits
//! in its record, and [`Cache::watched`] names the addresses, so a frame without a hit on
//! one is known not to have reached it.
//!
//! A row may instead hold the lean tier (`oracle_record::lean`): `lean.bin`, the records
//! without their pictures as a chain of memory changes, and `lean.txt`, an index of the same
//! layout under [`LEAN_HEAD`]. [`Cache::open`] reads the lean tier when the row has one,
//! the full tier otherwise; a reader that needs pictures needs a full tier taken with them.
//! [`Cache::write_lean`] writes a row's lean tier from either.

use crate::lock::{EngineRow, Frame, FrameCount, Row, Sha256, Stream};
use core::fmt;
use data::digest::Digest;
use data::span::{self, PortAccess, PortDirection, SpanFold};
use oracle_record::codec::{self, CodecError, PictureMode};
use oracle_record::lean;
use oracle_record::{ApuAccessKind, Beam, Cut, FrameRecord, Hit, NmiKind, PpuRegisters};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};

/// The index's first line: the layout's name and version, as `oracle::cache` writes it.
pub const HEAD: &str = "# sm-decomp record cache 1";
/// The lean tier's index's first line.
pub const LEAN_HEAD: &str = "# sm-decomp record cache lean 1";

/// The full tier's index and records, under a row's directory.
const FULL_FILES: (&str, &str) = ("index.txt", "records.bin");
/// The lean tier's.
const LEAN_FILES: (&str, &str) = ("lean.txt", "lean.bin");

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

    /// The index's letter for the kind.
    const fn letter(self) -> char {
        match self {
            Self::Requested => 'R',
            Self::Overrun => 'O',
            Self::VblankWithoutNmi => 'V',
            Self::NmiStillRunning => 'S',
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
    directory: PathBuf,
    entries: Vec<Entry>,
    pictures: bool,
    /// The records' file and how it holds them, its path for errors, and its length.
    records: Mutex<Records>,
    records_path: PathBuf,
    bytes: usize,
    /// The addresses `hits.txt` watched, and its hits by frame; empty without it.
    watched: Vec<u32>,
    hits: BTreeMap<u64, Vec<Hit>>,
}

/// A tier's records file, and what reading it has kept.
#[derive(Debug)]
struct Records {
    file: File,
    tier: Tier,
}

/// How a row's records file holds its records.
#[derive(Debug)]
enum Tier {
    /// Each record whole, in `codec`'s encoding.
    Full,
    /// A lean chain, with the memories of the record at position `at`, the last one read.
    Lean {
        memories: lean::Memories,
        at: Option<usize>,
    },
}

/// The watched addresses and the hits by frame of a `hits.txt`.
type Hits = (Vec<u32>, BTreeMap<u64, Vec<Hit>>);

fn parse_hits(path: &Path, text: &str) -> Result<Hits, CacheError> {
    let malformed = |line: usize| CacheError::Malformed {
        path: path.to_owned(),
        line,
    };
    let mut lines = text.lines().enumerate();
    let watched = lines
        .next()
        .and_then(|(_, line)| line.strip_prefix("# watched "))
        .ok_or_else(|| malformed(1))?
        .split(',')
        .map(|pc| u32::from_str_radix(pc, 16).map_err(|_| malformed(1)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut hits: BTreeMap<u64, Vec<Hit>> = BTreeMap::new();
    for (number, line) in lines.filter(|(_, line)| !line.starts_with('#')) {
        let bad = || malformed(number.saturating_add(1));
        let fields: Vec<&str> = line.split(' ').collect();
        let [frame, pc, v, h, peek] = fields[..] else {
            return Err(bad());
        };
        let hit = Hit {
            pc: u32::from_str_radix(pc, 16).map_err(|_| bad())?,
            beam: Beam {
                v: v.parse().map_err(|_| bad())?,
                h: h.parse().map_err(|_| bad())?,
            },
            peek: u32::from_str_radix(peek, 16)
                .map_err(|_| bad())?
                .to_le_bytes(),
        };
        hits.entry(frame.parse().map_err(|_| bad())?)
            .or_default()
            .push(hit);
    }
    Ok((watched, hits))
}

/// The pins and entries an index holds.
struct Index {
    lean: bool,
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
            "" => [HEAD, LEAN_HEAD]
                .contains(&line)
                .then(|| line.to_owned())
                .ok_or_else(malformed),
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
    let lean = next("")? == LEAN_HEAD;
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
        lean,
        row,
        seed_record,
        seed_words,
        pads,
        frames,
        pictures,
        entries,
    })
}

/// Whether `directory` holds a row's cache, in either tier.
#[must_use]
pub fn holds_row(directory: &Path) -> bool {
    [FULL_FILES, LEAN_FILES]
        .iter()
        .any(|(index, _)| directory.join(index).is_file())
}

fn read(path: PathBuf) -> Result<Vec<u8>, CacheError> {
    std::fs::read(&path).map_err(|error| CacheError::Io { path, error })
}

/// Files staged on the destination volume, removed on every exit path.
struct LeanWrite {
    staging: PathBuf,
    installed: Vec<PathBuf>,
}

impl LeanWrite {
    fn new(directory: &Path) -> Result<Self, CacheError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let staging = directory.join(format!(
                ".lean-write-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&staging) {
                Ok(()) => {
                    return Ok(Self {
                        staging,
                        installed: Vec::new(),
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(CacheError::Io {
                        path: staging,
                        error,
                    });
                }
            }
        }
    }

    /// A hard link publishes a whole file without replacing any existing path.
    fn install(&mut self, directory: &Path, name: &str) -> Result<(), CacheError> {
        let path = directory.join(name);
        std::fs::hard_link(self.staging.join(name), &path).map_err(|error| CacheError::Io {
            path: path.clone(),
            error,
        })?;
        self.installed.push(path);
        Ok(())
    }
}

impl Drop for LeanWrite {
    fn drop(&mut self) {
        for path in &self.installed {
            let _ = std::fs::remove_file(path);
        }
        let _ = std::fs::remove_dir_all(&self.staging);
    }
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
        let (index_name, records_name) = if directory.join(LEAN_FILES.0).is_file() {
            LEAN_FILES
        } else {
            FULL_FILES
        };
        let index_path = directory.join(index_name);
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
        let records_path = directory.join(records_name);
        let io = |error| CacheError::Io {
            path: records_path.clone(),
            error,
        };
        let records = File::open(&records_path).map_err(io)?;
        let bytes = records.metadata().map_err(io)?.len();
        let bytes = usize::try_from(bytes).map_err(|_| CacheError::Io {
            path: records_path.clone(),
            error: std::io::ErrorKind::FileTooLarge.into(),
        })?;
        let hits_path = directory.join("hits.txt");
        let (watched, hits) = match std::fs::read(&hits_path) {
            Ok(text) => parse_hits(&hits_path, &String::from_utf8_lossy(&text))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (Vec::new(), BTreeMap::new())
            }
            Err(error) => {
                return Err(CacheError::Io {
                    path: hits_path,
                    error,
                });
            }
        };
        let tier = if index.lean {
            Tier::Lean {
                memories: lean::Memories::default(),
                at: None,
            }
        } else {
            Tier::Full
        };
        Ok(Self {
            directory,
            entries: index.entries,
            pictures: index.pictures,
            records: Mutex::new(Records {
                file: records,
                tier,
            }),
            records_path,
            bytes,
            watched,
            hits,
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

    /// The instruction addresses the row was recorded watching (none without `hits.txt`).
    #[must_use]
    pub fn watched(&self) -> &[u32] {
        &self.watched
    }

    /// The bytes the records take.
    #[must_use]
    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    /// The record of the frame `entry` indexes.
    ///
    /// # Errors
    ///
    /// A [`CacheError`] when its bytes are outside the records, not a record, or not the
    /// record the index names.
    pub fn record(&self, entry: &Entry) -> Result<FrameRecord, CacheError> {
        let position = self
            .entries
            .binary_search_by_key(&entry.frame, |entry| entry.frame)
            .map_err(|_| CacheError::OutOfRange(entry.frame))?;
        self.read(position, PictureMode::Keep, &mut Vec::new())
    }

    /// Every frame's record, in frame order.
    pub fn records(&self) -> impl Iterator<Item = Result<(Frame, FrameRecord), CacheError>> + '_ {
        self.records_with(PictureMode::Keep)
    }

    /// Every frame's record without its picture, in frame order, for a reader that never
    /// looks at one: the picture is a third of a record.
    pub fn records_without_pictures(
        &self,
    ) -> impl Iterator<Item = Result<(Frame, FrameRecord), CacheError>> + '_ {
        self.records_with(PictureMode::Drop)
    }

    fn records_with(
        &self,
        pictures: PictureMode,
    ) -> impl Iterator<Item = Result<(Frame, FrameRecord), CacheError>> + '_ {
        let mut bytes = Vec::new();
        self.entries
            .iter()
            .enumerate()
            .map(move |(position, entry)| {
                let record = self.read(position, pictures, &mut bytes)?;
                Ok((entry.frame, record))
            })
    }

    /// The record at `position` in the index, checked, with its hits; `bytes` keeps its
    /// allocation from one record to the next. A lean record that doesn't follow the last
    /// one read is reached from the whole record that starts its stretch of the chain.
    fn read(
        &self,
        position: usize,
        pictures: PictureMode,
        bytes: &mut Vec<u8>,
    ) -> Result<FrameRecord, CacheError> {
        let entry = self
            .entries
            .get(position)
            .ok_or(CacheError::OutOfRange(Frame(u64::MAX)))?;
        // A poisoned lock only means another reader panicked; the file is still whole.
        let mut records = self.records.lock().unwrap_or_else(PoisonError::into_inner);
        let Records { file, tier } = &mut *records;
        let record = match tier {
            Tier::Full => {
                self.read_bytes(file, entry, bytes)?;
                codec::decode_with(bytes, pictures).map_err(|error| CacheError::Record {
                    frame: entry.frame,
                    error,
                })?
            }
            Tier::Lean { memories, at } => {
                let follows = at.and_then(|at| at.checked_add(1)) == Some(position);
                let first = if follows {
                    position
                } else {
                    position.saturating_sub(position % lean::KEY_INTERVAL)
                };
                *at = None;
                let mut last = None;
                for (link, entry) in self
                    .entries
                    .iter()
                    .enumerate()
                    .take(position.saturating_add(1))
                    .skip(first)
                {
                    self.read_bytes(file, entry, bytes)?;
                    let record =
                        lean::decode(bytes, memories).map_err(|error| CacheError::Record {
                            frame: entry.frame,
                            error,
                        })?;
                    *at = Some(link);
                    last = Some(record);
                }
                last.ok_or(CacheError::OutOfRange(entry.frame))?
            }
        };
        drop(records);
        self.checked(entry, record)
    }

    /// Reads the bytes of the record `entry` indexes from `file` into `bytes`.
    fn read_bytes(
        &self,
        file: &mut File,
        entry: &Entry,
        bytes: &mut Vec<u8>,
    ) -> Result<(), CacheError> {
        let within = entry
            .offset
            .checked_add(entry.length)
            .is_some_and(|end| end <= self.bytes);
        let start = u64::try_from(entry.offset)
            .ok()
            .filter(|_| within)
            .ok_or(CacheError::OutOfRange(entry.frame))?;
        bytes.resize(entry.length, 0);
        file.seek(SeekFrom::Start(start))
            .and_then(|_| file.read_exact(bytes))
            .map_err(|error| CacheError::Io {
                path: self.records_path.clone(),
                error,
            })
    }

    /// `record`, read for `entry`, checked against the index, with its hits.
    fn checked(&self, entry: &Entry, record: FrameRecord) -> Result<FrameRecord, CacheError> {
        if record.index != entry.frame.0 || CutKind::of(record.cut) != entry.cut {
            return Err(CacheError::Mismatch(entry.frame));
        }
        let mut record = record;
        if let Some(hits) = self.hits.get(&entry.frame.0) {
            record.hits.clone_from(hits);
        }
        Ok(record)
    }

    /// Writes this cache's lean tier for `row` under `root` (`root/<row>/`, with `hits.txt`
    /// copied beside it); returns the bytes the lean records take. Existing lean files
    /// are refused, including an in-place retry and path aliases. Conversion is staged
    /// before publishing the index last, preserving an existing full tier on failure.
    ///
    /// # Errors
    ///
    /// A [`CacheError`] when a record can't be read, a file can't be written, lean files
    /// already exist, or the destination has different instruction hits. The destination
    /// filesystem must support hard links; otherwise publication fails without replacing files.
    pub fn write_lean(&self, row: &Row, root: &Path) -> Result<usize, CacheError> {
        let directory = root.join(&row.id.0);
        let io = |path: &Path| {
            let path = path.to_owned();
            move |error| CacheError::Io { path, error }
        };
        std::fs::create_dir_all(&directory).map_err(io(&directory))?;
        for name in <[&str; 2]>::from(LEAN_FILES) {
            let path = directory.join(name);
            match std::fs::symlink_metadata(&path) {
                Ok(_) => return Err(io(&path)(std::io::ErrorKind::AlreadyExists.into())),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(io(&path)(error)),
            }
        }
        let hits_path = self.directory.join("hits.txt");
        let optional_hits = |path: &Path| match std::fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io(path)(error)),
        };
        let hits = optional_hits(&hits_path)?;
        let destination_hits = directory.join("hits.txt");
        let existing_hits = optional_hits(&destination_hits)?;
        if existing_hits.is_some() && existing_hits != hits {
            return Err(io(&destination_hits)(
                std::io::ErrorKind::AlreadyExists.into(),
            ));
        }
        let mut pending = LeanWrite::new(&directory)?;
        let index_path = pending.staging.join(LEAN_FILES.0);
        let records_path = pending.staging.join(LEAN_FILES.1);
        let mut records = BufWriter::new(File::create(&records_path).map_err(io(&records_path))?);
        let none = || "-".to_owned();
        let shown = |pin: Option<Sha256>| pin.as_ref().map_or_else(none, ToString::to_string);
        let mut index = format!(
            "{LEAN_HEAD}\nrow {}\nseed_record {}\nseed_words {}\npads {}\nframes {}\npictures no\n# frame offset length cut\n",
            row.id.0,
            shown(row.seed_record),
            shown(row.seed_words),
            row.pads,
            row.frames.0,
        );
        let mut memories = lean::Memories::default();
        let (mut bytes, mut offset) = (Vec::new(), 0_usize);
        for (position, item) in self.records_without_pictures().enumerate() {
            let (frame, record) = item?;
            bytes.clear();
            lean::encode(
                &record,
                &mut memories,
                position.is_multiple_of(lean::KEY_INTERVAL),
                &mut bytes,
            );
            records.write_all(&bytes).map_err(io(&records_path))?;
            let cut = CutKind::of(record.cut).letter();
            let _ = writeln!(index, "{} {offset} {} {cut}", frame.0, bytes.len());
            offset = offset.saturating_add(bytes.len());
        }
        records.flush().map_err(io(&records_path))?;
        drop(records);
        if let Some(hits) = &hits {
            let path = pending.staging.join("hits.txt");
            std::fs::write(&path, hits).map_err(io(&path))?;
        }
        std::fs::write(&index_path, index).map_err(io(&index_path))?;
        pending.install(&directory, LEAN_FILES.1)?;
        if hits.is_some() && existing_hits.is_none() {
            pending.install(&directory, "hits.txt")?;
        }
        pending.install(&directory, LEAN_FILES.0)?;
        pending.installed.clear();
        Ok(offset)
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
    use crate::lock::{RowId, Start, Ticks};
    use oracle_record::{CpuRegisters, Pad, Picture};

    const SHA: &str = "36f9967524b6959d84129fe8fc770ce83127c08b51bfaf8dd066208e9383a172";

    /// Frame `index` of an invented row, whose WRAM byte $123 and VRAM word $40 count frames
    /// and WRAM byte $200 counts hundreds of them, so a frame's changes alone don't make it.
    fn frame(index: u64) -> FrameRecord {
        let count = u8::try_from(index % 251).unwrap_or_default();
        let mut wram = vec![0x55; 0x400];
        if let Some(byte) = wram.get_mut(0x123) {
            *byte = count;
        }
        if let Some(byte) = wram.get_mut(0x200) {
            *byte = u8::try_from(index / 100).unwrap_or_default();
        }
        let mut vram: Box<[u16]> = (0..0x100).collect();
        if let Some(word) = vram.get_mut(0x40) {
            *word = u16::from(count);
        }
        FrameRecord {
            index,
            video_frame: 0,
            pad: Pad(0),
            cut: Cut::VblankWithoutNmi,
            joypad_reads: 0,
            wait_entry: None,
            overscan: false,
            cpu: CpuRegisters::default(),
            wram: wram.into_boxed_slice(),
            vram,
            cgram: vec![0x7FFF; 0x100].into_boxed_slice(),
            oam: vec![1; 0x220].into_boxed_slice(),
            sram: vec![2; 0x20].into_boxed_slice(),
            ppu: PpuRegisters::default(),
            apu_log: Vec::new(),
            picture: Some(Picture::default()),
            spc: None,
            bank_instructions: None,
            writes: Vec::new(),
            hits: Vec::new(),
            legacy_ppu_digest: 0,
        }
    }

    fn row(frames: u64) -> Option<Row> {
        Some(Row {
            id: RowId("lean-tier".to_owned()),
            start: Start::PowerOn,
            seed_record: None,
            seed_words: None,
            pads: Sha256::try_from(SHA).ok()?,
            frames: FrameCount(frames),
            ticks: Ticks(0),
            overruns: Vec::new(),
            nmi_off: Vec::new(),
            still_running: Vec::new(),
            rungs: Vec::new(),
            facts: Vec::new(),
            streams: BTreeMap::new(),
        })
    }

    /// Writes `row`'s full tier of `frames` invented records under `root`.
    fn full_tier(root: &Path, row: &Row, frames: u64) -> std::io::Result<()> {
        let directory = root.join(&row.id.0);
        std::fs::create_dir_all(&directory)?;
        let mut records = Vec::new();
        let mut index = format!(
            "{HEAD}\nrow {}\nseed_record -\nseed_words -\npads {SHA}\nframes {frames}\npictures yes\n# frame offset length cut\n",
            row.id.0
        );
        for number in 0..frames {
            let offset = records.len();
            codec::encode(&frame(number), PictureMode::Keep, &mut records);
            let _ = writeln!(
                index,
                "{number} {offset} {} V",
                records.len().saturating_sub(offset)
            );
        }
        std::fs::write(directory.join("records.bin"), records)?;
        std::fs::write(directory.join("index.txt"), index)
    }

    /// Whether `a` and `b` are the same frame with the same memories.
    fn same(a: &FrameRecord, b: &FrameRecord) -> bool {
        (a.index, &a.wram, &a.vram, &a.cgram) == (b.index, &b.wram, &b.vram, &b.cgram)
            && (&a.oam, &a.sram) == (&b.oam, &b.sram)
    }

    #[test]
    fn the_lean_tier_reads_as_the_full_tier_without_pictures_in_order_and_out_of_it() {
        let key = u64::try_from(lean::KEY_INTERVAL).unwrap_or(u64::MAX);
        let frames = key.saturating_add(40);
        let Some(row) = row(frames) else {
            return assert!(Sha256::try_from(SHA).is_ok());
        };
        let root = std::env::temp_dir().join(format!("verify-lean-tier-{}", std::process::id()));
        let (full, lean) = (root.join("full"), root.join("lean"));
        assert!(full_tier(&full, &row, frames).is_ok());
        let full = Cache::open(&full, &row);
        let written = full.as_ref().ok().map(|full| full.write_lean(&row, &lean));
        let lean = Cache::open(&lean, &row);
        let (Ok(full), Some(Ok(written)), Ok(lean)) = (full, written, lean) else {
            let _ = std::fs::remove_dir_all(&root);
            panic!("the tiers open");
        };
        assert_eq!(lean.bytes(), written);
        assert!(lean.bytes() * 10 < full.bytes());
        let expected: Vec<FrameRecord> = full
            .records_without_pictures()
            .filter_map(Result::ok)
            .map(|(_, record)| record)
            .collect();
        let in_order: Vec<FrameRecord> = lean
            .records()
            .filter_map(Result::ok)
            .map(|(_, record)| record)
            .collect();
        assert_eq!(in_order.len(), expected.len());
        assert!(
            in_order
                .iter()
                .zip(&expected)
                .all(|(lean, full)| same(lean, full) && lean.picture.is_none())
        );
        for position in [frames.saturating_sub(1), 3, key.saturating_add(1), 0] {
            let entry = usize::try_from(position)
                .ok()
                .and_then(|position| lean.entries().get(position));
            let read = entry.map(|entry| lean.record(entry));
            let want = usize::try_from(position)
                .ok()
                .and_then(|position| expected.get(position));
            let same = read
                .and_then(Result::ok)
                .zip(want)
                .is_some_and(|(read, want)| same(&read, want));
            assert!(same, "frame {position}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn retrying_a_lean_conversion_preserves_the_existing_tier() {
        let row = row(3).expect("invented row pins");
        let root = std::env::temp_dir().join(format!("verify-lean-retry-{}", std::process::id()));
        let full_root = root.join("full");
        let lean_root = root.join("lean");
        full_tier(&full_root, &row, 3).expect("invented full records");
        Cache::open(&full_root, &row)
            .expect("full cache")
            .write_lean(&row, &lean_root)
            .expect("first conversion");
        let directory = lean_root.join(&row.id.0);
        let before = <[&str; 2]>::from(LEAN_FILES)
            .map(|name| std::fs::read(directory.join(name)).expect("lean file"));
        let cache = Cache::open(&lean_root, &row).expect("lean cache");
        std::fs::create_dir(lean_root.join("alias")).expect("alias directory");
        for destination in [
            lean_root.clone(),
            lean_root.join("."),
            lean_root.join("alias/.."),
        ] {
            assert!(
                matches!(cache.write_lean(&row, &destination), Err(CacheError::Io { error, .. }) if error.kind() == std::io::ErrorKind::AlreadyExists)
            );
        }
        let after = <[&str; 2]>::from(LEAN_FILES).map(|name| std::fs::read(directory.join(name)));
        assert!(
            after
                .iter()
                .zip(&before)
                .all(|(after, before)| after.as_ref().is_ok_and(|after| after == before)),
            "retry preserves lean files"
        );
        let reopened = Cache::open(&lean_root, &row).expect("preserved lean cache");
        assert!(
            reopened
                .records()
                .enumerate()
                .all(|(number, record)| record.is_ok_and(|(_, record)| same(
                    &record,
                    &frame(u64::try_from(number).expect("frame"))
                )))
        );
        drop((cache, reopened));
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }

    #[test]
    fn a_full_tier_converts_in_place_and_its_lean_tier_converts_elsewhere() {
        let row = row(3).expect("invented row pins");
        let root =
            std::env::temp_dir().join(format!("verify-lean-in-place-{}", std::process::id()));
        let full_root = root.join("full");
        full_tier(&full_root, &row, 3).expect("invented full records");
        let directory = full_root.join(&row.id.0);
        let full_before = <[&str; 2]>::from(FULL_FILES)
            .map(|name| std::fs::read(directory.join(name)).expect("full file"));
        let hits = b"# watched 90bf25\n1 90bf25 40 512 0000a201\n";
        std::fs::write(directory.join("hits.txt"), hits).expect("invented hits");
        let full = Cache::open(&full_root, &row).expect("full cache");
        full.write_lean(&row, &full_root)
            .expect("in-place conversion");
        for (name, before) in <[&str; 2]>::from(FULL_FILES).iter().zip(&full_before) {
            assert_eq!(
                std::fs::read(directory.join(name)).expect("full file"),
                *before
            );
        }
        let lean = Cache::open(&full_root, &row).expect("lean cache");
        let other_root = root.join("other");
        lean.write_lean(&row, &other_root)
            .expect("lean to other directory");
        let other = Cache::open(&other_root, &row).expect("other cache");
        let expected = lean
            .records()
            .collect::<Result<Vec<_>, _>>()
            .expect("lean records");
        let actual = other
            .records()
            .collect::<Result<Vec<_>, _>>()
            .expect("copied records");
        assert_eq!(actual.len(), 3);
        for ((_, actual), (_, expected)) in actual.iter().zip(&expected) {
            let mut a = Vec::new();
            let mut b = Vec::new();
            codec::encode(actual, PictureMode::Keep, &mut a);
            codec::encode(expected, PictureMode::Keep, &mut b);
            assert_eq!(a, b);
            assert_eq!(actual.hits, expected.hits);
        }
        assert_eq!(
            std::fs::read(other_root.join(&row.id.0).join("hits.txt")).expect("copied hits"),
            hits
        );
        drop((full, lean, other));
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }

    #[test]
    fn a_decode_failure_preserves_an_existing_full_destination() {
        let row = row(3).expect("invented row pins");
        let root = std::env::temp_dir().join(format!("verify-lean-failure-{}", std::process::id()));
        let source = root.join("source");
        let destination = root.join("destination");
        full_tier(&source, &row, 3).expect("source records");
        full_tier(&destination, &row, 3).expect("destination records");
        let hits = b"# watched 90bf25\n1 90bf25 40 512 0000a201\n";
        for directory in [&source, &destination] {
            std::fs::write(directory.join(&row.id.0).join("hits.txt"), hits).expect("hits");
        }
        let cache = Cache::open(&source, &row).expect("source cache");
        let second = cache.entries().get(1).expect("second frame");
        let mut damaged = std::fs::OpenOptions::new()
            .write(true)
            .open(source.join(&row.id.0).join(FULL_FILES.1))
            .expect("source file");
        damaged
            .seek(SeekFrom::Start(
                u64::try_from(second.offset).expect("offset"),
            ))
            .expect("second record");
        damaged.write_all(&[0; 8]).expect("invalid record header");
        drop(damaged);
        let directory = destination.join(&row.id.0);
        let names = [FULL_FILES.0, FULL_FILES.1, "hits.txt"];
        let before =
            names.map(|name| std::fs::read(directory.join(name)).expect("destination file"));
        assert!(matches!(
            cache.write_lean(&row, &destination),
            Err(CacheError::Record {
                frame: Frame(1),
                ..
            })
        ));
        for (name, before) in names.iter().zip(&before) {
            assert_eq!(
                std::fs::read(directory.join(name)).expect("preserved file"),
                *before
            );
        }
        assert_eq!(
            std::fs::read_dir(&directory)
                .expect("destination directory")
                .count(),
            3
        );
        let preserved = Cache::open(&destination, &row).expect("destination cache");
        assert_eq!(
            preserved
                .records()
                .collect::<Result<Vec<_>, _>>()
                .expect("preserved records")
                .len(),
            3
        );
        drop((cache, preserved));
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }

    #[test]
    fn an_incomplete_lean_destination_is_preserved() {
        let row = row(3).expect("invented row pins");
        let root =
            std::env::temp_dir().join(format!("verify-lean-incomplete-{}", std::process::id()));
        let source = root.join("source");
        full_tier(&source, &row, 3).expect("source records");
        let cache = Cache::open(&source, &row).expect("source cache");
        for name in <[&str; 2]>::from(LEAN_FILES) {
            let destination = root.join(name);
            let directory = destination.join(&row.id.0);
            std::fs::create_dir_all(&directory).expect("destination directory");
            std::fs::write(directory.join(name), b"existing evidence").expect("existing file");
            assert!(
                matches!(cache.write_lean(&row, &destination), Err(CacheError::Io { error, .. }) if error.kind() == std::io::ErrorKind::AlreadyExists)
            );
            assert_eq!(
                std::fs::read(directory.join(name)).expect("preserved file"),
                b"existing evidence"
            );
            assert_eq!(
                std::fs::read_dir(directory)
                    .expect("destination directory")
                    .count(),
                1
            );
        }
        drop(cache);
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }

    #[test]
    fn a_publication_collision_rolls_back_only_new_files() {
        let row = row(3).expect("invented row pins");
        let root =
            std::env::temp_dir().join(format!("verify-lean-publication-{}", std::process::id()));
        full_tier(&root, &row, 3).expect("full destination");
        let directory = root.join(&row.id.0);
        let index = b"existing index";
        let hits = b"# watched 90bf25\n";
        let staging;
        {
            let mut pending = LeanWrite::new(&directory).expect("staging directory");
            staging = pending.staging.clone();
            std::fs::write(staging.join(LEAN_FILES.1), b"new records").expect("staged records");
            std::fs::write(staging.join(LEAN_FILES.0), b"new index").expect("staged index");
            pending
                .install(&directory, LEAN_FILES.1)
                .expect("published records");
            // Another writer claims the index before this writer can publish it.
            std::fs::write(directory.join(LEAN_FILES.0), index).expect("existing index");
            std::fs::write(directory.join("hits.txt"), hits).expect("existing hits");
            assert!(
                matches!(pending.install(&directory, LEAN_FILES.0), Err(CacheError::Io { error, .. }) if error.kind() == std::io::ErrorKind::AlreadyExists)
            );
        }
        assert!(!staging.exists());
        assert!(!directory.join(LEAN_FILES.1).exists());
        assert_eq!(
            std::fs::read(directory.join(LEAN_FILES.0)).expect("preserved index"),
            index
        );
        assert_eq!(
            std::fs::read(directory.join("hits.txt")).expect("preserved hits"),
            hits
        );
        // The colliding writer owns its index; removing it reveals the unchanged full tier.
        std::fs::remove_file(directory.join(LEAN_FILES.0)).expect("collision cleanup");
        let preserved = Cache::open(&root, &row).expect("preserved full destination");
        assert_eq!(
            preserved
                .records()
                .collect::<Result<Vec<_>, _>>()
                .expect("full records")
                .len(),
            3
        );
        drop(preserved);
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }

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

    #[test]
    fn a_hits_file_gives_the_watched_addresses_and_each_frames_hits() {
        let text = "# watched 90bf25,8081a6\n# frame pc line dot peek\n3 90bf25 40 512 0000a201\n3 8081a6 41 8 00000000\n7 90bf25 2 0 00000000\n";
        let parsed = parse_hits(Path::new("hits.txt"), text);
        let Ok((watched, hits)) = parsed else {
            return assert!(parsed.is_ok());
        };
        assert_eq!(watched, [0x90_BF25, 0x80_81A6]);
        assert_eq!(hits.keys().copied().collect::<Vec<_>>(), [3, 7]);
        assert_eq!(
            hits.get(&3).and_then(|frame| frame.first()),
            Some(&Hit {
                pc: 0x90_BF25,
                beam: Beam { v: 40, h: 512 },
                peek: [0x01, 0xA2, 0x00, 0x00],
            })
        );
        assert_eq!(hits.get(&3).map(Vec::len), Some(2));
    }

    #[test]
    fn a_malformed_hit_names_its_line() {
        let text = "# watched 90bf25\n# frame pc line dot peek\n3 90bf25 40\n";
        let parsed = parse_hits(Path::new("hits.txt"), text);
        assert!(matches!(parsed, Err(CacheError::Malformed { line: 3, .. })));
    }
}
