//! Complete WRAM input for dedicated native control processes.
use std::io::Read;
use std::path::Path;

pub(crate) fn read(path: &Path, control: &str) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open {control} fixture {}: {e}", path.display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() != 0x20000 {
        return Err("complete WRAM fixture required".into());
    }
    let mut bytes = Vec::with_capacity(0x20000);
    file.take(0x20001).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() != 0x20000 {
        return Err("complete WRAM fixture required".into());
    }
    Ok(bytes)
}
