use std::collections::BTreeSet;

use super::lex::{Kind, Toks, literal_path};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Region {
    Logic,
    Data,
    Types,
    Attr,
    Test,
}

pub enum Include {
    Literal { path: String, text: bool },
    Other(String),
}

pub struct Hit {
    pub tag: &'static str,
    pub detail: String,
}

/// A use of the vector reader: the domain a `vector_file::read("..")` call names, or `None` for
/// any other use of the crate, which could name another domain's file.
pub type VectorRead = Option<String>;

pub struct Source {
    pub hits: Vec<Hit>,
    pub includes: Vec<Include>,
    pub vector_reads: Vec<VectorRead>,
}

pub fn check(toks: Toks<'_>, test_file: bool, denied: &BTreeSet<String>) -> Source {
    let mut out = Source {
        hits: Vec::new(),
        includes: Vec::new(),
        vector_reads: vector_reads(toks),
    };
    renames(toks, &mut out.hits);
    out.includes = includes(toks);
    let regions = regions(toks);
    lints(toks, &regions, test_file, denied, &mut out.hits);
    out
}

fn hit(hits: &mut Vec<Hit>, tag: &'static str, detail: String) {
    hits.push(Hit { tag, detail });
}

fn renames(toks: Toks<'_>, hits: &mut Vec<Hit>) {
    for i in 0..toks.len() {
        let at = |k: usize| toks.text(i.saturating_add(k));
        let word = |k: usize| toks.is(i.saturating_add(k), Kind::Word);
        if at(0) == "std" && at(1) == "as" && word(2) {
            hit(hits, "std-alias", format!("std as {}", at(2)));
        }
        if at(0) == "std" && at(1) == "::" && at(2) == "{" {
            let end = toks.after_group(i.saturating_add(2));
            let renamed = (i.saturating_add(3)..end).any(|k| {
                matches!(toks.text(k.wrapping_sub(1)), "{" | ",")
                    && toks.text(k) == "self"
                    && toks.text(k.saturating_add(1)) == "as"
            });
            if renamed {
                hit(hits, "std-alias", "std::{self as ..}".to_owned());
            }
        }
        if at(0) == "std" && at(1) == "::" && at(2) == "$" && word(3) {
            hit(hits, "std-macro-ident", format!("std::${}", at(3)));
        }
        if matches!(at(0), "std" | "core" | "alloc")
            && at(1) == "::"
            && matches!(at(2), "include" | "include_str")
        {
            hit(hits, "macro-path", format!("{}::{}", at(0), at(2)));
        }
        if at(0) == "use" && toks.is(i, Kind::Word) {
            let end = (i..toks.len())
                .find(|&k| toks.punct(k, ";"))
                .unwrap_or(toks.len());
            if let Some(k) = (i..end).find(|&k| {
                toks.is(k, Kind::Word)
                    && matches!(toks.text(k), "include" | "include_str" | "include_bytes")
            }) {
                hit(hits, "macro-path", format!("use .. {}", toks.text(k)));
            }
        }
        if let Some(open) = toks.attribute_at(i) {
            let end = toks.after_group(open);
            if (open..end).any(|k| toks.text(k) == "path" && toks.punct(k.saturating_add(1), "=")) {
                hit(hits, "path-attr", "#[path = ..]".to_owned());
            }
        }
    }
}

fn includes(toks: Toks<'_>) -> Vec<Include> {
    let mut out = Vec::new();
    for i in 0..toks.len() {
        let name = toks.text(i);
        if !toks.is(i, Kind::Word) || !matches!(name, "include" | "include_str") {
            continue;
        }
        if !toks.punct(i.saturating_add(1), "!") || !toks.is(i.saturating_add(2), Kind::Open) {
            continue;
        }
        let path = toks
            .get(i.saturating_add(3))
            .and_then(literal_path)
            .filter(|_| toks.is(i.saturating_add(4), Kind::Close));
        out.push(path.map_or_else(
            || Include::Other(format!("{name}!(..)")),
            |path| Include::Literal {
                path: path.to_owned(),
                text: name == "include_str",
            },
        ));
    }
    out
}

