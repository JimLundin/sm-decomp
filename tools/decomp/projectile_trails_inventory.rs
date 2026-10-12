//! Standalone Rust producer for actual-ROM trail fixtures and natural C frames.
//!
//! Build with `rustc --edition=2024 -O tools/decomp/projectile_trails_inventory.rs
//! -o /tmp/sm-projectile-trails-producer`. All compiler/replay processes use the
//! existing sm-pipeline worker scheduler. No diagnostic admits a Rust replacement.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROM_SHA: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const INPUT_SHA: &str = "cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e";
const SOURCE_SHA: &str = "ad9f103509160abd8a9e0f85f3c3c96187c89f31dd770cf533fa32a97925247a";
const ARCHIVE_SHA: &str = "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193";
const TEMPLATE: &str = include_str!("projectile_trails_inventory.c.in");

fn require(condition: bool, message: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(message.into()) }
}
fn quoted(value: impl AsRef<str>) -> String {
    let mut text = String::from("\"");
    for c in value.as_ref().chars() {
        match c {
            '\\' => text.push_str("\\\\"),
            '"' => text.push_str("\\\""),
            '\n' => text.push_str("\\n"),
            '\r' => text.push_str("\\r"),
            '\t' => text.push_str("\\t"),
            _ => text.push(c),
        }
    }
    text.push('"');
    text
}
fn digest(path: &Path) -> Result<String> {
    let output = Command::new("sha256sum").arg("--").arg(path).output()?;
    require(output.status.success(), "SHA256 command failed")?;
    let text = String::from_utf8(output.stdout)?;
    let hash = text.get(..64).ok_or("SHA256 output malformed")?;
    require(hash.chars().all(|c| c.is_ascii_hexdigit()), "SHA256 output malformed")?;
    Ok(hash.to_owned())
}
fn array(values: &[String]) -> String {
    format!("[{}]", values.iter().map(quoted).collect::<Vec<_>>().join(", "))
}
fn word(rom: &[u8], pointer: u16) -> Result<u16> {
    require(pointer >= 0x8000, "script outside LoROM bank90")?;
    let offset = 0x80000 + usize::from(pointer & 0x7fff);
    let data = rom.get(offset..offset + 2).ok_or("script exceeds ROM extent")?;
    Ok(u16::from_le_bytes([data[0], data[1]]))
}
fn scripts(rom: &[u8]) -> Result<Vec<u16>> {
    let mut scripts = BTreeSet::new();
    for table in [0xb5bbu16, 0xb609] {
        for index in 0..34 {
            scripts.insert(word(rom, table + index * 2)?);
        }
    }
    for &start in &scripts {
        let mut pointer = start;
        for _ in 0..64 {
            let token = word(rom, pointer)?;
            if token == 0 {
                break;
            }
            require(token & 0x8000 == 0 || [0xb525, 0xb587, 0xb5b3].contains(&token), "native trail opcode unknown")?;
            pointer = pointer.checked_add(if token & 0x8000 == 0 { 4 } else { 2 }).ok_or("native script wraps")?;
        }
        require(word(rom, pointer)? == 0, "native trail termination missing")?;
    }
    Ok(scripts.into_iter().collect())
}

