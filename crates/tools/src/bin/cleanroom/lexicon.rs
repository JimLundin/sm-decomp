//! The hashed lexicon: what the original calls things and where it keeps them, stored so that
//! the files name nothing.
//!
//! `DIR/salt` holds the salt. Each other file is plain text, one entry per line:
//! - `addresses`: hashes of `w:OFFS` (work RAM), `o:OFFS` (a routine's offset in its bank) and
//!   `r:BBOFFS` (a routine's full address).
//! - `names`: hashes of `n:<2-4 word n-gram>` and `u:<word>` (distinctive single words).
//! - `codes`: `TABLE CODE HASH`, a code table's value and the hash of its normalised name.
//! - `fields`: hashes of `f:Struct.field`, a field the decoder fills from one word as is.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::text::{self, Tok};

const FILES: [&str; 4] = ["addresses", "names", "codes", "fields"];

/// The lexicon as the checks read it.
#[derive(Default)]
pub struct Lexicon {
    pub salt: String,
    pub addresses: HashSet<u64>,
    pub names: HashSet<u64>,
    /// Per table, each value's name hash and its code.
    pub codes: BTreeMap<u32, BTreeMap<u64, u32>>,
    pub fields: HashSet<u64>,
}

impl Lexicon {
    pub fn has(&self, set: &HashSet<u64>, item: &str) -> bool {
        set.contains(&text::hash(&self.salt, item))
    }

    pub fn load(dir: &Path) -> Result<Self, String> {
        let read = |name: &str| {
            fs::read_to_string(dir.join(name))
                .map_err(|e| format!("{}: {e}", dir.join(name).display()))
        };
        let parse = |s: &str| u64::from_str_radix(s, 16).map_err(|e| format!("bad hash {s}: {e}"));
        let set = |name: &str| -> Result<HashSet<u64>, String> {
            read(name)?
                .lines()
                .filter(|l| !l.is_empty())
                .map(parse)
                .collect()
        };
        let mut codes: BTreeMap<u32, BTreeMap<u64, u32>> = BTreeMap::new();
        for line in read("codes")?.lines() {
            let mut parts = line.split(' ');
            if let (Some(table), Some(code), Some(h)) = (parts.next(), parts.next(), parts.next()) {
                let table = table
                    .parse()
                    .map_err(|e| format!("bad table {table}: {e}"))?;
                let code = code.parse().map_err(|e| format!("bad code {code}: {e}"))?;
                codes.entry(table).or_default().insert(parse(h)?, code);
            }
        }
        Ok(Self {
            salt: read("salt")?.trim().to_owned(),
            addresses: set("addresses")?,
            names: set("names")?,
            codes,
            fields: set("fields")?,
        })
    }
}

/// The plain lexicon, before hashing; only the build holds it.
#[derive(Default)]
pub struct Plain {
    addresses: BTreeSet<String>,
    names: BTreeSet<String>,
    tables: BTreeMap<String, BTreeMap<String, u32>>,
    fields: BTreeSet<String>,
    /// The WRAM source constants seen so far; feed `source.rs` first.
    sources: BTreeSet<String>,
}

impl Plain {
    fn name(&mut self, name: &str) {
        self.names.extend(
            text::name_ngrams(name)
                .into_iter()
                .map(|g| format!("n:{g}")),
        );
    }

    fn code(&mut self, table: &str, code: u32, name: &str) {
        let name = text::normalise(name);
        if !name.is_empty() && !name.contains(char::is_numeric) {
            let entry = self.tables.entry(text::normalise(table)).or_default();
            entry.entry(name).or_insert(code);
        }
    }

