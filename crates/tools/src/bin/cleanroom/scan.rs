//! Checks 1 to 3 over any text: addresses, names and (in Rust) shapes.

use std::borrow::Cow;
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

/// Recognize only the absolute `::core::ptr::eq(...)` code spelling. This is a lexical
/// Name exception: library identity additionally requires the clean crate's mandatory
/// edition, manifest and extern-declaration gates; this scanner does not resolve bindings.
fn identity_names(src: &str) -> Cow<'_, str> {
    let tokens = text::rust_spans(src);
    let mut masked = None;
    for (start, path) in tokens.windows(10).enumerate() {
        let matches = matches!(
            path.iter().map(|t| &t.token).collect::<Vec<_>>().as_slice(),
            [Tok::Punct(':'), Tok::Punct(':'), Tok::Ident(core),
             Tok::Punct(':'), Tok::Punct(':'), Tok::Ident(ptr),
             Tok::Punct(':'), Tok::Punct(':'), Tok::Ident(eq), Tok::Punct('(')]
                if core == "core" && ptr == "ptr" && eq == "eq"
        );
        if !matches
            || ![0usize, 3, 6].into_iter().all(|i| {
                path.get(i)
                    .zip(path.get(i.saturating_add(1)))
                    .is_some_and(|(left, right)| left.end == right.start)
            })
        {
            continue;
        }
        // A preceding path segment, field, raw identifier or extra colon is not an
        // absolute root. Accept expression-start punctuation and the return keyword.
        if start
            .checked_sub(1)
            .and_then(|i| tokens.get(i))
            .is_some_and(|t| {
                !matches!(
                    t.token,
                    Tok::Punct(
                        '=' | '('
                            | '['
                            | '{'
                            | '}'
                            | ','
                            | ';'
                            | '!'
                            | '&'
                            | '|'
                            | '+'
                            | '-'
                            | '*'
                            | '/'
                            | '%'
                            | '^'
                    )
                ) && !matches!(&t.token, Tok::Ident(word) if word == "return")
            })
        {
            continue;
        }
        if let Some(ptr) = path.get(5) {
            let text = masked.get_or_insert_with(|| src.to_owned());
            text.replace_range(ptr.start..ptr.end, "   ");
        }
    }
    masked.map_or(Cow::Borrowed(src), Cow::Owned)
}

/// Rust's non-zero integers, which the name check would split into the words "non zero". One is
/// masked only where a `.rs` file's code writes it under its module, `num::NonZeroU16` (after
/// `use core::num;`, or within `core::num::NonZeroU16`): the spelling no declaration of the
/// file's own can take. A bare `NonZeroU16`, a declaration, a comment, a string and any other file
/// are always scanned. Every entry is a spelling the barrier no longer reads, so an addition needs
/// the same anchoring and a reviewer's sign-off.
const NUM_TYPES: &[&str] = &[
    "NonZeroU8",
    "NonZeroU16",
    "NonZeroU32",
    "NonZeroU64",
    "NonZeroU128",
    "NonZeroUsize",
    "NonZeroI8",
    "NonZeroI16",
    "NonZeroI32",
    "NonZeroI64",
    "NonZeroI128",
    "NonZeroIsize",
];

/// `src` with each of `NUM_TYPES` written as `num::` its name blanked; the `::` before it already
/// ends the phrase, so no words join across it.
fn library_names(src: &str) -> Cow<'_, str> {
    let tokens = text::rust_spans(src);
    let mut masked: Option<String> = None;
    for path in tokens.windows(4) {
        let [module, first, second, name] = path else {
            continue;
        };
        let qualified = matches!(&module.token, Tok::Ident(word) if word == "num")
            && first.token == Tok::Punct(':')
            && second.token == Tok::Punct(':')
            && first.end == second.start;
        let Tok::Ident(word) = &name.token else {
            continue;
        };
        if !qualified || !NUM_TYPES.contains(&word.as_str()) {
            continue;
        }
        let text = masked.get_or_insert_with(|| src.to_owned());
        text.replace_range(
            name.start..name.end,
            &" ".repeat(name.end.saturating_sub(name.start)),
        );
    }
    masked.map_or(Cow::Borrowed(src), Cow::Owned)
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
    let rust = std::path::Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("rs"));
    let name_src = if rust {
        Cow::Owned(library_names(&identity_names(src)).into_owned())
    } else {
        Cow::Borrowed(src)
    };
    names(lex, allow, &name_src, &mut push);
    if rust {
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
    "09486a2fd",
    "sm/src",
    "ram_map",
    "collab/reviews",
    "verify/src",
    "rom/sm.smc",
];

