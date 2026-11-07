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
//! A value that holds behaviour (control flow, a closure, a call) is logic all the same.
//!
//! The size rule is held on one weighted score of the logic (Jim, 2026-11-07), so that a lower
//! score means better code. Forms that hold no behaviour weigh nothing: `use` items, `mod`
//! declarations, a trailing comma before a closing bracket and the forced `const` of a
//! `const fn`; a constant's head (`const NAME: T =` and its `;`) is counted with the types. A
//! function signature weighs half, so a named step costs less than its body inlined twice.
//! `true` and `false` weigh double: flag-style code that a typed state replaces. Every other
//! logic token weighs one.

use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The command line `run` takes.
pub const USAGE: &str = "tools size PATH...";

/// Operators of more than one character, longest first so the scan takes the longest match.
const OPERATORS: &[&str] = &[
    "<<=", ">>=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=",
    "*=", "/=", "%=", "^=", "&=", "|=", "<<", ">>", "..",
];

/// A lexed token. A closing bracket is kept so the test filter can track depth, but a pair counts
/// once.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Token<'a> {
    Open(char),
    Close,
    Word(&'a str),
    Punct(&'a str),
    Literal,
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

/// The source's tokens, in order.
fn lex(src: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(c) = rest.chars().next() {
        let (lexeme, after) = next(rest, c);
        if let Lexeme::Token(token) = lexeme {
            out.push(token);
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
        (Lexeme::Token(Token::Literal), after)
    } else if c == '\'' {
        (Lexeme::Token(Token::Literal), after_quote(rest))
    } else if c.is_ascii_digit() {
        (Lexeme::Token(Token::Literal), after_number(rest))
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

/// What one logic token weighs in the score (Jim, 2026-11-07).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Weight {
    /// A form that holds no behaviour, so trading it costs nothing: a `use` item, a `mod`
    /// declaration, a trailing comma before a closing bracket (rustfmt's), the `const` of a
    /// `const fn` (`missing_const_for_fn` forces it).
    Free,
    /// A function signature, from `fn` to its body's `{`: a named step's price.
    Half,
    /// Every other logic token.
    Single,
    /// `true` or `false` in logic, so a typed state beats a flag.
    Double,
}

impl Weight {
    /// The weight in half tokens, which keeps a signature's half whole.
    const fn halves(self) -> usize {
        match self {
            Self::Free => 0,
            Self::Half => 1,
            Self::Single => 2,
            Self::Double => 4,
        }
    }
}

/// The weighted score of a source's logic, the number the size rule holds, kept in half tokens.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Score(usize);

impl Score {
    /// The score in half tokens.
    #[must_use]
    pub const fn halves(self) -> usize {
        self.0
    }

    const fn add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    const fn weigh(self, weight: Weight) -> Self {
        Self(self.0.saturating_add(weight.halves()))
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / 2;
        let text = if self.0.is_multiple_of(2) {
            whole.to_string()
        } else {
            format!("{whole}.5")
        };
        f.pad(&text)
    }
}

/// A source's size, split between logic, declarations and data, and its logic's score.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Size {
    /// The weighted logic, which the size rule holds.
    pub score: Score,
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
            logic: self.logic.saturating_add(other.logic),
            types: self.types.saturating_add(other.types),
            data: self.data.saturating_add(other.data),
        }
    }

    /// One more logic token of `weight`.
    const fn weigh(self, weight: Weight) -> Self {
        Self {
            score: self.score.weigh(weight),
            logic: self.logic.saturating_add(1),
            ..self
        }
    }

    /// That many counted tokens, all types.
    const fn typed(tokens: usize) -> Self {
        Self {
            score: Score(0),
            logic: 0,
            types: tokens,
            data: 0,
        }
    }
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
            Token::Word(_) | Token::Punct(_) | Token::Literal => {}
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

/// The count, without `#[cfg(test)]` items, with each bracket pair once, with the values of
/// item-level constants and statics as data unless they hold behaviour, and with each logic
/// token weighed.
fn count(tokens: &[Token<'_>]) -> Size {
    let mut size = Size::default();
    let mut rest = tokens;
    let mut previous = None;
    // How many tokens are left once the last signature reaches its body or its `;`.
    let mut signature_end = None;
    while let Some((&token, tail)) = rest.split_first() {
        if let Some(item) = rest.strip_prefix(&TEST_ATTRIBUTE[..]) {
            rest = after_item(item);
            continue;
        }
        if let Some((item, after)) = free_item(rest, previous) {
            size = size.add(item);
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
            size = size.add(Size::typed(counted(item)));
            previous = item.last().copied();
            rest = after;
            continue;
        }
        if token == Token::Word("fn") && matches!(tail.first(), Some(Token::Word(_))) {
            signature_end = Some(signature_body(tail).len());
        }
        if token != Token::Close {
            let in_signature = signature_end.is_some_and(|end| rest.len() > end);
            size = size.weigh(weight(token, tail.first(), in_signature));
        }
        previous = Some(token);
        rest = tail;
    }
    size
}

/// The size of the constant, `use` item or `mod` declaration at the start of `tokens`, whose
/// item `previous` ended, and the tokens after it.
fn free_item<'a, 'b>(
    tokens: &'a [Token<'b>],
    previous: Option<Token<'b>>,
) -> Option<(Size, &'a [Token<'b>])> {
    if !ends_item(previous) {
        return None;
    }
    let (&token, tail) = tokens.split_first()?;
    match token {
        Token::Word("const" | "static")
            if !matches!(tail.first(), Some(Token::Word("fn" | "unsafe" | "extern"))) =>
        {
            let (head, value, after) = split_value(tokens)?;
            let value = if holds_behaviour(value) {
                count(value)
            } else {
                Size {
                    data: counted(value),
                    ..Size::default()
                }
            };
            Some((Size::typed(head).add(value), after))
        }
        Token::Word("use") => {
            let after = after_semicolon(tokens);
            let logic = counted(before(tokens, after));
            Some((
                Size {
                    logic,
                    ..Size::default()
                },
                after,
            ))
        }
        Token::Word("mod") => match tail {
            [
                Token::Word(_),
                Token::Punct(";") | Token::Open('{'),
                after @ ..,
            ] => Some((
                Size {
                    logic: 3,
                    ..Size::default()
                },
                after,
            )),
            _ => None,
        },
        Token::Open(_) | Token::Close | Token::Word(_) | Token::Punct(_) | Token::Literal => None,
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
            Token::Word(_) | Token::Punct(_) | Token::Literal => {}
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
            Token::Word(_) | Token::Punct(_) | Token::Literal => {}
        }
        rest = tail;
    }
    rest
}

