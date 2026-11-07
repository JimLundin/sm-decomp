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
//! The size rule is held on logic.

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

/// A source's size, split between logic, declarations and data.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Size {
    /// Every counted token that is neither a declaration nor a constant's or a static's value.
    pub logic: usize,
    /// The tokens of item-level `struct`, `enum`, `union` and `type` items and of attributes:
    /// what the code's types say, reported apart so typing is visible on its own.
    pub types: usize,
    /// The tokens of `const` and `static` values: tables and parameters, the game's data, which
    /// the old engine read from the ROM instead (Jim, 2026-11-07).
    pub data: usize,
}

impl Size {
    const fn add(self, other: Self) -> Self {
        Self {
            logic: self.logic.saturating_add(other.logic),
            types: self.types.saturating_add(other.types),
            data: self.data.saturating_add(other.data),
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

/// The count, without `#[cfg(test)]` items, with each bracket pair once, and with the values of
/// item-level constants and statics as data.
fn count(tokens: &[Token<'_>]) -> Size {
    let mut size = Size::default();
    let mut rest = tokens;
    let mut previous = None;
    while let Some((&token, tail)) = rest.split_first() {
        if let Some(item) = rest.strip_prefix(&TEST_ATTRIBUTE[..]) {
            rest = after_item(item);
            continue;
        }
        let constant = matches!(token, Token::Word("const" | "static"))
            && ends_item(previous)
            && !matches!(tail.first(), Some(Token::Word("fn" | "unsafe" | "extern")));
        if constant && let Some((head, value, after)) = split_value(rest) {
            size = size.add(Size {
                logic: counted(head),
                types: 0,
                data: counted(value),
            });
            previous = head.last().copied();
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
            size.types = size.types.saturating_add(counted(item));
            previous = item.last().copied();
            rest = after;
            continue;
        }
        if token != Token::Close {
            size.logic = size.logic.saturating_add(1);
        }
        previous = Some(token);
        rest = tail;
    }
    size
}

/// The counted tokens of a run: all but closing brackets.
fn counted(tokens: &[Token<'_>]) -> usize {
    tokens.iter().filter(|t| **t != Token::Close).count()
}

/// A constant item split into its declaration up to `=`, its value, and the tokens after its `;`
/// (the `;` goes with the declaration's count by being left out of both).
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
                let eq = equals?;
                let head = tokens.get(..=eq)?;
                let value = tokens.get(eq.checked_add(1)?..at)?;
                let after = tokens.get(at..)?;
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

/// Each `.rs` file's logic and data tokens under the paths, one line each, then the totals.
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
    let mut out = String::from("  logic   types    data file\n");
    let mut total = Size::default();
    for file in &files {
        let src = fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let size = of(&src);
        total = total.add(size);
        let _ = writeln!(
            out,
            "{:>7} {:>7} {:>7} {}",
            size.logic,
            size.types,
            size.data,
            file.display()
        );
    }
    let _ = writeln!(
        out,
        "{:>7} {:>7} {:>7} total",
        total.logic, total.types, total.data
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{Size, of};

    fn tokens(src: &str) -> usize {
        let size = of(src);
        size.logic
            .saturating_add(size.types)
            .saturating_add(size.data)
    }

    #[test]
    fn layout_and_comments_cost_nothing() {
        // fn f () a : u8 -> u8 {} a + 1: twelve, each bracket pair once.
        let tight = "fn f(a: u8) -> u8 { a + 1 }";
        let spread =
            "/// Adds one.\nfn f(\n    a: u8, // the input\n) -> u8 {\n    a + 1 /* sum */\n}\n";
        assert_eq!(tokens(tight), 12);
        // The same, plus the trailing comma.
        assert_eq!(tokens(spread), 13);
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
    fn a_constants_value_is_data() {
        // pub const ROWS : [] u8 ; 2 = ; is logic; [] 1 , 2 is data.
        let table = "pub const ROWS: [u8; 2] = [1, 2];";
        assert_eq!(
            of(table),
            Size {
                logic: 10,
                types: 0,
                data: 4
            }
        );
        assert_eq!(
            of("static mut N: u8 = 3;"),
            Size {
                logic: 7,
                types: 0,
                data: 1
            }
        );
    }

    #[test]
    fn declarations_and_attributes_are_types() {
        // #[derive(Debug)] is 5 (# [ derive ( Debug), struct S { a : u8 } is 6; pub fn f ( { is logic.
        let src = "#[derive(Debug)]\npub struct S { a: u8 }\nfn f() {}";
        assert_eq!(
            of(src),
            Size {
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
}
