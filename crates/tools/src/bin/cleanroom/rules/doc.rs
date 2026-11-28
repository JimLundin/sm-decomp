use std::collections::BTreeSet;

use super::lex::{Comment, Kind, Toks};
use super::source::Hit;

const TAG: &str = "doc-code";

/// A code block in a doc comment or a `doc` attribute of one file, which `cargo test --doc`
/// compiles and runs whatever the manifest says. A `doc` attribute holds a string literal or a
/// macro's metavariable, whose literals `macros` reads.
pub fn check(toks: Toks<'_>, comments: &[Comment], hits: &mut Vec<Hit>) {
    let docs: Vec<(usize, &str, String)> = comments
        .iter()
        .filter_map(|c| doc_body(&c.text).map(|(kind, body)| (c.line, kind, body)))
        .collect();
    let base = docs
        .iter()
        .flat_map(|(_, _, body)| body.lines())
        .filter(|l| !l.trim().is_empty())
        .map(indent)
        .fold(1, usize::min);
    let mut at = 0usize;
    while let Some((line, kind, body)) = docs.get(at) {
        let mut text = body.clone();
        let mut next = at.saturating_add(1);
        while let Some((following, k, more)) = docs.get(next)
            && *k == *kind
            && kind.starts_with("//")
            && *following == line.saturating_add(next.saturating_sub(at))
        {
            text.push('\n');
            text.push_str(more);
            next = next.saturating_add(1);
        }
        blocks(&text, *line, base, hits);
        at = next;
    }
    for i in 0..toks.len() {
        let Some(open) = toks.attribute_at(i) else {
            continue;
        };
        for k in open..toks.after_group(open) {
            if !(toks.is(k, Kind::Word)
                && toks.text(k) == "doc"
                && toks.punct(k.saturating_add(1), "="))
            {
                continue;
            }
            let value = k.saturating_add(2);
            if toks.punct(value, "$") && toks.is(value.saturating_add(1), Kind::Word) {
                continue;
            }
            match toks
                .get(value)
                .filter(|t| t.kind == Kind::Lit)
                .and_then(|t| string(&t.text))
            {
                Some(text) => blocks(&text, toks.line(value), 0, hits),
                None => hits.push(Hit {
                    tag: TAG,
                    detail: format!(
                        "line {}: a doc attribute's value is neither a string literal nor a macro's metavariable",
                        toks.line(k)
                    ),
                }),
            }
        }
    }
}

/// The calls, across one crate's FILES, of every macro that writes a `doc` attribute from a
/// metavariable, directly or through another such macro. Each string literal a call passes is
/// read as markdown, and a macro call among its arguments is refused, since its text can't be.
pub fn macros(files: &[(String, Vec<super::lex::Token>)]) -> Vec<(String, Hit)> {
    let mut documenting: BTreeSet<&str> = BTreeSet::new();
    loop {
        let before = documenting.len();
        for (_, toks) in files {
            let toks = Toks(toks);
            for i in 0..toks.len() {
                if toks.text(i) != "macro_rules" || !toks.punct(i.saturating_add(1), "!") {
                    continue;
                }
                let name = i.saturating_add(2);
                let body = name.saturating_add(1);
                let writes = (body..toks.after_group(body)).any(|k| {
                    (toks.text(k) == "doc"
                        && toks.punct(k.saturating_add(1), "=")
                        && toks.punct(k.saturating_add(2), "$"))
                        || (documenting.contains(toks.text(k))
                            && toks.punct(k.saturating_add(1), "!"))
                });
                if writes && toks.is(name, Kind::Word) {
                    documenting.insert(toks.text(name));
                }
            }
        }
        if documenting.len() == before {
            break;
        }
    }
    let mut out = Vec::new();
    for (path, toks) in files {
        let toks = Toks(toks);
        let mut hits = Vec::new();
        for i in 0..toks.len() {
            let args = i.saturating_add(2);
            if !(documenting.contains(toks.text(i))
                && toks.punct(i.saturating_add(1), "!")
                && toks.is(args, Kind::Open))
            {
                continue;
            }
            for k in args..toks.after_group(args) {
                if toks.is(k, Kind::Word)
                    && toks.punct(k.saturating_add(1), "!")
                    && toks.is(k.saturating_add(2), Kind::Open)
                {
                    hits.push(Hit {
                        tag: TAG,
                        detail: format!(
                            "line {}: {}! inside the call of {}!, which writes a doc attribute",
                            toks.line(k),
                            toks.text(k),
                            toks.text(i)
                        ),
                    });
                }
                if let Some(text) = toks
                    .get(k)
                    .filter(|t| t.kind == Kind::Lit)
                    .and_then(|t| string(&t.text))
                {
                    blocks(&text, toks.line(k), 0, &mut hits);
                }
            }
        }
        out.extend(hits.into_iter().map(|hit| (path.clone(), hit)));
    }
    out
}

