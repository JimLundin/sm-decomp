//! Source facts without exposing lexical cleanup or token-scanning helpers.
use crate::inventory::{clean_bounded, names_bounded};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use tool_execution::bounded;

/// One bounded C source snapshot, with comments, literals and directives masked
/// privately so definitions and their dependencies use the same lexical view.
pub struct DefinitionSource {
    text: String,
    code: Vec<u8>,
}

/// Facts from one complete definition in a source snapshot. These facts alone
/// do not authenticate an inventory or grant Translation admission.
pub struct Definition<'a> {
    pub symbol: &'a str,
    pub text: &'a str,
    pub opening_line: &'a str,
    pub sha256: String,
    pub calls: Vec<String>,
    pub indirect_indexed_calls: Vec<String>,
}

impl DefinitionSource {
    pub fn new(text: String) -> Result<Self> {
        let code = clean_bounded(&text)?;
        Ok(Self { text, code })
    }

    /// Read the complete definition beginning at a one-based inventory line.
    pub fn at_line(&self, line: usize) -> Result<Definition<'_>> {
        bounded::check()?;
        ensure!(line > 0, "routine source line missing");
        let offset = self.text.split_inclusive('\n').take(line - 1).map(str::len).sum::<usize>();
        let start = self.text.get(offset..).context("routine source line outside pinned file")?;
        let clean = self.code.get(offset..).context("routine code outside pinned file")?;
        let open = clean.iter().position(|c| *c == b'{').context("routine definition body missing")?;
        let symbol = definition_symbol(&clean[..open]).context("routine definition symbol missing")?;
        let mut depth = 1;
        let mut end = open + 1;
        while end < start.len() && depth > 0 {
            if end.is_multiple_of(64 * 1024) {
                bounded::check()?;
            }
            match clean[end] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        ensure!(depth == 0, "routine definition identity differs");
        let text = &start[..end];
        let (calls, indirect_indexed_calls) = names_bounded(&clean[open..end])?;
        Ok(Definition {
            symbol,
            text,
            opening_line: start[open..].lines().next().unwrap_or_default(),
            sha256: format!("{:x}", Sha256::digest(text.as_bytes())),
            calls,
            indirect_indexed_calls,
        })
    }
}

fn definition_symbol(signature: &[u8]) -> Option<&str> {
    if signature.contains(&b';') {
        return None;
    }
    let paren = signature.iter().position(|byte| *byte == b'(')?;
    let end = signature[..paren].iter().rposition(|byte| !byte.is_ascii_whitespace())? + 1;
    let start = signature[..end]
        .iter()
        .rposition(|byte| !(byte.is_ascii_alphanumeric() || *byte == b'_'))
        .map_or(0, |index| index + 1);
    std::str::from_utf8(&signature[start..end]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_definition_binds_exact_clean_function_token() -> Result<()> {
        for (signature, expected) in [
            ("void Observed (void) ", Some("Observed")),
            ("void NotObserved(void) ", Some("NotObserved")),
            ("/* Observed( */ void NotObserved(void) ", Some("NotObserved")),
            ("void Observed(void); void NotObserved(void) ", None),
        ] {
            let source = DefinitionSource::new(format!("{signature} {{ Actual(); table[index](0); }}"))?;
            let definition = source.at_line(1);
            if let Some(expected) = expected {
                let definition = definition?;
                assert_eq!(definition.symbol, expected);
                assert_eq!(definition.calls, ["Actual"]);
                assert_eq!(definition.indirect_indexed_calls, ["table"]);
            } else {
                assert!(definition.is_err());
            }
        }
        Ok(())
    }
}

/// Extract one unambiguous bounded Recovered C definition for a source context.
pub fn recovered_definition(bank: &str, symbol: &str, max_bytes: u64) -> Result<String> {
    bounded::check()?;
    ensure!(crate::identifier(symbol), "recovered C symbol is invalid");
    let masked = clean_bounded(bank)?;
    let token = format!("{symbol}(");
    let mut starts = Vec::new();
    for (offset, _) in String::from_utf8_lossy(&masked).match_indices(&token) {
        bounded::check()?;
        let line_start = masked[..offset].iter().rposition(|byte| *byte == b'\n').map_or(0, |at| at + 1);
        let Some(brace) = masked[offset..].iter().position(|byte| b";{}".contains(byte)) else {
            continue;
        };
        let brace = offset + brace;
        if masked[brace] != b'{' {
            continue;
        }
        let prefix = String::from_utf8_lossy(&masked[line_start..offset]);
        if prefix.trim().is_empty() || prefix.contains('=') {
            continue;
        }
        let mut end = brace + 1;
        let mut depth = 1u32;
        while end < masked.len() && depth > 0 {
            if end.is_multiple_of(65536) {
                bounded::check()?;
            }
            if masked[end] == b'{' {
                depth += 1;
            }
            if masked[end] == b'}' {
                depth -= 1;
            }
            end += 1;
        }
        ensure!(depth == 0, "recovered C routine body is unbalanced");
        ensure!(
            end.saturating_sub(line_start) as u64 <= max_bytes,
            "recovered C definition exceeds remaining context budget"
        );
        ensure!(starts.is_empty(), "recovered C definition is ambiguous: {symbol}");
        starts.push(bank[line_start..end].to_owned());
    }
    ensure!(starts.len() == 1, "recovered C definition is missing or ambiguous: {symbol}");
    Ok(starts.remove(0))
}
