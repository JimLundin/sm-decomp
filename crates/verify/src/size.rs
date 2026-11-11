//! Code volume in logical tokens, for `tools size PATH...` (MODEL §4, "It's smaller").
//!
//! Lines reward cramming and punish types and tables laid out one field to a line; tokens count
//! what the code says. A logical token is an identifier or keyword, a literal, a lifetime, an
//! operator (`::`, `->` and `..=` are one each) or a bracket pair. Comments, doc comments,
//! whitespace and `#[cfg(test)]` items are not counted, nor is a `tests` directory or a `tests.rs`
//! file.
//!
//! The values of `const` and `static` items are reported apart, as data: the game's tables and
//! parameters, which the old engine read from the ROM and the new one states (Jim, 2026-11-07).
//! A value is data only while it is built from literals, paths, tuples, arrays, struct literals
//! and calls whose arguments are data; anything else (control flow, a closure, a method call,
//! an operator) holds behaviour and is logic all the same. An exhaustive lookup is data too
//! (Jim, 2026-11-11): the arms of a guard-free `match` from variants, constants or literals to
//! data values, and the rows of `enum_map!`, are tables whose members the compiler checks.
//!
//! The size rule is held on one weighted score of the logic (Jim, 2026-11-07), so that a lower
//! score means better code. Forms that hold no behaviour weigh nothing: `use` items, `mod`
//! declarations, visibility (`pub`, `pub(crate)`), an `impl` header, a trailing comma before a
//! closing bracket and the forced `const` of a `const fn`; a constant's head (`const NAME: T =`
//! and its `;`) is counted with the types. A function signature weighs half, so a named step
//! costs less than its body inlined twice. `true` and `false` weigh double: flag-style code that
//! a typed state replaces. Every other logic token weighs one.
//!
//! Two quality terms compose with the weights (`quality` states them). Depth (Q1, Jim,
//! 2026-11-07): a token `d` blocks deep weighs `1 + 0.25 × d` times its weight, so nested
//! decisions cost more than a flat dispatch. Duplication (Jim, 2026-11-11): a run of logic in a
//! function body that repeats one seen before, local names aside, pays its tokens again, so a
//! decision written twice costs more than one shared step. Clumps (Q2), values that travel
//! together without a type, are reported beside the score as a diagnostic and weigh nothing in
//! it.

mod quality;

use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use quality::Context;

/// The command line `run` takes.
pub const USAGE: &str = "tools size PATH...";

/// Operators of more than one character, longest first so the scan takes the longest match.
const OPERATORS: &[&str] = &[
    "<<=", ">>=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=",
    "*=", "/=", "%=", "^=", "&=", "|=", "<<", ">>", "..",
];

/// A lexed token. A closing bracket is kept so the test filter can track depth, but a pair counts
/// once. A literal keeps its text, so clumps (Q2) can tell argument expressions apart.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Token<'a> {
    Open(char),
    Close,
    Word(&'a str),
    Punct(&'a str),
    Literal(&'a str),
}

/// What a piece of source starts with, and the text after it.
enum Lexeme<'a> {
    Skip,
    Token(Token<'a>),
}

fn is_word_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// The text after the longest prefix of `rest` whose characters satisfy `keep`.
fn after_while(rest: &str, keep: impl Fn(char) -> bool) -> &str {
    rest.find(|c| !keep(c))
        .and_then(|at| rest.get(at..))
        .unwrap_or_default()
}

/// The source's tokens, in order, each with the byte range of its text.
fn lex(src: &str) -> Vec<(Token<'_>, Range<usize>)> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(c) = rest.chars().next() {
        let (lexeme, after) = next(rest, c);
        if let Lexeme::Token(token) = lexeme {
            let start = src.len().saturating_sub(rest.len());
            out.push((token, start..src.len().saturating_sub(after.len())));
        }
        rest = after;
    }
    out
}

/// The lexeme at the start of `rest`, whose first character is `c`.
fn next(rest: &str, c: char) -> (Lexeme<'_>, &str) {
    let one = rest.get(c.len_utf8()..).unwrap_or_default();
    if c.is_whitespace() {
        (Lexeme::Skip, one)
    } else if rest.starts_with("//") {
        (
            Lexeme::Skip,
            rest.find('\n')
                .and_then(|at| rest.get(at..))
                .unwrap_or_default(),
        )
    } else if rest.starts_with("/*") {
        (Lexeme::Skip, after_block_comment(rest))
    } else if let Some(after) = after_string(rest) {
        (Lexeme::Token(Token::Literal(consumed(rest, after))), after)
    } else if c == '\'' {
        let after = after_quote(rest);
        (Lexeme::Token(Token::Literal(consumed(rest, after))), after)
    } else if c.is_ascii_digit() {
        let after = after_number(rest);
        (Lexeme::Token(Token::Literal(consumed(rest, after))), after)
    } else if is_word_char(c) {
        let body = rest.strip_prefix("r#").unwrap_or(rest);
        let after = after_while(body, is_word_char);
        (Lexeme::Token(Token::Word(consumed(rest, after))), after)
    } else if let Some(open) = ['(', '[', '{'].into_iter().find(|&b| b == c) {
        (Lexeme::Token(Token::Open(open)), one)
    } else if [')', ']', '}'].contains(&c) {
        (Lexeme::Token(Token::Close), one)
    } else {
        let after = OPERATORS
            .iter()
            .find_map(|op| rest.strip_prefix(op))
            .unwrap_or(one);
        (Lexeme::Token(Token::Punct(consumed(rest, after))), after)
    }
}

/// The part of `rest` before its suffix `after`.
fn consumed<'a>(rest: &'a str, after: &str) -> &'a str {
    rest.get(..rest.len().saturating_sub(after.len()))
        .unwrap_or_default()
}