/// A doc comment's kind and markdown, or `None` for a plain comment.
fn doc_body(text: &str) -> Option<(&'static str, String)> {
    if let Some(rest) = text.strip_prefix("///").filter(|r| !r.starts_with('/')) {
        return Some(("///", rest.to_owned()));
    }
    if let Some(rest) = text.strip_prefix("//!") {
        return Some(("//!", rest.to_owned()));
    }
    let inner = text
        .strip_prefix("/**")
        .filter(|r| !r.starts_with(['*', '/']))
        .or_else(|| text.strip_prefix("/*!"))?;
    let inner = inner.strip_suffix("*/").unwrap_or(inner);
    let decorated = inner
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .all(|l| l.trim_start().starts_with('*'));
    let body = inner
        .lines()
        .enumerate()
        .map(|(n, l)| {
            if decorated && n > 0 {
                l.trim_start().strip_prefix('*').unwrap_or(l)
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(("/*", body))
}

/// A literal's string value, or `None` when it is no string.
fn string(literal: &str) -> Option<String> {
    if let Some(raw) = literal.strip_prefix('r') {
        let hashes = raw.len().saturating_sub(raw.trim_start_matches('#').len());
        let fence = "#".repeat(hashes);
        return raw
            .strip_prefix(&fence)?
            .strip_suffix(&fence)?
            .strip_prefix('"')?
            .strip_suffix('"')
            .map(str::to_owned);
    }
    let body = literal.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '0' => out.push('\0'),
            'x' => {
                let hex: String = chars.by_ref().take(2).collect();
                out.push(char::from(u8::from_str_radix(&hex, 16).ok()?));
            }
            'u' => {
                let code: String = chars.by_ref().take_while(|&c| c != '}').skip(1).collect();
                out.push(char::from_u32(u32::from_str_radix(&code, 16).ok()?)?);
            }
            '\n' => while chars.next_if(|c| c.is_whitespace()).is_some() {},
            other => out.push(other),
        }
    }
    Some(out)
}

fn indent(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

/// Whether a line leaves a paragraph open, so that an indented line after it continues the
/// paragraph rather than starting code.
fn continues(text: &str) -> bool {
    !(text.is_empty()
        || text.starts_with(['#', '|'])
        || text
            .chars()
            .all(|c| matches!(c, '-' | '*' | '_' | '=' | ' '))
        || (text.starts_with('[') && text.contains("]:")))
}

/// Every line of MARKDOWN that opens a fenced block or starts an indented one, read four columns
/// past BASE. A line any wider than markdown's list items allow counts too.
fn blocks(markdown: &str, first_line: usize, base: usize, hits: &mut Vec<Hit>) {
    let mut paragraph = false;
    for (n, line) in markdown.lines().enumerate() {
        let text = line.trim_start();
        let what = if text.starts_with("```") || text.starts_with("~~~") {
            "a fenced code block"
        } else if !text.is_empty() && !paragraph && indent(line).saturating_sub(base) >= 4 {
            "an indented code block"
        } else {
            paragraph = continues(text);
            continue;
        };
        paragraph = false;
        hits.push(Hit {
            tag: TAG,
            detail: format!(
                "line {}: {what} in a doc comment, which rustdoc runs as a doctest",
                first_line.saturating_add(n)
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{doc_body, string};

    #[test]
    fn only_doc_comments_hold_markdown() {
        let body = |text| doc_body(text).map(|(_, body)| body);
        assert_eq!(body("/// a").as_deref(), Some(" a"));
        assert_eq!(body("//! a").as_deref(), Some(" a"));
        assert_eq!(body("/*! a */").as_deref(), Some(" a "));
        assert_eq!(body("/** a\n * b\n */").as_deref(), Some(" a\n b\n "));
        for plain in ["// a", "//// a", "/* a */", "/*** a */", "/**/"] {
            assert_eq!(body(plain), None, "{plain}");
        }
    }

    #[test]
    fn a_doc_string_is_read_with_its_escapes() {
        assert_eq!(
            string(
                r#""\x60\x60\x60\n\u{60}\t\
                x""#
            )
            .as_deref(),
            Some("```\n`\tx")
        );
        assert_eq!(string("r#\"a\"b\"#").as_deref(), Some("a\"b"));
        assert_eq!(string("'a'"), None);
    }
}
