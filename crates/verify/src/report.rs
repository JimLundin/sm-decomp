//! The generated README: progress over the milestone's movie from the reviewed played runs.

mod readme;
pub(crate) mod seeds;

#[cfg(test)]
mod tests;

use std::fs;
use std::path::Path;

/// The command line `run` takes.
pub const USAGE: &str = "tools report --write | --check";
/// The one command that regenerates the committed README, from the repository root.
pub const COMMAND: &str = "cargo run -p tools --bin tools -- report --write";

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The committed README's staleness: an error naming the command that regenerates it when it
/// differs from what the tree at `root` generates.
///
/// # Errors
///
/// The regeneration command when the README is stale or missing, or why it can't be generated.
pub fn check(root: &Path) -> Result<(), String> {
    let fresh = readme::page(root)?;
    let committed = fs::read_to_string(root.join(readme::PATH)).unwrap_or_default();
    current_at(readme::PATH, &committed, &fresh)
}

/// Whether the committed page at `path` is the fresh one; CRLF line endings don't count.
fn current_at(path: &str, committed: &str, fresh: &str) -> Result<(), String> {
    if committed.replace("\r\n", "\n") == fresh {
        Ok(())
    } else {
        Err(format!(
            "{path} is stale: regenerate it with `{COMMAND}` from the repository root"
        ))
    }
}

/// With `--write`, writes the README for the tree in the current directory; with `--check`,
/// fails when the README is stale.
///
/// # Errors
///
/// The usage on a wrong command line, why the README can't be generated or written, or
/// [`check`]'s error.
pub fn run(args: &[String]) -> Result<String, String> {
    let root = Path::new(".");
    match args {
        [flag] if flag == "--write" => {
            let page = readme::page(root)?;
            fs::write(root.join(readme::PATH), page)
                .map_err(|e| format!("{}: {e}", readme::PATH))?;
            Ok(format!("wrote {}\n", readme::PATH))
        }
        [flag] if flag == "--check" => {
            check(root).map(|()| format!("{} is current\n", readme::PATH))
        }
        _ => Err(USAGE.to_owned()),
    }
}
