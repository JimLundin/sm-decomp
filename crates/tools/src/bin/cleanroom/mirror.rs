//! Check 4: whether a spec, through its decode notes, mirrors the original's routines.
//!
//! Each spec entry (`### Id`) collects the addresses its section and the decode notes give it.
//! An address belongs to the routine whose entry is the nearest known one at or below it, found
//! by walking down the hashed routine set. An entry is single-routine when every address it
//! cites is in one routine; a non-private model function (`pub`, `pub(crate)`, `pub(super)`)
//! maps onto a routine when the first entry it names is single-routine.

use std::collections::{BTreeMap, BTreeSet};

use crate::lexicon::Lexicon;
use crate::scan::{Class, Finding};

/// The gate: at most this share (in percent) of entries may come from a single routine.
pub const SINGLE_ROUTINE_LIMIT: usize = 30;

/// How far below an address the containing routine's entry may be.
const ROUTINE_REACH: u32 = 0x2000;

type Address = (String, u32);

/// `$BB:OOOO`, and `$OOOO` in the bank last named on the same text.
fn cited(text: &str) -> Vec<Address> {
    let mut out = Vec::new();
    let mut bank: Option<String> = None;
    for piece in text.split('$').skip(1) {
        let head: String = piece
            .chars()
            .take_while(|c| c.is_ascii_hexdigit() || *c == ':')
            .collect();
        let (b, offset) = match head.split_once(':') {
            Some((b, o)) if b.len() == 2 && o.len() >= 4 => (Some(b.to_uppercase()), o.get(..4)),
            None if head.len() == 4 => (None, Some(head.as_str())),
            _ => continue,
        };
        if b.is_some() {
            bank = b;
        }
        if let (Some(bank), Some(offset)) =
            (&bank, offset.and_then(|o| u32::from_str_radix(o, 16).ok()))
        {
            out.push((bank.clone(), offset));
        }
    }
    out
}

fn routine(lex: &Lexicon, (bank, offset): &Address) -> Address {
    let low = offset.saturating_sub(ROUTINE_REACH);
    (low..=*offset)
        .rev()
        .find(|o| lex.has(&lex.addresses, &format!("r:{bank}{o:04X}")))
        .map_or_else(|| (bank.clone(), *offset), |o| (bank.clone(), o))
}

