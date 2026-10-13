//! Lexical C definition/dependency discovery; this never grants translation admission.
use crate::{digest, family};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Serialize)]
struct Routine {
    symbol: String,
    source: String,
    line: usize,
    original_address: Option<String>,
    signature: String,
    body_lines: usize,
    definition_sha256: String,
    calls: Vec<String>,
    indirect_indexed_calls: Vec<String>,
    defined_dependencies: Vec<String>,
    external_or_macro_calls: Vec<String>,
    translation_status: String,
}
pub(crate) fn clean(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut result = bytes.to_vec();
    let mut i = 0;
    while i < bytes.len() {
        let end = if bytes[i..].starts_with(b"/*") {
            bytes[i + 2..].windows(2).position(|v| v == b"*/").map(|p| i + 4 + p).unwrap_or(bytes.len())
        } else if bytes[i..].starts_with(b"//") {
            bytes[i..].iter().position(|v| *v == b'\n').map(|p| i + p).unwrap_or(bytes.len())
        } else if b"'\"".contains(&bytes[i]) {
            let quote = bytes[i];
            let mut j = i + 1;
            while j < bytes.len() {
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
        let directive = continuation || line.iter().find(|value| !value.is_ascii_whitespace()) == Some(&b'#');
        continuation = directive && line.iter().rfind(|value| !value.is_ascii_whitespace()) == Some(&b'\\');
        if directive {
            line.fill(b' ');
        }
    }
    result
}
pub(crate) fn names(bytes: &[u8]) -> (Vec<String>, Vec<String>) {
    let mut calls = BTreeSet::new();
    let mut indexed = BTreeSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        let name = String::from_utf8_lossy(&bytes[start..i]).into_owned();
        let mut after = i;
        while after < bytes.len() && bytes[after].is_ascii_whitespace() {
            after += 1;
        }
        if bytes.get(after) == Some(&b'(')
            && !["if", "for", "while", "switch", "sizeof", "_Alignof", "return"].contains(&name.as_str())
        {
            calls.insert(name);
        } else if bytes.get(after) == Some(&b'[') {
            while after < bytes.len() && !b";{}]".contains(&bytes[after]) {
                after += 1;
            }
            if bytes.get(after) == Some(&b']') {
                after += 1;
                while after < bytes.len() && bytes[after].is_ascii_whitespace() {
                    after += 1;
                }
                if bytes.get(after) == Some(&b'(') {
                    indexed.insert(name);
                }
            }
        }
    }
    (calls.into_iter().collect(), indexed.into_iter().collect())
}
pub(crate) fn run(root: &Path, source: &Path, output: &Path) -> Result<()> {
    let identity = family::verify_source(root, source)?;
    let mut routines = Vec::new();
    let mut paths = fs::read_dir(source.join("src"))?.map(|e| Ok(e?.path())).collect::<Result<Vec<_>>>()?;
    paths.sort();
    for path in paths {
        if path.extension().is_none_or(|e| e != "c")
            || path.file_stem().is_none_or(|n| !n.to_string_lossy().starts_with("sm_"))
        {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        routines.extend(definitions(&text, &path.strip_prefix(source)?.to_string_lossy())?);
    }
    let symbols = routines.iter().map(|r| r.symbol.clone()).collect::<BTreeSet<_>>();
    for routine in &mut routines {
        routine.defined_dependencies = routine.calls.iter().filter(|c| symbols.contains(*c)).cloned().collect();
        routine.external_or_macro_calls = routine.calls.iter().filter(|c| !symbols.contains(*c)).cloned().collect();
    }
    #[derive(Serialize)]
    struct Bank {
        definitions: usize,
        body_lines: usize,
    }
    let mut banks = BTreeMap::new();
    for routine in &routines {
        let bank = banks.entry(routine.source.clone()).or_insert(Bank { definitions: 0, body_lines: 0 });
        bank.definitions += 1;
        bank.body_lines += routine.body_lines;
    }
    #[derive(Serialize)]
    struct Inventory {
        schema_version: u32,
        scope: String,
        source_identity: crate::family_model::Source,
        source_identity_sha256: String,
        definition_count: usize,
        banks: BTreeMap<String, Bank>,
        routines: Vec<Routine>,
    }
    fs::create_dir_all(output.parent().unwrap())?;
    let report=Inventory{schema_version:1,scope:"actual sm_*.c lexical definitions/calls; macros, externals and indirect calls require review; no acceptance inferred".into(),
        source_identity:identity,source_identity_sha256:digest(&root.join("tools/dependencies.toml"))?,definition_count:routines.len(),banks,routines};
    pipeline_model::write(output, &report)?;
    println!("Indexed {} definitions: {}", report.definition_count, output.display());
    Ok(())
}
fn definitions(text: &str, source: &str) -> Result<Vec<Routine>> {
    let mut routines = Vec::new();
    let bytes = clean(text);
    let mut offset = 0;
    let mut body_end = 0;
    for line in text.split_inclusive('\n') {
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
                && family::identifier(symbol)
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
                    if bytes[cursor] == b'{' {
                        depth += 1;
                    } else if bytes[cursor] == b'}' {
                        depth -= 1;
                    }
                    cursor += 1;
                }
                ensure!(depth == 0, "unbalanced C definition {source}:{symbol}");
                body_end = cursor;
                let (calls, indirect) = names(&bytes[start..cursor]);
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
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bank = root.join("deps/snesrev-sm/src/sm_a6.c");
        if !bank.is_file() {
            return;
        }
        let text = fs::read_to_string(bank).unwrap();
        let rows = definitions(&text, "src/sm_a6.c").unwrap();
        for symbol in ["Ridley_E90C", "Ridley_Func_40_Carry", "Ridley_Func_40_Sign"]
            .into_iter()
            .map(str::to_owned)
            .chain((1..=16).map(|number| format!("Ridley_Instr_{number}")))
        {
            let row = rows.iter().find(|row| row.symbol == symbol).unwrap_or_else(|| panic!("missing {symbol}"));
            assert!(text.lines().nth(row.line - 1).unwrap().contains(&symbol));
        }
        let text = fs::read_to_string(root.join("deps/snesrev-sm/src/sm_80.c")).unwrap();
        let rows = definitions(&text, "src/sm_80.c").unwrap();
        assert_eq!(rows[0].symbol, "APU_UploadBank");
        assert_eq!(rows[0].line, 16);
        assert!(rows.iter().all(|row| !row.signature.contains('#') && !row.signature.contains('=')));
    }
}
