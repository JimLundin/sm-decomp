//! Publish standalone generators through the canonical binary Fixture writer.
use std::{
    fs,
    io::Write,
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

pub fn write(pipeline: &Path, output: &Path, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    if output.extension().is_none_or(|extension| extension != "smf") {
        return Err("Fixture output must use the .smf extension".into());
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary =
        output.with_extension(format!("edit-{}-{}.toml", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        file.write_all(text.as_bytes())?;
        drop(file);
        let imported = Command::new(pipeline).args(["fixture", "import"]).arg(&temporary).arg(output).output()?;
        if !imported.status.success() {
            return Err(format!("binary Fixture import failed: {}", String::from_utf8_lossy(&imported.stderr)).into());
        }
        Ok(())
    })();
    fs::remove_file(temporary)?;
    result
}
