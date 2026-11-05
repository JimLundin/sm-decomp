//! The slice's headless tool: no window, sound device or comparison backend.
//!
//! `sm_replay prepare <rom> <exported start> <content out> <start out>`
//! `sm_replay gate <content> <start> <inputs> <ticks> <lock out>`
use sm_data::prepared::{PreparedContent, PreparedStart};
use sm_tools::slice::{Schedule, run};

fn read(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|error| format!("{path}: {error}"))
}

fn write(path: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|error| format!("{path}: {error}"))
}

fn prepare(rom: &str, exported: &str, content_out: &str, start_out: &str) -> Result<(), String> {
    let content = PreparedContent::prepare(read(rom)?);
    let start = PreparedStart::seal(&content, read(exported)?);
    write(content_out, &content.encode())?;
    write(start_out, &start.encode())?;
    println!("Prepared content {:016x}", content.id());
    Ok(())
}

type Loaded = (PreparedContent, PreparedStart, Vec<u16>);

fn load(content: &str, start: &str, inputs: &str, ticks: &str) -> Result<Loaded, String> {
    let content = PreparedContent::decode(&read(content)?).map_err(|error| error.to_string())?;
    let start = PreparedStart::decode(&read(start)?).map_err(|error| error.to_string())?;
    let text = String::from_utf8(read(inputs)?).map_err(|_| "inputs are not text")?;
    let ticks: usize = ticks.parse().map_err(|_| "bad tick count")?;
    let pads = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| u16::from_str_radix(line, 16).map_err(|_| format!("bad input {line}")))
        .take(ticks)
        .collect::<Result<Vec<u16>, String>>()?;
    Ok((content, start, pads))
}

/// One replay and nothing else, for a diagnostic's digest log.
fn one(args: &[String]) -> Result<(), String> {
    let (content, start, inputs, ticks, out) = match args {
        [content, start, inputs, ticks] => (content, start, inputs, ticks, None),
        [content, start, inputs, ticks, out] => (content, start, inputs, ticks, Some(out)),
        _ => {
            return Err(
                "usage: sm_replay one <content> <start> <inputs> <ticks> [<digest lines out>]"
                    .into(),
            );
        }
    };
    let (content, start, pads) = load(content, start, inputs, ticks)?;
    let schedule = Schedule {
        digests: true,
        ..Schedule::default()
    };
    let outcome = run(&content, &start, &pads, schedule)?;
    if let Some(out) = out {
        let mut text = outcome.digest_lines.join("\n");
        text.push('\n');
        write(out, text.as_bytes())?;
    }
    Ok(())
}

fn gate(args: &[String]) -> Result<(), String> {
    let [content, start, inputs, ticks, out] = args else {
        return Err("usage: sm_replay gate <content> <start> <inputs> <ticks> <lock out>".into());
    };
    let (content, start, pads) = load(content, start, inputs, ticks)?;
    let timed = std::time::Instant::now();
    let base = run(
        &content,
        &start,
        &pads,
        Schedule {
            digests: true,
            ..Schedule::default()
        },
    )?;
    println!(
        "Replayed {} ticks in {:.2}s; {} audio batches, {} cues, {} saves; fault: {}",
        base.ticks,
        timed.elapsed().as_secs_f64(),
        base.audio_batches,
        base.cues,
        base.saves,
        base.faulted
    );
    write(out, base.lock_toml.as_bytes())?;
    if base.faulted {
        return Err("the engine recorded a fault".into());
    }
    if base.audio_batches != base.ticks {
        return Err("a Tick's sound batch was skipped or duplicated".into());
    }
    let schedules = [
        (
            "the same inputs again",
            Schedule {
                digests: true,
                ..Schedule::default()
            },
        ),
        (
            "presenting every Tick (a 60 Hz display)",
            Schedule {
                digests: true,
                present_every: 1,
                ..Schedule::default()
            },
        ),
        (
            "presenting every second Tick (30 Hz)",
            Schedule {
                digests: true,
                present_every: 2,
                ..Schedule::default()
            },
        ),
        (
            "presenting every seventh Tick",
            Schedule {
                digests: true,
                present_every: 7,
                ..Schedule::default()
            },
        ),
        (
            "the audio queue held full",
            Schedule {
                digests: true,
                audio_held_offers: 5,
                ..Schedule::default()
            },
        ),
        (
            "the save sink accepting later",
            Schedule {
                digests: true,
                save_held_offers: 40,
                ..Schedule::default()
            },
        ),
        (
            "both held, without the digests read",
            Schedule {
                audio_held_offers: 3,
                save_held_offers: 9,
                ..Schedule::default()
            },
        ),
    ];
    for (name, schedule) in schedules {
        let outcome = run(&content, &start, &pads, schedule)?;
        let same_game = !schedule.digests || outcome.lock_toml == base.lock_toml;
        let same_effects = outcome.effect_trace == base.effect_trace
            && outcome.audio_batches == base.audio_batches
            && outcome.cues == base.cues
            && outcome.saves == base.saves;
        println!(
            "{name}: game {}, effects {}; saves acknowledged {}/{}; at most {} Tick batch waiting",
            if same_game { "identical" } else { "DIFFERS" },
            if same_effects { "identical" } else { "DIFFER" },
            outcome.receipts,
            outcome.saves,
            outcome.most_waiting
        );
        if !same_game || !same_effects || outcome.receipts != outcome.saves {
            return Err(format!("{name} changed the run"));
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.split_first() {
        Some((command, rest)) if command == "prepare" => match rest {
            [rom, exported, content, start] => prepare(rom, exported, content, start),
            _ => Err(
                "usage: sm_replay prepare <rom> <exported start> <content out> <start out>".into(),
            ),
        },
        Some((command, rest)) if command == "gate" => gate(rest),
        Some((command, rest)) if command == "one" => one(rest),
        _ => Err("usage: sm_replay prepare|gate ...".into()),
    };
    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
