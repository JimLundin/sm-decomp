//! What every check shares: the salted hash, normalised words and their phrases, and a small
//! Rust tokeniser.

use std::collections::BTreeSet;

/// FNV-1a over the salt, a separator and the item. The salt is public, so a reader with the
/// salt can test a guess; what the files don't give is a list to read.
pub fn hash(salt: &str, item: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in salt.bytes().chain([0]).chain(item.bytes()) {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Words that carry no name on their own; they are dropped without breaking a phrase, so
/// "open Samus' door" and "open the door" normalise alike.
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "of", "to", "if", "is", "are", "was", "be", "been", "and", "or", "for", "in",
    "on", "at", "by", "with", "from", "as", "this", "that", "it", "its", "into", "onto", "than",
    "then", "when", "while", "so", "but", "she", "her", "he", "his", "him", "they", "them",
    "their", "samus", "has", "have", "does", "do", "s", "self", "mut", "let", "fn", "pub", "use",
    "crate", "super", "const", "impl", "struct", "enum", "match", "return", "else", "t",
];

/// A light stemmer: the same on both sides, so only consistency matters.
pub fn stem(word: &str) -> String {
    let mut w = word.to_owned();
    for suffix in ["ing", "ed", "es", "s", "e"] {
        let keep = w.len().saturating_sub(suffix.len());
        if w.ends_with(suffix)
            && keep >= if suffix == "ing" { 4 } else { 3 }
            && !(suffix == "ed" && w.ends_with("eed"))
        {
            w.truncate(keep);
            break;
        }
    }
    let bytes = w.as_bytes();
    if let [.., a, b] = bytes
        && a == b
        && !matches!(a, b'l' | b's' | b'z' | b'e' | b'o')
        && bytes.len() > 3
    {
        w.pop();
    }
    w
}

/// One normalised word: its stem, the line it is on, and whether it continues the phrase
/// before it.
#[derive(Clone, Debug)]
pub struct Word {
    pub stem: String,
    pub line: usize,
    pub joined: bool,
    /// The source token it came from, when that token was a compound identifier
    /// (`OpenTheDoor`, `open_the_door`).
    pub compound: Option<usize>,
}

/// Splits an identifier at its case changes: `OpenTheDoor` is open, the, door.
fn split_camel(token: &str) -> Vec<String> {
    let chars: Vec<char> = token.chars().collect();
    let mut parts = Vec::new();
    let mut current = String::new();
    for (i, &c) in chars.iter().enumerate() {
        let prev = i.checked_sub(1).and_then(|p| chars.get(p)).copied();
        let next = chars.get(i.saturating_add(1)).copied();
        let boundary = prev.is_some_and(|p| {
            (p.is_lowercase() || p.is_ascii_digit()) && c.is_uppercase()
                || p.is_uppercase() && c.is_uppercase() && next.is_some_and(char::is_lowercase)
        });
        if boundary && !current.is_empty() {
            parts.push(std::mem::take(&mut current));
        }
        current.push(c);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Separators that end a phrase. Spaces, underscores, hyphens, slashes, quotes and backticks
/// join words; sentence and code punctuation doesn't.
const fn breaks(c: char) -> bool {
    matches!(
        c,
        '.' | ','
            | ';'
            | ':'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '|'
            | '='
            | '<'
            | '>'
            | '!'
            | '?'
            | '"'
            | '+'
            | '*'
            | '&'
            | '#'
            | '%'
            | '$'
            | '@'
    )
}

/// Every word of `text`, normalised: case split, lowercased, stemmed, stopwords and numbers
/// dropped.
pub fn words(text: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut line = 1usize;
    let mut token = String::new();
    let mut token_line = 1usize;
    let mut broken = true;
    let mut count = 0usize;
    let mut flush = |token: &mut String, token_line: usize, broken: &mut bool| {
        if token.is_empty() {
            return;
        }
        let parts: Vec<String> = token.split('_').flat_map(split_camel).collect();
        let compound = (parts.len() > 1).then_some(count);
        count = count.wrapping_add(1);
        for part in parts {
            let lower = part.to_lowercase();
            if lower.chars().any(|c| c.is_ascii_digit()) || STOPWORDS.contains(&lower.as_str()) {
                continue;
            }
            out.push(Word {
                stem: stem(&lower),
                line: token_line,
                joined: !*broken,
                compound,
            });
            *broken = false;
        }
        token.clear();
    };
    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' {
            if token.is_empty() {
                token_line = line;
            }
            token.push(c);
            continue;
        }
        flush(&mut token, token_line, &mut broken);
        if c == '\'' {
            continue;
        }
        if breaks(c) {
            broken = true;
        }
        if c == '\n' {
            line = line.saturating_add(1);
        }
    }
    flush(&mut token, token_line, &mut broken);
    out
}

/// The normalised form of a short phrase, as the lexicon and the allowlist hold it.
pub fn normalise(phrase: &str) -> String {
    words(phrase)
        .into_iter()
        .map(|w| w.stem)
        .collect::<Vec<_>>()
        .join(" ")
}

/// The n-grams (2 to 4 words) of each phrase-joined run of `words`, with the index of their
/// first word.
pub fn ngrams(words: &[Word]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    for start in 0..words.len() {
        for n in 2..=4usize {
            let Some(end) = start.checked_add(n) else {
                continue;
            };
            let Some(run) = words.get(start..end) else {
                break;
            };
            if run.iter().skip(1).any(|w| !w.joined) {
                break;
            }
            let text = run
                .iter()
                .map(|w| w.stem.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            out.push((start, n, text));
        }
    }
    out
}

/// Every 2-4 word n-gram of a lexicon name, split at the name's own punctuation.
pub fn name_ngrams(name: &str) -> BTreeSet<String> {
    ngrams(&words(name))
        .into_iter()
        .map(|(_, _, text)| text)
        .collect()
}

/// A token of Rust source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok {
    Ident(String),
    Num(String),
    Punct(char),
    Str,
}

/// Rust source as tokens with their lines; comments and string contents are dropped.
pub fn rust_tokens(src: &str) -> Vec<(Tok, usize)> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let at = |i: usize| chars.get(i).copied();
    while let Some(c) = at(i) {
        let next = at(i.saturating_add(1));
        if c == '\n' {
            line = line.saturating_add(1);
            i = i.saturating_add(1);
        } else if c == '/' && next == Some('/') {
            while at(i).is_some_and(|c| c != '\n') {
                i = i.saturating_add(1);
            }
        } else if c == '/' && next == Some('*') {
            while let Some(c) = at(i) {
                if c == '\n' {
                    line = line.saturating_add(1);
                }
                if c == '*' && at(i.saturating_add(1)) == Some('/') {
                    i = i.saturating_add(2);
                    break;
                }
                i = i.saturating_add(1);
            }
        } else if c == '"' {
            i = i.saturating_add(1);
            while let Some(c) = at(i) {
                i = i.saturating_add(1);
                match c {
                    '\\' => i = i.saturating_add(1),
                    '"' => break,
                    '\n' => line = line.saturating_add(1),
                    _ => {}
                }
            }
            out.push((Tok::Str, line));
        } else if c == '\'' && at(i.saturating_add(2)) == Some('\'') {
            i = i.saturating_add(3);
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while at(i).is_some_and(|c| c.is_alphanumeric() || c == '_') {
                i = i.saturating_add(1);
            }
            out.push((
                Tok::Ident(chars.get(start..i).unwrap_or_default().iter().collect()),
                line,
            ));
        } else if c.is_ascii_digit() {
            let start = i;
            while at(i).is_some_and(|c| c.is_alphanumeric() || c == '_') {
                i = i.saturating_add(1);
            }
            out.push((
                Tok::Num(chars.get(start..i).unwrap_or_default().iter().collect()),
                line,
            ));
        } else {
            if !c.is_whitespace() {
                out.push((Tok::Punct(c), line));
            }
            i = i.saturating_add(1);
        }
    }
    out
}

/// The value of an integer literal: decimal, `0x` hex or `0b` binary, with `_` and a type
/// suffix allowed.
pub fn literal(text: &str) -> Option<u32> {
    let clean: String = text.chars().filter(|&c| c != '_').collect();
    let (digits, radix) = clean.strip_prefix("0x").map_or_else(
        || {
            clean
                .strip_prefix("0b")
                .map_or((clean.as_str(), 10), |d| (d, 2))
        },
        |d| (d, 16),
    );
    let end = digits.find(['u', 'i']).unwrap_or(digits.len());
    u32::from_str_radix(digits.get(..end)?, radix).ok()
}
