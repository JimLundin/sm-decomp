//! Actual C callbacks remain measured dependencies of native Rust gameplay.
use crate::family_model::*;
use crate::{atomic_toml, builds, digest, family, inputs};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub fn abi(manifest: &Manifest, module: &Path) -> Result<String> {
    let context = manifest.plm_amount_context.as_deref().unwrap_or("collision");
    ensure!(["collision", "none"].contains(&context), "invalid PLM amount context");
    let amount = context == "collision";
    let power_bomb = module.file_name().is_some_and(|n| n == "power_bomb.rs")
        || manifest.callback_adapter.as_deref() == Some("power-bomb");
    let names = manifest.routines.iter().map(|r| r.symbol.as_str()).collect::<Vec<_>>();
    Ok(include_str!("callback_abi.rs.in")
        .replace("__MODULE__", &format!("{module:?}"))
        .replace("__PLM_HOST__", if power_bomb { "crate::projectile_special::Host" } else { "family::Host" })
        .replace("__HDMA_ABI__", if power_bomb { include_str!("power_bomb_abi.rs.in") } else { "" })
        .replace(
            "__BRIDGE__",
            if amount { "sm_rust_collision_spawn_plm" } else { "sm_rust_collision_spawn_plm_without_amount" },
        )
        .replace("__FFI_AMOUNT__", if amount { ",amount:*mut i32" } else { "" })
        .replace("__METHOD_AMOUNT__", if amount { ",amount:&mut i32" } else { "" })
        .replace("__CALL_AMOUNT__", if amount { ",amount" } else { "" })
        .replace("__NAMES__", &format!("{names:?}")))
}

