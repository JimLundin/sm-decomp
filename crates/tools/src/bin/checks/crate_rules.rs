use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

const ENGINE: [&str; 4] = ["data", "game", "runtime", "app"];
const ADDRESSED: [&str; 3] = ["game", "runtime", "app"];
const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

struct Rule {
    name: &'static str,
    crates: &'static [&'static str],
    patterns: &'static [&'static str],
    word_start: bool,
    word_end: bool,
}

const RULES: [Rule; 4] = [
    Rule {
        name: "rule 4 (no probe or observer)",
        crates: &ENGINE,
        patterns: &["[Pp]robes?", "[Oo]bservers?"],
        word_start: false,
        word_end: true,
    },
    Rule {
        name: "rule 5 (no SNES address literal)",
        crates: &ADDRESSED,
        patterns: &[
            "0x7[EeFf]_?%%%%",
            "$%%:?%%%%",
            "0x[89ABCDabcd]%_?[89ABCDEFabcdef]%%%",
            "0x21%%",
            "0x42%%",
            "0x43[01234567]%",
        ],
        word_start: false,
        word_end: true,
    },
    Rule {
        name: "K8 (no expect)",
        crates: &ENGINE,
        patterns: &["#!?\\[ expect ("],
        word_start: false,
        word_end: false,
    },
    Rule {
        name: "K9 (no routine-label names)",
        crates: &ENGINE,
        patterns: &["func_%%_?%%%%", "sub_%%_?%%%%", "loc_%%_?%%%%"],
        word_start: true,
        word_end: true,
    },
];

const fn word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || !byte.is_ascii()
}

fn at(text: &[u8], position: usize, pattern: &[u8]) -> Option<usize> {
    let Some((&first, rest)) = pattern.split_first() else {
        return Some(position);
    };
    let next = text.get(position).copied();
    match first {
        b'%' => next
            .filter(u8::is_ascii_hexdigit)
            .and_then(|_| at(text, position.checked_add(1)?, rest)),
        b' ' => {
            let spaces = text
                .get(position..)?
                .iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count();
            at(text, position.checked_add(spaces)?, rest)
        }
        b'[' => {
            let close = rest.iter().position(|&byte| byte == b']')?;
            let (set, after) = rest.split_at(close);
            next.filter(|byte| set.contains(byte))
                .and_then(|_| at(text, position.checked_add(1)?, after.get(1..)?))
        }
        b'\\' => {
            let (&literal, after) = rest.split_first()?;
            (next == Some(literal)).then_some(())?;
            at(text, position.checked_add(1)?, after)
        }
        literal => {
            if let Some((b'?', after)) = rest.split_first() {
                let with = (next == Some(literal))
                    .then(|| at(text, position.checked_add(1)?, after))
                    .flatten();
                return with.or_else(|| at(text, position, after));
            }
            (next == Some(literal)).then_some(())?;
            at(text, position.checked_add(1)?, rest)
        }
    }
}

fn matches(rule: &Rule, line: &str) -> bool {
    let text = line.as_bytes();
    (0..text.len()).any(|start| {
        let starts_word = start == 0 || text.get(start.wrapping_sub(1)).is_some_and(|&b| !word(b));
        if rule.word_start && !starts_word {
            return false;
        }
        rule.patterns.iter().any(|pattern| {
            at(text, start, pattern.as_bytes())
                .is_some_and(|end| !rule.word_end || text.get(end).is_none_or(|&b| !word(b)))
        })
    })
}

