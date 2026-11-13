//! The content cache (docs/architecture/assets.md §6): a record set an importer built, kept
//! in the user's cache directory so the next start doesn't import it again.
//!
//! The import is the authority and the cache only saves time. A file is used when it was
//! written from the same ROM, by the same revision of the importer, in the same shape of the
//! format, and its records still have the fingerprint it was written with; anything else is
//! a [`Miss`], and the set is built and the file replaced. No failure of the cache fails a
//! load: a file that can't be read or written costs the time of an import, and [`Source`]
//! says so.
//!
//! The file holds content from the player's ROM, so its directory is never one inside a
//! checkout.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use canonical::{Canonical, from_bytes, to_bytes};
use data::assets::fingerprint::ContentFingerprint;
use data::rom::sha256;

#[cfg(test)]
mod tests;

/// An importer's revision: it changes whenever the importer would write other records from
/// the same ROM, which the format's schema can't see.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Revision(pub u32);

/// A record set the cache keeps.
pub trait Cached: Canonical {
    /// The set's name, which names its file: lower-case letters and dashes.
    const NAME: &'static str;
    /// The revision of the importer that builds it. It rises with any change to the importer
    /// or to a registry it reads that changes the records it writes.
    const REVISION: Revision;
}

/// Why a set was built rather than read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Miss {
    /// No file is kept for this ROM.
    Absent,
    /// The file couldn't be read.
    Unreadable(io::ErrorKind),
    /// The file was written from another ROM, by another revision of the importer or in
    /// another shape of the format.
    OtherKey,
    /// The file's records don't have the fingerprint it was written with, or don't decode.
    Damaged,
}

/// Whether a built set was kept for the next load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kept {
    /// Its file was written.
    Written,
    /// Its file couldn't be written; the next load builds it again.
    NotWritten(io::ErrorKind),
}

/// Where a loaded set came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The cache's file.
    Cache,
    /// The importer, the file having missed.
    Built {
        /// Why the file wasn't used.
        miss: Miss,
        /// Whether the built set was kept.
        kept: Kept,
    },
}

/// A record set, loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loaded<T> {
    /// The records.
    pub content: T,
    /// Their fingerprint.
    pub fingerprint: ContentFingerprint,
    /// Where they came from.
    pub source: Source,
}

/// What a file must have been written with to be used: the format's shape, the importer's
/// revision and the ROM's SHA-256, as the bytes the file starts with.
struct Key([u8; Self::LEN]);

impl Key {
    const SCHEMA_LEN: usize = size_of::<u64>();
    const REVISION_LEN: usize = size_of::<u32>();
    const ROM_LEN: usize = 32;
    const LEN: usize = Self::SCHEMA_LEN + Self::REVISION_LEN + Self::ROM_LEN;

    fn of<T: Cached>(rom: [u8; Self::ROM_LEN]) -> Self {
        let mut bytes = Vec::with_capacity(Self::LEN);
        T::SCHEMA.encode(&mut bytes);
        T::REVISION.0.encode(&mut bytes);
        bytes.extend_from_slice(&rom);
        let mut key = [0; Self::LEN];
        key.copy_from_slice(&bytes);
        Self(key)
    }
}

/// The cache: a directory of record sets, one file per set and ROM.
#[derive(Clone, Debug)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    /// The cache in `dir`, which is made when a set is first kept.
    #[must_use]
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// The set `T` of the ROM `image`: from its file where that is usable, otherwise from
    /// `build`, whose result is then kept.
    ///
    /// # Errors
    ///
    /// `build`'s error, when the set had to be built and the image doesn't import.
    pub fn load<T: Cached, E>(
        &self,
        image: &[u8],
        build: impl FnOnce() -> Result<T, E>,
    ) -> Result<Loaded<T>, E> {
        let rom = sha256(image);
        let key = Key::of::<T>(rom);
        let path = self.path::<T>(rom);
        let miss = match read::<T>(&path, &key) {
            Ok(loaded) => return Ok(loaded),
            Err(miss) => miss,
        };
        let content = build()?;
        let encoding = to_bytes(&content);
        let fingerprint = ContentFingerprint::of_encoding::<T>(&encoding);
        let kept = match self.write(&path, &key, fingerprint, &encoding) {
            Ok(()) => Kept::Written,
            Err(error) => Kept::NotWritten(error.kind()),
        };
        Ok(Loaded {
            content,
            fingerprint,
            source: Source::Built { miss, kept },
        })
    }

    /// The file of set `T` for the ROM with this hash. The name carries the start of the
    /// hash so two ROMs' sets stand side by side; the key inside the file holds all of it.
    fn path<T: Cached>(&self, rom: [u8; Key::ROM_LEN]) -> PathBuf {
        let hash = rom.iter().take(8).fold(String::new(), |mut hash, byte| {
            // Writing to a `String` can't fail.
            let _ = write!(hash, "{byte:02x}");
            hash
        });
        self.dir.join(format!("{}-{hash}.content", T::NAME))
    }

    /// Writes the file whole or not at all: beside its place first, then moved onto it, so a
    /// load that is interrupted leaves the old file or none.
    fn write(
        &self,
        path: &Path,
        key: &Key,
        fingerprint: ContentFingerprint,
        encoding: &[u8],
    ) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let mut file = Vec::new();
        file.extend_from_slice(&key.0);
        fingerprint.encode(&mut file);
        file.extend_from_slice(encoding);
        let beside = path.with_extension(format!("{}.part", std::process::id()));
        fs::write(&beside, file)
            .and_then(|()| fs::rename(&beside, path))
            .inspect_err(|_| {
                // Nothing reads a part file; one left behind only takes space.
                let _ = fs::remove_file(&beside);
            })
    }
}

/// The set in the file at `path`, if it was written with `key` and is whole.
fn read<T: Cached>(path: &Path, key: &Key) -> Result<Loaded<T>, Miss> {
    let file = fs::read(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            Miss::Absent
        } else {
            Miss::Unreadable(error.kind())
        }
    })?;
    let (written_with, rest) = file
        .split_first_chunk::<{ Key::LEN }>()
        .ok_or(Miss::Damaged)?;
    if *written_with != key.0 {
        return Err(Miss::OtherKey);
    }
    let (fingerprint, encoding) = rest
        .split_first_chunk::<{ size_of::<u64>() }>()
        .ok_or(Miss::Damaged)?;
    let fingerprint = ContentFingerprint(u64::from_le_bytes(*fingerprint));
    if ContentFingerprint::of_encoding::<T>(encoding) != fingerprint {
        return Err(Miss::Damaged);
    }
    let content = from_bytes(encoding).map_err(|_| Miss::Damaged)?;
    Ok(Loaded {
        content,
        fingerprint,
        source: Source::Cache,
    })
}