/// Keywords that start control flow or a binding, which a value that is data doesn't hold.
const BEHAVIOUR_KEYWORDS: &[&str] = &["if", "match", "loop", "while", "for", "let", "fn", "move"];

/// Lower-case keywords that can stand before a `(` without calling anything.
const NOT_CALLS: &[&str] = &[
    "mut", "as", "in", "return", "ref", "dyn", "impl", "const", "unsafe", "crate", "super", "self",
];

/// Whether a constant's value holds behaviour, so it counts as logic (review L3): control flow,
/// a binding, a closure, a macro, or a call to a function or a method.
///
/// A call is a lower-case name followed by its arguments, `f(..)`, `T::new(..)`, `x.get(..)` or
/// `size_of::<T>()`. Rust names functions and methods in snake case and tuple structs and enum
/// variants in camel case (the `non_snake_case` and `non_camel_case_types` lints hold it), so a
/// table row built from constructors, `Held(Key::Jump)`, `Not(&Airborne)` or `Some(3)`, and
/// struct literals stay data.
fn holds_behaviour(value: &[Token<'_>]) -> bool {
    let mut previous = None;
    let mut rest = value;
    while let Some((&token, tail)) = rest.split_first() {
        let next = tail.first().copied();
        let behaviour = match token {
            Token::Word(word) if BEHAVIOUR_KEYWORDS.contains(&word) => true,
            Token::Word(word) => {
                let function = word.starts_with(|c: char| c.is_lowercase() || c == '_')
                    && !NOT_CALLS.contains(&word);
                function
                    && matches!(
                        (next, tail.get(1)),
                        (Some(Token::Open('(')), _)
                            | (Some(Token::Punct("::")), Some(Token::Punct("<")))
                    )
            }
            Token::Punct("!") => matches!(previous, Some(Token::Word(_))),
            Token::Punct("|" | "||") => matches!(
                previous,
                None | Some(Token::Open(_) | Token::Punct("," | ";" | ":" | "=>"))
            ),
            Token::Open(_) | Token::Close | Token::Punct(_) | Token::Literal => false,
        };
        if behaviour {
            return true;
        }
        previous = Some(token);
        rest = tail;
    }
    false
}

/// The counted tokens of a run: all but closing brackets.
fn counted(tokens: &[Token<'_>]) -> usize {
    tokens.iter().filter(|t| **t != Token::Close).count()
}

/// A constant item split into its head's count (from `const` to `=`, and the `;`), its value
/// (none in a trait's `const X: T;`), and the tokens after its `;`.
fn split_value<'a, 'b>(
    tokens: &'a [Token<'b>],
) -> Option<(usize, &'a [Token<'b>], &'a [Token<'b>])> {
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
                    return Some((counted(tokens.get(..=at)?), &[], after));
                };
                let head = counted(tokens.get(..=eq)?).saturating_add(1);
                let value = tokens.get(eq.checked_add(1)?..at)?;
                return Some((head, value, after));
            }
            Token::Word(_) | Token::Punct(_) | Token::Literal => {}
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
            Token::Word(_) | Token::Punct(_) | Token::Literal => {}
        }
    }
    rest
}

