//! Native offline family verification. Acceptance is limited to declared fixtures.
use crate::family_model::*;
use crate::{atomic_toml, builds, callback, inputs, metrics};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tool_execution::bounded::{self, digest};

const WRAM: usize = 0x20000;
use decomp_source::{c_units, verify_source};
use tool_execution::{bounded::object, unique_id};

fn fork_execution(request: &VerifyRequest) -> Result<bool> {
    ensure!(
        ["", "auto", "fresh", "fork"].contains(&request.execution.as_str()),
        "execution must be auto, fresh or fork"
    );
    let ordinary = !["callback", "power-bomb", "plm"].contains(&request.adapter.as_str());
    let supported = cfg!(all(target_os = "linux", target_arch = "x86_64")) && ordinary;
    ensure!(
        request.execution != "fork" || supported,
        "fork execution is supported only for Linux x86_64 family and Ridley subsystem adapters"
    );
    Ok(supported && request.execution != "fresh")
}
pub(crate) fn identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
pub(crate) fn source_path(checkout: &Path, text: &str) -> PathBuf {
    let path = Path::new(text);
    if path.is_absolute() { path.to_path_buf() } else { checkout.join(path) }
}
fn safe_relative(text: &str) -> bool {
    !text.is_empty() && Path::new(text).components().all(|c| matches!(c, Component::Normal(_)))
}
fn continuation_directory(case: &Case, run: &Path) -> Result<PathBuf> {
    let name = case.continue_from.as_deref().context("continuation missing")?;
    ensure!(safe_relative(name) && !name.contains('/') && name != case.name, "invalid continuation case");
    Ok(run.join(name))
}
fn host_bytes(case: &Case, run: &Path) -> Result<Vec<u8>> {
    host_bytes_with(case, run, &mut crate::admission_reader::Window::unsealed(run))
}
fn host_bytes_with(case: &Case, run: &Path, reader: &mut crate::admission_reader::Window<'_, '_>) -> Result<Vec<u8>> {
    if case.continue_from.is_some() {
        ensure!(case.host_words.is_none(), "continuation must inherit all external host words");
        let previous = continuation_directory(case, run)?;
        let c = reader.read(previous.join("c-host-after.bin"))?;
        ensure!(c.len() == 16 && c == reader.read(previous.join("rust-host-after.bin"))?, "continuation host differs");
        return Ok(c.to_vec());
    }
    let words = case.host_words.as_ref().context("host words missing")?;
    ensure!(words.len() == 8, "projectile host requires eight words");
    Ok(words.iter().flat_map(|v| v.to_le_bytes()).collect())
}
fn authenticate_host(directory: &Path, fixture: &Case) -> Result<()> {
    let run = directory.parent().context("case parent missing")?;
    authenticate_host_with(directory, fixture, &mut crate::admission_reader::Window::unsealed(run))
}
fn authenticate_host_with(
    directory: &Path,
    fixture: &Case,
    reader: &mut crate::admission_reader::Window<'_, '_>,
) -> Result<()> {
    let before = reader.read(directory.join("host-before.bin"))?;
    let c = reader.read(directory.join("c-host-after.bin"))?;
    let rust = reader.read(directory.join("rust-host-after.bin"))?;
    ensure!(
        before.as_ref() == host_bytes_with(fixture, directory.parent().context("case parent missing")?, reader)?
            && c.len() == 16
            && c == rust,
        "projectile host state differs"
    );
    Ok(())
}

fn asset(path: &Path, expected: &str, extent: Option<usize>) -> Result<Vec<u8>> {
    let bytes = if let Some(view) = crate::input_view::current() {
        view.read(path, Some(expected))?
    } else {
        let (bytes, hash) = bounded::read_with_digest(path)?;
        ensure!(hash == expected, "asset identity mismatch: {}", path.display());
        bytes
    };
    if let Some(n) = extent {
        ensure!(bytes.len() == n, "asset extent mismatch");
    }
    Ok(bytes)
}
pub(crate) fn prepare_case_with(case: &Case, directory: &Path, arg_types: &[String], resume: bool) -> Result<()> {
    metrics::measure("fixture-input-authentication", || prepare_case_inputs(case, directory, arg_types, resume))
}

fn prepare_case_inputs(case: &Case, directory: &Path, arg_types: &[String], resume: bool) -> Result<()> {
    let name = &case.name;
    ensure!(
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "unsafe case name"
    );
    let mut bytes = if case.continue_from.is_some() {
        ensure!(case.before_file.is_none() && case.fill.is_none(), "continuation cannot replace inherited state");
        let previous = continuation_directory(case, directory.parent().context("case parent missing")?)?;
        let c = bounded::read(previous.join("c-after.wram"))?;
        ensure!(
            c.len() == WRAM && c == bounded::read(previous.join("rust-after.wram"))?,
            "continuation requires matching previous WRAM"
        );
        c
    } else if let Some(before) = &case.before_file {
        ensure!(case.writes.is_empty() && case.fill.is_none(), "captured before_file cannot be patched");
        asset(before, case.before_sha256.as_deref().context("before digest missing")?, Some(WRAM))?
    } else {
        vec![case.fill.unwrap_or(0); WRAM]
    };
    for write in &case.writes {
        ensure!(
            write.offset.checked_add(write.data.len()).is_some_and(|end| end <= WRAM),
            "case write outside canonical WRAM"
        );
        bytes[write.offset..write.offset + write.data.len()].copy_from_slice(&write.data);
    }
    for (index, value) in [case.a, case.b].into_iter().enumerate() {
        let kind = arg_types.get(index).map(String::as_str).unwrap_or("uint16");
        let (low, high) = match kind {
            "uint8" => (0, 255),
            "uint16" => (0, 65535),
            "uint32" => (0, 4294967295),
            "int32" => (-2147483648, 2147483647),
            "rom_ptr" => (0x8000, 0xffffff),
            _ => bail!("invalid argument kind"),
        };
        ensure!(
            (low..=high).contains(&value) && (kind != "rom_ptr" || value & 0xffff >= 0x8000),
            "argument outside {kind}"
        );
    }
    if !resume || !directory.exists() {
        fs::create_dir(directory)?;
    }
    ensure!(!directory.is_symlink(), "case directory alias");
    write_build_input(&directory.join("before.wram"), &bytes, resume)?;
    if let Some(path) = &case.expected_after_file {
        write_build_input(
            &directory.join("expected-after.wram"),
            &asset(path, case.expected_after_sha256.as_deref().context("expected digest missing")?, Some(WRAM))?,
            resume,
        )?;
    }
    write_build_input(&directory.join("case.sha256"), crate::fixture_format::case_identity(case)?.as_bytes(), resume)
}

pub(crate) fn write_build_input(path: &Path, bytes: &[u8], immutable: bool) -> Result<()> {
    if immutable { tool_execution::bounded::immutable_bytes(path, bytes) } else { Ok(fs::write(path, bytes)?) }
}

