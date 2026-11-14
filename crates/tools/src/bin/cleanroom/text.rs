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

/// A token's original byte span, retained for slicing bodies without losing entry comments.
#[derive(Clone, Debug)]
pub struct SpannedToken {
    /// The code token; literal contents are opaque.
    pub token: Tok,
    /// Its opening line, one-based.
    pub line: usize,
    /// Inclusive byte offset in the source.
    pub start: usize,
    /// Exclusive byte offset in the source.
    pub end: usize,
}

/// Rust source as tokens with their lines; comments and literal contents are dropped.
pub fn rust_tokens(src: &str) -> Vec<(Tok, usize)> {
    rust_spans(src)
        .into_iter()
        .map(|t| (t.token, t.line))
        .collect()
}

/// Lexical tokens and byte spans. Braces inside literals and nested comments are not code.
pub fn rust_spans(src: &str) -> Vec<SpannedToken> {
    let chars: Vec<char> = src.chars().collect();
    let offsets: Vec<usize> = src
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(src.len()))
        .collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let at = |i: usize| chars.get(i).copied();
    let offset = |i: usize| offsets.get(i).copied().unwrap_or(src.len());
    while let Some(c) = at(i) {
        let start = i;
        let token_line = line;
        let next = at(i.saturating_add(1));
        if c.is_whitespace() {
            if c == '\n' {
                line = line.saturating_add(1);
            }
            i = i.saturating_add(1);
            continue;
        }
        if c == '/' && next == Some('/') {
            while at(i).is_some_and(|c| c != '\n') {
                i = i.saturating_add(1);
            }
            continue;
        }
        if c == '/' && next == Some('*') {
            i = i.saturating_add(2);
            let mut depth = 1usize;
            while let Some(c) = at(i) {
                if c == '/' && at(i.saturating_add(1)) == Some('*') {
                    depth = depth.saturating_add(1);
                    i = i.saturating_add(2);
                } else if c == '*' && at(i.saturating_add(1)) == Some('/') {
                    depth = depth.saturating_sub(1);
                    i = i.saturating_add(2);
                    if depth == 0 {
                        break;
                    }
                } else {
                    if c == '\n' {
                        line = line.saturating_add(1);
                    }
                    i = i.saturating_add(1);
                }
            }
            continue;
        }
        let token = if let Some(end) = string_literal_end(&chars, i) {
            line = line.saturating_add(
                chars
                    .get(i..end)
                    .unwrap_or_default()
                    .iter()
                    .filter(|c| **c == '\n')
                    .count(),
            );
            i = end;
            Tok::Str
        } else if let Some(end) = char_literal_end(&chars, i) {
            i = end;
            continue;
        } else if c.is_alphabetic() || c == '_' || c.is_ascii_digit() {
            i = i.saturating_add(1);
            while at(i).is_some_and(|c| c.is_alphanumeric() || c == '_') {
                i = i.saturating_add(1);
            }
            let text = src
                .get(offset(start)..offset(i))
                .unwrap_or_default()
                .to_owned();
            if c.is_ascii_digit() {
                Tok::Num(text)
            } else {
                Tok::Ident(text)
            }
        } else {
            i = i.saturating_add(1);
            Tok::Punct(c)
        };
        out.push(SpannedToken {
            token,
            line: token_line,
            start: offset(start),
            end: offset(i),
        });
    }
    out
}

/// The end of an ordinary or raw string, with its byte/C prefix when present.
fn string_literal_end(chars: &[char], start: usize) -> Option<usize> {
    let at = |i: usize| chars.get(i).copied();
    let c = at(start)?;
    let next = at(start.saturating_add(1));
    let raw_start = if matches!(c, 'b' | 'c') && next == Some('r') {
        start.saturating_add(1)
    } else {
        start
    };
    let mut quote = raw_start.saturating_add(1);
    if at(raw_start) == Some('r') {
        while at(quote) == Some('#') {
            quote = quote.saturating_add(1);
        }
    }
    let raw = at(raw_start) == Some('r') && at(quote) == Some('"');
    let ordinary_quote = if matches!(c, 'b' | 'c') && next == Some('"') {
        start.saturating_add(1)
    } else {
        start
    };
    if raw {
        let hashes = quote.saturating_sub(raw_start.saturating_add(1));
        let mut end = quote.saturating_add(1);
        while let Some(c) = at(end) {
            if c == '"' && (1..=hashes).all(|n| at(end.saturating_add(n)) == Some('#')) {
                return Some(end.saturating_add(hashes.saturating_add(1)));
            }
            end = end.saturating_add(1);
        }
        Some(chars.len())
    } else if at(ordinary_quote) == Some('"') {
        let mut end = ordinary_quote.saturating_add(1);
        while let Some(c) = at(end) {
            end = end.saturating_add(1);
            if c == '\\' {
                end = end.saturating_add(1);
            } else if c == '"' {
                return Some(end);
            }
        }
        Some(chars.len())
    } else {
        None
    }
}

/// A character literal consumes one scalar or one escape; a lifetime is not a literal.
fn char_literal_end(chars: &[char], start: usize) -> Option<usize> {
    let quote = if chars.get(start) == Some(&'b') {
        start.checked_add(1)?
    } else {
        start
    };
    if chars.get(quote) != Some(&'\'') {
        return None;
    }
    let value = quote.checked_add(1)?;
    let end = if chars.get(value) == Some(&'\\') {
        let escape = value.checked_add(1)?;
        match chars.get(escape)? {
            'x' => escape.checked_add(3)?,
            'u' if chars.get(escape.checked_add(1)?) == Some(&'{') => {
                let close = chars
                    .get(escape.checked_add(2)?..)?
                    .iter()
                    .position(|c| *c == '}')?;
                escape.checked_add(close)?.checked_add(3)?
            }
            _ => escape.checked_add(1)?,
        }
    } else {
        value.checked_add(1)?
    };
    (chars.get(end) == Some(&'\'')).then(|| end.saturating_add(1))
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

#[cfg(test)]
mod tests {
    use super::{Tok, rust_spans, rust_tokens};

    #[test]
    fn nested_comments_and_raw_literals_do_not_emit_code_tokens() {
        let src = "/* outer /* nested */ fake } */\nr###\"text \"## }\nfn fake() {}\"###\nvisible";
        assert_eq!(
            rust_tokens(src),
            [(Tok::Str, 2), (Tok::Ident("visible".to_owned()), 4)]
        );
    }

    #[test]
    fn spans_slice_original_unicode_source_and_track_lines() {
        let src = "/* é */\npub fn café() { let x = \"escaped\\\" }\"; }";
        let tokens = rust_spans(src);
        for token in &tokens {
            let original = src.get(token.start..token.end).unwrap();
            match &token.token {
                Tok::Ident(name) | Tok::Num(name) => assert_eq!(original, name),
                Tok::Punct(c) => assert_eq!(original, c.to_string()),
                Tok::Str => assert!(original.starts_with('"')),
            }
            assert_eq!(token.line, 2);
        }
        assert_eq!(
            tokens
                .iter()
                .filter(|t| matches!(t.token, Tok::Punct('{' | '}')))
                .count(),
            2
        );
    }
}