/// The `.rs` files under `path`, skipping `tests` directories and `tests.rs` files, sorted.
fn sources(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
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

/// The size of one source text.
#[must_use]
pub fn of(src: &str) -> Size {
    count(&lex(src))
}

/// Each `.rs` file's score, then its logic, types and data tokens, under the paths, one line
/// each, then the totals.
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
    let mut out = String::from("    score   logic   types    data file\n");
    let mut total = Size::default();
    for file in &files {
        let src = fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let size = of(&src);
        total = total.add(size);
        let _ = writeln!(out, "{} {}", columns(size), file.display());
    }
    let _ = writeln!(out, "{} total", columns(total));
    Ok(out)
}

/// A size's report columns: score, logic, types, data.
fn columns(size: Size) -> String {
    format!(
        "{:>9} {:>7} {:>7} {:>7}",
        size.score, size.logic, size.types, size.data
    )
}

#[cfg(test)]
mod tests {
    use super::{Score, Size, of};

    fn tokens(src: &str) -> usize {
        let size = of(src);
        size.logic
            .saturating_add(size.types)
            .saturating_add(size.data)
    }

    /// The score in half tokens.
    fn halves(src: &str) -> usize {
        of(src).score.halves()
    }

    #[test]
    fn layout_and_comments_cost_nothing() {
        // fn f () a : u8 -> u8 {} a + 1: twelve, each bracket pair once.
        let tight = "fn f(a: u8) -> u8 { a + 1 }";
        let spread =
            "/// Adds one.\nfn f(\n    a: u8, // the input\n) -> u8 {\n    a + 1 /* sum */\n}\n";
        assert_eq!(tokens(tight), 12);
        // The same, plus the trailing comma, which weighs nothing.
        assert_eq!(tokens(spread), 13);
        assert_eq!(halves(spread), halves(tight));
    }

