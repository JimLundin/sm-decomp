//! Direct native fixtures with injected WRAM; not a gameplay replay or verdict.
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 6 {
        return Err("usage: room-transition-host-control ARCHIVE ROM OUTPUT FIXTURE_WRAM MODE".into());
    }
    let mode = args[5].to_str().ok_or("room-transition mode must be UTF-8")?;
    sm_native_host::room_transition_control::run(
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
        Path::new(&args[4]),
        mode,
    )
}
