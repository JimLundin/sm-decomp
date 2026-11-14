//! The clean room's scans (MODEL §5, "The clean room is enforced by construction").
//!
//! `cleanroom scan [--lexicon DIR] [--allow FILE] PATH...`: checks 1 to 3 on files, directories
//! (recursively) or `-` (standard input, for commit messages and briefs). Exit 1 on a finding.
//!
//! `cleanroom mirror [--lexicon DIR] SPEC DECODE MODEL...`: check 4, on a spec, its decode notes
//! (or `-` for none) and the model's Rust files or directories.
//!
//! `cleanroom lexicon OUT SALT --logs FILE... --rust FILE... --notes FILE... --words FILE`: the
//! observer side. Builds the hashed lexicon from the bank logs, `verify`'s decoders and
//! sources, the decode notes and a list of distinctive single words. Nothing it reads is
//! written out unhashed.

mod lexicon;
mod mirror;
mod scan;
mod text;

use std::fmt::Write as _;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use lexicon::{Lexicon, Plain};
use scan::{Allow, Class, Finding};

const USAGE: &str = "usage: cleanroom scan [--lexicon DIR] [--allow FILE] PATH...\n       \
                     cleanroom mirror [--lexicon DIR] SPEC DECODE MODEL...\n       \
                     cleanroom lexicon OUT SALT --logs FILE... --rust FILE... --notes FILE... --words FILE";

const DEFAULT_LEXICON: &str = "tools/cleanroom/scan_lexicon";
const DEFAULT_ALLOW: &str = "tools/cleanroom/scan_allow.txt";

/// All files under `path`, or `path` itself; acquisition errors are never exclusions.
fn files(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if fs::metadata(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .is_dir()
    {
        let mut entries = directory_entries(
            fs::read_dir(path)
                .map_err(|e| format!("{}: {e}", path.display()))?
                .map(|entry| entry.map(|entry| entry.path())),
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
        entries.sort();
        for entry in entries {
            files(&entry, out)?;
        }
    } else {
        out.push(path.to_owned());
    }
    Ok(())
}

fn directory_entries(
    entries: impl Iterator<Item = std::io::Result<PathBuf>>,
) -> std::io::Result<Vec<PathBuf>> {
    entries.collect()
}

/// Only directory-discovered non-source binary members are deliberately excluded.
fn source_text(
    path: &Path,
    explicit: bool,
    bytes: std::io::Result<Vec<u8>>,
) -> Result<Option<String>, String> {
    let bytes = bytes.map_err(|e| format!("{}: {e}", path.display()))?;
    match String::from_utf8(bytes) {
        Ok(src) => Ok(Some(src)),
        Err(e)
            if explicit
                || path
                    .extension()
                    .is_some_and(|e| matches!(e.to_str(), Some("rs" | "md" | "toml"))) =>
        {
            Err(format!("{}: {e}", path.display()))
        }
        Err(_) => Ok(None),
    }
}

fn read(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| format!("stdin: {e}"))?;
        return Ok(s);
    }
    fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

/// `--flag VALUE` pairs at the front of `args`, then the rest.
type Options<'a> = (Vec<(&'a str, &'a str)>, &'a [String]);

fn options<'a>(args: &'a [String], flags: &[&str]) -> Result<Options<'a>, String> {
    let mut found = Vec::new();
    let mut rest = args;
    while let Some(flag) = rest.first().filter(|flag| flag.starts_with("--")) {
        let [_, value, tail @ ..] = rest else {
            return Err(USAGE.to_owned());
        };
        if !flags.contains(&flag.as_str()) || value.starts_with("--") {
            return Err(USAGE.to_owned());
        }
        found.push((flag.as_str(), value.as_str()));
        rest = tail;
    }
    if rest.iter().any(|arg| arg.starts_with("--")) {
        return Err(USAGE.to_owned());
    }
    Ok((found, rest))
}

fn option<'a>(found: &[(&str, &'a str)], flag: &str, default: &'a str) -> &'a str {
    found
        .iter()
        .find(|(f, _)| *f == flag)
        .map_or(default, |(_, v)| v)
}

