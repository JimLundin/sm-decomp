//! The lockstep cache's writing side (docs/ORACLE.md, the record cache).
//!
//! `corpus --cache DIR` plays each row from its frame pads and writes every frame's record
//! to `DIR/<row>/`: `records.bin`, the records one after another in their own encoding
//! (`oracle_record::codec`), and `index.txt`, the row's lock pins and one line per frame
//! (`frame offset length cut`). The index is written last, so a row that failed part way
//! has no index and a reader refuses it. ROM-derived: the cache stays in `$TEMP`.
//!
//! A run that watched instruction addresses also writes `hits.txt`: a `# watched` line
//! naming them, then one `frame pc line dot peek` line per hit (hex pc and peek). The hits
//! stay out of `records.bin`, so a row recorded again with watches has the same records
//! as without, and an earlier cache gains its hits by the file alone.
//!
//! Without pictures the writer takes the lean tier instead (`oracle_record::lean`): the
//! records as a chain of memory changes in `lean.bin`, indexed by `lean.txt` under
//! [`LEAN_HEAD`], a fraction of the bytes. With them it takes the full tier above. Either
//! way it removes both indexes first, so a row holds only the tier last written whole.
//!
//! `verify::cache` reads it.

use crate::record::FrameRecord;
use crate::record::codec::{self, PictureMode};
use crate::record::lean;
use std::fmt::Write as _;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};

/// The index's first line: the layout's name and version.
pub const HEAD: &str = "# sm-decomp record cache 1";
/// The lean tier's index's first line.
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

/// The tier a writer takes.
enum Tier {
    /// Each record whole, with its picture.
    Full,
    /// The lean chain, with the memories of the last record written.
    Lean(lean::Memories),
}

/// One row's cache, being written.
pub struct Writer {
    directory: PathBuf,
    records: BufWriter<std::fs::File>,
    tier: Tier,
    scratch: Vec<u8>,
    offset: u64,
    index: String,
    frames: u64,
    /// The watched addresses and the hits so far, when the run watches any.
    hits: Option<String>,
}

impl Writer {
    /// Start row `id`'s cache under `root`, removing any earlier index first: the full tier
    /// when `pictures` keeps them, the lean tier otherwise.
    ///
    /// # Errors
    ///
    /// The file system's error.
    pub fn create(root: &Path, id: &str, pictures: PictureMode) -> std::io::Result<Self> {
        let directory = root.join(id);
        std::fs::create_dir_all(&directory)?;
        for index in ["index.txt", "lean.txt"] {
            match std::fs::remove_file(directory.join(index)) {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
                Ok(()) | Err(_) => {}
            }
        }
        let (tier, records) = match pictures {
            PictureMode::Keep => (Tier::Full, "records.bin"),
            PictureMode::Drop => (Tier::Lean(lean::Memories::default()), "lean.bin"),
        };
        let records = BufWriter::new(std::fs::File::create(directory.join(records))?);
        Ok(Self {
            directory,
            records,
            tier,
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
        match &mut self.tier {
            Tier::Lean(memories) => {
                let whole = usize::try_from(self.frames)
                    .is_ok_and(|position| position.is_multiple_of(lean::KEY_INTERVAL));
                lean::encode(record, memories, whole, &mut self.scratch);
            }
            Tier::Full if record.hits.is_empty() => {
                codec::encode(record, PictureMode::Keep, &mut self.scratch);
            }
            Tier::Full => {
                let mut without = record.clone();
                without.hits.clear();
                codec::encode(&without, PictureMode::Keep, &mut self.scratch);
            }
        }
        self.records.write_all(&self.scratch)?;
        let length = u64::try_from(self.scratch.len()).unwrap_or(u64::MAX);
        let _ = writeln!(
            self.index,
            "{} {} {length} {}",
            self.frames,
            self.offset,
            crate::relock::cut_letter(record)
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
        let (head, pictures, index) = match self.tier {
            Tier::Full => (HEAD, "yes", "index.txt"),
            Tier::Lean(_) => (LEAN_HEAD, "no", "lean.txt"),
        };
        let text = format!(
            "{head}\nrow {id}\nseed_record {}\nseed_words {}\npads {}\nframes {}\npictures {pictures}\n# frame offset length cut\n{}",
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
        std::fs::write(self.directory.join(index), text)?;
        Ok(self.offset)
    }
}

/// Row `id`'s pins: the native seed `start` names under `native_seeds`, and its pad file.
pub fn pins(start: Option<&str>, native_seeds: Option<&Path>, pads: &Path) -> Pins {
    let seed = start
        .and_then(|start| start.split_once(':'))
        .zip(native_seeds)
        .and_then(|((capture, index), native)| {
            let index: u32 = index.parse().ok()?;
            Some(native.join(capture).join(format!("{index:04}.lss")))
        });
    Pins {
        seed_record: seed.as_deref().and_then(crate::relock::file_sha256),
        seed_words: seed
            .as_deref()
            .and_then(|state| crate::relock::file_sha256(&state.with_extension("words"))),
        pads: crate::relock::file_sha256(pads).unwrap_or_default(),
    }
}
