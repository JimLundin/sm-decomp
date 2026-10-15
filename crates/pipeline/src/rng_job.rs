//! One evidence-bound RNG translation job with actual C exhaustive and live oracles.
use crate::atomic_toml;
use crate::decomp_acceptance;
use crate::digest;
use crate::routine_live;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tool_execution::builds;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    pub checkout: PathBuf,
    pub output: PathBuf,
    pub root: PathBuf,
    pub accepted: PathBuf,
    pub workers: usize,
    pub timeout: u64,
    pub frames: u64,
    pub live_jobs: usize,
    pub count: i64,
    pub job: String,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            checkout: PathBuf::new(),
            output: PathBuf::new(),
            root: PathBuf::new(),
            accepted: PathBuf::new(),
            workers: 4,
            timeout: 240,
            frames: 9000,
            live_jobs: 4,
            count: 1,
            job: "next-random".into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Example {
    pub seed: usize,
    pub expected: u64,
    pub derivation: Option<String>,
}
#[derive(Deserialize)]
struct Job {
    job: String,
    source_revision: String,
    independent_examples: Vec<Example>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Synthetic {
    pub all_65536_seeds: bool,
    #[serde(rename = "full_WRAM_unchanged_outside_two_bytes")]
    pub full_wram_unchanged_outside_two_bytes: bool,
    pub independent_examples: Vec<Example>,
    pub table_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Predecessor {
    pub pointer: PathBuf,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capture {
    pub before: PathBuf,
    pub before_sha256: String,
    pub after: PathBuf,
    pub after_sha256: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Verdict {
    pub status: String,
    pub routine_accepted: bool,
    pub run: PathBuf,
    pub original_rom_parity: String,
    pub predecessor: Option<Predecessor>,
    pub source: Option<decomp_source::Source>,
    pub implementation: BTreeMap<String, String>,
    pub synthetic: Option<Synthetic>,
    pub captures: Vec<Capture>,
    pub observed_calls: u64,
    pub live_run: PathBuf,
    pub live_verdict_sha256: String,
    pub live_hits: BTreeMap<String, Vec<routine_live::RngEpoch>>,
    #[serde(rename = "remaining_CPU_fallback")]
    pub remaining_cpu_fallback: u64,
    pub binaries: BTreeMap<String, String>,
    pub artifacts: BTreeMap<String, String>,
    pub scope: String,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptedJob {
    pub job: String,
    pub exercised: bool,
    pub calls: u64,
    pub evidence: PathBuf,
    pub verdict_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub status: String,
    pub requested_count: i64,
    pub accepted_jobs: Vec<AcceptedJob>,
    #[serde(rename = "remaining_C_routines")]
    pub remaining_c_routines: String,
    #[serde(rename = "remaining_CPU_fallback")]
    pub remaining_cpu_fallback: Option<u64>,
    pub error: Option<String>,
}
fn execute(request: &Request, run: &Path, argv: &[String], label: &str) -> Result<Vec<u8>> {
    builds::execute_timeout(argv, run, label, &request.root, request.workers, request.timeout)?;
    Ok(fs::read(run.join(format!("{label}.stdout")))?)
}
fn table_word(table: &[u8], seed: usize) -> Result<u16> {
    let bytes = table.get(seed * 2..seed * 2 + 2).context("RNG seed outside exhaustive table")?;
    Ok(u16::from_le_bytes(bytes.try_into()?))
}
fn verify_effects(text: &str, table: &[u8], expected_calls: u64) -> Result<u64> {
    let mut calls = 0;
    for line in text.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.get(1) != Some(&"rng") {
            continue;
        }
        ensure!(parts.len() == 8, "RNG event extent differs");
        let seed = parts[3].parse::<usize>()?;
        let result = parts[4].parse::<u16>()?;
        let expected = table_word(table, seed)?;
        ensure!(
            result == expected
                && parts[6] == format!("1509:{}", expected as u8)
                && parts[7] == format!("1510:{}", expected >> 8),
            "first ordered RNG effect mismatch call={}",
            calls + 1
        );
        calls += 1;
    }
    ensure!(calls == expected_calls && calls > 0, "RNG hardware log does not cover every actual call");
    Ok(calls)
}
fn inventory(root: &Path, dir: &Path, entries: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        ensure!(!p.is_symlink(), "RNG evidence is a symlink");
        if p.is_dir() {
            inventory(root, &p, entries)?;
        } else if !p.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml")) {
            entries.insert(p.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&p)?);
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticateRequest {
    pub checkout: PathBuf,
    pub verdict: PathBuf,
}
fn sealed_path(run: &Path, relative: &str) -> Result<PathBuf> {
    use std::path::Component;
    ensure!(
        !relative.is_empty() && Path::new(relative).components().all(|c| matches!(c, Component::Normal(_))),
        "invalid RNG artifact path"
    );
    let mut path = run.to_path_buf();
    for component in Path::new(relative).components() {
        path.push(component);
        ensure!(!path.is_symlink(), "RNG artifact is a symlink");
    }
    ensure!(path.is_file(), "RNG artifact missing: {relative}");
    Ok(path)
}
fn authenticate_artifacts(run: &Path, artifacts: &BTreeMap<String, String>) -> Result<()> {
    ensure!(!artifacts.is_empty(), "empty RNG artifact inventory");
    for (relative, expected) in artifacts {
        ensure!(digest(&sealed_path(run, relative)?)? == *expected, "RNG artifact changed: {relative}");
    }
    let mut actual = BTreeMap::new();
    inventory(run, run, &mut actual)?;
    actual.remove("verdict.toml");
    ensure!(actual == *artifacts, "RNG retained artifact inventory differs");
    Ok(())
}
pub fn authenticate(request: AuthenticateRequest) -> Result<Verdict> {
    ensure!(!request.verdict.is_symlink(), "RNG verdict is a symlink");
    let checkout = fs::canonicalize(&request.checkout)?;
    let path = fs::canonicalize(&request.verdict)?;
    let run = path.parent().context("RNG verdict parent missing")?;
    let verdict: Verdict = toml::from_str(&fs::read_to_string(&path)?)?;
    ensure!(!request.verdict.parent().context("RNG verdict parent missing")?.is_symlink(), "RNG run is a symlink");
    ensure!(
        path.file_name().is_some_and(|n| n == "verdict.toml")
            && verdict.run == run
            && verdict.status == "accepted"
            && verdict.routine_accepted
            && verdict.error.is_none(),
        "RNG evidence is not an accepted native run"
    );
    authenticate_artifacts(run, &verdict.artifacts)?;
    let predecessor = verdict.predecessor.as_ref().context("RNG predecessor missing")?;
    decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
        checkout: checkout.clone(),
        pointer: predecessor.pointer.clone(),
    })?;
    ensure!(digest(&predecessor.pointer)? == predecessor.sha256, "RNG predecessor changed");
    let source = decomp_source::verify_source(&checkout, &checkout.join("deps/snesrev-sm"))?;
    ensure!(verdict.source.as_ref() == Some(&source), "RNG source identity changed");
    let required = [
        "tools/decomp/next-random-job.toml",
        "tools/decomp/next_random.rs",
        "tools/decomp/next_random_oracle.c",
        "tools/decomp/next_random_live.c",
        "tools/decomp/live.c",
        "crates/pipeline/src/rng_job.rs",
        "crates/pipeline/src/routine_live.rs",
    ];
    ensure!(verdict.implementation.len() == required.len(), "RNG implementation inventory differs");
    for name in required {
        let expected = verdict.implementation.get(name).context("RNG implementation inventory incomplete")?;
        ensure!(digest(&checkout.join(name))? == *expected, "RNG implementation changed: {name}");
    }
    ensure!(verdict.binaries.len() == 3, "RNG binary inventory differs");
    for name in ["c", "rust", "libnext_random.a"] {
        let expected = verdict.binaries.get(name).context("RNG binary inventory incomplete")?;
        ensure!(digest(&sealed_path(run, name)?)? == *expected, "RNG binary changed: {name}");
    }
    let proof = verdict.synthetic.as_ref().context("RNG synthetic proof missing")?;
    let table = fs::read(sealed_path(run, "c-exhaustive.stdout")?)?;
    ensure!(
        proof.all_65536_seeds
            && proof.full_wram_unchanged_outside_two_bytes
            && table.len() == 131072
            && table == fs::read(sealed_path(run, "rust-exhaustive.stdout")?)?
            && format!("{:x}", Sha256::digest(&table)) == proof.table_sha256,
        "RNG exhaustive proof differs"
    );
    let job: Job = toml::from_str(&fs::read_to_string(checkout.join("tools/decomp/next-random-job.toml"))?)?;
    ensure!(
        job.job == "next-random"
            && job.source_revision == source.revision
            && proof.independent_examples == job.independent_examples,
        "RNG proof contract differs"
    );
    let live_relative =
        verdict.live_run.strip_prefix(run).context("RNG live run outside retained run")?.join("verdict.toml");
    ensure!(live_relative != Path::new("verdict.toml"), "RNG live run must be a sealed child run");
    let live_path = sealed_path(run, live_relative.to_str().context("RNG live path must be UTF-8")?)?;
    ensure!(digest(&live_path)? == verdict.live_verdict_sha256, "RNG live verdict changed");
    let live: routine_live::Verdict = toml::from_str(&fs::read_to_string(live_path)?)?;
    ensure!(
        live.run == verdict.live_run
            && live.status == "mixed-C-Rust-integration"
            && live.mixed_integration
            && live.error.is_none()
            && live.contract.as_deref() == Some("decomp-live-v2")
            && live.offline_acceptance_sha256.as_deref() == Some(predecessor.sha256.as_str())
            && live.remaining_gameplay_cpu_fallback == verdict.remaining_cpu_fallback,
        "RNG live acceptance records differ"
    );
    ensure!(
        verdict.captures.len() == 16
            && verdict.observed_calls > 0
            && verdict.live_hits.len() == 4
            && live.rng_subjects.len() == 4
            && live.subjects.len() == 4,
        "RNG live coverage incomplete"
    );
    for name in ["c-0", "c-1", "mixed-0", "mixed-1"] {
        let mixed = name.starts_with("mixed");
        let rows = verdict.live_hits.get(name).context("RNG subject missing")?;
        let sealed = live.rng_subjects.get(name).context("RNG live subject missing")?;
        let decompressions = live.subjects.get(name).context("decomp live subject missing")?;
        ensure!(rows.len() == 2 && sealed.len() == 2 && decompressions.len() == 2, "RNG reset epochs incomplete");
        for epoch in 0..2 {
            let row = &rows[epoch];
            let original = &sealed[epoch];
            let decomp = &decompressions[epoch];
            ensure!(
                row.epoch == epoch
                    && original.epoch == epoch
                    && row.rng_c_hits == original.rng_c_hits
                    && row.rng_rust_hits == original.rng_rust_hits
                    && if mixed {
                        row.rng_rust_hits == verdict.observed_calls && row.rng_c_hits == 0
                    } else {
                        row.rng_c_hits == verdict.observed_calls && row.rng_rust_hits == 0
                    },
                "RNG epoch execution differs"
            );
            ensure!(
                decomp.epoch == epoch
                    && decomp.frames == 9000
                    && if mixed {
                        decomp.rust_hits == 23 && decomp.c_hits == 0
                    } else {
                        decomp.c_hits == 23 && decomp.rust_hits == 0
                    },
                "decomp epoch execution differs"
            );
        }
    }
    Ok(verdict)
}

fn run(request: &Request) -> Result<Report> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let output = &request.output;
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
        let accepted = &request.accepted;
        decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.clone(),
        })?;
        let predecessor = digest(accepted)?;
        verdict.predecessor = Some(Predecessor { pointer: accepted.clone(), sha256: predecessor.clone() });
        let source = checkout.join("deps/snesrev-sm");
        let identity = decomp_source::verify_source(&checkout, &source)?;
        let job = checkout.join("tools/decomp/next-random-job.toml");
        let manifest: Job = toml::from_str(&fs::read_to_string(&job)?)?;
        ensure!(
            manifest.job == "next-random" && manifest.source_revision == identity.revision,
            "RNG job source identity differs"
        );
        verdict.source = Some(identity.clone());
        let mut implementation = BTreeMap::new();
        for name in [
            "tools/decomp/next-random-job.toml",
            "tools/decomp/next_random.rs",
            "tools/decomp/next_random_oracle.c",
            "tools/decomp/next_random_live.c",
            "tools/decomp/live.c",
            "crates/pipeline/src/rng_job.rs",
            "crates/pipeline/src/routine_live.rs",
        ] {
            implementation.insert(name.to_owned(), digest(&checkout.join(name))?);
            fs::copy(checkout.join(name), run.join(Path::new(name).file_name().unwrap()))?;
        }
        verdict.implementation = implementation.clone();
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        fs::copy(source.join("src/sm_80.c"), run.join("sm_80.c"))?;
        let bank = run.join("sm_80.c");
        let rust = run.join("next_random.rs");
        let oracle = run.join("next_random_oracle.c");
        let c = run.join("c");
        let candidate = run.join("rust");
        let library = run.join("libnext_random.a");
        let strings =
            |items: Vec<PathBuf>| items.into_iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>();
        let mut command = vec![
            "cc".into(),
            "-O2".into(),
            "-fno-strict-aliasing".into(),
            "-ffunction-sections".into(),
            "-fdata-sections".into(),
            "-iquote".into(),
        ];
        command.extend(strings(vec![source.join("src"), bank, oracle]));
        command.extend(["-Wl,--gc-sections".into(), "-o".into(), c.to_string_lossy().into_owned()]);
        execute(request, &run, &command, "c-build")?;
        execute(
            request,
            &run,
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "-O".into(),
                "--cfg".into(),
                "cli".into(),
                rust.to_string_lossy().into_owned(),
                "-o".into(),
                candidate.to_string_lossy().into_owned(),
            ],
            "rust-build",
        )?;
        execute(
            request,
            &run,
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "-O".into(),
                "--crate-type".into(),
                "staticlib".into(),
                "-C".into(),
                "panic=abort".into(),
                rust.to_string_lossy().into_owned(),
                "-o".into(),
                library.to_string_lossy().into_owned(),
            ],
            "rust-library",
        )?;
        let table = execute(request, &run, &[c.to_string_lossy().into_owned(), "exhaustive".into()], "c-exhaustive")?;
        let actual = execute(
            request,
            &run,
            &[candidate.to_string_lossy().into_owned(), "exhaustive".into()],
            "rust-exhaustive",
        )?;
        ensure!(table.len() == 131072 && actual.len() == 131072, "incomplete exhaustive RNG oracle");
        ensure!(
            table == actual,
            "first exhaustive RNG mismatch seed={:?}",
            table.iter().zip(&actual).position(|(a, b)| a != b).map(|i| i / 2)
        );
        for example in &manifest.independent_examples {
            ensure!(
                u64::from(table_word(&table, example.seed)?) == example.expected,
                "independent RNG example differs"
            );
        }
        verdict.synthetic = Some(Synthetic {
            all_65536_seeds: true,
            full_wram_unchanged_outside_two_bytes: true,
            independent_examples: manifest.independent_examples.clone(),
            table_sha256: format!("{:x}", Sha256::digest(&table)),
        });
        fs::write(run.join("abi-control.c"), include_str!("rng_abi_control.c"))?;
        execute(
            request,
            &run,
            &[
                "cc".into(),
                run.join("abi-control.c").to_string_lossy().into_owned(),
                library.to_string_lossy().into_owned(),
                "-lm".into(),
                "-ldl".into(),
                "-lpthread".into(),
                "-o".into(),
                run.join("abi-control").to_string_lossy().into_owned(),
            ],
            "abi-build",
        )?;
        execute(request, &run, &[run.join("abi-control").to_string_lossy().into_owned()], "abi-control")?;
        let live = routine_live::verify(routine_live::Request {
            checkout: checkout.clone(),
            output: run.join("live"),
            root: request.root.clone(),
            accepted: accepted.clone(),
            workers: request.workers,
            timeout: request.timeout,
            frames: request.frames,
            live_jobs: request.live_jobs,
            next_random_library: Some(library.clone()),
            ..Default::default()
        })?;
        ensure!(live.mixed_integration, "RNG live integration failed: {:?}", live.error);
        let live_run = &live.run;
        let fixtures = live_run.join("c-0/epoch-0/fixtures");
        let mut before_files = Vec::new();
        for entry in fs::read_dir(&fixtures)? {
            let p = entry?.path();
            let n = p.file_name().unwrap().to_string_lossy();
            if n.starts_with("rng-") && n.ends_with("-before.wram") {
                before_files.push(p);
            }
        }
        before_files.sort();
        ensure!(before_files.len() == 16, "missing observed RNG fixtures");
        let mut captures = Vec::new();
        for (i, before) in before_files.iter().enumerate() {
            let name = before.file_name().unwrap().to_string_lossy().replace("before", "after");
            let after = fixtures.join(name);
            let expected = fs::read(&after)?;
            ensure!(expected.len() == 0x20000, "captured WRAM extent differs");
            for (name, binary) in [("c", &c), ("rust", &candidate)] {
                let output = run.join(format!("capture-{i}-{name}.wram"));
                let result = execute(
                    request,
                    &run,
                    &strings(vec![binary.clone(), before.clone(), output.clone()]),
                    &format!("capture-{i}-{name}"),
                )?;
                ensure!(
                    fs::read(output)? == expected
                        && std::str::from_utf8(&result)?.trim().parse::<u16>()?
                            == u16::from_le_bytes([expected[1509], expected[1510]]),
                    "captured RNG mismatch {i} {name}"
                );
            }
            captures.push(Capture {
                before: before.clone(),
                before_sha256: digest(before)?,
                after: after.clone(),
                after_sha256: digest(&after)?,
            });
        }
        let effects = execute(
            request,
            &run,
            &[
                "gzip".into(),
                "-cd".into(),
                "--".into(),
                live_run.join("c-0/epoch-0/hardware.log.gz").to_string_lossy().into_owned(),
            ],
            "effects-decode",
        )?;
        let calls = verify_effects(std::str::from_utf8(&effects)?, &table, live.rng_subjects["c-0"][0].rng_c_hits)?;
        // Decoded effects duplicate the sealed compressed evidence.
        fs::remove_file(run.join("effects-decode.stdout"))?;
        decomp_acceptance::authenticate(decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.clone(),
        })?;
        ensure!(digest(accepted)? == predecessor, "RNG predecessor changed");
        ensure!(
            decomp_source::verify_source(&checkout, &source)? == identity,
            "RNG upstream source changed during verification"
        );
        for (name, hash) in implementation {
            ensure!(digest(&checkout.join(&name))? == hash, "RNG implementation changed: {name}");
        }
        verdict.captures = captures;
        verdict.observed_calls = calls;
        verdict.live_run = live_run.clone();
        verdict.live_verdict_sha256 = digest(&live_run.join("verdict.toml"))?;
        verdict.live_hits = live.rng_subjects.clone();
        verdict.remaining_cpu_fallback = live.remaining_gameplay_cpu_fallback;
        let mut binaries = BTreeMap::new();
        for p in [c, candidate, library] {
            binaries.insert(p.file_name().unwrap().to_string_lossy().into_owned(), digest(&p)?);
        }
        verdict.binaries = binaries;
        let mut artifacts = BTreeMap::new();
        inventory(&run, &run, &mut artifacts)?;
        verdict.artifacts = artifacts;
        verdict.scope="NextRandom all input words offline; observed 9000-frame mixed C/Rust prefix only; semantic LE store order, not machine instruction timing".into();
        verdict.status = "accepted".into();
        verdict.routine_accepted = true;
        Ok(())
    })();
    if let Err(error) = result {
        verdict.error = Some(format!("{error:#}"));
    }
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    ensure!(verdict.routine_accepted, "{:?}; evidence={}", verdict.error, run.display());
    authenticate(AuthenticateRequest { checkout, verdict: run.join("verdict.toml") })?;
    Ok(Report {
        status: "accepted".into(),
        requested_count: 1,
        accepted_jobs: vec![AcceptedJob {
            job: "next-random".into(),
            exercised: true,
            calls: verdict.observed_calls,
            evidence: run.clone(),
            verdict_sha256: digest(&run.join("verdict.toml"))?,
        }],
        remaining_c_routines: "untranslated; no complete routine census".into(),
        remaining_cpu_fallback: Some(verdict.remaining_cpu_fallback),
        error: None,
    })
}
pub fn operate(request: Request) -> Result<Report> {
    fs::create_dir_all(&request.output)?;
    let mut report = Report {
        status: "error".into(),
        requested_count: request.count,
        accepted_jobs: vec![],
        remaining_c_routines: "untranslated; no complete routine census".into(),
        remaining_cpu_fallback: None,
        error: None,
    };
    let result = (|| -> Result<Report> {
        ensure!(request.job == "next-random", "unknown job; no candidate selected");
        ensure!(request.count == 1, "requested count must be exactly 1; only one dependency-ready job exists");
        run(&request)
    })();
    match result {
        Ok(value) => report = value,
        Err(error) => report.error = Some(format!("{error:#}")),
    };
    atomic_toml(&request.output.join("progress.toml"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retained_rng_artifacts_reject_missing_tampered_extra_symlink_and_escaping_members() {
        let run = std::env::temp_dir().join(format!(
            "sm-rng-auth-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&run).unwrap();
        fs::write(run.join("verdict.toml"), "status = \"accepted\"\n").unwrap();
        fs::write(run.join("state.bin"), b"sealed").unwrap();
        let artifacts = BTreeMap::from([("state.bin".into(), digest(&run.join("state.bin")).unwrap())]);
        authenticate_artifacts(&run, &artifacts).unwrap();
        fs::write(run.join("state.bin"), b"changed").unwrap();
        assert!(authenticate_artifacts(&run, &artifacts).is_err());
        fs::remove_file(run.join("state.bin")).unwrap();
        assert!(authenticate_artifacts(&run, &artifacts).is_err());
        fs::write(run.join("state.bin"), b"sealed").unwrap();
        fs::write(run.join("extra.bin"), b"unsealed").unwrap();
        assert!(authenticate_artifacts(&run, &artifacts).is_err());
        fs::remove_file(run.join("extra.bin")).unwrap();
        std::os::unix::fs::symlink(run.join("state.bin"), run.join("alias.bin")).unwrap();
        let aliases = BTreeMap::from([("alias.bin".into(), artifacts["state.bin"].clone())]);
        assert!(authenticate_artifacts(&run, &aliases).is_err());
        fs::remove_file(run.join("alias.bin")).unwrap();
        let escaping = BTreeMap::from([("../state.bin".into(), artifacts["state.bin"].clone())]);
        assert!(authenticate_artifacts(&run, &escaping).is_err());
        assert!(authenticate_artifacts(&run, &BTreeMap::new()).is_err());
        fs::remove_dir_all(run).unwrap();
    }

    #[test]
    fn ordered_rng_effects_bind_values_stores_and_call_count() {
        let mut table = vec![0; 131072];
        table[2..4].copy_from_slice(&278u16.to_le_bytes());
        let row = "0 rng 1 1 278 2 1509:22 1510:1\n";
        assert_eq!(verify_effects(row, &table, 1).unwrap(), 1);
        for changed in
            [row.replace("278", "279"), row.replace("1509:22 1510:1", "1510:1 1509:22"), format!("{row}{row}")]
        {
            assert!(verify_effects(&changed, &table, 1).is_err());
        }
    }
}
