//! Decompressor/RNG integration with two reset epochs in each independent subject.
use crate::atomic_toml;
use crate::decomp_acceptance;
use crate::digest;
use crate::tool_helpers::CargoBuildRequest;
use anyhow::{Result, ensure};
use decomp_source::Source;
use serde::{Deserialize, Serialize};
use tool_execution::build_cache::CacheResult;
use tool_execution::builds;
use tool_execution::builds::CBuildRequest;
use tool_execution::builds::ExecutionRecord;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    pub checkout: PathBuf,
    pub output: PathBuf,
    pub root: PathBuf,
    pub accepted: PathBuf,
    pub workers: usize,
    pub live_jobs: usize,
    pub frames: u64,
    pub timeout: u64,
    pub rom: Option<PathBuf>,
    pub inputs: Option<PathBuf>,
    pub next_random_library: Option<PathBuf>,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            checkout: PathBuf::new(),
            output: PathBuf::new(),
            root: PathBuf::new(),
            accepted: PathBuf::new(),
            workers: 4,
            live_jobs: 4,
            frames: 9000,
            timeout: 240,
            rom: None,
            inputs: None,
            next_random_library: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Epoch {
    pub epoch: usize,
    pub frames: u64,
    pub rust_hits: u64,
    pub c_hits: u64,
    pub modes: u64,
    pub fallback_calls: u64,
    pub fallback_instructions: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RngEpoch {
    pub epoch: usize,
    pub rng_c_hits: u64,
    pub rng_rust_hits: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbiControls {
    pub abi_controls: u64,
    pub repeat_calls: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transformation {
    pub original_sha256: String,
    pub transformed_sha256: String,
    pub definition_renames: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorControl {
    pub exit: u64,
    pub rust_hits: u64,
    pub c_hits: u64,
    pub calls: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Verdict {
    pub status: String,
    pub run: PathBuf,
    pub stages: Vec<String>,
    pub mixed_integration: bool,
    pub original_rom_parity: String,
    pub offline_acceptance_sha256: Option<String>,
    pub rom_sha256: Option<String>,
    pub inputs_sha256: Option<String>,
    pub input_adaptation: Option<String>,
    pub contract: Option<String>,
    pub source: Option<Source>,
    pub rust_cache: Option<CacheResult>,
    pub host_cache: Option<ExecutionRecord>,
    pub rng_bank_transformation: Option<Transformation>,
    pub rng_library_sha256: Option<String>,
    pub sources: BTreeMap<String, String>,
    pub rust_library_sha256: Option<String>,
    pub binary_sha256: Option<String>,
    pub abi_controls: Option<AbiControls>,
    pub subjects: BTreeMap<String, Vec<Epoch>>,
    pub rng_subjects: BTreeMap<String, Vec<RngEpoch>>,
    pub checkpoint_identities: BTreeMap<String, String>,
    pub observed_modes: Vec<String>,
    pub unexercised_modes: Vec<String>,
    pub error_control: Option<ErrorControl>,
    pub remaining_gameplay_cpu_fallback: u64,
    pub error: Option<String>,
}

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
fn inventory(root: &Path, directory: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "live artifact is a symlink");
        if path.is_dir() {
            inventory(root, &path, result)?;
        } else if !path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml")) {
            result.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
fn records(path: &Path) -> Result<(Vec<Epoch>, Vec<RngEpoch>, Option<AbiControls>)> {
    let mut epochs = vec![];
    let mut rng = vec![];
    let mut abi = None;
    for line in fs::read_to_string(path)?.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        match fields.first().copied() {
            Some("epoch") => {
                ensure!(fields.len() == 8, "invalid epoch columns");
                let n = fields[1..].iter().map(|s| Ok(s.parse::<u64>()?)).collect::<Result<Vec<_>>>()?;
                epochs.push(Epoch {
                    epoch: usize::try_from(n[0])?,
                    frames: n[1],
                    rust_hits: n[2],
                    c_hits: n[3],
                    modes: n[4],
                    fallback_calls: n[5],
                    fallback_instructions: n[6],
                });
            }
            Some("rng-epoch") => {
                ensure!(fields.len() == 4, "invalid RNG epoch columns");
                rng.push(RngEpoch {
                    epoch: fields[1].parse()?,
                    rng_c_hits: fields[2].parse()?,
                    rng_rust_hits: fields[3].parse()?,
                });
            }
            Some("abi-controls") => {
                ensure!(fields.len() == 3 && abi.is_none(), "invalid ABI control columns");
                abi = Some(AbiControls { abi_controls: fields[1].parse()?, repeat_calls: fields[2].parse()? });
            }
            _ => {}
        }
    }
    Ok((epochs, rng, abi))
}
pub fn verify(request: Request) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let output = request.output.as_path();
    fs::create_dir_all(output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut verdict = Verdict {
        status: "error".into(),
        run: run.clone(),
        original_rom_parity: "not-tested".into(),
        ..Default::default()
    };
    let result = (|| -> Result<()> {
        let accepted = request.accepted.as_path();
        decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.to_path_buf(),
        })?;
        let predecessor = digest(accepted)?;
        verdict.offline_acceptance_sha256 = Some(predecessor.clone());
        verdict.stages.push("offline-verified".into());
        let frames = request.frames;
        ensure!(frames == 9000, "live acceptance requires frozen 9000-frame prefix");
        let root = request.root.as_path();
        let workers = request.workers;
        let timeout = request.timeout;
        let rom = request.rom.clone().unwrap_or_else(|| checkout.join("rom/sm.sfc"));
        let inputs = request.inputs.clone().unwrap_or_else(|| checkout.join("movies/super-metroid-any-3653.inputs"));
        ensure!(
            digest(&rom)? == "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72",
            "original ROM identity differs"
        );
        ensure!(
            digest(&inputs)? == digest(&checkout.join("movies/super-metroid-any-3653.inputs"))?,
            "movie input identity differs"
        );
        verdict.rom_sha256 = Some(digest(&rom)?);
        verdict.inputs_sha256 = Some(digest(&inputs)?);
        verdict.input_adaptation = Some("bit-reversal-only; no lag compensation".into());
        verdict.contract = Some("decomp-live-v2".into());
        let source = checkout.join("deps/snesrev-sm");
        verdict.source = Some(decomp_source::verify_source(&checkout, &source)?);
        let library = run.join("libdecomp.a");
        verdict.rust_cache = Some(crate::tool_helpers::cargo_build(CargoBuildRequest {
            checkout: checkout.clone(),
            root: root.to_path_buf(),
            workers,
            run: run.clone(),
            package: "decomp".into(),
            member: "libdecomp.a".into(),
            destination: library.clone(),
        })?);
        decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.to_path_buf(),
        })?;
        let rng = request.next_random_library.clone();
        let mut units = decomp_source::c_units(&source)?;
        let mut identities = BTreeMap::new();
        for unit in &units {
            identities.insert(unit.to_string_lossy().into_owned(), digest(unit)?);
        }
        for name in ["live.c", "headless.c", "headless_frame.h", "next_random_live.c"] {
            let path = checkout.join("tools/decomp").join(name);
            fs::copy(&path, run.join(name))?;
            identities.insert(path.to_string_lossy().into_owned(), digest(&path)?);
        }
        if let Some(rng) = &rng {
            let bank = source.join("src/sm_80.c");
            let signature = "uint16 NextRandom(void) {";
            let text = fs::read_to_string(&bank)?;
            ensure!(text.matches(signature).count() == 1, "NextRandom definition identity/count differs");
            let transformed = run.join("sm_80_rng.c");
            fs::write(&transformed, text.replace(signature, "uint16 C_NextRandom(void) {"))?;
            let index = units.iter().position(|p| *p == bank).unwrap();
            units[index] = transformed.clone();
            verdict.rng_bank_transformation = Some(Transformation {
                original_sha256: digest(&bank)?,
                transformed_sha256: digest(&transformed)?,
                definition_renames: 1,
            });
            verdict.rng_library_sha256 = Some(digest(rng)?);
        }
        verdict.sources = identities.clone();
        verdict.rust_library_sha256 = Some(digest(&library)?);
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
        if rng.is_some() {
            command.push("-DNEXT_RANDOM_JOB".into());
        }
        command.extend(units.iter().map(|p| p.to_string_lossy().into_owned()));
        command.extend([
            run.join("live.c").to_string_lossy().into_owned(),
            library.to_string_lossy().into_owned(),
            "-Wl,--gc-sections".into(),
            "-Wl,--wrap=Call".into(),
            "-Wl,--wrap=cpu_runOpcode".into(),
            "-Wl,--wrap=DecompressToMem".into(),
            "-Wl,--wrap=ppu_write".into(),
            "-Wl,--wrap=RtlApuWrite".into(),
        ]);
        if let Some(rng) = rng {
            command.push(rng.to_string_lossy().into_owned());
        }
        command.extend([
            "-lm".into(),
            "-ldl".into(),
            "-lpthread".into(),
            "-lz".into(),
            "-o".into(),
            run.join("live").to_string_lossy().into_owned(),
        ]);
        verdict.host_cache = Some(builds::c_build(CBuildRequest {
            checkout: checkout.clone(),
            root: root.to_path_buf(),
            workers,
            run: run.clone(),
            command,
        })?);
        verdict.binary_sha256 = Some(digest(&run.join("live"))?);
        verdict.stages.push("built".into());
        builds::execute_timeout(
            &[run.join("live").to_string_lossy().into_owned(), "--abi-control".into()],
            &run,
            "abi-control",
            root,
            workers,
            timeout,
        )?;
        verdict.abi_controls = records(&run.join("abi-control.stdout"))?.2;
        ensure!(
            verdict.abi_controls.as_ref().is_some_and(|a| a.abi_controls == 7 && a.repeat_calls == 2),
            "missing ABI controls"
        );
        let subjects = ["c-0", "c-1", "mixed-0", "mixed-1"];
        let next = AtomicUsize::new(0);
        let outcomes = Mutex::new(Vec::new());
        let jobs = request.live_jobs;
        ensure!((1..=4).contains(&jobs), "live jobs must be 1..4");
        std::thread::scope(|scope| {
            for _ in 0..jobs {
                scope.spawn(|| {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(name) = subjects.get(index) else {
                            break;
                        };
                        let directory = run.join(name);
                        let outcome = (|| -> Result<()> {
                            fs::create_dir(&directory)?;
                            builds::execute_timeout(
                                &[
                                    run.join("live").to_string_lossy().into_owned(),
                                    name.split('-').next().unwrap().into(),
                                    rom.to_string_lossy().into_owned(),
                                    inputs.to_string_lossy().into_owned(),
                                    frames.to_string(),
                                    "unused".into(),
                                ],
                                &directory,
                                "replay",
                                root,
                                workers,
                                timeout,
                            )?;
                            Ok(())
                        })();
                        outcomes.lock().unwrap().push(outcome);
                    }
                });
            }
        });
        for result in outcomes.into_inner().unwrap() {
            result?;
        }
        let mut inventories = BTreeMap::new();
        let mut subject_rows = BTreeMap::new();
        let mut rng_rows = BTreeMap::new();
        for name in subjects {
            let directory = run.join(name);
            let (rows, rng_epochs, _) = records(&directory.join("replay.stdout"))?;
            ensure!(rows.len() == 2, "incomplete same-process replay");
            for (epoch, row) in rows.iter().enumerate() {
                let mixed = name.starts_with("mixed");
                ensure!(row.epoch == epoch && row.frames == frames, "incomplete reset epoch");
                ensure!(
                    if mixed {
                        row.rust_hits == 23 && row.c_hits == 0 && row.modes > 0
                    } else {
                        row.c_hits == 23 && row.rust_hits == 0
                    },
                    "replacement was not exclusively exercised"
                );
            }
            if request.next_random_library.is_some() {
                ensure!(rng_epochs.len() == 2, "missing RNG epochs");
                for (epoch, row) in rng_epochs.iter().enumerate() {
                    let mixed = name.starts_with("mixed");
                    ensure!(
                        row.epoch == epoch
                            && if mixed {
                                row.rng_rust_hits > 0 && row.rng_c_hits == 0
                            } else {
                                row.rng_c_hits > 0 && row.rng_rust_hits == 0
                            },
                        "NextRandom replacement not exclusively exercised"
                    );
                }
                rng_rows.insert(name.to_owned(), rng_epochs);
            }
            let mut epochs = Vec::new();
            for epoch in 0..2 {
                let directory = directory.join(format!("epoch-{epoch}"));
                let mut artifacts = BTreeMap::new();
                inventory(&directory, &directory, &mut artifacts)?;
                ensure!(
                    artifacts.keys().filter(|p| p.starts_with("frame-")).count() == 10
                        && artifacts
                            .keys()
                            .filter(|p| p.starts_with("fixtures/decompress-") && p.ends_with(".wram"))
                            .count()
                            == 46,
                    "missing checkpoints or call states"
                );
                epochs.push(artifacts);
            }
            ensure!(epochs[0] == epochs[1], "same-process reset diverged: {name}");
            inventories.insert(name, epochs.remove(0));
            subject_rows.insert(name.to_owned(), rows);
        }
        ensure!(inventories.values().all(|v| *v == inventories["c-0"]), "independent replay diverged");
        let control = run.join("error-control");
        fs::create_dir(&control)?;
        let failed = builds::execute_timeout(
            &[
                run.join("live").to_string_lossy().into_owned(),
                "error".into(),
                rom.to_string_lossy().into_owned(),
                inputs.to_string_lossy().into_owned(),
                frames.to_string(),
                "unused".into(),
            ],
            &control,
            "replay",
            root,
            workers,
            timeout,
        );
        ensure!(failed.is_err(), "error control succeeded");
        let timing: ExecutionRecord = toml::from_str(&fs::read_to_string(control.join("replay.timing.toml"))?)?;
        ensure!(
            timing.exit_code == Some(4)
                && fs::read_to_string(control.join("replay.stderr"))?
                    .contains("ABI error status=2 rust_hits=0 c_hits=0 calls=1"),
            "ABI error control did not stop at exercised replacement"
        );
        let mut artifacts = BTreeMap::new();
        inventory(&control, &control, &mut artifacts)?;
        ensure!(
            !artifacts.keys().any(|p| p.contains("frame-") || p.ends_with("after.wram")),
            "ABI error control emitted successful state"
        );
        decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.to_path_buf(),
        })?;
        ensure!(digest(accepted)? == predecessor, "predecessor changed");
        for (path, hash) in identities {
            ensure!(digest(Path::new(&path))? == hash, "source changed during live verification");
        }
        let mask = subject_rows["mixed-0"][0].modes;
        let modes = [
            "literal",
            "byte-fill",
            "word-fill",
            "increment-fill",
            "absolute-copy",
            "absolute-xor",
            "relative-copy",
            "relative-xor",
        ];
        verdict.status = "mixed-C-Rust-integration".into();
        verdict.mixed_integration = true;
        verdict.subjects = subject_rows;
        verdict.rng_subjects = rng_rows;
        verdict.checkpoint_identities = inventories["c-0"].clone();
        verdict.observed_modes =
            modes.iter().enumerate().filter_map(|(i, n)| (mask & (1 << i) != 0).then_some((*n).to_owned())).collect();
        verdict.unexercised_modes =
            modes.iter().enumerate().filter_map(|(i, n)| (mask & (1 << i) == 0).then_some((*n).to_owned())).collect();
        verdict.error_control = Some(ErrorControl { exit: 4, rust_hits: 0, c_hits: 0, calls: 1 });
        verdict.remaining_gameplay_cpu_fallback = verdict.subjects["mixed-0"][0].fallback_calls;
        verdict.stages.push("compared".into());
        Ok(())
    })();
    if let Err(error) = result {
        verdict.error = Some(format!("{error:#}"));
    }
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_epoch_rows_preserve_reset_and_exclusive_dispatch_counts() {
        let path = std::env::temp_dir().join(format!("sm-native-epochs-{}", std::process::id()));
        fs::write(&path,"abi-controls 7 2\nepoch 0 9000 23 0 255 17 8192\nrng-epoch 0 0 113\nepoch 1 9000 23 0 255 17 8192\nrng-epoch 1 0 113\n").unwrap();
        let (epochs, rng, abi) = records(&path).unwrap();
        assert_eq!(epochs.len(), 2);
        assert_eq!(epochs[1].epoch, 1);
        assert_eq!(epochs[0].fallback_instructions, 8192);
        assert_eq!(rng[1].rng_rust_hits, 113);
        assert_eq!(abi.unwrap().abi_controls, 7);
        fs::write(&path, "epoch 0 9000 23 0 255 17\n").unwrap();
        assert!(records(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
