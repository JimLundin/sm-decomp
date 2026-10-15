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
    if manifest.callback_adapter.as_deref() == Some("game-host") {
        ensure!(!amount, "shared game host requires PLM without collision amount");
        return Ok(include_str!("game_host_abi.rs.in")
            .replace("__MODULE__", &format!("{module:?}"))
            .replace("__NAMES__", &format!("{names:?}")));
    }
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

pub(crate) fn build(
    recipe: Option<&crate::recipe::Recipe>,
    request: &VerifyRequest,
    manifest: &Manifest,
    checkout: &Path,
    source: &Path,
    run: &Path,
    execution: Option<&builds::StrictExecution>,
) -> Result<Implementation> {
    let module = fs::canonicalize(family::source_path(checkout, &manifest.rust_module))?;
    let game_host = manifest.callback_adapter.as_deref() == Some("game-host");
    let power_bomb = request.adapter == "power-bomb" || game_host;
    let mut manifest = manifest.clone();
    if power_bomb && !game_host {
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
    let memory = if let Some(execution) = execution {
        inputs::canonical_wrapper(checkout, run, &execution.rust_inputs)?
    } else {
        inputs::wrapper(inputs::WrapperRequest {
            checkout: checkout.into(),
            run: run.into(),
            modules: vec![module.clone()],
            imports: manifest.rust_imports.clone(),
        })?
    };
    let root_imports = manifest
        .rust_imports
        .iter()
        .map(|name| {
            ensure!(family::identifier(name), "invalid import");
            Ok(format!("use memory::{name};\n"))
        })
        .collect::<Result<String>>()?;
    let wrapper = if execution.is_some() {
        format!(
            "include!(\"/inputs/generated/memory.rs\");\n{}",
            abi(&manifest, &module)?.replace(
                &format!("#[path={module:?}] mod family;"),
                &format!("use {} as family;", module.file_stem().unwrap().to_string_lossy())
            )
        )
    } else {
        format!("#[path={:?}] mod memory;use memory::*;\n{root_imports}{}", memory.path, abi(&manifest, &module)?)
    };
    family::write_build_input(&run.join("wrapper.rs"), wrapper.as_bytes(), execution.is_some())?;
    let headless = if execution.is_some() {
        "/inputs/rust/tools/decomp/headless.c".into()
    } else {
        checkout.join("tools/decomp/headless.c")
    };
    let mut oracle = include_str!("callback_host.c.in")
        .replace("__HEADLESS__", &format!("{headless:?}"))
        .replace(
            "__HDMA_HOST__",
            &format!(
                "{}{}",
                if power_bomb { include_str!("power_bomb_host.c.in") } else { "" },
                if game_host { include_str!("game_host_dependency.c.in") } else { "" }
            ),
        )
        .replace("__BRANCHES__", &branches.join("else "));
    if execution.is_some() {
        oracle = packed_snapshots(&oracle)?;
    }
    family::write_build_input(&run.join("oracle.c"), oracle.as_bytes(), execution.is_some())?;
    let root = &request.root;
    let workers = request.workers;
    let description = crate::recipe::rust_compiler_command(recipe, run);
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "rust-build", checkout, source)?;
    } else {
        builds::execute(&description.command, run, "rust-build", root, workers)?;
    }
    if power_bomb {
        let original = fs::read_to_string(source.join("src/sm_88.c"))?;
        let token = "uint16 SpawnHdmaObject(uint8 db, const void *p)";
        ensure!(original.matches(token).count() == 1, "HDMA definition seal not found");
        let seam = run.join("sm_88-allocation-seam.c");
        family::write_build_input(
            &seam,
            original.replace(token, "uint16 PortActualSpawnHdmaObject(uint8 db, const void *p)").as_bytes(),
            execution.is_some(),
        )?;
    }
    if game_host {
        let original = fs::read_to_string(source.join("src/sm_80.c"))?;
        let token = "void NMI_ProcessVramWriteQueue(void)";
        ensure!(original.matches(token).count() == 1, "VRAM definition seal not found");
        let seam = run.join("sm_80-vram-seam.c");
        family::write_build_input(
            &seam,
            original.replace(token, "void PortActualNmiProcessVramWriteQueue(void)").as_bytes(),
            execution.is_some(),
        )?;
    }
    let description =
        crate::recipe::c_compiler_command(recipe, &manifest, source, run, &family::c_units(source)?, false)?;
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "c-build", checkout, source)?;
    } else {
        builds::c_build(builds::CBuildRequest {
            command: description.command,
            run: run.into(),
            checkout: checkout.into(),
            root: root.clone(),
            workers,
        })?;
    }
    fs::copy(&module, run.join("candidate.rs"))?;
    let dependency_request = inputs::DependenciesRequest {
        checkout: checkout.into(),
        run: run.into(),
        extra: manifest.rust_dependencies.clone(),
    };
    let dependencies = if execution.is_some() {
        inputs::confined_dependencies(dependency_request)?
    } else {
        inputs::dependencies(dependency_request)?
    };
    let mut harness = inputs::implementation_sources(checkout, run, execution.is_some())?;
    let key = "tools/decomp/headless.c";
    harness.insert(key.to_owned(), digest(&checkout.join(key))?);
    fs::copy(checkout.join(key), run.join("pipeline-headless.c"))?;
    Ok(Implementation {
        fixture_execution: Some("dependency-v1-fresh".into()),
        manifest: None,
        dependency_contract: "rust-callback-v2".into(),
        callback_adapter: Some(
            if game_host {
                "game-host"
            } else if power_bomb {
                "power-bomb"
            } else {
                "callback"
            }
            .into(),
        ),
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

fn check_game_host_log(log: &str, count: u64) -> Result<()> {
    let mut requests = 0_u64;
    let mut vram = None;
    for line in log.lines() {
        let words = line.split_whitespace().collect::<Vec<_>>();
        let number = |at: usize| -> Result<u64> { Ok(words.get(at).context("truncated game host effect")?.parse()?) };
        match words.first().copied() {
            Some("vram-begin") => {
                ensure!(words.len() == 2 && vram.is_none(), "invalid nested VRAM request");
                requests += 1;
                ensure!(number(1)? == requests, "VRAM request order differs");
                vram = Some(requests);
            }
            Some("vram-end") => {
                ensure!(words.len() == 2 && vram == Some(number(1)?), "VRAM request return differs");
                vram = None;
            }
            Some("hdma") => {
                ensure!(words.len() == 6 && vram.is_none(), "invalid HDMA host effect");
                for at in 1..6 {
                    number(at)?;
                }
                requests += 1;
            }
            Some("register" | "bus-read" | "bus-write") => {
                ensure!(words.len() == 3, "invalid hardware host effect");
                number(1)?;
                ensure!(number(2)? <= 255, "hardware byte outside range");
            }
            Some("ppu") => {
                ensure!(
                    words.len() == 4 && number(1)? <= 1 && number(2)? <= 255 && number(3)? <= 255,
                    "invalid PPU host effect"
                );
            }
            Some(_) => {
                ensure!(words.len() == 4 && vram.is_none(), "invalid PLM host effect");
                number(0)?;
                words[1].parse::<i32>()?;
                words[2].parse::<i32>()?;
                ensure!(number(3)? <= 255, "PLM result outside range");
                requests += 1;
            }
            None => anyhow::bail!("empty game host effect"),
        }
    }
    ensure!(vram.is_none() && requests == count, "game host request extent differs");
    Ok(())
}

pub fn check(directory: &Path, fixture: &Case, game_host: bool) -> Result<CaseRecord> {
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
    if game_host {
        check_game_host_log(&log, count)?;
    } else {
        ensure!(log.lines().count() == count as usize, "callback log count differs");
    }
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

pub(crate) fn case(
    manifest: &Manifest,
    routine: &Routine,
    fixture: &Case,
    run: &Path,
    resume: bool,
    mut execute: impl FnMut(&Path, &str) -> Result<()>,
) -> Result<CaseRecord> {
    let name = fixture.name.as_str();
    let symbol = routine.symbol.as_str();
    let directory = run.join(name);
    family::prepare_case_with(fixture, &directory, &family::types(routine)?, resume)?;
    for subject in ["rust", "c"] {
        fs::create_dir_all(directory.join(subject))?;
        execute(&directory, subject)?;
        if resume {
            unpack_snapshots(&directory, subject)?;
        }
    }
    let mut record = check(&directory, fixture, manifest.callback_adapter.as_deref() == Some("game-host"))?;
    record.routine = symbol.into();
    record.name = name.into();
    Ok(record)
}

/// Keep the same sequence and all full-WRAM images while using a fixed set of
/// output files inside confinement. The parent reconstructs historical names.
fn packed_snapshots(oracle: &str) -> Result<String> {
    let start = oracle.find("static void CallbackSnapshot(").context("callback snapshot declaration missing")?;
    let end = start + oracle[start..].find("\n}\n").context("callback snapshot body missing")? + 3;
    let mut result = oracle.to_owned();
    result.replace_range(start..end, "static FILE *callback_states;\nstatic void CallbackSnapshot(const char *phase) {\n uint64_t index=callbacks; uint8_t after=!strcmp(phase,\"after\");\n if(fwrite(&index,8,1,callback_states)!=1||fwrite(&after,1,1,callback_states)!=1||fwrite(g_ram,1,sizeof(g_ram),callback_states)!=sizeof(g_ram))exit(2);\n}\n");
    let token = "callback_log=fopen(log_path,\"w\");if(!callback_log)return 2;";
    ensure!(result.matches(token).count() == 1, "callback log initialization differs");
    result = result.replace(token, &format!("{token}\n snprintf(log_path,sizeof(log_path),\"%s/callback-states.bin\",callback_directory);callback_states=fopen(log_path,\"wb\");if(!callback_states)return 2;"));
    result = result
        .replace("if(fclose(callback_log))return 2;", "if(fclose(callback_log)||fclose(callback_states))return 2;");
    Ok(result)
}

fn unpack_snapshots(directory: &Path, subject: &str) -> Result<()> {
    snapshots(directory, subject, true)
}

pub(crate) fn authenticate_packed_snapshots(directory: &Path, subject: &str) -> Result<()> {
    snapshots(directory, subject, false)
}

fn snapshots(directory: &Path, subject: &str, publish: bool) -> Result<()> {
    let bytes = inputs::bounded::read(directory.join(format!("{subject}-callback-states.bin")))?;
    let count = metrics(directory, subject)?.c_gameplay_callbacks;
    let stride = 9 + 0x20000;
    ensure!(count <= 4096 && bytes.len() == usize::try_from(count)? * 2 * stride, "packed callback extent differs");
    for (at, record) in bytes.chunks_exact(stride).enumerate() {
        let index = u64::from_le_bytes(record[..8].try_into()?);
        let after = record[8];
        ensure!(index == (at / 2 + 1) as u64 && after == (at % 2) as u8, "packed callback sequence differs");
        let phase = if after == 0 { "before" } else { "after" };
        let path = directory.join(subject).join(format!("callback-{index:04}-{phase}.wram"));
        if publish {
            inputs::immutable_bytes(&path, &record[9..])?;
        } else {
            ensure!(
                inputs::bounded::read(&path)? == record[9..],
                "retained callback snapshot differs from confined output"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_host_log_preserves_ordered_hardware_and_requires_complete_vram_requests() {
        let log =
            "vram-begin 1\nregister 8469 128\nppu 1 21 128\nbus-read 82944 12\nvram-end 1\nhdma 136 64 40 35534 4\n";
        assert!(check_game_host_log(log, 2).is_ok());
        assert!(check_game_host_log(&log.replace("vram-end 1", "vram-end 2"), 2).is_err());
        assert!(check_game_host_log("vram-begin 1\nregister 1 256\nvram-end 1\n", 1).is_err());
        assert!(check_game_host_log("vram-begin 1\n", 1).is_err());
    }
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
