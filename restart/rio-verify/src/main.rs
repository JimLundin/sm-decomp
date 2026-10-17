//! Independent actual-C body comparison. No native terrain/live acceptance.
#![forbid(unsafe_code)]
use sha2::{Digest, Sha256};
use sm_rio::{
    Output, WRAM_SIZE, execute,
    fixtures::{ScriptedServices, body_fixtures},
};
use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

// These are embedded by rustc together with the linked subject. Before executing
// any producer the run rejects later edits and preserves these compiled bytes.
const SUBJECT: [(&str, &[u8]); 9] = [
    ("Cargo.toml", include_bytes!("../../rio/Cargo.toml")),
    ("src/lib.rs", include_bytes!("../../rio/src/lib.rs")),
    ("src/actor.rs", include_bytes!("../../rio/src/actor.rs")),
    ("src/memory.rs", include_bytes!("../../rio/src/memory.rs")),
    ("src/service_protocol.rs", include_bytes!("../../rio/src/service_protocol.rs")),
    ("src/service_writes.rs", include_bytes!("../../rio/src/service_writes.rs")),
    ("src/sine.rs", include_bytes!("../../rio/src/sine.rs")),
    ("src/fixtures.rs", include_bytes!("../../rio/src/fixtures.rs")),
    ("src/tests.rs", include_bytes!("../../rio/src/tests.rs")),
];
const COMPILED_LOCK: &[u8] = include_bytes!("../Cargo.lock");

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hash_file(path: &Path) -> Result<String, String> {
    Ok(hash(&fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?))
}
fn put(bytes: &mut Vec<u8>, value: u32, n: usize) {
    bytes.extend_from_slice(&value.to_le_bytes()[..n]);
}
fn run(cmd: &mut Command, log: &Path) -> Result<(), String> {
    let stdout = log.with_extension("stdout");
    let stderr = log.with_extension("stderr");
    let out = fs::File::create(&stdout).map_err(|e| e.to_string())?;
    let err = fs::File::create(&stderr).map_err(|e| e.to_string())?;
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("{cmd:?}: {e}"))?;
    // Each reader retains at most 512 KiB, then closes its pipe. A single excess
    // byte is observed but never retained. Readers cannot grow files unboundedly.
    let out_pipe = child.stdout.take().ok_or("missing stdout pipe")?;
    let err_pipe = child.stderr.take().ok_or("missing stderr pipe")?;
    fn drain(mut pipe: impl Read, mut file: fs::File) -> io::Result<bool> {
        io::copy(&mut pipe.by_ref().take(512 * 1024), &mut file)?;
        let mut extra = [0];
        Ok(pipe.read(&mut extra)? != 0)
    }
    let out_reader = thread::spawn(move || drain(out_pipe, out));
    let err_reader = thread::spawn(move || drain(err_pipe, err));
    let started = Instant::now();
    let deadline = Duration::from_secs(60);
    let mut failure = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => (),
            Err(e) => {
                failure = Some(format!("process observation failed: {e}"));
                break None;
            }
        }
        let output_bytes =
            fs::metadata(&stdout).map(|m| m.len()).unwrap_or(0) + fs::metadata(&stderr).map(|m| m.len()).unwrap_or(0);
        if started.elapsed() >= deadline || output_bytes >= 1024 * 1024 {
            failure = Some("process exceeded 60-second deadline or 1-MiB output budget".into());
            break None;
        }
        thread::sleep(Duration::from_millis(20));
    };
    if let Some(reason) = failure {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("{cmd:?}: {reason}; see {}", log.display()));
    }
    let status = status.ok_or("missing process status")?;
    // Descendants may retain inherited pipes. Keep observation inside the same
    // deadline and do not block waiting for a stuck compiler helper.
    while !out_reader.is_finished() || !err_reader.is_finished() {
        if started.elapsed() >= deadline {
            return Err("log drain exceeded producer deadline; inspect compiler descendants".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    let out_excess = out_reader.join().map_err(|_| "stdout reader panicked")?.map_err(|e| e.to_string())?;
    let err_excess = err_reader.join().map_err(|_| "stderr reader panicked")?.map_err(|e| e.to_string())?;
    if out_excess || err_excess {
        return Err("producer exceeded 512-KiB per-stream output cap".into());
    }
    if !status.success() {
        return Err(format!("{cmd:?} failed: {status}; see {}", log.display()));
    }
    Ok(())
}
// Brace extraction is deliberately narrow: only sealed, known recovered source
// definitions with no braces in strings/comments inside these bodies are used.
fn definition(source: &str, start: &str) -> Result<String, String> {
    let matches: Vec<_> = source.match_indices(start).collect();
    if matches.len() != 1 {
        return Err(format!("expected one definition prefix {start:?}"));
    }
    let at = matches[0].0;
    let open = at + source[at..].find('{').ok_or("missing definition brace")?;
    let mut depth = 0;
    for (offset, b) in source.as_bytes()[open..].iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let mut end = open + offset + 1;
                    if source.as_bytes().get(end) == Some(&b';') {
                        end += 1;
                    }
                    return Ok(source[at..end].to_owned());
                }
            }
            _ => (),
        }
    }
    Err("unterminated extracted definition".into())
}
fn main() {
    if let Err(e) = verify() {
        eprintln!("Rio actual-C verification failed: {e}");
        std::process::exit(1);
    }
}
fn verify() -> Result<(), String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: sm-rio-verify CHECKOUT ROM GCC NEW_RUN_DIRECTORY".into());
    }
    let root = fs::canonicalize(&args[0]).map_err(|e| e.to_string())?;
    let rom_path = fs::canonicalize(&args[1]).map_err(|e| e.to_string())?;
    let gcc = fs::canonicalize(&args[2]).map_err(|e| e.to_string())?;
    let out_arg = PathBuf::from(&args[3]);
    fs::create_dir(&out_arg).map_err(|e| format!("new run directory required: {e}"))?;
    let out = fs::canonicalize(out_arg).map_err(|e| e.to_string())?;
    fs::write(out.join("status.txt"), "running; no passing verdict yet\n").map_err(|e| e.to_string())?;
    // Failure is terminal and retained; verdict is only written after all cases.
    let source = root.join("deps/snesrev-sm/src");
    let mut manifest = String::from(
        "schema = \"rio-actual-c-v1\"\nsource_revision = \"578f90b3cc49557bb70060ad033bb90b8cf8ac50\"\nmocked_services = [\"horizontal terrain\", \"vertical terrain\", \"angle\"]\nreal_terrain = false\nlive = false\noriginal_rom_parity = false\n",
    );
    let subject = Path::new(env!("CARGO_MANIFEST_DIR")).join("../rio");
    for (name, compiled) in SUBJECT {
        if fs::read(subject.join(name)).map_err(|e| e.to_string())? != compiled {
            return Err(format!("subject changed since driver compilation: {name}"));
        }
    }
    let inputs = out.join("inputs");
    fs::create_dir(&inputs).map_err(|e| e.to_string())?;
    for line in include_str!("../pinned-sha256.txt").lines() {
        let (name, expected) = line.split_once(' ').ok_or("bad pin record")?;
        let bytes = fs::read(source.join(name)).map_err(|e| e.to_string())?;
        if hash(&bytes) != expected {
            return Err(format!("pinned source mismatch: {name}"));
        }
        fs::write(inputs.join(name), &bytes).map_err(|e| e.to_string())?;
        manifest.push_str(&format!("\n[[inputs]]\npath = \"inputs/{name}\"\nsha256 = \"{expected}\"\n"));
    }
    let rom = fs::read(&rom_path).map_err(|e| e.to_string())?;
    if rom.len() != 0x300000 || hash(&rom) != "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72" {
        return Err("pristine ROM identity mismatch".into());
    }
    let mut bodies = String::new();
    for (file, starts) in [
        (
            "sm_a2.c",
            vec![
                "const uint16 *EnemyInstr_Rio_Instr_1(",
                "void Rio_Init(",
                "void CallRioFunc(",
                "void Rio_Main(",
                "void Rio_1(",
                "void Rio_2(",
                "void Rio_3(",
                "void Rio_4(",
                "void Rio_5(",
                "void Rio_6(",
            ],
        ),
        ("sm_80.c", vec!["uint16 NextRandom(", "void QueueSfx2_Max6(", "void QueueSfx2_Internal("]),
        ("sm_a0.c", vec!["uint8 CompareDistToSamus_X(", "uint16 CheckIfEnemyIsOnScreen("]),
        ("sm_94.c", vec!["const int16 kSinCosTable8bit_Sext[320] ="]),
    ] {
        let text = fs::read_to_string(inputs.join(file)).map_err(|e| e.to_string())?;
        for start in starts {
            bodies.push_str(&format!("\n/* Exact definition from {file}: {start} */\n"));
            bodies.push_str(&definition(&text, start)?);
            bodies.push('\n');
        }
        if file == "sm_a2.c" {
            for name in ["g_word_A2BBBB", "g_word_A2BBBF"] {
                let line =
                    text.lines().find(|l| l.starts_with(&format!("#define {name} "))).ok_or("missing ROM macro")?;
                bodies.insert_str(0, &format!("{line}\n"));
            }
        }
    }
    let harness = include_str!("../harness.c.in").replace("/* RECOVERED_BODIES */", &bodies);
    let c_path = out.join("reference.c");
    fs::write(&c_path, &harness).map_err(|e| e.to_string())?;
    let exe = out.join(if cfg!(windows) { "reference.exe" } else { "reference" });
    // COMPILER_TCC only selects portable assert.h in upstream types.h. Original
    // structs/macros/bodies are unchanged. Disable optimizer alias/overflow assumptions.
    let flags = ["-std=c11", "-O0", "-fno-strict-aliasing", "-fwrapv", "-DCOMPILER_TCC=1"];
    manifest.push_str(&format!("\n[execution]\nrom_sha256 = \"{}\"\ncompiler_sha256 = \"{}\"\nharness_sha256 = \"{}\"\nverifier_executable_sha256 = \"{}\"\nflags = {:?}\n", hash(&rom), hash_file(&gcc)?, hash(harness.as_bytes()), hash_file(&env::current_exe().map_err(|e| e.to_string())?)?, flags));
    // Retain exactly the embedded Rust module and verifier source.
    let driver: [(&str, &[u8]); 4] = [
        ("main.rs", include_bytes!("main.rs")),
        ("Cargo.toml", include_bytes!("../Cargo.toml")),
        ("harness.c.in", include_bytes!("../harness.c.in")),
        ("pinned-sha256.txt", include_bytes!("../pinned-sha256.txt")),
    ];
    for (label, files) in [("rust-subject", SUBJECT.as_slice()), ("rust-driver", driver.as_slice())] {
        let dest = out.join(label);
        fs::create_dir(&dest).map_err(|e| e.to_string())?;
        for &(name, bytes) in files {
            if let Some(parent) = dest.join(name).parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(dest.join(name), bytes).map_err(|e| e.to_string())?;
            manifest
                .push_str(&format!("\n[[rust_sources]]\npath = \"{label}/{name}\"\nsha256 = \"{}\"\n", hash(bytes)));
        }
    }
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.lock");
    let lock = fs::read(lock_path).map_err(|e| e.to_string())?;
    if lock != COMPILED_LOCK {
        return Err("verifier lockfile changed since compilation".into());
    }
    fs::write(out.join("rust-driver/Cargo.lock"), &lock).map_err(|e| e.to_string())?;
    manifest
        .push_str(&format!("\n[[rust_sources]]\npath = \"rust-driver/Cargo.lock\"\nsha256 = \"{}\"\n", hash(&lock)));
    fs::write(out.join("inputs.toml"), &manifest).map_err(|e| e.to_string())?;
    run(Command::new(&gcc).args(flags).arg("-I").arg(&inputs).arg(&c_path).arg("-o").arg(&exe), &out.join("compile"))?;
    manifest.push_str(&format!("\n[reference]\nexecutable_sha256 = \"{}\"\n", hash_file(&exe)?));
    let mut entries = BTreeSet::new();
    let cases = body_fixtures();
    if cases.is_empty() {
        return Err("empty fixture corpus".into());
    }
    for (i, f) in cases.iter().enumerate() {
        entries.insert(f.entry as u32);
        let prefix = format!("case-{i:04}");
        let input = out.join(format!("{prefix}.input.bin"));
        let actual_path = out.join(format!("{prefix}.actual.bin"));
        let expected_path = out.join(format!("{prefix}.rust.bin"));
        let mut bytes = Vec::new();
        put(&mut bytes, 0x314f4952, 4);
        put(&mut bytes, f.entry as u32, 4);
        put(&mut bytes, f.k.into(), 2);
        put(&mut bytes, f.arg, 4);
        put(&mut bytes, f.steps.len() as u32, 4);
        if f.wram.len() != WRAM_SIZE {
            return Err("fixture WRAM incomplete".into());
        }
        bytes.extend_from_slice(&f.wram);
        for step in &f.steps {
            put(&mut bytes, step.kind as u32, 1);
            put(&mut bytes, step.result.into(), 2);
            put(&mut bytes, step.patches.len() as u32, 4);
            for &(offset, value) in &step.patches {
                put(&mut bytes, offset as u32, 4);
                put(&mut bytes, value.into(), 1);
            }
        }
        fs::write(&input, &bytes).map_err(|e| e.to_string())?;
        let mut state = f.wram.clone();
        let mut services = ScriptedServices::new(f.steps.clone());
        let result = execute(f.entry, f.k, f.arg, &mut state, &rom, &mut services)?
            .ok_or_else(|| format!("{} refused", f.name))?;
        if !services.complete() {
            return Err(format!("{} Rust script not consumed", f.name));
        }
        let mut expected = Vec::new();
        put(&mut expected, 0x3154554f, 4);
        let (tag, continuation) = match result {
            Output::Done => (0, 0),
            Output::Continue(j) => (1, u32::from(j)),
        };
        put(&mut expected, tag, 4);
        put(&mut expected, continuation, 4);
        put(&mut expected, services.calls.len() as u32, 4);
        put(&mut expected, services.steps.len() as u32, 4);
        expected.extend_from_slice(&state);
        for call in &services.calls {
            put(&mut expected, call.kind as u32, 1);
            put(&mut expected, call.k.into(), 2);
            put(&mut expected, call.amount as u32, 4);
        }
        fs::write(&expected_path, &expected).map_err(|e| e.to_string())?;
        run(Command::new(&exe).arg(&rom_path).arg(&input).arg(&actual_path), &out.join(&prefix))?;
        let actual = fs::read(&actual_path).map_err(|e| e.to_string())?;
        if actual != expected {
            let first =
                actual.iter().zip(&expected).position(|(a, b)| a != b).unwrap_or(actual.len().min(expected.len()));
            return Err(format!("{} mismatch at protocol byte {first:#x}; retained {prefix}", f.name));
        }
        manifest.push_str(&format!("\n[[cases]]\nname = {:?}\nentry = {}\ninput = \"{prefix}.input.bin\"\ninput_sha256 = \"{}\"\nactual = \"{prefix}.actual.bin\"\nrust = \"{prefix}.rust.bin\"\noutput_sha256 = \"{}\"\nconsumed_steps = {}\n", f.name, f.entry as u32, hash(&bytes), hash(&actual), services.calls.len()));
        fs::write(out.join("progress.toml"), &manifest).map_err(|e| e.to_string())?;
    }
    if entries != (0..10).collect() {
        return Err(format!("incomplete owned entry coverage: {entries:?}"));
    }
    manifest.push_str(&format!("\n[verdict]\nstatus = \"passed\"\ncases = {}\nentries = 10\nfull_wram = true\noutputs = true\nordered_calls = true\nscript_consumption = true\n", cases.len()));
    fs::write(out.join("verdict.toml"), manifest).map_err(|e| e.to_string())?;
    fs::write(out.join("status.txt"), "passed; see verdict.toml\n").map_err(|e| e.to_string())?;
    println!("passed {} Rio actual-C cases over all 10 entries; {}", cases.len(), out.join("verdict.toml").display());
    Ok(())
}
