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
    time::{Instant, SystemTime, UNIX_EPOCH},
};
const SUBJECTS: [&str; 4] = ["c-0", "c-1", "mixed-0", "mixed-1"];
const PROJECTILE_ENTRIES: [&str; 3] = ["SpawnEprojWithRoomGfx", "EprojRunAll", "SpawnEprojWithGfx"];
const PROJECTILE_IDS: [u16; 9] = [0x9642, 0x9650, 0x965e, 0x966c, 0x967a, 0x9688, 0x9696, 0xa387, 0xa395];
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const RIDLEY_OPCODES: [u32; 8] = [0xa6e517, 0xa6e4d2, 0xa6e4be, 0xa6e4ca, 0xa6e84d, 0xa6e904, 0xa6e909, 0xa6812f];

fn bounded_module(module: &Path, checkout: &Path) -> Option<&'static str> {
    ["projectile_trails", "camera_scroll", "room_scroll", "samus_projectile_frame", "ridley_enemy_instructions"]
        .into_iter()
        .find(|name| module == checkout.join(format!("crates/port/src/{name}.rs")))
}
fn bounded_root(module: &str, symbol: &str) -> bool {
    match module {
        "projectile_trails" => symbol == "HandleProjectileTrails",
        "camera_scroll" => symbol == "MainScrollingRoutine",
        "room_scroll" => matches!(symbol, "UpdateScrollVarsUpdateMap" | "CalculateLayer2PosAndScrollsWhenScrolling"),
        "samus_projectile_frame" => matches!(
            symbol,
            "HandleProjectile" | "DrawPlayerExplosions2" | "DrawBombAndProjectileExplosions"
        ),
        "ridley_enemy_instructions" => symbol == "ProcessEnemyInstructions",
        _ => false,
    }
}
fn bounded_contract(manifest: &Manifest, module: &str) -> Result<()> {
    ensure!(manifest.rom.is_some(), "bounded frame requires native ROM evidence");
    ensure!(manifest.projectile_host == (module == "ridley_enemy_instructions"), "bounded frame host contract differs");
    let mut roots = BTreeSet::new();
    for routine in &manifest.routines {
        if !bounded_root(module, &routine.symbol) {
            continue;
        }
        ensure!(
            family::types(routine)?.is_empty() && routine.returns == "void",
            "bounded frame native signature differs"
        );
        ensure!(routine.live_eligible != Some(false), "bounded native frame marked offline-only");
        ensure!(roots.insert(routine.symbol.as_str()), "duplicate bounded native frame");
    }
    ensure!(!roots.is_empty(), "no verified bounded native frame");
    let expected = match module {
        "projectile_trails" => vec!["HandleProjectileTrails"],
        "camera_scroll" => vec!["MainScrollingRoutine"],
        "room_scroll" => vec!["UpdateScrollVarsUpdateMap", "CalculateLayer2PosAndScrollsWhenScrolling"],
        "ridley_enemy_instructions" => vec!["ProcessEnemyInstructions"],
        "samus_projectile_frame" => {
            vec!["HandleProjectile", "DrawPlayerExplosions2", "DrawBombAndProjectileExplosions"]
        }
        _ => anyhow::bail!("unknown bounded module"),
    };
    ensure!(roots == expected.into_iter().collect(), "bounded frame predecessor does not cover complete native roots");
    Ok(())
}

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

