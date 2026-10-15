//! Pinned actual-C baseline and source-authenticated full-state captures.
use crate::atomic_toml;
use crate::digest;
use anyhow::{Context, Result, ensure};
use decomp_source::Source;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tool_execution::builds;
const REVISION: &str = "578f90b3cc49557bb70060ad033bb90b8cf8ac50";
const ROM: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StateFile {
    pub file: String,
    pub sha256: String,
    pub size: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureCall {
    pub sequence: u64,
    pub frame: u64,
    pub source_address: u32,
    pub destination_offset: usize,
    pub before: StateFile,
    pub after: StateFile,
    pub fixture_id: String,
    pub rom_view_file: String,
    pub rom_view_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureManifest {
    pub contract: String,
    pub routine: String,
    pub routine_address: u64,
    pub source_revision: String,
    pub original_rom_sha256: String,
    pub rom_view_file: String,
    pub rom_view_sha256: String,
    pub rom_view_size: u64,
    pub rom_expansion: String,
    pub calls: Vec<CaptureCall>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Progress {
    pub frame: u64,
    pub game_state: u64,
    pub room: u64,
    pub samus_x: u64,
    pub samus_y: u64,
    pub rng: u64,
    pub fallback_calls: u64,
    pub fallback_instructions: u64,
    pub decompression_calls: u64,
}
pub(crate) fn progress(path: &Path) -> Result<Vec<Progress>> {
    let mut rows = Vec::new();
    for line in fs::read_to_string(path)?.lines().filter(|l| l.starts_with("progress ")) {
        let v = line.split_whitespace().skip(1).map(str::parse::<u64>).collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(v.len() == 9, "invalid headless progress row");
        rows.push(Progress {
            frame: v[0],
            game_state: v[1],
            room: v[2],
            samus_x: v[3],
            samus_y: v[4],
            rng: v[5],
            fallback_calls: v[6],
            fallback_instructions: v[7],
            decompression_calls: v[8],
        });
    }
    Ok(rows)
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Controls {
    pub fallback_calls: u64,
    pub fallback_instructions: u64,
    pub decompression_calls: u64,
}
fn control(path: &Path) -> Result<Controls> {
    let text = fs::read_to_string(path)?;
    let line = text.lines().find(|l| l.starts_with("control ")).context("missing observation control report")?;
    let v = line.split_whitespace().skip(1).map(str::parse::<u64>).collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(v.len() == 3, "invalid control report");
    Ok(Controls { fallback_calls: v[0], fallback_instructions: v[1], decompression_calls: v[2] })
}
fn capture(subject: &Path) -> Result<(CaptureManifest, BTreeMap<String, String>)> {
    let directory = subject.join("fixtures");
    let mut hashes = BTreeMap::new();
    for entry in fs::read_dir(&directory)? {
        let path = entry?.path();
        if path.extension().is_some_and(|s| s == "wram") {
            hashes.insert(path.file_name().unwrap().to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    let bytes = fs::read(directory.join("calls.bin"))?;
    ensure!(bytes.starts_with(b"SMDC2") && (bytes.len() - 5) % 32 == 0, "invalid capture binary version or length");
    let mut calls = Vec::new();
    for (index, record) in bytes[5..].as_chunks::<32>().0.iter().enumerate() {
        let fields = record.as_chunks::<8>().0.iter().map(|b| u64::from_le_bytes(*b)).collect::<Vec<_>>();
        ensure!(fields[0] == index as u64 + 1 && fields[3] < 0x20000, "invalid captured routine arguments");
        let id = format!("decompress-{:04}", index + 1);
        let state = |phase: &str| -> Result<StateFile> {
            let name = format!("{id}-{phase}.wram");
            ensure!(directory.join(&name).metadata()?.len() == 0x20000, "incomplete routine state");
            Ok(StateFile {
                sha256: hashes.get(&name).context("fixture hash missing")?.clone(),
                file: name,
                size: 0x20000,
            })
        };
        let rom = directory.join(format!("{id}.rom"));
        let hash = digest(&rom)?;
        let name = format!("rom-{hash}.bin");
        let retained = directory.join(&name);
        if retained.exists() {
            ensure!(digest(&retained)? == hash, "retained ROM view corrupt");
            fs::remove_file(rom)?;
        } else {
            fs::rename(rom, &retained)?;
        }
        calls.push(CaptureCall {
            sequence: fields[0],
            frame: fields[1],
            source_address: u32::try_from(fields[2])?,
            destination_offset: usize::try_from(fields[3])?,
            before: state("before")?,
            after: state("after")?,
            fixture_id: id,
            rom_view_file: name,
            rom_view_sha256: hash,
        });
    }
    let view = subject.join("rom-view.bin");
    let manifest = CaptureManifest {
        contract: "decomp-c-captures-v2".into(),
        routine: "DecompressToMem".into(),
        routine_address: 0x80b119,
        source_revision: REVISION.into(),
        original_rom_sha256: ROM.into(),
        rom_view_file: "../rom-view.bin".into(),
        rom_view_sha256: digest(&view)?,
        rom_view_size: view.metadata()?.len(),
        rom_expansion: "3 MiB to 4 MiB: mirror final MiB before patches".into(),
        calls,
    };
    atomic_toml(&directory.join("manifest.toml"), &manifest)?;
    Ok((manifest, hashes))
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub checkout: PathBuf,
    pub output: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    #[serde(default = "default_frames")]
    pub frames: u64,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    pub rom: Option<PathBuf>,
    pub inputs: Option<PathBuf>,
}
fn default_frames() -> u64 {
    9000
}
fn default_timeout() -> u64 {
    240
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct RomPatches {
    pub changed_bytes: usize,
    pub file_offsets: Vec<usize>,
    pub rom_view_sha256: String,
    pub original_size: usize,
    pub rom_view_size: usize,
    pub expansion: String,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Verdict {
    pub contract: String,
    pub source_revision: String,
    pub requested_frames: u64,
    pub rust_routines: u64,
    pub original_rom_parity: String,
    pub input_adaptation: String,
    pub status: String,
    pub evidence: PathBuf,
    pub source: Option<Source>,
    pub source_archive_sha256: String,
    pub rom_sha256: String,
    pub inputs_sha256: String,
    pub license_sha256: String,
    pub compiler: String,
    pub build_command: Vec<String>,
    pub build_cache: Option<builds::ExecutionRecord>,
    pub binary_sha256: String,
    pub sources: BTreeMap<String, String>,
    pub observation_controls: Option<Controls>,
    pub in_memory_rom_patches: Option<RomPatches>,
    pub replay_commands: Vec<Vec<String>>,
    pub terminal: Option<Progress>,
    pub fixtures: BTreeMap<String, String>,
    pub state_sha256: String,
    pub determinism: String,
    pub error: Option<String>,
}
pub fn operate(request: Request) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    fs::create_dir_all(&request.output)?;
    let run = request.output.join(format!(
        "{}-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        std::process::id()
    ));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let frames = request.frames;
    let timeout = request.timeout;
    let mut verdict = Verdict {
        contract: "decomp-c-baseline-v2".into(),
        source_revision: REVISION.into(),
        requested_frames: frames,
        original_rom_parity: "not-tested".into(),
        input_adaptation: "bit-reversal-only; no lag compensation".into(),
        status: "error".into(),
        evidence: run.clone(),
        ..Default::default()
    };
    let outcome = (|| -> Result<()> {
        ensure!((1..=1000000).contains(&frames) && timeout > 0, "frames/timeout outside supported bounds");
        let root = &request.root;
        let workers = request.workers;
        let source = checkout.join("deps/snesrev-sm");
        let identity = decomp_source::verify_source(&checkout, &source)?;
        ensure!(identity.revision == REVISION, "frozen source revision differs");
        verdict.source_archive_sha256 = identity.archive_sha256.clone();
        verdict.source = Some(identity.clone());
        let rom = request.rom.clone().unwrap_or_else(|| checkout.join("rom/sm.sfc"));
        let inputs = request.inputs.clone().unwrap_or_else(|| checkout.join("movies/super-metroid-any-3653.inputs"));
        let inputs = fs::canonicalize(&inputs).with_context(|| format!("missing inputs {}", inputs.display()))?;
        ensure!(digest(&rom)? == ROM, "unsupported ROM identity");
        verdict.rom_sha256 = digest(&rom)?;
        verdict.inputs_sha256 = digest(&inputs)?;
        let units = decomp_source::c_units(&source)?;
        let mut sources = BTreeMap::new();
        for unit in &units {
            sources.insert(unit.to_string_lossy().into_owned(), digest(unit)?);
        }
        for name in ["headless.c", "headless_frame.h"] {
            let path = checkout.join("tools/decomp").join(name);
            let hash = digest(&path)?;
            let target = run.join(if name == "headless.c" { "host.c" } else { name });
            fs::copy(&path, &target)?;
            ensure!(digest(&target)? == hash, "baseline host changed during copy");
            sources.insert(path.to_string_lossy().into_owned(), hash);
        }
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        verdict.license_sha256 = digest(&run.join("LICENSE.txt"))?;
        fs::copy(std::env::current_exe()?, run.join("sm-pipeline"))?;
        let compiler = std::process::Command::new("cc").arg("--version").output()?;
        ensure!(compiler.status.success(), "C compiler unavailable");
        verdict.compiler =
            String::from_utf8(compiler.stdout)?.lines().next().context("C compiler version missing")?.into();
        let mut command = vec![
            "cc".into(),
            "-std=gnu11".into(),
            "-O2".into(),
            "-fno-strict-aliasing".into(),
            "-ffunction-sections".into(),
            "-fdata-sections".into(),
            "-iquote".into(),
            source.join("src").to_string_lossy().into_owned(),
        ];
        command.extend(units.iter().map(|p| p.to_string_lossy().into_owned()));
        command.extend([
            run.join("host.c").to_string_lossy().into_owned(),
            "-Wl,--gc-sections".into(),
            "-Wl,--wrap=Call".into(),
            "-Wl,--wrap=cpu_runOpcode".into(),
            "-Wl,--wrap=DecompressToMem".into(),
            "-lm".into(),
            "-o".into(),
            run.join("headless").to_string_lossy().into_owned(),
        ]);
        verdict.build_command = command.clone();
        verdict.build_cache = Some(builds::c_build(builds::CBuildRequest {
            checkout: checkout.clone(),
            root: root.clone(),
            workers,
            run: run.clone(),
            command,
        })?);
        verdict.binary_sha256 = digest(&run.join("headless"))?;
        verdict.sources = sources;
        let control_dir = run.join("observation-control");
        for directory in ["fixtures", "saves"] {
            fs::create_dir_all(control_dir.join(directory))?;
        }
        builds::execute_timeout(
            &[
                run.join("headless").to_string_lossy().into_owned(),
                "--observation-control".into(),
                rom.to_string_lossy().into_owned(),
                "unused".into(),
            ],
            &control_dir,
            "control",
            root,
            workers,
            timeout,
        )?;
        let measured = control(&control_dir.join("control.stdout"))?;
        ensure!(
            measured.fallback_calls == 1 && measured.fallback_instructions > 0 && measured.decompression_calls == 1,
            "observation control did not exercise real fallback/decompression wrappers"
        );
        verdict.observation_controls = Some(measured);
        let mut first = None;
        let mut fixture_hashes = None;
        for prefix in ["replay", "repeat"] {
            let subject = run.join(prefix);
            for directory in ["fixtures", "saves"] {
                fs::create_dir_all(subject.join(directory))?;
            }
            let command = vec![
                run.join("headless").to_string_lossy().into_owned(),
                rom.to_string_lossy().into_owned(),
                inputs.to_string_lossy().into_owned(),
                frames.to_string(),
                run.join(format!("{prefix}.state.bin")).to_string_lossy().into_owned(),
            ];
            builds::execute_timeout(&command, &subject, "replay", root, workers, timeout)?;
            verdict.replay_commands.push(command);
            let (manifest, hashes) = capture(&subject)?;
            if let Some(previous) = &first {
                ensure!(*previous == manifest, "routine manifests differ on fresh-process replay");
                ensure!(fixture_hashes.as_ref() == Some(&hashes), "routine fixtures differ on fresh-process replay");
            } else {
                let original = fs::read(&rom)?;
                ensure!(original.len() == 0x300000, "unexpected original ROM size");
                let mut expanded = original.clone();
                expanded.extend_from_slice(&original[0x200000..]);
                let patched = fs::read(subject.join("rom-view.bin"))?;
                ensure!(expanded.len() == patched.len(), "unexpected in-memory ROM size");
                let changed = expanded
                    .iter()
                    .zip(&patched)
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .map(|(i, _)| i)
                    .collect::<Vec<_>>();
                verdict.in_memory_rom_patches = Some(RomPatches {
                    changed_bytes: changed.len(),
                    file_offsets: changed,
                    rom_view_sha256: manifest.rom_view_sha256.clone(),
                    original_size: original.len(),
                    rom_view_size: patched.len(),
                    expansion: "3 MiB to 4 MiB: mirror final MiB before patches".into(),
                });
                first = Some(manifest);
                fixture_hashes = Some(hashes);
            }
        }
        for suffix in ["stdout", "stderr"] {
            ensure!(
                digest(&run.join("replay").join(format!("replay.{suffix}")))?
                    == digest(&run.join("repeat").join(format!("replay.{suffix}")))?,
                "fresh-process replay log nondeterministic: {suffix}"
            );
        }
        ensure!(
            digest(&run.join("replay.state.bin"))? == digest(&run.join("repeat.state.bin"))?,
            "fresh-process terminal state nondeterministic"
        );
        let terminal =
            progress(&run.join("replay/replay.stdout"))?.last().context("missing terminal replay record")?.clone();
        ensure!(terminal.frame == frames, "missing terminal replay frame");
        ensure!(
            fixture_hashes.as_ref().unwrap().len() as u64 == 2 * terminal.decompression_calls,
            "routine fixture coverage incomplete"
        );
        verdict.terminal = Some(terminal);
        verdict.fixtures = fixture_hashes.unwrap();
        verdict.state_sha256 = digest(&run.join("replay.state.bin"))?;
        ensure!(
            decomp_source::verify_source(&checkout, &source)? == identity
                && digest(&rom)? == ROM
                && verdict.inputs_sha256 == digest(&inputs)?,
            "baseline source/assets changed during replay"
        );
        verdict.determinism = "two fresh processes; progress, fallback/decompression log, WRAM and SRAM match".into();
        verdict.status = "replay-completed".into();
        Ok(())
    })();
    if let Err(error) = outcome {
        verdict.error = Some(format!("{error:#}"));
    }
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    Ok(verdict)
}
