//! The lockstep cache's reading side (docs/ORACLE.md, the record cache).
//!
//! `oracle corpus --cache DIR` writes each row's frame records to `DIR/<row>/`. `lean.bin`
//! holds the records as a chain of memory changes (`oracle_record::lean`). `lean.txt` starts
//! with [`LEAN_HEAD`] and holds the row's lock pins, the [`Recorder`] and one
//! `frame offset length cut` line per frame. A [`Cache`] opens a row's directory, refuses it
//! unless its pins are the lock row's, and gives its records in frame order, read from the
//! file a record at a time.
//!
//! A row recorded with watched instruction addresses also has `hits.txt` (a `# watched`
//! line, then `frame pc line dot peek` per hit); [`Cache::record`] puts each frame's hits
//! in its record, and [`Cache::watched`] names the addresses, so a frame without a hit on
//! one is known not to have reached it.

use crate::lock::{Frame, FrameCount, Row, Sha256};
use core::fmt;
use oracle_record::codec::CodecError;
use oracle_record::lean::{self, LEAN_HEAD};
use oracle_record::{Beam, Cut, FrameRecord, Hit, NmiKind};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

/// A row's index and records, under its directory.
const FILES: (&str, &str) = ("lean.txt", "lean.bin");

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
    /// The index is an older version of the layout and has to be recorded again.
    OlderFormat(PathBuf),
    /// The index isn't the layout [`LEAN_HEAD`] names.
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
    /// A frame's bytes lie outside `lean.bin`.
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
            Self::OlderFormat(path) => write!(
                f,
                "{}: the cache is in an older format and must be recorded again",
                path.display()
            ),
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

/// The oracle build a row's cache was recorded by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recorder {
    /// SHA-256 of the lsnes executable.
    pub lsnes: Sha256,
    /// SHA-256 of the server script it ran.
    pub server: Sha256,
}

/// One index line: where a frame's record lies and how its interval ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The frame.
    pub frame: Frame,
    /// Its record's first byte in `lean.bin`.
    pub offset: usize,
    /// Its record's length.
    pub length: usize,
    /// Its cut.
    pub cut: CutKind,
}

/// One row's cache, opened and checked.
#[derive(Debug)]
pub struct Cache {
    recorder: Recorder,
    entries: Vec<Entry>,
    /// The records' file and what reading it has kept, its path for errors, and its length.
    records: Mutex<Records>,
    records_path: PathBuf,
    bytes: usize,
    /// The addresses `hits.txt` watched, and its hits by frame; empty without it.
    watched: Vec<u32>,
    hits: BTreeMap<u64, Vec<Hit>>,
}

/// The records file, with the memories of the record at position `at`, the last one read.
#[derive(Debug)]
struct Records {
    file: File,
    memories: lean::Memories,
    at: Option<usize>,
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
    row: String,
    seed_record: Option<Sha256>,
    seed_words: Option<Sha256>,
    pads: Sha256,
    frames: FrameCount,
    recorder: Recorder,
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
            "" if line == LEAN_HEAD => Ok(line.to_owned()),
            "" if LEAN_HEAD
                .rsplit_once(' ')
                .is_some_and(|(layout, _)| line.starts_with(layout)) =>
            {
                Err(CacheError::OlderFormat(path.to_owned()))
            }
            "" => Err(malformed()),
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
    let recorder = Recorder {
        lsnes: sha(&next("lsnes")?, 7)?.ok_or_else(|| malformed(7))?,
        server: sha(&next("server")?, 8)?.ok_or_else(|| malformed(8))?,
    };
    let entries = text
        .lines()
        .enumerate()
        .skip(8)
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
        recorder,
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
        let (index_name, records_name) = FILES;
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
        Ok(Self {
            recorder: index.recorder,
            entries: index.entries,
            records: Mutex::new(Records {
                file: records,
                memories: lean::Memories::default(),
                at: None,
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

    /// The oracle build that recorded the row.
    #[must_use]
    pub const fn recorder(&self) -> Recorder {
        self.recorder
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
        self.read(position, &mut Vec::new())
    }

    /// Every frame's record, in frame order.
    pub fn records(&self) -> impl Iterator<Item = Result<(Frame, FrameRecord), CacheError>> + '_ {
        let mut bytes = Vec::new();
        self.entries
            .iter()
            .enumerate()
            .map(move |(position, entry)| {
                let record = self.read(position, &mut bytes)?;
                Ok((entry.frame, record))
            })
    }

    /// The record at `position` in the index, checked, with its hits; `bytes` keeps its
    /// allocation from one record to the next. A record that doesn't follow the last one
    /// read is reached from the whole record that starts its stretch of the chain.
    fn read(&self, position: usize, bytes: &mut Vec<u8>) -> Result<FrameRecord, CacheError> {
        let entry = self
            .entries
            .get(position)
            .ok_or(CacheError::OutOfRange(Frame(u64::MAX)))?;
        // A poisoned lock only means another reader panicked; the file is still whole.
        let mut records = self.records.lock().unwrap_or_else(PoisonError::into_inner);
        let Records { file, memories, at } = &mut *records;
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
            let record = lean::decode(bytes, memories).map_err(|error| CacheError::Record {
                frame: entry.frame,
                error,
            })?;
            *at = Some(link);
            last = Some(record);
        }
        let record = last.ok_or(CacheError::OutOfRange(entry.frame))?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock::{RowId, Start};
    use oracle_record::{CpuRegisters, Pad, PpuRegisters};
    use std::fmt::Write as _;

    const SHA: &str = "36f9967524b6959d84129fe8fc770ce83127c08b51bfaf8dd066208e9383a172";
    const LSNES: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const SERVER: &str = "2222222222222222222222222222222222222222222222222222222222222222";

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
            hits: Vec::new(),
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
        })
    }

