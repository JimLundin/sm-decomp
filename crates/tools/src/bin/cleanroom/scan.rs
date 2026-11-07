//! Checks 1 to 3 over any text: addresses, names and (in Rust) shapes.

use std::collections::BTreeSet;
use std::fmt;

use crate::lexicon::{self, Lexicon};
use crate::text::{self, Tok};

/// What a finding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// `$BB:OOOO`, `$OOOO`, `0xBB, 0xOOOO`, a `bank NN`, an address constructor.
    AddressForm,
    /// A hex token that is a known WRAM or routine address.
    AddressKnown,
    /// An n-gram of the original's routine and variable names.
    Name,
    /// An enum whose variants follow a known code table's order.
    EnumOrder,
    /// A struct field the decoder fills from one word as is.
    RamField,
    /// A spec entry whose source is one routine (counted, gated by share).
    SingleRoutine,
    /// A public model function that maps onto one routine.
    EntryPoint,
    /// A pointer to a forbidden source: the bank logs, the decode notes, the old code.
    Route,
}

impl Class {
    pub const ALL: [Self; 8] = [
        Self::AddressForm,
        Self::AddressKnown,
        Self::Name,
        Self::EnumOrder,
        Self::RamField,
        Self::SingleRoutine,
        Self::EntryPoint,
        Self::Route,
    ];
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AddressForm => "address-form",
            Self::AddressKnown => "address-known",
            Self::Name => "name",
            Self::EnumOrder => "enum-order",
            Self::RamField => "ram-field",
            Self::SingleRoutine => "single-routine",
            Self::EntryPoint => "entry-point",
            Self::Route => "route",
        })
    }
}

/// One finding: where, what, and the scanned text's own words (never the lexicon's).
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub class: Class,
    pub detail: String,
}

/// The reviewed player vocabulary: whole phrases, and single words any phrase may be made of.
pub struct Allow {
    phrases: BTreeSet<String>,
    words: BTreeSet<String>,
}

impl Allow {
    pub fn parse(src: &str) -> Self {
        let mut phrases = BTreeSet::new();
        let mut words = BTreeSet::new();
        for line in src.lines() {
            let entry = line.split('#').next().unwrap_or_default().trim();
            let norm = text::normalise(entry);
            if norm.is_empty() {
                continue;
            }
            if norm.contains(' ') {
                phrases.insert(norm);
            } else {
                words.insert(norm);
            }
        }
        Self { phrases, words }
    }

    fn passes(&self, gram: &str, identifier: bool) -> bool {
        self.phrases.contains(gram)
            || (!identifier && gram.split(' ').all(|w| self.words.contains(w)))
    }
}

const fn hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn hex_run(chars: &[char], from: usize) -> usize {
    chars.iter().skip(from).take_while(|&&c| hex(c)).count()
}