/// Every `vector_file` word but a call of a local function by that name.
fn vector_reads(toks: Toks<'_>) -> Vec<VectorRead> {
    let mut out = Vec::new();
    for i in 0..toks.len() {
        if !toks.is(i, Kind::Word) || toks.text(i) != "vector_file" {
            continue;
        }
        let at = |k: usize| i.saturating_add(k);
        let local_fn = toks.text(at(1)) == "(" && !toks.punct(i.saturating_sub(1), "::");
        if local_fn {
            continue;
        }
        let call = toks.punct(at(1), "::")
            && toks.text(at(2)) == "read"
            && toks.text(at(3)) == "("
            && toks.is(at(5), Kind::Close);
        out.push(
            toks.get(at(4))
                .filter(|_| call)
                .and_then(literal_path)
                .map(str::to_owned),
        );
    }
    out
}

fn regions(toks: Toks<'_>) -> Vec<Region> {
    let n = toks.len();
    let mut region = vec![Region::Logic; n];
    let mark = |from: usize, to: usize, r: Region, region: &mut Vec<Region>| {
        for slot in region.iter_mut().take(to).skip(from) {
            *slot = r;
        }
    };
    let mut a = 0usize;
    let mut previous: Option<usize> = None;
    while a < n {
        if toks.cfg_test_at(a) {
            let e = toks.after_item(a.saturating_add(7));
            mark(a, e, Region::Test, &mut region);
            previous = e.checked_sub(1);
            a = e;
            continue;
        }
        if let Some(open) = toks.attribute_at(a) {
            let e = toks.after_group(open);
            mark(a, e, Region::Attr, &mut region);
            previous = e.checked_sub(1);
            a = e;
            continue;
        }
        let word = toks.is(a, Kind::Word) && ends_item(toks, previous);
        if word && matches!(toks.text(a), "struct" | "enum" | "union" | "type") {
            let e = toks.after_item(a);
            mark(a, e, Region::Types, &mut region);
            previous = e.checked_sub(1);
            a = e;
            continue;
        }
        if word
            && matches!(toks.text(a), "const" | "static")
            && !matches!(toks.text(a.saturating_add(1)), "fn" | "unsafe" | "extern")
            && let Some((eq, end)) = constant_value(toks, a)
        {
            mark(a, eq.saturating_add(1), Region::Types, &mut region);
            let value = if Data::behaviour(toks, eq.saturating_add(1), end) {
                Region::Logic
            } else {
                Region::Data
            };
            mark(eq.saturating_add(1), end, value, &mut region);
            if let Some(slot) = region.get_mut(end) {
                *slot = Region::Types;
            }
            previous = Some(end);
            a = end.saturating_add(1);
            continue;
        }
        if let Some(slot) = region.get_mut(a) {
            *slot = Region::Logic;
        }
        previous = Some(a);
        a = a.saturating_add(1);
    }
    for (from, to) in lookups(toks, &region) {
        mark(from, to, Region::Data, &mut region);
    }
    region
}

fn constant_value(toks: Toks<'_>, from: usize) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    let mut eq = None;
    let mut end = toks.len();
    for k in from..toks.len() {
        match toks.kind(k) {
            Some(Kind::Open) => depth = depth.saturating_add(1),
            Some(Kind::Close) => depth = depth.saturating_sub(1),
            _ if depth == 0 && eq.is_none() && toks.text(k) == "=" => eq = Some(k),
            _ if depth == 0 && toks.text(k) == ";" => {
                end = k;
                break;
            }
            _ => {}
        }
    }
    eq.map(|eq| (eq, end))
}

fn ends_item(toks: Toks<'_>, previous: Option<usize>) -> bool {
    let Some(p) = previous else {
        return true;
    };
    toks.punct(p, ";")
        || (toks.is(p, Kind::Open) && toks.text(p) == "{")
        || toks.is(p, Kind::Close)
        || (toks.is(p, Kind::Word) && matches!(toks.text(p), "pub" | "mut"))
}

struct Data<'a> {
    toks: Toks<'a>,
    at: usize,
    lim: usize,
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "dyn", "else", "enum", "extern", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "static", "struct", "trait", "type", "union", "unsafe", "use", "where", "while",
    "yield",
];

impl<'a> Data<'a> {
    const fn new(toks: Toks<'a>, at: usize, lim: usize) -> Self {
        Self { toks, at, lim }
    }

    fn behaviour(toks: Toks<'a>, from: usize, to: usize) -> bool {
        let mut data = Self::new(toks, from, to);
        !(data.expr() && data.at == data.lim)
    }

    fn tok(&self, kind: Kind, text: &str) -> bool {
        self.at < self.lim
            && self.toks.is(self.at, kind)
            && (text.is_empty() || self.toks.text(self.at) == text)
    }

    const fn step(&mut self) {
        self.at = self.at.saturating_add(1);
    }

