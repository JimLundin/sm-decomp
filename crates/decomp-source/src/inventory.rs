//! Lexical C definition/dependency discovery; this never grants translation admission.
use crate::{Source, context, verify_source};
use anyhow::{Result, ensure};
use bounded::digest;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use tool_execution::bounded;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Routine {
    pub symbol: String,
    pub source: String,
    pub line: usize,
    pub original_address: Option<String>,
    pub signature: String,
    pub body_lines: usize,
    pub definition_sha256: String,
    pub calls: Vec<String>,
    pub indirect_indexed_calls: Vec<String>,
    pub defined_dependencies: Vec<String>,
    pub external_or_macro_calls: Vec<String>,
    pub translation_status: String,
}
#[cfg(test)]
fn clean(text: &str) -> Vec<u8> {
    clean_with(text, &mut || Ok(())).expect("unbounded lexical cleanup is infallible")
}

pub(crate) fn clean_bounded(text: &str) -> Result<Vec<u8>> {
    ensure!(text.len() as u64 <= bounded::MAX_DOCUMENT_BYTES, "lexical C source extent exceeded");
    clean_with(text, &mut bounded::check)
}

fn clean_with(text: &str, check: &mut impl FnMut() -> Result<()>) -> Result<Vec<u8>> {
    check()?;
    let bytes = text.as_bytes();
    let mut result = bytes.to_vec();
    let mut i = 0;
    let mut next_check = 0;
    while i < bytes.len() {
        if i >= next_check {
            check()?;
            next_check = i + 64 * 1024;
        }
        let end = if bytes[i..].starts_with(b"/*") {
            bytes[i + 2..].windows(2).position(|v| v == b"*/").map(|p| i + 4 + p).unwrap_or(bytes.len())
        } else if bytes[i..].starts_with(b"//") {
            bytes[i..].iter().position(|v| *v == b'\n').map(|p| i + p).unwrap_or(bytes.len())
        } else if b"'\"".contains(&bytes[i]) {
            let quote = bytes[i];
            let mut j = i + 1;
            while j < bytes.len() {
                if j.is_multiple_of(64 * 1024) {
                    check()?;
                }
                if bytes[j] == b'\\' {
                    j += 2;
                } else if bytes[j] == quote {
                    j += 1;
                    break;
                } else {
                    j += 1;
                }
            }
            j.min(bytes.len())
        } else {
            i += 1;
            continue;
        };
        for value in &mut result[i..end] {
            if *value != b'\n' {
                *value = b' ';
            }
        }
        i = end;
    }
    // Directives are not C declarations. In particular, a function-like macro
    // must not supply the name of the first function body later in the file.
    let mut continuation = false;
    for line in result.split_mut(|value| *value == b'\n') {
        check()?;
        let directive = continuation || line.iter().find(|value| !value.is_ascii_whitespace()) == Some(&b'#');
        continuation = directive && line.iter().rfind(|value| !value.is_ascii_whitespace()) == Some(&b'\\');
        if directive {
            line.fill(b' ');
        }
    }
    check()?;
    Ok(result)
}
#[cfg(test)]
fn names(bytes: &[u8]) -> (Vec<String>, Vec<String>) {
    names_with(bytes, usize::MAX, &mut || Ok(())).expect("unbounded lexical name discovery is infallible")
}

pub(crate) fn names_bounded(bytes: &[u8]) -> Result<(Vec<String>, Vec<String>)> {
    ensure!(bytes.len() as u64 <= bounded::MAX_DOCUMENT_BYTES, "lexical C definition extent exceeded");
    names_with(bytes, 65536, &mut bounded::check)
}

