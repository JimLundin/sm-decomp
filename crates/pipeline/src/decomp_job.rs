//! Bounded decompressor admission, ordered writes and exercised fault controls.
use crate::atomic_toml;
use crate::decomp_acceptance;
use crate::decomp_model::*;
use crate::digest;
use anyhow::{Context, Result, ensure};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tool_execution::builds;
const FAULTS: [&str; 11] = [
    "wrong-output",
    "write-restore",
    "disabled",
    "unexecuted",
    "reduced-branch",
    "build-error",
    "crash",
    "timeout",
    "altered-identity",
    "removed-fixture",
    "reduced-cases",
];
#[derive(Serialize)]
struct NegativeControlLedger<'a> {
    controls: &'a [Control],
}
fn document<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
fn identifier(v: &str) -> bool {
    !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}
fn admit(mut take: impl FnMut(usize) -> Result<Vec<u8>>, destination: usize) -> Result<(usize, Vec<[usize; 2]>)> {
    let mut stores = 0;
    let mut reads = Vec::new();
    loop {
        let header = take(1)?[0];
        if header == 255 {
            return Ok((stores, reads));
        }
        let (mode, length) = if header & 0xe0 == 0xe0 {
            ((header.wrapping_mul(8)) & 0xe0, (((header & 3) as usize) << 8 | take(1)?[0] as usize) + 1)
        } else {
            (header & 0xe0, (header & 31) as usize + 1)
        };
        ensure!(
            destination.checked_add(stores + length).is_some_and(|v| v <= 0x20000),
            "fixture destination writes outside WRAM"
        );
        if mode & 0x80 != 0 {
            let reference = if mode >= 0xc0 {
                (destination + stores)
                    .checked_sub(take(1)?[0] as usize)
                    .context("fixture backreference reads outside WRAM")?
            } else {
                let data = take(2)?;
                destination + u16::from_le_bytes([data[0], data[1]]) as usize
            };
            ensure!(reference + length <= 0x20000, "fixture backreference reads outside WRAM");
            reads.push([reference, reference + length]);
        } else {
            take(if mode == 0 {
                length
            } else if mode == 0x40 {
                2
            } else {
                1
            })?;
        }
        stores += length;
    }
}
fn advance(address: &mut u32) -> Result<usize> {
    ensure!(*address <= 0xffffff && *address & 0x8000 != 0, "source outside bounded LoROM");
    let offset = (((*address >> 16) << 15) | (*address & 0x7fff)) as usize & 0x3fffff;
    let old = *address;
    *address += 1;
    if old & 0xffff == 0xffff {
        *address += 0x8000;
    }
    Ok(offset)
}
fn prepare(fixture: &Fixture, directory: &Path) -> Result<()> {
    ensure!(identifier(&fixture.id), "invalid fixture id");
    let destination = fixture.destination;
    let fill = fixture.initial_fill;
    ensure!(destination < 0x20000, "fixture destination outside WRAM");
    let stream = fixture.stream.clone();
    let mut cursor = 0;
    let (stores, _) = admit(
        |count| {
            ensure!(cursor + count <= stream.len(), "truncated fixture operand or data");
            let data = stream[cursor..cursor + count].to_vec();
            cursor += count;
            Ok(data)
        },
        destination,
    )?;
    ensure!(cursor == stream.len(), "fixture stream has trailing bytes");
    let expected = fixture.expected_bytes.clone();
    ensure!(
        expected.len() == stores && fixture.expected_written == stores,
        "fixture expected extent differs from declared write extent"
    );
    let mut address = fixture.source;
    let mut placements = Vec::new();
    for byte in stream {
        placements.push((advance(&mut address)?, byte));
    }
    ensure!(
        placements.iter().map(|(offset, _)| *offset).collect::<BTreeSet<_>>().len() == placements.len(),
        "fixture source aliases earlier stream bytes"
    );
    let mut rom = vec![0; placements.iter().map(|(offset, _)| *offset).max().context("empty stream")? + 1];
    for (offset, byte) in placements {
        rom[offset] = byte;
    }
    fs::create_dir(directory)?;
    let initial = vec![fill; 0x20000];
    let mut ram = initial.clone();
    ram[destination..destination + expected.len()].copy_from_slice(&expected);
    fs::write(directory.join("rom.bin"), rom)?;
    fs::write(directory.join("before.wram"), initial)?;
    fs::write(directory.join("expected.wram"), ram)?;
    atomic_toml(&directory.join("fixture.toml"), fixture)?;
    Ok(())
}
fn asset(root: &Path, name: &str, hash: &str, size: usize) -> Result<Vec<u8>> {
    let path = root.join(name);
    let data = fs::read(&path)?;
    ensure!(data.len() == size && digest(&path)? == hash, "capture artifact identity differs: {name}");
    Ok(data)
}
fn captures(path: &Path, manifest: &Job, run: &Path) -> Result<(Vec<Fixture>, crate::baseline::CaptureManifest)> {
    let captures: crate::baseline::CaptureManifest = document(path)?;
    ensure!(
        captures.source_revision == manifest.source_revision,
        "capture source identity differs from routine manifest"
    );
    ensure!(
        captures.contract == "decomp-c-captures-v2"
            && captures.routine == "DecompressToMem"
            && captures.routine_address == 0x80b119,
        "capture routine contract differs"
    );
    let root = path.parent().context("capture parent missing")?;
    ensure!(
        captures.rom_view_size == 0x400000 && !&captures.calls.is_empty(),
        "captures require full ROM views and at least one call"
    );
    asset(root, &captures.rom_view_file, &captures.rom_view_sha256, 0x400000)?;
    fs::copy(path, run.join("captures.toml"))?;
    let mut fixtures = Vec::new();
    let mut ids = BTreeSet::new();
    for (index, call) in captures.calls.iter().enumerate() {
        let id = &call.fixture_id;
        ensure!(
            call.sequence == (index + 1) as u64 && identifier(id) && ids.insert(id),
            "capture sequence/identity differs"
        );
        ensure!(call.before.size == 0x20000 && call.after.size == 0x20000, "capture WRAM declared extent differs");
        let rom = asset(root, &call.rom_view_file, &call.rom_view_sha256, 0x400000)?;
        let before = asset(root, &call.before.file, &call.before.sha256, 0x20000)?;
        let after = asset(root, &call.after.file, &call.after.sha256, 0x20000)?;
        let source = call.source_address;
        let destination = call.destination_offset;
        ensure!(destination < 0x20000, "capture destination outside WRAM");
        let mut address = source;
        let mut stream = Vec::new();
        let mut offsets = Vec::new();
        let (stores, reads) = admit(
            |count| {
                let mut values = Vec::new();
                for _ in 0..count {
                    ensure!(stream.len() < 0x400000, "capture source exceeds bounded ROM");
                    let offset = advance(&mut address)?;
                    values.push(rom[offset]);
                    stream.push(rom[offset]);
                    offsets.push(offset);
                }
                Ok(values)
            },
            destination,
        )?;
        let fixture = Fixture {
            id: id.clone(),
            captured: true,
            source,
            destination,
            expected_written: stores,
            expected_final_source: address,
            source_offsets: offsets,
            stream_sha256: format!("{:x}", Sha256::digest(stream)),
            read_extents: reads,
            capture: Some(call.clone()),
            ..Default::default()
        };
        let directory = run.join(id);
        fs::create_dir(&directory)?;
        fs::write(directory.join("rom.bin"), rom)?;
        fs::write(directory.join("before.wram"), before)?;
        fs::write(directory.join("expected.wram"), after)?;
        atomic_toml(&directory.join("fixture.toml"), &fixture)?;
        fixtures.push(fixture);
    }
    Ok((fixtures, captures))
}
fn inventory(root: &Path, directory: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "job artifact symlink");
        if path.is_dir() {
            inventory(root, &path, out)?;
        } else if !path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml")) {
            out.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
