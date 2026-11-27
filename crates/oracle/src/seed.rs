//! Migration seeds: the lsnes (bsnes v085) room captures the corpus once
//! started from (`target/seeds/<capture>/<NNNN>-<ROOM>-*`). Harness-native
//! seeds replaced them; `oracle movie --seeds` still compares a movie's work
//! RAM with theirs.

use std::path::Path;

/// The work RAM of capture `index` (1-based) in `directory`, whatever its room.
pub fn capture_wram(directory: &Path, index: u32) -> Result<Vec<u8>, String> {
    let prefix = format!("{index:04}-");
    let entries = std::fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    let path = entries
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix) && name.ends_with("-wram.bin"))
        })
        .ok_or_else(|| format!("no capture {index} in {}", directory.display()))?;
    std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))
}