fn header(module: &Path, rom: &Path, rom_sha: &str) -> String {
    format!(
        "family = \"projectile-trails\"\nrust_module = {}\nbank = \"sm_90.c\"\nc_support = [\"sm_rtl.c\"]\ndomain = \"Native18-slot reverse trail frames, actual ROM scripts, full WRAM/raw return; odd direct opcode slots floor. All timer and camera widths, frozen frames, native partial OAM word writes and WRAM aliases. Invalid or unsupported bytecode refuses before mutation.\"\n[rom]\nfile = {}\nsha256 = {}\n",
        quoted(module.to_string_lossy()),
        quoted(rom.to_string_lossy()),
        quoted(rom_sha)
    )
}
fn routine(text: &mut String, name: &str, args: usize, live: bool) {
    text.push_str(&format!(
        "\n[[routines]]\nsymbol = {}\nargs = {args}\nreturn = \"void\"\nlive_eligible = {live}\n",
        quoted(name)
    ));
}
fn bytes(text: &mut String, offset: usize, data: &[u8]) {
    text.push_str(&format!(
        "{{offset = {offset}, data = [{}]}},\n",
        data.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
    ));
}
fn words(text: &mut String, values: &[(usize, u16)]) {
    for &(offset, value) in values {
        bytes(text, offset, &value.to_le_bytes());
    }
}
fn start_case(text: &mut String, name: &str, raw: u16, frozen: u16, cursor: u16) {
    text.push_str(&format!(
        "\n[[routines.cases]]\nname = {}\na = {raw}\nfill = 165\nexpected_return = 0\nwrites = [\n",
        quoted(name)
    ));
    bytes(text, 0xd658, &[0; 72]);
    words(text, &[(0xa78, frozen), (0x590, cursor), (0x911, 0), (0x915, 0)]);
}
fn side(text: &mut String, k: usize, right: bool, timer: u16, pointer: u16, x: u16, y: u16, tile: u16) {
    let d = usize::from(right) * 36 + k;
    words(text, &[(0xd658 + d, timer), (0xd6a0 + d, pointer), (0xd6e8 + d, tile), (0xd730 + d, x), (0xd778 + d, y)]);
}
fn fixtures(checkout: &Path, rom_path: &Path, output: &Path) -> Result<()> {
    require(digest(rom_path)? == ROM_SHA, "fixture ROM identity differs")?;
    let rom = fs::read(rom_path)?;
    let script_pointers = scripts(&rom)?;
    let module = checkout.join("crates/port/src/projectile_trails.rs");
    let mut text = header(&module, rom_path, ROM_SHA);
    routine(&mut text, "HandleProjectileTrails", 0, true);
    let mut cases = 0;
    for k in (0..36).step_by(2) {
        for right in [false, true] {
            for timer in [0, 1, 2, 0x7fff, 0x8000, 0x8001, 0xffff] {
                for cursor in [0, 508, 511, 512] {
                    let name = format!("timer-{k}-{right}-{timer}-{cursor}");
                    start_case(&mut text, &name, 0, 0, cursor);
                    side(&mut text, k, right, timer, script_pointers[(k / 2) % script_pointers.len()], 23, 45, 0xbeef);
                    text.push_str("]\n");
                    cases += 1;
                }
            }
        }
    }
    for frozen in [0, 1, 0xffff] {
        for right in [false, true] {
            for x in [0, 1, 255, 256, 0x7fff, 0x8000, 0xffff] {
                for y in [0, 1, 255, 256, 0x7fff, 0x8000, 0xffff] {
                    for cursor in [0, 509, 0x81ff, 0x8200, 0xffff] {
                        start_case(&mut text, &format!("clip-{frozen}-{right}-{x}-{y}-{cursor}"), 0, frozen, cursor);
                        side(&mut text, 34, right, 2, script_pointers[0], x, y, 0xcdef);
                        text.push_str("]\n");
                        cases += 1;
                    }
                }
            }
        }
    }
    for frozen in [0, 1] {
        for cursor in [0, 508, 509, 510, 511, 512, 0xd2e8, 0xd30c, 0xd330, 0xd378, 0xd39a, 0xd408, 0xfffe] {
            start_case(&mut text, &format!("ordered-pool-{frozen}-{cursor}"), 0, frozen, cursor);
            for k in (0..36).step_by(2) {
                side(&mut text, k, false, 2, script_pointers[0], k as u16 + 10, k as u16 + 30, 0x1234 + k as u16);
                side(&mut text, k, true, 2, script_pointers[0], k as u16 + 11, k as u16 + 31, 0x5678 + k as u16);
            }
            text.push_str("]\n");
            cases += 1;
        }
    }
    for (index, &pointer) in script_pointers.iter().enumerate() {
        for right in [false, true] {
            let base = format!("native-list-{index}-{right}");
            let mut pc = pointer;
            let mut frames = 1usize;
            loop {
                let token = word(&rom, pc)?;
                if token == 0 {
                    break;
                }
                if token & 0x8000 == 0 {
                    frames += usize::from(token);
                    pc += 4;
                } else {
                    pc += 2;
                }
            }
            require(frames <= 128, "native fixture chain exceeds bounded frame count")?;
            for frame in 0..=frames {
                let name = format!("{base}-frame-{frame}");
                if frame == 0 {
                    start_case(&mut text, &name, 0, 0, 0);
                    side(&mut text, 18, right, 1, pointer, 25, 25, 0xa5a5);
                    text.push_str("]\n");
                } else {
                    text.push_str(&format!(
                        "\n[[routines.cases]]\nname = {}\ncontinue_from = {}\nexpected_return = 0\n",
                        quoted(name),
                        quoted(format!("{base}-frame-{}", frame - 1))
                    ));
                }
                cases += 1;
            }
        }
    }
    for symbol in [
        "ProjInstr_MoveLeftProjectileTrailDown",
        "ProjInstr_MoveRightProjectileTrailDown",
        "ProjInstr_MoveLeftProjectileTrailUp",
    ] {
        routine(&mut text, symbol, 1, false);
        for k in 0..36 {
            for position in [0, 1, 0x7fff, 0x8000, 0xfffe, 0xffff] {
                start_case(&mut text, &format!("{symbol}-{k}-{position}"), k, 0, 0);
                words(&mut text, &[(0xd778 + usize::from(k & !1), position), (0xd79c + usize::from(k & !1), position)]);
                text.push_str("]\n");
                cases += 1;
            }
        }
    }
    fs::write(output, text)?;
    println!("fixtures={cases} native_script_starts={} output={}", script_pointers.len(), output.display());
    Ok(())
}