fn report(findings: &[Finding], out: &mut String) {
    for f in findings {
        let _ = writeln!(out, "{}:{}: {}: {}", f.path, f.line, f.class, f.detail);
    }
    let counts: Vec<String> = Class::ALL
        .iter()
        .map(|class| {
            format!(
                "{class} {}",
                findings.iter().filter(|f| f.class == *class).count()
            )
        })
        .collect();
    let _ = writeln!(out, "counts: {}", counts.join(", "));
}

fn run_scan(args: &[String]) -> Result<(String, bool), String> {
    let (opts, paths) = options(args, &["--lexicon", "--allow"])?;
    if paths.is_empty() {
        return Err(USAGE.to_owned());
    }
    let lex = Lexicon::load(Path::new(option(&opts, "--lexicon", DEFAULT_LEXICON)))?;
    let allow = Allow::parse(&read(option(&opts, "--allow", DEFAULT_ALLOW))?);
    let mut findings = Vec::new();
    for path in paths {
        if path == "-" {
            scan::file(&lex, &allow, "-", &read("-")?, &mut findings);
            continue;
        }
        let mut list = Vec::new();
        files(Path::new(path), &mut list)?;
        for file in list {
            let Some(src) = source_text(&file, file == Path::new(path), fs::read(&file))? else {
                continue;
            };
            scan::file(
                &lex,
                &allow,
                &file.display().to_string(),
                &src,
                &mut findings,
            );
        }
    }
    let mut out = String::new();
    report(&findings, &mut out);
    Ok((out, !findings.is_empty()))
}

fn run_mirror(args: &[String]) -> Result<(String, bool), String> {
    let (opts, rest) = options(args, &["--lexicon"])?;
    let [spec, decode, model @ ..] = rest else {
        return Err(USAGE.to_owned());
    };
    let lex = Lexicon::load(Path::new(option(&opts, "--lexicon", DEFAULT_LEXICON)))?;
    let decode = if decode == "-" {
        String::new()
    } else {
        read(decode)?
    };
    let mut sources = Vec::new();
    for path in model {
        let mut list = Vec::new();
        files(Path::new(path), &mut list)?;
        for file in list
            .into_iter()
            .filter(|f| f.extension().is_some_and(|e| e == "rs"))
        {
            sources.push((
                file.display().to_string(),
                read(&file.display().to_string())?,
            ));
        }
    }
    let mut failed = false;
    let (findings, summary) =
        mirror::check(&lex, spec, &read(spec)?, &decode, &sources, &mut failed);
    let mut out = String::new();
    report(&findings, &mut out);
    out.push_str(&summary);
    Ok((out, failed))
}