#[cfg(test)]
mod tests {
    use super::{Allow, Class, file};
    use crate::{lexicon::Lexicon, text};

    #[test]
    fn identity_exception_preserves_known_addresses_and_shapes() {
        // All lexicon entries here are invented checker witnesses.
        let mut lex = Lexicon::default();
        lex.addresses.insert(text::hash(&lex.salt, "w:AB12"));
        lex.fields
            .insert(text::hash(&lex.salt, "f:Example.payload"));
        lex.codes.insert(
            0,
            ["Alpha", "Bravo", "Charlie"]
                .into_iter()
                .zip(0..)
                .map(|(name, code)| (text::hash(&lex.salt, &text::normalise(name)), code))
                .collect(),
        );
        let leak = "0xAB12; struct Example { payload: u16 } enum Kind { Alpha, Bravo, Charlie }";
        let scan = |src: &str| {
            let mut out = Vec::new();
            file(&lex, &Allow::parse(""), "x.rs", src, &mut out);
            out.into_iter()
                .map(|f| (f.line, f.class))
                .collect::<Vec<_>>()
        };
        let expected = [
            (1, Class::AddressKnown),
            (1, Class::RamField),
            (1, Class::EnumOrder),
        ];
        assert_eq!(scan(leak), expected);
        assert_eq!(scan(&format!("::core::ptr::eq(a, b); {leak}")), expected);
    }

    /// The lines of `src`, scanned as `path`, with a name finding, against an invented lexicon of
    /// the names "non zero" and "read wait".
    fn name_lines(path: &str, src: &str) -> Vec<usize> {
        let mut lex = Lexicon::default();
        for name in ["non zero", "read wait"] {
            lex.names.insert(text::hash(
                &lex.salt,
                &format!("n:{}", text::normalise(name)),
            ));
        }
        let mut out = Vec::new();
        file(&lex, &Allow::parse(""), path, src, &mut out);
        out.into_iter()
            .filter(|f| f.class == Class::Name)
            .map(|f| f.line)
            .collect()
    }

    #[test]
    fn non_zero_integers_written_under_num_are_not_the_phrase() {
        let none = Vec::<usize>::new();
        let src = "use core::num;\nstruct Wait(num::NonZeroU16);\nfn f(n: num::NonZeroU8) {}";
        assert_eq!(name_lines("x.rs", src), none);
        assert_eq!(
            name_lines("x.rs", "struct Wait(core::num::NonZeroU16);"),
            none
        );
        assert_eq!(
            name_lines("x.rs", "struct Wait(::std::num::NonZeroU32);"),
            none
        );
    }

    #[test]
    fn every_other_spelling_is_still_the_phrase() {
        for src in [
            // Bare, even when imported: the import itself is under `num::`, the use isn't.
            "use core::num::NonZeroU16;\nstruct Wait(NonZeroU16);",
            // The file's own names.
            "enum Wait { NonZero, Zero }",
            "enum Wait { NonZeroU16 }",
            "struct NonZero;",
            "type NonZeroU8 = u8;",
            "let r#NonZero = 1;",
            "struct S { NonZeroU8: u8 }",
            // A module of the file's own named `num`: its declaration still scans.
            "mod num { pub struct NonZeroU16; }\nfn f(_: num::NonZeroU16) {}",
            // Comments and strings, even written under `num::`.
            "/// The `NonZero` gate, a num::NonZeroU16.",
            "let s = \"num::NonZeroU16\";",
        ] {
            assert_ne!(name_lines("x.rs", src), Vec::<usize>::new(), "{src} passed");
        }
        assert_ne!(
            name_lines(
                "brief.md",
                "The `NonZero` routine, a `core::num::NonZeroU16`."
            ),
            Vec::<usize>::new()
        );
    }
}
