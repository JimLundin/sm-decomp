//! The oracle harness's command line; see `oracle::cli`.

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(oracle::cli::main(&arguments));
}
