//! Authenticated live observer generation and four independent packed subjects.
use crate::{atomic_toml, builds, callback, digest, family, inputs, observations, packed};
use crate::{
    family_model::{AuthenticateRequest, Manifest},
    live_model::*,
};
use anyhow::{Context, Result, ensure};
use serde::de::DeserializeOwned;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};
const SUBJECTS: [&str; 4] = ["c-0", "c-1", "mixed-0", "mixed-1"];
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";

fn document<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
// Match only a complete definition at the beginning of a source line. This
// preserves call sites and prototypes, including bank-local helper declarations.
fn definition(text: &str, symbol: &str) -> Result<std::ops::Range<usize>> {
    ensure!(family::identifier(symbol), "invalid live symbol");
    let mut found = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if !line.starts_with(char::is_whitespace)
            && let Some(at) = line.find(symbol)
        {
            let prefix = &line[..at];
            let suffix = &line[at + symbol.len()..];
            if !prefix.is_empty()
                && prefix.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ' || c == '\t' || c == '*')
                && prefix.ends_with([' ', '\t', '*'])
                && suffix.trim_start().starts_with('(')
                && let Some(end) = suffix.find('{')
                && !suffix[..end].contains(';')
            {
                found.push(offset..offset + at + symbol.len() + end + 1);
            }
        }
        offset += line.len();
    }
    ensure!(found.len() == 1, "expected one definition for {symbol}");
    Ok(found.remove(0))
}
fn rename(text: &str, symbol: &str, replacement: &str, prototype: bool) -> Result<String> {
    let range = definition(text, symbol)?;
    let original = &text[range.clone()];
    let at = range.start + original.find(symbol).unwrap();
    let mut text = text.to_owned();
    text.replace_range(at..at + symbol.len(), replacement);
    if prototype {
        let anchor = "#include \"funcs.h\"";
        let at = text.find(anchor).context("owning unit lacks declarations include")?;
        let end = at + text[at..].find('\n').context("include newline missing")? + 1;
        text.insert_str(end, &format!("{};\n", original.trim_end_matches('{').trim_end()));
    }
    Ok(text)
}

fn dispatcher_contract(manifest: &mut Manifest, module: &Path, checkout: &Path) -> Result<()> {
    ensure!(
        module == checkout.join("crates/port/src/enemy_projectile_instructions.rs") && manifest.bank == "sm_86.c",
        "dispatcher does not seal installed bank86 module"
    );
    let mut eligible = false;
    for routine in &mut manifest.routines {
        let symbol = routine.symbol.as_str();
        if ["EprojInstruction", "EprojPreinstruction"].contains(&symbol) {
            routine.live_eligible = Some(false);
            continue;
        }
        if routine.live_eligible == Some(false) {
            continue;
        }
        let kind = match symbol {
            "MoveEprojWithVelocity" | "MoveEprojWithVelocityX" | "MoveEprojWithVelocityY" | "Eproj_FuncE722" => {
                "uint16"
            }
            "Eproj_FuncE73E_MoveXY" | "EprojPreInstr_CeresElevatorPlatform" | "EprojPreInstr_CeresElevatorPad" => {
                "void"
            }
            "EprojInstr_Delete" | "EprojInstr_SetPreInstr_" | "EprojInstr_ClearPreInstr" | "EprojInstr_Goto" => {
                "rom_ptr"
            }
            _ => anyhow::bail!("dispatcher routine requires support selection: {symbol}"),
        };
        let args =
            if kind == "rom_ptr" { vec!["uint16".to_owned(), "rom_ptr".to_owned()] } else { vec!["uint16".to_owned()] };
        ensure!(family::types(routine)? == args && routine.returns == kind, "dispatcher leaf signature differs");
        if kind == "rom_ptr" {
            ensure!(routine.pointer_bank == Some(0x86), "dispatcher leaf pointer bank differs");
        }
        eligible = true;
    }
    ensure!(eligible, "no live dispatcher leaf contracts");
    manifest
        .rust_imports
        .extend(["audio", "enemy", "enemy_projectiles", "enemy_projectile_collision", "system"].map(str::to_owned));
    manifest.rust_imports.sort();
    manifest.rust_imports.dedup();
    Ok(())
}