/// Check 1 on one line.
fn addresses(lex: &Lexicon, line: &str, mut push: impl FnMut(Class, String)) {
    let chars: Vec<char> = line.chars().collect();
    let lower = line.to_lowercase();
    for marker in ["romaddress::new(", "source::bank(", "lorom::", "lorom("] {
        if lower.contains(marker) {
            push(
                Class::AddressForm,
                marker.trim_end_matches(['(', ':']).to_owned(),
            );
        }
    }
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '$')
        .collect();
    for pair in words.windows(2) {
        if let [first, second] = pair
            && (*first == "bank" || *first == "banks")
            && second.trim_start_matches('$').len() == 2
            && second.trim_start_matches('$').chars().all(hex)
        {
            push(Class::AddressForm, format!("bank {second}"));
        }
    }
    let mut i = 0usize;
    while let Some(&c) = chars.get(i) {
        let prev = i.checked_sub(1).and_then(|p| chars.get(p)).copied();
        let word_start = !prev.is_some_and(|p| p.is_alphanumeric() || p == '_' || p == '.');
        let (prefix, start) = if c == '$' {
            ("$", i.saturating_add(1))
        } else if c == '0' && chars.get(i.saturating_add(1)) == Some(&'x') && word_start {
            ("0x", i.saturating_add(2))
        } else if word_start && hex(c) {
            ("", i)
        } else {
            i = i.saturating_add(1);
            continue;
        };
        let run = hex_run(&chars, start);
        let end = start.saturating_add(run);
        let after = chars.get(end).copied();
        let token: String = chars.get(start..end).unwrap_or_default().iter().collect();
        i = end.max(i.saturating_add(1));
        if after.is_some_and(|a| a.is_alphanumeric() || a == '_' || a == '.') && after != Some(':')
        {
            continue;
        }
        if prefix == "$"
            && run == 2
            && after == Some(':')
            && hex_run(&chars, end.saturating_add(1)) == 4
        {
            push(Class::AddressForm, format!("${token}:"));
            i = end.saturating_add(5);
            continue;
        }
        if prefix == "$" && run >= 4 {
            push(Class::AddressForm, format!("${token}"));
            continue;
        }
        if prefix == "0x" && run == 2 {
            let rest: String = chars.iter().skip(end).collect();
            let rest = rest.trim_start_matches([',', ' ']);
            if rest.starts_with("0x") && hex_run(&rest.chars().collect::<Vec<_>>(), 2) == 4 {
                push(Class::AddressForm, format!("0x{token}, 0x…"));
            }
            continue;
        }
        // A bare token needs a digit and a letter to be read as hex at all.
        if prefix.is_empty()
            && !(token.chars().any(|c| c.is_ascii_digit())
                && token.chars().any(char::is_alphabetic))
        {
            continue;
        }
        if known(lex, &token) {
            push(Class::AddressKnown, format!("{prefix}{token}"));
        }
    }
}

fn known(lex: &Lexicon, token: &str) -> bool {
    let up = token.to_uppercase();
    // Round values (X000) are the usual sub-pixel fractions and bit masks of a clean model; a
    // round address still trips the `$` and split forms.
    if up.len() == 4 && up.ends_with("000") {
        return false;
    }
    match up.len() {
        4 => {
            lex.has(&lex.addresses, &format!("w:{up}"))
                || lex.has(&lex.addresses, &format!("o:{up}"))
        }
        6 => {
            lex.has(&lex.addresses, &format!("r:{up}"))
                || (up.starts_with("7E")
                    && up
                        .get(2..)
                        .is_some_and(|w| lex.has(&lex.addresses, &format!("w:{w}"))))
        }
        _ => false,
    }
}

/// Check 2 over a whole text: one finding per phrase run, with its longest hit. An n-gram
/// inside one compound identifier (`OpenTheDoor`) passes only as an allowed phrase: a name
/// built from player words is still the original's name.
fn names(lex: &Lexicon, allow: &Allow, src: &str, mut push: impl FnMut(usize, Class, String)) {
    let words = text::words(src);
    let mut covered_to = 0usize;
    let one_identifier = |start: usize, n: usize| {
        let run = words
            .get(start..start.saturating_add(n))
            .unwrap_or_default();
        run.first()
            .and_then(|w| w.compound)
            .is_some_and(|t| run.iter().all(|w| w.compound == Some(t)))
    };
    let mut hits: Vec<(usize, usize, String)> = text::ngrams(&words)
        .into_iter()
        .filter(|(start, n, g)| {
            lex.has(&lex.names, &format!("n:{g}")) && !allow.passes(g, one_identifier(*start, *n))
        })
        .collect();
    for (k, w) in words.iter().enumerate() {
        if lex.has(&lex.names, &format!("u:{}", w.stem)) && !allow.passes(&w.stem, false) {
            hits.push((k, 1, w.stem.clone()));
        }
    }
    hits.sort_by_key(|(start, n, _)| (*start, std::cmp::Reverse(*n)));
    for (start, n, gram) in hits {
        if start < covered_to {
            continue;
        }
        covered_to = start.saturating_add(n);
        let line = words.get(start).map_or(0, |w| w.line);
        push(line, Class::Name, gram);
    }
}

