//! Pinned TAS corpus downloads and imports with movie/ROM identity checks.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub movies: Vec<MoviePin>,
}
#[derive(Deserialize)]
pub struct MoviePin {
    pub file: String,
    pub sha256: String,
    pub rom_sha256: String,
    pub frames: u64,
    pub download: String,
}
#[derive(Serialize)]
pub struct CorpusReport {
    pub status: String,
    pub movies: Vec<MovieRecord>,
}
#[derive(Serialize)]
pub struct MovieRecord {
    pub file: String,
    pub sha256: String,
    pub frames: u64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<std::path::PathBuf>,
}
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Cursor, Read},
    path::{Component, Path},
    process::Command,
};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn authenticate(entry: &MoviePin, bytes: &[u8], rom: Option<&str>) -> Result<()> {
    ensure!(hash(bytes) == entry.sha256.as_str(), "movie checksum mismatch: {}", entry.file.as_str());
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut expected = String::new();
    archive.by_name("rom.sha256")?.read_to_string(&mut expected)?;
    ensure!(
        expected.trim() == entry.rom_sha256.as_str() && rom.is_none_or(|rom| rom == expected.trim()),
        "movie ROM checksum mismatch"
    );
    Ok(())
}

pub fn corpus(root: &Path, operation: &str, offline: bool) -> Result<CorpusReport> {
    let manifest: Manifest = toml::from_str(&fs::read_to_string(root.join("movies/manifest.toml"))?)?;
    ensure!(manifest.schema_version == 1, "unknown movie manifest version");
    let rom = root.join("rom/sm.sfc");
    let rom_hash = if rom.exists() { Some(hash(&fs::read(rom)?)) } else { None };
    let mut records = Vec::new();
    for entry in &manifest.movies {
        let name = entry.file.as_str();
        ensure!(
            Path::new(name).components().count() == 1
                && Path::new(name).components().all(|v| matches!(v, Component::Normal(_))),
            "unsafe corpus movie filename"
        );
        let destination = root.join("movies").join(name);
        let bytes = if destination.exists() {
            fs::read(&destination)?
        } else {
            ensure!(operation == "fetch-corpus" && !offline, "missing movie: {name}");
            let downloads = root.join("deps/downloads/tas");
            fs::create_dir_all(&downloads)?;
            let path = downloads.join(format!("{name}.zip"));
            let status = Command::new("curl")
                .args([
                    "-L",
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--retry",
                    "3",
                    "--connect-timeout",
                    "30",
                    entry.download.as_str(),
                    "-o",
                ])
                .arg(&path)
                .status()?;
            ensure!(status.success(), "movie download failed");
            let mut outer = zip::ZipArchive::new(Cursor::new(fs::read(path)?))?;
            let names =
                outer.file_names().filter(|name| name.ends_with(".lsmv")).map(str::to_owned).collect::<Vec<_>>();
            ensure!(names.len() == 1, "expected one LSMV in TASVideos download");
            let mut bytes = Vec::new();
            outer.by_name(&names[0])?.read_to_end(&mut bytes)?;
            bytes
        };
        authenticate(entry, &bytes, rom_hash.as_deref())?;
        if !destination.exists() {
            fs::write(&destination, &bytes)?;
        }
        let mut record = MovieRecord {
            file: name.into(),
            sha256: entry.sha256.clone(),
            frames: entry.frames,
            status: "verified".into(),
            output: None,
        };
        if operation == "import-corpus" {
            let movie = crate::load(&destination)?;
            ensure!(movie.frames.len() as u64 == entry.frames, "movie frame count mismatch: {name}");
            let output = destination.with_extension("inputs");
            fs::write(&output, crate::write_inputs(&movie.frames))?;
            record.output = Some(output);
        }
        records.push(record);
    }
    Ok(CorpusReport { status: "complete".into(), movies: records })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn pinned_container_checks_movie_and_rom_before_publication() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file("rom.sha256", zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(b"expected-rom\n").unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let entry = MoviePin {
            file: "test.lsmv".into(),
            sha256: hash(&bytes),
            rom_sha256: "expected-rom".into(),
            frames: 0,
            download: String::new(),
        };
        authenticate(&entry, &bytes, Some("expected-rom")).unwrap();
        assert!(authenticate(&entry, &bytes, Some("wrong-rom")).is_err());
        let mut corrupt = bytes.clone();
        corrupt[0] ^= 1;
        assert!(authenticate(&entry, &corrupt, None).is_err());
    }
}
