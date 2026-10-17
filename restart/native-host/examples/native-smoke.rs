use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 {
        return Err("usage: smoke ARCHIVE ROM OUTPUT".into());
    }
    sm_native_host::smoke_control::run(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]))
}