fn replace_one(text: &mut String, original: &str, replacement: &str) -> Result<()> {
    require(text.matches(original).count() == 1, "native instrumentation anchor is not unique")?;
    *text = text.replace(original, replacement);
    Ok(())
}
fn instrument(source: &str, host: &str) -> Result<(String, String)> {
    let mut bank = source.to_owned();
    replace_one(&mut bank, "void HandleProjectileTrails(void) {", "void C_HandleProjectileTrails(void) {")?;
    replace_one(
        &mut bank,
        "void CallProjInstr(uint32 ea, uint16 j) {",
        "void CallProjInstr(uint32 ea, uint16 j) {\n  TrailOpcode(ea);",
    )?;
    let point = bank.find("void CallProjInstr(uint32 ea, uint16 j) {").ok_or("native trail dispatch missing")?;
    bank.insert_str(point, TEMPLATE);
    let mut host = host.to_owned();
    let start = host.find("static void SaveRoutineState").ok_or("host routine capture missing")?;
    let end = start + host[start..].find("// Linker wrapping").ok_or("host linker anchor missing")?;
    host.replace_range(start..end, "");
    let start = host.find("void __wrap_DecompressToMem").ok_or("host decompressor wrapper missing")?;
    let end = start + host[start..].find("// The host has no audio thread").ok_or("host audio anchor missing")?;
    host.replace_range(start..end, "void __wrap_DecompressToMem(uint32 src,uint8 *dst) { decompression_calls++; __real_DecompressToMem(src,dst); }\nextern void TrailInventorySave(void);\n");
    replace_one(
        &mut host,
        "  fclose(inputs);\n  return 0;",
        "  fclose(inputs);\n  TrailInventorySave();\n  return 0;",
    )?;
    Ok((bank, host))
}
fn snapshot(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        require(!path.is_symlink(), "pinned source symlink")?;
        let target = destination.join(path.file_name().ok_or("source filename missing")?);
        if path.is_dir() {
            snapshot(&path, &target)?;
        } else {
            fs::copy(&path, target)?;
        }
    }
    Ok(())
}
fn files(path: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        require(!path.is_symlink(), "evidence/source symlink")?;
        if path.is_dir() {
            files(&path, result)?;
        } else {
            result.push(path);
        }
    }
    result.sort();
    Ok(())
}
fn pinned(main: &Path, run: &Path) -> Result<BTreeMap<String, String>> {
    let archive = main.join("deps/downloads/snesrev-sm.tar.gz");
    require(digest(&archive)? == ARCHIVE_SHA, "pinned source archive differs")?;
    let extracted = run.join("archive-check");
    fs::create_dir(&extracted)?;
    let status =
        Command::new("tar").arg("-xzf").arg(&archive).arg("--strip-components=1").arg("-C").arg(&extracted).status()?;
    require(status.success(), "pinned archive extraction failed")?;
    let mut members = Vec::new();
    files(&extracted, &mut members)?;
    let mut hashes = BTreeMap::new();
    for member in members {
        let relative = member.strip_prefix(&extracted)?;
        let source = main.join("deps/snesrev-sm").join(relative);
        require(fs::read(&member)? == fs::read(&source)?, "pinned C source differs from archive")?;
        hashes.insert(relative.to_string_lossy().into_owned(), digest(&source)?);
    }
    fs::remove_dir_all(&extracted)?;
    Ok(hashes)
}
fn pipeline(executable: &Path, operation: &str, request: &str) -> Result<String> {
    let mut child = Command::new(executable)
        .arg(operation)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child.stdin.take().ok_or("pipeline input unavailable")?.write_all(request.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("pipeline {operation}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
fn build_command(run: &Path) -> Result<Vec<String>> {
    let source = run.join("original-source");
    let mut units = Vec::new();
    for entry in fs::read_dir(&source)? {
        let path = entry?.path();
        let stem = path.file_stem().ok_or("source unit stem missing")?.to_string_lossy();
        if path.extension().is_some_and(|e| e == "c")
            && (stem.starts_with("sm_") || ["spc_player", "tracing", "util"].contains(&stem.as_ref()))
        {
            units.push(if stem == "sm_90" { run.join("sm_90.c") } else { path });
        }
    }
    for entry in fs::read_dir(source.join("snes"))? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "c") {
            units.push(path);
        }
    }
    units.sort();
    let mut command = vec![
        "cc".into(),
        "-O2".into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.to_string_lossy().into_owned(),
    ];
    command.extend(units.iter().map(|p| p.to_string_lossy().into_owned()));
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
fn seal(
    run: &Path,
    status: &str,
    frames: u64,
    source: &BTreeMap<String, String>,
    error: Option<&str>,
    seconds: f64,
) -> Result<()> {
    let mut members = Vec::new();
    files(run, &mut members)?;
    let mut text = format!(
        "schema_version = 1\nstatus = {}\naccepted = false\nframes = {frames}\nseconds = {seconds}\nrom_sha256 = {ROM_SHA:?}\ninputs_sha256 = {INPUT_SHA:?}\nscope = \"Naturally observed actual-C whole trail frames and full WRAM captures; no Rust ownership or original-ROM parity admission\"\n",
        quoted(status)
    );
    if let Some(error) = error {
        text.push_str(&format!("error = {}\n", quoted(error)));
    }
    text.push_str("\n[source]\n");
    for (name, hash) in source {
        text.push_str(&format!("{} = {}\n", quoted(name), quoted(hash)));
    }
    text.push_str("\n[artifacts]\n");
    for member in members {
        let relative = member.strip_prefix(run)?;
        if relative == Path::new("verdict.toml") {
            continue;
        }
        text.push_str(&format!("{} = {}\n", quoted(relative.to_string_lossy()), quoted(digest(&member)?)));
    }
    fs::write(run.join("verdict.toml"), text)?;
    Ok(())
}
fn inventory(checkout: &Path, main: &Path, executable: &Path, run: &Path, frames: u64) -> Result<()> {
    require((1..=129712).contains(&frames), "inventory frame bounds differ")?;
    fs::create_dir(run)?;
    let run = fs::canonicalize(run)?;
    let started = Instant::now();
    let mut hashes = BTreeMap::new();
    let result = (|| -> Result<()> {
        hashes = pinned(main, &run)?;
        require(digest(&main.join("rom/sm.sfc"))? == ROM_SHA, "inventory ROM differs")?;
        require(digest(&main.join("movies/super-metroid-any-3653.inputs"))? == INPUT_SHA, "inventory inputs differ")?;
        require(digest(&main.join("deps/snesrev-sm/src/sm_90.c"))? == SOURCE_SHA, "native bank90 differs")?;
        snapshot(&main.join("deps/snesrev-sm/src"), &run.join("original-source"))?;
        fs::copy(main.join("deps/snesrev-sm/LICENSE.txt"), run.join("LICENSE.txt"))?;
        let (bank, host) = instrument(
            &fs::read_to_string(main.join("deps/snesrev-sm/src/sm_90.c"))?,
            &fs::read_to_string(main.join("tools/decomp/headless.c"))?,
        )?;
        fs::write(run.join("sm_90.c"), bank)?;
        fs::write(run.join("headless.c"), host)?;
        fs::write(run.join("instrumentation.c.in"), TEMPLATE)?;
        fs::copy(checkout.join("tools/decomp/projectile_trails_inventory.rs"), run.join("producer.rs"))?;
        fs::copy(main.join("tools/decomp/headless.c"), run.join("original-headless.c"))?;
        fs::copy(main.join("tools/decomp/headless_frame.h"), run.join("headless_frame.h"))?;
        let command = build_command(&run)?;
        let shared =
            std::env::var("SM_PIPELINE_ROOT").unwrap_or_else(|_| main.join("out/pipeline").display().to_string());
        let workers: usize = std::env::var("SM_PIPELINE_WORKERS").unwrap_or_else(|_| "4".into()).parse()?;
        require((1..=64).contains(&workers), "worker capacity differs")?;
        let request = format!(
            "checkout = {}\nroot = {}\nrun = {}\nworkers = {workers}\ncommand = {}\n",
            quoted(main.to_string_lossy()),
            quoted(&shared),
            quoted(run.to_string_lossy()),
            array(&command)
        );
        fs::write(run.join("build-request.toml"), &request)?;
        let receipt = pipeline(executable, "c-build", &request)?;
        fs::write(run.join("build-receipt.toml"), receipt)?;
        let command = vec![
            run.join("inventory").to_string_lossy().into_owned(),
            main.join("rom/sm.sfc").to_string_lossy().into_owned(),
            main.join("movies/super-metroid-any-3653.inputs").to_string_lossy().into_owned(),
            frames.to_string(),
            run.join("terminal.bin").to_string_lossy().into_owned(),
        ];
        let request = format!(
            "root = {}\nrun = {}\nworkers = {workers}\ncommand = {}\nlabel = \"replay\"\ntimeout = 60\n",
            quoted(&shared),
            quoted(run.to_string_lossy()),
            array(&command)
        );
        fs::write(run.join("replay-request.toml"), &request)?;
        let receipt = pipeline(executable, "exec", &request)?;
        fs::write(run.join("replay-receipt.toml"), receipt)?;
        let stdout = fs::read_to_string(run.join("replay.stdout"))?;
        let progress = stdout
            .lines()
            .filter(|line| line.starts_with("progress "))
            .last()
            .ok_or("replay terminal progress missing")?;
        let fields = progress.split_whitespace().collect::<Vec<_>>();
        require(
            fields.len() == 10 && fields[1].parse::<u64>()? == frames && fields[7] == "0" && fields[8] == "0",
            "natural C replay incomplete or uses CPU fallback",
        )?;
        let counters = fs::read_to_string(run.join("trail-counts.tsv"))?;
        let fields = counters.lines().nth(1).ok_or("trail counter record missing")?.split('\t').collect::<Vec<_>>();
        require(fields.len() == 10, "trail counter shape differs")?;
        require(
            fields[1].parse::<u64>()? > 0 && fields[2].parse::<u64>()? > 0 && fields[3].parse::<u64>()? > 0,
            "natural replay lacks active script/render evidence",
        )?;
        println!("natural_trail_counts={} seconds={:.3}", fields.join(","), started.elapsed().as_secs_f64());
        for (relative, hash) in &hashes {
            require(
                digest(&main.join("deps/snesrev-sm").join(relative))? == *hash,
                "source changed during natural replay",
            )?;
        }
        Ok(())
    })();
    seal(
        &run,
        if result.is_ok() { "diagnostic-complete" } else { "diagnostic-failed" },
        frames,
        &hashes,
        result.as_ref().err().map(|e| e.to_string()).as_deref(),
        started.elapsed().as_secs_f64(),
    )?;
    result
}
fn authenticate(checkout: &Path, run: &Path) -> Result<()> {
    let verdict = fs::read_to_string(run.join("verdict.toml"))?;
    require(verdict.contains("status = \"diagnostic-complete\"\n"), "inventory did not finish")?;
    let (_, artifact_text) = verdict.split_once("\n[artifacts]\n").ok_or("inventory seals missing")?;
    let mut checked = 0;
    for line in artifact_text.lines().filter(|line| !line.is_empty()) {
        let (name, hash) = line.split_once(" = ").ok_or("inventory seal malformed")?;
        let name = name.strip_prefix('"').and_then(|s| s.strip_suffix('"')).ok_or("inventory name malformed")?;
        let hash = hash.strip_prefix('"').and_then(|s| s.strip_suffix('"')).ok_or("inventory hash malformed")?;
        require(!name.contains("..") && !Path::new(name).is_absolute(), "unsafe inventory member")?;
        require(digest(&run.join(name))? == hash, "inventory artifact changed")?;
        checked += 1;
    }
    let (bank, host) = instrument(
        &fs::read_to_string(run.join("original-source/sm_90.c"))?,
        &fs::read_to_string(run.join("original-headless.c"))?,
    )?;
    require(
        fs::read_to_string(run.join("sm_90.c"))? == bank && fs::read_to_string(run.join("headless.c"))? == host,
        "native instrumentation differs",
    )?;
    require(
        fs::read(run.join("producer.rs"))? == fs::read(checkout.join("tools/decomp/projectile_trails_inventory.rs"))?,
        "producer source changed",
    )?;
    require(fs::read_to_string(run.join("instrumentation.c.in"))? == TEMPLATE, "instrumentation template changed")?;
    println!("natural_inventory_authenticated=true artifacts={checked}");
    Ok(())
}
fn capture_fixture(checkout: &Path, run: &Path, output: &Path) -> Result<()> {
    authenticate(checkout, run)?;
    let records = fs::read_to_string(run.join("trail-captures.tsv"))?;
    let mut first_rom: Option<PathBuf> = None;
    let mut cases = Vec::new();
    for line in records.lines().skip(1) {
        let fields = line.split('\t').collect::<Vec<_>>();
        require(fields.len() == 10, "natural capture shape differs")?;
        let rom = run.join(fields[9]);
        if let Some(first) = &first_rom {
            require(digest(first)? == digest(&rom)?, "captured trail ROM changed; split family required")?;
        } else {
            first_rom = Some(rom);
        }
        cases.push((fields[0].to_owned(), run.join(fields[7]), run.join(fields[8])));
    }
    let rom = first_rom.ok_or("no natural active captures")?;
    let mut text = header(&checkout.join("crates/port/src/projectile_trails.rs"), &rom, &digest(&rom)?);
    routine(&mut text, "HandleProjectileTrails", 0, true);
    for (name, before, after) in &cases {
        text.push_str(&format!("\n[[routines.cases]]\nname = {}\nexpected_return = 0\nbefore_file = {}\nbefore_sha256 = {}\nexpected_after_file = {}\nexpected_after_sha256 = {}\n", quoted(format!("natural-active-call-{name}")), quoted(before.to_string_lossy()), quoted(digest(before)?), quoted(after.to_string_lossy()), quoted(digest(after)?)));
    }
    fs::write(output, text)?;
    println!("natural_active_fixtures={} output={}", cases.len(), output.display());
    Ok(())
}
fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("fixtures") if args.len() == 4 => fixtures(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3])),
        Some("inventory") if args.len() == 6 => inventory(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4]), args[5].parse()?),
        Some("authenticate") if args.len() == 3 => authenticate(Path::new(&args[1]), Path::new(&args[2])),
        Some("captures") if args.len() == 4 => capture_fixture(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3])),
        _ => Err("usage: producer fixtures CHECKOUT ROM OUTPUT | inventory CHECKOUT MAIN PIPELINE NEW_RUN FRAMES | authenticate CHECKOUT RUN | captures CHECKOUT RUN OUTPUT".into()),
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("projectile trail producer: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instrumentation_owns_native_boundary_and_removes_decompression_storage() {
        let source = "void HandleProjectileTrails(void) { }\nvoid CallProjInstr(uint32 ea, uint16 j) { }\n";
        let host = "static void SaveRoutineState\n// Linker wrapping\nvoid __wrap_DecompressToMem\n// The host has no audio thread\n  fclose(inputs);\n  return 0;";
        let (bank, host) = instrument(source, host).unwrap();
        assert!(bank.contains("void C_HandleProjectileTrails(void) { }"));
        assert!(bank.contains("TrailOpcode(ea);"));
        assert!(bank.contains("C_HandleProjectileTrails();"));
        assert!(!host.contains("SaveRoutineState"));
        assert!(host.contains("TrailInventorySave();"));
        assert!(instrument(&format!("{source}{source}"), &host).is_err());
    }
    #[test]
    fn serialization_preserves_literal_paths_without_shell_interpolation() {
        assert_eq!(quoted("a\nb\\c\"d"), "\"a\\nb\\\\c\\\"d\"");
        assert_eq!(array(&["$()".into(), "`x`".into()]), "[\"$()\", \"`x`\"]");
    }
}
