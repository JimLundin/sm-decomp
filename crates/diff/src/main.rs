//! diff <movie> (--generated|--fallback-only) [--frames N] [--rom PATH] [--out DIR]
use std::{fs, path::PathBuf, time::Instant};

use anyhow::{Context, Result, ensure};
use diff::{Console, Outcome};
use snes::{Reference, Subject};

struct ReferenceConsole(Reference, Vec<u8>, Vec<u8>);
impl Console for ReferenceConsole {
    fn name(&self) -> &str {
        "reference"
    }
    fn run_frame(&mut self, pad: u16) -> Result<()> {
        self.0.run_frame(pad)?;
        self.1 = self.0.state_bytes();
        self.2 = self.0.framebuffer_bytes();
        Ok(())
    }
    fn regions(&self) -> Vec<(&'static str, &[u8])> {
        vec![("CPU+clocks", &self.1), ("WRAM", self.0.wram()), ("framebuffer", &self.2)]
    }
}
struct SubjectConsole(Subject, Vec<u8>, Vec<u8>);
impl Console for SubjectConsole {
    fn name(&self) -> &str {
        "subject"
    }
    fn run_frame(&mut self, pad: u16) -> Result<()> {
        self.0.run_frame(pad)?;
        self.1 = self.0.state_bytes();
        self.2 = self.0.framebuffer_bytes();
        Ok(())
    }
    fn regions(&self) -> Vec<(&'static str, &[u8])> {
        vec![("CPU+clocks", &self.1), ("WRAM", self.0.wram()), ("framebuffer", &self.2)]
    }
}

struct Args {
    input: PathBuf,
    rom: PathBuf,
    out: PathBuf,
    limit: Option<usize>,
    generated: bool,
}

fn parse_args() -> Result<Args> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .context("usage: diff <movie> (--generated|--fallback-only) [--frames N] [--rom PATH] [--out DIR]")?;
    let mut result =
        Args { input: input.into(), rom: "rom/sm.sfc".into(), out: "out/diff".into(), limit: None, generated: false };
    let mut fallback_only = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fallback-only" => fallback_only = true,
            "--generated" => result.generated = true,
            "--frames" => result.limit = Some(args.next().context("--frames needs a number")?.parse()?),
            "--rom" => result.rom = args.next().context("--rom needs a path")?.into(),
            "--out" => result.out = args.next().context("--out needs a directory")?.into(),
            _ => anyhow::bail!("unknown argument {arg}"),
        }
    }
    ensure!(fallback_only != result.generated, "select exactly one of --generated or --fallback-only");
    Ok(result)
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

