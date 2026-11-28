//! The repository's source checks that no standard tool holds: the crate plan's rules and the
//! documents' relative links.

mod crate_rules;
mod doc_links;

use std::io::Write as _;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: checks crate-rules\n       checks doc-links [FILE.md...]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command] if command == "crate-rules" => crate_rules::run(Path::new(".")),
        [command, files @ ..] if command == "doc-links" => doc_links::run(Path::new("."), files),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(findings) if findings.is_empty() => ExitCode::SUCCESS,
        Ok(findings) => {
            let _ = std::io::stdout().write_all(findings.join("\n").as_bytes());
            let _ = writeln!(std::io::stdout());
            ExitCode::FAILURE
        }
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "checks: {error}");
            ExitCode::from(2)
        }
    }
}