/// The text after a block comment; they nest.
fn after_block_comment(rest: &str) -> &str {
    let mut depth = 0usize;
    let mut here = rest;
    while let Some(c) = here.chars().next() {
        if let Some(inner) = here.strip_prefix("/*") {
            depth = depth.saturating_add(1);
            here = inner;
        } else if let Some(outer) = here.strip_prefix("*/") {
            depth = depth.saturating_sub(1);
            here = outer;
            if depth == 0 {
                return here;
            }
        } else {
            here = here.get(c.len_utf8()..).unwrap_or_default();
        }
    }
    here
}

/// The text after a string literal (prefixes `b`, `c`, `r`, `br`, `cr` and raw hashes included)
/// or a byte character `b'x'`, if `rest` starts with one.
fn after_string(rest: &str) -> Option<&str> {
    if let Some(byte_char) = rest.strip_prefix('b')
        && byte_char.starts_with('\'')
    {
        return Some(after_quote(byte_char));
    }
    for raw in ["br", "cr", "r"] {
        let Some(hashed) = rest.strip_prefix(raw) else {
            continue;
        };
        let body = hashed.trim_start_matches('#');
        let Some(text) = body.strip_prefix('"') else {
            continue;
        };
        let close = format!("\"{}", "#".repeat(hashed.len().saturating_sub(body.len())));
        let end = text.find(&close).unwrap_or(text.len());
        return Some(
            text.get(end..)
                .unwrap_or_default()
                .get(close.len()..)
                .unwrap_or_default(),
        );
    }
    let text = ["b", "c", ""]
        .into_iter()
        .find_map(|p| rest.strip_prefix(p).and_then(|s| s.strip_prefix('"')))?;
    let mut escaped = false;
    for (at, ch) in text.char_indices() {
        match (escaped, ch) {
            (true, _) => escaped = false,
            (false, '\\') => escaped = true,
            (false, '"') => return text.get(at..).and_then(|s| s.strip_prefix('"')),
            (false, _) => {}
        }
    }
    Some("")
}

/// The text after a character literal or a lifetime; `rest` starts with `'`.
fn after_quote(rest: &str) -> &str {
    let body = rest.strip_prefix('\'').unwrap_or(rest);
    if let Some(escape) = body.strip_prefix('\\') {
        // The escaped character itself may be a quote: `'\''`.
        let mut chars = escape.chars();
        let escape = chars.next().map_or(escape, |_| chars.as_str());
        let end = escape.find('\'').unwrap_or(escape.len());
        return escape
            .get(end..)
            .unwrap_or_default()
            .strip_prefix('\'')
            .unwrap_or_default();
    }
    let mut chars = body.chars();
    let after_one = chars.next().map_or(body, |_| chars.as_str());
    after_one
        .strip_prefix('\'')
        .unwrap_or_else(|| after_while(body, is_word_char))
}

/// The text after a number: digits, `_`, a radix prefix, a suffix, a fraction or a signed
/// exponent, but not the `..` of a range or a method call's dot.
fn after_number(rest: &str) -> &str {
    let hex = rest.starts_with("0x");
    let mut here = rest;
    let mut previous = '\0';
    while let Some(c) = here.chars().next() {
        let tail = here.get(c.len_utf8()..).unwrap_or_default();
        let signed_exponent = matches!(c, '+' | '-') && matches!(previous, 'e' | 'E') && !hex;
        let fraction = c == '.' && tail.starts_with(|d: char| d.is_ascii_digit());
        if !(c.is_ascii_alphanumeric() || c == '_' || signed_exponent || fraction) {
            break;
        }
        previous = c;
        here = tail;
    }
    here
}

/// The tokens that make up `#[cfg(test)]`.
const TEST_ATTRIBUTE: [Token<'static>; 7] = [
    Token::Punct("#"),
    Token::Open('['),
    Token::Word("cfg"),
    Token::Open('('),
    Token::Word("test"),
    Token::Close,
    Token::Close,
];

/// What one logic token weighs in the score (Jim, 2026-11-07), before depth (Q1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Weight {
    /// A form that holds no behaviour, so trading it costs nothing: a `use` item, a `mod`
    /// declaration, visibility (`pub`, `pub(crate)`: widening or narrowing it changes no
    /// behaviour), an `impl` header up to its body's `{` (no behaviour can live there), a
    /// trailing comma before a closing bracket (rustfmt's), the `const` of a `const fn`
    /// (`missing_const_for_fn` forces it).
    Free,
    /// A function signature, from `fn` to its body's `{`: a named step's price.
    Half,
    /// Every other logic token.
    Single,
    /// `true` or `false` in logic, so a typed state beats a flag.
    Double,
}

impl Weight {
    /// The weight in half tokens.
    const fn halves(self) -> usize {
        match self {
            Self::Free => 0,
            Self::Half => 1,
            Self::Single => 2,
            Self::Double => 4,
        }
    }
}

/// A weighted score, the number the size rule holds, kept in eighths of a token: a signature's
/// half at a depth's quarter steps stays whole.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Score(usize);

impl Score {
    /// The score in eighths of a token.
    #[must_use]
    pub const fn eighths(self) -> usize {
        self.0
    }

