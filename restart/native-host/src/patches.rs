//! Exact source replacements shared by native preparation and focused checks.
use super::PatchRecord;
use std::{fs, path::Path};
pub(super) fn replace_once(
    root: &Path,
    path: &str,
    before: &str,
    after: &str,
    patches: &mut Vec<PatchRecord>,
) -> Result<(), String> {
    let file = root.join(path);
    // Recovered archive files and checkout-owned templates can use different
    // line endings on Windows. Match one canonical representation while keeping
    // exact-one-anchor rejection and recording the actual published replacement.
    let source = fs::read_to_string(&file).map_err(|e| e.to_string())?.replace("\r\n", "\n");
    let before = before.replace("\r\n", "\n");
    let after = after.replace("\r\n", "\n");
    if source.matches(&before).count() != 1 {
        return Err(format!("expected exactly one instrumentation anchor in {path}: {before:?}"));
    }
    fs::write(file, source.replacen(&before, &after, 1)).map_err(|e| e.to_string())?;
    patches.push(PatchRecord { path: path.into(), before, after });
    Ok(())
}