pub(crate) fn c_type(kind: &str) -> Result<&str> {
    Ok(match kind {
        "void" => "void",
        "uint8" => "uint8_t",
        "uint16" => "uint16_t",
        "uint32" => "uint32_t",
        "int32" => "int32_t",
        "rom_ptr" => "const uint16_t *",
        _ => bail!("invalid routine type"),
    })
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
    let fork = fork_execution(request)?;
    ensure!(execution.is_none() || !fork, "confined adapter requires fresh fixtures");
    let dispatcher = manifest.uses_dispatcher();
    let complete_c = matches!(recipe, Some(crate::recipe::Recipe::CompleteC));
    let eproj = manifest.projectile_host
        || manifest.projectile_rendering
        || manifest.routines.iter().any(|r| r.symbol == "EprojInstruction");
    let host = manifest.projectile_host;
    ensure!(
        !host
            || (eproj
                && manifest.rom.as_ref().is_some()
                && Path::new(&manifest.rust_module).file_name().is_some_and(|n| {
                    n == "enemy_projectiles.rs"
                        || n == "observed_projectile_ownership.rs"
                        || n == "ridley_enemy_instructions.rs"
                })),
        "projectile host requires ROM and an admitted host dispatcher"
    );
    let mut prototypes = Vec::new();
    let mut branches = Vec::new();
    for routine in &manifest.routines {
        let symbol = &routine.symbol;
        let kind = routine.returns.as_str();
        let args = routine.argument_types()?;
        ensure!(identifier(symbol) && args.len() <= 2, "invalid routine signature");
        let arguments = args.iter().map(|kind| c_type(kind)).collect::<Result<Vec<_>>>()?;
        prototypes.push(format!(
            "extern {} {symbol}({});",
            c_type(kind)?,
            if arguments.is_empty() { "void".to_owned() } else { arguments.join(", ") }
        ));
        let actual = args
            .iter()
            .zip(["a", "b"])
            .map(|(kind, arg)| {
                if kind == "rom_ptr" {
                    ensure!(manifest.rom.as_ref().is_some(), "ROM pointer argument requires ROM asset");
                    Ok(format!(
                        "(const uint16_t*)(g_rom+((((uint32_t){arg}>>16)<<15)|((uint32_t){arg}&0x7fff))%0x400000)"
                    ))
                } else {
                    Ok(format!("({}){arg}", c_type(kind)?))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let mut call = format!("{symbol}({})", actual.join(", "));
        if routine.packed_args.is_some() || symbol == "SpawnEprojWithGfx" {
            ensure!(
                routine.packed_args.as_deref() == Some("eproj-with-gfx-v1")
                    && symbol == "SpawnEprojWithGfx"
                    && args == ["uint16", "uint32"]
                    && kind == "void",
                "invalid native projectile packed signature"
            );
            *prototypes.last_mut().unwrap() = "extern void SpawnEprojWithGfx(uint16_t,uint16_t,uint16_t);".into();
            call = "SpawnEprojWithGfx((uint16_t)a,(uint16_t)b,(uint16_t)((uint32_t)b>>16))".into();
        }
        if let Some(words) = routine.packed_u16_args {
            match words {
                3 if args == ["uint16", "uint32"] => {
                    *prototypes.last_mut().unwrap() =
                        format!("extern {} {symbol}(uint16_t,uint16_t,uint16_t);", c_type(kind)?);
                    call = format!("{symbol}((uint16_t)a,(uint16_t)b,(uint16_t)((uint32_t)b>>16))");
                }
                3 if args == ["uint32", "uint16"] => {
                    *prototypes.last_mut().unwrap() =
                        format!("extern {} {symbol}(uint16_t,uint16_t,uint16_t);", c_type(kind)?);
                    call = format!("{symbol}((uint16_t)a,(uint16_t)((uint32_t)a>>16),(uint16_t)b)");
                }
                4 if args == ["uint32", "uint32"] => {
                    *prototypes.last_mut().unwrap() =
                        format!("extern {} {symbol}(uint16_t,uint16_t,uint16_t,uint16_t);", c_type(kind)?);
                    call = format!(
                        "{symbol}((uint16_t)a,(uint16_t)((uint32_t)a>>16),(uint16_t)b,(uint16_t)((uint32_t)b>>16))"
                    );
                }
                _ => anyhow::bail!("invalid packed three- or four-word signature"),
            }
        }
        let mut result = if kind == "void" { format!("{call}; result=0;") } else { format!("result={call};") };
        if routine.pair_u16_return {
            ensure!(kind == "uint32" && args == ["uint16", "uint16"], "invalid pair return signature");
            *prototypes.last_mut().unwrap() = format!("extern PairU16 {symbol}(uint16_t,uint16_t);");
            result = format!("PairU16 pair={call}; result=(uint32_t)pair.k|((uint32_t)pair.j<<16);");
        }
        if kind == "rom_ptr" {
            let bank = if dispatcher {
                let bank = routine.pointer_bank.unwrap_or(if eproj { 0x86 } else { 0x84 });
                ensure!([0x84, 0x86, 0xa6, 0xb2].contains(&bank), "invalid pointer bank");
                bank.to_string()
            } else {
                "(uint16_t)a".to_owned()
            };
            result = format!("const uint16_t *pointer={call}; result=PortPointerResult(pointer,{bank});");
        }
        branches.push(format!("if (!strcmp(argv[1], \"{symbol}\")) {{ {result} }}"));
    }
    let bank_name = &manifest.bank;
    let support = &manifest.c_support;
    let uses_rtl = bank_name == "sm_rtl.c" || support.iter().any(|v| v == "sm_rtl.c");
    let globals = if dispatcher || complete_c {
        "extern uint8_t g_ram[0x20000]; extern const uint8_t *g_rom;"
    } else if uses_rtl {
        "extern uint8_t g_ram[0x20000]; extern const uint8_t *g_rom; bool g_fail;"
    } else {
        "uint8_t g_ram[0x20000]; const uint8_t *g_rom;"
    };
    let rom_setup = if manifest.rom.as_ref().is_some() {
        "FILE *rf=fopen(argv[6],\"rb\"); if(!rf)return 2; if(fseek(rf,0,SEEK_END))return 2; long n=ftell(rf); rewind(rf); uint8_t *rom=malloc(n); if(!rom||fread(rom,1,n,rf)!=(size_t)n||fclose(rf))return 2; g_rom=rom;"
    } else {
        ""
    };
    let mut oracle = format!(
        "{}\n{globals}\n{}\n{}\nint main(int argc,char **argv){{\n if(argc!=6 && argc!=7)return 2;\n int64_t a=strtoll(argv[2],0,10),b=strtoll(argv[3],0,10); uint32_t result=0;\n {rom_setup}\n FILE *f=fopen(argv[4],\"rb\"); if(!f||fread(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fgetc(f)!=EOF||fclose(f))return 2;\n {} else return 2;\n f=fopen(argv[5],\"wb\"); if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))return 2;\n printf(\"%u\\n\",result); return 0;\n}}\n",
        include_str!("family_header.c"),
        if dispatcher { include_str!("dispatcher_pointer.c") } else { include_str!("family_pointer.c") },
        prototypes.join("\n"),
        branches.join(" else ")
    );
    if host {
        oracle = oracle.replace("int main(","void PortHostLoad(uint16_t*); void PortHostSave(uint16_t*);\nint main(")
            .replace("if(argc!=6 && argc!=7)","if(argc!=9)")
            .replace(" int64_t a=", " uint16_t host[8]; FILE *hf=fopen(argv[7],\"rb\"); if(!hf||fread(host,2,8,hf)!=8||fgetc(hf)!=EOF||fclose(hf))return 2; PortHostLoad(host);\n int64_t a=")
            .replace(" printf(", " PortHostSave(host);hf=fopen(argv[8],\"wb\");if(!hf||fwrite(host,2,8,hf)!=8||fclose(hf))return 2;\n printf(");
    }
    if fork {
        oracle = oracle.replace("int main(", "int PortFixtureInvoke(");
        oracle = oracle.replace(rom_setup, "");
        oracle.push_str("\nextern int sm_fixture_worker(int (*invoke)(int,char**));\nint main(int argc,char **argv){if(argc==3&&!strcmp(argv[1],\"--fixture-worker\")){\n");
        if manifest.rom.is_some() {
            oracle.push_str("FILE *rf=fopen(argv[2],\"rb\");if(!rf||fseek(rf,0,SEEK_END))return 2;long n=ftell(rf);if(n<=0||n>0x400000)return 2;rewind(rf);uint8_t *rom=malloc(n);if(!rom||fread(rom,1,n,rf)!=(size_t)n||fclose(rf))return 2;g_rom=rom;\n");
        }
        oracle.push_str("return sm_fixture_worker(PortFixtureInvoke);}\n");
        oracle.push_str(rom_setup);
        oracle.push_str("return PortFixtureInvoke(argc,argv);}\n");
    }
    write_build_input(&run.join("oracle.c"), oracle.as_bytes(), execution.is_some())?;
    let module = fs::canonicalize(source_path(checkout, &manifest.rust_module))?;
    let imports = &manifest.rust_imports;
    for name in imports {
        ensure!(identifier(name), "invalid Rust module import");
    }
    let alias_module = dispatcher
        && module.file_name().is_some_and(|n| {
            n == "enemy_projectiles.rs"
                || n == "observed_projectile_ownership.rs"
                || n == "ridley_enemy_instructions.rs"
        });
    let mut wrapper_imports = imports.clone();
    if alias_module {
        let name = module.file_stem().unwrap().to_string_lossy().into_owned();
        if !wrapper_imports.contains(&name) {
            wrapper_imports.push(name);
        }
    }
    let memory = if let Some(execution) = execution {
        inputs::canonical_wrapper(checkout, run, &execution.rust_inputs)?
    } else {
        inputs::wrapper(crate::authoring::WrapperRequest {
            checkout: checkout.into(),
            run: run.into(),
            modules: vec![module.clone()],
            imports: wrapper_imports,
        })?
    };
    let signed = manifest.routines.iter().any(|r| r.arg_types.is_some());
    let (setup, call) = if manifest.rom.as_ref().is_some() {
        (
            "let rom=std::fs::read(&args[6]).unwrap();",
            "family::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(),&rom,&args[1],args[2].parse::<i64>().unwrap() as i32,args[3].parse::<i64>().unwrap() as i32)",
        )
    } else if signed {
        (
            "",
            "family::dispatch_i32(&mut Wram::new(&mut bytes).unwrap(),&args[1],args[2].parse::<i64>().unwrap() as i32,args[3].parse::<i64>().unwrap() as i32)",
        )
    } else {
        (
            "",
            "family::dispatch(&mut Wram::new(&mut bytes).unwrap(),&args[1],args[2].parse().unwrap(),args[3].parse().unwrap())",
        )
    };
    let root_imports = imports.iter().map(|v| format!("use memory::{v};\n")).collect::<String>();
    let mut wrapper = format!(
        "#[path={:?}] mod memory;\nuse memory::*;\n{root_imports}#[path={module:?}] mod family;\nfn main(){{\n let args:Vec<String>=std::env::args().collect(); assert!(args.len()==6 || args.len()==7);\n let mut bytes=std::fs::read(&args[4]).unwrap(); {setup}\n let result={call}.unwrap();\n std::fs::write(&args[5],bytes).unwrap(); println!(\"{{}}\",result);\n}}\n",
        memory.path
    );
    if execution.is_some() {
        wrapper = wrapper.replace(
            &format!(
                "#[path={:?}] mod memory;\nuse memory::*;\n{root_imports}#[path={module:?}] mod family;",
                memory.path
            ),
            &format!(
                "include!(\"/inputs/generated/memory.rs\");\nuse {} as family;",
                module.file_stem().unwrap().to_string_lossy()
            ),
        );
    } else if alias_module {
        wrapper = wrapper.replace(
            &format!("#[path={module:?}] mod family;"),
            &format!("use memory::{} as family;", module.file_stem().unwrap().to_string_lossy()),
        );
    }
    if host {
        wrapper = wrapper.replace("assert!(args.len()==6 || args.len()==7)", "assert_eq!(args.len(),9)")
            .replace("let result=family::dispatch_rom(","let raw=std::fs::read(&args[7]).unwrap();assert_eq!(raw.len(),16);let mut words=[0;8];for (i,w) in words.iter_mut().enumerate(){*w=u16::from_le_bytes([raw[i*2],raw[i*2+1]]);}let mut host=family::ProjectileHost::from_words(words);\n let result=family::dispatch_host(")
            .replace("&rom,&args[1]", "&rom,&mut host,&args[1]")
            .replace("family::ProjectileHost::from_words", "memory::enemy_projectiles::ProjectileHost::from_words")
            .replace(" std::fs::write(&args[5]", " let out:Vec<u8>=host.words().into_iter().flat_map(u16::to_le_bytes).collect();std::fs::write(&args[8],out).unwrap();\n std::fs::write(&args[5]");
    }
    if fork {
        wrapper = wrapper.replace(
            "fn main(){\n let args:Vec<String>=std::env::args().collect();",
            "fn invoke(args:&[String],rom:&[u8]){\n",
        );
        wrapper = wrapper.replace("let rom=std::fs::read(&args[6]).unwrap();", "");
        wrapper.push_str(&format!("\n#[path={:?}] mod fixture_worker;\nfn main(){{let args:Vec<String>=std::env::args().collect();let worker=args.get(1).is_some_and(|a|a==\"--fixture-worker\");let rom=if {}{{std::fs::read(if worker{{&args[2]}}else{{&args[6]}}).unwrap()}}else{{vec![]}};if worker{{fixture_worker::serve(|args|{{invoke(args,&rom);0}}).unwrap();}}else{{invoke(&args,&rom);}}}}\n",checkout.join("crates/offline-verification/src/fixture_worker.rs"),manifest.rom.is_some()));
        fs::write(run.join("fixture-worker.rs"), include_str!("fixture_worker.rs"))?;
        let worker_source = checkout.join("crates/offline-verification/src/fixture_worker.rs");
        let worker_hash = digest(&run.join("fixture-worker.rs"))?;
        ensure!(digest(&worker_source)? == worker_hash, "fixture worker source differs from this pipeline");
        builds::prepare_fixture_worker(crate::authoring::FixtureWorkerRequest {
            checkout: checkout.into(),
            root: request.root.clone(),
            run: run.into(),
            workers: request.workers,
        })?;
        ensure!(digest(&worker_source)? == worker_hash, "fixture worker source changed during build");
    }
    write_build_input(&run.join("wrapper.rs"), wrapper.as_bytes(), execution.is_some())?;
    let root = &request.root;
    let workers = request.workers;
    if dispatcher {
        let mut shim = if eproj {
            format!("#include {:?}\n{}", source.join("src/sm_86.c"), include_str!("dispatcher_eproj.c"))
        } else {
            include_str!("dispatcher_plm.c").to_owned()
        };
        if host {
            shim.push_str(include_str!("dispatcher_host.c"));
        }
        if eproj {
            shim.push_str(
                "void EprojInitializer(uint16 slot,uint16 address) { CallEprojInit(address|0x860000,slot); }\n",
            );
        }
        if manifest.projectile_rendering {
            shim.push_str("uint32 PortScreenShake(void) { Point16U p=GetValuesForScreenShaking(); return p.x|((uint32)p.y<<16); }\nvoid PortDrawEprojs(uint16 k,uint32 xy) { DrawEprojs(k,(Point16U){(uint16)xy,(uint16)(xy>>16)}); }\nvoid PortDrawEprojSpritemap(uint32 xy,uint32 map_gfx) { DrawEprojSpritemapWithBaseTile(0x8D,(uint16)map_gfx,(uint16)xy,(uint16)(xy>>16),(uint8)(map_gfx>>16),(uint16)(map_gfx>>16)&0xFF00); }\nvoid PortDrawEprojSpritemapOffscreen(uint32 xy,uint32 map_gfx) { DrawEprojSpritemapWithBaseTileOffscreen(0x8D,(uint16)map_gfx,(uint16)xy,(uint16)(xy>>16),(uint8)(map_gfx>>16),(uint16)(map_gfx>>16)&0xFF00); }\n");
        }
        if execution.is_some() {
            shim = shim.replace(&source.display().to_string(), "/inputs/c");
        }
        write_build_input(&run.join("plm-shim.c"), shim.as_bytes(), execution.is_some())?;
        let headless = if execution.is_some() {
            PathBuf::from("/inputs/rust/tools/decomp/headless.c")
        } else {
            checkout.join("tools/decomp/headless.c")
        };
        write_build_input(
            &run.join("host-support.c"),
            format!("#define main retained_main\n#include {headless:?}\n").as_bytes(),
            execution.is_some(),
        )?;
    }
    let units = if dispatcher || complete_c { c_units(source)? } else { vec![] };
    let description = crate::recipe::c_compiler_command(recipe, manifest, source, run, &units, fork)?;
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "c-build", checkout, source)?;
    } else {
        builds::ordinary_c_build(&description, run, checkout, root, workers)?;
    }
    let description = crate::recipe::rust_compiler_command(recipe, run);
    if let Some(execution) = execution {
        execution.compile_description(&description, run, "rust-build", checkout, source)?;
    } else {
        tool_execution::builds::execute(&description.command, run, "rust-build", root, workers)?;
    }
    write_build_input(&run.join("candidate.rs"), &bounded::read(&module)?, execution.is_some())?;
    let dependency_request = crate::authoring::DependenciesRequest {
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
    if dispatcher {
        let name = "tools/decomp/headless.c";
        harness.insert(name.to_owned(), digest(&checkout.join(name))?);
        fs::copy(checkout.join(name), run.join("pipeline-headless.c"))?;
    }
    Ok(Implementation {
        fixture_execution: Some(if fork { "dependency-v1-fork" } else { "dependency-v1-fresh" }.into()),
        dependency_contract: "rust-family-v2".into(),
        source_root: checkout.into(),
        rust_dependencies: dependencies,
        harness_sources: harness,
        module: module.clone(),
        module_sha256: digest(&module)?,
        memory_sha256: digest(&checkout.join("crates/port/src/memory.rs"))?,
        binaries: {
            let mut binaries =
                BTreeMap::from([("c".into(), digest(&run.join("c"))?), ("rust".into(), digest(&run.join("rust"))?)]);
            if fork {
                binaries.insert("libfixture_worker.a".into(), digest(&run.join("libfixture_worker.a"))?);
            }
            binaries
        },
        ..Default::default()
    })
}

fn numeric(path: &Path) -> Result<u64> {
    let data = bounded::text(path)?;
    numeric_text(&data)
}
fn numeric_text(data: &str) -> Result<u64> {
    let final_line = data.lines().last().context("missing final numeric routine return")?;
    ensure!(
        !final_line.is_empty() && final_line.bytes().all(|v| v.is_ascii_digit()),
        "missing final numeric routine return"
    );
    Ok(final_line.parse()?)
}
pub(crate) fn ordered_tasks(manifest: &Manifest) -> Result<Vec<(&Routine, &Case, usize)>> {
    Ok(ordered_task_indices(manifest)?
        .into_iter()
        .map(|(routine, case)| (&manifest.routines[routine], &manifest.routines[routine].cases[case], routine))
        .collect())
}

pub(crate) fn ordered_task_indices(manifest: &Manifest) -> Result<Vec<(usize, usize)>> {
    let mut tasks = Vec::new();
    let mut names = BTreeSet::new();
    for (index, routine) in manifest.routines.iter().enumerate() {
        ensure!(!routine.cases.is_empty(), "empty declared routine coverage");
        for (case_index, case) in routine.cases.iter().enumerate() {
            ensure!(names.insert(case.name.as_str()), "duplicate fixture case name");
            tasks.push((index, case_index));
        }
    }
    if let Some(order) = &manifest.case_order {
        ensure!(order.len() == tasks.len(), "case order must include every unique case exactly once");
        let original: BTreeMap<_, _> =
            tasks.into_iter().map(|task| (manifest.routines[task.0].cases[task.1].name.as_str(), task)).collect();
        tasks = Vec::with_capacity(order.len());
        let mut seen = BTreeSet::new();
        for name in order {
            let task = original.get(name.as_str()).context("unknown ordered case")?;
            ensure!(seen.insert(name.as_str()), "duplicate ordered case");
            tasks.push(*task);
        }
    }
    let mut seen = BTreeSet::new();
    for &(routine, case) in &tasks {
        let case = &manifest.routines[routine].cases[case];
        if let Some(previous) = &case.continue_from {
            ensure!(seen.contains(previous.as_str()), "continuation must follow its verified predecessor");
        }
        seen.insert(case.name.as_str());
    }
    Ok(tasks)
}
fn authenticate_continuation(
    case: &Case,
    run: &Path,
    before: &[u8],
    reader: &mut crate::admission_reader::Window<'_, '_>,
) -> Result<()> {
    if case.continue_from.is_none() {
        return Ok(());
    }
    let previous = continuation_directory(case, run)?;
    let previous_c = reader.read(previous.join("c-after.wram"))?;
    ensure!(
        previous_c.len() == WRAM && previous_c == reader.read(previous.join("rust-after.wram"))?,
        "continuation previous WRAM differs"
    );
    let mut expected = previous_c.to_vec();
    for write in &case.writes {
        ensure!(
            write.offset.checked_add(write.data.len()).is_some_and(|end| end <= WRAM),
            "continuation write outside WRAM"
        );
        expected[write.offset..write.offset + write.data.len()].copy_from_slice(&write.data);
    }
    ensure!(before == expected, "continuation did not inherit previous WRAM");
    Ok(())
}

pub(crate) fn case(
    manifest: &Manifest,
    routine: &Routine,
    case: &Case,
    run: &Path,
    resume: bool,
    mut execute: impl FnMut(&Path, &str) -> Result<()>,
) -> Result<CaseRecord> {
    let name = &case.name;
    let symbol = &routine.symbol;
    let directory = run.join(name);
    prepare_case_with(case, &directory, &routine.argument_types()?, resume)?;
    if manifest.projectile_host {
        fs::write(directory.join("host-before.bin"), host_bytes(case, run)?)?;
    }
    let mut returns = BTreeMap::new();
    let mut hashes = BTreeMap::new();
    let mut states = [Vec::new(), Vec::new()];
    for (subject, state) in ["rust", "c"].into_iter().zip(&mut states) {
        metrics::measure("fixture-execution", || execute(&directory, subject))?;
        metrics::measure("fixture-comparison", || {
            returns.insert(subject.into(), numeric(&directory.join(format!("{subject}.stdout")))?);
            let (bytes, hash) = bounded::read_with_digest(directory.join(format!("{subject}-after.wram")))?;
            *state = bytes;
            hashes.insert(subject.into(), hash);
            Ok(())
        })?;
    }
    metrics::measure("fixture-comparison", || {
        let mut record = CaseRecord {
            routine: symbol.clone(),
            name: name.clone(),
            returns,
            before_sha256: digest(&directory.join("before.wram"))?,
            after_sha256: hashes,
            ..Default::default()
        };
        ensure!(record.returns["c"] == record.returns["rust"], "return mismatch: {symbol}/{name}");
        if let Some(expected) = case.expected_return {
            ensure!(expected == record.returns["c"], "independent expected return mismatch: {name}");
        }
        let [rust, c] = &states;
        ensure!(c.len() == WRAM && rust.len() == WRAM, "subject did not retain full WRAM extent");
        if c != rust {
            let offset = c.iter().zip(rust).position(|(a, b)| a != b).unwrap();
            record.first_mismatch = Some(Mismatch { offset, c: c[offset], rust: rust[offset] });
            atomic_toml(&directory.join("rejection.toml"), &record)?;
            bail!("WRAM mismatch at {offset:#x}: {symbol}/{name}");
        }
        if case.expected_after_file.is_some() {
            let (expected, hash) = bounded::read_with_digest(directory.join("expected-after.wram"))?;
            ensure!(*c == expected, "captured expected WRAM mismatch: {name}");
            record.capture_after_sha256 = Some(hash);
        }
        if manifest.projectile_host {
            authenticate_host(&directory, case)?;
        }
        Ok(record)
    })
}

pub(crate) fn inventory(run: &Path, base: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    decomp_source::context::limited(|| inventory_bounded(run, base, result, 0, ArtifactLimits::default()))
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ArtifactLimits {
    pub files: usize,
    pub paths: usize,
}
impl Default for ArtifactLimits {
    fn default() -> Self {
        Self { files: 65536, paths: 65536 }
    }
}
impl ArtifactLimits {
    pub(crate) const MAX_FILES: usize = 2 * 1024 * 1024;

    pub(crate) fn validate(self) -> Result<()> {
        ensure!(
            self.files > 0
                && self.files <= Self::MAX_FILES
                && self.paths >= self.files
                && self.paths <= bounded::MAX_DECLARED_PATHS,
            "declared family artifact ceiling exceeded"
        );
        Ok(())
    }
}

pub(crate) fn inventory_with_limits(
    run: &Path,
    base: &Path,
    result: &mut BTreeMap<String, String>,
    limits: ArtifactLimits,
) -> Result<()> {
    limits.validate()?;
    decomp_source::context::discovery_with_limit(limits.paths, || inventory_bounded(run, base, result, 0, limits))
}

fn inventory_bounded(
    run: &Path,
    base: &Path,
    result: &mut BTreeMap<String, String>,
    depth: usize,
    limits: ArtifactLimits,
) -> Result<()> {
    ensure!(depth <= 32, "family artifact inventory depth exceeded");
    let paths = bounded::entries(base)?;
    for path in paths {
        bounded::check()?;
        ensure!(!path.is_symlink(), "run artifact symlink");
        if path.is_dir() {
            inventory_bounded(run, &path, result, depth + 1, limits)?;
        } else if path != run.join("verdict.toml")
            && !path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml"))
        {
            result.insert(path.strip_prefix(run)?.to_string_lossy().into_owned(), inputs::artifact_digest(run, &path)?);
            ensure!(result.len() <= limits.files, "family artifact inventory count exceeded");
        }
    }
    Ok(())
}

pub fn verify(request: VerifyRequest) -> Result<Verdict> {
    verify_with_declarations(request, &Arc::new(crate::declarations::Declarations::standard()), None, None, None)
}

pub(crate) fn verify_with_declarations(
    request: VerifyRequest,
    declarations: &Arc<crate::declarations::Declarations>,
    captured: Option<Arc<crate::declarations::Declaration>>,
    captured_inputs: Option<&Arc<crate::input_view::InputView>>,
    node: Option<crate::work_graph::FamilyNode<'_>>,
) -> Result<Verdict> {
    use crate::work_graph::{FamilyNode, Phase};
    let checkout = fs::canonicalize(&request.checkout)?;
    let source = fs::canonicalize(&request.source)?;
    let output = request.output.clone();
    fs::create_dir_all(&output)?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros();
    let run = output.join(format!("{stamp}-{}-{}", std::process::id(), unique_id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut metrics = crate::metrics::Run::start(run.join("family.metrics.toml"), "family")?;
    metrics.context.configuration("manifest", request.manifest.display());
    metrics.context.configuration("workers", request.workers);
    metrics.context.configuration("case-jobs", request.case_jobs);
    metrics.context.configuration(
        "fixture-comparison-accounting",
        "output parsing/digests plus complete equality/capture checks; spans may overlap across independent Fixtures",
    );
    metrics.context.expect_stages(&[
        "family-input-authentication",
        "family-builds",
        "fixture-execution-and-comparison",
        "fixture-execution",
        "fixture-comparison",
        "family-publication",
    ]);
    let mut verdict = Verdict {
        status: "rejected".into(),
        run: run.clone(),
        original_rom_parity: "not-tested".into(),
        ..Default::default()
    };
    let result = (|| -> Result<()> {
        if let Some(node) = node {
            node.check_admission()?;
            node.phase(Phase::Preparing);
        }
        let preparing_metadata = node.map(FamilyNode::metadata_permit);
        let preparing = metrics.context.stage("family-input-authentication")?;
        let manifest_path = source_path(&checkout, request.manifest.to_str().context("non-UTF8 manifest")?);
        let declaration = if let Some(captured) = captured {
            ensure!(digest(&manifest_path)? == captured.sha256, "family manifest changed after capture");
            captured
        } else {
            declarations.read(&manifest_path, None)?
        };
        let manifest_sha256 = declaration.sha256.clone();
        fs::write(run.join(crate::fixture_format::RUN_MANIFEST), declaration.bytes.as_slice())?;
        let mut manifest = (*declaration.manifest).clone();
        let recipe = crate::recipe::for_adapter(&request.adapter, &manifest)?;
        manifest = crate::recipe::normalize_manifest(&checkout, manifest, recipe.as_ref())?;
        let manifest = Arc::new(manifest);
        verdict.family = manifest.family.clone();
        let view = if let Some(view) = captured_inputs {
            ensure!(view.checkout() == checkout && view.source() == source, "family captured source context differs");
            view.clone()
        } else {
            crate::input_view::InputView::capture(
                &inputs::host_inputs()?,
                &checkout,
                &source,
                &[&manifest],
                false,
                declarations,
            )?
        };
        let view = view.for_family(&manifest)?;
        view.validate_manifest_inputs(&manifest)?;
        verdict.source = Some(view.source_identity().clone());
        fs::write(run.join("LICENSE.txt"), view.read(&source.join("LICENSE.txt"), None)?)?;
        let tasks = ordered_tasks(&manifest)?;
        ensure!(!tasks.is_empty(), "empty family cannot be verified");
        metrics.context.configuration("declared-cases", tasks.len());
        metrics.context.configuration("declared-routines", manifest.routines.len());
        if recipe.is_some() {
            for name in ["fixture-execution", "fixture-comparison"] {
                metrics.context.stage_unavailable(name, "adapter currently reports the combined Fixture interval; attributable producer intervals remain available");
            }
        }
        let jobs = request.case_jobs.max(1);
        let fork = fork_execution(&request)?;
        ensure!((1..=64).contains(&jobs), "case jobs must be 1..64");
        if let Some(rom) = manifest.rom.as_ref() {
            let bytes = view.read(&rom.file, Some(&rom.sha256))?;
            ensure!(!bytes.is_empty() && bytes.len() <= 0x400000, "ROM extent outside LoROM view");
            fs::write(run.join("rom.bin"), &bytes)?;
            verdict.rom = Some(RomIdentity { sha256: rom.sha256.clone(), bytes: bytes.len() });
        }
        preparing.finish("success")?;
        drop(preparing_metadata);
        if let Some(node) = node {
            node.check_admission()?;
            node.phase(Phase::Building);
        }
        verdict.implementation = Some(metrics::measure("family-builds", || {
            FamilyNode::metadata(node, || {
                view.validate_compiler_current()?;
                view.validate_materialized()
            })?;
            let result = view.enter(|| {
                crate::recipe::build(
                    recipe.as_ref(),
                    &manifest,
                    &request,
                    &view.physical_checkout(),
                    &view.physical_source(),
                    &run,
                    None,
                )
            });
            FamilyNode::metadata(node, || {
                view.validate_materialized()?;
                view.validate_compiler_current()
            })?;
            result
        })?);
        verdict.implementation.as_mut().unwrap().manifest =
            Some(Asset { file: fs::canonicalize(&manifest_path)?, sha256: manifest_sha256.clone() });
        atomic_toml(
            &run.join("manifest-input.toml"),
            verdict.implementation.as_ref().unwrap().manifest.as_ref().unwrap(),
        )?;
        crate::recipe::start_verdict(recipe.as_ref(), &manifest, &mut verdict);
        metrics::measure("fixture-execution-and-comparison", || {
            let outcomes = if let Some(node) = node {
                node.fixtures(crate::work_graph::Fixtures::new(
                    manifest.clone(),
                    recipe.clone(),
                    view.clone(),
                    run.clone(),
                    jobs,
                    fork,
                )?)?
            } else {
                crate::fixture_execution::run(
                    &tasks,
                    jobs,
                    &run,
                    &request.root,
                    request.workers,
                    fork,
                    |(routine, fixture, routine_index), worker| {
                        view.enter(|| {
                            crate::recipe::fixture(
                                recipe.as_ref(),
                                &manifest,
                                routine,
                                fixture,
                                routine_index,
                                &run,
                                crate::recipe::FixtureExecution::Ordinary { worker },
                            )
                        })
                    },
                )?
            };
            // The batch owns a bounded result reservation. Preallocate the
            // exact joined diagnostic extent before consuming errors, so it
            // never retains a Vec of copied messages or a growing String.
            let mut joined = if node.is_some() {
                let bytes = outcomes.iter().filter_map(|(_, result)| result.as_ref().err()).try_fold(
                    0usize,
                    |bytes, error| {
                        bytes
                            .checked_add(format!("{error:#}").len())
                            .and_then(|bytes| bytes.checked_add(2))
                            .context("batch diagnostic aggregate extent overflow")
                    },
                )?;
                Some(String::with_capacity(bytes))
            } else {
                None
            };
            let mut error_count = 0;
            let mut errors = Vec::new();
            for (index, outcome) in outcomes {
                match outcome {
                    Ok(record) => verdict.cases.push(record),
                    Err(error) => {
                        let rejection = tasks.get(index).map(|task| run.join(&task.1.name).join("rejection.toml"));
                        if let Some(rejection) = rejection.filter(|p| p.is_file()) {
                            verdict.cases.push(object(&rejection)?);
                        }
                        if let Some(joined) = &mut joined {
                            use std::fmt::Write as _;
                            if error_count > 0 {
                                joined.push_str("; ");
                            }
                            write!(joined, "{error:#}")?;
                            error_count += 1;
                        } else {
                            errors.push(format!("{error:#}"));
                        }
                    }
                }
            }
            if error_count > 0 {
                return Err(anyhow::Error::msg(joined.expect("batch diagnostic allocation")));
            }
            ensure!(errors.is_empty(), "{}", errors.join("; "));
            Ok(())
        })?;
        FamilyNode::metadata(node, || {
            view.validate_manifest_inputs(&manifest)?;
            ensure!(
                digest(&manifest_path)? == manifest_sha256 && digest(&run.join("manifest.smf"))? == manifest_sha256,
                "family manifest changed during verification"
            );
            Ok(())
        })?;
        verdict.status = "offline-verified".into();
        verdict.verified_cases = tasks.len();
        verdict.verified_routines = manifest.routines.len();
        crate::recipe::verdict_fields(recipe.as_ref(), &manifest, &mut verdict)?;
        verdict.scope = "declared valid fixture domain only; full canonical WRAM and return values".into();
        Ok(())
    })();
    if let Err(error) = result {
        verdict.error = Some(if node.is_some() {
            // A batch's full canonical diagnostic is already an owned String.
            // Move it into evidence instead of allocating a second full copy.
            error.downcast::<String>().unwrap_or_else(|error| format!("{error:#}"))
        } else {
            format!("{error:#}")
        });
        if let Some(node) = node {
            node.stop(verdict.error.as_deref().unwrap());
        }
    }
    if let Some(node) = node {
        node.phase(Phase::Publishing);
    }
    let publication = metrics::measure("family-publication", || {
        FamilyNode::metadata(node, || {
            let mut artifacts = BTreeMap::new();
            inventory(&run, &run, &mut artifacts)?;
            metrics.context.configuration("artifact-files", artifacts.len());
            verdict.artifacts = artifacts;
            atomic_toml(&run.join("verdict.toml"), &verdict)
        })
    });
    if let Err(error) = &publication {
        metrics.finish(crate::metrics::outcome(Some(&format!("{error:#}"))), Some(format!("{error:#}")))?;
    }
    publication?;
    let metrics_error = verdict
        .error
        .as_deref()
        .map(|error| if node.is_some() { crate::work_graph::diagnostic(error) } else { error.to_owned() });
    metrics.finish(crate::metrics::outcome(verdict.error.as_deref()), metrics_error)?;
    Ok(verdict)
}

pub fn authenticate(request: AuthenticateRequest) -> Result<Verdict> {
    let source = request.checkout.join("deps/snesrev-sm");
    authenticate_with_source(request, &source)
}

pub(crate) fn authenticate_with_source(request: AuthenticateRequest, source: &Path) -> Result<Verdict> {
    Ok(admit_with_source(request, source, &crate::declarations::Declarations::standard())?.report)
}

pub(crate) struct AdmittedFamily {
    pub report: Verdict,
    pub original: Arc<Manifest>,
    pub effective: Arc<Manifest>,
}

pub(crate) fn admit_with_source(
    request: AuthenticateRequest,
    source: &Path,
    declarations: &crate::declarations::Declarations,
) -> Result<AdmittedFamily> {
    decomp_source::context::limited(|| {
        authenticate_with_source_bounded(request, source, ArtifactLimits::default(), declarations)
    })
}

pub(crate) fn authenticate_with_source_limits(
    request: AuthenticateRequest,
    source: &Path,
    limits: ArtifactLimits,
) -> Result<Verdict> {
    Ok(admit_with_source_limits(request, source, limits, &crate::declarations::Declarations::standard())?.report)
}

pub(crate) fn admit_with_source_limits(
    request: AuthenticateRequest,
    source: &Path,
    limits: ArtifactLimits,
    declarations: &crate::declarations::Declarations,
) -> Result<AdmittedFamily> {
    limits.validate()?;
    decomp_source::context::discovery_with_limit(limits.paths, || {
        authenticate_with_source_bounded(request, source, limits, declarations)
    })
}

fn authenticate_with_source_bounded(
    request: AuthenticateRequest,
    source: &Path,
    limits: ArtifactLimits,
    declarations: &crate::declarations::Declarations,
) -> Result<AdmittedFamily> {
    metrics::measure("family-evidence-admission", || authenticate_contents(request, source, limits, declarations))
}

fn authenticate_contents(
    request: AuthenticateRequest,
    source: &Path,
    limits: ArtifactLimits,
    declarations: &crate::declarations::Declarations,
) -> Result<AdmittedFamily> {
    let context = metrics::active();
    if let Some(context) = &context {
        context.expect_stages(&[
            "family-admission-inventory",
            "family-admission-input-authentication",
            "family-admission-comparison",
        ]);
    }
    let checkout = fs::canonicalize(&request.checkout)?;
    let path = &request.verdict;
    let run = fs::canonicalize(path.parent().context("verdict parent missing")?)?;
    let verdict: Verdict = object(path)?;
    ensure!(verdict.status == "offline-verified", "family evidence is not offline verification");
    // Expected seals are provisional until every member has been authenticated
    // by this invocation. They cannot construct an admitted result themselves.
    let actual = &verdict.artifacts;
    let mut admission = metrics::measure("family-admission-inventory", || {
        crate::admission_reader::Admission::open(&run, actual, limits)
    })?;
    let mut reader = admission.window();
    let authenticating =
        context.as_ref().map(|context| context.stage("family-admission-input-authentication")).transpose()?;
    for required in
        ["manifest.smf", "candidate.rs", "memory.rs", "oracle.c", "wrapper.rs", "compiler.d", "pipeline-inputs.toml"]
    {
        ensure!(actual.contains_key(required), "family artifact missing: {required}");
    }
    let manifest_bytes = reader.read(run.join(crate::fixture_format::RUN_MANIFEST))?;
    let original =
        declarations.decode_shared(manifest_bytes.shared_bytes(), manifest_bytes.hash().to_owned())?.manifest.clone();
    drop(manifest_bytes);
    let mut manifest = original.clone();
    ensure!(manifest.family == verdict.family, "family identity differs");
    let implementation = verdict.implementation.as_ref().context("missing family implementation")?;
    ensure!(
        (manifest.callback_adapter.as_deref() == Some("game-host"))
            == (implementation.callback_adapter.as_deref() == Some("game-host")),
        "shared game host adapter identity differs"
    );
    if let Some(execution) = &implementation.fixture_execution {
        ensure!(
            ["dependency-v1-fork", "dependency-v1-fresh"].contains(&execution.as_str()),
            "unknown fixture execution contract"
        );
        for name in [
            "crates/offline-verification/src/fixture_execution.rs",
            "crates/tool-execution/src/metrics.rs",
            "crates/tool-execution/src/resource_sampler.rs",
        ] {
            ensure!(implementation.harness_sources.contains_key(name), "fixture execution source seal missing: {name}");
        }
        if execution == "dependency-v1-fork" {
            ensure!(
                ["rust-family-v2", crate::ridley_subsystem_adapter::CONTRACT]
                    .contains(&implementation.dependency_contract.as_str()),
                "fork adapter dependency contract is not admitted"
            );
            ensure!(
                implementation.harness_sources.contains_key("crates/offline-verification/src/fixture_worker.rs"),
                "fixture worker source seal missing"
            );
            ensure!(
                actual.contains_key("fixture-worker.rs")
                    && (implementation.callback_adapter.as_deref() == Some("ridley-subsystem")
                        || actual.contains_key("libfixture_worker.a")),
                "fixture worker artifact missing"
            );
            ensure!(
                actual.get("fixture-worker.rs")
                    == implementation.harness_sources.get("crates/offline-verification/src/fixture_worker.rs"),
                "fixture worker generated source differs"
            );
            ensure!(
                implementation.callback_adapter.as_deref() == Some("ridley-subsystem")
                    || implementation.binaries.get("libfixture_worker.a") == actual.get("libfixture_worker.a"),
                "fixture worker library seal differs"
            );
        }
    } else {
        ensure!(
            !actual.contains_key("sources/crates/offline-verification/src/fixture_execution.rs"),
            "fixture execution contract missing"
        );
    }
    let manifest_identity = implementation.manifest.as_ref().context("current family manifest seal missing")?;
    ensure!(actual.contains_key("manifest-input.toml"), "current source inputs manifest seal missing");
    let input_manifest: Asset = reader.object(run.join("manifest-input.toml"))?;
    ensure!(input_manifest == *manifest_identity, "family manifest source input differs");
    let manifest_source = manifest_identity
        .file
        .strip_prefix(&implementation.source_root)
        .map(|relative| checkout.join(relative))
        .unwrap_or_else(|_| manifest_identity.file.clone());
    ensure!(
        digest(&manifest_source)? == manifest_identity.sha256
            && actual[crate::fixture_format::RUN_MANIFEST] == manifest_identity.sha256,
        "family manifest changed"
    );
    let callbacks = implementation.dependency_contract == "rust-callback-v2";
    let plm = implementation.dependency_contract == "rust-plm-v2";
    let ridley = implementation.dependency_contract == crate::ridley_subsystem_adapter::CONTRACT;
    if plm {
        manifest = Arc::new(crate::plm_spawn::normalize(&manifest)?);
    }
    ordered_tasks(&manifest)?;
    ensure!(
        ridley || callbacks || plm || implementation.dependency_contract == "rust-family-v2",
        "unsupported family dependency contract"
    );
    let binaries = if ridley {
        vec!["ridley", "c", "rust", "libridley.a"]
    } else if plm {
        vec!["spawn", "libplm.a"]
    } else if callbacks {
        vec!["collision", "libcollision.a"]
    } else {
        vec!["c", "rust"]
    };
    for name in &binaries {
        ensure!(actual.contains_key(*name), "family binary artifact missing: {name}");
    }
    let pinned = verify_source(&checkout, source)?;
    ensure!(Some(pinned) == verdict.source, "family pinned source differs");
    let captured_ridley = ridley && inputs::captured_ridley_manifest(&manifest);
    for map in [&implementation.rust_dependencies, &implementation.harness_sources] {
        for (name, expected) in map {
            let path = source_path(&checkout, name);
            let hash = if captured_ridley && path == Path::new(inputs::CAPTURED_RIDLEY_EVENT) {
                inputs::captured_ridley_event_digest(&path)?
            } else {
                digest(&path)?
            };
            ensure!(hash == *expected, "family implementation changed: {name}");
        }
    }
    inputs::authenticate_implementation(&run, implementation, actual, &mut reader)?;
    if ridley {
        ensure!(
            implementation.callback_adapter.as_deref() == Some("ridley-subsystem")
                && manifest.callback_adapter.as_deref() == Some("ridley-subsystem"),
            "Ridley adapter seal differs"
        );
        ensure!(
            actual.contains_key("pipeline-headless.c")
                && implementation.harness_sources.contains_key("tools/decomp/headless.c"),
            "Ridley host input missing"
        );
        crate::ridley_subsystem_adapter::check_instrumented_units(source, &run, &mut reader)?;
        ensure!(verdict.gameplay_translation == Some(true), "Ridley translation scope missing");
        ensure!(
            verdict.rejected_abi_inputs == 7
                && reader.text(run.join("abi-controls.stdout"))?.lines().last() == Some("7 rejected ABI inputs"),
            "Ridley ABI controls missing"
        );
        ensure!(
            verdict.remaining_c_gameplay_callbacks
                == usize::try_from(verdict.cases.iter().map(|case| case.callbacks).sum::<u64>())?,
            "Ridley C service count differs"
        );
    } else if plm {
        ensure!(
            actual.contains_key("pipeline-headless.c")
                && implementation.harness_sources.contains_key("tools/decomp/headless.c"),
            "PLM host input missing"
        );
        ensure!(
            verdict.rejected_abi_inputs == 5
                && reader.text(run.join("abi-controls.stdout"))?.lines().last() == Some("5 rejected ABI inputs"),
            "PLM ABI rejection evidence incomplete"
        );
        let counts = manifest.routines.iter().map(|r| (r.symbol.clone(), r.cases.len())).collect();
        ensure!(verdict.allocator_cases == counts, "allocator coverage changed");
    } else if callbacks {
        ensure!(
            actual.contains_key("pipeline-headless.c")
                && implementation.harness_sources.contains_key("tools/decomp/headless.c"),
            "callback host input missing"
        );
        if matches!(implementation.callback_adapter.as_deref(), Some("power-bomb" | "game-host")) {
            let original = bounded::text(source.join("src/sm_88.c"))?;
            let token = "uint16 SpawnHdmaObject(uint8 db, const void *p)";
            ensure!(
                original.matches(token).count() == 1
                    && reader.text(run.join("sm_88-allocation-seam.c"))?
                        == original.replace(token, "uint16 PortActualSpawnHdmaObject(uint8 db, const void *p)"),
                "power bomb allocation seam changed"
            );
        }
        if implementation.callback_adapter.as_deref() == Some("game-host") {
            for required in ["game_host_abi.rs.in", "game_host_dependency.c.in"] {
                ensure!(
                    implementation.harness_sources.contains_key(&format!("crates/offline-verification/src/{required}")),
                    "shared host input missing: {required}"
                );
            }
            let original = bounded::text(source.join("src/sm_80.c"))?;
            let token = "void NMI_ProcessVramWriteQueue(void)";
            ensure!(
                original.matches(token).count() == 1
                    && reader.text(run.join("sm_80-vram-seam.c"))?
                        == original.replace(token, "void PortActualNmiProcessVramWriteQueue(void)"),
                "shared host VRAM seam changed"
            );
            ensure!(
                verdict.remaining_c_gameplay_callbacks
                    == usize::try_from(verdict.cases.iter().map(|case| case.callbacks).sum::<u64>())?,
                "shared host delegated callback count differs"
            );
        }
        ensure!(
            verdict.gameplay_translation == Some(!manifest.infrastructure_probe),
            "callback gameplay scope differs"
        );
    } else if manifest.uses_dispatcher() {
        for required in ["plm-shim.c", "host-support.c", "pipeline-headless.c"] {
            ensure!(actual.contains_key(required), "dispatcher artifact missing: {required}");
        }
        ensure!(
            implementation.harness_sources.contains_key("tools/decomp/headless.c"),
            "dispatcher host input missing"
        );
    }
    ensure!(
        implementation.rust_dependencies.contains_key("crates/port/src/memory.rs"),
        "canonical memory missing from compiler closure"
    );
    let module = source_path(&checkout, &manifest.rust_module);
    let module = fs::canonicalize(module)?;
    let key = module.strip_prefix(&checkout).unwrap_or(&module).to_string_lossy();
    ensure!(implementation.rust_dependencies.contains_key(key.as_ref()), "candidate missing from compiler closure");
    ensure!(
        digest(&module)? == implementation.module_sha256
            && digest(&checkout.join("crates/port/src/memory.rs"))? == implementation.memory_sha256,
        "family implementation identity differs"
    );
    for binary in binaries {
        ensure!(actual.get(binary) == implementation.binaries.get(binary), "family binary changed");
    }
    if let Some(authenticating) = authenticating {
        authenticating.finish("success")?;
    }
    drop(reader);
    metrics::measure("family-admission-comparison", || {
        let mut expected = BTreeMap::new();
        for routine in &manifest.routines {
            ensure!(!&routine.cases.is_empty(), "empty family coverage");
            for fixture in &routine.cases {
                ensure!(
                    expected.insert((routine.symbol.as_str(), fixture.name.as_str()), fixture).is_none(),
                    "duplicate family coverage"
                );
            }
        }
        let mut observed = BTreeSet::new();
        for record in &verdict.cases {
            let key = (record.routine.as_str(), record.name.as_str());
            let fixture = expected.get(&key).context("unexpected family coverage")?;
            ensure!(observed.insert(key), "duplicate observed case");
            let directory = run.join(key.1);
            let mut reader = admission.window();
            ensure!(
                reader.text(directory.join("case.sha256"))? == crate::fixture_format::case_identity(fixture)?,
                "retained fixture declaration identity differs"
            );
            if ridley {
                let checked = crate::ridley_subsystem_adapter::check_with(&directory, fixture, &mut reader)?;
                ensure!(
                    checked.results == record.results
                        && checked.returns == record.returns
                        && checked.callbacks == record.callbacks
                        && checked.before_sha256 == record.before_sha256
                        && checked.after_sha256 == record.after_sha256
                        && checked.capture_after_sha256 == record.capture_after_sha256,
                    "Ridley retained case differs"
                );
                continue;
            }
            if plm {
                ensure!(
                    crate::plm_spawn::check_with(&directory, fixture, &mut reader)?.spawn_results
                        == record.spawn_results,
                    "retained PLM result differs"
                );
                continue;
            }
            if callbacks {
                let checked = callback::check_with(
                    &directory,
                    fixture,
                    manifest.callback_adapter.as_deref() == Some("game-host"),
                    &mut reader,
                )?;
                ensure!(
                    checked.results == record.results && checked.callbacks == record.callbacks,
                    "retained callback result differs"
                );
                continue;
            }
            let before = reader.read(directory.join("before.wram"))?;
            authenticate_continuation(fixture, &run, &before, &mut reader)?;
            let c = reader.read(directory.join("c-after.wram"))?;
            let rust = reader.read(directory.join("rust-after.wram"))?;
            ensure!(
                before.len() == WRAM && c.len() == WRAM && rust.len() == WRAM && c == rust,
                "family actual full WRAM outputs differ"
            );
            ensure!(
                before.hash() == record.before_sha256
                    && c.hash() == record.after_sha256["c"]
                    && rust.hash() == record.after_sha256["rust"],
                "family case hashes differ"
            );
            let a = numeric_text(&reader.text(directory.join("c.stdout"))?)?;
            let b = numeric_text(&reader.text(directory.join("rust.stdout"))?)?;
            ensure!(
                a == b && record.returns["c"] == a && record.returns["rust"] == b,
                "family actual return outputs differ"
            );
            if let Some(value) = fixture.expected_return {
                ensure!(value == a, "family expected return differs");
            }
            if fixture.expected_after_file.as_ref().is_some() {
                ensure!(
                    c == reader.read(directory.join("expected-after.wram"))?,
                    "family actual captured output differs"
                );
            }
            if manifest.projectile_host {
                authenticate_host_with(&directory, fixture, &mut reader)?;
            }
        }
        ensure!(
            !observed.is_empty()
                && observed.len() == expected.len()
                && verdict.verified_cases == observed.len()
                && verdict.verified_routines == manifest.routines.len(),
            "family coverage incomplete"
        );
        Ok(())
    })?;
    let read_stats = metrics::measure("family-admission-inventory", || admission.finish())?;
    if let Some(context) = &context {
        context.configuration("admitted-cases", verdict.verified_cases);
        context.configuration("admitted-routines", verdict.verified_routines);
        context.configuration("admitted-artifact-files", actual.len());
        context.configuration("admitted-payload-reads", read_stats.reads);
        context.configuration("admitted-payload-read-bytes", read_stats.bytes);
        context.configuration("admitted-payload-window-hits", read_stats.cache_hits);
        context.configuration("admitted-payload-window-peak-bytes", read_stats.peak_window_bytes);
    }
    Ok(AdmittedFamily { report: verdict, original, effective: manifest })
}

#[cfg(test)]
mod source_tests {
    use super::*;

    #[test]
    fn explicit_artifact_profile_keeps_a_finite_file_count() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        fs::write(directory.0.join("first"), b"one")?;
        fs::write(directory.0.join("second"), b"two")?;
        let mut inventory = BTreeMap::new();
        let error =
            inventory_with_limits(&directory.0, &directory.0, &mut inventory, ArtifactLimits { files: 1, paths: 4 })
                .unwrap_err();
        assert!(error.to_string().contains("inventory count"));
        inventory.clear();
        inventory_with_limits(&directory.0, &directory.0, &mut inventory, ArtifactLimits { files: 2, paths: 4 })?;
        assert_eq!(inventory.len(), 2);
        assert!(
            ArtifactLimits { files: ArtifactLimits::MAX_FILES + 1, paths: bounded::MAX_DECLARED_PATHS }
                .validate()
                .is_err()
        );
        assert_eq!(ArtifactLimits::default().files, 65536);
        assert_eq!(ArtifactLimits::default().paths, 65536);
        Ok(())
    }
}

#[cfg(test)]
mod host_tests {
    use super::*;
    #[test]
    fn projectile_rendering_selects_specialized_bank86_adapter_without_dispatcher_symbols() {
        let manifest = Manifest {
            bank: "sm_86.c".into(),
            projectile_rendering: true,
            routines: vec![Routine { symbol: "PortScreenShake".into(), ..Default::default() }],
            ..Default::default()
        };
        assert!(manifest.uses_dispatcher());
    }
    #[test]
    fn host_only_mismatch_and_missing_sidecar_fail_independently_of_wram() {
        let directory = std::env::temp_dir()
            .join(format!("sm-host-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&directory).unwrap();
        let fixture = Case { host_words: Some(vec![0, 1, 2, 3, 4, 5, 6, 65535]), ..Default::default() };
        let data = host_bytes(&fixture, &directory).unwrap();
        for name in ["host-before.bin", "c-host-after.bin", "rust-host-after.bin"] {
            fs::write(directory.join(name), &data).unwrap();
        }
        authenticate_host(&directory, &fixture).unwrap();
        let mut changed = data.clone();
        changed[0] = 1;
        fs::write(directory.join("rust-host-after.bin"), changed).unwrap();
        assert!(authenticate_host(&directory, &fixture).is_err());
        fs::remove_file(directory.join("rust-host-after.bin")).unwrap();
        assert!(authenticate_host(&directory, &fixture).is_err());
        assert!(host_bytes(&Case { host_words: Some(vec![0, 0]), ..Default::default() }, &directory).is_err());
        assert!(host_bytes(&Case { host_words: None, ..Default::default() }, &directory).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn continuation_orders_and_inherits_verified_wram_and_host() {
        let directory =
            std::env::temp_dir().join(format!("sm-continuation-test-{}-{}", std::process::id(), unique_id()));
        fs::create_dir(&directory).unwrap();
        fs::create_dir(directory.join("first")).unwrap();
        let before = vec![3; WRAM];
        let host = vec![7; 16];
        for subject in ["c", "rust"] {
            fs::write(directory.join("first").join(format!("{subject}-after.wram")), &before).unwrap();
            fs::write(directory.join("first").join(format!("{subject}-host-after.bin")), &host).unwrap();
        }
        let fixture = Case {
            name: "next".into(),
            continue_from: Some("first".into()),
            writes: vec![Write { offset: 42, data: vec![99] }].into(),
            ..Default::default()
        };
        prepare_case_with(&fixture, &directory.join("next"), &[], false).unwrap();
        let inherited = bounded::read(directory.join("next/before.wram")).unwrap();
        assert_eq!(inherited[42], 99);
        assert_eq!(inherited[41], 3);
        authenticate_continuation(
            &fixture,
            &directory,
            &inherited,
            &mut crate::admission_reader::Window::unsealed(&directory),
        )
        .unwrap();
        assert_eq!(host_bytes(&fixture, &directory).unwrap(), host);
        let manifest = Manifest {
            routines: vec![Routine {
                symbol: "Touch".into(),
                cases: vec![fixture.clone(), Case { name: "first".into(), ..Default::default() }],
                ..Default::default()
            }],
            case_order: Some(vec!["first".into(), "next".into()]),
            ..Default::default()
        };
        assert_eq!(ordered_tasks(&manifest).unwrap()[0].1.name, "first");
        let mut wrong = manifest.clone();
        wrong.case_order = Some(vec!["next".into(), "first".into()]);
        assert!(ordered_tasks(&wrong).is_err());
        let mut wrong = fixture.clone();
        wrong.host_words = Some(vec![0; 8]);
        assert!(host_bytes(&wrong, &directory).is_err());
        fs::write(directory.join("first/rust-after.wram"), vec![4; WRAM]).unwrap();
        assert!(
            authenticate_continuation(
                &fixture,
                &directory,
                &inherited,
                &mut crate::admission_reader::Window::unsealed(&directory),
            )
            .is_err()
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
