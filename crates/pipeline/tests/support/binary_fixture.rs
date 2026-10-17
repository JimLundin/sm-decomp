//! Exercise the same explicit authoring/inspection commands used by maintainers.
use std::{
    fs, io,
    path::Path,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);

pub fn write(path: &Path, text: impl AsRef<[u8]>) -> io::Result<()> {
    let temporary =
        path.with_extension(format!("edit-{}-{}.toml", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::write(&temporary, text)?;
    let result =
        Command::new(env!("CARGO_BIN_EXE_sm-pipeline")).args(["fixture", "import"]).arg(&temporary).arg(path).output();
    fs::remove_file(temporary)?;
    let result = result?;
    if !result.status.success() {
        return Err(io::Error::other(String::from_utf8_lossy(&result.stderr).into_owned()));
    }
    Ok(())
}

#[allow(dead_code)]
pub fn read(path: &Path) -> io::Result<String> {
    let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline")).args(["fixture", "inspect"]).arg(path).output()?;
    if !result.status.success() {
        return Err(io::Error::other(String::from_utf8_lossy(&result.stderr).into_owned()));
    }
    String::from_utf8(result.stdout).map_err(io::Error::other)
}