pub fn generate(request: Request) -> Result<Generated> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let run = fs::canonicalize(&request.run)?;
    let source = checkout.join("deps/snesrev-sm");
    let source_identity = family::verify_source(&checkout, &source)?;
    let mut families = Vec::new();
    let mut predecessors = Vec::new();
    for (values, kind) in [
        (&request.family_verdicts, "family"),
        (&request.dispatcher_verdicts, "dispatcher"),
        (&request.callback_verdicts, "callback"),
    ] {
        for verdict_path in values {
            let evidence = family::authenticate(AuthenticateRequest {
                checkout: checkout.clone(),
                verdict: verdict_path.clone(),
            })?;
            ensure!(evidence.source.as_ref() == Some(&source_identity), "family source identity differs");
            let mut manifest: Manifest =
                document(&verdict_path.parent().context("predecessor parent missing")?.join("manifest.toml"))?;
            let module = fs::canonicalize(family::source_path(&checkout, &manifest.rust_module))?;
            if kind == "dispatcher" {
                dispatcher_contract(&mut manifest, &module, &checkout)?;
            }
            if kind == "callback" {
                ensure!(
                    evidence.implementation.as_ref().context("missing implementation")?.dependency_contract
                        == "rust-callback-v2"
                        && evidence.gameplay_translation == Some(true),
                    "callback infrastructure cannot replace gameplay"
                );
            }
            if kind == "family" {
                ensure!(
                    evidence.implementation.as_ref().context("missing implementation")?.dependency_contract
                        == "rust-family-v2",
                    "family evidence must seal an ordinary family adapter"
                );
            }
            families.push((manifest, module, kind));
            predecessors.push(Predecessor {
                path: verdict_path.clone(),
                sha256: digest(verdict_path)?,
                kind: kind.into(),
            });
        }
    }
    ensure!(!families.is_empty(), "no family contracts");
    let positioned = request.room_plm;
    let plm = request.plm_verdict.as_ref();
    ensure!(!positioned || plm.is_some(), "room PLM requires allocation evidence");
    if let Some(plm) = plm {
        let evidence = family::authenticate(AuthenticateRequest { checkout: checkout.clone(), verdict: plm.clone() })?;
        ensure!(
            evidence.source.as_ref() == Some(&source_identity)
                && evidence.implementation.as_ref().context("missing implementation")?.dependency_contract
                    == "rust-plm-v2",
            "native PLM source identity differs"
        );
        let manifest: Manifest =
            document(&Path::new(plm).parent().context("PLM predecessor parent missing")?.join("manifest.toml"))?;
        ensure!(
            fs::canonicalize(family::source_path(&checkout, &manifest.rust_module))?
                == checkout.join("crates/port/src/plm.rs"),
            "native PLM does not seal installed module"
        );
        if positioned {
            ensure!(
                evidence.allocator_cases.contains_key("room") && evidence.allocator_cases.contains_key("hardcoded"),
                "positioned allocation coverage missing"
            );
        }
        predecessors.push(Predecessor { path: plm.clone(), sha256: digest(plm)?, kind: "plm".into() });
    }
    let imports = families.iter().flat_map(|(m, _, _)| m.rust_imports.clone()).collect::<BTreeSet<_>>();
    let mut modules = families.iter().map(|(_, module, _)| module.clone()).collect::<Vec<_>>();
    if plm.is_some() {
        modules.push(checkout.join("crates/port/src/plm.rs"));
    }
    let memory = inputs::wrapper(inputs::WrapperRequest {
        checkout: checkout.clone(),
        run: run.clone(),
        modules,
        imports: imports.into_iter().collect(),
    })?;
    let mut rust = format!("#[path={:?}] mod memory;use memory::*;\n", memory.path);
    if plm.is_some() {
        rust.push_str(include_str!("live_plm.rs.in"));
    }
    let mut units = family::c_units(&source)?;
    let banks = units
        .iter()
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("sm_")))
        .cloned()
        .collect::<Vec<_>>();
    let originals =
        banks.iter().map(|p| Ok((p.clone(), fs::read_to_string(p)?))).collect::<Result<BTreeMap<_, _>>>()?;
    let mut transformed = BTreeMap::new();
    let mut wrappers = String::new();
    let mut symbols = Vec::new();
    let mut dispatcher_symbols = BTreeSet::new();
    for (family_index, (manifest, module, kind)) in families.iter().enumerate() {
        let names = manifest.routines.iter().map(|r| r.symbol.as_str()).collect::<Vec<_>>();
        let abi_symbol = if *kind == "callback" {
            format!("port_collision_{family_index}")
        } else {
            format!("port_family_{family_index}")
        };
        if *kind == "callback" {
            rust.push_str(&format!(
                "mod collision{family_index}{{use super::*;\n{}\n}}\n",
                callback::abi(manifest, module)?.replace("port_collision_call", &abi_symbol)
            ));
        } else {
            let call = if manifest.rom.as_ref().is_some() {
                format!("family{family_index}::dispatch_rom(&mut Wram::new(bytes).unwrap(),rom,name,a,b)")
            } else if manifest.routines.iter().any(|r| r.arg_types.is_some()) {
                format!("family{family_index}::dispatch_i32(&mut Wram::new(bytes).unwrap(),name,a,b)")
            } else {
                format!("family{family_index}::dispatch(&mut Wram::new(bytes).unwrap(),name,a as u16,b as u16)")
            };
            rust.push_str(
                &include_str!("live_family.rs.in")
                    .replace("__MODULE__", &format!("{module:?}"))
                    .replace("__INDEX__", &family_index.to_string())
                    .replace("__NAMES__", &format!("{names:?}"))
                    .replace("__CALL__", &call),
            );
        }
        wrappers.push_str(&format!(
            "extern int64_t {abi_symbol}(uint8_t*,size_t,const uint8_t*,size_t,uint32_t,int32_t,int32_t);\n"
        ));
        for (routine_index, routine) in manifest.routines.iter().enumerate() {
            if routine.live_eligible == Some(false) {
                continue;
            }
            let symbol = routine.symbol.as_str();
            ensure!(
                family::identifier(symbol) && !symbols.iter().any(|v| v == symbol),
                "duplicate or invalid live symbol"
            );
            let owners = originals
                .iter()
                .filter(|(_, text)| definition(text, symbol).is_ok())
                .map(|(p, _)| p.clone())
                .collect::<Vec<_>>();
            ensure!(owners.len() == 1, "expected one owning source unit for {symbol}");
            let owner = &owners[0];
            let text = transformed.get(owner).unwrap_or(&originals[owner]);
            let text = rename(text, symbol, &format!("C_{symbol}"), true)?;
            transformed.insert(owner.clone(), text);
            let args = family::types(routine)?;
            ensure!(args.len() <= 2, "unsupported live signature");
            let ret = routine.returns.as_str();
            let mut ctype = family::c_type(ret)?.to_owned();
            let dispatcher = *kind == "dispatcher";
            if dispatcher && ret == "rom_ptr" {
                ctype = "const uint8_t *".into();
                dispatcher_symbols.insert(symbol.to_owned());
            } else if dispatcher {
                dispatcher_symbols.insert(symbol.to_owned());
            }
            let mut declaration = args
                .iter()
                .zip(["a", "b"])
                .map(|(t, n)| {
                    Ok(format!(
                        "{} {n}",
                        if dispatcher && t == "rom_ptr" { "const uint8_t *" } else { family::c_type(t)? }
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(",");
            if declaration.is_empty() {
                declaration = "void".into();
            }
            let mut arguments = ["a", "b"][..args.len()].join(",");
            let mut actual_args = [
                if args.is_empty() { "0".to_owned() } else { "a".to_owned() },
                if args.len() < 2 { "0".to_owned() } else { "b".to_owned() },
            ];
            for (index, t) in args.iter().enumerate() {
                if t == "rom_ptr" {
                    let name = ["a", "b"][index];
                    actual_args[index] = if dispatcher {
                        format!("PortBank86PointerResult({name})")
                    } else {
                        format!("PortPointerResult({name},a)")
                    };
                }
            }
            let mut actual = actual_args.join(",");
            if routine.packed_u16_args.is_some() {
                ensure!(
                    args == ["uint32", "uint32"] && routine.packed_u16_args == Some(4),
                    "invalid packed four-word signature"
                );
                declaration = "uint16_t a,uint16_t b,uint16_t c,uint16_t d".into();
                arguments = "a,b,c,d".into();
                actual = "(uint32_t)a|((uint32_t)b<<16),(uint32_t)c|((uint32_t)d<<16)".into();
            }
            let call = format!("C_{symbol}({arguments})");
            let mut reference = if ret == "void" { format!("{call};") } else { format!("result=(uint32_t){call};") };
            let mut returned = if ret == "void" { String::new() } else { format!("return ({ctype})result;") };
            if ret == "rom_ptr" {
                reference = if dispatcher {
                    format!("{ctype}reference={call};result=PortBank86PointerResult(reference);")
                } else {
                    format!("const uint16_t *reference={call};result=PortPointerResult(reference,a);")
                };
                returned = if dispatcher {
                    "return (const uint8_t *)PortPointerFromResult((uint32_t)result);".into()
                } else {
                    "return PortPointerFromResult((uint32_t)result);".into()
                };
            }
            let index = symbols.len();
            symbols.push(symbol.to_owned());
            let mut wrapper = include_str!("live_wrapper.c.in").to_owned();
            for (key, value) in [
                ("__CTYPE__", ctype),
                ("__SYMBOL__", symbol.into()),
                ("__DECLARATION__", declaration),
                ("__INDEX__", index.to_string()),
                ("__ABI__", abi_symbol.clone()),
                ("__ROUTINE__", routine_index.to_string()),
                ("__ACTUAL__", actual),
                ("__REFERENCE__", reference),
                ("__ARG_A__", actual_args[0].clone()),
                ("__ARG_B__", actual_args[1].clone()),
                ("__RETURNED__", returned),
            ] {
                wrapper = wrapper.replace(key, &value);
            }
            wrappers.push_str(&wrapper);
        }
    }
    if positioned {
        let bank = source.join("src/sm_84.c");
        let mut text = transformed.get(&bank).unwrap_or(&originals[&bank]).clone();
        for symbol in ["SpawnRoomPLM", "SpawnHardcodedPlm"] {
            text = rename(&text, symbol, &format!("PortC_{symbol}"), false)?;
        }
        transformed.insert(bank, text);
    }
    let bank = source.join("src/sm_80.c");
    let text = transformed.get(&bank).unwrap_or(&originals[&bank]);
    transformed.insert(bank.clone(), rename(text, "NextRandom", "C_NextRandom", false)?);
    let mut transformations = BTreeMap::new();
    for (bank, text) in transformed {
        let generated = run.join(bank.file_name().unwrap());
        fs::write(&generated, text)?;
        let index = units.iter().position(|p| *p == bank).unwrap();
        units[index] = generated.clone();
        transformations.insert(
            bank.file_name().unwrap().to_string_lossy().into_owned(),
            Transformation { original_sha256: digest(&bank)?, transformed_sha256: digest(&generated)? },
        );
    }
    let mut host_dependencies = BTreeMap::new();
    for name in ["headless.c", "headless_frame.h"] {
        let source = checkout.join("tools/decomp").join(name);
        fs::copy(&source, run.join(name))?;
        host_dependencies.insert(name.into(), digest(&source)?);
    }
    wrappers.insert_str(
        0,
        &format!(
            "static unsigned long port_c_hits[{}],port_rust_hits[{}],port_outer_hits[{}];\n",
            symbols.len(),
            symbols.len(),
            symbols.len()
        ),
    );
    let host=include_str!("live_host.c.in").replace("__HEADLESS__",&format!("{:?}",run.join("headless.c"))).replace("__EFFECT_WRITER__",include_str!("effect_writer.c.in")).replace("__WRAPPERS__",&wrappers).replace("__POSITIONED_HOST__",if positioned{include_str!("live_positioned.c.in")}else{""}).replace("__NATIVE_PLM__",if plm.is_some(){"#define PORT_NATIVE_PLM\nextern int32_t port_native_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);"}else{""});
    fs::write(run.join("host.c"), host)?;
    fs::write(run.join("families.rs"), rust)?;
    Ok(Generated {
        source: source_identity,
        symbols,
        dispatcher_symbols,
        transformations,
        host_dependencies,
        units,
        predecessors,
        room_plm_installed: positioned,
    })
}

pub(crate) fn progress_row(line: &str) -> Result<Progress> {
    let values = line.split_whitespace().skip(1).map(str::parse::<u64>).collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(values.len() == 9, "progress record extent");
    Ok(Progress {
        frame: values[0],
        game_state: values[1],
        room: values[2],
        samus_x: values[3],
        samus_y: values[4],
        rng: values[5],
        fallback_calls: values[6],
        fallback_instructions: values[7],
        decompression_calls: values[8],
    })
}
fn counters(directory: &Path, symbols: &[String], mode: &str) -> Result<Counters> {
    let text = fs::read_to_string(directory.join("replay.stdout"))?;
    let mut hits = BTreeMap::new();
    let mut progress = Vec::new();
    let mut predecessor = None;
    let mut callbacks = None;
    let mut room = BTreeMap::new();
    for line in text.lines() {
        let words = line.split_whitespace().collect::<Vec<_>>();
        match words.first().copied() {
            Some("progress") => {
                progress.push(progress_row(line)?);
            }
            Some("port-hit") => {
                ensure!(words.len() == 4, "invalid routine hit extent");
                let index = words[1].parse::<usize>()?;
                let symbol = symbols.get(index).context("invalid routine hit index")?;
                let c = words[2].parse::<u64>()?;
                let rust = words[3].parse::<u64>()?;
                ensure!(
                    (mode == "c" && rust == 0) || (mode == "mixed" && c == 0),
                    "replacement execution was not exclusive"
                );
                ensure!(hits.insert(symbol.clone(), Hit { c, rust }).is_none(), "duplicate routine hit counter");
            }
            Some("predecessor-hit") => {
                ensure!(words.len() == 5 && predecessor.is_none(), "missing or duplicate predecessor counters");
                let dc = words[1].parse::<u64>()?;
                let dr = words[2].parse::<u64>()?;
                let rc = words[3].parse::<u64>()?;
                let rr = words[4].parse::<u64>()?;
                ensure!(
                    (mode == "c" && dr == 0 && rr == 0 && dc > 0 && rc > 0)
                        || (mode == "mixed" && dc == 0 && rc == 0 && dr > 0 && rr > 0),
                    "prior Rust replacements were not exclusively exercised"
                );
                predecessor = Some(PriorHit { decompressor_c: dc, decompressor_rust: dr, rng_c: rc, rng_rust: rr });
            }
            Some("plm-callback-hit") => {
                ensure!(words.len() == 5 && callbacks.is_none(), "missing or duplicate PLM counters");
                let total = words[1].parse::<u64>()?;
                let c = words[2].parse::<u64>()?;
                let rust = words[3].parse::<u64>()?;
                let setups = words[4].parse::<u64>()?;
                ensure!(
                    total == c + rust && setups <= rust && (mode != "c" || rust == 0),
                    "invalid PLM callback ownership counts"
                );
                callbacks = Some(Ownership { total, c, rust, rust_setups: setups });
            }
            Some("room-plm-hit") => {
                ensure!(words.len() == 6, "invalid positioned counter extent");
                let kind = words[1].parse::<u64>()?;
                let total = words[2].parse::<u64>()?;
                let c = words[3].parse::<u64>()?;
                let rust = words[4].parse::<u64>()?;
                let setups = words[5].parse::<u64>()?;
                ensure!(
                    kind < 2 && total == c + rust && setups <= rust && (mode != "c" || rust == 0),
                    "invalid positioned ownership"
                );
                ensure!(
                    room.insert(kind.to_string(), Ownership { total, c, rust, rust_setups: setups }).is_none(),
                    "duplicate positioned counter"
                );
            }
            _ => {}
        }
    }
    ensure!(
        hits.len() == symbols.len() && !progress.is_empty() && room.len() == 2,
        "missing execution counters or progress"
    );
    Ok(Counters {
        hits,
        progress,
        predecessor_hits: predecessor.context("missing predecessor counters")?,
        plm_callback_hits: callbacks.context("missing PLM counters")?,
        room_plm_hits: room,
    })
}

pub fn verify(request: Request) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let output = &request.output;
    fs::create_dir_all(output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut verdict = Verdict {
        status: "rejected".into(),
        run: run.clone(),
        original_rom_parity: "not-tested".into(),
        ..Default::default()
    };
    let result = (|| -> Result<()> {
        let frames = request.frames;
        let jobs = request.live_jobs;
        ensure!((1..=129712).contains(&frames) && (1..=4).contains(&jobs), "invalid frame extent or live jobs");
        let root = &request.root;
        let workers = request.workers;
        let rom = checkout.join("rom/sm.sfc");
        let movie = checkout.join("movies/super-metroid-any-3653.inputs");
        ensure!(digest(&rom)? == ROM_SHA256, "original ROM identity differs");
        let accepted = &request.accepted;
        crate::decomp_acceptance::authenticate(crate::decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.clone(),
        })?;
        let accepted_hash = digest(accepted)?;
        let mut generate = request.clone();
        generate.run = run.clone();
        let generated = self::generate(generate)?;
        verdict.source = Some(generated.source.clone());
        verdict.predecessors = generated.predecessors.clone();
        verdict.decompressor_predecessor_sha256 = accepted_hash.clone();
        verdict.transformations = generated.transformations.clone();
        verdict.host_dependencies = generated.host_dependencies.clone();
        verdict.dispatcher_symbols = generated.dispatcher_symbols.clone();
        verdict.room_plm_installed = generated.room_plm_installed;
        verdict.rom_sha256 = digest(&rom)?;
        verdict.inputs_sha256 = digest(&movie)?;
        verdict.frames = frames;
        verdict.hardware_storage = "packed".into();
        let cli = run.join("sm-evidence");
        verdict.evidence_cache = Some(builds::cargo_build(builds::CargoBuildRequest {
            checkout: checkout.clone(),
            root: root.clone(),
            workers,
            run: run.clone(),
            package: "evidence".into(),
            member: "sm-evidence".into(),
            destination: cli.clone(),
        })?);
        verdict.evidence_reader_sha256 = digest(&cli)?;
        builds::execute(
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "-O".into(),
                "--crate-type=staticlib".into(),
                format!("--emit=link,dep-info={}", run.join("compiler.d").display()),
                run.join("families.rs").to_string_lossy().into_owned(),
                "-o".into(),
                run.join("libfamilies.a").to_string_lossy().into_owned(),
            ],
            &run,
            "rust-build",
            root,
            workers,
        )?;
        verdict.rust_dependencies = inputs::dependencies(inputs::DependenciesRequest {
            checkout: checkout.clone(),
            run: run.clone(),
            extra: vec![
                "crates/pipeline/src/live.rs",
                "crates/pipeline/src/live_host.c.in",
                "crates/pipeline/src/live_family.rs.in",
                "crates/pipeline/src/live_wrapper.c.in",
                "crates/pipeline/src/live_plm.rs.in",
                "crates/pipeline/src/live_positioned.c.in",
                "crates/pipeline/src/effect_writer.c.in",
                "crates/pipeline/src/callback.rs",
                "crates/pipeline/src/callback_abi.rs.in",
                "crates/pipeline/src/packed.rs",
                "crates/pipeline/src/observations.rs",
            ]
            .into_iter()
            .map(PathBuf::from)
            .collect(),
        })?;
        let decomp = run.join("libdecomp.a");
        verdict.decompressor_cache = Some(builds::cargo_build(builds::CargoBuildRequest {
            checkout: checkout.clone(),
            root: root.clone(),
            workers,
            run: run.clone(),
            package: "decomp".into(),
            member: "libdecomp.a".into(),
            destination: decomp.clone(),
        })?);
        crate::decomp_acceptance::authenticate(crate::decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.clone(),
        })?;
        let rng = checkout.join("tools/decomp/next_random.rs");
        let rng_hash = digest(&rng)?;
        builds::execute(
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "-O".into(),
                "--crate-type=staticlib".into(),
                rng.to_string_lossy().into_owned(),
                "-o".into(),
                run.join("librng.a").to_string_lossy().into_owned(),
            ],
            &run,
            "rng-build",
            root,
            workers,
        )?;
        builds::execute(
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "-O".into(),
                "--cfg".into(),
                "cli".into(),
                rng.to_string_lossy().into_owned(),
                "-o".into(),
                run.join("rng-cli").to_string_lossy().into_owned(),
            ],
            &run,
            "rng-cli-build",
            root,
            workers,
        )?;
        builds::execute(
            &[run.join("rng-cli").to_string_lossy().into_owned(), "exhaustive".into()],
            &run,
            "rng-exhaustive",
            root,
            workers,
        )?;
        let rng_predecessor =
            request.rng_predecessor.clone().context("RNG predecessor required; run decomp-next-random first")?;
        let rng_evidence = crate::rng_job::authenticate(crate::rng_job::AuthenticateRequest {
            checkout: checkout.clone(),
            verdict: rng_predecessor.clone(),
        })?;
        ensure!(
            rng_evidence.status == "accepted"
                && rng_evidence.implementation["tools/decomp/next_random.rs"] == rng_hash
                && rng_evidence.synthetic.as_ref().context("missing synthetic proof")?.table_sha256
                    == digest(&run.join("rng-exhaustive.stdout"))?,
            "RNG exhaustive behavior or source differs from predecessor"
        );
        verdict.rng_source_sha256 = rng_hash;
        verdict.rng_predecessor_sha256 = digest(&rng_predecessor)?;
        let mut command = vec![
            "cc".into(),
            "-O2".into(),
            "-fno-strict-aliasing".into(),
            "-ffunction-sections".into(),
            "-fdata-sections".into(),
            "-iquote".into(),
            checkout.join("deps/snesrev-sm/src").to_string_lossy().into_owned(),
        ];
        command.extend(generated.units.iter().map(|v| v.to_string_lossy().into_owned()));
        command.extend([
            run.join("host.c").to_string_lossy().into_owned(),
            run.join("libfamilies.a").to_string_lossy().into_owned(),
            decomp.to_string_lossy().into_owned(),
            run.join("librng.a").to_string_lossy().into_owned(),
            "-Wl,--gc-sections".into(),
            "-Wl,--wrap=Call".into(),
            "-Wl,--wrap=cpu_runOpcode".into(),
            "-Wl,--wrap=DecompressToMem".into(),
            "-Wl,--wrap=ppu_write".into(),
            "-Wl,--wrap=RtlApuWrite".into(),
            "-Wl,--wrap=RtlRunFrame".into(),
            "-Wl,--wrap=SpawnPLM".into(),
            "-lm".into(),
            "-ldl".into(),
            "-lpthread".into(),
            "-o".into(),
            run.join("live").to_string_lossy().into_owned(),
        ]);
        verdict.host_cache = Some(builds::c_build(builds::CBuildRequest {
            checkout: checkout.clone(),
            root: root.clone(),
            workers,
            run: run.clone(),
            command,
        })?);
        let binary_hash = digest(&run.join("live"))?;
        verdict.binary_sha256 = binary_hash.clone();
        let requests = SUBJECTS
            .iter()
            .map(|name| -> Result<packed::CaptureRequest> {
                let directory = run.join(name);
                fs::create_dir(&directory)?;
                Ok(packed::CaptureRequest {
                    directory: directory.clone(),
                    reader: cli.clone(),
                    root: root.clone(),
                    workers,
                    command: vec![
                        run.join("live").to_string_lossy().into_owned(),
                        name.split('-').next().unwrap().into(),
                        rom.to_string_lossy().into_owned(),
                        movie.to_string_lossy().into_owned(),
                        frames.to_string(),
                        directory.join(".evidence-stage/terminal.bin").to_string_lossy().into_owned(),
                    ],
                    provenance: evidence::Provenance {
                        schema_version: 2,
                        subject: Some((*name).into()),
                        binary_sha256: Some(binary_hash.clone()),
                        reader_sha256: Some(verdict.evidence_reader_sha256.clone()),
                        rom_sha256: Some(verdict.rom_sha256.clone()),
                        inputs_sha256: Some(verdict.inputs_sha256.clone()),
                        frames: Some(frames),
                        source: Some(evidence::SourceIdentity {
                            revision: generated.source.revision.clone(),
                            archive_sha256: generated.source.archive_sha256.clone(),
                            files: generated.source.files.clone(),
                            freshness: generated.source.freshness.clone(),
                        }),
                        ..Default::default()
                    },
                    timeout: 240.0f64.max(frames as f64 / 40.0),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let next = AtomicUsize::new(0);
        let outcomes = Mutex::new(Vec::new());
        let started = Instant::now();
        std::thread::scope(|scope| {
            for _ in 0..jobs {
                scope.spawn(|| {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(request) = requests.get(index) else {
                            break;
                        };
                        outcomes.lock().unwrap().push((index, packed::capture(request.clone())));
                    }
                });
            }
        });
        let mut outcomes = outcomes.into_inner().unwrap();
        outcomes.sort_by_key(|(index, _)| *index);
        let mut pointers = BTreeMap::new();
        let mut failures = Vec::new();
        for (index, result) in outcomes {
            match result {
                Ok(pointer) => {
                    pointers.insert(SUBJECTS[index].to_owned(), pointer);
                }
                Err(error) => failures.push(format!("{}: {error:#}", SUBJECTS[index])),
            }
        }
        ensure!(failures.is_empty(), "{}", failures.join("; "));
        verdict.capture_seconds = started.elapsed().as_secs_f64();
        let mut inventories = BTreeMap::new();
        for name in SUBJECTS {
            let inventory = pointers[name]
                .artifacts
                .iter()
                .filter(|(name, _)| {
                    Path::new(name).file_name().is_none_or(|n| n != "calls.bin")
                        && Path::new(name).extension().is_none_or(|n| n != "rom")
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<BTreeMap<_, _>>();
            inventories.insert(name.to_owned(), inventory);
        }
        ensure!(
            inventories.values().all(|v| v == &inventories["c-0"]),
            "independent C/mixed state or effects diverged"
        );
        let symbols = &generated.symbols;
        let mut counts = BTreeMap::new();
        for name in SUBJECTS {
            counts.insert(name.to_owned(), counters(&run.join(name), symbols, name.split('-').next().unwrap())?);
        }
        ensure!(
            counts.values().all(|v| v.progress == counts["c-0"].progress),
            "gameplay progress or measured fallback diverged"
        );
        ensure!(
            counts.values().all(|v| v.plm_callback_hits.total == counts["c-0"].plm_callback_hits.total),
            "PLM callback counts diverged"
        );
        ensure!(
            counts["mixed-0"].plm_callback_hits == counts["mixed-1"].plm_callback_hits,
            "native PLM ownership differs across replays"
        );
        for symbol in &generated.dispatcher_symbols {
            let actual = SUBJECTS.map(|name| {
                if name.starts_with("c-") { counts[name].hits[symbol].c } else { counts[name].hits[symbol].rust }
            });
            ensure!(
                actual[0] > 0 && actual.iter().all(|v| *v == actual[0]),
                "dispatcher leaf lacks matching nonzero exclusive execution: {symbol}"
            );
        }
        ensure!(counts["mixed-0"].hits.values().any(|v| v.rust > 0), "no Rust gameplay family executed");
        if generated.room_plm_installed {
            ensure!(
                counts["mixed-0"].room_plm_hits == counts["mixed-1"].room_plm_hits
                    && counts["mixed-0"].room_plm_hits.values().any(|v| v.rust_setups > 0),
                "positioned ownership differs or no native setup executed"
            );
        } else if request.plm_verdict.is_some() {
            ensure!(counts["mixed-0"].plm_callback_hits.rust_setups > 0, "no native Rust PLM setup executed");
        }
        verdict.state_effects_match = true;
        verdict.artifacts = inventories["c-0"].clone();
        verdict.counters = counts.clone();
        if !request.retain_private_stores {
            let store = request.evidence_store.clone().unwrap_or_else(|| checkout.join("out/evidence"));
            for name in SUBJECTS {
                pointers.insert(
                    name.into(),
                    packed::publish(packed::PublishRequest {
                        directory: run.join(name),
                        root: root.clone(),
                        workers,
                        store: store.clone(),
                    })?,
                );
            }
        }
        let mut subjects = BTreeMap::new();
        for name in SUBJECTS {
            subjects.insert(
                name.into(),
                StorageSubject {
                    run: pointers[name].run.clone(),
                    observations: pointers[name].observations.clone(),
                    pointer_sha256: digest(&run.join(name).join("evidence.toml"))?,
                },
            );
        }
        verdict.evidence_storage = Storage { format: "sm-evidence-v2".into(), subjects };
        verdict.storage_metrics = pointers;
        crate::decomp_acceptance::authenticate(crate::decomp_acceptance::AuthenticateRequest {
            checkout: checkout.clone(),
            pointer: accepted.clone(),
        })?;
        ensure!(digest(accepted)? == accepted_hash, "decompressor predecessor changed");
        crate::rng_job::authenticate(crate::rng_job::AuthenticateRequest {
            checkout: checkout.clone(),
            verdict: rng_predecessor.clone(),
        })?;
        ensure!(digest(&rng_predecessor)? == verdict.rng_predecessor_sha256, "RNG predecessor changed during replay");
        for predecessor in &generated.predecessors {
            let path = &predecessor.path;
            family::authenticate(AuthenticateRequest { checkout: checkout.clone(), verdict: path.clone() })?;
            ensure!(digest(path)? == predecessor.sha256, "family predecessor changed during replay");
        }
        for (name, hash) in &verdict.rust_dependencies {
            ensure!(
                digest(&family::source_path(&checkout, name))? == *hash,
                "live candidate source changed during replay"
            );
        }
        verdict.status = "mixed-C-Rust-integration".into();
        verdict.live_exercised = true;
        verdict.remaining_cpu_fallback = counts["mixed-0"].progress.last().unwrap().fallback_calls;
        verdict.scope="four fresh processes; periodic hardware/game state, full terminal WRAM/SRAM, ordered PPU/APU effects and captures; original-ROM parity separate".into();
        Ok(())
    })();
    if let Err(error) = result {
        verdict.error = Some(format!("{error:#}"));
    }
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    if verdict.status == "mixed-C-Rust-integration" {
        observations::authenticate(observations::AuthenticateRequest {
            verdict: run.join("verdict.toml"),
            root: request.root.clone(),
            workers: request.workers,
        })?;
    }
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dispatcher_contract_rejects_unowned_bank_signature_and_ineligible_only_matrix() {
        let root = Path::new("/checkout");
        let module = root.join("crates/port/src/enemy_projectile_instructions.rs");
        let leaf = crate::family_model::Routine {
            symbol: "EprojInstr_Delete".into(),
            arg_types: Some(vec!["uint16".into(), "rom_ptr".into()]),
            returns: "rom_ptr".into(),
            pointer_bank: Some(0x86),
            ..Default::default()
        };
        let manifest = Manifest {
            bank: "sm_86.c".into(),
            routines: vec![
                crate::family_model::Routine { symbol: "EprojInstruction".into(), ..Default::default() },
                leaf,
            ],
            ..Default::default()
        };
        let mut valid = manifest.clone();
        dispatcher_contract(&mut valid, &module, root).unwrap();
        assert_eq!(valid.routines[0].live_eligible, Some(false));
        for mode in 0..4 {
            let mut wrong = manifest.clone();
            match mode {
                0 => wrong.bank = "sm_84.c".into(),
                1 => wrong.routines[1].returns = "int32".into(),
                2 => wrong.routines[1].pointer_bank = Some(0xb2),
                _ => wrong.routines[1].live_eligible = Some(false),
            };
            assert!(dispatcher_contract(&mut wrong, &module, root).is_err());
        }
    }
    #[test]
    fn counter_exclusivity_rejects_double_execution_and_unexecuted_predecessors() {
        let directory = std::env::temp_dir().join(format!("sm-counter-controls-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let valid = "progress 1 0 0 0 0 1 0 0 1\nport-hit 0 1 0\npredecessor-hit 1 0 1 0\nplm-callback-hit 0 0 0 0\nroom-plm-hit 0 0 0 0 0\nroom-plm-hit 1 0 0 0 0\n";
        fs::write(directory.join("replay.stdout"), valid).unwrap();
        assert!(counters(&directory, &["Touch".into()], "c").is_ok());
        for wrong in [
            valid.replace("port-hit 0 1 0", "port-hit 0 1 1"),
            valid.replace("predecessor-hit 1 0 1 0", "predecessor-hit 0 0 0 0"),
            valid.replace("port-hit 0 1 0", "port-hit 0 1 0\nport-hit 0 1 0"),
        ] {
            fs::write(directory.join("replay.stdout"), wrong).unwrap();
            assert!(counters(&directory, &["Touch".into()], "c").is_err());
        }
        fs::remove_dir_all(directory).unwrap();
    }
}
