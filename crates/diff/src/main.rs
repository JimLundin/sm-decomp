//! diff <movie> (--generated|--fallback-only) [--frames N] [--rom PATH] [--out DIR]
use std::{fs, path::PathBuf, time::Instant};

use anyhow::{Context, Result, ensure};
use diff::{Console, Outcome};
use snes::{Reference, Subject};
fn save_profile(output: &std::path::Path, subject: &Subject) -> Result<()> {
    pipeline_model::write(&output.join("coverage.toml"), &subject.coverage())
}

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

fn save_report(out: &std::path::Path, report: &Report) -> Result<()> {
    let temporary = out.join("parity.toml.tmp");
    fs::write(&temporary, toml::to_string_pretty(report)?)?;
    fs::rename(temporary, out.join("parity.toml"))?;
    Ok(())
}

#[derive(serde::Serialize, Default)]
struct Identities {
    rom_sha256: String,
    movie_sha256: String,
    input_sha256: String,
    binary_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    compiled_rom_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_input_sha256: Option<String>,
}
#[derive(serde::Serialize)]
struct CpuState {
    a: u16,
    x: u16,
    y: u16,
    s: u16,
    d: u16,
    pc: u16,
    db: u8,
    pb: u8,
    p: u8,
    emulation: u8,
    waiting: u8,
    stopped: u8,
    irq_masked: u8,
    cpu_cycles: u64,
    interrupts_taken: u64,
    master_clocks: u64,
}
#[derive(serde::Serialize)]
struct Divergence {
    frame: usize,
    region: String,
    total_diffs: usize,
    bytes: Vec<(usize, u8, u8)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_cpu: Option<CpuState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_cpu: Option<CpuState>,
}
#[derive(serde::Serialize)]
struct Report {
    schema_version: u32,
    status: String,
    complete: bool,
    mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_frames: Option<usize>,
    completed_frames: usize,
    matched_frames: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    available_frames: Option<usize>,
    compiled_table_entries: usize,
    generated_steps: u64,
    interpreted_steps: u64,
    regions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_divergence: Option<Divergence>,
    seed: u64,
    controller: String,
    profile: String,
    frame_convention: String,
    stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    identities: Option<Identities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage_sha256: Option<String>,
    elapsed_seconds: f64,
}
fn decoded_cpu(bytes: &[u8]) -> Option<CpuState> {
    if bytes.len() != 43 {
        return None;
    }
    let word = |i: usize| u16::from_le_bytes([bytes[i * 2], bytes[i * 2 + 1]]);
    let wide = |i: usize| u64::from_le_bytes(bytes[19 + i * 8..27 + i * 8].try_into().unwrap());
    Some(CpuState {
        a: word(0),
        x: word(1),
        y: word(2),
        s: word(3),
        d: word(4),
        pc: word(5),
        db: bytes[12],
        pb: bytes[13],
        p: bytes[14],
        emulation: bytes[15],
        waiting: bytes[16],
        stopped: bytes[17],
        irq_masked: bytes[18],
        cpu_cycles: wide(0),
        interrupts_taken: wide(1),
        master_clocks: wide(2),
    })
}