    /// Writes `row`'s cache of `frames` invented records under `root`.
    fn lean_tier(root: &Path, row: &Row, frames: u64) -> std::io::Result<()> {
        let directory = root.join(&row.id.0);
        std::fs::create_dir_all(&directory)?;
        let mut records = Vec::new();
        let mut memories = lean::Memories::default();
        let mut index = format!(
            "{LEAN_HEAD}\nrow {}\nseed_record -\nseed_words -\npads {SHA}\nframes {frames}\nlsnes {LSNES}\nserver {SERVER}\n# frame offset length cut\n",
            row.id.0
        );
        for number in 0..frames {
            let offset = records.len();
            let whole = usize::try_from(number)
                .is_ok_and(|position| position.is_multiple_of(lean::KEY_INTERVAL));
            lean::encode(&frame(number), &mut memories, whole, &mut records);
            let length = records.len().saturating_sub(offset);
            let _ = writeln!(index, "{number} {offset} {length} V");
        }
        std::fs::write(directory.join(FILES.1), records)?;
        std::fs::write(directory.join(FILES.0), index)
    }

    /// Whether `a` and `b` are the same frame with the same memories.
    fn same(a: &FrameRecord, b: &FrameRecord) -> bool {
        (a.index, &a.wram, &a.vram, &a.cgram) == (b.index, &b.wram, &b.vram, &b.cgram)
            && (&a.oam, &a.sram) == (&b.oam, &b.sram)
    }

    #[test]
    fn the_records_read_in_order_and_out_of_it() {
        let key = u64::try_from(lean::KEY_INTERVAL).unwrap_or(u64::MAX);
        let frames = key.saturating_add(40);
        let row = row(frames).expect("invented row pins");
        let root = std::env::temp_dir().join(format!("verify-lean-tier-{}", std::process::id()));
        lean_tier(&root, &row, frames).expect("invented records");
        let cache = Cache::open(&root, &row).expect("the cache opens");
        let in_order: Vec<FrameRecord> = cache
            .records()
            .map(|record| record.expect("a record").1)
            .collect();
        assert_eq!(in_order.len(), usize::try_from(frames).unwrap_or(0));
        assert!(
            in_order
                .iter()
                .zip(0..)
                .all(|(read, number)| same(read, &frame(number)))
        );
        for position in [frames.saturating_sub(1), 3, key.saturating_add(1), 0] {
            let entry = usize::try_from(position)
                .ok()
                .and_then(|position| cache.entries().get(position))
                .expect("an entry");
            let read = cache.record(entry).expect("a record");
            assert!(same(&read, &frame(position)), "frame {position}");
        }
        drop(cache);
        std::fs::remove_dir_all(root).expect("fixture cleanup");
    }

    fn index(frames: &str) -> String {
        format!(
            "{LEAN_HEAD}\nrow area-brinstar\nseed_record {SHA}\nseed_words -\npads {SHA}\nframes {frames}\nlsnes {LSNES}\nserver {SERVER}\n# frame offset length cut\n0 0 10 R\n1 10 12 O\n"
        )
    }

    #[test]
    fn an_index_gives_its_pins_and_frames() {
        let parsed = parse_index(Path::new("lean.txt"), &index("2"));
        let Ok(parsed) = parsed else {
            return assert!(parsed.is_ok());
        };
        assert_eq!(parsed.row, "area-brinstar");
        assert_eq!(parsed.seed_record, Sha256::try_from(SHA).ok());
        assert_eq!(parsed.seed_words, None);
        assert_eq!(parsed.frames, FrameCount(2));
        assert_eq!(
            parsed.recorder,
            Recorder {
                lsnes: Sha256([0x11; 32]),
                server: Sha256([0x22; 32]),
            }
        );
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
        let parsed = parse_index(Path::new("lean.txt"), &index("two"));
        assert!(matches!(parsed, Err(CacheError::Malformed { line: 6, .. })));
        let unpinned = index("2").replacen(&format!("server {SERVER}"), "server -", 1);
        let parsed = parse_index(Path::new("lean.txt"), &unpinned);
        assert!(matches!(parsed, Err(CacheError::Malformed { line: 8, .. })));
    }

    #[test]
    fn an_index_of_an_older_layout_version_is_refused_as_older() {
        let older = index("2").replacen(LEAN_HEAD, "# sm-decomp record cache lean 1", 1);
        let parsed = parse_index(Path::new("lean.txt"), &older);
        assert!(matches!(parsed, Err(CacheError::OlderFormat(_))));
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