    fn take(&mut self, kind: Kind, text: &str) -> bool {
        let found = self.tok(kind, text);
        if found {
            self.step();
        }
        found
    }

    fn literal(&mut self) -> bool {
        if self.take(Kind::Num, "") || self.take(Kind::Lit, "") {
            return true;
        }
        let from = self.at;
        if self.take(Kind::Punct, "-") && (self.take(Kind::Num, "") || self.take(Kind::Lit, "")) {
            return true;
        }
        self.at = from;
        false
    }

    fn range_dots(&mut self) -> bool {
        self.take(Kind::Punct, "..") || self.take(Kind::Punct, "..=")
    }

    fn expr(&mut self) -> bool {
        let from = self.at;
        if self.range_end() && self.range_dots() {
            return self.range_end();
        }
        self.at = from;
        if self.take(Kind::Num, "") || self.take(Kind::Lit, "") {
            return true;
        }
        if self.take(Kind::Punct, "-") {
            return self.take(Kind::Num, "") || self.take(Kind::Lit, "");
        }
        if self.take(Kind::Punct, "&") {
            return self.expr();
        }
        if self.take(Kind::Open, "(") {
            return self.list();
        }
        if self.take(Kind::Open, "[") {
            return self.array();
        }
        if self.tok(Kind::Word, "") || self.tok(Kind::Punct, "::") {
            if !self.path() {
                return false;
            }
            if self.take(Kind::Open, "(") {
                return self.list();
            }
            if self.take(Kind::Open, "{") {
                return self.fields();
            }
            return true;
        }
        false
    }

    fn path(&mut self) -> bool {
        self.take(Kind::Punct, "::");
        loop {
            if !self.tok(Kind::Word, "") || KEYWORDS.contains(&self.toks.text(self.at)) {
                return false;
            }
            self.step();
            if !self.take(Kind::Punct, "::") {
                return true;
            }
        }
    }

    fn range_end(&mut self) -> bool {
        if self.literal() {
            return true;
        }
        if !self.tok(Kind::Word, "") && !self.tok(Kind::Punct, "::") {
            return false;
        }
        self.path() && !self.tok(Kind::Open, "(") && !self.tok(Kind::Open, "{")
    }

    fn list(&mut self) -> bool {
        loop {
            if self.take(Kind::Close, "") {
                return true;
            }
            if !self.expr() {
                return false;
            }
            if self.take(Kind::Punct, ",") {
                continue;
            }
            return self.take(Kind::Close, "");
        }
    }

    fn array(&mut self) -> bool {
        let from = self.at;
        if self.expr() && self.take(Kind::Punct, ";") {
            return self.expr() && self.take(Kind::Close, "");
        }
        self.at = from;
        self.list()
    }

    fn fields(&mut self) -> bool {
        loop {
            if self.take(Kind::Close, "") {
                return true;
            }
            if self.take(Kind::Punct, "..") {
                return self.expr() && self.take(Kind::Close, "");
            }
            if !self.take(Kind::Word, "") && !self.take(Kind::Num, "") {
                return false;
            }
            if self.take(Kind::Punct, ":") && !self.expr() {
                return false;
            }
            if self.take(Kind::Punct, ",") {
                continue;
            }
            return self.take(Kind::Close, "");
        }
    }

    fn key(toks: Toks<'a>, from: usize, to: usize) -> bool {
        let mut data = Self::new(toks, from, to);
        if data.literal() {
            if data.at == to {
                return true;
            }
            return data.range_dots() && data.literal() && data.at == to;
        }
        data.at = from;
        if !data.path()
            || !toks
                .text(data.at.wrapping_sub(1))
                .starts_with(char::is_uppercase)
        {
            return false;
        }
        let open = data.at;
        if open == to {
            return true;
        }
        if toks.text(open) != "(" || !toks.is(open, Kind::Open) || toks.after_group(open) != to {
            return false;
        }
        split_top(toks, open.saturating_add(1), to.saturating_sub(1), ",")
            .into_iter()
            .all(|(s, e)| e <= s || pattern(toks, s, e))
    }
}

fn split_top(toks: Toks<'_>, from: usize, to: usize, separator: &str) -> Vec<(usize, usize)> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = from;
    for k in from..=to {
        if k < to {
            match toks.kind(k) {
                Some(Kind::Open) => {
                    depth = depth.saturating_add(1);
                    continue;
                }
                Some(Kind::Close) => {
                    depth = depth.saturating_sub(1);
                    continue;
                }
                _ if !(depth == 0 && toks.punct(k, separator)) => continue,
                _ => {}
            }
        }
        parts.push((start, k));
        start = k.saturating_add(1);
    }
    parts
}

