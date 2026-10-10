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

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .context("usage: diff <movie> (--generated|--fallback-only) [--frames N] [--rom PATH] [--out DIR]")?;
    let mut rom = PathBuf::from("rom/sm.sfc");
    let mut out = PathBuf::from("out/diff");
    let mut limit = None;
    let mut fallback_only = false;
    let mut generated = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fallback-only" => fallback_only = true,
            "--generated" => generated = true,
            "--frames" => limit = Some(args.next().context("--frames needs a number")?.parse::<usize>()?),
            "--rom" => rom = args.next().context("--rom needs a path")?.into(),
            "--out" => out = args.next().context("--out needs a directory")?.into(),
            _ => anyhow::bail!("unknown argument {arg}"),
        }
    }
    ensure!(fallback_only != generated, "select exactly one of --generated or --fallback-only");
    let movie = movie::load(input.as_ref())?;
    let rom = fs::read(rom)?;
    snes::verify_movie_rom(&movie, &rom)?;
    let count = limit.unwrap_or(movie.frames.len()).min(movie.frames.len());
    ensure!(count > 0, "no frames selected");
    let steps = if generated { game::steps::STEP_TABLE } else { &[] };
    ensure!(!generated || !steps.is_empty(), "no compiled steps; run just recompile-steps first");
    if generated {
        let identity = movie::Movie { frames: Vec::new(), rom_sha256: game::steps::ROM_SHA256.map(str::to_owned) };
        ensure!(identity.rom_sha256.is_some(), "compiled table has no ROM identity; regenerate");
        snes::verify_movie_rom(&identity, &rom)?;
    }
    let mut reference = ReferenceConsole(Reference::new(&rom, 0)?, Vec::new(), Vec::new());
    let mut subject = SubjectConsole(Subject::with_steps(&rom, 0, steps)?, Vec::new(), Vec::new());
    fs::create_dir_all(&out)?;
    // A reused output directory must never retain the previous run's verdict.
    for report in ["match.txt", "divergence.txt"] {
        match fs::remove_file(out.join(report)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let started = Instant::now();
    for (index, chunk) in movie.frames[..count].chunks(600).enumerate() {
        let result = diff::run(&mut reference, &mut subject, chunk);
        fs::write(out.join("misses.csv"), subject.0.misses_csv())?;
        fs::write(
            out.join("coverage.txt"),
            format!(
                "compiled table entries: {}\ngenerated steps: {}\ninterpreted steps: {}\n",
                steps.len(),
                subject.0.generated_steps,
                subject.0.interpreted_steps
            ),
        )?;
        match result? {
            Outcome::Match { .. } => {}
            Outcome::Diverged(mut divergence) => {
                divergence.frame += index * 600;
                let report = diff::report(&divergence, Some(&subject.0.misses_csv()));
                fs::write(out.join("divergence.txt"), &report)?;
                anyhow::bail!("{report}");
            }
        }
        eprintln!(
            "diff: {}/{} frames match ({:.1}s)",
            index * 600 + chunk.len(),
            count,
            started.elapsed().as_secs_f64()
        );
    }
    ensure!(
        !generated || subject.0.generated_steps > 0,
        "no compiled steps executed; selected functions are not covered by this prefix"
    );
    let summary = format!(
        "MATCH: {count} frames, {}, CPU/clocks + WRAM + framebuffer; {:.2}s\nInput: {input}\nGenerated steps: {}\nInterpreted steps: {}\n",
        if generated { "generated subject" } else { "fallback-only subject" },
        started.elapsed().as_secs_f64(),
        subject.0.generated_steps,
        subject.0.interpreted_steps
    );
    fs::write(out.join("match.txt"), &summary)?;
    print!("{summary}");
    Ok(())
}