fn names_with(
    bytes: &[u8],
    maximum: usize,
    check: &mut impl FnMut() -> Result<()>,
) -> Result<(Vec<String>, Vec<String>)> {
    check()?;
    let mut calls = BTreeSet::new();
    let mut indexed = BTreeSet::new();
    let mut i = 0;
    let mut next_check = 0;
    while i < bytes.len() {
        if i >= next_check {
            check()?;
            next_check = i + 64 * 1024;
        }
        if !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            if i.is_multiple_of(64 * 1024) {
                check()?;
            }
            i += 1;
        }
        let name = String::from_utf8_lossy(&bytes[start..i]).into_owned();
        let mut after = i;
        while after < bytes.len() && bytes[after].is_ascii_whitespace() {
            if after.is_multiple_of(64 * 1024) {
                check()?;
            }
            after += 1;
        }
        if bytes.get(after) == Some(&b'(')
            && !["if", "for", "while", "switch", "sizeof", "_Alignof", "return"].contains(&name.as_str())
        {
            calls.insert(name);
        } else if bytes.get(after) == Some(&b'[') {
            while after < bytes.len() && !b";{}]".contains(&bytes[after]) {
                if after.is_multiple_of(64 * 1024) {
                    check()?;
                }
                after += 1;
            }
            if bytes.get(after) == Some(&b']') {
                after += 1;
                while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                    if after.is_multiple_of(64 * 1024) {
                        check()?;
                    }
                    after += 1;
                }
                if bytes.get(after) == Some(&b'(') {
                    indexed.insert(name);
                }
            }
        }
        ensure!(calls.len() + indexed.len() <= maximum, "lexical C dependency inventory exceeded");
    }
    check()?;
    Ok((calls.into_iter().collect(), indexed.into_iter().collect()))
}
fn definitions(text: &str, source: &str) -> Result<Vec<Routine>> {
    bounded::check()?;
    ensure!(text.len() as u64 <= bounded::MAX_DOCUMENT_BYTES, "C definition source extent exceeded");
    let mut routines = Vec::new();
    let bytes = clean_bounded(text)?;
    let mut offset = 0;
    let mut body_end = 0;
    for line in text.split_inclusive('\n') {
        bounded::check()?;
        let end = offset + line.len();
        if offset < body_end || bytes[offset..end].iter().all(|value| value.is_ascii_whitespace()) {
            offset = end;
            continue;
        }
        // A signature may span lines. Bound the search at the first body or
        // declaration terminator so prototypes cannot absorb a later body.
        let boundary = bytes[offset..].iter().position(|c| b";{}".contains(c)).map(|at| offset + at + 1).unwrap_or(end);
        let clean_line = &bytes[offset..boundary];
        let open = clean_line.iter().position(|c| *c == b'(');
        let brace = clean_line.iter().position(|c| *c == b'{');
        if let (Some(open), Some(brace)) = (open, brace) {
            let prefix = String::from_utf8_lossy(&clean_line[..open]);
            let prefix = prefix.trim();
            let symbol =
                prefix.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).rfind(|s| !s.is_empty()).unwrap_or("");
            if open < brace
                && !clean_line[..brace].contains(&b';')
                && crate::identifier(symbol)
                && prefix.len() > symbol.len()
                && prefix.bytes().all(|value| {
                    value.is_ascii_alphanumeric() || value == b'_' || value == b'*' || value.is_ascii_whitespace()
                })
                && !clean_line[..brace].contains(&b'=')
                && !["if", "for", "while", "switch"].contains(&symbol)
            {
                let start = offset + brace;
                let mut cursor = start + 1;
                let mut depth = 1;
                while cursor < bytes.len() && depth > 0 {
                    if cursor.is_multiple_of(64 * 1024) {
                        bounded::check()?;
                    }
                    if bytes[cursor] == b'{' {
                        depth += 1;
                    } else if bytes[cursor] == b'}' {
                        depth -= 1;
                    }
                    cursor += 1;
                }
                ensure!(depth == 0, "unbalanced C definition {source}:{symbol}");
                body_end = cursor;
                let (calls, indirect) = names_bounded(&bytes[start..cursor])?;
                let opening_line = text[start..].split('\n').next().unwrap_or("");
                let address = opening_line
                    .find("//")
                    .and_then(|at| {
                        opening_line[at + 2..].split_whitespace().find(|s| s.starts_with("0x") && s.len() == 8)
                    })
                    .map(str::to_owned);
                use sha2::{Digest, Sha256};
                routines.push(Routine {
                    symbol: symbol.into(),
                    source: source.into(),
                    line: text[..offset].bytes().filter(|v| *v == b'\n').count() + 1,
                    original_address: address,
                    signature: String::from_utf8_lossy(&bytes[offset..start])
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                    body_lines: bytes[start..cursor].iter().filter(|c| **c == b'\n').count() + 1,
                    definition_sha256: format!("{:x}", Sha256::digest(&text.as_bytes()[offset..cursor])),
                    calls,
                    indirect_indexed_calls: indirect,
                    defined_dependencies: vec![],
                    external_or_macro_calls: vec![],
                    translation_status: "unverified".into(),
                });
                ensure!(routines.len() <= 65536, "C definition inventory exceeded");
            }
        }
        offset = end;
    }
    Ok(routines)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallible_lexical_readers_preserve_facts_and_honor_inherited_deadline() -> Result<()> {
        let text = "Actual(); array[index](0); /* Hidden(); */ \"String();\";\n";
        assert_eq!(clean_bounded(text)?, clean(text));
        assert_eq!(names_bounded(&clean_bounded(text)?)?, names(&clean(text)));
        let error = bounded::until(0, 1024, || clean_bounded(text)).unwrap_err();
        assert!(error.to_string().contains("deadline"));
        let adversarial = "a[".repeat(100_000);
        let entered = std::time::Instant::now();
        let error = bounded::until(bounded::now_unix_ms()? + 20, bounded::DEFAULT_BYTES, || {
            names_bounded(adversarial.as_bytes())
        })
        .unwrap_err();
        assert!(error.to_string().contains("deadline"));
        assert!(entered.elapsed() < std::time::Duration::from_secs(1));
        Ok(())
    }

    #[test]
    fn inventory_generation_inherits_expired_budget_before_any_output() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let output = directory.0.join("out/inventory.toml");
        let error =
            inventory_until(&directory.0.join("absent"), &directory.0.join("absent-source"), 0, 1024).unwrap_err();
        assert!(error.to_string().contains("deadline"));
        assert!(!output.parent().unwrap().exists());
        Ok(())
    }

    #[test]
    fn parenthesized_return_dependency_facts_match_definition_inventory() {
        let text = "uint8 CheckBossBitForCurArea(uint16 a) { // 0x8081DC\n return (a & boss_bits_for_area[area_index]) != 0;\n}\n";
        let rows = definitions(text, "src/sm_80.c").unwrap();
        assert_eq!(rows.len(), 1);
        let masked = clean(text);
        let start = masked.iter().position(|b| *b == b'{').unwrap();
        let (admitted_calls, admitted_indirect) = names(&masked[start..]);
        assert_eq!(rows[0].calls, admitted_calls);
        assert_eq!(rows[0].indirect_indexed_calls, admitted_indirect);
        assert!(admitted_calls.is_empty());
    }

    #[test]
    fn comments_strings_and_nested_bodies_do_not_create_dependencies() {
        let text = "Real(); /* Fake(); } */ array[index](0); \"Ghost(); {\"; // Hidden();\nif (x) { Other(); }";
        let (calls, indexed) = names(&clean(text));
        assert_eq!(calls, ["Other", "Real"]);
        assert_eq!(indexed, ["array"]);
    }

    #[test]
    fn directives_prototypes_and_initializers_do_not_consume_definitions() {
        let text = "// Bank prelude\n#define Pointer (*(LongPtr*)RomPtr(0))\n\
                    #define Fake(x) \\\n  Helper(x)\n\
                    static const uint16 values[] = {1, 2};\n\
                    static uint8 Carry(void);\n\n\
                    void First(void) { // 0x808028\n  return (Helper());\n}\n\n\
                    const uint16 *Ridley_Instr_5(uint16 k,\n  const uint16 *jp) { // 0xA6E4BE\n  return jp;\n}\n\n\
                    static uint8 Carry(void) { return 1; }\n";
        let rows = definitions(text, "src/sm_a6.c").unwrap();
        assert_eq!(
            rows.iter().map(|row| row.symbol.as_str()).collect::<Vec<_>>(),
            ["First", "Ridley_Instr_5", "Carry"]
        );
        assert_eq!(rows[0].line, 8);
        assert_eq!(rows[0].signature, "void First(void)");
        assert_eq!(rows[0].calls, ["Helper"]);
        assert_eq!(rows[1].original_address.as_deref(), Some("0xA6E4BE"));
        assert_eq!(rows[1].signature, "const uint16 *Ridley_Instr_5(uint16 k, const uint16 *jp)");
    }

    #[test]
    fn pinned_ridley_callbacks_and_first_bank_function_are_indexed_exactly() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let bank = root.join("deps/snesrev-sm/src/sm_a6.c");
        if !bank.is_file() {
            return;
        }
        let text = bounded::text(bank).unwrap();
        let rows = definitions(&text, "src/sm_a6.c").unwrap();
        for symbol in ["Ridley_E90C", "Ridley_Func_40_Carry", "Ridley_Func_40_Sign"]
            .into_iter()
            .map(str::to_owned)
            .chain((1..=16).map(|number| format!("Ridley_Instr_{number}")))
        {
            let row = rows.iter().find(|row| row.symbol == symbol).unwrap_or_else(|| panic!("missing {symbol}"));
            assert!(text.lines().nth(row.line - 1).unwrap().contains(&symbol));
        }
        let text = bounded::text(root.join("deps/snesrev-sm/src/sm_80.c")).unwrap();
        let rows = definitions(&text, "src/sm_80.c").unwrap();
        assert_eq!(rows[0].symbol, "APU_UploadBank");
        assert_eq!(rows[0].line, 16);
        assert!(rows.iter().all(|row| !row.signature.contains('#') && !row.signature.contains('=')));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bank {
    pub definitions: usize,
    pub body_lines: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub schema_version: u32,
    pub scope: String,
    pub source_identity: Source,
    pub source_identity_sha256: String,
    pub definition_count: usize,
    pub banks: BTreeMap<String, Bank>,
    pub routines: Vec<Routine>,
}
pub fn inventory(root: &Path, source: &Path) -> Result<Inventory> {
    context::limited(|| inventory_bounded(root, source))
}
pub fn inventory_until(root: &Path, source: &Path, deadline: u64, max_bytes: u64) -> Result<Inventory> {
    context::until(deadline, max_bytes, || inventory_bounded(root, source))
}
fn inventory_bounded(root: &Path, source: &Path) -> Result<Inventory> {
    let identity = verify_source(root, source)?;
    let mut routines = Vec::new();
    let paths = bounded::entries(&source.join("src"))?;
    for path in paths {
        bounded::check()?;
        if path.extension().is_none_or(|e| e != "c")
            || path.file_stem().is_none_or(|n| !n.to_string_lossy().starts_with("sm_"))
        {
            continue;
        }
        let text = bounded::text(&path)?;
        routines.extend(definitions(&text, &path.strip_prefix(source)?.to_string_lossy())?);
        ensure!(routines.len() <= 65536, "source definition inventory exceeded");
    }
    let symbols = routines.iter().map(|r| r.symbol.clone()).collect::<BTreeSet<_>>();
    for routine in &mut routines {
        bounded::check()?;
        routine.defined_dependencies = routine.calls.iter().filter(|c| symbols.contains(*c)).cloned().collect();
        routine.external_or_macro_calls = routine.calls.iter().filter(|c| !symbols.contains(*c)).cloned().collect();
    }
    let mut banks = BTreeMap::new();
    for routine in &routines {
        bounded::check()?;
        let bank = banks.entry(routine.source.clone()).or_insert(Bank { definitions: 0, body_lines: 0 });
        bank.definitions += 1;
        bank.body_lines += routine.body_lines;
    }
    Ok(Inventory { schema_version: 1, scope: "actual sm_*.c lexical definitions/calls; macros, externals and indirect calls require review; no acceptance inferred".into(), source_identity: identity, source_identity_sha256: digest(&root.join("tools/dependencies.toml"))?, definition_count: routines.len(), banks, routines })
}