fn rust_files(directory: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{}: {e}", directory.display()))?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            rust_files(&path, out)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn commented(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

fn scan(root: &Path, findings: &mut Vec<String>) -> Result<(), String> {
    for rule in &RULES {
        for krate in rule.crates {
            let directory = root.join("crates").join(krate);
            if !directory.is_dir() {
                continue;
            }
            let mut files = Vec::new();
            rust_files(&directory, &mut files)?;
            for file in files {
                let text =
                    fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
                let shown = file
                    .strip_prefix(root)
                    .unwrap_or(&file)
                    .display()
                    .to_string();
                for (number, line) in (1_usize..).zip(text.lines()) {
                    if !commented(line) && matches(rule, line) {
                        findings.push(format!("{}: {shown}:{number}:{line}", rule.name));
                    }
                }
            }
        }
    }
    Ok(())
}

fn has_doc_address(line: &str) -> bool {
    let text = line.as_bytes();
    let hex = |position: usize| text.get(position).is_some_and(u8::is_ascii_hexdigit);
    let ends = |position: usize| !hex(position);
    text.iter().enumerate().any(|(index, &byte)| {
        let after = index.saturating_add(1);
        let address = byte == b'$'
            && ((hex(after)
                && hex(after.saturating_add(1))
                && text.get(after.saturating_add(2)) == Some(&b':')
                && (3..7).all(|offset| hex(after.saturating_add(offset))))
                || ((0..4).all(|offset| hex(after.saturating_add(offset)))
                    && ends(after.saturating_add(4))));
        let bank = |prefix: &[u8]| {
            text.get(index..)
                .is_some_and(|rest| rest.starts_with(prefix))
                && {
                    let mut position = index.saturating_add(prefix.len());
                    if text.get(position) == Some(&b'`') {
                        position = position.saturating_add(1);
                    }
                    text.get(position) == Some(&b'$')
                        && hex(position.saturating_add(1))
                        && hex(position.saturating_add(2))
                        && ends(position.saturating_add(3))
                }
        };
        address || bank(b"bank ") || bank(b"Bank ")
    })
}

fn doc_line(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    (trimmed.starts_with("///") || trimmed.starts_with("//!")) && !trimmed.starts_with("////")
}

fn public(next: &str) -> bool {
    let trimmed = next.trim_start_matches([' ', '\t']);
    if trimmed.starts_with("pub ") || trimmed.starts_with("pub(") {
        return true;
    }
    let mut chars = trimmed.chars();
    if !chars.next().is_some_and(|first| first.is_ascii_uppercase()) {
        return false;
    }
    let after = chars
        .as_str()
        .trim_start_matches(|c: char| c.is_ascii_alphanumeric() || c == '_');
    let after = after.trim_start_matches([' ', '\t']);
    after.is_empty() || after.starts_with(['(', '{', ',', '='])
}

struct DocRun {
    lines: usize,
    inner: bool,
    first: Option<(usize, String)>,
}

fn doc_addresses(file: &str, text: &str, findings: &mut Vec<String>) {
    let mut run = DocRun {
        lines: 0,
        inner: false,
        first: None,
    };
    let mut flush = |run: &mut DocRun, next: &str| {
        if run.lines > 0
            && let Some((number, line)) = run.first.take()
            && (run.inner || public(next))
        {
            findings.push(format!(
                "rule 6 (no address in a public doc): {file}:{number}: {line}"
            ));
        }
        run.lines = 0;
        run.first = None;
    };
    for (number, line) in (1_usize..).zip(text.lines()) {
        if doc_line(line) {
            if run.lines == 0 {
                run.inner = line.trim_start_matches([' ', '\t']).starts_with("//!");
            }
            run.lines = run.lines.saturating_add(1);
            if run.first.is_none() && has_doc_address(line) {
                run.first = Some((number, line.to_owned()));
            }
        } else if run.lines > 0 && !line.trim_start_matches([' ', '\t']).starts_with("#[") {
            flush(&mut run, line);
        }
    }
    flush(&mut run, "");
}

#[derive(Deserialize)]
struct Deny {
    bans: Bans,
}

#[derive(Deserialize)]
struct Bans {
    build: Build,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Build {
    allow_build_scripts: Vec<String>,
}

/// The parts of a JSON value `cargo metadata` output needs: numbers are skipped.
enum Json {
    Null,
    Bool,
    Number,
    Text(String),
    List(Vec<Self>),
    Object(Vec<(String, Self)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Self> {
        let Self::Object(fields) = self else {
            return None;
        };
        fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn text(&self) -> Option<&str> {
        let Self::Text(text) = self else {
            return None;
        };
        Some(text)
    }

    fn list(&self) -> &[Self] {
        let Self::List(items) = self else {
            return &[];
        };
        items
    }
}

struct Reader<'a> {
    text: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn peek(&mut self) -> Option<u8> {
        while self.text.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at = self.at.saturating_add(1);
        }
        self.text.get(self.at).copied()
    }

    fn take(&mut self, byte: u8) -> Result<(), String> {
        if self.peek() == Some(byte) {
            self.at = self.at.saturating_add(1);
            Ok(())
        } else {
            Err(format!(
                "cargo metadata: expected `{}` at byte {}",
                char::from(byte),
                self.at
            ))
        }
    }

    fn word(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self
            .text
            .get(self.at..)
            .is_some_and(|rest| rest.starts_with(word.as_bytes()))
        {
            self.at = self.at.saturating_add(word.len());
            Ok(value)
        } else {
            Err(format!(
                "cargo metadata: unreadable value at byte {}",
                self.at
            ))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.take(b'"')?;
        let mut out = Vec::new();
        loop {
            let byte = *self
                .text
                .get(self.at)
                .ok_or("cargo metadata: unterminated string")?;
            self.at = self.at.saturating_add(1);
            match byte {
                b'"' => return String::from_utf8(out).map_err(|e| format!("cargo metadata: {e}")),
                b'\\' => {
                    let escaped = *self
                        .text
                        .get(self.at)
                        .ok_or("cargo metadata: unterminated string")?;
                    self.at = self.at.saturating_add(1);
                    match escaped {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        // A code point stands as U+FFFD: no field the check reads holds one.
                        b'u' => {
                            self.at = self.at.saturating_add(4);
                            out.extend_from_slice("\u{fffd}".as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek() {
            Some(b'{') => {
                self.take(b'{')?;
                let mut fields = Vec::new();
                while self.peek() != Some(b'}') {
                    if !fields.is_empty() {
                        self.take(b',')?;
                    }
                    let key = self.string()?;
                    self.take(b':')?;
                    fields.push((key, self.value()?));
                }
                self.take(b'}')?;
                Ok(Json::Object(fields))
            }
            Some(b'[') => {
                self.take(b'[')?;
                let mut items = Vec::new();
                while self.peek() != Some(b']') {
                    if !items.is_empty() {
                        self.take(b',')?;
                    }
                    items.push(self.value()?);
                }
                self.take(b']')?;
                Ok(Json::List(items))
            }
            Some(b'"') => self.string().map(Json::Text),
            Some(b'n') => self.word("null", Json::Null),
            Some(b't') => self.word("true", Json::Bool),
            Some(b'f') => self.word("false", Json::Bool),
            Some(b'-' | b'0'..=b'9') => {
                while self
                    .text
                    .get(self.at)
                    .is_some_and(|byte| byte.is_ascii_digit() || b"+-.eE".contains(byte))
                {
                    self.at = self.at.saturating_add(1);
                }
                Ok(Json::Number)
            }
            _ => Err(format!(
                "cargo metadata: unreadable value at byte {}",
                self.at
            )),
        }
    }
}

fn toml_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The crate an `allow-build-scripts` entry names: cargo-deny takes `name`, `name@version` and
/// `name:requirement`.
fn allowed_name(entry: &str) -> &str {
    entry.split(['@', ':']).next().unwrap_or(entry)
}

/// Every package with a build script whose source is not crates.io, from `cargo metadata` output.
/// cargo-deny's exceptions match names and versions, which a path or git copy can share, so one
/// deny.toml allows is refused too.
fn build_script_findings(metadata: &str, allowed: &[String]) -> Result<Vec<String>, String> {
    let mut reader = Reader {
        text: metadata.as_bytes(),
        at: 0,
    };
    let metadata = reader.value()?;
    let mut findings = Vec::new();
    for package in metadata.get("packages").map_or(&[][..], Json::list) {
        let source = package.get("source").and_then(Json::text);
        let build = package
            .get("targets")
            .map_or(&[][..], Json::list)
            .iter()
            .any(|target| {
                target
                    .get("kind")
                    .map_or(&[][..], Json::list)
                    .iter()
                    .any(|kind| kind.text() == Some("custom-build"))
            });
        if source == Some(CRATES_IO) || !build {
            continue;
        }
        let name = package.get("name").and_then(Json::text).unwrap_or("");
        let version = package.get("version").and_then(Json::text).unwrap_or("");
        let what = if allowed.iter().any(|entry| allowed_name(entry) == name) {
            "build script allowed outside crates.io"
        } else {
            "build script outside crates.io"
        };
        findings.push(format!(
            "{what}: {name}@{version} from {}",
            source.unwrap_or("a path")
        ));
    }
    Ok(findings)
}

fn build_script_sources(root: &Path, findings: &mut Vec<String>) -> Result<(), String> {
    let deny: Deny = toml_file(&root.join("deny.toml"))?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = std::process::Command::new(cargo)
        .args([
            "metadata",
            "--locked",
            "--all-features",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata = String::from_utf8(output.stdout).map_err(|e| format!("cargo metadata: {e}"))?;
    findings.extend(build_script_findings(
        &metadata,
        &deny.bans.build.allow_build_scripts,
    )?);
    Ok(())
}

pub fn run(root: &Path) -> Result<Vec<String>, String> {
    let mut findings = Vec::new();
    build_script_sources(root, &mut findings)?;
    scan(root, &mut findings)?;
    let mut files = Vec::new();
    rust_files(&root.join("crates/data/src"), &mut files)?;
    for file in files {
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let shown = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .display()
            .to_string();
        doc_addresses(&shown, &text, &mut findings);
    }
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str) -> &'static Rule {
        RULES
            .iter()
            .find(|rule| rule.name.starts_with(name))
            .unwrap()
    }

    #[test]
    fn each_scan_finds_its_forms_and_passes_their_neighbours() {
        let cases: [(&str, &str, bool); 16] = [
            ("rule 4", "let probes = 1;", true),
            ("rule 4", "struct Observer;", true),
            ("rule 4", "let probed = 1;", false),
            ("rule 5", "const A: u32 = 0x7E_0AF6;", true),
            ("rule 5", "let a = 0x7f1234;", true),
            ("rule 5", "let a = 0x7E12345;", false),
            ("rule 5", "at $80:8000", true),
            ("rule 5", "let a = 0x9F_8000;", true),
            ("rule 5", "let a = 0x2100;", true),
            ("rule 5", "let a = 0x4380;", false),
            ("rule 5", "let a = 0x4372;", true),
            ("K8", "#![ expect (clippy::x)]", true),
            ("K8", "#[expected]", false),
            ("K9", "fn func_80_8000() {}", true),
            ("K9", "fn my_func_808000() {}", false),
            ("K9", "fn loc_8080001() {}", false),
        ];
        for (name, line, found) in cases {
            assert_eq!(matches(rule(name), line), found, "{name}: {line}");
        }
    }

    #[test]
    fn a_public_doc_with_an_address_is_found_and_a_private_one_is_not() {
        let text = "/// Reads $7E:0AF6.\n#[derive(Debug)]\npub struct A;\n/// Reads $0AF6.\nfn b() {}\n\
                    /// Bank `$80` holds it.\nVariant,\n//! Module at bank $8F.\n";
        let mut findings = Vec::new();
        doc_addresses("a.rs", text, &mut findings);
        assert_eq!(
            findings,
            [
                "rule 6 (no address in a public doc): a.rs:1: /// Reads $7E:0AF6.",
                "rule 6 (no address in a public doc): a.rs:6: /// Bank `$80` holds it.",
                "rule 6 (no address in a public doc): a.rs:8: //! Module at bank $8F.",
            ]
        );
    }

    #[test]
    fn a_build_script_outside_crates_io_is_found_by_its_name_allowed_or_not() {
        let metadata = r#"{"packages": [
            {"name": "serde", "version": "1.0.229", "source": "registry+https://github.com/rust-lang/crates.io-index",
             "targets": [{"kind": ["custom-build"], "edition": "2021", "doctest": true, "n": -1.5e3}]},
            {"name": "quote", "version": "9.9.9", "source": "git+https://example.com/quote?rev=1#abc",
             "targets": [{"kind": ["lib"]}, {"kind": ["custom-build"]}]},
            {"name": "proc-macro2", "version": "1.0.107", "source": null, "targets": [{"kind": ["custom-build"]}]},
            {"name": "other", "version": "0.1.0", "source": "git+https://example.com/other#def",
             "targets": [{"kind": ["custom-build"]}]},
            {"name": "plain", "version": "0.1.0", "source": null, "targets": [{"kind": ["lib"]}]},
            {"name": "say \"hi\"\u00e9", "version": "0.1.0", "source": null, "targets": []}
        ], "version": 1}"#;
        let allowed = [
            "serde@1.0.229".to_owned(),
            "quote".to_owned(),
            "proc-macro2:^1".to_owned(),
        ];
        assert_eq!(
            build_script_findings(metadata, &allowed).unwrap(),
            [
                "build script allowed outside crates.io: quote@9.9.9 from git+https://example.com/quote?rev=1#abc",
                "build script allowed outside crates.io: proc-macro2@1.0.107 from a path",
                "build script outside crates.io: other@0.1.0 from git+https://example.com/other#def",
            ]
        );
    }

    #[test]
    fn unreadable_metadata_is_an_error() {
        assert!(build_script_findings(r#"{"packages": [}"#, &[]).is_err());
        assert!(build_script_findings(r#"{"packages": ["#, &[]).is_err());
    }
}