    /// A bank log (the `.txt` form of a bank page): routine headings, the RAM it names and
    /// the codes its comments give names to.
    pub fn bank_log(&mut self, src: &str) {
        let bank = src.lines().find_map(bank_of_log_line).unwrap_or_default();
        let mut immediate: Option<u32> = None;
        for line in src.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix(";;; $") {
                self.heading(&bank, rest);
                continue;
            }
            for (pos, _) in line.match_indices("[$7E:") {
                if let Some(hex) = line.get(pos.saturating_add(5)..pos.saturating_add(9)) {
                    self.addresses.insert(format!("w:{}", hex.to_uppercase()));
                }
            }
            let Some((code, comment)) = line.split_once(';') else {
                continue;
            };
            let comment = comment.trim_start_matches(['\\', '}', '/', '{', ' ']);
            let value = code
                .split_once("#$")
                .and_then(|(_, v)| u32::from_str_radix(v.get(..4).unwrap_or(v).trim(), 16).ok());
            if code.contains("LDA #$") {
                immediate = value;
            }
            for bracket in comment.split('[').skip(1).filter_map(|b| b.split_once(']')) {
                if !bracket.0.starts_with('$') {
                    self.name(bracket.0);
                }
            }
            let Some((lhs, rhs)) = comment
                .split_once(" = ")
                .or_else(|| comment.split_once(" != "))
            else {
                continue;
            };
            let var = lhs.trim_start_matches("If ").trim_matches(['[', ']', ' ']);
            let rhs = rhs
                .trim_end_matches([':', ' '])
                .split(':')
                .next()
                .unwrap_or_default();
            self.name(var);
            let code = if code.contains("CMP #$") {
                value
            } else if code.contains("STZ ") {
                Some(0)
            } else if code.contains("STA ") {
                immediate
            } else {
                None
            };
            if let Some(code) = code
                && rhs.split_whitespace().count() <= 3
            {
                self.code(var, code, rhs);
            }
        }
    }

    fn heading(&mut self, bank: &str, rest: &str) {
        let Some((addr, name)) = rest.split_once(": ") else {
            return;
        };
        let offset = addr.split("..").next().unwrap_or_default().to_uppercase();
        if offset.len() != 4 || !offset.chars().all(|c| c.is_ascii_hexdigit()) {
            return;
        }
        self.addresses.insert(format!("o:{offset}"));
        if !bank.is_empty() {
            self.addresses.insert(format!("r:{bank}{offset}"));
        }
        self.name(name.trim_end_matches([';', ' ']));
    }

    /// Rust on the observer side: the WRAM sources and their names, every match arm from a
    /// code to a variant, every enum's declared order, and every struct field filled from one
    /// source as is.
    pub fn observer_rust(&mut self, src: &str) {
        let toks: Vec<Tok> = text::rust_tokens(src).into_iter().map(|(t, _)| t).collect();
        let ident = |i: usize| match toks.get(i) {
            Some(Tok::Ident(s)) => Some(s.as_str()),
            _ => None,
        };
        let punct = |i: usize, c: char| toks.get(i) == Some(&Tok::Punct(c));
        let mut impl_target = String::new();
        for i in 0..toks.len() {
            let next = i.saturating_add(1);
            match (ident(i), toks.get(next)) {
                (Some("offset"), Some(Tok::Punct(':'))) => {
                    if let Some(Tok::Num(n)) = toks.get(i.saturating_add(2))
                        && let Some(v) = text::literal(n)
                    {
                        self.addresses.insert(format!("w:{v:04X}"));
                    }
                }
                (Some("const"), _) => {
                    if let (Some(name), Some(kind)) = (ident(next), ident(i.saturating_add(3)))
                        && kind.starts_with("Wram")
                    {
                        self.sources.insert(name.to_owned());
                        self.name(&name.replace('_', " "));
                    }
                }
                (Some("impl"), _) => {
                    let mut j = next;
                    let mut target = None;
                    while let Some(t) = toks.get(j) {
                        match t {
                            Tok::Punct('{') => break,
                            Tok::Ident(s) if s == "for" => target = None,
                            Tok::Ident(s)
                                if target.is_none() && s.starts_with(char::is_uppercase) =>
                            {
                                target = Some(s.clone());
                            }
                            Tok::Ident(_) | Tok::Num(_) | Tok::Punct(_) | Tok::Str => {}
                        }
                        j = j.saturating_add(1);
                    }
                    impl_target = target.unwrap_or_default();
                }
                (Some("enum"), Some(Tok::Ident(name))) => {
                    for (k, (variant, code)) in enum_variants(&toks, next).into_iter().enumerate() {
                        let code = code.or_else(|| u32::try_from(k).ok()).unwrap_or_default();
                        self.code(&format!("enum {name}"), code, &variant);
                    }
                }
                _ => {}
            }
            if let Some(Tok::Num(n)) = toks.get(i)
                && !punct(i.wrapping_sub(1), '.')
                && let Some((ty, variant)) = arm_target(&toks, next)
                && let Some(code) = text::literal(n)
            {
                let ty = if ty == "Self" {
                    impl_target.as_str()
                } else {
                    ty
                };
                if !ty.is_empty() {
                    self.code(&format!("match {ty}"), code, variant);
                }
            }
        }
        let sources = self.sources.clone();
        self.struct_literals(&toks, &sources);
    }

    /// `Struct { field: <one source, as is> }` in a decoder.
    fn struct_literals(&mut self, toks: &[Tok], sources: &BTreeSet<String>) {
        let is_source = |s: &str| sources.contains(s);
        let mut lets: BTreeMap<String, String> = BTreeMap::new();
        let mut i = 0usize;
        while let Some(t) = toks.get(i) {
            let next = i.saturating_add(1);
            if t == &Tok::Ident("let".to_owned())
                && let Some(Tok::Ident(name)) = toks.get(next)
                && let Some(single) = single_source(toks, i.saturating_add(3), &is_source, true)
            {
                lets.insert(name.clone(), single);
            }
            if let (Tok::Ident(ty), Some(Tok::Punct('{'))) = (t, toks.get(next))
                && ty.starts_with(char::is_uppercase)
                && !matches!(
                    toks.get(i.wrapping_sub(1)),
                    Some(Tok::Ident(k)) if ["struct", "enum", "impl", "for", "trait", "union"].contains(&k.as_str())
                )
            {
                let mut j = i.saturating_add(2);
                while let Some(Tok::Ident(field)) = toks.get(j) {
                    let after = j.saturating_add(1);
                    let (copied, end) = match toks.get(after) {
                        Some(Tok::Punct(':')) => {
                            let start = after.saturating_add(1);
                            let via_let = match (toks.get(start), toks.get(start.saturating_add(1)))
                            {
                                (Some(Tok::Ident(n)), Some(Tok::Punct(',' | '}'))) => {
                                    lets.contains_key(n)
                                }
                                _ => false,
                            };
                            let single = single_source(toks, start, &is_source, false).is_some();
                            (via_let || single, skip_expr(toks, start))
                        }
                        Some(Tok::Punct(',' | '}')) => (lets.contains_key(field), after),
                        _ => break,
                    };
                    if copied {
                        self.fields.insert(format!("f:{ty}.{field}"));
                    }
                    if !matches!(toks.get(end), Some(Tok::Punct(','))) {
                        break;
                    }
                    j = end.saturating_add(1);
                }
            }
            i = next;
        }
    }

    pub fn distinctive(&mut self, word: &str) {
        self.names.insert(format!("u:{}", text::normalise(word)));
    }

    /// Writes the hashed files under `dir` with a fresh salt.
    pub fn write(&self, dir: &Path, salt: &str) -> Result<String, String> {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let hashed = |set: &BTreeSet<String>| {
            let mut v: Vec<u64> = set.iter().map(|s| text::hash(salt, s)).collect();
            v.sort_unstable();
            v.dedup();
            v.iter().fold(String::new(), |mut out, h| {
                let _ = writeln!(out, "{h:016x}");
                out
            })
        };
        let mut codes = String::new();
        let mut kept = 0usize;
        for (names, table) in self.tables.values().filter(|t| t.len() >= 2).zip(0u32..) {
            kept = kept.saturating_add(1);
            for (name, code) in names {
                let _ = writeln!(codes, "{table} {code} {:016x}", text::hash(salt, name));
            }
        }
        let contents = [
            hashed(&self.addresses),
            hashed(&self.names),
            codes,
            hashed(&self.fields),
        ];
        for (file, body) in FILES.iter().zip(&contents) {
            fs::write(dir.join(file), body).map_err(|e| format!("{file}: {e}"))?;
        }
        fs::write(dir.join("salt"), format!("{salt}\n")).map_err(|e| format!("salt: {e}"))?;
        Ok(format!(
            "lexicon: {} addresses, {} names, {kept} code tables, {} fields\n",
            self.addresses.len(),
            self.names.len(),
            self.fields.len()
        ))
    }
}