fn pattern(toks: Toks<'_>, from: usize, to: usize) -> bool {
    if to == from.saturating_add(1) && toks.text(from) == "_" {
        return true;
    }
    split_top(toks, from, to, "|")
        .into_iter()
        .all(|(s, e)| Data::key(toks, s, e))
}

fn top_find(toks: Toks<'_>, from: usize, to: usize, text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for k in from..to {
        if depth == 0 && toks.text(k) == text && !toks.is(k, Kind::Lit) {
            return Some(k);
        }
        match toks.kind(k) {
            Some(Kind::Open) => depth = depth.saturating_add(1),
            Some(Kind::Close) => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn names_local(toks: Toks<'_>, from: usize, to: usize) -> bool {
    (from..to).any(|k| {
        let text = toks.text(k);
        toks.is(k, Kind::Word)
            && text.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
            && !matches!(text, "true" | "false")
            && !(k > from && toks.text(k.wrapping_sub(1)) == "::")
            && !(k.saturating_add(1) < to
                && matches!(toks.text(k.saturating_add(1)), "(" | "::" | ":"))
    })
}

fn rows(toks: Toks<'_>, from: usize, to: usize) -> Option<Vec<(usize, usize)>> {
    let mut values = Vec::new();
    for (s, e) in split_top(toks, from, to, ",") {
        if e <= s {
            continue;
        }
        let arrow = top_find(toks, s, e, "=>")?;
        let value = arrow.saturating_add(1);
        let data = e > value && !Data::behaviour(toks, value, e) && !names_local(toks, value, e);
        if !pattern(toks, s, arrow) || !data {
            return None;
        }
        values.push((value, e));
    }
    Some(values)
}

fn lookups(toks: Toks<'_>, region: &[Region]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for a in 0..toks.len() {
        if region.get(a) != Some(&Region::Logic) || !toks.is(a, Kind::Word) {
            continue;
        }
        let values = if toks.text(a) == "match" {
            let Some(open) = top_find(toks, a.saturating_add(1), toks.len(), "{")
                .filter(|o| toks.is(*o, Kind::Open))
            else {
                continue;
            };
            let end = toks.after_group(open);
            rows(toks, open.saturating_add(1), end.saturating_sub(1)).filter(|v| v.len() >= 2)
        } else if toks.text(a) == "enum_map"
            && toks.text(a.saturating_add(1)) == "!"
            && toks.is(a.saturating_add(2), Kind::Open)
        {
            let end = toks.after_group(a.saturating_add(2));
            rows(toks, a.saturating_add(3), end.saturating_sub(1))
        } else {
            None
        };
        out.extend(values.into_iter().flatten());
    }
    out
}

fn lint_attr(
    toks: Toks<'_>,
    from: usize,
    to: usize,
    in_test: bool,
    denied: &BTreeSet<String>,
    hits: &mut Vec<Hit>,
) {
    let mut k = from;
    while k < to {
        let level = toks.text(k);
        if !(toks.is(k, Kind::Word)
            && matches!(level, "allow" | "expect" | "warn")
            && toks.text(k.saturating_add(1)) == "(")
        {
            k = k.saturating_add(1);
            continue;
        }
        let end = toks.after_group(k.saturating_add(1));
        let mut m = k.saturating_add(2);
        while m < end.saturating_sub(1) {
            if toks.text(m) == "reason" && toks.text(m.saturating_add(1)) == "=" {
                m = m.saturating_add(3);
                continue;
            }
            if toks.is(m, Kind::Word) {
                let mut lint = toks.text(m).to_owned();
                if toks.text(m.saturating_add(1)) == "::"
                    && toks.is(m.saturating_add(2), Kind::Word)
                {
                    lint = format!("{lint}::{}", toks.text(m.saturating_add(2)));
                    m = m.saturating_add(2);
                }
                if !in_test && denied.contains(&lint) {
                    hit(
                        hits,
                        "lint-attr",
                        format!(
                            "line {}: {level}({lint}) suppresses a lint the workspace denies",
                            toks.line(k)
                        ),
                    );
                }
            }
            m = m.saturating_add(1);
        }
        k = end;
    }
}

fn variant_imports(toks: Toks<'_>, at: usize, hits: &mut Vec<Hit>) {
    let mut end = at;
    while end < toks.len() && !toks.punct(end, ";") {
        end = if toks.is(end, Kind::Open) {
            toks.after_group(end)
        } else {
            end.saturating_add(1)
        };
    }
    for k in at.saturating_add(1)..end {
        let enum_name = toks.text(k);
        if !toks.is(k, Kind::Word)
            || !enum_name.starts_with(char::is_uppercase)
            || enum_name == "Self"
            || toks.text(k.saturating_add(1)) != "::"
        {
            continue;
        }
        let t = k.saturating_add(2);
        let names = if toks.text(t) == "*" {
            "*".to_owned()
        } else if toks.is(t, Kind::Word) {
            toks.text(t).to_owned()
        } else if toks.is(t, Kind::Open) && toks.text(t) == "{" {
            let group = toks.after_group(t);
            let mut depth = 0usize;
            let mut names = Vec::new();
            for m in t.saturating_add(1)..group.saturating_sub(1) {
                match toks.kind(m) {
                    Some(Kind::Open) => depth = depth.saturating_add(1),
                    Some(Kind::Close) => depth = depth.saturating_sub(1),
                    Some(Kind::Word)
                        if depth == 0
                            && !matches!(toks.text(m), "self" | "as")
                            && toks.text(m.wrapping_sub(1)) != "as" =>
                    {
                        names.push(toks.text(m));
                    }
                    _ => {}
                }
            }
            if names.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", names.join(", "))
            }
        } else {
            String::new()
        };
        if !names.is_empty() {
            hit(
                hits,
                "variant-import",
                format!(
                    "line {}: {enum_name}::{names} (name a variant through its enum)",
                    toks.line(k)
                ),
            );
        }
    }
}

