//! The lockstep cache's writing side (docs/ORACLE.md, the record cache).
//!
//! `corpus --cache DIR` plays each row from its frame pads and writes every frame's record
//! to `DIR/<row>/`: `lean.bin`, the records as a chain of memory changes
//! (`oracle_record::lean`), and `lean.txt` under [`LEAN_HEAD`], the row's lock pins and one
//! line per frame (`frame offset length cut`). The index is removed first and written last,
//! so a row that failed part way has no index and a reader refuses it. ROM-derived: the
//! cache stays in `$TEMP`.
//!
//! A run that watched instruction addresses also writes `hits.txt`: a `# watched` line
//! naming them, then one `frame pc line dot peek` line per hit (hex pc and peek). The hits
//! stay out of `lean.bin`, so a row recorded again with watches has the same records
//! as without, and an earlier cache gains its hits by the file alone.
//!
//! `verify::cache` reads it.

use crate::record::lean;
use crate::record::{Cut, FrameRecord, NmiKind};
use std::fmt::Write as _;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};

/// The index's first line: the layout's name and version.
pub const LEAN_HEAD: &str = "# sm-decomp record cache lean 1";

/// The lock pins a row's cache is valid for, as the lock writes them.
#[derive(Clone, Debug, Default)]
pub struct Pins {
    /// SHA-256 of the native seed's state file.
    pub seed_record: Option<String>,
    /// SHA-256 of the native seed's WRAM edits.
    pub seed_words: Option<String>,
    /// SHA-256 of the row's frame pad file.
    pub pads: String,
}

impl From<&crate::lock::Row> for Pins {
    fn from(row: &crate::lock::Row) -> Self {
        Self {
            seed_record: row.seed_record.map(|sha| sha.to_string()),
            seed_words: row.seed_words.map(|sha| sha.to_string()),
            pads: row.pads.to_string(),
        }
    }
}

/// The letter the index names a record's cut by.
const fn cut_letter(record: &FrameRecord) -> char {
    match record.cut {
        Cut::NmiReturn(nmi) => match nmi.kind {
            NmiKind::Requested => 'R',
            NmiKind::Overrun => 'O',
        },
        Cut::VblankWithoutNmi => 'V',
        Cut::NmiStillRunning { .. } => 'S',
    }
}

/// One row's cache, being written.
pub struct Writer {
    directory: PathBuf,
    records: BufWriter<std::fs::File>,
    /// The memories of the last record written, which the next one's changes apply to.
    memories: lean::Memories,
    scratch: Vec<u8>,
    offset: u64,
    index: String,
    frames: u64,
    /// The watched addresses and the hits so far, when the run watches any.
    hits: Option<String>,
}

impl Writer {
    /// Start row `id`'s cache under `root`, removing any earlier index first.
    ///
    /// # Errors
    ///
    /// The file system's error.
    pub fn create(root: &Path, id: &str) -> std::io::Result<Self> {
        let directory = root.join(id);
        std::fs::create_dir_all(&directory)?;
        match std::fs::remove_file(directory.join("lean.txt")) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            Ok(()) | Err(_) => {}
        }
        let records = BufWriter::new(std::fs::File::create(directory.join("lean.bin"))?);
        Ok(Self {
            directory,
            records,
            memories: lean::Memories::default(),
            scratch: Vec::with_capacity(0x6_0000),
            offset: 0,
            index: String::new(),
            frames: 0,
            hits: None,
        })
    }

    /// Keep the hits on `addresses` in `hits.txt` (none watched: no file).
    #[must_use]
    pub fn watching(mut self, addresses: &[u32]) -> Self {
        self.hits = (!addresses.is_empty()).then(|| {
            let list: Vec<String> = addresses.iter().map(|pc| format!("{pc:06x}")).collect();
            format!("# watched {}\n# frame pc line dot peek\n", list.join(","))
        });
        self
    }

    /// Append the row's next frame.
    ///
    /// # Errors
    ///
    /// The file system's error.
    pub fn push(&mut self, record: &FrameRecord) -> std::io::Result<()> {
        self.scratch.clear();
        if let Some(hits) = self.hits.as_mut() {
            for hit in &record.hits {
                let _ = writeln!(
                    hits,
                    "{} {:06x} {} {} {:08x}",
                    self.frames,
                    hit.pc,
                    hit.beam.v,
                    hit.beam.h,
                    u32::from_le_bytes(hit.peek)
                );
            }
        }
        let whole = usize::try_from(self.frames)
            .is_ok_and(|position| position.is_multiple_of(lean::KEY_INTERVAL));
        lean::encode(record, &mut self.memories, whole, &mut self.scratch);
        self.records.write_all(&self.scratch)?;
        let length = u64::try_from(self.scratch.len()).unwrap_or(u64::MAX);
        let _ = writeln!(
            self.index,
            "{} {} {length} {}",
            self.frames,
            self.offset,
            cut_letter(record)
        );
        self.offset = self.offset.saturating_add(length);
        self.frames = self.frames.saturating_add(1);
        Ok(())
    }

    /// Finish the row: flush the records and write the index with `pins`.
    ///
    /// # Errors
    ///
    /// The file system's error.
    pub fn finish(mut self, id: &str, pins: &Pins) -> std::io::Result<u64> {
        self.records.flush()?;
        let none = || "-".to_owned();
        let text = format!(
            "{LEAN_HEAD}\nrow {id}\nseed_record {}\nseed_words {}\npads {}\nframes {}\npictures no\n# frame offset length cut\n{}",
            pins.seed_record.clone().unwrap_or_else(none),
            pins.seed_words.clone().unwrap_or_else(none),
            pins.pads,
            self.frames,
            self.index,
        );
        let hits_path = self.directory.join("hits.txt");
        match &self.hits {
            Some(hits) => std::fs::write(&hits_path, hits)?,
            None => match std::fs::remove_file(&hits_path) {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
                Ok(()) | Err(_) => {}
            },
        }
        std::fs::write(self.directory.join("lean.txt"), text)?;
        Ok(self.offset)
    }
}