/// The bank a bank log line's instruction or heading sits in, from the nearest `$BB:` on it.
fn bank_of_log_line(line: &str) -> Option<String> {
    let pos = line.find('$')?;
    let bank = line.get(pos.saturating_add(1)..pos.saturating_add(3))?;
    (line.get(pos.saturating_add(3)..pos.saturating_add(4)) == Some(":"))
        .then(|| bank.to_uppercase())
}

/// An enum's variants after `name_at` (the token holding its name), each with its explicit
/// discriminant.
pub fn enum_variants(toks: &[Tok], name_at: usize) -> Vec<(String, Option<u32>)> {
    let mut i = name_at;
    while toks
        .get(i)
        .is_some_and(|t| t != &Tok::Punct('{') && t != &Tok::Punct(';'))
    {
        i = i.saturating_add(1);
    }
    if toks.get(i) != Some(&Tok::Punct('{')) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut expect = true;
    while let Some(t) = toks.get(i) {
        match t {
            Tok::Punct('{' | '(' | '[') => depth = depth.saturating_add(1),
            Tok::Punct('}' | ')' | ']') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            Tok::Punct(',') if depth == 1 => expect = true,
            Tok::Punct('#') if depth == 1 => {}
            Tok::Ident(name) if depth == 1 && expect => {
                expect = false;
                let code = match (toks.get(i.saturating_add(1)), toks.get(i.saturating_add(2))) {
                    (Some(Tok::Punct('=')), Some(Tok::Num(n))) => text::literal(n),
                    _ => None,
                };
                out.push((name.clone(), code));
            }
            Tok::Ident(_) | Tok::Num(_) | Tok::Punct(_) | Tok::Str => {}
        }
        i = i.saturating_add(1);
    }
    out
}