    #[test]
    fn literals_lifetimes_and_operators_are_one_each() {
        assert_eq!(tokens(r##"r#"a "b" c"#"##), 1);
        assert_eq!(tokens(r#"b"x\"y""#), 1);
        assert_eq!(tokens("b'x'"), 1);
        assert_eq!(tokens("'a'"), 1);
        assert_eq!(tokens("'\\n'"), 1);
        assert_eq!(tokens("'\\''"), 1);
        assert_eq!(tokens("&'a str"), 3);
        assert_eq!(tokens("0..=0x1F"), 3);
        assert_eq!(tokens("1.5e-3"), 1);
        assert_eq!(tokens("x.0.max(1)"), 7);
        assert_eq!(tokens("a::b"), 3);
        assert_eq!(tokens("r#type"), 1);
        assert_eq!(tokens("/* a /* nested */ b */ x"), 1);
    }

    #[test]
    fn test_items_are_not_counted() {
        let src = "fn a() {}\n#[cfg(test)]\nmod tests { fn b() { c(); } }\n#[cfg(test)]\nuse x::y;\nfn d() {}";
        assert_eq!(tokens(src), tokens("fn a() {} fn d() {}"));
    }

    #[test]
    fn a_constants_value_is_data_and_its_head_a_type() {
        // const ROWS : [] u8 ; 2 = and the ; are types; [] 1 , 2 is data; pub is logic.
        let table = "pub const ROWS: [u8; 2] = [1, 2];";
        assert_eq!(
            of(table),
            Size {
                score: Score(2),
                logic: 1,
                types: 9,
                data: 4
            }
        );
        assert_eq!(
            of("static mut N: u8 = 3;"),
            Size {
                score: Score(0),
                logic: 0,
                types: 7,
                data: 1
            }
        );
        // A trait's constant without a value is a declaration too.
        assert_eq!(of("trait T { const X: u8; }").types, 5);
    }

    #[test]
    fn declarations_and_attributes_are_types() {
        // #[derive(Debug)] is 5 (# [ derive ( Debug), struct S { a : u8 } is 6; pub fn f ( { is logic.
        let src = "#[derive(Debug)]\npub struct S { a: u8 }\nfn f() {}";
        assert_eq!(
            of(src),
            Size {
                // pub 1, then fn f ( at half each, then { 1.
                score: Score(7),
                logic: 5,
                types: 11,
                data: 0
            }
        );
        assert_eq!(of("fn f() -> S { S { a: 1 } }").types, 0);
    }

    #[test]
    fn const_fns_generics_and_locals_are_logic() {
        assert_eq!(of("const fn f() {}").data, 0);
        assert_eq!(of("fn f<const N: usize>() {}").data, 0);
        assert_eq!(of("fn f() { let a = 1; }").data, 0);
        // A constant inside a function body is still a constant's value.
        assert_eq!(of("fn f() { const A: u8 = 1; }").data, 1);
    }

    #[test]
    fn imports_and_module_declarations_weigh_nothing() {
        let src = "use a::{b, c};\nmod m;\nmod n { fn f() {} }";
        // f's body brace alone weighs one; fn f ( weigh half each.
        assert_eq!(of(src).logic, 18);
        assert_eq!(halves(src), halves("fn f() {}"));
        // A `use` inside a body is an import too.
        assert_eq!(halves("{ use E::A; }"), 2);
    }

    #[test]
    fn a_const_fns_const_weighs_nothing() {
        assert_eq!(halves("const fn f() {}"), halves("fn f() {}"));
        assert_eq!(of("const fn f() {}").logic, 5);
    }

    #[test]
    fn a_signature_weighs_half() {
        // fn f ( a : u8 -> u8 is eight tokens at half, the body's { a + 1 four at one.
        assert_eq!(halves("fn f(a: u8) -> u8 { a + 1 }"), 8 + 2 * 4);
        // Up to a declaration's `;`, with generics and bounds; the `;` is the item's, at one.
        assert_eq!(halves("fn g<T: Into<u8>>(t: T) where T: Copy;"), 17 + 2);
        // A function-pointer type is not a signature.
        assert_eq!(halves("let p: fn(u8) = q;"), 2 * 9);
    }

    #[test]
    fn true_and_false_weigh_double_in_logic() {
        assert_eq!(halves("x = true;"), 2 + 2 + 4 + 2);
        assert_eq!(halves("x = y;"), 2 * 4);
        // In a constant's value they are data.
        assert_eq!(halves("const A: bool = true;"), 0);
        assert_eq!(of("const A: bool = true;").data, 1);
    }

    #[test]
    fn a_table_of_constructors_stays_data() {
        let rows = "const ROWS: &[Row] = &[Row { needs: &[Held(Key::Jump), Not(&Airborne)], \
                    aim: AIM_UP_ELSE_DOWN, at: Some(Fixed(3)) }, Row { needs: &[], ..BASE }];";
        let size = of(rows);
        assert_eq!((size.logic, size.score), (0, Score(0)));
        assert!(size.data > 0);
        // Arithmetic and bit operators on constants are a stated value, not behaviour.
        assert_eq!(of("const M: u16 = A | B << 4;").logic, 0);
    }

    #[test]
    fn a_constant_holding_behaviour_is_logic() {
        for value in [
            "if A { 1 } else { 2 }",
            "match A { B => 1, C => 2 }",
            "{ let a = 3; a }",
            "|a: u8| a",
            "&[(1, |a| a)]",
            "move || 0",
            "Fixed::new(3)",
            "A.wrapping_add(B)",
            "size_of::<u8>()",
            "build()",
            "[f(1), f(2)]",
            "concat!(1)",
        ] {
            let size = of(&format!("const A: T = {value};"));
            assert_eq!(size.data, 0, "{value}");
            assert!(size.logic > 0, "{value}");
            // The head is still a declaration.
            assert_eq!(size.types, 6, "{value}");
        }
    }

    #[test]
    fn a_score_prints_its_half() {
        assert_eq!(format!("{:>6}", Score(15)), "   7.5");
        assert_eq!(Score(14).to_string(), "7");
    }
}