fn run_lexicon(args: &[String]) -> Result<(String, bool), String> {
    let [out_dir, salt, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let mut awaiting_input = false;
    for arg in rest {
        if arg.starts_with("--") {
            if awaiting_input
                || !matches!(arg.as_str(), "--logs" | "--rust" | "--notes" | "--words")
            {
                return Err(USAGE.to_owned());
            }
            awaiting_input = true;
        } else {
            awaiting_input = false;
        }
    }
    if awaiting_input {
        return Err(USAGE.to_owned());
    }
    let mut plain = Plain::default();
    let mut mode = "";
    for arg in rest {
        if arg.starts_with("--") {
            mode = arg.as_str();
            continue;
        }
        let src = read(arg)?;
        match mode {
            "--logs" => plain.bank_log(&src),
            "--rust" => plain.observer_rust(&src),
            "--notes" => plain.decode_notes(&src),
            "--words" => src
                .lines()
                .map(|l| l.split('#').next().unwrap_or_default().trim())
                .filter(|w| !w.is_empty())
                .for_each(|w| plain.distinctive(w)),
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok((plain.write(Path::new(out_dir), salt)?, false))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [command, rest @ ..] if command == "scan" => run_scan(rest),
        [command, rest @ ..] if command == "mirror" => run_mirror(rest),
        [command, rest @ ..] if command == "lexicon" => run_lexicon(rest),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok((out, failed)) => {
            let _ = std::io::stdout().write_all(out.as_bytes());
            if failed {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            let _ = writeln!(std::io::stderr(), "{e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        DEFAULT_ALLOW, DEFAULT_LEXICON, directory_entries, files, options, run_lexicon, run_scan,
        scan, source_text,
    };
    use crate::lexicon::Lexicon;
    use crate::scan::{Allow, Class};

    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
    }

    fn findings(name: &str, src: &str) -> Vec<Class> {
        let lex = Lexicon::load(&root().join(DEFAULT_LEXICON)).unwrap();
        let allow = Allow::parse(&std::fs::read_to_string(root().join(DEFAULT_ALLOW)).unwrap());
        let mut out = Vec::new();
        scan::file(&lex, &allow, name, src, &mut out);
        out.into_iter().map(|f| f.class).collect()
    }

    #[test]
    fn clean_fixtures_pass() {
        let mut list = Vec::new();
        files(&root().join("tools/cleanroom/scan_fixtures"), &mut list).unwrap();
        assert_eq!(list.len(), 2);
        for file in list {
            let src = std::fs::read_to_string(&file).unwrap();
            assert_eq!(
                findings(&file.display().to_string(), &src),
                [],
                "{}",
                file.display()
            );
        }
    }

    #[test]
    fn address_forms_are_caught() {
        let classes = findings(
            "x.md",
            "Source::bank(0x12, 0x3456), $12:3456, $3456, bank $12",
        );
        let forms = classes.iter().filter(|c| **c == Class::AddressForm).count();
        assert_eq!(forms, 5);
    }

    #[test]
    fn routes_are_caught() {
        assert_eq!(findings("x.md", "read the bank logs"), [Class::Route]);
    }

    #[test]
    fn acquisition_errors_cannot_certify_clean_inputs() {
        let missing = root().join("missing-scanner-input.rs");
        assert!(files(&missing, &mut Vec::new()).is_err());
        let entries = [
            Ok(missing),
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
        ];
        assert!(directory_entries(entries.into_iter()).is_err());
        for explicit in [false, true] {
            assert!(
                source_text(
                    Path::new("input.bin"),
                    explicit,
                    Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
                )
                .is_err()
            );
        }
    }

    #[test]
    fn binary_exclusion_is_only_for_discovered_non_source_members() {
        assert_eq!(
            source_text(Path::new("asset.bin"), false, Ok(vec![255])).unwrap(),
            None
        );
        assert!(source_text(Path::new("asset.bin"), true, Ok(vec![255])).is_err());
        for name in ["input.rs", "input.md", "input.toml"] {
            assert!(source_text(Path::new(name), false, Ok(vec![255])).is_err());
        }
        let text = source_text(
            Path::new("input.md"),
            false,
            Ok(b"read the bank logs".to_vec()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(findings("input.md", &text), [Class::Route]);
    }

    #[test]
    fn options_require_operands() {
        for args in [
            vec!["--lexicon"],
            vec!["--allow", "--lexicon"],
            vec!["input.md", "--allow"],
            vec!["--unknown", "value"],
        ] {
            let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
            assert!(options(&args, &["--lexicon", "--allow"]).is_err());
        }
        for flag in ["--logs", "--rust", "--notes", "--words", "--unknown"] {
            let args = [
                "unused-output".to_owned(),
                "salt".to_owned(),
                flag.to_owned(),
            ];
            assert!(run_lexicon(&args).is_err());
        }
    }

    #[test]
    fn an_explicit_missing_input_fails_the_command() {
        let args = [
            "--lexicon".to_owned(),
            root().join(DEFAULT_LEXICON).display().to_string(),
            "--allow".to_owned(),
            root().join(DEFAULT_ALLOW).display().to_string(),
            root()
                .join("missing-scanner-input.rs")
                .display()
                .to_string(),
        ];
        assert!(run_scan(&args).is_err());
    }
}