/// Spec entries in order, with their line and the text that cites their sources.
fn entries(spec: &str) -> Vec<(String, usize, String)> {
    let mut out: Vec<(String, usize, String)> = Vec::new();
    let mut open = false;
    for (n, line) in spec.lines().enumerate() {
        if line.starts_with('#') {
            let id = line.trim_start_matches('#').trim();
            open =
                line.starts_with("### ") && !id.is_empty() && id.chars().all(char::is_alphanumeric);
            if open {
                out.push((id.to_owned(), n.saturating_add(1), String::new()));
            }
        } else if open && let Some((_, _, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

/// The identifiers of a text, in order.
fn idents(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
}

/// Visibility immediately before a function, allowing qualifiers and restricted paths.
fn is_visible(before: &[crate::text::SpannedToken]) -> bool {
    use crate::text::Tok;
    let mut end = before.len();
    while let Some(token) = before.get(end.saturating_sub(1)).filter(|_| end > 0) {
        match &token.token {
            Tok::Ident(word)
                if matches!(word.as_str(), "const" | "async" | "unsafe" | "extern") =>
            {
                end = end.saturating_sub(1);
            }
            Tok::Str => end = end.saturating_sub(1),
            _ => break,
        }
    }
    if before
        .get(end.saturating_sub(1))
        .is_some_and(|t| t.token == Tok::Punct(')'))
    {
        let mut depth = 1usize;
        end = end.saturating_sub(1);
        while end > 0 {
            end = end.saturating_sub(1);
            match before.get(end).map(|t| &t.token) {
                Some(Tok::Punct(')')) => depth = depth.saturating_add(1),
                Some(Tok::Punct('(')) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    end > 0
        && before
            .get(end.saturating_sub(1))
            .is_some_and(|t| t.token == Tok::Ident("pub".to_owned()))
}

/// Match code braces using lexical spans, then slice the original text to retain entry tags.
fn public_fns(src: &str) -> Vec<(String, usize, String)> {
    use crate::text::{Tok, rust_spans};
    let tokens = rust_spans(src);
    let mut out = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.token != Tok::Ident("fn".to_owned())
            || !is_visible(tokens.get(..index).unwrap_or_default())
        {
            continue;
        }
        let Some(crate::text::SpannedToken {
            token: Tok::Ident(name),
            ..
        }) = tokens.get(index.saturating_add(1))
        else {
            continue;
        };
        let mut parens = 0usize;
        let mut brackets = 0usize;
        let mut angles = 0usize;
        let mut body = None;
        let mut depth = 0usize;
        let mut previous = None;
        for current in tokens.iter().skip(index.saturating_add(2)) {
            if let Some(open) = body {
                match current.token {
                    Tok::Punct('{') => depth = depth.saturating_add(1),
                    Tok::Punct('}') => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            out.push((
                                name.clone(),
                                token.line,
                                src.get(open..current.end).unwrap_or_default().to_owned(),
                            ));
                            break;
                        }
                    }
                    _ => {}
                }
            } else {
                match current.token {
                    Tok::Punct('(') => parens = parens.saturating_add(1),
                    Tok::Punct(')') => parens = parens.saturating_sub(1),
                    Tok::Punct('[') => brackets = brackets.saturating_add(1),
                    Tok::Punct(']') => brackets = brackets.saturating_sub(1),
                    // Comparisons in const expressions and arrows in bounds are not generic delimiters.
                    Tok::Punct('<') if parens == 0 && brackets == 0 && depth == 0 => {
                        angles = angles.saturating_add(1);
                    }
                    Tok::Punct('>')
                        if parens == 0
                            && brackets == 0
                            && depth == 0
                            && previous != Some(&Tok::Punct('-')) =>
                    {
                        angles = angles.saturating_sub(1);
                    }
                    Tok::Punct(';') if parens == 0 && brackets == 0 && angles == 0 => break,
                    Tok::Punct('{')
                        if parens == 0 && brackets == 0 && angles == 0 && depth == 0 =>
                    {
                        body = Some(current.start);
                        depth = 1;
                    }
                    Tok::Punct('{') => depth = depth.saturating_add(1),
                    Tok::Punct('}') => depth = depth.saturating_sub(1),
                    _ => {}
                }
                previous = Some(&current.token);
            }
        }
    }
    out
}

/// Check 4. Returns the findings and a summary line; `failed` is set when a gate is crossed.
pub fn check(
    lex: &Lexicon,
    spec_path: &str,
    spec: &str,
    decode: &str,
    model: &[(String, String)],
    failed: &mut bool,
) -> (Vec<Finding>, String) {
    let mut findings = Vec::new();
    let mut routines: BTreeMap<String, BTreeSet<Address>> = BTreeMap::new();
    let all = entries(spec);
    let ids: BTreeSet<&str> = all.iter().map(|(id, _, _)| id.as_str()).collect();
    for (id, _, body) in &all {
        let set = routines.entry(id.clone()).or_default();
        set.extend(cited(body).iter().map(|a| routine(lex, a)));
        for (_, _, decode_body) in entries(decode).iter().filter(|(d, _, _)| d == id) {
            set.extend(cited(decode_body).iter().map(|a| routine(lex, a)));
        }
        for line in decode.lines().filter(|l| idents(l).any(|w| w == id)) {
            set.extend(cited(line).iter().map(|a| routine(lex, a)));
        }
    }
    let mut single = 0usize;
    let mut unsourced = 0usize;
    for (id, line, _) in &all {
        match routines.get(id).map_or(0, BTreeSet::len) {
            0 => unsourced = unsourced.saturating_add(1),
            1 => {
                single = single.saturating_add(1);
                findings.push(Finding {
                    path: spec_path.to_owned(),
                    line: *line,
                    class: Class::SingleRoutine,
                    detail: format!("{id}: its source is one routine"),
                });
            }
            _ => {}
        }
    }
    let total = all.len();
    let share = single.saturating_mul(100).checked_div(total).unwrap_or(0);
    if share > SINGLE_ROUTINE_LIMIT {
        *failed = true;
    }
    let mut entry_points = 0usize;
    let mut mapped = 0usize;
    for (path, src) in model {
        for (name, line, body) in public_fns(src) {
            let Some(first) = idents(&body).find(|w| ids.contains(w)) else {
                continue;
            };
            entry_points = entry_points.saturating_add(1);
            if routines.get(first).is_some_and(|r| r.len() == 1) {
                mapped = mapped.saturating_add(1);
                *failed = true;
                findings.push(Finding {
                    path: path.clone(),
                    line,
                    class: Class::EntryPoint,
                    detail: format!("{name}: opens with {first}, whose source is one routine"),
                });
            }
        }
    }
    let summary = format!(
        "mirror: {single} of {total} entries single-routine ({share} %, limit {SINGLE_ROUTINE_LIMIT} %), \
         {unsourced} unsourced; {mapped} of {entry_points} tagged non-private functions map onto one routine\n"
    );
    (findings, summary)
}

#[cfg(test)]
mod tests {
    use super::{check, public_fns};
    use crate::lexicon::Lexicon;
    use crate::scan::Class;

    fn names(src: &str) -> Vec<String> {
        public_fns(src)
            .into_iter()
            .map(|(name, _, _)| name)
            .collect()
    }

    #[test]
    fn pub_fns_are_tagged() {
        assert_eq!(names("pub fn step() { Rolling }"), ["step"]);
        assert_eq!(names("pub const fn step() -> u8 { 1 }"), ["step"]);
    }

    #[test]
    fn crate_visible_fns_are_tagged() {
        assert_eq!(names("pub(crate) fn step() { Rolling }"), ["step"]);
        assert_eq!(names("pub(crate) const fn step() -> u8 { 1 }"), ["step"]);
    }

    #[test]
    fn parent_visible_fns_are_tagged() {
        assert_eq!(names("pub(super) fn step() { Rolling }"), ["step"]);
        assert_eq!(
            names("    pub(super) unsafe fn step() { Rolling }"),
            ["step"]
        );
    }

    #[test]
    fn private_fns_are_not_tagged() {
        assert_eq!(names("fn step() { Rolling }"), Vec::<String>::new());
        assert_eq!(names("const fn step() -> u8 { 1 }"), Vec::<String>::new());
        assert_eq!(names("pub struct S; fn step() {}"), Vec::<String>::new());
        assert_eq!(names("pub fn_like(); let my_fn = 1;"), Vec::<String>::new());
    }

    #[test]
    fn body_and_line_are_kept() {
        let found =
            public_fns("struct S;\nimpl S {\n    pub(crate) fn step(&self) { Rolling }\n}\n");
        assert_eq!(found.len(), 1);
        let (name, line, body) = found.first().cloned().unwrap_or_default();
        assert_eq!((name.as_str(), line), ("step", 3));
        assert!(body.contains("Rolling"));
    }

    /// A spec whose one entry cites one address, so its source is one routine.
    const SPEC: &str = "### Rolling\n\nShe rolls ($91:8000).\n";

    fn gate(model: &str) -> (usize, bool) {
        let mut failed = false;
        let source = [("model.rs".to_owned(), model.to_owned())];
        let (findings, _) = check(
            &Lexicon::default(),
            "spec.md",
            SPEC,
            "",
            &source,
            &mut failed,
        );
        let mapped = findings
            .iter()
            .filter(|f| f.class == Class::EntryPoint)
            .count();
        (mapped, failed)
    }

    #[test]
    fn every_visible_step_is_gated() {
        for visibility in ["pub", "pub(crate)", "pub(super)"] {
            let model = format!("impl S {{\n    {visibility} fn step(&self) {{ Rolling }}\n}}\n");
            assert_eq!(gate(&model), (1, true), "{visibility}");
        }
        assert_eq!(gate("fn step() { Rolling }"), (0, true));
    }

    #[test]
    fn literal_and_comment_braces_leave_entry_comments_in_the_body() {
        for literal in [
            r#""}""#,
            r#""escaped \" }""#,
            "\"multiline\n}\"",
            r###"r##"} "# pub fn fake() {"##"###,
            r#"b"}""#,
            r#"c"}""#,
            r##"br#"}"#"##,
            r##"cr#"}"#"##,
            "'}'",
            r"'\u{7d}'",
            r"'\x7d'",
            r"'\\'",
        ] {
            let model = format!(
                "pub fn step() {{ let value = {literal}; /* }} /* nested */ }} */ // }}\n/* Rolling */ }}"
            );
            assert_eq!(names(&model), ["step"], "{literal}");
            assert_eq!(gate(&model), (1, true), "{literal}");
        }
    }

    #[test]
    fn fake_functions_and_declarations_do_not_acquire_bodies() {
        let model = r###"
// pub fn line_fake() { Rolling }
/* /* nested */ pub fn block_fake() { Rolling } */
const TEXT: &str = r##"pub fn string_fake() { Rolling }"##;
pub fn declaration();
fn private() { Rolling }
pub (in crate::scope)
    async unsafe fn real<'a>(value: &'a [u8; { 2 }]) { /* Rolling */ }
"###;
        assert_eq!(names(model), ["real"]);
        assert_eq!(gate(model), (1, true));
    }

    #[test]
    fn signature_const_comparisons_do_not_hide_function_bodies() {
        for signature in [
            "pub fn step(value: [u8; if 1 < 2 { 1 } else { 2 }])",
            "pub fn step(value: [u8; if 2 > 1 { 1 } else { 2 }])",
            "pub fn step() -> Wrap<{ if 1 < 2 { 1 } else { 2 } }>",
            "pub fn step<F: Fn() -> Wrap<{ if 2 > 1 { 1 } else { 2 } }>>()",
        ] {
            let model = format!("{signature} {{ /* Rolling */ }}");
            assert_eq!(names(&model), ["step"], "{signature}");
            assert_eq!(gate(&model), (1, true), "{signature}");
            assert_eq!(
                public_fns(&model).first().map(|(_, _, body)| body.as_str()),
                Some("{ /* Rolling */ }")
            );
        }
    }
}
