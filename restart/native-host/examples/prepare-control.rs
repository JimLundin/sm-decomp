//! The full control is also callable through the local Rust DLL adapter.
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = if args.len() == 3 {
        std::env::current_exe().map_err(|error| error.to_string()).and_then(|producer| {
            sm_native_host::preparation_control::run(
                std::path::Path::new(&args[0]),
                std::path::Path::new(&args[1]),
                std::path::Path::new(&args[2]),
                &producer,
            )
        })
    } else {
        Err("usage: prepare-control PINNED_ARCHIVE FRESH_OUTPUT PUBLIC_RUNTIME".into())
    };
    if let Err(error) = result {
        eprintln!("production preparation control failed: {error}");
        std::process::exit(1);
    }
}