fn save_report(out: &std::path::Path, report: &serde_json::Value) -> Result<()> {
    let temporary = out.join("parity.json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(report)?)?;
    fs::rename(temporary, out.join("parity.json"))?;
    Ok(())
}

fn decoded_cpu(bytes: &[u8]) -> serde_json::Value {
    if bytes.len() != 43 {
        return serde_json::Value::Null;
    }
    let mut values = serde_json::Map::new();
    for (index, name) in ["a", "x", "y", "s", "d", "pc"].iter().enumerate() {
        values.insert((*name).into(), u16::from_le_bytes([bytes[index * 2], bytes[index * 2 + 1]]).into());
    }
    for (index, name) in ["db", "pb", "p", "emulation", "waiting", "stopped", "irq_masked"].iter().enumerate() {
        values.insert((*name).into(), bytes[12 + index].into());
    }
    for (index, name) in ["cpu_cycles", "interrupts_taken", "master_clocks"].iter().enumerate() {
        values.insert(
            (*name).into(),
            u64::from_le_bytes(bytes[19 + index * 8..27 + index * 8].try_into().unwrap()).into(),
        );
    }
    values.into()
}

fn replay(args: &Args, report: &mut serde_json::Value) -> Result<()> {
    report["stage"] = "inputs".into();
    let movie_bytes = fs::read(&args.input)?;
    let movie = movie::load(&args.input)?;
    let rom = fs::read(&args.rom)?;
    snes::verify_movie_rom(&movie, &rom)?;
    let count = args.limit.unwrap_or(movie.frames.len());
    report["expected_frames"] = count.into();
    report["available_frames"] = movie.frames.len().into();
    report["identities"] = serde_json::json!({
        "rom_sha256": sha256(&rom), "movie_sha256": sha256(&movie_bytes),
        "input_sha256": sha256(movie::write_inputs(&movie.frames).as_bytes()),
        "binary_sha256": sha256(&fs::read(std::env::current_exe()?)?),
        "compiled_rom_sha256": if args.generated { game::steps::ROM_SHA256 } else { None },
    });
    if count > movie.frames.len() {
        report["status"] = "incomplete".into();
        anyhow::bail!("movie is shorter than the requested frame scope");
    }
    ensure!(count > 0, "no frames selected");
    report["identities"]["selected_input_sha256"] =
        sha256(movie::write_inputs(&movie.frames[..count]).as_bytes()).into();
    let steps = if args.generated { game::steps::STEP_TABLE } else { &[] };
    report["compiled_table_entries"] = steps.len().into();
    if args.generated && steps.is_empty() {
        report["status"] = "uncovered".into();
        anyhow::bail!("no compiled steps; regenerate a nonempty step table");
    }
    if args.generated {
        let identity = movie::Movie { frames: Vec::new(), rom_sha256: game::steps::ROM_SHA256.map(str::to_owned) };
        ensure!(identity.rom_sha256.is_some(), "compiled table has no ROM identity; regenerate");
        snes::verify_movie_rom(&identity, &rom)?;
    }
    report["stage"] = "initialization".into();
    let mut reference = ReferenceConsole(Reference::new(&rom, 0)?, Vec::new(), Vec::new());
    let mut subject = SubjectConsole(Subject::with_steps(&rom, 0, steps)?, Vec::new(), Vec::new());
    report["stage"] = "replay".into();
    save_report(&args.out, report)?;
    for (frame, pad) in movie.frames[..count].iter().enumerate() {
        let outcome = diff::run(&mut reference, &mut subject, std::slice::from_ref(pad));
        report["generated_steps"] = subject.0.generated_steps.into();
        report["interpreted_steps"] = subject.0.interpreted_steps.into();
        match outcome? {
            Outcome::Match { .. } => {
                report["completed_frames"] = (frame + 1).into();
                report["matched_frames"] = (frame + 1).into();
            }
            Outcome::Diverged(mut divergence) => {
                divergence.frame = frame;
                report["completed_frames"] = (frame + 1).into();
                report["status"] = "divergence".into();
                report["first_divergence"] = serde_json::json!({"frame": frame,
                    "region": divergence.region, "total_diffs": divergence.total_diffs,
                    "bytes": divergence.bytes,
                    "reference_cpu": decoded_cpu(&reference.1), "subject_cpu": decoded_cpu(&subject.1)});
                fs::write(args.out.join("divergence.txt"), diff::report(&divergence, Some(&subject.0.misses_csv())))?;
                fs::write(args.out.join("misses.csv"), subject.0.misses_csv())?;
                anyhow::bail!("first frame disagreement at input index {frame} in {}", divergence.region);
            }
        }
        if (frame + 1) % 600 == 0 || frame + 1 == count {
            fs::write(args.out.join("misses.csv"), subject.0.misses_csv())?;
            save_report(&args.out, report)?;
            eprintln!("diff: {}/{} frames match", frame + 1, count);
        }
    }
    if args.generated && subject.0.generated_steps == 0 {
        report["status"] = "uncovered".into();
        anyhow::bail!("no compiled steps executed; selected keys are not covered by this prefix");
    }
    report["status"] = "pass".into();
    report["complete"] = true.into();
    report["stage"] = "complete".into();
    let summary = format!(
        "MATCH: {count} frames, {}, CPU/clocks + WRAM + framebuffer\nGenerated steps: {}\nInterpreted steps: {}\n",
        if args.generated { "generated subject" } else { "fallback-only subject" },
        subject.0.generated_steps,
        subject.0.interpreted_steps
    );
    fs::write(args.out.join("match.txt"), &summary)?;
    print!("{summary}");
    Ok(())
}

fn execute() -> Result<i32> {
    let args = parse_args()?;
    ensure!(!args.out.exists(), "output directory already exists; choose a fresh evidence directory");
    if let Some(parent) = args.out.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(&args.out)?;
    let started = Instant::now();
    let mut report = serde_json::json!({"schema_version": 1, "status": "incomplete", "complete": false,
        "mode": if args.generated { "generated" } else { "fallback-only" },
        "expected_frames": args.limit, "completed_frames": 0, "matched_frames": 0,
        "available_frames": null, "compiled_table_entries": 0, "generated_steps": 0, "interpreted_steps": 0,
        "regions": ["CPU+clocks", "WRAM", "framebuffer"], "first_divergence": null,
        "seed": 0, "controller": "P1", "profile": "lsnes-profile", "frame_convention": "zero-based input indices",
        "stage": "allocated"});
    save_report(&args.out, &report)?;
    if let Err(error) = replay(&args, &mut report) {
        let reason = format!("{error:#}");
        if report["status"] == "incomplete" && report["stage"] != "inputs" {
            report["status"] = if reason.contains("budget") { "incomplete" } else { "error" }.into();
        } else if report["status"] == "incomplete" && !reason.contains("shorter") {
            report["status"] = "error".into();
        }
        report["reason"] = reason.clone().into();
        eprintln!("{reason}");
    }
    if let Some(identities) = report["identities"].as_object() {
        let mutation = [("movie_sha256", &args.input), ("rom_sha256", &args.rom)].iter().any(|(name, path)| {
            fs::read(path).map_or(true, |bytes| identities.get(*name) != Some(&sha256(&bytes).into()))
        });
        if mutation {
            report["status"] = "error".into();
            report["complete"] = false.into();
            report["reason"] = "Movie or ROM changed during parity replay".into();
        }
    }
    report["elapsed_seconds"] = started.elapsed().as_secs_f64().into();
    save_report(&args.out, &report)?;
    Ok(match report["status"].as_str() {
        Some("pass") => 0,
        Some("divergence" | "uncovered") => 1,
        _ => 2,
    })
}

fn main() {
    match execute() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(2);
        }
    }
}