    const fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// `halves` half tokens, each `quarters` quarters of its weight.
    const fn of(halves: usize, quarters: usize) -> Self {
        Self(halves.saturating_mul(quarters))
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / 8;
        let thousandths = (self.0 % 8).saturating_mul(125);
        let text = if thousandths == 0 {
            whole.to_string()
        } else {
            let fraction = format!("{thousandths:03}");
            format!("{whole}.{}", fraction.trim_end_matches('0'))
        };
        f.pad(&text)
    }
}

/// A source's size, split between logic, declarations and data, and its logic's score.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Size {
    /// The weighted logic, which the size rule holds, its quality terms included.
    pub score: Score,
    /// The part of the score that depth adds (Q1).
    pub depth: Score,
    /// The part of the score that duplication adds: each repeated token once more, at its depth.
    pub duplication: Score,
    /// What clumps (Q2) would add: a diagnostic, not part of the score.
    pub clumps: Score,
    /// Every counted token that is neither a declaration nor a constant's or a static's value.
    pub logic: usize,
    /// The tokens of item-level `struct`, `enum`, `union` and `type` items, of attributes and of
    /// constants' heads (`const NAME: T =` and the `;`): what the code's types say, reported
    /// apart so typing is visible on its own.
    pub types: usize,
    /// The tokens of `const` and `static` values: tables and parameters, the game's data, which
    /// the old engine read from the ROM instead (Jim, 2026-11-07).
    pub data: usize,
}

impl Size {
    const fn add(self, other: Self) -> Self {
        Self {
            score: self.score.add(other.score),
            depth: self.depth.add(other.depth),
            duplication: self.duplication.add(other.duplication),
            clumps: self.clumps.add(other.clumps),
            logic: self.logic.saturating_add(other.logic),
            types: self.types.saturating_add(other.types),
            data: self.data.saturating_add(other.data),
        }
    }

    /// One more token of `class`, in `context`.
    fn with(self, class: Class, context: Context) -> Self {
        match class {
            Class::Logic(weight) => {
                let quarters = context.depth.saturating_add(4);
                // A parameter in a clump (Q2) would lose the signature's half.
                let clump = if context.clump && weight == Weight::Half {
                    Score::of(
                        Weight::Single
                            .halves()
                            .saturating_sub(Weight::Half.halves()),
                        quarters,
                    )
                } else {
                    Score::default()
                };
                // A repeated token is paid again at one token, at its depth.
                let again = if context.repeated {
                    Score::of(Weight::Single.halves(), quarters)
                } else {
                    Score::default()
                };
                Self {
                    score: self
                        .score
                        .add(Score::of(weight.halves(), quarters))
                        .add(again),
                    depth: self.depth.add(Score::of(weight.halves(), context.depth)),
                    duplication: self.duplication.add(again),
                    clumps: self.clumps.add(clump),
                    logic: self.logic.saturating_add(1),
                    ..self
                }
            }
            Class::Type => Self {
                types: self.types.saturating_add(1),
                ..self
            },
            Class::Data => Self {
                data: self.data.saturating_add(1),
                ..self
            },
            Class::Uncounted => self,
        }
    }

    /// The size with `tokens` more that a repeated argument pair (Q2) would pay, at one each.
    const fn repeated(self, tokens: usize) -> Self {
        let again = Score::of(Weight::Single.halves().saturating_mul(tokens), 4);
        Self {
            clumps: self.clumps.add(again),
            ..self
        }
    }
}

/// How one token is reported.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    /// Logic, of a weight.
    Logic(Weight),
    /// A declaration, an attribute or a constant's head.
    Type,
    /// A constant's or a static's value that holds no behaviour.
    Data,
    /// A closing bracket: its pair is counted at the opening one.
    Uncounted,
}

/// The index just past the bracket group that opens at `open`, or the end.
fn group_end(tokens: &[Token<'_>], open: usize) -> usize {
    let rest = tokens.get(open..).unwrap_or_default();
    tokens.len().saturating_sub(after_group(rest).len())
}

/// The tokens after the bracket group that `tokens` starts with.
const fn after_group<'a, 'b>(tokens: &'a [Token<'b>]) -> &'a [Token<'b>] {
    let mut depth = 0usize;
    let mut rest = tokens;
    while let Some((&token, tail)) = rest.split_first() {
        rest = tail;
        match token {
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return rest;
                }
            }
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    rest
}

/// The tokens after an attribute, `#[...]` or `#![...]`, if `tokens` starts with one.
fn after_attribute<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let body = tokens.strip_prefix(&[Token::Punct("#")][..])?;
    let body = body.strip_prefix(&[Token::Punct("!")][..]).unwrap_or(body);
    matches!(body.first(), Some(Token::Open('['))).then(|| after_group(body))
}

/// The part of `tokens` before its suffix `after`.
fn before<'a, 'b>(tokens: &'a [Token<'b>], after: &[Token<'b>]) -> &'a [Token<'b>] {
    tokens
        .get(..tokens.len().saturating_sub(after.len()))
        .unwrap_or_default()
}

/// Whether a token can end the item before it, so a `const` or `static` after it starts an item
/// rather than a generic parameter (`<const N: usize>`).
fn ends_item(previous: Option<Token<'_>>) -> bool {
    matches!(
        previous,
        None | Some(
            Token::Punct(";") | Token::Open('{') | Token::Close | Token::Word("pub" | "mut")
        )
    )
}

/// The tokens without their `#[cfg(test)]` items, each with its text's byte range.
fn without_tests<'a>(lexed: &[(Token<'a>, Range<usize>)]) -> Vec<(Token<'a>, Range<usize>)> {
    let tokens: Vec<Token<'a>> = lexed.iter().map(|(token, _)| *token).collect();
    let mut out = Vec::with_capacity(tokens.len());
    let mut rest = tokens.as_slice();
    while let Some((_, tail)) = rest.split_first() {
        if let Some(item) = rest.strip_prefix(&TEST_ATTRIBUTE[..]) {
            rest = after_item(item);
            continue;
        }
        out.extend(lexed.get(tokens.len().saturating_sub(rest.len())).cloned());
        rest = tail;
    }
    out
}

