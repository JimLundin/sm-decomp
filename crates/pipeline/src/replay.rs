//! Generate original-emulator input scripts from validated SNES masks.
use crate::{atomic_toml, builds, digest};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct LsnesRequest {
    pub checkout: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub movie: PathBuf,
    pub rom: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub frames: Option<u64>,
    pub workers: Option<usize>,
    pub timeout: Option<u64>,
}
#[derive(Serialize, Deserialize)]
pub struct MesenRequest {
    pub inputs: PathBuf,
    pub output: Option<PathBuf>,
    pub frames: Option<u64>,
}
#[derive(Serialize)]
pub struct CaptureReceipt {
    pub status: String,
    pub frames: u64,
    pub output: PathBuf,
    pub metadata: PathBuf,
}
#[derive(Serialize)]
pub struct ScriptReceipt {
    pub frames: usize,
    pub output: PathBuf,
}
use std::{fs, path::Path};
use std::{fs::File, io::Read, path::PathBuf};
pub(crate) fn lsnes_observer(output: &Path) -> Result<String> {
    let output = output.to_str().context("output path is not UTF-8")?;
    ensure!(!output.contains("]]"), "unsupported output path for Lua long string");
    Ok(format!(
        "local out = assert(io.open([[{output}]], 'w'))\nout:write('frame,event,{}\\n')\nlocal fields = {{2456,1947,2806,2810,2498,2468,2522,2524,2526,2528}}\n{}",
        crate::checkpoints::FIELDS.join(","),
        include_str!("lsnes_observer.lua")
    ))
}
pub fn lsnes(request: LsnesRequest) -> Result<CaptureReceipt> {
    let checkout = request.checkout.unwrap_or(std::env::current_dir()?);
    let absolute = |path: &Path| -> Result<PathBuf> { Ok(fs::canonicalize(checkout.join(path))?) };
    let movie = absolute(&request.movie)?;
    let rom = absolute(request.rom.as_deref().unwrap_or(Path::new("rom/sm.sfc")))?;
    let output = checkout.join(request.output.as_deref().unwrap_or(Path::new("out/sync/lsnes.csv")));
    let frames = request.frames.unwrap_or(12000);
    ensure!(frames > 0, "frames must be positive");
    let binary = absolute(Path::new("deps/lsnes/source/src/util/lsnes-dumpavi.util"))
        .context("recording emulator missing: run just setup-lsnes")?;
    let rom_sha = digest(&rom)?;
    let mut archive = zip::ZipArchive::new(File::open(&movie)?)?;
    for (name, expected) in [("rom.sha256", rom_sha.as_str()), ("coreversion", "bsnes v085 (Compatibility core)")] {
        let mut contents = String::new();
        archive.by_name(name)?.read_to_string(&mut contents)?;
        ensure!(contents.trim() == expected, "movie {name} identity differs");
    }
    fs::create_dir_all(output.parent().context("output parent missing")?)?;
    let output =
        fs::canonicalize(output.parent().unwrap())?.join(output.file_name().context("output filename missing")?);
    let metadata = output.with_extension("capture.toml");
    if metadata.exists() {
        fs::remove_file(&metadata)?;
    }
    let observer = output.with_extension("lua");
    fs::write(&observer, lsnes_observer(&output)?)?;
    let mut command = vec![
        binary.to_string_lossy().into_owned(),
        "--dumper=INTERNAL-NULL".into(),
        format!("--rom-a={}", rom.display()),
        format!("--length={frames}"),
        format!("--lua={}", observer.display()),
        movie.to_string_lossy().into_owned(),
    ];
    command.insert(0, format!("XDG_CONFIG_HOME={}", output.parent().unwrap().join("lsnes-config").display()));
    command.insert(0, "env".into());
    let libraries = fs::canonicalize(checkout.join("deps/lsnes-build-deps/usr/lib/x86_64-linux-gnu"))?;
    command.insert(1, format!("LD_LIBRARY_PATH={}", libraries.display()));
    let shared = request.root.unwrap_or(checkout.join("out/pipeline"));
    let log_root = output.with_extension("capture");
    builds::execute_timeout(
        &command,
        &log_root,
        "lsnes",
        &shared,
        request.workers.unwrap_or(4),
        request.timeout.unwrap_or(600),
    )?;
    crate::checkpoints::read(&output)?;
    let report = crate::checkpoints::CaptureMetadata {
        completed_frames: frames,
        checkpoint_sha256: digest(&output)?,
        movie_sha256: Some(digest(&movie)?),
        rom_sha256: Some(rom_sha),
        core: Some("lsnes rr2-beta23 / bsnes v085 Compatibility".into()),
    };
    atomic_toml(&metadata, &report)?;
    Ok(CaptureReceipt { status: "captured".into(), frames, output, metadata })
}

pub fn mesen(request: MesenRequest) -> Result<ScriptReceipt> {
    let input = request.inputs.as_path();
    let output = request.output.as_deref().unwrap_or(Path::new("out/mesen-replay.lua"));
    let frames = request.frames.unwrap_or(3600);
    ensure!(frames > 0, "frames must be positive");
    let pads = fs::read_to_string(input)?
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| Ok(u16::from_str_radix(line.trim(), 16)?))
        .collect::<Result<Vec<_>>>()?;
    ensure!(pads.iter().all(|pad| pad & 15 == 0), "inputs must contain SNES masks in bits 4 through 15");
    let pads = pads.into_iter().take(frames as usize).map(|pad| pad.to_string()).collect::<Vec<_>>();
    ensure!(!pads.is_empty(), "no frames selected");
    let script = format!("local pads = {{{}}}\n{}", pads.join(","), include_str!("mesen_replay.lua"));
    fs::create_dir_all(output.parent().context("output parent missing")?)?;
    fs::write(output, script)?;
    Ok(ScriptReceipt { frames: pads.len(), output: output.to_path_buf() })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_observer_preserves_actual_fetch_and_literal_paths() {
        let script = lsnes_observer(Path::new("/tmp/path with spaces/capture.csv")).unwrap();
        assert!(script.contains("io.open([[/tmp/path with spaces/capture.csv]], 'w')"));
        assert!(script.contains("memory.registerexec('BUS', 0x828948, checkpoint)"));
        assert!(script.contains(
            "frame,event,game_state,room,x,y,health,items,time_frames,time_seconds,time_minutes,time_hours\\n"
        ));
        assert!(script.contains("table.concat(parts, ','), '\\n'"));
        assert!(lsnes_observer(Path::new("/tmp/]]bad")).is_err());
    }
}
