//! Native offline family verification. Acceptance is limited to declared fixtures.
use crate::family_model::*;
use crate::{atomic_toml, builds, callback, digest, inputs};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;

#[derive(Deserialize)]
struct DependencyPins {
    dependencies: Vec<DependencyPin>,
}
#[derive(Deserialize)]
struct DependencyPin {
    name: String,
    archive: String,
    revision: String,
    sha256: String,
}
const WRAM: usize = 0x20000;
static NEXT: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn object<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
pub(crate) fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key].as_str().with_context(|| format!("missing {key}"))
}
pub(crate) fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value[key].as_array().with_context(|| format!("missing {key}"))
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
pub(crate) fn types(routine: &Routine) -> Result<Vec<String>> {
    if let Some(values) = &routine.arg_types {
        return Ok(values.clone());
    }
    ensure!(routine.args <= 2, "too many arguments");
    Ok(vec!["uint16".into(); routine.args])
}
pub(crate) fn dispatcher(manifest: &Manifest) -> bool {
    manifest.projectile_host
        || manifest.projectile_rendering
        || manifest.routines.iter().any(|r| {
            r.pointer_bank.is_some()
                || matches!(
                    r.symbol.as_str(),
                    "EprojInstruction" | "EprojPreinstruction" | "PlmInstruction" | "PlmFrame" | "PlmControlFrame"
                )
        })
}
fn continuation_directory(case: &Case, run: &Path) -> Result<PathBuf> {
    let name = case.continue_from.as_deref().context("continuation missing")?;
    ensure!(safe_relative(name) && !name.contains('/') && name != case.name, "invalid continuation case");
    Ok(run.join(name))
}
fn host_bytes(case: &Case, run: &Path) -> Result<Vec<u8>> {
    if case.continue_from.is_some() {
        ensure!(case.host_words.is_none(), "continuation must inherit all external host words");
        let previous = continuation_directory(case, run)?;
        let c = fs::read(previous.join("c-host-after.bin"))?;
        ensure!(c.len() == 16 && c == fs::read(previous.join("rust-host-after.bin"))?, "continuation host differs");
        return Ok(c);
    }
    let words = case.host_words.as_ref().context("host words missing")?;
    ensure!(words.len() == 8, "projectile host requires eight words");
    Ok(words.iter().flat_map(|v| v.to_le_bytes()).collect())
}
fn authenticate_host(directory: &Path, fixture: &Case) -> Result<()> {
    let before = fs::read(directory.join("host-before.bin"))?;
    let c = fs::read(directory.join("c-host-after.bin"))?;
    let rust = fs::read(directory.join("rust-host-after.bin"))?;
    ensure!(
        before == host_bytes(fixture, directory.parent().context("case parent missing")?)?
            && c.len() == 16
            && c == rust,
        "projectile host state differs"
    );
    Ok(())
}

fn tar(archive: &Path, member: Option<&str>) -> Result<Vec<u8>> {
    let mut command = Command::new("tar");
    if let Some(member) = member {
        command.arg("-xOzf").arg(archive).arg("--").arg(member);
    } else {
        command.arg("-tzf").arg(archive);
    }
    let output = command.output()?;
    ensure!(output.status.success(), "read pinned source archive failed");
    Ok(output.stdout)
}