/// One class for each token: the values of item-level constants and statics as data unless
/// they hold behaviour, declarations as types, each logic token weighed and each closing bracket
/// uncounted. Lookups are left as logic; [`lookups`] turns their arms into data.
fn classify(tokens: &[Token<'_>], out: &mut Vec<Class>) {
    let mut rest = tokens;
    let mut previous = None;
    // How many tokens are left once the last signature reaches its body or its `;`.
    let mut signature_end = None;
    // How many tokens are left once the last visibility or `impl` header ends.
    let mut free_end = None;
    while let Some((&token, tail)) = rest.split_first() {
        if free_end.is_none_or(|end| rest.len() <= end) {
            free_end = free_span(token, tail, previous);
        }
        if let Some(after) = free_item(rest, previous, out) {
            previous = before(rest, after).last().copied();
            rest = after;
            continue;
        }
        let declaration = matches!(token, Token::Word("struct" | "enum" | "union" | "type"))
            && ends_item(previous);
        let after = if declaration {
            Some(after_item(rest))
        } else {
            after_attribute(rest)
        };
        if let Some(after) = after {
            let item = before(rest, after);
            mark(out, item, Class::Type);
            previous = item.last().copied();
            rest = after;
            continue;
        }
        if token == Token::Word("fn") && matches!(tail.first(), Some(Token::Word(_))) {
            signature_end = Some(signature_body(tail).len());
        }
        out.push(if token == Token::Close {
            Class::Uncounted
        } else if free_end.is_some_and(|end| rest.len() > end) {
            Class::Logic(Weight::Free)
        } else {
            let in_signature = signature_end.is_some_and(|end| rest.len() > end);
            Class::Logic(weight(token, tail.first(), in_signature))
        });
        previous = Some(token);
        rest = tail;
    }
}

/// How many tokens are left after the form that holds no behaviour starting at `token`, whose
/// item `previous` ended, if one starts there: a visibility (`pub`, `pub(crate)`) or an `impl`
/// header through its body's `{`. `tail` follows `token`.
fn free_span(token: Token<'_>, tail: &[Token<'_>], previous: Option<Token<'_>>) -> Option<usize> {
    match token {
        Token::Word("pub") => Some(match tail.first() {
            Some(Token::Open('(')) => after_group(tail).len(),
            _ => tail.len(),
        }),
        // An `impl` in a type (`-> impl Iterator`, `&mut impl World`) stands after no item.
        Token::Word("impl") if ends_item(previous) && previous != Some(Token::Word("mut")) => Some(
            signature_body(tail)
                .split_first()
                .map_or(0, |(_, body)| body.len()),
        ),
        Token::Open(_) | Token::Close | Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {
            None
        }
    }
}

/// One `class` for each of `tokens`, closing brackets uncounted.
fn mark(out: &mut Vec<Class>, tokens: &[Token<'_>], class: Class) {
    out.extend(tokens.iter().map(|&token| {
        if token == Token::Close {
            Class::Uncounted
        } else {
            class
        }
    }));
}

/// The classes of the constant, `use` item or `mod` declaration at the start of `tokens`, whose
/// item `previous` ended, pushed onto `out`, and the tokens after it.
fn free_item<'a, 'b>(
    tokens: &'a [Token<'b>],
    previous: Option<Token<'b>>,
    out: &mut Vec<Class>,
) -> Option<&'a [Token<'b>]> {
    if !ends_item(previous) {
        return None;
    }
    let (&token, tail) = tokens.split_first()?;
    match token {
        Token::Word("const" | "static")
            if !matches!(tail.first(), Some(Token::Word("fn" | "unsafe" | "extern"))) =>
        {
            let (head, value, after) = split_value(tokens)?;
            mark(out, head, Class::Type);
            if head.last() == Some(&Token::Punct("=")) {
                if holds_behaviour(value) {
                    classify(value, out);
                } else {
                    mark(out, value, Class::Data);
                }
                // Its `;`.
                out.push(Class::Type);
            }
            Some(after)
        }
        Token::Word("use") => {
            let after = after_semicolon(tokens);
            mark(out, before(tokens, after), Class::Logic(Weight::Free));
            Some(after)
        }
        Token::Word("mod") => match tail {
            [
                Token::Word(_),
                Token::Punct(";") | Token::Open('{'),
                after @ ..,
            ] => {
                mark(out, before(tokens, after), Class::Logic(Weight::Free));
                Some(after)
            }
            _ => None,
        },
        Token::Open(_) | Token::Close | Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {
            None
        }
    }
}

