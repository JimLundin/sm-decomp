//! Passive actual-C enemy instruction observations, sealed independently of Rust admission.
use crate::{atomic_toml, builds, digest, family};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use toml::Value;

const SUPPORT: &str = include_str!("enemy_instruction_inventory.c.in");
const HOST_SAVE: &str = r#"
void InventoryProjectileHostSave(uint16 *v) {
  v[0]=eproj_spawn_pt.x;v[1]=eproj_spawn_pt.y;v[2]=eproj_spawn_rect.x;v[3]=eproj_spawn_rect.y;
  v[4]=eproj_spawn_rect.w;v[5]=eproj_spawn_rect.h;v[6]=eproj_spawn_r22;v[7]=eproj_spawn_varE24;
}
"#;
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const INPUT_SHA256: &str = "cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e";

pub(crate) fn instrument(a0: &str, bank86: &str, host: &str) -> Result<(String, String, String)> {
    let mut a0 = a0.to_owned();
    for (anchor, renamed) in [
        ("void ProcessEnemyInstructions(void) {", "void CInventory_ProcessEnemyInstructions(void) {"),
        (
            "const uint16 *CallEnemyInstr(uint32 ea, uint16 k, const uint16 *j) {",
            "const uint16 *CInventory_CallEnemyInstr(uint32 ea, uint16 k, const uint16 *j) {",
        ),
    ] {
        ensure!(a0.matches(anchor).count() == 1, "enemy inventory definition anchor must be unique: {anchor}");
        a0 = a0.replace(anchor, renamed);
    }
    let include = "#include \"enemy_types.h\"\n";
    ensure!(a0.matches(include).count() == 1, "enemy types include must be unique");
    a0 = a0.replace(include, &format!("{include}{SUPPORT}"));
    let mut bank86 = bank86.to_owned();
    bank86.push_str(HOST_SAVE);
    let mut host = host.to_owned();
    let start = host.find("static void SaveRoutineState").context("routine capture anchor missing")?;
    let end = start + host[start..].find("// Linker wrapping").context("linker wrapping anchor missing")?;
    host.replace_range(start..end, "");
    let start = host.find("void __wrap_DecompressToMem").context("decompress wrapper missing")?;
    let end = start + host[start..].find("// The host has no audio thread").context("audio anchor missing")?;
    host.replace_range(
        start..end,
        "void __wrap_DecompressToMem(uint32 src,uint8 *dst) {\n  decompression_calls++;__real_DecompressToMem(src,dst);\n}\nextern void EnemyInventorySave(void);\n",
    );
    let anchor = "  fclose(inputs);\n  return 0;";
    ensure!(host.matches(anchor).count() == 1, "inventory completion anchor must be unique");
    host = host.replace(anchor, "  fclose(inputs);\n  EnemyInventorySave();\n  return 0;");
    Ok((a0, bank86, host))
}

fn snapshot(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "inventory source snapshot contains a symlink");
        let target = destination.join(path.file_name().context("source filename missing")?);
        if path.is_dir() {
            snapshot(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
            ensure!(digest(&path)? == digest(&target)?, "source changed while preserving inventory snapshot");
        }
    }
    Ok(())
}
fn artifacts(root: &Path, directory: &Path, hashes: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "inventory artifact symlink");
        if path.is_dir() {
            artifacts(root, &path, hashes)?;
        } else {
            hashes.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
fn build_command(source: &Path, run: &Path) -> Result<Vec<String>> {
    let src = source.join("src");
    let mut command = vec![
        "cc".into(),
        "-O2".into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        src.to_string_lossy().into_owned(),
    ];
    command.extend(family::c_units(source)?.into_iter().map(|unit| {
        if unit.file_name().is_some_and(|name| name == "sm_a0.c" || name == "sm_86.c") {
            run.join(unit.file_name().unwrap()).to_string_lossy().into_owned()
        } else {
            unit.to_string_lossy().into_owned()
        }
    }));
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
    Ok(command)
}
fn validate_counters(table: &str, frames: u64) -> Result<usize> {
    ensure!(
        table.starts_with("kind\taddress\twindow\thits\tfirst_frame\tlast_frame\n"),
        "enemy inventory table missing"
    );
    let mut rows = 0;
    for line in table.lines().skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        ensure!(fields.len() == 6, "invalid enemy inventory row extent");
        let valid_address = match fields[0] {
            "frame-bank" => fields[2] == "all" && u8::from_str_radix(fields[1], 16).is_ok(),
            "opcode" => {
                fields[1].len() == 6
                    && fields[1].starts_with("a6")
                    && u16::from_str_radix(&fields[1][2..], 16).is_ok()
                    && matches!(fields[2], "0" | "1" | "2")
            }
            _ => false,
        };
        ensure!(valid_address && fields[3].parse::<u64>()? > 0, "invalid enemy inventory counter");
        let first = fields[4].parse::<u64>()?;
        let last = fields[5].parse::<u64>()?;
        ensure!(first <= last && last < frames, "enemy inventory frame bounds differ");
        rows += 1;
    }
    Ok(rows)
}