const SUFFIXES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];

fn zero_or_one(text: &str) -> bool {
    let plain: String = text.chars().filter(|c| *c != '_').collect();
    let strip = |s: &str, floats: bool| -> String {
        let suffix = SUFFIXES
            .iter()
            .chain(if floats {
                ["f32", "f64"].iter()
            } else {
                [].iter()
            })
            .find(|suffix| s.ends_with(*suffix));
        suffix
            .and_then(|suffix| s.strip_suffix(suffix))
            .unwrap_or(s)
            .to_owned()
    };
    let small = |digits: &str| matches!(digits.trim_start_matches('0'), "" | "1");
    if let Some(digits) = ["0x", "0o", "0b"]
        .iter()
        .find_map(|p| plain.strip_prefix(p))
    {
        let digits = strip(digits, false);
        return !digits.is_empty() && small(&digits);
    }
    let value = strip(&plain, true);
    let (whole, fraction) = value.split_once('.').unwrap_or((&value, ""));
    value.contains(|c: char| c.is_ascii_digit())
        && small(whole)
        && whole.chars().all(|c| c.is_ascii_digit())
        && fraction.chars().all(|c| c == '0')
}

fn lints(
    toks: Toks<'_>,
    region: &[Region],
    test_file: bool,
    denied: &BTreeSet<String>,
    hits: &mut Vec<Hit>,
) {
    let mut a = 0usize;
    while a < toks.len() {
        let here = region.get(a).copied().unwrap_or(Region::Logic);
        if let Some(open) = toks.attribute_at(a) {
            let end = toks.after_group(open);
            lint_attr(
                toks,
                a,
                end,
                test_file || here == Region::Test,
                denied,
                hits,
            );
            a = end;
            continue;
        }
        let in_test = test_file || here == Region::Test;
        if !in_test
            && toks.is(a, Kind::Word)
            && toks.text(a) == "use"
            && toks.text(a.saturating_add(1)) != "<"
        {
            variant_imports(toks, a, hits);
        }
        let magic = !in_test
            && here == Region::Logic
            && toks.is(a, Kind::Num)
            && !zero_or_one(toks.text(a))
            && !(toks.text(a.wrapping_sub(1)) == ";" && toks.text(a.saturating_add(1)) == "]");
        if magic {
            hit(
                hits,
                "magic-literal",
                format!("line {}: {}", toks.line(a), toks.text(a)),
            );
        }
        a = a.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::zero_or_one;

    #[test]
    fn zero_and_one_in_every_spelling() {
        for text in [
            "0", "1", "0x0", "0b1", "1u8", "0_u16", "1.0", "0.0f32", "00",
        ] {
            assert!(zero_or_one(text), "{text}");
        }
        for text in ["2", "15_i16", "0x7FFF", "0b10", "0.5", "1e5", "10"] {
            assert!(!zero_or_one(text), "{text}");
        }
    }
}