#[derive(Serialize)]
struct Instrumentation<'a> {
    contract: &'a str,
    original_sha256: String,
    instrumented_sha256: String,
    replacement_counts: BTreeMap<&'a str, usize>,
    replacements: BTreeMap<&'a str, &'a str>,
}
fn instrument(source: &Path, run: &Path) -> Result<PathBuf> {
    let bank = fs::read_to_string(source.join("src/sm_80.c"))?;
    let start = bank.find("void DecompressToMem(").context("routine start missing")?;
    let end = start + bank[start..].find("static uint8 ReadPpuByte").context("routine end missing")?;
    let mut routine = bank[start..end].to_owned();
    let mut replacements = BTreeMap::new();
    let mut counts = BTreeMap::new();
    for (old, new, count) in [
        ("decompress_dst[dst_pos++] = b;", "DecompTraceStore(decompress_dst + dst_pos++, b);", 4),
        ("decompress_dst[dst_pos++] = b2;", "DecompTraceStore(decompress_dst + dst_pos++, b2);", 1),
        ("decompress_dst[dst_pos++] = b++;", "DecompTraceStore(decompress_dst + dst_pos++, b++);", 1),
    ] {
        ensure!(routine.matches(old).count() == count, "instrumentation destination store contract differs");
        routine = routine.replace(old, new);
        replacements.insert(old, new);
        counts.insert(old, count);
    }
    let path = run.join("sm_80.instrumented.c");
    fs::write(
        &path,
        format!("{}extern void DecompTraceStore(uint8 *, uint8);\n{routine}{}", &bank[..start], &bank[end..]),
    )?;
    atomic_toml(
        &run.join("instrumentation.toml"),
        &Instrumentation {
            contract: "decomp-destination-stores-v2",
            original_sha256: digest(&source.join("src/sm_80.c"))?,
            instrumented_sha256: digest(&path)?,
            replacement_counts: counts,
            replacements,
        },
    )?;
    Ok(path)
}
fn fault(name: &str, run: &Path) -> Result<Fault> {
    let main = run.join("main.rs");
    let lib = run.join("lib.rs");
    let mut text = fs::read_to_string(&main)?;
    match name{
        "wrong-output"=>text=text.replace("    // Finished the observed WRAM mutation.","    wram.write(destination, 0x99).map_err(|e|format!(\"{e:?}\"))?;\n    // Finished the observed WRAM mutation."),
        "write-restore"=>text=text.replace("    let result = decompress_to_mem(","    let old=wram.read(destination).unwrap();wram.write(destination,old^255).unwrap();wram.write(destination,old).unwrap();\n    let result = decompress_to_mem("),
        "disabled"|"unexecuted"=>text=text.replace("    let result = decompress_to_mem(","    return Err(\"replacement was not executed\".into());\n    let result = decompress_to_mem("),
        "reduced-branch"=>fs::write(&lib,fs::read_to_string(&lib)?.replace("    loop {\n        let header","    loop {\n        if written>0{return Err(DecompressError::UnsupportedCommand(0));}\n        let header"))?,
        "build-error"=>text.push_str("\ninvalid Rust syntax!"),"crash"=>text=text.replace("    let rom =","    std::process::abort();\n    let rom ="),"timeout"=>text=text.replace("    let rom =","    loop{std::thread::sleep(std::time::Duration::from_secs(1));}\n    let rom ="),_=>{}
    }
    fs::write(&main, text)?;
    Ok(Fault { fault: name.into(), main_sha256: digest(&main)?, lib_sha256: digest(&lib)? })
}
fn execute(request: &Request, run: &Path, command: &[String], label: &str, timeout: u64) -> Result<String> {
    builds::execute_timeout(command, run, label, &request.root, request.workers, timeout).with_context(|| {
        ["stdout", "stderr"]
            .iter()
            .map(|extension| fs::read_to_string(run.join(format!("{label}.{extension}"))).unwrap_or_default())
            .collect::<String>()
            .trim()
            .to_owned()
    })?;
    Ok(fs::read_to_string(run.join(format!("{label}.stdout")))?)
}