pub fn operate(request: Value) -> Result<Value> {
    let checkout = fs::canonicalize(
        request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?),
    )?;
    let source = fs::canonicalize(
        request
            .get("source")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_else(|| checkout.join("deps/snesrev-sm")),
    )?;
    let assets = fs::canonicalize(
        request.get("asset_root").and_then(Value::as_str).map(PathBuf::from).unwrap_or_else(|| checkout.clone()),
    )?;
    let frames = u64::try_from(request.get("frames").and_then(Value::as_integer).unwrap_or(129712))?;
    let timeout = u64::try_from(request.get("timeout").and_then(Value::as_integer).unwrap_or(60))?;
    ensure!((1..=129712).contains(&frames), "inventory frames must be 1..129712");
    ensure!((1..=3600).contains(&timeout), "inventory timeout must be 1..3600 seconds");
    let output = request
        .get("output")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| assets.join("out/port/ridley-instructions/inventory"));
    fs::create_dir_all(&output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    fs::create_dir(run.join("fixtures"))?;
    let run = fs::canonicalize(run)?;
    let started = Instant::now();
    let mut verdict = crate::report!({
        "schema_version":1,"status":"diagnostic-failed","accepted":false,
        "live_exercised":false,"frames":frames,"evidence":run,
        "scope":"passive pinned actual-C native enemy frames/opcodes; no Rust or original-ROM parity claim",
        "capture_windows":["prefix<36000","first-combat36000..49999","later-combat>=50000"]
    });
    let result = (|| -> Result<()> {
        let identity = family::verify_source(&checkout, &source)?;
        for unit in family::c_units(&source)? {
            ensure!(
                identity.files.get(&unit.strip_prefix(&source)?.to_string_lossy().into_owned())
                    == Some(&digest(&unit)?),
                "native inventory C unit is outside pinned source identity"
            );
        }
        verdict.as_table_mut().unwrap().insert("source".into(), Value::try_from(identity)?);
        let rom = assets.join("rom/sm.sfc");
        let inputs = assets.join("movies/super-metroid-any-3653.inputs");
        ensure!(digest(&rom)? == ROM_SHA256 && digest(&inputs)? == INPUT_SHA256, "native inventory assets differ");
        snapshot(&source.join("src"), &run.join("original-source"))?;
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        let originals =
            [source.join("src/sm_a0.c"), source.join("src/sm_86.c"), checkout.join("tools/decomp/headless.c")];
        let input_hashes = originals
            .iter()
            .map(|p| Ok((p.to_string_lossy().into_owned(), digest(p)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let (a0, bank86, host) = instrument(
            &fs::read_to_string(&originals[0])?,
            &fs::read_to_string(&originals[1])?,
            &fs::read_to_string(&originals[2])?,
        )?;
        fs::write(run.join("sm_a0.c"), a0)?;
        fs::write(run.join("sm_86.c"), bank86)?;
        fs::write(run.join("headless.c"), host)?;
        fs::write(run.join("instrumentation.c.in"), SUPPORT)?;
        fs::copy(checkout.join("crates/pipeline/src/enemy_instruction_inventory.rs"), run.join("producer.rs"))?;
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
        let command = build_command(&source, &run)?;
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
        let replay_record = builds::execute_timeout(&replay, &run, "replay", &shared, workers, timeout)?;
        verdict.as_table_mut().unwrap().insert("replay".into(), Value::try_from(replay_record)?);
        for (path, expected) in &input_hashes {
            ensure!(digest(Path::new(path))? == *expected, "native inventory source changed during execution");
        }
        ensure!(
            digest(&rom)? == ROM_SHA256 && digest(&inputs)? == INPUT_SHA256,
            "native inventory assets changed during execution"
        );
        family::verify_source(&checkout, &source)?;
        let rows = validate_counters(&fs::read_to_string(run.join("enemy-instructions.tsv"))?, frames)?;
        let progress = fs::read_to_string(run.join("replay.stdout"))?;
        let terminal = progress
            .lines()
            .filter(|l| l.starts_with("progress "))
            .next_back()
            .context("inventory has no terminal progress")?
            .split_whitespace()
            .collect::<Vec<_>>();
        ensure!(
            terminal.len() == 10 && terminal[1].parse::<u64>()? == frames,
            "inventory replay did not complete requested inputs"
        );
        ensure!(terminal[7] == "0" && terminal[8] == "0", "native inventory used CPU fallback");
        verdict.as_table_mut().unwrap().insert("counter_rows".into(), crate::report!(rows));
        verdict.as_table_mut().unwrap().insert("input_hashes".into(), crate::report!(input_hashes));
        verdict.as_table_mut().unwrap().insert("rom_sha256".into(), ROM_SHA256.into());
        verdict.as_table_mut().unwrap().insert("inputs_sha256".into(), INPUT_SHA256.into());
        verdict.as_table_mut().unwrap().insert("status".into(), "diagnostic-complete".into());
        Ok(())
    })();
    if let Err(error) = result {
        verdict.as_table_mut().unwrap().insert("error".into(), format!("{error:#}").into());
    }
    // Failed and interrupted experiments retain their exact partial observations too.
    let mut hashes = BTreeMap::new();
    artifacts(&run, &run, &mut hashes)?;
    verdict.as_table_mut().unwrap().insert("artifacts".into(), crate::report!(hashes));
    verdict.as_table_mut().unwrap().insert("seconds".into(), started.elapsed().as_secs_f64().into());
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instrumentation_preserves_call_sites_and_rejects_duplicate_definitions() {
        let a0 = "#include \"enemy_types.h\"\nvoid ProcessEnemyInstructions(void) { CallEnemyInstr(0,0,0); }\nconst uint16 *CallEnemyInstr(uint32 ea, uint16 k, const uint16 *j) { ProcessEnemyInstructions(); return j; }\n";
        let host = "static void SaveRoutineState() {}\n// Linker wrapping\nvoid __wrap_DecompressToMem() {}\n// The host has no audio thread\nint main(){\n  fclose(inputs);\n  return 0;}";
        let (transformed, bank86, headless) = instrument(a0, "bank86", host).unwrap();
        assert!(transformed.contains("CInventory_ProcessEnemyInstructions(void) { CallEnemyInstr(0,0,0);"));
        assert!(
            transformed.contains(
                "CInventory_CallEnemyInstr(uint32 ea, uint16 k, const uint16 *j) { ProcessEnemyInstructions();"
            )
        );
        assert!(bank86.contains("v[7]=eproj_spawn_varE24"));
        assert!(headless.contains("EnemyInventorySave();"));
        assert!(!headless.contains("SaveRoutineState"));
        assert!(instrument(&format!("{a0}{a0}"), "bank86", host).is_err());
    }
    #[test]
    fn counters_validate_native_banks_windows_and_frame_bounds() {
        let header = "kind\taddress\twindow\thits\tfirst_frame\tlast_frame\n";
        assert_eq!(
            validate_counters(
                &format!("{header}frame-bank\ta6\tall\t2\t0\t1\nopcode\ta6e84d\t1\t8\t36000\t40000\n"),
                45000
            )
            .unwrap(),
            2
        );
        for row in [
            "opcode\ta6e84d\t3\t8\t0\t1",
            "opcode\ta6e84d\t1\t0\t0\t1",
            "frame-bank\ta6\tall\t2\t2\t1",
            "opcode\ta6e84d\t1\t8\t0\t45000",
        ] {
            assert!(validate_counters(&format!("{header}{row}\n"), 45000).is_err());
        }
    }
}
