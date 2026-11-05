//! Headless replay of a prepared start: no window, device or comparison backend.
//! `sm_replay <content> <start> <inputs> <frames> <lock-out>`
use sm_tools::trace::{Running, Tick};

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [content, start, inputs, frames, out] = args.as_slice() else {
        return Err("usage: sm_replay <content> <start> <inputs> <frames> <lock-out>".into());
    };
    let read = |path: &str| std::fs::read(path).map_err(|error| format!("{path}: {error}"));
    let text = String::from_utf8(read(inputs)?).map_err(|_| "inputs are not text")?;
    let pads = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| u16::from_str_radix(line, 16).map_err(|_| format!("bad input {line}")))
        .collect::<Result<Vec<u16>, String>>()?;
    let frames: usize = frames.parse().map_err(|_| "bad frame count")?;
    let mut engine = sm_host::replay::Engine::open(&read(content)?, &read(start)?)?;
    let mut running = Running::default();
    let started = std::time::Instant::now();
    for (index, &pad) in pads.iter().take(frames).enumerate() {
        engine.step(pad);
        running.fold(&Tick {
            index: index as u32,
            tick: index as i32,
            machine: engine.digests(),
            typed: None,
            session: None,
        });
    }
    std::fs::write(out, running.lock_toml()).map_err(|error| format!("{out}: {error}"))?;
    println!(
        "Replayed {} ticks in {:.2}s; failed: {}",
        frames.min(pads.len()),
        started.elapsed().as_secs_f64(),
        engine.failed()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
