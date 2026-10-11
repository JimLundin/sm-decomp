//! Source-authenticated C trajectory counters; diagnostic output never admits a Rust port.
use crate::{atomic_toml, builds, digest, family};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
const SUPPORT: &str = r#"
#include <stdio.h>
static unsigned long inventory_hits[4][65536];
static int inventory_first[4][65536], inventory_last[4][65536];
static void InventoryHit(unsigned kind,uint16 address) {
 if(!inventory_hits[kind][address])inventory_first[kind][address]=snes_frame_counter;
 inventory_hits[kind][address]++;inventory_last[kind][address]=snes_frame_counter;
}
void InventorySave(void) {
 FILE *f=fopen("projectiles.tsv","w");if(!f)exit(2);
 fprintf(f,"kind\taddress\thits\tfirst_frame\tlast_frame\n");
 const char *names[]={"id","init","pre","instruction"};
 for(unsigned kind=0;kind<4;kind++)for(unsigned i=0;i<65536;i++)if(inventory_hits[kind][i])
 fprintf(f,"%s\t%04x\t%lu\t%d\t%d\n",names[kind],i,inventory_hits[kind][i],inventory_first[kind][i],inventory_last[kind][i]);
 if(fclose(f))exit(2);
}
"#;
fn instrument(source: &str, host: &str) -> Result<(String, String)> {
    let mut source = source.to_owned();
    for (anchor, statement) in [
        ("static void EprojRunOne(uint16 k) {", "InventoryHit(0,eproj_id[k>>1]);"),
        ("static void CallEprojInit(uint32 ea, uint16 j) {", "InventoryHit(1,(uint16)ea);"),
        ("static void CallEprojPreInstr(uint32 ea, uint16 k) {", "InventoryHit(2,(uint16)ea);"),
        ("static const uint8 *CallEprojInstr(uint32 ea, uint16 k, const uint8 *j) {", "InventoryHit(3,(uint16)ea);"),
    ] {
        ensure!(source.matches(anchor).count() == 1, "projectile instrumentation anchor must be unique: {anchor}");
        source = source.replace(anchor, &format!("{anchor}\n  {statement}"));
    }
    let point = source.find("static void CallEprojPreInstr").context("projectile support insertion anchor missing")?;
    source.insert_str(point, SUPPORT);
    let mut host = host.to_owned();
    let start = host.find("static void SaveRoutineState").context("routine capture anchor missing")?;
    let end = start + host[start..].find("// Linker wrapping").context("linker anchor missing")?;
    host.replace_range(start..end, "");
    let start = host.find("void __wrap_DecompressToMem").context("decompress wrapper missing")?;
    let end = start + host[start..].find("// The host has no audio thread").context("audio anchor missing")?;
    host.replace_range(start..end,"void __wrap_DecompressToMem(uint32 src,uint8 *dst) {\n decompression_calls++;__real_DecompressToMem(src,dst);\n}\nextern void InventorySave(void);\n");
    let anchor = "  fclose(inputs);\n  return 0;";
    ensure!(host.matches(anchor).count() == 1, "inventory save anchor must be unique");
    host = host.replace(anchor, "  fclose(inputs);\n  InventorySave();\n  return 0;");
    Ok((source, host))
}
fn artifacts(root: &Path, directory: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "inventory evidence symlink");
        if path.is_dir() {
            artifacts(root, &path, result)?;
        } else {
            result.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
fn snapshot(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "original C snapshot symlink");
        let target = destination.join(path.file_name().context("source filename missing")?);
        if path.is_dir() {
            snapshot(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
            ensure!(digest(&path)? == digest(&target)?, "original C input changed while preserving snapshot");
        }
    }
    Ok(())
}
pub fn operate(request: Value) -> Result<Value> {
    let checkout = fs::canonicalize(
        request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?),
    )?;
    let frames = u64::try_from(request.get("frames").and_then(Value::as_integer).unwrap_or(129712))?;
    ensure!((1..=129712).contains(&frames), "inventory frames must be 1..129712");
    let output = request
        .get("output")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| checkout.join("out/port/projectile-inventory"));
    fs::create_dir_all(&output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut verdict = crate::report!({"schema_version":2,"status":"diagnostic-failed","accepted":false,"frames":frames,"live_exercised":false,"scope":"fresh C trajectory; counters only; no Rust replacement/parity claim","evidence":run});
    let result = (|| -> Result<()> {
        let source = checkout.join("deps/snesrev-sm");
        let identity = family::verify_source(&checkout, &source)?;
        verdict.as_table_mut().unwrap().insert("source".into(), Value::try_from(identity)?);
        let rom = checkout.join("rom/sm.sfc");
        let inputs = checkout.join("movies/super-metroid-any-3653.inputs");
        let rom_hash = digest(&rom)?;
        let input_hash = digest(&inputs)?;
        ensure!(
            rom_hash == "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72",
            "unsupported inventory ROM identity"
        );
        let src = source.join("src");
        snapshot(&src, &run.join("original-source"))?;
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        let original = src.join("sm_86.c");
        let host = checkout.join("tools/decomp/headless.c");
        let source_hash = digest(&original)?;
        let host_hash = digest(&host)?;
        let (code, host_code) = instrument(&fs::read_to_string(&original)?, &fs::read_to_string(&host)?)?;
        fs::write(run.join("sm_86.c"), code)?;
        fs::write(run.join("headless.c"), host_code)?;
        fs::copy(checkout.join("tools/decomp/headless_frame.h"), run.join("headless_frame.h"))?;
        let common = std::process::Command::new("git")
            .args(["rev-parse", "--git-common-dir"])
            .current_dir(&checkout)
            .output()?;
        ensure!(common.status.success(), "Git common directory missing");
        let common = fs::canonicalize(checkout.join(String::from_utf8(common.stdout)?.trim()))?;
        let shared = std::env::var_os("SM_PIPELINE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| common.parent().unwrap().join("out/pipeline"));
        let workers = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse()?;
        let units = family::c_units(&source)?;
        let mut command = vec![
            "cc".into(),
            "-O2".into(),
            "-fno-strict-aliasing".into(),
            "-ffunction-sections".into(),
            "-fdata-sections".into(),
            "-iquote".into(),
            src.to_string_lossy().into_owned(),
        ];
        command.extend(
            units
                .iter()
                .map(|p| if p.file_name().is_some_and(|n| n == "sm_86.c") { run.join("sm_86.c") } else { p.clone() })
                .map(|p| p.to_string_lossy().into_owned()),
        );
        command.extend([
            run.join("headless.c").to_string_lossy().into_owned(),
            "-Wl,--gc-sections".into(),
            "-Wl,--wrap=Call".into(),
            "-Wl,--wrap=cpu_runOpcode".into(),
            "-Wl,--wrap=DecompressToMem".into(),
            "-lm".into(),
            "-o".into(),
            run.join("inventory").to_string_lossy().into_owned(),
        ]);
        verdict.as_table_mut().unwrap().insert("build_command".into(), crate::report!(command));
        verdict.as_table_mut().unwrap().insert(
            "build_cache".into(),
            Value::try_from(builds::c_build(builds::CBuildRequest {
                checkout: checkout.clone(),
                root: shared.clone(),
                workers,
                run: run.clone(),
                command,
            })?)?,
        );
        let replay = vec![
            run.join("inventory").to_string_lossy().into_owned(),
            rom.to_string_lossy().into_owned(),
            inputs.to_string_lossy().into_owned(),
            frames.to_string(),
            run.join("terminal.bin").to_string_lossy().into_owned(),
        ];
        verdict.as_table_mut().unwrap().insert("replay_command".into(), crate::report!(replay));
        builds::execute_timeout(&replay, &run, "replay", &shared, workers, 300)?;
        ensure!(
            digest(&original)? == source_hash
                && digest(&host)? == host_hash
                && digest(&rom)? == rom_hash
                && digest(&inputs)? == input_hash,
            "inventory inputs changed during execution"
        );
        family::verify_source(&checkout, &source)?;
        let table = fs::read_to_string(run.join("projectiles.tsv"))?;
        ensure!(table.starts_with("kind\taddress\thits\tfirst_frame\tlast_frame\n"), "inventory table missing");
        for line in table.lines().skip(1) {
            let columns = line.split('\t').collect::<Vec<_>>();
            ensure!(
                columns.len() == 5
                    && ["id", "init", "pre", "instruction"].contains(&columns[0])
                    && u16::from_str_radix(columns[1], 16).is_ok()
                    && columns[2].parse::<u64>()? > 0,
                "malformed inventory counter"
            );
            let first = columns[3].parse::<u64>()?;
            let last = columns[4].parse::<u64>()?;
            ensure!(first <= last && last < frames, "inventory frame bounds differ");
        }
        let mut hashes = BTreeMap::new();
        artifacts(&run, &run, &mut hashes)?;
        verdict.as_table_mut().unwrap().insert("artifacts".into(), crate::report!(hashes));
        verdict.as_table_mut().unwrap().insert("rom_sha256".into(), rom_hash.into());
        verdict.as_table_mut().unwrap().insert("inputs_sha256".into(), input_hash.into());
        verdict.as_table_mut().unwrap().insert("original_sm86_sha256".into(), source_hash.into());
        verdict.as_table_mut().unwrap().insert("host_sha256".into(), host_hash.into());
        verdict.as_table_mut().unwrap().insert("status".into(), "diagnostic-complete".into());
        Ok(())
    })();
    if let Err(error) = result {
        verdict.as_table_mut().unwrap().insert("error".into(), format!("{error:#}").into());
    }
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    Ok(verdict)
}