/// After a match arm's first literal: further `| literal`s, an optional guard, `=>` and
/// `Type::Variant`.
fn arm_target(toks: &[Tok], mut i: usize) -> Option<(&str, &str)> {
    loop {
        match (toks.get(i)?, toks.get(i.saturating_add(1))?) {
            (Tok::Punct('|'), Tok::Num(_)) => i = i.saturating_add(2),
            (Tok::Punct('.'), Tok::Punct('.')) => i = i.saturating_add(3),
            (Tok::Punct('='), Tok::Punct('>')) => break,
            _ => return None,
        }
    }
    match (
        toks.get(i.saturating_add(2))?,
        toks.get(i.saturating_add(3))?,
        toks.get(i.saturating_add(5))?,
    ) {
        (Tok::Ident(ty), Tok::Punct(':'), Tok::Ident(variant)) => Some((ty, variant)),
        _ => None,
    }
}

/// The index past an expression that ends at a `,` or closing bracket of its own depth.
fn skip_expr(toks: &[Tok], mut i: usize) -> usize {
    let mut depth = 0i32;
    while let Some(t) = toks.get(i) {
        match t {
            Tok::Punct('{' | '(' | '[') => depth = depth.saturating_add(1),
            Tok::Punct('}' | ')' | ']' | ',' | ';') if depth == 0 => break,
            Tok::Punct('}' | ')' | ']') => depth = depth.saturating_sub(1),
            Tok::Ident(_) | Tok::Num(_) | Tok::Punct(_) | Tok::Str => {}
        }
        i = i.saturating_add(1);
    }
    i
}

/// The one source an expression reads as is: no arithmetic, at most `!= 0`, with a value and
/// its sub-unit word (`X` and `X_SUB…`) counting as one. `matches` lets a `match` on the
/// source count, for a `let` whose arms map codes.
fn single_source(
    toks: &[Tok],
    start: usize,
    is_source: &impl Fn(&str) -> bool,
    matches: bool,
) -> Option<String> {
    let end = skip_expr(toks, start);
    let expr = toks.get(start..end)?;
    let mut found: Vec<&str> = Vec::new();
    let mut is_match = false;
    for t in expr {
        match t {
            Tok::Ident(s) if s == "match" => is_match = true,
            Tok::Ident(s) if s.len() > 3 && s.contains('_') && is_source(s) => found.push(s),
            Tok::Punct('+' | '-' | '*' | '/' | '&' | '^' | '<' | '>') if !matches => return None,
            Tok::Ident(_) | Tok::Num(_) | Tok::Punct(_) | Tok::Str => {}
        }
    }
    if is_match && !matches {
        return None;
    }
    found.dedup();
    match found.as_slice() {
        [one] => Some((*one).to_owned()),
        [value, sub] if sub.contains("SUB") => Some((*value).to_owned()),
        _ => None,
    }
}

impl Plain {
    /// Decode notes: every backticked 4-digit RAM word, and every `N Name, N Name` list (a code
    /// table written out) on one line.
    pub fn decode_notes(&mut self, src: &str) {
        for (n, line) in src.lines().enumerate() {
            for quoted in line.split('`').skip(1).step_by(2) {
                for part in quoted.split('.') {
                    if part.len() == 4 && part.chars().all(|c| c.is_ascii_hexdigit()) {
                        self.addresses.insert(format!("w:{}", part.to_uppercase()));
                    }
                }
            }
            for list in line.split([':', ';', '(']) {
                let pairs: Vec<(u32, &str)> = list
                    .split([',', '.'])
                    .filter_map(|item| {
                        let mut it = item.split_whitespace().filter(|w| *w != "and");
                        let code = u32::from_str_radix(it.next()?, 16).ok()?;
                        let name = it.next()?.trim_matches('`');
                        (it.next().is_none() && name.starts_with(char::is_uppercase))
                            .then_some((code, name))
                    })
                    .collect();
                if pairs.len() >= 2 {
                    for (code, name) in pairs {
                        self.code(&format!("notes {n}"), code, name);
                    }
                }
            }
        }
    }
}
