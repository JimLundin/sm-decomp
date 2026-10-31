//! The cartridge battery's SRAM image, kept on the host between runs.
#![forbid(unsafe_code)]
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

/// Where the game's SRAM image is kept.
pub trait SaveStorage {
    /// Up to `limit` bytes of the stored image, or `None` when there is none.
    fn load(&mut self, limit: usize) -> Option<Vec<u8>>;
    /// Replace the stored image with `sram`; false when it cannot be written.
    fn store(&mut self, sram: &[u8]) -> bool;
}

/// A save file on disk; storing keeps the previous file as `<path>.bak`.
pub struct SaveFile {
    path: PathBuf,
}

impl SaveFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl SaveStorage for SaveFile {
    fn load(&mut self, limit: usize) -> Option<Vec<u8>> {
        let file = fs::File::open(&self.path).ok()?;
        let mut bytes = Vec::with_capacity(limit);
        // A read error keeps the bytes before it, as a short file does.
        let _ = file
            .take(u64::try_from(limit).ok()?)
            .read_to_end(&mut bytes);
        Some(bytes)
    }

    fn store(&mut self, sram: &[u8]) -> bool {
        let mut backup = self.path.clone().into_os_string();
        backup.push(".bak");
        let _ = fs::rename(&self.path, backup);
        let file = fs::File::create(&self.path);
        file.map(|mut file| drop(file.write_all(sram))).is_ok()
    }
}