fn replay(args: &Args, report: &mut Report) -> Result<()> {
    report.stage = "inputs".into();
    let movie_bytes = fs::read(&args.input)?;
    let movie = movie::load(&args.input)?;
    let rom = fs::read(&args.rom)?;
    snes::verify_movie_rom(&movie, &rom)?;
    let count = args.limit.unwrap_or(movie.frames.len());
    report.expected_frames = Some(count);
    report.available_frames = Some(movie.frames.len());
    report.identities = Some(Identities {
        rom_sha256: sha256(&rom),
        movie_sha256: sha256(&movie_bytes),
        input_sha256: sha256(movie::write_inputs(&movie.frames).as_bytes()),
        binary_sha256: sha256(&fs::read(std::env::current_exe()?)?),
        compiled_rom_sha256: if args.generated { game::steps::ROM_SHA256.map(str::to_owned) } else { None },
        selected_input_sha256: None,
    });
    if count > movie.frames.len() {
        report.status = "incomplete".into();
        anyhow::bail!("movie is shorter than the requested frame scope");
    }
    ensure!(count > 0, "no frames selected");
    report.identities.as_mut().unwrap().selected_input_sha256 =
        Some(sha256(movie::write_inputs(&movie.frames[..count]).as_bytes()));
    let steps = if args.generated { game::steps::STEP_TABLE } else { &[] };
    report.compiled_table_entries = steps.len();
    if args.generated && steps.is_empty() {
        report.status = "uncovered".into();
        anyhow::bail!("no compiled steps; regenerate a nonempty step table");
    }
    if args.generated {
        let identity = movie::Movie { frames: Vec::new(), rom_sha256: game::steps::ROM_SHA256.map(str::to_owned) };
        ensure!(identity.rom_sha256.is_some(), "compiled table has no ROM identity; regenerate");
        snes::verify_movie_rom(&identity, &rom)?;
    }
    report.stage = "initialization".into();
    let mut reference = ReferenceConsole(Reference::new(&rom, 0)?, Vec::new(), Vec::new());
    let mut subject = SubjectConsole(Subject::with_steps(&rom, 0, steps)?, Vec::new(), Vec::new());
    report.stage = "replay".into();
    save_profile(&args.out, &subject.0)?;
    save_report(&args.out, report)?;
    for (frame, pad) in movie.frames[..count].iter().enumerate() {
        let outcome = diff::run(&mut reference, &mut subject, std::slice::from_ref(pad));
        report.generated_steps = subject.0.generated_steps;
        report.interpreted_steps = subject.0.interpreted_steps;
        if outcome.is_err() {
            save_profile(&args.out, &subject.0)?;
        }
        match outcome? {
            Outcome::Match { .. } => {
                report.completed_frames = frame + 1;
                report.matched_frames = frame + 1;
            }
            Outcome::Diverged(mut divergence) => {
                divergence.frame = frame;
                report.completed_frames = frame + 1;
                report.status = "divergence".into();
                report.first_divergence = Some(Divergence {
                    frame,
                    region: divergence.region.into(),
                    total_diffs: divergence.total_diffs,
                    bytes: divergence.bytes.clone(),
                    reference_cpu: decoded_cpu(&reference.1),
                    subject_cpu: decoded_cpu(&subject.1),
                });
                fs::write(args.out.join("divergence.txt"), diff::report(&divergence, Some(&subject.0.misses_csv())))?;
                fs::write(args.out.join("misses.csv"), subject.0.misses_csv())?;
                save_profile(&args.out, &subject.0)?;
                anyhow::bail!("first frame disagreement at input index {frame} in {}", divergence.region);
            }
        }
        if (frame + 1) % 600 == 0 || frame + 1 == count {
            fs::write(args.out.join("misses.csv"), subject.0.misses_csv())?;
            save_profile(&args.out, &subject.0)?;
            save_report(&args.out, report)?;
            eprintln!("diff: {}/{} frames match", frame + 1, count);
        }
    }
    if args.generated && subject.0.generated_steps == 0 {
        report.status = "uncovered".into();
        anyhow::bail!("no compiled steps executed; selected keys are not covered by this prefix");
    }
    report.status = "pass".into();
    report.complete = true;
    report.stage = "complete".into();
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
    let mut report = Report {
        schema_version: 2,
        status: "incomplete".into(),
        complete: false,
        mode: if args.generated { "generated" } else { "fallback-only" }.into(),
        expected_frames: args.limit,
        completed_frames: 0,
        matched_frames: 0,
        available_frames: None,
        compiled_table_entries: 0,
        generated_steps: 0,
        interpreted_steps: 0,
        regions: vec!["CPU+clocks".into(), "WRAM".into(), "framebuffer".into()],
        first_divergence: None,
        seed: 0,
        controller: "P1".into(),
        profile: "lsnes-profile".into(),
        frame_convention: "zero-based input indices".into(),
        stage: "allocated".into(),
        identities: None,
        reason: None,
        coverage_sha256: None,
        elapsed_seconds: 0.0,
    };
    save_report(&args.out, &report)?;
    if let Err(error) = replay(&args, &mut report) {
        let reason = format!("{error:#}");
        if report.status == "incomplete" && report.stage != "inputs" {
            report.status = if reason.contains("budget") { "incomplete" } else { "error" }.into();
        } else if report.status == "incomplete" && !reason.contains("shorter") {
            report.status = "error".into();
        }
        report.reason = Some(reason.clone());
        eprintln!("{reason}");
    }
    if let Some(identities) = &report.identities {
        let mutation = [(&identities.movie_sha256, &args.input), (&identities.rom_sha256, &args.rom)]
            .iter()
            .any(|(expected, path)| fs::read(path).map_or(true, |bytes| sha256(&bytes) != **expected));
        if mutation {
            report.status = "error".into();
            report.complete = false;
            report.reason = Some("Movie or ROM changed during parity replay".into());
        }
    }
    if args.out.join("coverage.toml").is_file() {
        report.coverage_sha256 = Some(sha256(&fs::read(args.out.join("coverage.toml"))?));
    }
    report.elapsed_seconds = started.elapsed().as_secs_f64();
    save_report(&args.out, &report)?;
    Ok(match report.status.as_str() {
        "pass" => 0,
        "divergence" | "uncovered" => 1,
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