/// Check 3 on Rust: enums in a known code table's order, fields the decoder copies as is.
fn shapes(lex: &Lexicon, src: &str, mut push: impl FnMut(usize, Class, String)) {
    let toks = text::rust_tokens(src);
    let bare: Vec<Tok> = toks.iter().map(|(t, _)| t.clone()).collect();
    let mut i = 0usize;
    while let Some((t, line)) = toks.get(i) {
        let next = i.saturating_add(1);
        match (t, bare.get(next)) {
            (Tok::Ident(k), Some(Tok::Ident(name))) if k == "enum" => {
                let variants = lexicon::enum_variants(&bare, next);
                if let Some(detail) = enum_order(lex, &variants) {
                    push(*line, Class::EnumOrder, format!("{name}: {detail}"));
                }
            }
            (Tok::Ident(k), Some(Tok::Ident(name))) if k == "struct" => {
                for field in struct_fields(&bare, next) {
                    if lex.has(&lex.fields, &format!("f:{name}.{field}")) {
                        push(*line, Class::RamField, format!("{name}.{field}"));
                    }
                }
            }
            _ => {}
        }
        i = next;
    }
}

/// Whether `variants` follow one table: at least two (or every one of a two-variant enum)
/// named as the table's values, in rising code order; "value for value" when the implied
/// discriminants equal the codes.
fn enum_order(lex: &Lexicon, variants: &[(String, Option<u32>)]) -> Option<String> {
    let mut implied = 0u32;
    let named: Vec<(u64, u32)> = variants
        .iter()
        .map(|(v, explicit)| {
            let code = explicit.unwrap_or(implied);
            implied = code.saturating_add(1);
            (text::hash(&lex.salt, &text::normalise(v)), code)
        })
        .collect();
    let mut best: Option<(usize, bool)> = None;
    for table in lex.codes.values() {
        let hits: Vec<(u32, u32)> = named
            .iter()
            .filter_map(|(h, own)| table.get(h).map(|code| (*code, *own)))
            .collect();
        let rising = hits.windows(2).all(|w| matches!(w, [a, b] if a.0 < b.0));
        let enough = hits.len() >= 3;
        if rising && enough && best.is_none_or(|(n, _)| hits.len() > n) {
            best = Some((hits.len(), hits.iter().all(|(code, own)| code == own)));
        }
    }
    best.map(|(n, exact)| {
        format!(
            "{n} of {} variants in a RAM code table's order{}",
            variants.len(),
            if exact { ", value for value" } else { "" }
        )
    })
}

fn struct_fields(toks: &[Tok], name_at: usize) -> Vec<String> {
    let mut i = name_at;
    while toks
        .get(i)
        .is_some_and(|t| !matches!(t, Tok::Punct('{' | ';' | '(')))
    {
        i = i.saturating_add(1);
    }
    if toks.get(i) != Some(&Tok::Punct('{')) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut depth = 0i32;
    while let Some(t) = toks.get(i) {
        match t {
            Tok::Punct('{' | '(' | '[' | '<') => depth = depth.saturating_add(1),
            Tok::Punct('}' | ')' | ']' | '>') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            Tok::Ident(field)
                if depth == 1 && toks.get(i.saturating_add(1)) == Some(&Tok::Punct(':')) =>
            {
                out.push(field.clone());
            }
            Tok::Ident(_) | Tok::Num(_) | Tok::Punct(_) | Tok::Str => {}
        }
        i = i.saturating_add(1);
    }
    out
}

/// Every check that applies to `path`'s contents.
pub fn file(lex: &Lexicon, allow: &Allow, path: &str, src: &str, out: &mut Vec<Finding>) {
    let mut push = |line: usize, class: Class, detail: String| {
        out.push(Finding {
            path: path.to_owned(),
            line,
            class,
            detail,
        });
    };
    for (n, line) in src.lines().enumerate() {
        addresses(lex, line, |class, detail| {
            push(n.saturating_add(1), class, detail);
        });
        let lower = line.to_lowercase();
        for marker in ROUTES.iter().filter(|m| lower.contains(*m)) {
            push(n.saturating_add(1), Class::Route, (*marker).to_owned());
        }
    }
    names(lex, allow, src, &mut push);
    if std::path::Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("rs"))
    {
        shapes(lex, src, &mut push);
    }
}

/// Where the original and the observer's notes are: text that points an implementer at them.
const ROUTES: &[&str] = &[
    "patrickjohnston",
    "bank log",
    "c4a/logs",
    ".decode.md",
    "decode notes",
    "archive/old-engine",
    "sm/src",
    "ram_map",
    "collab/reviews",
    "verify/src",
    "rom/sm.smc",
];