/// The tokens after the depth-0 `;` that ends the item at the start of `tokens`, past any
/// braces it holds (`use a::{b, c};`).
fn after_semicolon<'a, 'b>(tokens: &'a [Token<'b>]) -> &'a [Token<'b>] {
    let mut depth = 0usize;
    let mut rest = tokens;
    while let Some((&token, tail)) = rest.split_first() {
        rest = tail;
        match token {
            Token::Punct(";") if depth == 0 => return rest,
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => depth = depth.saturating_sub(1),
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    rest
}

/// What a logic token weighs, given the token after it and whether it is in a signature.
fn weight(token: Token<'_>, next: Option<&Token<'_>>, in_signature: bool) -> Weight {
    let free = matches!(
        (token, next),
        (Token::Punct(","), Some(Token::Close)) | (Token::Word("const"), Some(Token::Word("fn")))
    );
    if free {
        Weight::Free
    } else if in_signature {
        Weight::Half
    } else if matches!(token, Token::Word("true" | "false")) {
        Weight::Double
    } else {
        Weight::Single
    }
}

/// The tokens from a signature's body `{` or its `;` on; `tokens` follows its `fn`. Brackets
/// inside the signature (parameters, generics, a `where` clause's bounds) are passed over.
fn signature_body<'a, 'b>(tokens: &'a [Token<'b>]) -> &'a [Token<'b>] {
    let mut depth = 0usize;
    let mut rest = tokens;
    while let Some((&token, tail)) = rest.split_first() {
        match token {
            Token::Open('{') | Token::Punct(";") if depth == 0 => return rest,
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => depth = depth.saturating_sub(1),
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
        rest = tail;
    }
    rest
}

/// Keywords, which never name a constant, a variant or a function in a value that is data.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "dyn", "else", "enum", "extern", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "static", "struct", "trait", "type", "union", "unsafe", "use", "where", "while",
    "yield",
];