pub fn verify(request: Request) -> Result<Verdict> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let source = checkout.join("deps/snesrev-sm");
    let output = if request.output.is_absolute() { request.output.clone() } else { checkout.join(&request.output) };
    fs::create_dir_all(&output)?;
    let run =
        output.join(format!("{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id()));
    fs::create_dir(&run)?;
    let run = fs::canonicalize(run)?;
    let mut verdict = Verdict {
        contract: "decomp-offline-v2".into(),
        status: "error".into(),
        run: run.clone(),
        live_rust_ownership: "not-tested".into(),
        ..Default::default()
    };
    let result = (|| -> Result<()> {
        let timeout = request.timeout.max(1);
        ensure!(timeout > 0, "timeout must be positive");
        let fault_name = request.fault.as_deref();
        if let Some(fault) = fault_name {
            ensure!(FAULTS.contains(&fault), "unknown fault");
        }
        let path = request
            .manifest
            .as_ref()
            .map(|p| if p.is_absolute() { p.clone() } else { checkout.join(p) })
            .unwrap_or_else(|| checkout.join("tools/decomp/decompress-job.toml"));
        let mut manifest: Job = document(&path)?;
        fs::copy(&path, run.join("job.toml"))?;
        verdict.manifest_sha256 = digest(&path)?;
        if fault_name == Some("altered-identity") {
            manifest.source_revision = "intentionally-altered".into();
        }
        verdict.source_revision = manifest.source_revision.clone();
        verdict.source_identities = decomp_acceptance::source(&checkout, &manifest)?;
        fs::copy(source.join("LICENSE.txt"), run.join("LICENSE.txt"))?;
        let mut identities = BTreeMap::new();
        for name in [
            "tools/decomp/routine_oracle.c",
            "crates/decomp/src/main.rs",
            "crates/decomp/src/lib.rs",
            "crates/decomp/src/abi.rs",
            "crates/decomp/src/ffi.rs",
            "crates/decomp/Cargo.toml",
            "Cargo.lock",
            "tools/decomp/original_oracle.c",
            "tools/decomp/offline-acceptance-v1.toml",
            "crates/pipeline/src/decomp_job.rs",
            "crates/pipeline/src/decomp_acceptance.rs",
            "crates/tool-execution/src/builds.rs",
            "crates/tool-execution/src/build_cache.rs",
        ] {
            let path = checkout.join(name);
            if path.is_file() {
                identities.insert(name.to_owned(), digest(&path)?);
                fs::copy(&path, run.join(path.file_name().unwrap()))?;
            }
        }
        verdict.implementation_identities = identities;
        for fixture in &manifest.fixtures {
            prepare(fixture, &run.join(&fixture.id))?;
        }
        if fault_name == Some("removed-fixture") {
            manifest.fixtures.pop();
        }
        if fault_name == Some("reduced-cases") {
            manifest.required_cases.pop_first();
        }
        ensure!(
            manifest == document(&checkout.join("tools/decomp/decompress-job.toml"))?,
            "manifest differs from frozen canonical job contract"
        );
        let mut fixtures = manifest.fixtures.clone();
        let mut captured = Vec::new();
        let mut capture_manifest = None;
        if let Some(path) = &request.captures {
            let path = if path.is_absolute() { path.clone() } else { checkout.join(path) };
            let (added, doc) = captures(&path, &manifest, &run)?;
            captured = added.clone();
            fixtures.extend(added);
            capture_manifest = Some(doc);
            verdict.captures_manifest_sha256 = Some(digest(&path)?);
        }
        if let Some(name) = fault_name {
            verdict.fault_injection = Some(fault(name, &run)?);
        }
        verdict.stages.push("prepare".into());
        for (label, bank, trace) in
            [("c-oracle-build", source.join("src/sm_80.c"), false), ("c-trace-build", instrument(&source, &run)?, true)]
        {
            let mut command = vec![
                "cc".into(),
                "-std=c11".into(),
                "-O2".into(),
                "-fno-strict-aliasing".into(),
                "-ffunction-sections".into(),
                "-fdata-sections".into(),
                "-iquote".into(),
                source.join("src").to_string_lossy().into_owned(),
                bank.to_string_lossy().into_owned(),
                run.join("routine_oracle.c").to_string_lossy().into_owned(),
                "-Wl,--gc-sections".into(),
                "-o".into(),
                run.join(if trace { "c-trace" } else { "c-oracle" }).to_string_lossy().into_owned(),
            ];
            if trace {
                command.insert(1, "-DDECOMP_TRACE".into());
            }
            execute(&request, &run, &command, label, timeout)?;
        }
        execute(
            &request,
            &run,
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "--crate-type=rlib".into(),
                "--crate-name=decomp".into(),
                "-O".into(),
                run.join("lib.rs").to_string_lossy().into_owned(),
                "-o".into(),
                run.join("libdecomp.rlib").to_string_lossy().into_owned(),
            ],
            "rust-library-build",
            timeout,
        )?;
        execute(
            &request,
            &run,
            &[
                "rustc".into(),
                "--edition=2024".into(),
                "-O".into(),
                run.join("main.rs").to_string_lossy().into_owned(),
                "--extern".into(),
                format!("decomp={}", run.join("libdecomp.rlib").display()),
                "-o".into(),
                run.join("rust-candidate").to_string_lossy().into_owned(),
            ],
            "rust-candidate-build",
            timeout,
        )?;
        verdict.binaries = ["c-oracle", "c-trace", "rust-candidate"]
            .iter()
            .map(|n| Ok((n.to_string(), digest(&run.join(n))?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        verdict.stages.push("build".into());
        let mut observed = BTreeSet::new();
        for fixture in &fixtures {
            let id = &fixture.id;
            let directory = run.join(id);
            let mut record = FixtureRecord {
                id: id.clone(),
                status: "error".into(),
                fixture_sha256: digest(&directory.join("fixture.toml"))?,
                rom_sha256: digest(&directory.join("rom.bin"))?,
                initial_sha256: digest(&directory.join("before.wram"))?,
                expected_sha256: digest(&directory.join("expected.wram"))?,
                ..Default::default()
            };
            verdict.fixtures.push(record.clone());
            let outcome = (|| -> Result<()> {
                for (subject, binary) in [
                    ("c", "c-oracle"),
                    ("rust", "rust-candidate"),
                    ("c_repeat", "c-oracle"),
                    ("rust_repeat", "rust-candidate"),
                    ("c_trace", "c-trace"),
                ] {
                    let command = vec![
                        run.join(binary).to_string_lossy().into_owned(),
                        directory.join("rom.bin").to_string_lossy().into_owned(),
                        directory.join("before.wram").to_string_lossy().into_owned(),
                        fixture.source.to_string(),
                        fixture.destination.to_string(),
                        directory.join(format!("{subject}.wram")).to_string_lossy().into_owned(),
                    ];
                    let stdout = execute(
                        &request,
                        &directory,
                        &command,
                        subject,
                        if fault_name == Some("timeout") && binary == "rust-candidate" { 1 } else { timeout },
                    )?;
                    fs::write(directory.join(format!("{subject}.toml")), stdout)?;
                    record.subject_hashes.insert(subject.into(), digest(&directory.join(format!("{subject}.wram")))?);
                }
                let candidate: Diagnostic = document(&directory.join("rust.toml"))?;
                ensure!(
                    candidate.status == "ok"
                        && candidate.written == fixture.expected_written
                        && candidate.final_source == fixture.expected_final_source
                        && (fixture.captured || candidate.cases == fixture.expected_cases),
                    "candidate diagnostics differ: {id}"
                );
                record.observed_cases = candidate.cases.clone();
                observed.extend(candidate.cases.clone());
                let expected = fs::read(directory.join("expected.wram"))?;
                for subject in ["c", "rust", "c_repeat", "rust_repeat", "c_trace"] {
                    let actual = fs::read(directory.join(format!("{subject}.wram")))?;
                    ensure!(actual == expected, "whole WRAM mismatch: {id}/{subject}");
                }
                ensure!(
                    fs::read(directory.join("rust.toml"))? == fs::read(directory.join("rust_repeat.toml"))?,
                    "repeat diagnostics differ: {id}"
                );
                let c_writes = directory.join("c_trace.wram.writes");
                let rust_writes = directory.join("rust.wram.writes");
                record.c_ordered_writes_sha256 = digest(&c_writes)?;
                record.rust_ordered_writes_sha256 = digest(&rust_writes)?;
                let trace = fs::read_to_string(&c_writes)?;
                let mut write_set = BTreeSet::new();
                for line in trace.lines() {
                    write_set.insert(line.split_whitespace().next().context("invalid trace")?.parse::<usize>()?);
                }
                record.write_count = trace.lines().count();
                record.write_set = write_set;
                ensure!(
                    trace.lines().count() == fixture.expected_written
                        && fs::read(&c_writes)? == fs::read(&rust_writes)?,
                    "ordered WRAM stores differ: {id}"
                );
                ensure!(
                    fs::read(rust_writes)? == fs::read(directory.join("rust_repeat.wram.writes"))?,
                    "repeat ordered WRAM stores differ: {id}"
                );
                record.status = "matched".into();
                Ok(())
            })();
            *verdict.fixtures.last_mut().unwrap() = record;
            outcome?;
        }
        verdict.observed_cases = observed.clone();
        let required = &manifest.required_cases;
        verdict.missing_cases = required.difference(&observed).cloned().collect();
        ensure!(required.is_subset(&observed), "required synthetic cases were not exercised");
        verdict.synthetic_verified = true;
        if let Some(captures) = capture_manifest {
            verdict.captured_calls_verified = captures.calls.len();
            if let Some(rom) = &request.original_rom {
                verdict.original_rom_parity =
                    Some(original(&request, &checkout, &source, &run, rom, &captures, &captured)?);
            }
        } else {
            ensure!(request.original_rom.is_none(), "original ROM checks require captured calls");
        }
        verdict.stages.push("compare".into());
        verdict.status = "partial".into();
        if request.accept {
            accept(&request, &checkout, &run, &manifest, &mut verdict)?;
        }
        verdict.stages.push("verdict".into());
        Ok(())
    })();
    if let Err(error) = result {
        verdict.error = Some(format!("{error:#}"));
        verdict.status = "error".into();
        verdict.routine_accepted = false;
    }
    atomic_toml(&run.join("verdict.toml"), &verdict)?;
    let latest = Pointer {
        run: run.clone(),
        verdict_sha256: digest(&run.join("verdict.toml"))?,
        routine_accepted: verdict.routine_accepted,
        status: verdict.status.clone(),
    };
    atomic_toml(&output.join("latest-attempt.toml"), &latest)?;
    if verdict.routine_accepted {
        atomic_toml(&output.join("decomp-acceptance.toml"), &latest)?;
    }
    Ok(verdict)
}

fn original(
    request: &Request,
    checkout: &Path,
    source: &Path,
    run: &Path,
    rom: &Path,
    captures: &crate::baseline::CaptureManifest,
    fixtures: &[Fixture],
) -> Result<Original> {
    ensure!(
        digest(rom)? == captures.original_rom_sha256 && fs::metadata(rom)?.len() == 0x300000,
        "original ROM identity differs from capture manifest"
    );
    let original = fs::read(rom)?;
    let cpu = source.join("src/snes/cpu.c");
    fs::copy(&cpu, run.join("original_cpu.c"))?;
    let timeout = request.timeout.max(1);
    execute(
        request,
        run,
        &[
            "cc".into(),
            "-std=c11".into(),
            "-O2".into(),
            "-ffunction-sections".into(),
            "-fdata-sections".into(),
            "-iquote".into(),
            source.join("src").to_string_lossy().into_owned(),
            cpu.to_string_lossy().into_owned(),
            run.join("original_oracle.c").to_string_lossy().into_owned(),
            "-Wl,--gc-sections".into(),
            "-o".into(),
            run.join("original-oracle").to_string_lossy().into_owned(),
        ],
        "original-build",
        timeout,
    )?;
    let mut evidence = Original {
        status: "bounded-routine-matched".into(),
        rom_sha256: digest(rom)?,
        cpu_sha256: digest(&cpu)?,
        shim_sha256: digest(&checkout.join("tools/decomp/original_oracle.c"))?,
        binary_sha256: digest(&run.join("original-oracle"))?,
        oracle: "unchanged original ROM executed by pinned upstream 65816 instruction CPU".into(),
        limit: "three captured calls; output below 64 KiB; no timing or whole-game claim".into(),
        fixtures: vec![],
        ineligible: vec![],
        source_stream_checks: vec![],
    };
    for fixture in fixtures {
        let id = &fixture.id;
        let directory = run.join(id);
        let rom_view = fs::read(directory.join("rom.bin"))?;
        let offsets = &fixture.source_offsets;
        let matches = offsets.iter().all(|v| *v < original.len() && original.get(*v) == rom_view.get(*v));
        evidence.source_stream_checks.push(StreamCheck {
            id: id.clone(),
            stream_sha256: fixture.stream_sha256.clone(),
            bytes: offsets.len(),
            matches_unmodified_rom: matches,
        });
        if fixture.expected_written >= 0x10000
            || fixture.destination < 0x2000
            || fixture.read_extents.iter().any(|v| v[0] < 0x2000)
            || !matches
        {
            evidence.ineligible.push(id.clone());
            continue;
        }
        if evidence.fixtures.len() == 3 {
            continue;
        }
        for subject in ["original", "original_repeat"] {
            let stdout = execute(
                request,
                &directory,
                &[
                    run.join("original-oracle").to_string_lossy().into_owned(),
                    rom.to_string_lossy().into_owned(),
                    directory.join("before.wram").to_string_lossy().into_owned(),
                    fixture.source.to_string(),
                    fixture.destination.to_string(),
                    directory.join(format!("{subject}.wram")).to_string_lossy().into_owned(),
                ],
                subject,
                timeout,
            )?;
            fs::write(directory.join(format!("{subject}.toml")), stdout)?;
        }
        let actual = fs::read(directory.join("original.wram"))?;
        let expected = fs::read(directory.join("expected.wram"))?;
        let diagnostic: Diagnostic = document(&directory.join("original.toml"))?;
        ensure!(
            actual == fs::read(directory.join("original_repeat.wram"))?
                && diagnostic == document(&directory.join("original_repeat.toml"))?,
            "original repeat differs: {id}"
        );
        let mut excluded = (0x47..0x51).chain(0x1fd..0x200).collect::<BTreeSet<_>>();
        excluded.extend(diagnostic.stack_written_min..=diagnostic.stack_written_max);
        ensure!(actual.len() == 0x20000, "original WRAM extent missing");
        let mismatch =
            actual.iter().zip(&expected).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| i).collect::<Vec<_>>();
        let unexcluded = mismatch.iter().filter(|i| !excluded.contains(i)).copied().collect::<Vec<_>>();
        let record = OriginalRecord {
            diagnostic,
            id: id.clone(),
            source: fixture.source,
            destination: fixture.destination,
            stream_sha256: fixture.stream_sha256.clone(),
            stream_matches_unmodified_rom: true,
            excluded_offsets: excluded,
            raw_mismatch_offsets: mismatch,
            unexcluded_mismatch_offsets: unexcluded.clone(),
            sha256: digest(&directory.join("original.wram"))?,
            status: "matched".into(),
        };
        evidence.fixtures.push(record);
        atomic_toml(&run.join("original-evidence.toml"), &evidence)?;
        ensure!(unexcluded.is_empty(), "original ROM routine mismatch: {id}");
    }
    ensure!(evidence.fixtures.len() == 3, "fewer than three eligible original ROM calls");
    Ok(evidence)
}
fn accept(request: &Request, checkout: &Path, run: &Path, manifest: &Job, verdict: &mut Verdict) -> Result<()> {
    let contract_path = checkout.join("tools/decomp/offline-acceptance-v1.toml");
    let contract: Contract = document(&contract_path)?;
    ensure!(contract.schema == "decomp-offline-acceptance-contract-v2", "unsupported acceptance contract");
    ensure!(
        request.fault.is_none() && request.captures.is_some() && request.original_rom.is_some(),
        "acceptance requires genuine candidate, frozen captures and original ROM"
    );
    ensure!(
        verdict.manifest_sha256 == contract.job_sha256
            && contract
                .capture_manifest_sha256
                .as_ref()
                .is_none_or(|hash| verdict.captures_manifest_sha256.as_ref() == Some(hash))
            && digest(request.original_rom.as_ref().context("original ROM missing")?)? == contract.original_rom_sha256,
        "acceptance fixture/source identity differs from frozen contract"
    );
    ensure!(
        manifest.fixtures.len() == contract.synthetic_fixture_count
            && verdict.captured_calls_verified == contract.captured_call_count
            && verdict.original_rom_parity.as_ref().context("original evidence missing")?.fixtures.len()
                == contract.original_call_count,
        "acceptance coverage differs from frozen contract"
    );
    fs::copy(&contract_path, run.join("acceptance-contract.toml"))?;
    let mut controls = Vec::new();
    for fault in FAULTS {
        let mut control = request.clone();
        control.captures = None;
        control.original_rom = None;
        control.accept = false;
        control.fault = Some(fault.into());
        control.output = run.join("negative-controls");
        let result = verify(control)?;
        let expected = match fault {
            "wrong-output" => "whole WRAM mismatch",
            "write-restore" => "ordered WRAM stores differ",
            "disabled" | "unexecuted" => "replacement was not executed",
            "reduced-branch" => "UnsupportedCommand",
            "build-error" => "rust",
            "crash" => "rust",
            "timeout" => "timeout",
            "altered-identity" => "source archive identity",
            "removed-fixture" | "reduced-cases" => "frozen canonical job",
            _ => unreachable!(),
        };
        let mut rejected = result.status == "error"
            && !result.routine_accepted
            && result.error.as_deref().is_some_and(|s| s.contains(expected));
        if ["wrong-output", "write-restore", "disabled", "unexecuted", "reduced-branch", "crash", "timeout"]
            .contains(&fault)
        {
            rejected &= result.stages.contains(&"build".into()) && !result.fixtures.is_empty();
        }
        let control_run = &result.run;
        controls.push(Control {
            fault: fault.into(),
            rejected,
            run: control_run.clone(),
            verdict_sha256: digest(&control_run.join("verdict.toml"))?,
            error: result.error.clone(),
        });
        atomic_toml(&run.join("negative-controls.toml"), &NegativeControlLedger { controls: &controls })?;
        ensure!(rejected, "fault control was not rejected: {fault}");
    }
    verdict.negative_controls = controls.clone();
    ensure!(
        decomp_acceptance::source(checkout, manifest)? == verdict.source_identities,
        "source changed during acceptance"
    );
    for (name, hash) in &verdict.implementation_identities {
        ensure!(digest(&checkout.join(name))? == *hash, "implementation changed during acceptance: {name}");
    }
    ensure!(
        digest(&run.join("captures.toml"))?
            == *verdict.captures_manifest_sha256.as_ref().context("capture digest missing")?,
        "capture manifest changed during acceptance"
    );
    let mut artifacts = BTreeMap::new();
    inventory(run, run, &mut artifacts)?;
    let seal = Seal {
        contract,
        contract_sha256: digest(&contract_path)?,
        run: run.into(),
        job_sha256: verdict.manifest_sha256.clone(),
        source: verdict.source_identities.clone(),
        implementation: verdict.implementation_identities.clone(),
        binaries: verdict.binaries.clone(),
        fixtures: verdict.fixtures.clone(),
        observed_cases: verdict.observed_cases.clone(),
        negative_controls: controls,
        capture_manifest_sha256: verdict.captures_manifest_sha256.clone().context("capture digest missing")?,
        artifact_inventory_schema: "decomp-artifact-inventory-v2".into(),
        artifacts,
        original_rom_parity: verdict.original_rom_parity.clone().context("original evidence missing")?,
        status: "accepted".into(),
        routine_accepted: true,
    };
    atomic_toml(&run.join("acceptance.toml"), &seal)?;
    verdict.acceptance_sha256 = Some(digest(&run.join("acceptance.toml"))?);
    verdict.status = "accepted".into();
    verdict.routine_accepted = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_control_ledger_roundtrips_as_a_toml_document() {
        #[derive(serde::Deserialize)]
        struct Ledger {
            controls: Vec<Control>,
        }
        let controls = vec![Control {
            fault: "wrong-output".into(),
            rejected: true,
            run: PathBuf::from("negative-controls/one"),
            verdict_sha256: "5".repeat(64),
            error: Some("whole WRAM mismatch".into()),
        }];
        let text = toml::to_string_pretty(&NegativeControlLedger { controls: &controls }).unwrap();
        let restored: Ledger = toml::from_str(&text).unwrap();
        assert_eq!(restored.controls, controls);
        assert!(text.contains("[[controls]]"));
    }
    #[test]
    fn malformed_streams_never_enter_unsafe_oracle() {
        for (stream, destination) in [
            (vec![0xe0], 4096),
            (vec![0x40, 0x12], 4096),
            (vec![0xc0, 1, 255], 0),
            (vec![0x80, 1, 0, 255], 0x1ffff),
            (vec![0, 0x12], 4096),
        ] {
            let mut cursor = 0;
            assert!(
                admit(
                    |count| {
                        ensure!(cursor + count <= stream.len(), "truncated");
                        let data = stream[cursor..cursor + count].to_vec();
                        cursor += count;
                        Ok(data)
                    },
                    destination
                )
                .is_err()
            );
        }
    }
}
