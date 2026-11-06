//! The lockstep cache's writing side (docs/architecture/SLICE0.md, lane A2).
//!
//! `corpus --cache DIR` plays each row from its frame pads and writes every frame's record
//! to `DIR/<row>/`: `records.bin`, the records one after another in their own encoding
//! (`oracle_record::codec`), and `index.txt`, the row's lock pins and one line per frame
//! (`frame offset length cut`). The index is written last, so a row that failed part way
//! has no index and a reader refuses it. ROM-derived: the cache stays in `$TEMP`.
//!
//! `verify::cache` reads it.

use crate::record::FrameRecord;
use crate::record::codec::{self, PictureMode};
use std::fmt::Write as _;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};

/// The index's first line: the layout's name and version.
pub const HEAD: &str = "# sm-decomp record cache 1";

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

/// One row's cache, being written.
pub struct Writer {
    directory: PathBuf,
    records: BufWriter<std::fs::File>,
    pictures: PictureMode,
    scratch: Vec<u8>,
    offset: u64,
    index: String,
    frames: u64,
}

impl Writer {
    /// Start row `id`'s cache under `root`, removing any earlier index first.
    ///
    /// # Errors
    ///
    /// The file system's error.
    pub fn create(root: &Path, id: &str, pictures: PictureMode) -> std::io::Result<Self> {
        let directory = root.join(id);
        std::fs::create_dir_all(&directory)?;
        match std::fs::remove_file(directory.join("index.txt")) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            Ok(()) | Err(_) => {}
        }
        let records = BufWriter::new(std::fs::File::create(directory.join("records.bin"))?);
        Ok(Self {
            directory,
            records,
            pictures,
            scratch: Vec::with_capacity(0x6_0000),
            offset: 0,
            index: String::new(),
            frames: 0,
        })
    }

    /// Append the row's next frame.
    ///
    /// # Errors
    ///
    /// The file system's error.
    pub fn push(&mut self, record: &FrameRecord) -> std::io::Result<()> {
        self.scratch.clear();
        codec::encode(record, self.pictures, &mut self.scratch);
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
        let pictures = match self.pictures {
            PictureMode::Keep => "yes",
            PictureMode::Drop => "no",
        };
        let text = format!(
            "{HEAD}\nrow {id}\nseed_record {}\nseed_words {}\npads {}\nframes {}\npictures {pictures}\n# frame offset length cut\n{}",
            pins.seed_record.clone().unwrap_or_else(none),
            pins.seed_words.clone().unwrap_or_else(none),
            pins.pads,
            self.frames,
            self.index,
        );
        std::fs::write(self.directory.join("index.txt"), text)?;
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