/// Whether a constant's value holds behaviour, so it counts as logic (review L3): anything but
/// one data expression. Data is a literal, a minus on a literal, a reference to data, a path (a
/// constant, an enum variant, a unit struct), a tuple, an array (`[a, b]` or `[a; N]`), a struct
/// literal whose fields are data, a call of a path whose every argument is data (the
/// constructors of a typed table, `Picture::new(0x009)`, `Some(Fixed(3))`), or a range whose two
/// ends are literals or paths (`0x280..0x500`, `A..=B`). Control flow, a closure, a macro, a
/// method call on a value (`.expect(..)`), a field or an index, a turbofish and every other
/// operator are logic, wherever they stand.
fn holds_behaviour(value: &[Token<'_>]) -> bool {
    after_data(value) != Some(&[])
}

/// Turns the arms of each exhaustive lookup among `tokens` into data (Jim, 2026-11-11): a table
/// the compiler checks for a missing or repeated member is data as a constant's rows are. A
/// lookup is a `match` that [`lookup_arms`] accepts or an `enum_map!` that [`lookup_rows`]
/// accepts. The `match`, its scrutinee and its braces, and the macro's name and brackets, stay
/// logic. `code_check.sh`'s L2 (`lint_scan.awk`) exempts the same values from the
/// magic-literal rule, but not a pattern's literals: a key that is a number is still named.
fn lookups(tokens: &[Token<'_>], classes: &mut [Class]) {
    for (at, pair) in tokens.windows(2).enumerate() {
        if !matches!(classes.get(at), Some(Class::Logic(_))) {
            continue;
        }
        let arms = match pair {
            [Token::Word("match"), _] => lookup_arms(tokens, at),
            [Token::Word("enum_map"), Token::Punct("!")] => lookup_rows(tokens, at),
            _ => None,
        };
        for class in arms
            .and_then(|arms| classes.get_mut(arms))
            .into_iter()
            .flatten()
        {
            if matches!(class, Class::Logic(_)) {
                *class = Class::Data;
            }
        }
    }
}

/// The arms of the `match` at `at`, if it is a lookup: two or more arms, each a
/// [`lookup_pattern`] with no guard, then `=>` and a [`lookup_value`] that is no block. A guard,
/// a binding or a block arm keeps a decision in the arm, so the whole `match` stays logic.
fn lookup_arms(tokens: &[Token<'_>], at: usize) -> Option<Range<usize>> {
    let scrutinee = tokens.get(at.saturating_add(1)..)?;
    let open = tokens.len().saturating_sub(scrutinee.len()).saturating_add(
        top_level(scrutinee)
            .find(|&(_, t)| t == Token::Open('{'))?
            .0,
    );
    let arms = open.saturating_add(1)..group_end(tokens, open).saturating_sub(1);
    let rows = rows_of(tokens.get(arms.clone())?)?;
    (rows >= 2).then_some(arms)
}

/// The rows of the `enum_map!` at `at`, if each is a [`lookup_pattern`], `=>` and a
/// [`lookup_value`].
fn lookup_rows(tokens: &[Token<'_>], at: usize) -> Option<Range<usize>> {
    let open = at.saturating_add(2);
    if !matches!(tokens.get(open), Some(Token::Open(_))) {
        return None;
    }
    let rows = open.saturating_add(1)..group_end(tokens, open).saturating_sub(1);
    rows_of(tokens.get(rows.clone())?)?;
    Some(rows)
}

/// How many lookup rows `tokens` holds, separated by commas, if every one is a
/// [`lookup_pattern`], `=>` and a [`lookup_value`].
fn rows_of(tokens: &[Token<'_>]) -> Option<usize> {
    let mut rows = 0usize;
    for row in split_top(tokens, Token::Punct(",")) {
        if row.is_empty() {
            continue;
        }
        let arrow = top_level(row).find(|&(_, t)| t == Token::Punct("=>"))?.0;
        let (pattern, value) = (row.get(..arrow)?, row.get(arrow.saturating_add(1)..)?);
        if !(lookup_pattern(pattern) && lookup_value(value)) {
            return None;
        }
        rows = rows.saturating_add(1);
    }
    Some(rows)
}

/// Whether an arm's pattern is a lookup's key: `_`, or alternatives (`A | B`) each a path to a
/// variant or a constant (its last part capitalised), a literal (a minus on one included) or a
/// range of two literals. A binding, a guard's `if` and a destructuring pattern are not keys.
fn lookup_pattern(pattern: &[Token<'_>]) -> bool {
    pattern == [Token::Word("_")]
        || split_top(pattern, Token::Punct("|"))
            .into_iter()
            .all(|key| match after_literal(key) {
                Some([]) => true,
                Some([Token::Punct(".." | "..="), end @ ..]) => after_literal(end) == Some(&[]),
                Some(_) => false,
                None => {
                    after_path(key) == Some(&[])
                        && matches!(key.last(), Some(Token::Word(name)) if name.starts_with(char::is_uppercase))
                }
            })
}

/// Whether an arm's value is a lookup's: data by the constant rule ([`holds_behaviour`]) that
/// names no local. A local, a field and `self` read the moment's state, so an arm holding one
/// is a decision.
fn lookup_value(value: &[Token<'_>]) -> bool {
    !value.is_empty() && !holds_behaviour(value) && !names_local(value)
}

/// Whether a value names a local: a lowercase word (`self` included) that is no part of a path,
/// no function it calls, no field's label in a struct literal, and not `true` or `false`.
fn names_local(value: &[Token<'_>]) -> bool {
    value.iter().enumerate().any(|(at, &token)| {
        let Token::Word(word) = token else {
            return false;
        };
        let previous = at.checked_sub(1).and_then(|p| value.get(p));
        let next = value.get(at.saturating_add(1));
        word.starts_with(|c: char| c.is_lowercase() || c == '_')
            && !matches!(word, "true" | "false")
            && previous != Some(&Token::Punct("::"))
            && !matches!(next, Some(Token::Open('(') | Token::Punct("::" | ":")))
    })
}

/// The tokens after a literal, or a minus on one, at the start of `tokens`.
fn after_literal<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    match tokens {
        [Token::Literal(_), rest @ ..] | [Token::Punct("-"), Token::Literal(_), rest @ ..] => {
            Some(rest)
        }
        _ => None,
    }
}

/// The tokens of `tokens` outside every bracket group, each with its index.
fn top_level<'a, 'b>(tokens: &'a [Token<'b>]) -> impl Iterator<Item = (usize, Token<'b>)> + 'a {
    let mut depth = 0usize;
    tokens.iter().enumerate().filter_map(move |(at, &token)| {
        let outside = depth == 0;
        match token {
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => depth = depth.saturating_sub(1),
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
        (outside && token != Token::Close).then_some((at, token))
    })
}

/// `tokens` split at each `separator` outside every bracket group.
fn split_top<'a, 'b>(tokens: &'a [Token<'b>], separator: Token<'b>) -> Vec<&'a [Token<'b>]> {
    let mut parts = Vec::new();
    let mut start = 0;
    for (at, token) in top_level(tokens) {
        if token == separator {
            parts.extend(tokens.get(start..at));
            start = at.saturating_add(1);
        }
    }
    parts.extend(tokens.get(start..));
    parts
}

/// The tokens after the data expression at the start of `tokens`, if one starts there.
fn after_data<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    if let Some(range) = after_range(tokens) {
        return Some(range);
    }
    let (&token, tail) = tokens.split_first()?;
    match token {
        Token::Literal(_) => Some(tail),
        Token::Punct("-") => match tail.split_first()? {
            (Token::Literal(_), after) => Some(after),
            _ => None,
        },
        Token::Punct("&") => after_data(tail),
        Token::Open('(') => after_list(tail),
        Token::Open('[') => after_array(tail),
        Token::Word(_) | Token::Punct("::") => {
            let rest = after_path(tokens)?;
            match rest.split_first() {
                Some((Token::Open('('), arguments)) => after_list(arguments),
                Some((Token::Open('{'), fields)) => after_fields(fields),
                _ => Some(rest),
            }
        }
        Token::Open(_) | Token::Close | Token::Punct(_) => None,
    }
}

/// The tokens after a range of two ends, `a..b` or `a..=b`, at the start of `tokens`.
fn after_range<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let (dots, end) = after_range_end(tokens)?.split_first()?;
    if !matches!(dots, Token::Punct(".." | "..=")) {
        return None;
    }
    after_range_end(end)
}

/// The tokens after a range's end, a literal (a minus on one included) or a path that neither
/// calls nor builds anything.
fn after_range_end<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let rest = match tokens.split_first()? {
        (Token::Literal(_), tail) => tail,
        (Token::Punct("-"), tail) => match tail.split_first()? {
            (Token::Literal(_), after) => after,
            _ => return None,
        },
        (Token::Word(_) | Token::Punct("::"), _) => after_path(tokens)?,
        _ => return None,
    };
    match rest.first() {
        Some(Token::Open('(' | '{')) => None,
        _ => Some(rest),
    }
}

/// The tokens after a path, `A`, `Key::Jump` or `::m::A`, at the start of `tokens`.
fn after_path<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let mut rest = tokens.strip_prefix(&[Token::Punct("::")]).unwrap_or(tokens);
    loop {
        let (Token::Word(word), tail) = rest.split_first()? else {
            return None;
        };
        if KEYWORDS.contains(word) {
            return None;
        }
        match tail.strip_prefix(&[Token::Punct("::")]) {
            Some(next) => rest = next,
            None => return Some(tail),
        }
    }
}

/// The tokens after a comma-separated list of data and its closing bracket; `tokens` follows the
/// opening one.
fn after_list<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let mut rest = tokens;
    loop {
        if let Some(after) = rest.strip_prefix(&[Token::Close]) {
            return Some(after);
        }
        let item = after_data(rest)?;
        match item.split_first()? {
            (Token::Punct(","), next) => rest = next,
            (Token::Close, after) => return Some(after),
            _ => return None,
        }
    }
}

/// The tokens after an array, `[a, b]` or `[a; N]`; `tokens` follows its `[`.
fn after_array<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let repeated = after_data(tokens).and_then(|item| item.strip_prefix(&[Token::Punct(";")]));
    match repeated {
        Some(length) => after_data(length)?.strip_prefix(&[Token::Close]),
        None => after_list(tokens),
    }
}

/// The tokens after a struct literal's fields and its `}`; `tokens` follows its `{`. A field is
/// `name: data` or a shorthand `name`, and a base `..data` may come last.
fn after_fields<'a, 'b>(tokens: &'a [Token<'b>]) -> Option<&'a [Token<'b>]> {
    let mut rest = tokens;
    loop {
        let field = match rest.split_first()? {
            (Token::Close, after) => return Some(after),
            (Token::Punct(".."), base) => return after_data(base)?.strip_prefix(&[Token::Close]),
            (Token::Word(_) | Token::Literal(_), tail) => match tail.split_first()? {
                (Token::Punct(":"), value) => after_data(value)?,
                _ => tail,
            },
            _ => return None,
        };
        match field.split_first()? {
            (Token::Punct(","), next) => rest = next,
            (Token::Close, after) => return Some(after),
            _ => return None,
        }
    }
}

