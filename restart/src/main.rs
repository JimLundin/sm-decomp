mod inputs;
use sm_restart::runtime_cli;

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "Usage: sm-restart doctor --rom PATH --reference-archive PATH
       sm-restart prepare --reference-archive PATH --out DIRECTORY
       sm-restart run --library PATH --rom PATH --out DIRECTORY [OPTIONS]
       sm-restart play --library PATH --rom PATH --out DIRECTORY [OPTIONS]

Run options: --scenario boot|ceres|parlor|morph|missile|energy|none, --inputs PATH, --steps N,
  --tail-policy parlor|morph|missile|energy (append bounded input policy to its exact retained prefix),
             --deadline-seconds N, --capture-every N,
             --present no|replay|interactive, --sdl-library PATH,
             --gameplay c|rust (projectiles, scrolling, Ceres doors, room transitions, directional motion, camera and item instructions).
             --item-instructions c|rust overrides item ownership for a paired comparison.
             --morph-instructions c|rust overrides only Morph ownership (defaults to item ownership).
             --tank-instructions c|rust overrides exposed tank ownership (defaults to item ownership).
             --player-motion c|rust selects vertical movement and solid-enemy contact (defaults to c).
             --persistence c|rust selects the bounded SRAM codec (defaults to gameplay ownership).
             --waver c|rust selects Waver actor logic (defaults to c).
             --ripper c|rust selects Ripper variants' initialization and patrol (defaults to c).
             --rio c|rust selects ordinary Rio initialization, attack and recovery (defaults to c).
             --rio-trace-start N --rio-trace-steps N retains a bounded host-step interval (maximum 2048 steps).
             --trace yes records complete per-step WRAM, external state and ordered effects hashes.
             --trace-reference PATH --trace-reference-sha256 SHA compares an authenticated reference.
             Trace requires exact recorded inputs, explicit steps, no tail policy and no presentation.
Output directories must be fresh. Play uses interactive presentation.

Doctor checks the pinned ROM size/SHA-256 and recovered-C archive SHA-256.
Inputs are read only. Extracted source is not validated. Gameplay is not tested.

Exit codes: 0 = command succeeded (or help), 1 = execution failed, 2 = usage error.";

enum Command {
    Help,
    Doctor { rom: PathBuf, reference_archive: PathBuf },
}

fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("--help" | "-h") if args.next().is_none() => return Ok(Command::Help),
        Some("doctor") => {}
        _ => return Err("expected the doctor command".into()),
    }

    let mut rom = None;
    let mut reference_archive = None;
    while let Some(option) = args.next() {
        let destination = match option.to_str() {
            Some("--help" | "-h") if rom.is_none() && reference_archive.is_none() && args.next().is_none() => {
                return Ok(Command::Help);
            }
            Some("--rom") => &mut rom,
            Some("--reference-archive") => &mut reference_archive,
            _ => return Err(format!("unknown argument: {}", option.to_string_lossy())),
        };
        if destination.is_some() {
            return Err(format!("duplicate option: {}", option.to_string_lossy()));
        }
        let path = args
            .next()
            .filter(|value| !value.to_string_lossy().starts_with('-'))
            .ok_or_else(|| format!("{} requires a path", option.to_string_lossy()))?;
        *destination = Some(PathBuf::from(path));
    }
    Ok(Command::Doctor {
        rom: rom.ok_or("missing --rom PATH")?,
        reference_archive: reference_archive.ok_or("missing --reference-archive PATH")?,
    })
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if matches!(args.first().and_then(|s| s.to_str()), Some("prepare" | "run" | "play" | "run-worker")) {
        if args.len() == 2 && matches!(args[1].to_str(), Some("--help" | "-h")) {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        if let Err(error) = runtime_cli::validate(args.clone()) {
            eprintln!("{error}\n\n{USAGE}");
            return ExitCode::from(2);
        }
        return match runtime_cli::run(args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        };
    }
    match parse_args(env::args_os().skip(1)) {
        Ok(Command::Help) => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}\n\n{USAGE}");
            ExitCode::from(2)
        }
        Ok(Command::Doctor { rom, reference_archive }) => {
            let mut passed = true;
            for (label, path, pin) in
                [("ROM", rom, &inputs::ROM), ("Recovered C archive", reference_archive, &inputs::REFERENCE_ARCHIVE)]
            {
                match inputs::check_file(&path, pin) {
                    Ok(identity) => {
                        println!("{label}: verified ({} bytes; SHA-256 {})", identity.bytes, identity.sha256)
                    }
                    Err(error) => {
                        println!("{label}: failed");
                        eprintln!("{label} ({}): {error}", path.display());
                        passed = false;
                    }
                }
            }
            println!("Recovered C revision pin: {} (archive identity only)", inputs::REFERENCE_REVISION);
            println!("Extracted C source: not validated");
            println!("Gameplay: not tested");
            println!("Input checks: {}", if passed { "passed" } else { "failed" });
            if passed { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
    }
}