struct Extracted(PathBuf);
impl Drop for Extracted {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn verify_source(checkout: &Path, source: &Path) -> Result<Source> {
    let pins: DependencyPins = object(&checkout.join("tools/dependencies.toml"))?;
    let pin = pins.dependencies.iter().find(|v| v.name == "snesrev-sm").context("snesrev pin missing")?;
    let archive = source.parent().context("source parent missing")?.join("downloads").join(&pin.archive);
    ensure!(digest(&archive)? == pin.sha256, "pinned archive identity changed");
    let listing = String::from_utf8(tar(&archive, None)?)?;
    for member in listing.lines().filter(|v| !v.ends_with('/')) {
        ensure!(
            member.split_once('/').is_some_and(|(_, relative)| safe_relative(relative)),
            "unsafe source archive member"
        );
    }
    let temporary = std::env::temp_dir().join(format!(
        "sm-pinned-source-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&temporary)?;
    let extracted = Extracted(temporary);
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .args(["--strip-components=1", "-C"])
        .arg(&extracted.0)
        .status()?;
    ensure!(status.success(), "extract pinned source archive failed");
    let mut files = BTreeMap::new();
    for member in listing.lines().filter(|v| !v.ends_with('/')) {
        let (_, relative) = member.split_once('/').context("invalid source archive member")?;
        ensure!(safe_relative(relative), "unsafe source archive member");
        let path = source.join(relative);
        let expected = fs::read(extracted.0.join(relative))?;
        ensure!(!path.is_symlink() && fs::read(&path)? == expected, "pinned source changed: {relative}");
        files.insert(relative.to_owned(), digest(&path)?);
    }
    ensure!(!files.is_empty(), "empty pinned source archive");
    Ok(Source {
        revision: pin.revision.clone(),
        archive_sha256: pin.sha256.clone(),
        files,
        freshness: "exact pinned archive; no latest-upstream assertion".into(),
    })
}

fn asset(path: &Path, expected: &str, extent: Option<usize>) -> Result<Vec<u8>> {
    ensure!(digest(path)? == expected, "asset identity mismatch: {}", path.display());
    let bytes = fs::read(path)?;
    if let Some(n) = extent {
        ensure!(bytes.len() == n, "asset extent mismatch");
    }
    Ok(bytes)
}
pub(crate) fn prepare_case(case: &Case, directory: &Path, arg_types: &[String]) -> Result<()> {
    let name = &case.name;
    ensure!(
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
        "unsafe case name"
    );
    let mut bytes = if case.continue_from.is_some() {
        ensure!(case.before_file.is_none() && case.fill.is_none(), "continuation cannot replace inherited state");
        let previous = continuation_directory(case, directory.parent().context("case parent missing")?)?;
        let c = fs::read(previous.join("c-after.wram"))?;
        ensure!(
            c.len() == WRAM && c == fs::read(previous.join("rust-after.wram"))?,
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
    fs::create_dir(directory)?;
    fs::write(directory.join("before.wram"), bytes)?;
    if let Some(path) = &case.expected_after_file {
        fs::write(
            directory.join("expected-after.wram"),
            asset(path, case.expected_after_sha256.as_deref().context("expected digest missing")?, Some(WRAM))?,
        )?;
    }
    atomic_toml(&directory.join("case.toml"), case)
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

pub(crate) fn c_units(source: &Path) -> Result<Vec<PathBuf>> {
    let mut units = fs::read_dir(source.join("src"))?.map(|entry| Ok(entry?.path())).collect::<Result<Vec<_>>>()?;
    units.retain(|path| {
        path.extension().is_some_and(|v| v == "c")
            && path.file_stem().is_some_and(|v| {
                v.to_string_lossy().starts_with("sm_")
                    || ["spc_player", "tracing", "util"].contains(&v.to_string_lossy().as_ref())
            })
    });
    for entry in fs::read_dir(source.join("src/snes"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|v| v == "c") {
            units.push(path);
        }
    }
    units.sort();
    Ok(units)
}

fn build(
    request: &VerifyRequest,
    manifest: &Manifest,
    checkout: &Path,
    source: &Path,
    run: &Path,
) -> Result<Implementation> {
    let dispatcher = dispatcher(manifest);
    let eproj = manifest.projectile_host
        || manifest.projectile_rendering
        || manifest.routines.iter().any(|r| r.symbol == "EprojInstruction");
    let host = manifest.projectile_host;
    ensure!(
        !host
            || (eproj
                && manifest.rom.as_ref().is_some()
                && Path::new(&manifest.rust_module)
                    .file_name()
                    .is_some_and(|n| n == "enemy_projectiles.rs" || n == "observed_projectile_ownership.rs")),
        "projectile host requires ROM and bank86 allocator dispatcher"
    );
    let mut prototypes = Vec::new();
    let mut branches = Vec::new();
    for routine in &manifest.routines {
        let symbol = &routine.symbol;
        let kind = routine.returns.as_str();
        let args = types(routine)?;
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
        if routine.packed_u16_args.is_some() {
            ensure!(
                routine.packed_u16_args == Some(4) && args == ["uint32", "uint32"],
                "invalid packed four-word signature"
            );
            *prototypes.last_mut().unwrap() =
                format!("extern {} {symbol}(uint16_t,uint16_t,uint16_t,uint16_t);", c_type(kind)?);
            call = format!("{symbol}((uint16_t)a,(uint16_t)((uint32_t)a>>16),(uint16_t)b,(uint16_t)((uint32_t)b>>16))");
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
                ensure!([0x84, 0x86, 0xb2].contains(&bank), "invalid pointer bank");
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
    let globals = if dispatcher {
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
    fs::write(run.join("oracle.c"), oracle)?;
    let module = fs::canonicalize(source_path(checkout, &manifest.rust_module))?;
    let imports = &manifest.rust_imports;
    for name in imports {
        ensure!(identifier(name), "invalid Rust module import");
    }
    let alias_module = dispatcher
        && module.file_name().is_some_and(|n| n == "enemy_projectiles.rs" || n == "observed_projectile_ownership.rs");
    let mut wrapper_imports = imports.clone();
    if alias_module {
        let name = module.file_stem().unwrap().to_string_lossy().into_owned();
        if !wrapper_imports.contains(&name) {
            wrapper_imports.push(name);
        }
    }
    let memory = inputs::wrapper(inputs::WrapperRequest {
        checkout: checkout.into(),
        run: run.into(),
        modules: vec![module.clone()],
        imports: wrapper_imports,
    })?;
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
    if alias_module {
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
    fs::write(run.join("wrapper.rs"), wrapper)?;
    let root = &request.root;
    let workers = request.workers;
    let mut command = vec![
        "cc".to_owned(),
        "-O2".into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.join("src").to_string_lossy().into_owned(),
    ];
    for name in std::iter::once(bank_name).chain(support) {
        ensure!(
            safe_relative(name) && !name.contains('/') && name.starts_with("sm_") && name.ends_with(".c"),
            "invalid source bank"
        );
        let path = source.join("src").join(name).to_string_lossy().into_owned();
        if !command.contains(&path) {
            command.push(path);
        }
    }
    if dispatcher {
        command.truncate(7);
        let mut units = fs::read_dir(source.join("src"))?.map(|v| Ok(v?.path())).collect::<Result<Vec<_>>>()?;
        units.retain(|p| {
            p.extension().is_some_and(|v| v == "c")
                && p.file_stem().is_some_and(|v| {
                    v.to_string_lossy().starts_with("sm_")
                        || ["spc_player", "tracing", "util"].contains(&v.to_string_lossy().as_ref())
                })
                && (!eproj || p.file_name().is_none_or(|v| v != "sm_86.c"))
        });
        for entry in fs::read_dir(source.join("src/snes"))? {
            let p = entry?.path();
            if p.extension().is_some_and(|v| v == "c") {
                units.push(p);
            }
        }
        units.sort();
        command.extend(units.iter().map(|p| p.to_string_lossy().into_owned()));
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
        fs::write(run.join("plm-shim.c"), shim)?;
        fs::write(
            run.join("host-support.c"),
            format!("#define main retained_main\n#include {:?}\n", checkout.join("tools/decomp/headless.c")),
        )?;
        command.extend([
            run.join("plm-shim.c").to_string_lossy().into_owned(),
            run.join("host-support.c").to_string_lossy().into_owned(),
            "-lm".into(),
        ]);
    }
    command.extend([
        run.join("oracle.c").to_string_lossy().into_owned(),
        "-Wl,--gc-sections".into(),
        "-o".into(),
        run.join("c").to_string_lossy().into_owned(),
    ]);
    builds::c_build(builds::CBuildRequest {
        command,
        run: run.into(),
        checkout: checkout.into(),
        root: root.clone(),
        workers,
    })?;
    let command = vec![
        "rustc".into(),
        "--edition=2024".into(),
        "-O".into(),
        format!("--emit=link,dep-info={}", run.join("compiler.d").display()),
        run.join("wrapper.rs").to_string_lossy().into_owned(),
        "-o".into(),
        run.join("rust").to_string_lossy().into_owned(),
    ];
    builds::execute(&command, run, "rust-build", root, workers)?;
    fs::copy(&module, run.join("candidate.rs"))?;
    let dependencies = inputs::dependencies(inputs::DependenciesRequest {
        checkout: checkout.into(),
        run: run.into(),
        extra: manifest.rust_dependencies.clone(),
    })?;
    let mut harness = BTreeMap::new();
    for (name, compiled) in [
        ("family.rs", include_str!("family.rs")),
        ("batch.rs", include_str!("batch.rs")),
        ("family_header.c", include_str!("family_header.c")),
        ("family_pointer.c", include_str!("family_pointer.c")),
        ("dispatcher_pointer.c", include_str!("dispatcher_pointer.c")),
        ("dispatcher_plm.c", include_str!("dispatcher_plm.c")),
        ("dispatcher_eproj.c", include_str!("dispatcher_eproj.c")),
        ("dispatcher_host.c", include_str!("dispatcher_host.c")),
        ("plm_fixtures.rs", include_str!("plm_fixtures.rs")),
        ("cleanup.rs", include_str!("cleanup.rs")),
    ] {
        let path = format!("crates/pipeline/src/{name}");
        let hash = digest(&checkout.join(&path))?;
        ensure!(hash == format!("{:x}", Sha256::digest(compiled)), "stale family executable: {name}");
        harness.insert(path, hash);
        fs::copy(checkout.join(format!("crates/pipeline/src/{name}")), run.join(format!("pipeline-{name}")))?;
    }
    if dispatcher {
        let name = "tools/decomp/headless.c";
        harness.insert(name.to_owned(), digest(&checkout.join(name))?);
        fs::copy(checkout.join(name), run.join("pipeline-headless.c"))?;
    }
    Ok(Implementation {
        dependency_contract: "rust-family-v2".into(),
        source_root: checkout.into(),
        rust_dependencies: dependencies,
        harness_sources: harness,
        module: module.clone(),
        module_sha256: digest(&module)?,
        memory_sha256: digest(&checkout.join("crates/port/src/memory.rs"))?,
        binaries: BTreeMap::from([("c".into(), digest(&run.join("c"))?), ("rust".into(), digest(&run.join("rust"))?)]),
        ..Default::default()
    })
}

fn numeric(path: &Path) -> Result<u64> {
    let data = fs::read_to_string(path)?;
    let final_line = data.lines().last().context("missing final numeric routine return")?;
    ensure!(
        !final_line.is_empty() && final_line.bytes().all(|v| v.is_ascii_digit()),
        "missing final numeric routine return"
    );
    Ok(final_line.parse()?)
}
fn ordered_tasks(manifest: &Manifest) -> Result<Vec<(&Routine, &Case, usize)>> {
    let mut tasks = Vec::new();
    let mut names = BTreeSet::new();
    for (index, routine) in manifest.routines.iter().enumerate() {
        ensure!(!routine.cases.is_empty(), "empty declared routine coverage");
        for case in &routine.cases {
            ensure!(names.insert(case.name.as_str()), "duplicate fixture case name");
            tasks.push((routine, case, index));
        }
    }
    if let Some(order) = &manifest.case_order {
        ensure!(order.len() == tasks.len(), "case order must include every unique case exactly once");
        let original = tasks;
        tasks = Vec::new();
        let mut seen = BTreeSet::new();
        for name in order {
            let task = original.iter().find(|(_, case, _)| &case.name == name).context("unknown ordered case")?;
            ensure!(seen.insert(name.as_str()), "duplicate ordered case");
            tasks.push(*task);
        }
    }
    let mut seen = BTreeSet::new();
    for (_, case, _) in &tasks {
        if let Some(previous) = &case.continue_from {
            ensure!(seen.contains(previous.as_str()), "continuation must follow its verified predecessor");
        }
        seen.insert(case.name.as_str());
    }
    Ok(tasks)
}
fn authenticate_continuation(case: &Case, run: &Path, before: &[u8]) -> Result<()> {
    if case.continue_from.is_none() {
        return Ok(());
    }
    let previous = continuation_directory(case, run)?;
    let mut expected = fs::read(previous.join("c-after.wram"))?;
    ensure!(
        expected.len() == WRAM && expected == fs::read(previous.join("rust-after.wram"))?,
        "continuation previous WRAM differs"
    );
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

fn case(
    request: &VerifyRequest,
    manifest: &Manifest,
    routine: &Routine,
    case: &Case,
    run: &Path,
) -> Result<CaseRecord> {
    let name = &case.name;
    let symbol = &routine.symbol;
    let directory = run.join(name);
    prepare_case(case, &directory, &types(routine)?)?;
    if manifest.projectile_host {
        fs::write(directory.join("host-before.bin"), host_bytes(case, run)?)?;
    }
    let mut returns = BTreeMap::new();
    let mut hashes = BTreeMap::new();
    for subject in ["rust", "c"] {
        let mut command = vec![
            run.join(subject).to_string_lossy().into_owned(),
            symbol.clone(),
            case.a.to_string(),
            case.b.to_string(),
            directory.join("before.wram").to_string_lossy().into_owned(),
            directory.join(format!("{subject}-after.wram")).to_string_lossy().into_owned(),
        ];
        if manifest.rom.is_some() {
            command.push(run.join("rom.bin").to_string_lossy().into_owned());
        }
        if manifest.projectile_host {
            command.extend([
                directory.join("host-before.bin").to_string_lossy().into_owned(),
                directory.join(format!("{subject}-host-after.bin")).to_string_lossy().into_owned(),
            ]);
        }
        builds::execute(&command, &directory, subject, &request.root, request.workers)?;
        returns.insert(subject.into(), numeric(&directory.join(format!("{subject}.stdout")))?);
        hashes.insert(subject.into(), digest(&directory.join(format!("{subject}-after.wram")))?);
    }
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
    let c = fs::read(directory.join("c-after.wram"))?;
    let rust = fs::read(directory.join("rust-after.wram"))?;
    ensure!(c.len() == WRAM && rust.len() == WRAM, "subject did not retain full WRAM extent");
    if c != rust {
        let offset = c.iter().zip(&rust).position(|(a, b)| a != b).unwrap();
        record.first_mismatch = Some(Mismatch { offset, c: c[offset], rust: rust[offset] });
        atomic_toml(&directory.join("rejection.toml"), &record)?;
        bail!("WRAM mismatch at {offset:#x}: {symbol}/{name}");
    }
    if case.expected_after_file.is_some() {
        ensure!(c == fs::read(directory.join("expected-after.wram"))?, "captured expected WRAM mismatch: {name}");
        record.capture_after_sha256 = Some(digest(&directory.join("expected-after.wram"))?);
    }
    if manifest.projectile_host {
        authenticate_host(&directory, case)?;
    }
    Ok(record)
}

fn inventory(run: &Path, base: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    let mut paths = fs::read_dir(base)?.map(|v| Ok(v?.path())).collect::<Result<Vec<_>>>()?;
    paths.sort();
    for path in paths {
        ensure!(!path.is_symlink(), "run artifact symlink");
        if path.is_dir() {
            inventory(run, &path, result)?;
        } else if path != run.join("verdict.toml")
            && !path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml"))
        {
            result.insert(path.strip_prefix(run)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}

pub fn verify(request: VerifyRequest) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let source = fs::canonicalize(&request.source)?;
    let output = request.output.clone();
    fs::create_dir_all(&output)?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros();
    let run = output.join(format!("{stamp}-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut metrics = crate::metrics::Run::start(run.join("family.metrics.toml"), "family")?;
    metrics.context.configuration("manifest", request.manifest.display());
    metrics.context.configuration("workers", request.workers);
    let mut verdict = Verdict {
        status: "rejected".into(),
        run: run.clone(),
        original_rom_parity: "not-tested".into(),
        ..Default::default()
    };
    let result = (|| -> Result<()> {
        let manifest_path = source_path(&checkout, request.manifest.to_str().context("non-UTF8 manifest")?);
        let manifest_bytes = fs::read(&manifest_path)?;
        fs::write(run.join("manifest.toml"), &manifest_bytes)?;
        let mut manifest: Manifest = toml::from_str(std::str::from_utf8(&manifest_bytes)?)?;
        let callbacks = request.adapter == "callback" || request.adapter == "power-bomb";
        let plm = request.adapter == "plm";
        if plm {
            manifest = crate::plm_spawn::normalize(&manifest)?;
        }
        verdict.family = manifest.family.clone();
        verdict.source = Some(verify_source(&checkout, &source)?);
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        let tasks = ordered_tasks(&manifest)?;
        ensure!(!tasks.is_empty(), "empty family cannot be verified");
        let jobs =
            if tasks.iter().any(|(_, case, _)| case.continue_from.is_some()) { 1 } else { request.case_jobs.max(1) };
        ensure!((1..=64).contains(&jobs), "case jobs must be 1..64");
        if let Some(rom) = manifest.rom.as_ref() {
            let bytes = asset(&rom.file, &rom.sha256, None)?;
            ensure!(!bytes.is_empty() && bytes.len() <= 0x400000, "ROM extent outside LoROM view");
            fs::write(run.join("rom.bin"), &bytes)?;
            verdict.rom = Some(RomIdentity { sha256: digest(&run.join("rom.bin"))?, bytes: bytes.len() });
        }
        verdict.implementation = Some(if plm {
            crate::plm_spawn::build(&request, &manifest, &checkout, &source, &run)?
        } else if callbacks {
            callback::build(&request, &manifest, &checkout, &source, &run)?
        } else {
            build(&request, &manifest, &checkout, &source, &run)?
        });
        verdict.implementation.as_mut().unwrap().manifest =
            Some(Asset { file: fs::canonicalize(&manifest_path)?, sha256: digest(&run.join("manifest.toml"))? });
        atomic_toml(
            &run.join("manifest-input.toml"),
            verdict.implementation.as_ref().unwrap().manifest.as_ref().unwrap(),
        )?;
        if callbacks {
            verdict.gameplay_translation = Some(!manifest.infrastructure_probe);
        }
        let next = AtomicUsize::new(0);
        let outcomes = Mutex::new(Vec::new());
        let context = metrics.context.clone();
        std::thread::scope(|scope| {
            for _ in 0..jobs.min(tasks.len()) {
                scope.spawn(|| {
                    let _entered = context.enter();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some((routine, fixture, routine_index)) = tasks.get(index) else {
                            break;
                        };
                        let outcome = if plm {
                            crate::plm_spawn::case(&request, routine, fixture, &run)
                        } else if callbacks {
                            callback::case(&request, &manifest, routine, fixture, *routine_index, &run)
                        } else {
                            case(&request, &manifest, routine, fixture, &run)
                        };
                        outcomes.lock().unwrap().push((index, outcome));
                    }
                });
            }
        });
        let mut outcomes = outcomes.into_inner().unwrap();
        outcomes.sort_by_key(|(index, _)| *index);
        let mut errors = Vec::new();
        for (index, outcome) in outcomes {
            match outcome {
                Ok(record) => verdict.cases.push(record),
                Err(error) => {
                    let rejection = run.join(&tasks[index].1.name).join("rejection.toml");
                    if rejection.is_file() {
                        verdict.cases.push(object(&rejection)?);
                    }
                    errors.push(format!("{error:#}"));
                }
            }
        }
        ensure!(errors.is_empty(), "{}", errors.join("; "));
        ensure!(
            digest(&manifest_path)? == digest(&run.join("manifest.toml"))?,
            "family manifest changed during verification"
        );
        verdict.status = "offline-verified".into();
        verdict.verified_cases = tasks.len();
        verdict.verified_routines = manifest.routines.len();
        if plm {
            verdict.rejected_abi_inputs = 5;
            verdict.native_rust_spawn_calls = tasks.len();
            verdict.remaining_c_gameplay_callbacks = 0;
            let counts = manifest.routines.iter().map(|r| (r.symbol.clone(), r.cases.len())).collect();
            verdict.allocator_cases = counts;
        }
        verdict.scope = "declared valid fixture domain only; full canonical WRAM and return values".into();
        Ok(())
    })();
    if let Err(error) = result {
        verdict.error = Some(format!("{error:#}"));
    }
    let mut artifacts = BTreeMap::new();
    inventory(&run, &run, &mut artifacts)?;
    verdict.artifacts = artifacts;
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    metrics.finish(crate::metrics::outcome(verdict.error.as_deref()), verdict.error.clone())?;
    Ok(verdict)
}

pub fn authenticate(request: AuthenticateRequest) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let path = &request.verdict;
    let run = fs::canonicalize(path.parent().context("verdict parent missing")?)?;
    let verdict: Verdict = object(path)?;
    ensure!(verdict.status == "offline-verified", "family evidence is not offline verification");
    let mut actual = BTreeMap::new();
    inventory(&run, &run, &mut actual)?;
    ensure!(actual == verdict.artifacts, "family artifact inventory differs");
    for required in
        ["manifest.toml", "candidate.rs", "memory.rs", "oracle.c", "wrapper.rs", "compiler.d", "pipeline-inputs.toml"]
    {
        ensure!(actual.contains_key(required), "family artifact missing: {required}");
    }
    let mut manifest: Manifest = object(&run.join("manifest.toml"))?;
    ensure!(manifest.family == verdict.family, "family identity differs");
    let implementation = verdict.implementation.as_ref().context("missing family implementation")?;
    let manifest_identity = implementation.manifest.as_ref().context("current family manifest seal missing")?;
    ensure!(actual.contains_key("manifest-input.toml"), "current source inputs manifest seal missing");
    let input_manifest: Asset = object(&run.join("manifest-input.toml"))?;
    ensure!(input_manifest == *manifest_identity, "family manifest source input differs");
    ensure!(
        digest(&manifest_identity.file)? == manifest_identity.sha256
            && digest(&run.join("manifest.toml"))? == manifest_identity.sha256,
        "family manifest changed"
    );
    let callbacks = implementation.dependency_contract == "rust-callback-v2";
    let plm = implementation.dependency_contract == "rust-plm-v2";
    if plm {
        manifest = crate::plm_spawn::normalize(&manifest)?;
    }
    ordered_tasks(&manifest)?;
    ensure!(
        callbacks || plm || implementation.dependency_contract == "rust-family-v2",
        "unsupported family dependency contract"
    );
    let binaries = if plm {
        vec!["spawn", "libplm.a"]
    } else if callbacks {
        vec!["collision", "libcollision.a"]
    } else {
        vec!["c", "rust"]
    };
    for name in &binaries {
        ensure!(actual.contains_key(*name), "family binary artifact missing: {name}");
    }
    let source = verify_source(&checkout, &checkout.join("deps/snesrev-sm"))?;
    ensure!(Some(source) == verdict.source, "family pinned source differs");
    for map in [&implementation.rust_dependencies, &implementation.harness_sources] {
        for (name, expected) in map {
            ensure!(digest(&source_path(&checkout, name))? == *expected, "family implementation changed: {name}");
        }
    }
    for required in ["main.rs", "inputs.rs", "builds.rs"] {
        ensure!(
            implementation.rust_dependencies.contains_key(&format!("crates/pipeline/src/{required}")),
            "pipeline input missing: {required}"
        );
    }
    let required_harness = if plm {
        vec!["family.rs", "batch.rs", "cleanup.rs", "plm_spawn.rs", "plm_spawn_abi.rs.in", "plm_spawn_host.c.in"]
    } else if callbacks {
        vec![
            "family.rs",
            "batch.rs",
            "cleanup.rs",
            "callback.rs",
            "callback_abi.rs.in",
            "callback_host.c.in",
            "power_bomb_abi.rs.in",
            "power_bomb_host.c.in",
        ]
    } else {
        vec![
            "family.rs",
            "batch.rs",
            "cleanup.rs",
            "family_header.c",
            "family_pointer.c",
            "dispatcher_pointer.c",
            "dispatcher_plm.c",
            "dispatcher_eproj.c",
            "dispatcher_host.c",
            "plm_fixtures.rs",
        ]
    };
    for required in required_harness {
        ensure!(
            implementation.harness_sources.contains_key(&format!("crates/pipeline/src/{required}")),
            "family input missing: {required}"
        );
    }
    if plm {
        ensure!(
            actual.contains_key("pipeline-headless.c")
                && implementation.harness_sources.contains_key("tools/decomp/headless.c"),
            "PLM host input missing"
        );
        ensure!(
            verdict.rejected_abi_inputs == 5
                && fs::read_to_string(run.join("abi-controls.stdout"))?.lines().last() == Some("5 rejected ABI inputs"),
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
        if implementation.callback_adapter.as_deref() == Some("power-bomb") {
            let original = fs::read_to_string(checkout.join("deps/snesrev-sm/src/sm_88.c"))?;
            let token = "uint16 SpawnHdmaObject(uint8 db, const void *p)";
            ensure!(
                original.matches(token).count() == 1
                    && fs::read_to_string(run.join("sm_88-allocation-seam.c"))?
                        == original.replace(token, "uint16 PortActualSpawnHdmaObject(uint8 db, const void *p)"),
                "power bomb allocation seam changed"
            );
        }
        ensure!(
            verdict.gameplay_translation == Some(!manifest.infrastructure_probe),
            "callback gameplay scope differs"
        );
    } else if dispatcher(&manifest) {
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
        ensure!(digest(&run.join(binary))? == implementation.binaries[binary], "family binary changed");
    }
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
        if plm {
            ensure!(
                crate::plm_spawn::check(&directory, fixture)?.spawn_results == record.spawn_results,
                "retained PLM result differs"
            );
            continue;
        }
        if callbacks {
            let checked = callback::check(&directory, fixture)?;
            ensure!(
                checked.results == record.results && checked.callbacks == record.callbacks,
                "retained callback result differs"
            );
            continue;
        }
        let before = fs::read(directory.join("before.wram"))?;
        authenticate_continuation(fixture, &run, &before)?;
        let c = fs::read(directory.join("c-after.wram"))?;
        let rust = fs::read(directory.join("rust-after.wram"))?;
        ensure!(
            before.len() == WRAM && c.len() == WRAM && rust.len() == WRAM && c == rust,
            "family actual full WRAM outputs differ"
        );
        ensure!(
            digest(&directory.join("before.wram"))? == record.before_sha256
                && digest(&directory.join("c-after.wram"))? == record.after_sha256["c"]
                && digest(&directory.join("rust-after.wram"))? == record.after_sha256["rust"],
            "family case hashes differ"
        );
        let a = numeric(&directory.join("c.stdout"))?;
        let b = numeric(&directory.join("rust.stdout"))?;
        ensure!(
            a == b && record.returns["c"] == a && record.returns["rust"] == b,
            "family actual return outputs differ"
        );
        if let Some(value) = fixture.expected_return {
            ensure!(value == a, "family expected return differs");
        }
        if fixture.expected_after_file.as_ref().is_some() {
            ensure!(c == fs::read(directory.join("expected-after.wram"))?, "family actual captured output differs");
        }
        if manifest.projectile_host {
            authenticate_host(&directory, fixture)?;
        }
    }
    ensure!(
        !observed.is_empty()
            && observed.len() == expected.len()
            && verdict.verified_cases == observed.len()
            && verdict.verified_routines == manifest.routines.len(),
        "family coverage incomplete"
    );
    Ok(verdict)
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
        assert!(dispatcher(&manifest));
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
        let directory = std::env::temp_dir().join(format!(
            "sm-continuation-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
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
            writes: vec![Write { offset: 42, data: vec![99] }],
            ..Default::default()
        };
        prepare_case(&fixture, &directory.join("next"), &[]).unwrap();
        let inherited = fs::read(directory.join("next/before.wram")).unwrap();
        assert_eq!(inherited[42], 99);
        assert_eq!(inherited[41], 3);
        authenticate_continuation(&fixture, &directory, &inherited).unwrap();
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
        assert!(authenticate_continuation(&fixture, &directory, &inherited).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