pub fn build(
    request: &VerifyRequest,
    manifest: &Manifest,
    checkout: &Path,
    source: &Path,
    run: &Path,
) -> Result<Implementation> {
    let module = fs::canonicalize(family::source_path(checkout, &manifest.rust_module))?;
    let power_bomb = request.adapter == "power-bomb";
    let mut manifest = manifest.clone();
    if power_bomb {
        manifest.callback_adapter = Some("power-bomb".into());
    }
    let mut branches = Vec::new();
    for routine in &manifest.routines {
        let symbol = routine.symbol.as_str();
        let kind = routine.returns.as_str();
        let args = family::types(routine)?;
        ensure!(family::identifier(symbol) && args.len() <= 2, "invalid callback signature");
        let actual = args
            .iter()
            .zip(["a", "b"])
            .map(|(kind, arg)| {
                if kind == "rom_ptr" {
                    Ok(format!("(const uint16 *)(g_rom+(((uint32_t){arg}>>16&127)*0x8000)+((uint32_t){arg}&0x7fff))"))
                } else {
                    Ok(format!("({}){arg}", family::c_type(kind)?))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        family::c_type(kind)?;
        let call = format!("{symbol}({})", actual.join(","));
        let statement = if manifest.infrastructure_probe {
            ensure!(symbol == "CollisionCallbackProbe", "infrastructure probe cannot claim actual routine");
            "int32_t amount=a;cur_coll_amt32=&amount;__wrap_SpawnPLM((uint16_t)b);cur_coll_amt32=NULL;result=(uint32_t)amount;".to_owned()
        } else if kind == "void" {
            format!("{call};result=0;")
        } else if kind == "rom_ptr" {
            format!(
                "const uint16 *p={call};uintptr_t start=(uintptr_t)g_rom+(((uint32_t)b>>16&127)*0x8000);if((uintptr_t)p<start||(uintptr_t)p>=start+0x8000)return 3;result=((uint32_t)b&0xff0000)|0x8000|((uintptr_t)p-start);"
            )
        } else {
            format!("result=(uint32_t){call};")
        };
        branches.push(format!("if(!strcmp(argv[2],\"{symbol}\")){{{statement}}}"));
    }
    ensure!(manifest.rom.as_ref().is_some(), "callback verification requires ROM");
    let memory = inputs::wrapper(inputs::WrapperRequest {
        checkout: checkout.into(),
        run: run.into(),
        modules: vec![module.clone()],
        imports: manifest.rust_imports.clone(),
    })?;
    let root_imports = manifest
        .rust_imports
        .iter()
        .map(|name| {
            ensure!(family::identifier(name), "invalid import");
            Ok(format!("use memory::{name};\n"))
        })
        .collect::<Result<String>>()?;
    fs::write(
        run.join("wrapper.rs"),
        format!("#[path={:?}] mod memory;use memory::*;\n{root_imports}{}", memory.path, abi(&manifest, &module)?),
    )?;
    fs::write(
        run.join("oracle.c"),
        include_str!("callback_host.c.in")
            .replace("__HEADLESS__", &format!("{:?}", checkout.join("tools/decomp/headless.c")))
            .replace("__HDMA_HOST__", if power_bomb { include_str!("power_bomb_host.c.in") } else { "" })
            .replace("__BRANCHES__", &branches.join("else ")),
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
            run.join("libcollision.a").to_string_lossy().into_owned(),
        ],
        run,
        "rust-build",
        root,
        workers,
    )?;
    let mut command = vec![
        "cc".into(),
        if power_bomb { "-O0" } else { "-O2" }.into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.join("src").to_string_lossy().into_owned(),
    ];
    let mut units = family::c_units(source)?;
    if power_bomb {
        let original = fs::read_to_string(source.join("src/sm_88.c"))?;
        let token = "uint16 SpawnHdmaObject(uint8 db, const void *p)";
        ensure!(original.matches(token).count() == 1, "HDMA definition seal not found");
        let seam = run.join("sm_88-allocation-seam.c");
        fs::write(&seam, original.replace(token, "uint16 PortActualSpawnHdmaObject(uint8 db, const void *p)"))?;
        let bank =
            units.iter_mut().find(|p| p.file_name().is_some_and(|n| n == "sm_88.c")).context("bank88 missing")?;
        *bank = seam;
    }
    command.extend(units.iter().map(|p| p.to_string_lossy().into_owned()));
    command.extend([
        run.join("oracle.c").to_string_lossy().into_owned(),
        run.join("libcollision.a").to_string_lossy().into_owned(),
        "-Wl,--gc-sections".into(),
        "-Wl,--wrap=SpawnPLM".into(),
        "-Wl,--wrap=Call".into(),
        "-Wl,--wrap=cpu_runOpcode".into(),
        "-Wl,--wrap=DecompressToMem".into(),
        "-lm".into(),
        "-ldl".into(),
        "-lpthread".into(),
        "-o".into(),
        run.join("collision").to_string_lossy().into_owned(),
    ]);
    builds::c_build(builds::CBuildRequest {
        command,
        run: run.into(),
        checkout: checkout.into(),
        root: root.clone(),
        workers,
    })?;
    fs::copy(&module, run.join("candidate.rs"))?;
    let dependencies = inputs::dependencies(inputs::DependenciesRequest {
        checkout: checkout.into(),
        run: run.into(),
        extra: manifest.rust_dependencies.clone(),
    })?;
    let mut harness = BTreeMap::new();
    for (name, compiled) in [
        ("callback.rs", include_str!("callback.rs")),
        ("callback_abi.rs.in", include_str!("callback_abi.rs.in")),
        ("callback_host.c.in", include_str!("callback_host.c.in")),
        ("power_bomb_abi.rs.in", include_str!("power_bomb_abi.rs.in")),
        ("power_bomb_host.c.in", include_str!("power_bomb_host.c.in")),
        ("family.rs", include_str!("family.rs")),
        ("batch.rs", include_str!("batch.rs")),
        ("cleanup.rs", include_str!("cleanup.rs")),
    ] {
        let key = format!("crates/pipeline/src/{name}");
        let hash = digest(&checkout.join(&key))?;
        ensure!(hash == format!("{:x}", Sha256::digest(compiled)), "stale callback executable: {name}");
        harness.insert(key, hash);
        fs::copy(checkout.join(format!("crates/pipeline/src/{name}")), run.join(format!("pipeline-{name}")))?;
    }
    let key = "tools/decomp/headless.c";
    harness.insert(key.to_owned(), digest(&checkout.join(key))?);
    fs::copy(checkout.join(key), run.join("pipeline-headless.c"))?;
    Ok(Implementation {
        manifest: None,
        dependency_contract: "rust-callback-v2".into(),
        callback_adapter: Some(if power_bomb { "power-bomb" } else { "callback" }.into()),
        source_root: checkout.into(),
        rust_dependencies: dependencies,
        harness_sources: harness,
        module: module.clone(),
        module_sha256: digest(&module)?,
        memory_sha256: digest(&checkout.join("crates/port/src/memory.rs"))?,
        binaries: BTreeMap::from([
            ("collision".into(), digest(&run.join("collision"))?),
            ("libcollision.a".into(), digest(&run.join("libcollision.a"))?),
        ]),
    })
}

fn metrics(directory: &Path, subject: &str) -> Result<CallbackMetrics> {
    let text = fs::read_to_string(directory.join(format!("{subject}.stdout")))?;
    let line = text.lines().last().context("missing callback metrics")?;
    let values = line.split_whitespace().map(str::parse::<u64>).collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(values.len() == 4, "callback metrics extent");
    Ok(CallbackMetrics {
        returns: values[0],
        c_gameplay_callbacks: values[1],
        cpu_fallback_calls: values[2],
        cpu_fallback_instructions: values[3],
    })
}

fn callback_files(directory: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let p = entry?.path();
        ensure!(!p.is_symlink() && p.is_file(), "invalid callback artifact");
        files.insert(p.file_name().context("callback filename missing")?.to_string_lossy().into_owned(), fs::read(p)?);
    }
    Ok(files)
}

pub fn check(directory: &Path, fixture: &Case) -> Result<CaseRecord> {
    let c = metrics(directory, "c")?;
    let rust = metrics(directory, "rust")?;
    ensure!(c == rust, "actual callback metrics mismatch");
    ensure!(
        rust.cpu_fallback_calls == 0 && rust.cpu_fallback_instructions == 0,
        "CPU fallback during offline callback comparison"
    );
    let count = rust.c_gameplay_callbacks;
    ensure!(count >= fixture.minimum_callbacks, "callback coverage missing");
    if let Some(expected) = fixture.expected_return {
        ensure!(expected == rust.returns, "independent expected callback return differs");
    }
    let c_files = callback_files(&directory.join("c"))?;
    let rust_files = callback_files(&directory.join("rust"))?;
    ensure!(c_files == rust_files, "ordered actual C callback state/argument sequence mismatch");
    ensure!(c_files.len() == 1 + 2 * count as usize, "callback record extent missing");
    let log = String::from_utf8(c_files.get("callbacks.txt").context("callback log missing")?.clone())?;
    ensure!(log.lines().count() == count as usize, "callback log count differs");
    for index in 1..=count {
        for phase in ["before", "after"] {
            ensure!(
                c_files.get(&format!("callback-{index:04}-{phase}.wram")).is_some_and(|v| v.len() == 0x20000),
                "callback full WRAM extent missing"
            );
        }
    }
    for name in ["before.wram", "c-after.wram", "rust-after.wram"] {
        ensure!(fs::metadata(directory.join(name))?.len() == 0x20000, "callback full WRAM extent missing");
    }
    let c_state = fs::read(directory.join("c-after.wram"))?;
    let rust_state = fs::read(directory.join("rust-after.wram"))?;
    if c_state != rust_state {
        let offset = c_state.iter().zip(&rust_state).position(|(a, b)| a != b).unwrap();
        atomic_toml(
            &directory.join("rejection.toml"),
            &CaseRecord {
                first_mismatch: Some(Mismatch { offset, c: c_state[offset], rust: rust_state[offset] }),
                ..Default::default()
            },
        )?;
        anyhow::bail!("WRAM mismatch at {offset:#x}");
    }
    if fixture.expected_after_file.as_ref().is_some() {
        ensure!(c_state == fs::read(directory.join("expected-after.wram"))?, "captured full WRAM mismatch");
    }
    Ok(CaseRecord {
        results: BTreeMap::from([("c".into(), c), ("rust".into(), rust)]),
        callbacks: count,
        ..Default::default()
    })
}

pub fn case(
    request: &VerifyRequest,
    _manifest: &Manifest,
    routine: &Routine,
    fixture: &Case,
    index: usize,
    run: &Path,
) -> Result<CaseRecord> {
    let name = fixture.name.as_str();
    let symbol = routine.symbol.as_str();
    let directory = run.join(name);
    family::prepare_case(fixture, &directory, &family::types(routine)?)?;
    for subject in ["rust", "c"] {
        fs::create_dir(directory.join(subject))?;
        builds::execute(
            &[
                run.join("collision").to_string_lossy().into_owned(),
                subject.into(),
                symbol.into(),
                fixture.a.to_string(),
                fixture.b.to_string(),
                directory.join("before.wram").to_string_lossy().into_owned(),
                directory.join(format!("{subject}-after.wram")).to_string_lossy().into_owned(),
                index.to_string(),
                run.join("rom.bin").to_string_lossy().into_owned(),
                directory.join(subject).to_string_lossy().into_owned(),
            ],
            &directory,
            subject,
            &request.root,
            request.workers,
        )?;
    }
    let mut record = check(&directory, fixture)?;
    record.routine = symbol.into();
    record.name = name.into();
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_amount_uses_a_distinct_callback_signature_and_invalid_context_fails() {
        let source = abi(
            &Manifest {
                plm_amount_context: Some("none".into()),
                routines: vec![Routine { symbol: "Flight".into(), ..Default::default() }],
                ..Default::default()
            },
            Path::new("/tmp/flight.rs"),
        )
        .unwrap();
        assert!(source.contains("fn sm_rust_collision_spawn_plm_without_amount("));
        assert!(source.contains("header:u16)->Result<u8"));
        assert!(!source.contains("amount:&mut i32"));
        assert!(
            abi(
                &Manifest { plm_amount_context: Some("unknown".into()), ..Default::default() },
                Path::new("/tmp/flight.rs")
            )
            .is_err()
        );
    }
}
