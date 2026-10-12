//! Native terrain, room and hardcoded allocation with synchronous amount checks.
use crate::family_model::*;
use crate::{atomic_toml, builds, digest, family, inputs};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub fn normalize(manifest: &Manifest) -> Result<Manifest> {
    let mut normalized = manifest.clone();
    let mut groups = BTreeMap::<String, Vec<Case>>::new();
    for case in &manifest.cases {
        let kind = case.allocator.as_deref().unwrap_or("terrain");
        ensure!(["terrain", "room", "hardcoded"].contains(&kind), "invalid allocator");
        groups.entry(kind.into()).or_default().push(case.clone());
    }
    normalized.routines = groups
        .into_iter()
        .map(|(symbol, cases)| Routine {
            symbol,
            arg_types: Some(vec!["uint16".into(), "int32".into()]),
            returns: "uint32".into(),
            cases,
            ..Default::default()
        })
        .collect();
    Ok(normalized)
}

pub fn build(
    request: &VerifyRequest,
    manifest: &Manifest,
    checkout: &Path,
    source: &Path,
    run: &Path,
) -> Result<Implementation> {
    let module = fs::canonicalize(family::source_path(checkout, &manifest.rust_module))?;
    let memory = inputs::wrapper(inputs::WrapperRequest {
        checkout: checkout.into(),
        run: run.into(),
        modules: vec![module.clone()],
        imports: vec![],
    })?;
    fs::write(
        run.join("wrapper.rs"),
        include_str!("plm_spawn_abi.rs.in")
            .replace("__MEMORY__", &format!("{:?}", memory.path))
            .replace("__MODULE__", &format!("{module:?}")),
    )?;
    fs::write(
        run.join("oracle.c"),
        include_str!("plm_spawn_host.c.in")
            .replace("__HEADLESS__", &format!("{:?}", checkout.join("tools/decomp/headless.c"))),
    )?;
    let root = &request.root;
    let workers = request.workers;
    builds::execute(
        &[
            "rustc".into(),
            "--edition=2024".into(),
            "-O".into(),
            "--crate-type=staticlib".into(),
            format!("--emit=link,dep-info={}", run.join("compiler.d").display()),
            run.join("wrapper.rs").to_string_lossy().into_owned(),
            "-o".into(),
            run.join("libplm.a").to_string_lossy().into_owned(),
        ],
        run,
        "rust-build",
        root,
        workers,
    )?;
    let mut command = vec![
        "cc".into(),
        "-O0".into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.join("src").to_string_lossy().into_owned(),
    ];
    command.extend(family::c_units(source)?.iter().map(|p| p.to_string_lossy().into_owned()));
    command.extend([
        run.join("oracle.c").to_string_lossy().into_owned(),
        run.join("libplm.a").to_string_lossy().into_owned(),
        "-Wl,--gc-sections".into(),
        "-Wl,--wrap=Call".into(),
        "-Wl,--wrap=cpu_runOpcode".into(),
        "-Wl,--wrap=DecompressToMem".into(),
        "-lm".into(),
        "-ldl".into(),
        "-lpthread".into(),
        "-o".into(),
        run.join("spawn").to_string_lossy().into_owned(),
    ]);
    builds::c_build(builds::CBuildRequest {
        command,
        run: run.into(),
        checkout: checkout.into(),
        root: root.clone(),
        workers,
    })?;
    builds::execute(
        &[
            run.join("spawn").to_string_lossy().into_owned(),
            "abi".into(),
            "0".into(),
            "0".into(),
            "reserved".into(),
            "reserved".into(),
            run.join("rom.bin").to_string_lossy().into_owned(),
            "reserved".into(),
        ],
        run,
        "abi-controls",
        root,
        workers,
    )?;
    fs::copy(&module, run.join("candidate.rs"))?;
    let dependencies = inputs::dependencies(inputs::DependenciesRequest {
        checkout: checkout.into(),
        run: run.into(),
        extra: manifest.rust_dependencies.clone(),
    })?;
    let mut harness = BTreeMap::new();
    for (name, compiled) in [
        ("plm_spawn.rs", include_str!("plm_spawn.rs")),
        ("plm_spawn_abi.rs.in", include_str!("plm_spawn_abi.rs.in")),
        ("plm_spawn_host.c.in", include_str!("plm_spawn_host.c.in")),
        ("family.rs", include_str!("family.rs")),
        ("batch.rs", include_str!("batch.rs")),
        ("cleanup.rs", include_str!("cleanup.rs")),
        ("fixture_execution.rs", include_str!("fixture_execution.rs")),
        ("metrics.rs", include_str!("metrics.rs")),
        ("resource_sampler.rs", include_str!("resource_sampler.rs")),
    ] {
        let key = format!("crates/pipeline/src/{name}");
        let hash = digest(&checkout.join(&key))?;
        ensure!(hash == format!("{:x}", Sha256::digest(compiled)), "stale PLM executable: {name}");
        harness.insert(key, hash);
        fs::copy(checkout.join(format!("crates/pipeline/src/{name}")), run.join(format!("pipeline-{name}")))?;
    }
    let key = "tools/decomp/headless.c";
    harness.insert(key.to_owned(), digest(&checkout.join(key))?);
    fs::copy(checkout.join(key), run.join("pipeline-headless.c"))?;
    Ok(Implementation {
        fixture_execution: Some("dependency-v1-fresh".into()),
        dependency_contract: "rust-plm-v2".into(),
        source_root: checkout.into(),
        rust_dependencies: dependencies,
        harness_sources: harness,
        module: module.clone(),
        module_sha256: digest(&module)?,
        memory_sha256: digest(&checkout.join("crates/port/src/memory.rs"))?,
        binaries: BTreeMap::from([
            ("spawn".into(), digest(&run.join("spawn"))?),
            ("libplm.a".into(), digest(&run.join("libplm.a"))?),
        ]),
        ..Default::default()
    })
}