fn projectile_contract(manifest: &Manifest, module: &Path, checkout: &Path) -> Result<()> {
    ensure!(
        module == checkout.join("crates/port/src/observed_projectile_ownership.rs")
            && manifest.bank == "sm_86.c"
            && manifest.projectile_host
            && manifest.rom.is_some(),
        "projectile evidence requires installed host-aware ownership module and bank86 ROM"
    );
    let mut entries = BTreeSet::new();
    for routine in &manifest.routines {
        if routine.symbol == "EprojInstruction" {
            ensure!(routine.live_eligible == Some(false), "projectile synthetic instruction must be offline-only");
            continue;
        }
        if routine.live_eligible == Some(false) {
            continue;
        }
        let args: Vec<String> = match routine.symbol.as_str() {
            "SpawnEprojWithRoomGfx" => vec!["uint16".into(), "uint16".into()],
            "EprojRunAll" => vec![],
            "SpawnEprojWithGfx" => vec!["uint16".into(), "uint32".into()],
            _ => anyhow::bail!("unsupported native projectile entry"),
        };
        ensure!(
            entries.insert(routine.symbol.as_str()) && family::types(routine)? == args && routine.returns == "void",
            "projectile native signature differs"
        );
        ensure!(
            if routine.symbol == "SpawnEprojWithGfx" {
                routine.packed_args.as_deref() == Some("eproj-with-gfx-v1")
            } else {
                routine.packed_args.is_none()
            },
            "projectile packed marker differs"
        );
    }
    ensure!(
        entries == PROJECTILE_ENTRIES.into_iter().collect(),
        "projectile allocator and scheduler contracts required"
    );
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
                let implementation = evidence.implementation.as_ref().context("missing implementation")?;
                ensure!(
                    implementation.dependency_contract == "rust-family-v2"
                        || (bounded_module(&module, &checkout) == Some("samus_projectile_frame")
                            && implementation.dependency_contract == "rust-callback-v2"
                            && implementation.callback_adapter.as_deref() == Some("power-bomb")
                            && manifest.plm_amount_context.as_deref() == Some("none")
                            && evidence.gameplay_translation == Some(true)),
                    "family evidence must seal the admitted family adapter"
                );
            }
            if let Some(bounded) = bounded_module(&module, &checkout) {
                ensure!(kind == "family", "bounded frame requires ordinary family evidence");
                bounded_contract(&manifest, bounded)?;
            }
            families.push((manifest, module, kind));
            predecessors.push(Predecessor {
                path: verdict_path.clone(),
                sha256: digest(verdict_path)?,
                kind: kind.into(),
            });
        }
    }
    let projectile = request.projectile_verdict.as_ref();
    if let Some(path) = projectile {
        let evidence = family::authenticate(AuthenticateRequest { checkout: checkout.clone(), verdict: path.clone() })?;
        ensure!(evidence.source.as_ref() == Some(&source_identity), "projectile source identity differs");
        let manifest: Manifest =
            document(&path.parent().context("projectile predecessor parent missing")?.join("manifest.toml"))?;
        let module = fs::canonicalize(family::source_path(&checkout, &manifest.rust_module))?;
        projectile_contract(&manifest, &module, &checkout)?;
        predecessors.push(Predecessor { path: path.clone(), sha256: digest(path)?, kind: "projectile".into() });
    }
    ensure!(!families.is_empty() || projectile.is_some(), "no family contracts");
    let ridley =
        families.iter().any(|(_, module, _)| bounded_module(module, &checkout) == Some("ridley_enemy_instructions"));
    ensure!(!ridley || projectile.is_some(), "Ridley attack frame requires native projectile predecessor");
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
    let mut imports = families.iter().flat_map(|(m, _, _)| m.rust_imports.clone()).collect::<BTreeSet<_>>();
    let mut modules = families.iter().map(|(_, module, _)| module.clone()).collect::<Vec<_>>();
    if plm.is_some() {
        modules.push(checkout.join("crates/port/src/plm.rs"));
    }
    if projectile.is_some() {
        modules.push(checkout.join("crates/port/src/observed_projectile_ownership.rs"));
        imports.extend(["enemy_projectiles", "observed_projectile_ownership"].map(str::to_owned));
    }
    if ridley {
        imports.insert("ridley_enemy_instructions".into());
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
    if projectile.is_some() {
        rust.push_str(include_str!("live_projectile.rs.in"));
    }
    if ridley {
        rust.push_str(include_str!("live_ridley.rs.in"));
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
    let mut bounded_symbols = BTreeSet::new();
    for (family_index, (manifest, module, kind)) in families.iter().enumerate() {
        let bounded = bounded_module(module, &checkout);
        if bounded == Some("ridley_enemy_instructions") {
            bounded_symbols.insert("ProcessEnemyInstructions".into());
            continue;
        }
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
            let support = match bounded {
                Some("projectile_trails" | "samus_projectile_frame") => format!(
                    "family{family_index}::supports_call(&Wram::new(bytes).unwrap(),rom,name,a,b).unwrap_or(false)"
                ),
                Some("camera_scroll") => format!("family{family_index}::supports(&Wram::new(bytes).unwrap(),name)"),
                Some("room_scroll") => format!("family{family_index}::supports(&Wram::new(bytes).unwrap(),name,a)"),
                _ => String::new(),
            };
            rust.push_str(
                &if bounded.is_some() {
                    include_str!("live_bounded_family.rs.in")
                } else {
                    include_str!("live_family.rs.in")
                }
                .replace("__MODULE__", &format!("{module:?}"))
                .replace("__INDEX__", &family_index.to_string())
                .replace("__NAMES__", &format!("{names:?}"))
                .replace("__CALL__", &call)
                .replace("__SUPPORT__", &support),
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
            if bounded.is_some_and(|module| !bounded_root(module, symbol)) {
                continue;
            }
            ensure!(!manifest.projectile_host, "projectile host requires explicit projectile verdict ABI");
            ensure!(
                projectile.is_none() || !PROJECTILE_ENTRIES.contains(&symbol),
                "native projectile entries require only projectile verdict"
            );
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
            let mut wrapper = if bounded.is_some() {
                include_str!("live_bounded_wrapper.c.in")
            } else {
                include_str!("live_wrapper.c.in")
            }
            .to_owned();
            if let Some(module) = bounded {
                bounded_symbols.insert(symbol.into());
                let active = match module {
                    "projectile_trails" => "PortTrailsActive()",
                    "camera_scroll" => {
                        "GET_WORD(g_ram+0xaf6)!=GET_WORD(g_ram+0xb10)||GET_WORD(g_ram+0xafa)!=GET_WORD(g_ram+0xb14)||GET_WORD(g_ram+0xaf8)!=GET_WORD(g_ram+0xb12)||GET_WORD(g_ram+0xafc)!=GET_WORD(g_ram+0xb16)"
                    }
                    "room_scroll" => "!GET_WORD(g_ram+0xa78)",
                    "samus_projectile_frame" => "PortSamusProjectilesActive()",
                    _ => unreachable!(),
                };
                wrapper = wrapper.replace("__ACTIVE__", active);
            }
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
            if request.capture_active_eprojs {
                wrapper = wrapper.replace(" if(outer && capture<=8)", &format!(" unsigned active_capture=outer?PortActiveSequence({index}):0;\n if(active_capture)PortActiveCapture({index},active_capture,\"before\");\n if(outer && capture<=8)"))
                    .replace("  if(capture<=8)", &format!("  if(active_capture)PortActiveCapture({index},active_capture,\"after\");\n  if(capture<=8)"));
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
    if projectile.is_some() {
        let bank = source.join("src/sm_86.c");
        let mut text = transformed.get(&bank).unwrap_or(&originals[&bank]).clone();
        for symbol in PROJECTILE_ENTRIES {
            text = rename(&text, symbol, &format!("PortC_{symbol}"), false)?;
        }
        text.push_str(include_str!("live_projectile_bank.c.in"));
        transformed.insert(bank, text);
    }
    if ridley {
        let bank = source.join("src/sm_a0.c");
        let mut text = transformed.get(&bank).unwrap_or(&originals[&bank]).clone();
        for symbol in ["ProcessEnemyInstructions", "CallEnemyInstr"] {
            text = rename(&text, symbol, &format!("PortC_{symbol}"), false)?;
        }
        text.push_str(include_str!("live_ridley_bank.c.in"));
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
            symbols.len().max(1),
            symbols.len().max(1),
            symbols.len().max(1)
        ),
    );
    if !bounded_symbols.is_empty() {
        let ordinary_bounded = symbols
            .iter()
            .map(|symbol| if bounded_symbols.contains(symbol) { format!("\"{symbol}\"") } else { "NULL".into() })
            .collect::<Vec<_>>()
            .join(",");
        wrappers.insert_str(0, &format!("static unsigned long port_frame_refused[{size}],port_frame_active_c[{size}],port_frame_active_rust[{size}];\nstatic unsigned port_frame_captures[{size}];\nstatic const char *port_bounded[{size}]={{{flags}}};\nstatic int PortTrailsActive(void){{for(unsigned k=0;k<36;k+=2)if(GET_WORD(g_ram+0xd658+k)||GET_WORD(g_ram+0xd67c+k))return 1;return 0;}}\nstatic int PortSamusProjectilesActive(void){{for(unsigned k=0;k<20;k+=2)if(GET_WORD(g_ram+0xc40+k))return 1;return 0;}}\nstatic void PortFrameCapture(const char *routine,unsigned sequence,const char *phase){{char name[160];snprintf(name,sizeof(name),\"fixtures/frame-%s-%03u-%s.wram\",routine,sequence,phase);FILE *f=fopen(name,\"wb\");if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))exit(2);}}\n", size=symbols.len().max(1), flags=if ordinary_bounded.is_empty(){"NULL"}else{&ordinary_bounded}));
    }
    if request.capture_active_eprojs {
        wrappers = format!(
            "static unsigned port_active_captures[{}];\nstatic unsigned PortActiveSequence(unsigned routine) {{ if(port_active_captures[routine]>=8)return 0; for(unsigned offset=0x1997;offset<0x1997+36;offset+=2)if(g_ram[offset] || g_ram[offset+1])return ++port_active_captures[routine];return 0;}}\nstatic void PortActiveCapture(unsigned routine,unsigned sequence,const char *phase) {{char name[120];snprintf(name,sizeof(name),\"fixtures/active-port-%03u-%03u-%s.wram\",routine,sequence,phase);FILE *f=fopen(name,\"wb\");if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))exit(2);}}\n{wrappers}",
            symbols.len().max(1)
        );
    }
    let mut host=include_str!("live_host.c.in").replace("__HEADLESS__",&format!("{:?}",run.join("headless.c"))).replace("__EFFECT_WRITER__",include_str!("effect_writer.c.in")).replace("__WRAPPERS__",&wrappers).replace("__POSITIONED_HOST__",if positioned{include_str!("live_positioned.c.in")}else{""}).replace("__NATIVE_PLM__",if plm.is_some(){"#define PORT_NATIVE_PLM\nextern int32_t port_native_plm(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);"}else{""});
    if projectile.is_some() {
        host = host.replace("static unsigned port_frame;", &format!("{}\nstatic unsigned port_frame;", include_str!("live_projectile_host.c.in")))
            .replace("  if(fclose(f))exit(2);", "  if(fclose(f))exit(2);\n  snprintf(name,sizeof(name),\"frame-%06u.host\",port_frame);PortProjectileState(name);")
            .replace(" if(PortEffectsClose())", " PortProjectileState(\"projectile-terminal.host\");PortProjectileCounters();\n if(PortEffectsClose())");
    }
    if ridley {
        host = host
            .replace(
                "static unsigned port_frame;",
                &format!("{}\nstatic unsigned port_frame;", include_str!("live_ridley_host.c.in")),
            )
            .replace(" if(PortEffectsClose())", " PortRidleyCounters();\n if(PortEffectsClose())");
    }
    if !bounded_symbols.is_empty() {
        host = host.replace(" printf(\"predecessor-hit", " for(unsigned i=0;i<sizeof(port_c_hits)/sizeof(*port_c_hits);i++)if(port_bounded[i])printf(\"frame-hit %s %lu %lu %lu %lu %lu %lu\\n\",port_bounded[i],port_c_hits[i]+port_rust_hits[i],port_c_hits[i],port_rust_hits[i],port_frame_refused[i],port_frame_active_c[i],port_frame_active_rust[i]);\n printf(\"predecessor-hit");
    }
    if symbols.is_empty() {
        host = host.replace("i<sizeof(port_c_hits)/sizeof(*port_c_hits)", "i<0");
    }
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
        projectile_native_installed: projectile.is_some(),
        bounded_symbols,
        ridley_native_installed: ridley,
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
pub(crate) fn counters(directory: &Path, symbols: &[String], mode: &str) -> Result<Counters> {
    let text = fs::read_to_string(directory.join("replay.stdout"))?;
    let mut hits = BTreeMap::new();
    let mut progress = Vec::new();
    let mut predecessor = None;
    let mut callbacks = None;
    let mut room = BTreeMap::new();
    let mut projectile_hits = BTreeMap::new();
    let mut projectile_id_hits = BTreeMap::new();
    let mut projectile_allocation_hits = BTreeMap::new();
    let mut frame_hits = BTreeMap::new();
    let mut ridley_opcode_hits = BTreeMap::new();
    for line in text.lines() {
        let words = line.split_whitespace().collect::<Vec<_>>();
        match words.first().copied() {
            Some("frame-hit") => {
                ensure!(words.len() == 8, "invalid bounded frame counter extent");
                let symbol = if words[1].bytes().any(|byte| !byte.is_ascii_digit()) {
                    ensure!(
                        words[1] == "ProcessEnemyInstructions" || symbols.iter().any(|symbol| symbol == words[1]),
                        "unknown bounded frame symbol"
                    );
                    words[1].to_owned()
                } else {
                    symbols.get(words[1].parse::<usize>()?).context("invalid frame symbol index")?.clone()
                };
                let hit = FrameHit {
                    total: words[2].parse()?,
                    c: words[3].parse()?,
                    rust: words[4].parse()?,
                    refused: words[5].parse()?,
                    active_c: words[6].parse()?,
                    active_rust: words[7].parse()?,
                };
                ensure!(
                    hit.total == hit.c + hit.rust
                        && hit.refused == if mode == "mixed" { hit.c } else { 0 }
                        && hit.active_c <= hit.c
                        && hit.active_rust <= hit.rust
                        && (mode != "c" || hit.rust == 0),
                    "invalid bounded frame ownership counts"
                );
                ensure!(frame_hits.insert(symbol, hit).is_none(), "duplicate bounded frame counter");
            }
            Some("ridley-opcode-hit") => {
                ensure!(words.len() == 4, "invalid Ridley opcode counter extent");
                let opcode = words[1].parse::<u32>()?;
                ensure!(RIDLEY_OPCODES.contains(&opcode), "unknown Ridley opcode counter");
                let hit = Hit { c: words[2].parse()?, rust: words[3].parse()? };
                ensure!(
                    (mode != "c" || hit.rust == 0) && (mode != "mixed" || hit.c == 0),
                    "Ridley opcode execution not exclusive"
                );
                ensure!(
                    ridley_opcode_hits.insert(opcode.to_string(), hit).is_none(),
                    "duplicate Ridley opcode counter"
                );
            }
            Some("projectile-hit") => {
                ensure!(words.len() == 10, "invalid projectile counter extent");
                let v = words[1..].iter().map(|n| n.parse::<u64>()).collect::<std::result::Result<Vec<_>, _>>()?;
                let hit = ProjectileHit {
                    total: v[1],
                    c: v[2],
                    rust: v[3],
                    refused: v[4],
                    active_c: v[5],
                    active_rust: v[6],
                    combat_c: v[7],
                    combat_rust: v[8],
                };
                ensure!(
                    v[0] < 3
                        && hit.total == hit.c + hit.rust
                        && hit.refused == if mode == "mixed" { hit.c } else { 0 }
                        && hit.combat_c <= hit.active_c
                        && hit.active_c <= hit.c
                        && hit.combat_rust <= hit.active_rust
                        && hit.active_rust <= hit.rust
                        && (mode != "c" || hit.rust == 0),
                    "invalid projectile ownership counts"
                );
                ensure!(projectile_hits.insert(v[0].to_string(), hit).is_none(), "duplicate projectile counter");
            }
            Some("projectile-id-hit") => {
                ensure!(words.len() == 5, "invalid projectile ID counter extent");
                let id = words[1].parse::<u16>()?;
                let hit = ProjectileIdHit { c: words[2].parse()?, rust: words[3].parse()?, refused: words[4].parse()? };
                ensure!(
                    PROJECTILE_IDS.contains(&id)
                        && hit.refused == if mode == "mixed" { hit.c } else { 0 }
                        && (mode != "c" || hit.rust == 0),
                    "invalid projectile ID ownership counts"
                );
                ensure!(projectile_id_hits.insert(id.to_string(), hit).is_none(), "duplicate projectile ID counter");
            }
            Some("projectile-allocation-hit") => {
                ensure!(words.len() == 6, "invalid projectile allocation counter extent");
                let id = words[1].parse::<u16>()?;
                let hit = ProjectileAllocationHit {
                    total: words[2].parse()?,
                    c: words[3].parse()?,
                    rust: words[4].parse()?,
                    refused: words[5].parse()?,
                };
                ensure!(
                    PROJECTILE_IDS.contains(&id)
                        && hit.total == hit.c + hit.rust
                        && hit.refused == if mode == "mixed" { hit.c } else { 0 }
                        && (mode != "c" || hit.rust == 0),
                    "invalid projectile allocation ownership counts"
                );
                ensure!(
                    projectile_allocation_hits.insert(id.to_string(), hit).is_none(),
                    "duplicate projectile allocation counter"
                );
            }
            Some("progress") => {
                progress.push(progress_row(line)?);
            }
            Some("port-hit") => {
                ensure!(words.len() == 4, "invalid routine hit extent");
                let index = words[1].parse::<usize>()?;
                let symbol = symbols.get(index).context("invalid routine hit index")?;
                let c = words[2].parse::<u64>()?;
                let rust = words[3].parse::<u64>()?;
                ensure!(mode != "c" || rust == 0, "C baseline executed Rust family");
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
    for (symbol, hit) in &hits {
        if let Some(frame) = frame_hits.get(symbol) {
            ensure!(frame.c == hit.c && frame.rust == hit.rust, "frame and routine counters differ");
        } else {
            ensure!(mode != "mixed" || hit.c == 0, "replacement execution was not exclusive");
        }
    }
    Ok(Counters {
        hits,
        progress,
        predecessor_hits: predecessor.context("missing predecessor counters")?,
        plm_callback_hits: callbacks.context("missing PLM counters")?,
        room_plm_hits: room,
        projectile_hits,
        projectile_id_hits,
        projectile_allocation_hits,
        frame_hits,
        ridley_opcode_hits,
    })
}

pub(crate) fn projectile_admission(
    counts: &BTreeMap<String, Counters>,
    artifacts: &BTreeMap<String, String>,
    frames: u64,
) -> Result<()> {
    ensure!(SUBJECTS.iter().all(|name| counts.contains_key(*name)), "missing projectile subjects");
    let baseline = &counts["c-0"];
    let mixed = &counts["mixed-0"];
    for names in [["c-0", "c-1"], ["mixed-0", "mixed-1"]] {
        let a = &counts[names[0]];
        let b = &counts[names[1]];
        ensure!(
            a.projectile_hits == b.projectile_hits
                && a.projectile_id_hits == b.projectile_id_hits
                && a.projectile_allocation_hits == b.projectile_allocation_hits,
            "projectile ownership differs across independent replays"
        );
    }
    ensure!(
        mixed.progress.last().context("missing projectile progress")?.fallback_calls == 0,
        "native projectile admission requires zero CPU fallback"
    );
    let material = |prefix: &str| -> Result<()> {
        for phase in ["before", "after"] {
            for suffix in ["wram", "host"] {
                ensure!(
                    artifacts.contains_key(&format!("fixtures/{prefix}-001-{phase}.{suffix}")),
                    "missing material projectile WRAM/eight-word host capture"
                );
            }
        }
        Ok(())
    };
    for kind in 0..3 {
        let key = kind.to_string();
        let c = baseline.projectile_hits.get(&key).context("missing native projectile ownership counters")?;
        let r = mixed.projectile_hits.get(&key).context("missing native projectile ownership counters")?;
        ensure!(
            c.total == r.total && c.active_c == r.active_c + r.active_rust && c.combat_c == r.combat_c + r.combat_rust,
            "native projectile traversal counts diverged"
        );
        ensure!(
            r.total > 0 && r.rust == r.total && r.c == 0 && r.refused == 0 && r.active_rust > 0,
            "native projectile requires exclusive nonempty Rust execution and zero refusals"
        );
        if kind < 2 {
            ensure!(r.combat_rust > 0 && r.combat_c == 0, "native projectile lacks exclusive nonempty 9642 execution");
            material(&format!("projectile-9642-{}", kind + 1))?;
        }
    }
    ensure!(
        baseline.projectile_id_hits.len() == 9
            && mixed.projectile_id_hits.len() == 9
            && baseline.projectile_allocation_hits.len() == 9
            && mixed.projectile_allocation_hits.len() == 9,
        "projectile per-ID counters incomplete"
    );
    for id in PROJECTILE_IDS {
        let key = id.to_string();
        let c = baseline.projectile_id_hits.get(&key).context("missing projectile ID")?;
        let r = mixed.projectile_id_hits.get(&key).context("missing projectile ID")?;
        ensure!(c.c == r.c + r.rust, "projectile scheduler traversal counts diverged");
        if frames == 129712 || c.c > 0 || [0x9642, 0x9650, 0x966c, 0x967a, 0xa387, 0xa395].contains(&id) {
            ensure!(r.rust > 0 && r.c == 0, "observed projectile ID lacks exclusive Rust scheduler execution");
            material(&format!("projectile-id-{id:04x}"))?;
        }
        let c = baseline.projectile_allocation_hits.get(&key).context("missing projectile allocation ID")?;
        let r = mixed.projectile_allocation_hits.get(&key).context("missing projectile allocation ID")?;
        ensure!(c.total == r.total, "projectile allocation traversal counts diverged");
        if c.total > 0 {
            ensure!(r.c == 0 && r.rust > 0, "observed allocation lacks exclusive Rust execution");
            material(&format!("projectile-allocation-{id:04x}"))?;
        }
        if [0x9642, 0xa387, 0xa395].contains(&id) {
            ensure!(r.rust > 0, "native allocation lacks Ridley/Ceres exercise");
            if frames == 129712 {
                ensure!(r.rust == if id == 0x9642 { 16 } else { 4 }, "full-route native allocation count differs");
            }
        }
    }
    Ok(())
}

pub(crate) fn frame_admission(
    counts: &BTreeMap<String, Counters>,
    artifacts: &BTreeMap<String, String>,
    symbols: &BTreeSet<String>,
    frames: u64,
) -> Result<()> {
    ensure!(SUBJECTS.iter().all(|name| counts.contains_key(*name)), "missing frame subjects");
    let baseline = &counts["c-0"];
    let mixed = &counts["mixed-0"];
    for names in [["c-0", "c-1"], ["mixed-0", "mixed-1"]] {
        ensure!(
            counts[names[0]].frame_hits == counts[names[1]].frame_hits
                && counts[names[0]].ridley_opcode_hits == counts[names[1]].ridley_opcode_hits,
            "bounded ownership differs across independent replays"
        );
    }
    ensure!(
        mixed.progress.last().context("missing frame progress")?.fallback_calls == 0,
        "bounded frame requires zero CPU fallback"
    );
    for symbol in symbols {
        let c = baseline.frame_hits.get(symbol).context("missing bounded frame counters")?;
        let r = mixed.frame_hits.get(symbol).context("missing bounded frame counters")?;
        ensure!(
            c.total == r.total && c.active_c == r.active_c + r.active_rust,
            "bounded native traversal counts diverged"
        );
        ensure!(r.rust > 0 && r.active_rust > 0, "bounded native frame lacks material Rust execution: {symbol}");
        if symbol == "ProcessEnemyInstructions" {
            let captured = artifacts
                .keys()
                .filter(|name| name.starts_with("fixtures/ridley-frame-") && name.ends_with("-after.host"))
                .collect::<Vec<_>>();
            ensure!(!captured.is_empty(), "missing Ridley full-WRAM/host captures");
            for name in captured {
                let prefix = name.strip_suffix("-after.host").unwrap();
                for phase in ["before", "after"] {
                    for suffix in ["wram", "host"] {
                        ensure!(
                            artifacts.contains_key(&format!("{prefix}-{phase}.{suffix}")),
                            "incomplete Ridley WRAM/eight-word capture pair"
                        );
                    }
                }
            }
            ensure!(
                baseline.ridley_opcode_hits.len() == 8 && mixed.ridley_opcode_hits.len() == 8,
                "Ridley opcode counters incomplete"
            );
            for (opcode, expected) in RIDLEY_OPCODES.into_iter().zip([2, 4, 4, 2, 18, 4, 12, 2]) {
                let key = opcode.to_string();
                let a = &baseline.ridley_opcode_hits[&key];
                let b = &mixed.ridley_opcode_hits[&key];
                ensure!(a.c == b.rust && a.rust == 0 && b.c == 0, "Ridley opcode counts diverged");
                if frames == 129712 {
                    ensure!(b.rust == expected, "full-route Ridley opcode counts differ");
                }
            }
            if frames == 129712 {
                ensure!(r.rust == 2393 && r.active_rust == 36, "full-route supported Ridley frame counts differ");
            }
        } else {
            for phase in ["before", "after"] {
                ensure!(
                    artifacts.contains_key(&format!("fixtures/frame-{symbol}-001-{phase}.wram")),
                    "missing active bounded frame capture: {symbol}"
                );
            }
        }
    }
    Ok(())
}

pub fn verify(request: Request) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let output = &request.output;
    fs::create_dir_all(output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut metrics = crate::metrics::Run::start(run.join("pipeline.metrics.toml"), "live")?;
    metrics.context.configuration("frames", request.frames);
    metrics.context.configuration("live-jobs", request.live_jobs);
    metrics.context.configuration("workers", request.workers);
    metrics.context.configuration("worker-root", request.root.display());
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
        metrics.context.configuration("source-revision", &generated.source.revision);
        metrics.context.configuration("source-archive-sha256", &generated.source.archive_sha256);
        verdict.source = Some(generated.source.clone());
        verdict.predecessors = generated.predecessors.clone();
        verdict.decompressor_predecessor_sha256 = accepted_hash.clone();
        verdict.transformations = generated.transformations.clone();
        verdict.host_dependencies = generated.host_dependencies.clone();
        verdict.dispatcher_symbols = generated.dispatcher_symbols.clone();
        verdict.routine_symbols = generated.symbols.clone();
        verdict.room_plm_installed = generated.room_plm_installed;
        verdict.projectile_native_installed = generated.projectile_native_installed;
        verdict.bounded_symbols = generated.bounded_symbols.clone();
        verdict.ridley_native_installed = generated.ridley_native_installed;
        verdict.active_eproj_captures = request.capture_active_eprojs;
        if generated.projectile_native_installed {
            verdict.projectile_abi = Some("synchronous-eight-word-host-v2-with-gfx".into());
        }
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
                "crates/pipeline/src/live_projectile.rs.in",
                "crates/pipeline/src/live_projectile_host.c.in",
                "crates/pipeline/src/live_projectile_bank.c.in",
                "crates/pipeline/src/live_bounded_family.rs.in",
                "crates/pipeline/src/live_bounded_wrapper.c.in",
                "crates/pipeline/src/live_ridley.rs.in",
                "crates/pipeline/src/live_ridley_host.c.in",
                "crates/pipeline/src/live_ridley_bank.c.in",
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
        let started = Instant::now();
        let captured = packed::capture_parallel(&requests, jobs)?;
        let mut pointers = SUBJECTS
            .iter()
            .zip(captured)
            .map(|(name, pointer)| ((*name).to_owned(), pointer))
            .collect::<BTreeMap<_, _>>();
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
        ensure!(
            counts["mixed-0"].hits.values().any(|v| v.rust > 0)
                || generated.projectile_native_installed
                || generated.ridley_native_installed,
            "no Rust gameplay family executed"
        );
        if generated.projectile_native_installed {
            projectile_admission(&counts, &inventories["c-0"], frames)?;
        }
        if !generated.bounded_symbols.is_empty() {
            frame_admission(&counts, &inventories["c-0"], &generated.bounded_symbols, frames)?;
        }
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
    let publication = metrics.context.stage("final-verdict-publication")?;
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    publication.finish("success")?;
    if verdict.status == "mixed-C-Rust-integration" {
        let auth = metrics.context.stage("final-authentication")?;
        observations::authenticate(observations::AuthenticateRequest {
            verdict: run.join("verdict.toml"),
            root: request.root.clone(),
            workers: request.workers,
        })?;
        auth.finish("success")?;
    }
    let cleanup = metrics.context.stage("scoped-cleanup")?;
    crate::cleanup::cleanup(crate::cleanup::CleanupRequest { root: run.clone(), apply: true, deduplicate: false })?;
    crate::cleanup::cleanup(crate::cleanup::CleanupRequest { root: run.clone(), apply: true, deduplicate: true })?;
    cleanup.finish("success")?;
    if verdict.status == "mixed-C-Rust-integration" {
        let auth = metrics.context.stage("post-cleanup-authentication")?;
        observations::authenticate(observations::AuthenticateRequest {
            verdict: run.join("verdict.toml"),
            root: request.root.clone(),
            workers: request.workers,
        })?;
        auth.finish("success")?;
    }
    metrics.finish(crate::metrics::outcome(verdict.error.as_deref()), verdict.error.clone())?;
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_contract_installs_whole_verified_roots_and_rejects_wrong_host_or_signature() {
        let mut manifest = Manifest {
            rom: Some(Default::default()),
            routines: vec![
                crate::family_model::Routine {
                    symbol: "MainScrollingRoutine".into(),
                    returns: "void".into(),
                    ..Default::default()
                },
                crate::family_model::Routine {
                    symbol: "Samus_HandleScroll_X".into(),
                    args: 1,
                    returns: "void".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        bounded_contract(&manifest, "camera_scroll").unwrap();
        assert!(!bounded_root("camera_scroll", "Samus_HandleScroll_X"));
        manifest.projectile_host = true;
        assert!(bounded_contract(&manifest, "camera_scroll").is_err());
        manifest.projectile_host = false;
        manifest.routines[0].args = 1;
        assert!(bounded_contract(&manifest, "camera_scroll").is_err());
        manifest.routines[0].args = 0;
        manifest.routines[0].live_eligible = Some(false);
        assert!(bounded_contract(&manifest, "camera_scroll").is_err());
    }
    #[test]
    fn bounded_counter_admission_keeps_explicit_c_scope_and_rejects_missing_material() {
        let symbol = "ProcessEnemyInstructions".to_owned();
        let mut baseline = Counters { progress: vec![Progress::default()], ..Default::default() };
        let mut mixed = baseline.clone();
        baseline
            .frame_hits
            .insert(symbol.clone(), FrameHit { total: 10000, c: 10000, active_c: 36, ..Default::default() });
        mixed.frame_hits.insert(
            symbol.clone(),
            FrameHit { total: 10000, c: 7607, rust: 2393, refused: 7607, active_rust: 36, ..Default::default() },
        );
        for (opcode, amount) in RIDLEY_OPCODES.into_iter().zip([2, 4, 4, 2, 18, 4, 12, 2]) {
            baseline.ridley_opcode_hits.insert(opcode.to_string(), Hit { c: amount, rust: 0 });
            mixed.ridley_opcode_hits.insert(opcode.to_string(), Hit { c: 0, rust: amount });
        }
        let counts = BTreeMap::from([
            ("c-0".into(), baseline.clone()),
            ("c-1".into(), baseline),
            ("mixed-0".into(), mixed.clone()),
            ("mixed-1".into(), mixed),
        ]);
        let artifacts = ["before", "after"]
            .into_iter()
            .flat_map(|phase| {
                ["wram", "host"].into_iter().map(move |suffix| {
                    (format!("fixtures/ridley-frame-e752-window0-001-{phase}.{suffix}"), "sealed".into())
                })
            })
            .collect();
        let roots = BTreeSet::from([symbol]);
        frame_admission(&counts, &artifacts, &roots, 129712).unwrap();
        assert!(frame_admission(&counts, &BTreeMap::new(), &roots, 129712).is_err());
        let mut wrong = counts.clone();
        wrong.get_mut("mixed-1").unwrap().frame_hits.get_mut("ProcessEnemyInstructions").unwrap().rust -= 1;
        assert!(frame_admission(&wrong, &artifacts, &roots, 129712).is_err());
    }
    #[test]
    fn native_projectile_contract_requires_owned_host_and_packed_gfx_signature() {
        let root = Path::new("/checkout");
        let module = root.join("crates/port/src/observed_projectile_ownership.rs");
        let mut manifest = Manifest {
            bank: "sm_86.c".into(),
            projectile_host: true,
            rom: Some(Default::default()),
            routines: vec![
                crate::family_model::Routine {
                    symbol: "SpawnEprojWithRoomGfx".into(),
                    args: 2,
                    returns: "void".into(),
                    ..Default::default()
                },
                crate::family_model::Routine {
                    symbol: "EprojRunAll".into(),
                    returns: "void".into(),
                    ..Default::default()
                },
                crate::family_model::Routine {
                    symbol: "SpawnEprojWithGfx".into(),
                    arg_types: Some(vec!["uint16".into(), "uint32".into()]),
                    returns: "void".into(),
                    packed_args: Some("eproj-with-gfx-v1".into()),
                    ..Default::default()
                },
                crate::family_model::Routine {
                    symbol: "EprojInstruction".into(),
                    live_eligible: Some(false),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        projectile_contract(&manifest, &module, root).unwrap();
        manifest.routines[2].packed_args = None;
        assert!(projectile_contract(&manifest, &module, root).is_err());
        manifest.routines[2].packed_args = Some("eproj-with-gfx-v1".into());
        manifest.routines[3].live_eligible = None;
        assert!(projectile_contract(&manifest, &module, root).is_err());
    }
    #[test]
    fn projectile_admission_rejects_refusal_missing_material_and_wrong_full_route_allocations() {
        let mut baseline = Counters { progress: vec![Progress::default()], ..Default::default() };
        let mut mixed = baseline.clone();
        let mut artifacts = BTreeMap::new();
        let mut material = |prefix: String| {
            for phase in ["before", "after"] {
                for suffix in ["host", "wram"] {
                    artifacts.insert(format!("fixtures/{prefix}-001-{phase}.{suffix}"), "sealed".into());
                }
            }
        };
        for kind in 0..3 {
            baseline.projectile_hits.insert(
                kind.to_string(),
                ProjectileHit { total: 3, c: 3, active_c: 2, combat_c: 1, ..Default::default() },
            );
            mixed.projectile_hits.insert(
                kind.to_string(),
                ProjectileHit { total: 3, rust: 3, active_rust: 2, combat_rust: 1, ..Default::default() },
            );
            if kind < 2 {
                material(format!("projectile-9642-{}", kind + 1));
            }
        }
        for id in PROJECTILE_IDS {
            baseline.projectile_id_hits.insert(id.to_string(), ProjectileIdHit { c: 3, ..Default::default() });
            mixed.projectile_id_hits.insert(id.to_string(), ProjectileIdHit { rust: 3, ..Default::default() });
            material(format!("projectile-id-{id:04x}"));
            let total = match id {
                0x9642 => 16,
                0xa387 | 0xa395 => 4,
                _ => 0,
            };
            baseline
                .projectile_allocation_hits
                .insert(id.to_string(), ProjectileAllocationHit { total, c: total, ..Default::default() });
            mixed
                .projectile_allocation_hits
                .insert(id.to_string(), ProjectileAllocationHit { total, rust: total, ..Default::default() });
            if total > 0 {
                material(format!("projectile-allocation-{id:04x}"));
            }
        }
        let counts = BTreeMap::from([
            ("c-0".into(), baseline.clone()),
            ("c-1".into(), baseline),
            ("mixed-0".into(), mixed.clone()),
            ("mixed-1".into(), mixed),
        ]);
        projectile_admission(&counts, &artifacts, 129712).unwrap();
        assert!(projectile_admission(&counts, &BTreeMap::new(), 129712).is_err());
        for control in 0..3 {
            let mut wrong = counts.clone();
            for name in ["mixed-0", "mixed-1"] {
                let count = wrong.get_mut(name).unwrap();
                match control {
                    0 => count.progress[0].fallback_calls = 1,
                    1 => count.projectile_hits.get_mut("0").unwrap().refused = 1,
                    _ => count.projectile_allocation_hits.get_mut(&0x9642_u16.to_string()).unwrap().rust = 15,
                }
            }
            assert!(projectile_admission(&wrong, &artifacts, 129712).is_err());
        }
        let serialized = toml::to_string(&counts).unwrap();
        let _: BTreeMap<String, Counters> = toml::from_str(&serialized).unwrap();
    }
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