/// The counted tokens of a run: all but closing brackets.
fn counted(tokens: &[Token<'_>]) -> usize {
    tokens.iter().filter(|t| **t != Token::Close).count()
}

/// A constant item split into its head, its value and the tokens after its `;`. The head runs
/// from `const` to the `=`; a trait's `const X: T;` has no value, and its head is the whole item.
fn split_value<'a, 'b>(
    tokens: &'a [Token<'b>],
) -> Option<(&'a [Token<'b>], &'a [Token<'b>], &'a [Token<'b>])> {
    let mut open = 0usize;
    let mut equals = None;
    for (at, token) in tokens.iter().enumerate() {
        match token {
            Token::Open(_) => open = open.saturating_add(1),
            Token::Close => open = open.saturating_sub(1),
            Token::Punct("=") if open == 0 && equals.is_none() => equals = Some(at),
            Token::Punct(";") if open == 0 => {
                let after = tokens.get(at.checked_add(1)?..)?;
                let Some(eq) = equals else {
                    return Some((tokens.get(..=at)?, &[], after));
                };
                let value = tokens.get(eq.checked_add(1)?..at)?;
                return Some((tokens.get(..=eq)?, value, after));
            }
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    None
}

/// The tokens after the item at the start of `tokens`: past its `;`, or past the brace that
/// closes its body.
fn after_item<'a, 'b>(tokens: &'a [Token<'b>]) -> &'a [Token<'b>] {
    let mut open = Vec::new();
    let mut rest = tokens;
    while let Some((&token, tail)) = rest.split_first() {
        rest = tail;
        match token {
            Token::Open(bracket) => open.push(bracket),
            Token::Close => {
                if open.pop() == Some('{') && open.is_empty() {
                    return rest;
                }
            }
            Token::Punct(";") if open.is_empty() => return rest,
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    rest
}

/// The `.rs` files under `path`, skipping `tests` directories and `tests.rs` files, sorted, pushed
/// onto `out`.
///
/// # Errors
///
/// The path and the error when a directory can't be read.
pub fn sources(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().is_some_and(|e| e == "rs") {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = fs::read_dir(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for entry in entries {
        if entry
            .file_name()
            .is_some_and(|n| n == "tests" || n == "tests.rs")
        {
            continue;
        }
        sources(&entry, out)?;
    }
    Ok(())
}

/// One measured source: its tokens, each with its text's byte range and its share of the size.
/// A repeated argument pair's pay (Q2) is the share of the token that opens its call or literal.
struct Measured<'a> {
    tokens: Vec<Token<'a>>,
    spans: Vec<Range<usize>>,
    shares: Vec<Size>,
}

/// The sources measured together: a repeated run or a clump (Q2) recurs across them.
fn measured<'a>(srcs: &[&'a str]) -> Vec<Measured<'a>> {
    let (tokens, spans): (Vec<Vec<Token<'a>>>, Vec<Vec<Range<usize>>>) = srcs
        .iter()
        .map(|src| without_tests(&lex(src)).into_iter().unzip())
        .unzip();
    let classes: Vec<Vec<Class>> = tokens
        .iter()
        .map(|tokens| {
            let mut classes = Vec::with_capacity(tokens.len());
            classify(tokens, &mut classes);
            lookups(tokens, &mut classes);
            classes
        })
        .collect();
    // Duplication counts what holds behaviour at full weight: no signature, no free form, no data.
    let weighed: Vec<Vec<bool>> = classes
        .iter()
        .map(|classes| {
            classes
                .iter()
                .map(|class| matches!(class, Class::Logic(Weight::Single | Weight::Double)))
                .collect()
        })
        .collect();
    let repeats = quality::repeats(&tokens, &weighed);
    let quality = quality::of(&tokens);
    tokens
        .into_iter()
        .zip(spans)
        .zip(classes)
        .zip(quality.into_iter().zip(repeats))
        .map(
            |(((tokens, spans), classes), ((contexts, repeated), repeats))| {
                let mut shares: Vec<Size> = classes
                    .into_iter()
                    .zip(contexts)
                    .zip(repeats)
                    .map(|((class, context), repeats)| {
                        Size::default().with(
                            class,
                            Context {
                                repeated: repeats,
                                ..context
                            },
                        )
                    })
                    .collect();
                for pay in repeated {
                    if let Some(share) = shares.get_mut(pay.at) {
                        *share = share.repeated(pay.tokens);
                    }
                }
                Measured {
                    tokens,
                    spans,
                    shares,
                }
            },
        )
        .collect()
}

/// The sum of some tokens' shares.
fn total(shares: &[Size]) -> Size {
    shares
        .iter()
        .fold(Size::default(), |sum, &share| sum.add(share))
}

/// The size of each source text, measured together: a clump (Q2) recurs across them.
#[must_use]
pub fn sizes(srcs: &[&str]) -> Vec<Size> {
    measured(srcs)
        .iter()
        .map(|source| total(&source.shares))
        .collect()
}

/// A function of a source, with its size: from its `fn`, and the visibility and qualifiers
/// before it, to the end of its body.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Function<'a> {
    /// Its name.
    pub name: &'a str,
    /// Its size, measured with the rest of its source.
    pub size: Size,
    /// Its text, comments inside it included.
    pub text: &'a str,
}