pub fn check(directory: &Path, fixture: &Case) -> Result<CaseRecord> {
    let mut results = BTreeMap::new();
    for subject in ["c", "rust"] {
        let text = fs::read_to_string(directory.join(format!("{subject}.stdout")))?;
        let values = text
            .lines()
            .last()
            .context("PLM metrics missing")?
            .split_whitespace()
            .map(str::parse::<i64>)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(values.len() == 4, "PLM metric extent");
        results.insert(
            subject.into(),
            SpawnMetrics {
                flags: values[0],
                amount: values[1],
                cpu_fallback_calls: u64::try_from(values[2])?,
                cpu_fallback_instructions: u64::try_from(values[3])?,
            },
        );
    }
    ensure!(results["c"] == results["rust"], "native SpawnPLM flags/amount mismatch");
    let rust = &results["rust"];
    ensure!(rust.cpu_fallback_calls == 0 && rust.cpu_fallback_instructions == 0, "SpawnPLM CPU fallback");
    if let Some(expected) = fixture.expected_amount {
        ensure!(expected == rust.amount, "independent amount mismatch");
    }
    if let Some(expected) = fixture.expected_flags {
        ensure!(expected == rust.flags, "independent flags mismatch");
    }
    let before = fs::read(directory.join("before.wram"))?;
    let c = fs::read(directory.join("c-after.wram"))?;
    let rust = fs::read(directory.join("rust-after.wram"))?;
    ensure!(before.len() == 0x20000 && c.len() == 0x20000 && rust.len() == 0x20000, "PLM full WRAM extent");
    if c != rust {
        let offset = c.iter().zip(&rust).position(|(a, b)| a != b).unwrap();
        atomic_toml(
            &directory.join("rejection.toml"),
            &CaseRecord {
                first_mismatch: Some(Mismatch { offset, c: c[offset], rust: rust[offset] }),
                ..Default::default()
            },
        )?;
        anyhow::bail!("WRAM mismatch at {offset:#x}");
    }
    if fixture.expected_after_file.as_ref().is_some() {
        ensure!(c == fs::read(directory.join("expected-after.wram"))?, "captured PLM WRAM mismatch");
    }
    Ok(CaseRecord { spawn_results: results, ..Default::default() })
}

pub fn case(
    _request: &VerifyRequest,
    routine: &Routine,
    fixture: &Case,
    run: &Path,
    worker: &mut crate::fixture_execution::Worker,
) -> Result<CaseRecord> {
    let name = fixture.name.as_str();
    let kind = routine.symbol.as_str();
    let directory = run.join(name);
    family::prepare_case(fixture, &directory, &["uint16".into(), "int32".into()])?;
    for subject in ["rust", "c"] {
        worker.execute(
            &[
                run.join("spawn").to_string_lossy().into_owned(),
                subject.into(),
                fixture.a.to_string(),
                fixture.b.to_string(),
                directory.join("before.wram").to_string_lossy().into_owned(),
                directory.join(format!("{subject}-after.wram")).to_string_lossy().into_owned(),
                run.join("rom.bin").to_string_lossy().into_owned(),
                match kind {
                    "terrain" => "0",
                    "room" => "1",
                    "hardcoded" => "2",
                    _ => anyhow::bail!("invalid allocator"),
                }
                .into(),
            ],
            &directory,
            subject,
        )?;
    }
    let mut record = check(&directory, fixture)?;
    record.name = name.into();
    record.routine = kind.into();
    Ok(record)
}