/// Each source's outermost functions, measured together as [`sizes`] measures the sources.
///
/// A function inside another's body is part of that one. What lies in no function is the rest of
/// the source's size.
#[must_use]
pub fn functions<'a>(srcs: &[&'a str]) -> Vec<Vec<Function<'a>>> {
    srcs.iter()
        .zip(measured(srcs))
        .map(|(src, source)| {
            function_spans(&source.tokens)
                .into_iter()
                .filter_map(|(name, tokens)| {
                    let first = source.spans.get(tokens.start)?.start;
                    let last = source.spans.get(tokens.end.checked_sub(1)?)?.end;
                    Some(Function {
                        name,
                        size: total(source.shares.get(tokens)?),
                        text: src.get(first..last)?,
                    })
                })
                .collect()
        })
        .collect()
}

/// The outermost functions of a source: each one's name and its tokens, from its qualifiers to
/// its body's end or its `;`.
fn function_spans<'a>(tokens: &[Token<'a>]) -> Vec<(&'a str, Range<usize>)> {
    let mut out: Vec<(&'a str, Range<usize>)> = Vec::new();
    for (at, pair) in tokens.windows(2).enumerate() {
        let &[Token::Word("fn"), Token::Word(name)] = pair else {
            continue;
        };
        if out.last().is_some_and(|(_, last)| at < last.end) {
            continue;
        }
        let signature = tokens.get(at.saturating_add(1)..).unwrap_or_default();
        let body = tokens.len().saturating_sub(signature_body(signature).len());
        let end = if tokens.get(body) == Some(&Token::Open('{')) {
            group_end(tokens, body)
        } else {
            body.saturating_add(1).min(tokens.len())
        };
        out.push((name, qualified_start(tokens, at)..end));
    }
    out
}

/// Where the function whose `fn` is at `at` starts: at its visibility (`pub`, `pub(crate)`) and
/// its qualifiers (`const`, `async`, `unsafe`, `extern`), if it has them.
fn qualified_start(tokens: &[Token<'_>], at: usize) -> usize {
    let mut start = at;
    loop {
        start = match tokens.get(..start).unwrap_or_default() {
            [
                ..,
                Token::Word("pub" | "const" | "async" | "unsafe" | "extern"),
            ] => start.saturating_sub(1),
            [
                ..,
                Token::Word("pub"),
                Token::Open('('),
                Token::Word(_),
                Token::Close,
            ] => start.saturating_sub(4),
            _ => return start,
        };
    }
}

/// The size of one source text.
#[must_use]
pub fn of(src: &str) -> Size {
    sizes(&[src]).into_iter().next().unwrap_or_default()
}

/// Each `.rs` file's size under the paths, one line each, then the totals.
///
/// A line holds the score, the logic, types and data tokens, the score's depth and duplication
/// parts and the clumps diagnostic. Repeated runs and clumps are found over all the files
/// together.
///
/// # Errors
///
/// The usage when no path is given, or the path and the error when one can't be read.
pub fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err(USAGE.to_owned());
    }
    let mut files = Vec::new();
    for arg in args {
        sources(Path::new(arg), &mut files)?;
    }
    let texts = files
        .iter()
        .map(|file| fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display())))
        .collect::<Result<Vec<_>, _>>()?;
    let srcs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let mut out =
        String::from("      score   logic   types    data     depth duplication    clumps file\n");
    let mut total = Size::default();
    for (file, size) in files.iter().zip(sizes(&srcs)) {
        total = total.add(size);
        let _ = writeln!(out, "{} {}", columns(size), file.display());
    }
    let _ = writeln!(out, "{} total", columns(total));
    Ok(out)
}

/// A size's report columns: score, logic, types, data, depth, duplication, clumps.
fn columns(size: Size) -> String {
    format!(
        "{:>11} {:>7} {:>7} {:>7} {:>9} {:>11} {:>9}",
        size.score, size.logic, size.types, size.data, size.depth, size.duplication, size.clumps
    )
}

#[cfg(test)]
mod tests;
