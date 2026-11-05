//! Code volume measured in lexer tokens rather than physical lines
//! (docs/VERIFY_PIPELINE.md, "Code volume").
//!
//! Joining or splitting lines moves `production_lines` but not a token count,
//! so the engine's size is measured in tokens:
//!
//! - **logic tokens**: every token outside comments, `#[cfg(test)]` items and
//!   pure data items;
//! - **data tokens**: the tokens of pure data items, a `const` or `static`
//!   whose value is only literals, arrays, tuples, struct and tuple-struct
//!   literals and paths, with their visibility, plus the items' entry counts;
//! - **comment lines**: lines holding any comment or doc comment, reported only;
//! - **structure counts**: lexical heuristics, listed on [`Volume`].
//!
//! The scope is `production_lines`' (tools/ram_ratchet.py): `.rs` files under
//! `crates/*/src` and `sm/src`. The engine is crates/sm_data, sm_game,
//! sm_runtime and sm_app outside `verification` directories, reported per crate
//! and per top-level sm_game module. sm/src outside `verification` (the oracle
//! and the legacy game) and verification (`verification` directories,
//! crates/sm_tools/src outside its `fixture` test inputs) are reported
//! separately, so the engine measures do not move when the oracle does.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::ops::AddAssign;
use std::path::{Path, PathBuf};

/// The engine packages, in report order.
pub const ENGINE: [&str; 4] = ["sm_data", "sm_game", "sm_runtime", "sm_app"];

/// The measures `tools/ram_ratchet.py` reads, in report order.
pub const MEASURES: [&str; 4] = [
    "engine_logic_tokens",
    "engine_data_tokens",
    "engine_traits",
    "engine_blanket_impls",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Ident,
    Lifetime,
    Literal,
    Punct,
}

#[derive(Clone, Debug)]
struct Token {
    kind: Kind,
    text: String,
    line: usize,
}

/// Compound punctuation, longest first; anything else is one character.
const PUNCTS: [&str; 24] = [
    "<<=", ">>=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=",
    "*=", "/=", "%=", "^=", "&=", "|=", "<<", ">>", "..",
];

/// Keywords that make a `const` or `static` value computed rather than data.
const COMPUTING: [&str; 20] = [
    "if", "else", "match", "loop", "while", "for", "unsafe", "async", "await", "move", "return",
    "let", "as", "break", "continue", "fn", "impl", "dyn", "const", "static",
];

fn ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

struct Lexer {
    chars: Vec<char>,
    at: usize,
    line: usize,
    tokens: Vec<Token>,
    comment_lines: Vec<bool>,
}

impl Lexer {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.at + ahead).copied()
    }

    fn bump(&mut self) {
        if self.peek(0) == Some('\n') {
            self.line += 1;
        }
        self.at += 1;
    }

    fn bump_while(&mut self, keep: impl Fn(char) -> bool) {
        while self.peek(0).is_some_and(&keep) {
            self.bump();
        }
    }

    fn push(&mut self, kind: Kind, start: usize, line: usize) {
        let text = self.chars[start..self.at].iter().collect();
        self.tokens.push(Token { kind, text, line });
    }

    /// Marks the lines from `from` through the current one as holding a comment.
    fn comment(&mut self, from: usize) {
        if self.comment_lines.len() <= self.line {
            self.comment_lines.resize(self.line + 1, false);
        }
        self.comment_lines[from..=self.line].fill(true);
    }

    /// A nested block comment, from its opening `/*`.
    fn block_comment(&mut self) {
        let mut depth = 0usize;
        while let Some(c) = self.peek(0) {
            match (c, self.peek(1)) {
                ('/', Some('*')) => {
                    depth += 1;
                    self.at += 2;
                }
                ('*', Some('/')) => {
                    self.at += 2;
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                _ => self.bump(),
            }
        }
    }

    /// The rest of a quoted literal after its opening quote, with escapes.
    fn quoted(&mut self, close: char) {
        while let Some(c) = self.peek(0) {
            self.bump();
            if c == '\\' {
                self.bump();
            } else if c == close {
                return;
            }
        }
    }

    /// A raw string whose `#`s or quote start `offset` characters ahead, if
    /// one does; consumed through its closing quote and `#`s.
    fn raw_string(&mut self, offset: usize) -> bool {
        let hashes = (0..)
            .take_while(|&ahead| self.peek(offset + ahead) == Some('#'))
            .count();
        if self.peek(offset + hashes) != Some('"') {
            return false;
        }
        self.at += offset + hashes + 1;
        while let Some(c) = self.peek(0) {
            self.bump();
            if c == '"' && (0..hashes).all(|ahead| self.peek(ahead) == Some('#')) {
                self.at += hashes;
                return true;
            }
        }
        true
    }

    /// An identifier, keyword or raw identifier, or a literal with a prefix.
    fn word(&mut self, start: usize, line: usize) {
        let literal = match (self.peek(0), self.peek(1), self.peek(2)) {
            (Some('r'), Some('#' | '"'), _) => self.raw_string(1),
            (Some('b' | 'c'), Some('r'), Some('#' | '"')) => self.raw_string(2),
            (Some('b' | 'c'), Some('"'), _) => {
                self.at += 2;
                self.quoted('"');
                true
            }
            (Some('b'), Some('\''), _) => {
                self.at += 2;
                self.quoted('\'');
                true
            }
            _ => false,
        };
        if literal {
            self.push(Kind::Literal, start, line);
            return;
        }
        if self.peek(0) == Some('r') && self.peek(1) == Some('#') {
            self.at += 2;
        }
        self.bump_while(ident_continue);
        self.push(Kind::Ident, start, line);
    }

    /// A character literal or a lifetime (or loop label), from its quote.
    fn quote(&mut self, start: usize, line: usize) {
        match (self.peek(1), self.peek(2)) {
            (Some('\\'), _) => {
                self.bump();
                self.quoted('\'');
                self.push(Kind::Literal, start, line);
            }
            (Some(_), Some('\'')) => {
                self.at += 3;
                self.push(Kind::Literal, start, line);
            }
            (Some(c), _) if ident_start(c) => {
                self.bump();
                self.bump_while(ident_continue);
                self.push(Kind::Lifetime, start, line);
            }
            _ => {
                self.bump();
                self.push(Kind::Punct, start, line);
            }
        }
    }

    /// A numeric literal with its suffix.
    fn number(&mut self, start: usize, line: usize) {
        let digit = |c: char| c.is_ascii_digit() || c == '_';
        if self.peek(0) == Some('0') && matches!(self.peek(1), Some('x' | 'o' | 'b')) {
            self.at += 2;
            self.bump_while(ident_continue);
        } else {
            self.bump_while(digit);
            if self.peek(0) == Some('.') && self.peek(1).is_some_and(|c| c.is_ascii_digit()) {
                self.bump();
                self.bump_while(digit);
            }
            let exponent = match (self.peek(1), self.peek(2)) {
                (Some(c), _) if c.is_ascii_digit() => 1,
                (Some('+' | '-'), Some(c)) if c.is_ascii_digit() => 2,
                _ => 0,
            };
            if matches!(self.peek(0), Some('e' | 'E')) && exponent > 0 {
                self.at += 1 + exponent;
                self.bump_while(digit);
            }
            self.bump_while(ident_continue);
        }
        self.push(Kind::Literal, start, line);
    }

    fn punct(&mut self, start: usize, line: usize) {
        let width = PUNCTS
            .iter()
            .find(|punct| {
                punct
                    .chars()
                    .enumerate()
                    .all(|(ahead, c)| self.peek(ahead) == Some(c))
            })
            .map_or(1, |punct| punct.len());
        self.at += width;
        self.push(Kind::Punct, start, line);
    }

    fn run(mut self) -> (Vec<Token>, Vec<bool>) {
        while let Some(c) = self.peek(0) {
            let (start, line) = (self.at, self.line);
            match (c, self.peek(1)) {
                (c, _) if c.is_whitespace() || c == '\u{feff}' => self.bump(),
                ('/', Some('/')) => {
                    self.bump_while(|c| c != '\n');
                    self.comment(line);
                }
                ('/', Some('*')) => {
                    self.block_comment();
                    self.comment(line);
                }
                ('"', _) => {
                    self.bump();
                    self.quoted('"');
                    self.push(Kind::Literal, start, line);
                }
                ('\'', _) => self.quote(start, line),
                (c, _) if c.is_ascii_digit() => self.number(start, line),
                (c, _) if ident_start(c) => self.word(start, line),
                _ => self.punct(start, line),
            }
        }
        (self.tokens, self.comment_lines)
    }
}

fn lex(text: &str) -> (Vec<Token>, Vec<bool>) {
    Lexer {
        chars: text.chars().collect(),
        at: 0,
        line: 0,
        tokens: Vec::new(),
        comment_lines: Vec::new(),
    }
    .run()
}

/// The size and shape of some source.
///
/// The structure counts are lexical:
/// - `traits`: `trait` followed by a name;
/// - `impls`: `impl` at the start of an item (after `;`, `{`, `}`, an
///   attribute's `]`, `(`, `unsafe` or `default`), so `impl Trait` types don't count;
/// - `blanket_impls`: such an `impl<...>` whose self type after `for` is one
///   of its own type parameters, possibly behind `&`, `&mut` or a lifetime;
/// - `fns`: `fn` followed by a name, with or without a body (not `fn(..)` types);
/// - `match_arms`: `=>` outside `macro_rules!` bodies (a macro invocation's own
///   `=>` syntax counts too);
/// - `macros`: `macro_rules!` definitions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Volume {
    pub files: usize,
    /// Physical lines outside `#[cfg(test)]` items, as `production_lines` counts.
    pub lines: usize,
    pub comment_lines: usize,
    pub logic_tokens: usize,
    pub data_tokens: usize,
    pub data_items: usize,
    /// The top-level elements of each data item's outermost array or slice;
    /// a repeat array `[x; N]` and a value that is no array count one.
    pub data_entries: usize,
    pub traits: usize,
    pub impls: usize,
    pub blanket_impls: usize,
    pub fns: usize,
    pub match_arms: usize,
    pub macros: usize,
}

impl AddAssign for Volume {
    fn add_assign(&mut self, other: Self) {
        self.files += other.files;
        self.lines += other.lines;
        self.comment_lines += other.comment_lines;
        self.logic_tokens += other.logic_tokens;
        self.data_tokens += other.data_tokens;
        self.data_items += other.data_items;
        self.data_entries += other.data_entries;
        self.traits += other.traits;
        self.impls += other.impls;
        self.blanket_impls += other.blanket_impls;
        self.fns += other.fns;
        self.match_arms += other.match_arms;
        self.macros += other.macros;
    }
}

fn text_is(tokens: &[&Token], at: usize, text: &str) -> bool {
    tokens.get(at).is_some_and(|token| token.text == text)
}

fn is_name(token: Option<&&Token>) -> bool {
    token.is_some_and(|token| token.kind == Kind::Ident)
}

/// The index of the bracket closing the one at `open`, or the end.
fn closing(tokens: &[&Token], open: usize) -> usize {
    let mut depth = 0usize;
    for (at, token) in tokens.iter().enumerate().skip(open) {
        match token.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            _ => {}
        }
    }
    tokens.len()
}

/// The length of a `#[cfg(test)]` (or inner `#![cfg(test)]`) attribute at `at`.
fn cfg_test(tokens: &[&Token], at: usize) -> Option<(usize, bool)> {
    let inner = text_is(tokens, at + 1, "!");
    let from = at + 1 + usize::from(inner);
    let matched = text_is(tokens, at, "#")
        && ["[", "cfg", "(", "test", ")", "]"]
            .iter()
            .enumerate()
            .all(|(offset, text)| text_is(tokens, from + offset, text));
    matched.then_some((from + 6 - at, inner))
}

/// The end (exclusive) of the item starting at `from`: past further attributes,
/// through its `;` or its closing brace (and a `;` after it), or before the
/// bracket that closes the item's container.
fn item_end(tokens: &[&Token], from: usize) -> usize {
    let mut at = from;
    while text_is(tokens, at, "#") && text_is(tokens, at + 1, "[") {
        at = closing(tokens, at + 1) + 1;
    }
    let mut depth = 0usize;
    while let Some(token) = tokens.get(at) {
        match token.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" if depth == 0 => return at,
            ")" | "]" => depth -= 1,
            "}" => {
                depth -= 1;
                if depth == 0 {
                    return at + 1 + usize::from(text_is(tokens, at + 1, ";"));
                }
            }
            ";" if depth == 0 => return at + 1,
            _ => {}
        }
        at += 1;
    }
    tokens.len()
}

/// A `const` or `static` item with a value at `at`: the item's span (with its
/// visibility) and its value's span.
fn constant(tokens: &[&Token], at: usize) -> Option<(usize, usize, usize, usize)> {
    let keyword = tokens[at];
    if keyword.kind != Kind::Ident || !matches!(keyword.text.as_str(), "const" | "static") {
        return None;
    }
    // Generic const parameters, pointer types and `&raw const` are not items.
    if at > 0 && matches!(tokens[at - 1].text.as_str(), "<" | "," | "*" | "raw") {
        return None;
    }
    let mut name = at + 1;
    if keyword.text == "static" && text_is(tokens, name, "mut") {
        name += 1;
    }
    if !is_name(tokens.get(name))
        || COMPUTING.contains(&tokens[name].text.as_str())
        || !text_is(tokens, name + 1, ":")
    {
        return None;
    }
    let end_of = |from: usize, stop: &str| {
        let mut depth = 0usize;
        for (index, token) in tokens.iter().enumerate().skip(from) {
            match token.text.as_str() {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth = depth.checked_sub(1)?,
                text if depth == 0 && (text == stop || text == ";") => {
                    return Some((index, text == stop));
                }
                _ => {}
            }
        }
        None
    };
    let (equals, valued) = end_of(name + 2, "=")?;
    if !valued {
        return None;
    }
    let (semicolon, _) = end_of(equals + 1, ";")?;
    let mut start = at;
    if start > 0 && tokens[start - 1].text == ")" {
        let open = (0..start - 1)
            .rev()
            .find(|&index| tokens[index].text == "(");
        if let Some(open) = open.filter(|&open| open > 0 && tokens[open - 1].text == "pub") {
            start = open - 1;
        }
    } else if start > 0 && tokens[start - 1].text == "pub" {
        start -= 1;
    }
    Some((start, semicolon + 1, equals + 1, semicolon))
}

/// Whether a constant's value is only literals, arrays, tuples, struct or
/// tuple-struct literals and paths. A lowercase name before `(` is a call; an
/// uppercase one is a tuple struct or variant.
fn pure(value: &[&Token]) -> bool {
    value.iter().enumerate().all(|(at, token)| {
        let previous = at.checked_sub(1).map(|index| value[index]);
        match token.kind {
            Kind::Literal | Kind::Lifetime => true,
            Kind::Ident => {
                !COMPUTING.contains(&token.text.as_str())
                    && !(text_is(value, at + 1, "(")
                        && token.text.starts_with(|c: char| c.is_lowercase()))
            }
            Kind::Punct => match token.text.as_str() {
                "::" | "," | ":" | ";" | "(" | ")" | "[" | "]" | "}" | ".." => true,
                "{" => previous.is_some_and(|previous| {
                    previous.kind == Kind::Ident && !COMPUTING.contains(&previous.text.as_str())
                }),
                "-" | "&" => previous.is_none_or(|previous| {
                    matches!(
                        previous.text.as_str(),
                        "," | "(" | "[" | "{" | ":" | ";" | "&"
                    )
                }),
                _ => false,
            },
        }
    })
}

/// The entries of a pure value: its outermost array's top-level elements.
fn entries(value: &[&Token]) -> usize {
    let open = value
        .iter()
        .position(|token| !matches!(token.text.as_str(), "&" | "mut"));
    let Some(open) = open.filter(|&open| value[open].text == "[") else {
        return 1;
    };
    let close = closing(value, open);
    let inner = &value[open + 1..close.min(value.len())];
    if inner.is_empty() {
        return 0;
    }
    let mut depth = 0usize;
    let mut commas = 0;
    for token in inner {
        match token.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth -= 1,
            ";" if depth == 0 => return 1,
            "," if depth == 0 => commas += 1,
            _ => {}
        }
    }
    commas + 1 - usize::from(inner.last().is_some_and(|token| token.text == ","))
}

/// Whether the `impl` at `at` is a blanket impl: `impl<..., T, ...> X for T`.
fn blanket(tokens: &[&Token], at: usize) -> bool {
    if !text_is(tokens, at + 1, "<") {
        return false;
    }
    let mut parameters = Vec::new();
    let (mut angles, mut brackets, mut starts) = (0i32, 0i32, false);
    let mut index = at + 1;
    while let Some(token) = tokens.get(index) {
        match token.text.as_str() {
            "<" => {
                angles += 1;
                starts = angles == 1 && brackets == 0;
                index += 1;
                if angles == 1 {
                    continue;
                }
            }
            ">" => angles -= 1,
            ">>" => angles -= 2,
            "(" | "[" => brackets += 1,
            ")" | "]" => brackets -= 1,
            "," if angles == 1 && brackets == 0 => {
                starts = true;
                index += 1;
                continue;
            }
            _ => {}
        }
        if angles <= 0 {
            break;
        }
        if starts {
            starts = false;
            if token.kind == Kind::Ident && token.text != "const" {
                parameters.push(token.text.as_str());
            }
        }
        index += 1;
    }
    let mut depth = 0i32;
    let mut target = None;
    for (offset, token) in tokens.iter().enumerate().skip(index + 1) {
        match token.text.as_str() {
            "<" | "(" | "[" => depth += 1,
            ">" | ")" | "]" => depth -= 1,
            ">>" => depth -= 2,
            "{" | ";" | "where" if depth == 0 => return false,
            "for" if depth == 0 => {
                target = Some(offset + 1);
                break;
            }
            _ => {}
        }
    }
    let Some(mut target) = target else {
        return false;
    };
    while tokens.get(target).is_some_and(|token| {
        token.kind == Kind::Lifetime || matches!(token.text.as_str(), "&" | "&&" | "mut")
    }) {
        target += 1;
    }
    is_name(tokens.get(target))
        && parameters.contains(&tokens[target].text.as_str())
        && (text_is(tokens, target + 1, "{") || text_is(tokens, target + 1, "where"))
}

/// Measures one source file.
pub fn measure(text: &str) -> Volume {
    let (tokens, comment_lines) = lex(text);
    let all: Vec<&Token> = tokens.iter().collect();
    let total_lines = text.matches('\n').count();
    let mut test_lines = vec![false; total_lines + 1];
    let mut kept = Vec::with_capacity(all.len());
    let mut at = 0;
    while at < all.len() {
        match cfg_test(&all, at) {
            Some((_, true)) => {
                return Volume {
                    files: 1,
                    ..Volume::default()
                };
            }
            Some((length, false)) => {
                let end = item_end(&all, at + length).max(at + length);
                let last = all[end.min(all.len()) - 1].line;
                test_lines[all[at].line..=last].fill(true);
                at = end;
            }
            None => {
                kept.push(all[at]);
                at += 1;
            }
        }
    }
    let mut volume = Volume {
        files: 1,
        lines: total_lines
            - test_lines[..total_lines]
                .iter()
                .filter(|&&test| test)
                .count(),
        comment_lines: comment_lines
            .iter()
            .enumerate()
            .filter(|&(line, &comment)| comment && !test_lines.get(line).copied().unwrap_or(false))
            .count(),
        ..Volume::default()
    };
    let mut at = 0;
    while at < kept.len() {
        match constant(&kept, at) {
            Some((start, end, value, semicolon)) => {
                let value = &kept[value..semicolon];
                if pure(value) {
                    volume.data_items += 1;
                    volume.data_tokens += end - start;
                    volume.data_entries += entries(value);
                    // The visibility before `const` was counted as logic.
                    volume.logic_tokens -= at - start;
                } else {
                    volume.logic_tokens += end - at;
                }
                at = end;
            }
            None => {
                volume.logic_tokens += 1;
                at += 1;
            }
        }
    }
    let mut macro_end = 0;
    for (at, token) in kept.iter().enumerate() {
        let next = kept.get(at + 1);
        let previous = at.checked_sub(1).map(|index| kept[index].text.as_str());
        match token.text.as_str() {
            "macro_rules" if text_is(&kept, at + 1, "!") => {
                volume.macros += 1;
                if let Some(open) = (at + 2..kept.len())
                    .find(|&index| matches!(kept[index].text.as_str(), "{" | "(" | "["))
                {
                    macro_end = closing(&kept, open);
                }
            }
            "trait" if is_name(next) => volume.traits += 1,
            "fn" if is_name(next) => volume.fns += 1,
            "impl"
                if previous.is_none_or(|previous| {
                    matches!(previous, ";" | "{" | "}" | "]" | "(" | "unsafe" | "default")
                }) =>
            {
                volume.impls += 1;
                volume.blanket_impls += usize::from(blanket(&kept, at));
            }
            "=>" if at > macro_end => volume.match_arms += 1,
            _ => {}
        }
    }
    volume
}

/// Where a file's volume is reported.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Row {
    /// An engine package, and the top-level module for sm_game.
    Engine(usize, Option<String>),
    /// sm/src outside verification: the oracle and the legacy game.
    Oracle,
    Verification,
}

/// The row of a `.rs` file, from its path relative to the repository root.
pub fn row(relative: &str) -> Option<Row> {
    let parts: Vec<&str> = relative.split('/').collect();
    let verification = parts.contains(&"verification");
    match parts.as_slice() {
        ["crates", "sm_tools", "src", rest @ ..] => {
            (!rest.contains(&"fixture")).then_some(Row::Verification)
        }
        ["crates", _, "src", ..] if verification => Some(Row::Verification),
        // The engine host compiles the legacy game moved out of sm/src.
        ["crates", "sm_host", "src", ..] => Some(Row::Oracle),
        ["crates", package, "src", first, ..] => {
            let index = ENGINE.iter().position(|engine| engine == package)?;
            let module = (*package == "sm_game")
                .then(|| first.strip_suffix(".rs").unwrap_or(*first).to_owned());
            Some(Row::Engine(index, module))
        }
        ["sm", "src", ..] if verification => Some(Row::Verification),
        ["sm", "src", ..] => Some(Row::Oracle),
        _ => None,
    }
}

fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) -> Result<(), String> {
    let listing = std::fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    for entry in listing {
        let path = entry
            .map_err(|error| format!("{}: {error}", directory.display()))?
            .path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "tests") {
                rust_files(&path, found)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    Ok(())
}

fn read(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    String::from_utf8(bytes).map_err(|error| format!("{}: {error}", path.display()))
}

/// The volume of every row under a repository root.
pub fn survey(root: &Path) -> Result<BTreeMap<Row, Volume>, String> {
    let mut files = Vec::new();
    for top in ["crates", "sm/src"] {
        let directory = root.join(top);
        if directory.is_dir() {
            rust_files(&directory, &mut files)?;
        }
    }
    let mut rows = BTreeMap::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if let Some(row) = row(&relative) {
            *rows.entry(row).or_default() += measure(&read(&path)?);
        }
    }
    Ok(rows)
}

/// The engine total of a survey.
pub fn engine(rows: &BTreeMap<Row, Volume>) -> Volume {
    let mut total = Volume::default();
    for (row, volume) in rows {
        if matches!(row, Row::Engine(..)) {
            total += *volume;
        }
    }
    total
}

/// The measures `tools/ram_ratchet.py` reads, with their values.
pub fn measures(rows: &BTreeMap<Row, Volume>) -> [(&'static str, usize); 4] {
    let total = engine(rows);
    let values = [
        total.logic_tokens,
        total.data_tokens,
        total.traits,
        total.blanket_impls,
    ];
    let mut measured = [("", 0); 4];
    for ((slot, name), value) in measured.iter_mut().zip(MEASURES).zip(values) {
        *slot = (name, value);
    }
    measured
}

const HEADER: &str = "lines comments logic data items entries traits impls blanket fns arms macros";

fn table_row(report: &mut String, label: &str, volume: &Volume) {
    let columns = [
        volume.lines,
        volume.comment_lines,
        volume.logic_tokens,
        volume.data_tokens,
        volume.data_items,
        volume.data_entries,
        volume.traits,
        volume.impls,
        volume.blanket_impls,
        volume.fns,
        volume.match_arms,
        volume.macros,
    ];
    let _ = write!(report, "{label:<28}");
    for (column, width) in columns.iter().zip([8, 9, 9, 9, 6, 8, 7, 6, 8, 6, 6, 7]) {
        let _ = write!(report, "{column:>width$}");
    }
    report.push('\n');
}

/// The report: engine per package and per sm_game module, then the oracle and
/// verification, then the ratchet's measures as `name: value` lines.
pub fn report(rows: &BTreeMap<Row, Volume>) -> String {
    let mut report = String::new();
    let _ = write!(report, "{:<28}", "code volume");
    for (name, width) in HEADER.split(' ').zip([8, 9, 9, 9, 6, 8, 7, 6, 8, 6, 6, 7]) {
        let _ = write!(report, "{name:>width$}");
    }
    report.push('\n');
    table_row(&mut report, "engine", &engine(rows));
    for (index, package) in ENGINE.iter().enumerate() {
        let mut total = Volume::default();
        for (row, volume) in rows {
            if matches!(row, Row::Engine(at, _) if *at == index) {
                total += *volume;
            }
        }
        table_row(&mut report, &format!("  {package}"), &total);
    }
    for (row, volume) in rows {
        if let Row::Engine(_, Some(module)) = row {
            table_row(&mut report, &format!("    sm_game::{module}"), volume);
        }
    }
    let zero = Volume::default();
    table_row(
        &mut report,
        "oracle and legacy (sm/src)",
        rows.get(&Row::Oracle).unwrap_or(&zero),
    );
    table_row(
        &mut report,
        "verification",
        rows.get(&Row::Verification).unwrap_or(&zero),
    );
    for (name, value) in measures(rows) {
        let _ = writeln!(report, "{name}: {value}");
    }
    report
}

/// The `"name": value` pairs of a baseline file such as tools/ram_ratchet.py.
pub fn baseline(text: &str) -> BTreeMap<String, usize> {
    let mut found = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix('"') else {
            continue;
        };
        let Some((name, value)) = rest.split_once("\":") else {
            continue;
        };
        let digits: String = value
            .trim_start()
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(value) = digits.parse() {
            found.insert(name.to_owned(), value);
        }
    }
    found
}

/// Each measure against its baseline, and whether none rose.
pub fn compare(rows: &BTreeMap<Row, Volume>, baseline: &BTreeMap<String, usize>) -> (String, bool) {
    let mut text = String::new();
    let mut held = true;
    for (name, value) in measures(rows) {
        let Some(&base) = baseline.get(name) else {
            let _ = writeln!(text, "{name}: {value} (no baseline)");
            continue;
        };
        let delta = i128::try_from(value).unwrap_or(i128::MAX) - i128::try_from(base).unwrap_or(0);
        let _ = writeln!(text, "{name}: {value} (baseline {base}, delta {delta:+})");
        held &= value <= base;
    }
    (text, held)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: Volume = Volume {
        files: 1,
        lines: 0,
        comment_lines: 0,
        logic_tokens: 0,
        data_tokens: 0,
        data_items: 0,
        data_entries: 0,
        traits: 0,
        impls: 0,
        blanket_impls: 0,
        fns: 0,
        match_arms: 0,
        macros: 0,
    };

    /// Each fixture under fixture/volume, with the counts worked out by hand.
    const CASES: &[(&str, &str, Volume)] = &[
        (
            "comments",
            include_str!("fixture/volume/comments.rs"),
            Volume {
                lines: 10,
                comment_lines: 8,
                logic_tokens: 12,
                data_tokens: 7,
                data_items: 1,
                data_entries: 1,
                fns: 2,
                ..ONE
            },
        ),
        (
            "strings",
            include_str!("fixture/volume/strings.rs"),
            Volume {
                lines: 11,
                logic_tokens: 46,
                fns: 1,
                ..ONE
            },
        ),
        (
            "chars",
            include_str!("fixture/volume/chars.rs"),
            Volume {
                lines: 13,
                logic_tokens: 68,
                fns: 2,
                ..ONE
            },
        ),
        (
            "numbers",
            include_str!("fixture/volume/numbers.rs"),
            Volume {
                lines: 12,
                logic_tokens: 67,
                fns: 1,
                ..ONE
            },
        ),
        (
            "data",
            include_str!("fixture/volume/data.rs"),
            Volume {
                lines: 14,
                logic_tokens: 109,
                data_tokens: 102,
                data_items: 5,
                data_entries: 8,
                traits: 1,
                fns: 1,
                ..ONE
            },
        ),
        (
            "structure",
            include_str!("fixture/volume/structure.rs"),
            Volume {
                lines: 28,
                logic_tokens: 183,
                traits: 2,
                impls: 5,
                blanket_impls: 2,
                fns: 4,
                match_arms: 2,
                macros: 1,
                ..ONE
            },
        ),
        (
            "cfg_test",
            include_str!("fixture/volume/cfg_test.rs"),
            Volume {
                lines: 2,
                logic_tokens: 6,
                data_tokens: 7,
                data_items: 1,
                data_entries: 1,
                fns: 1,
                ..ONE
            },
        ),
        (
            "split",
            include_str!("fixture/volume/split.rs"),
            Volume {
                lines: 13,
                logic_tokens: 42,
                data_tokens: 18,
                data_items: 1,
                data_entries: 3,
                impls: 1,
                blanket_impls: 1,
                fns: 1,
                match_arms: 2,
                ..ONE
            },
        ),
    ];

    #[test]
    fn measures_each_fixture_as_counted_by_hand() {
        let differ: Vec<String> = CASES
            .iter()
            .filter(|(_, source, expected)| measure(source) != *expected)
            .map(|(name, source, expected)| {
                format!(
                    "{name}: measured {:?}, expected {expected:?}",
                    measure(source)
                )
            })
            .collect();
        assert!(differ.is_empty(), "{}", differ.join("\n"));
    }

    #[test]
    fn joining_lines_changes_only_the_line_count() {
        let split = measure(include_str!("fixture/volume/split.rs"));
        let joined = measure(include_str!("fixture/volume/joined.rs"));
        assert_eq!((split.lines, joined.lines), (13, 2));
        assert_eq!(Volume { lines: 0, ..split }, Volume { lines: 0, ..joined });
    }

    #[test]
    fn scopes_files_as_production_lines_does() {
        let game = |module: &str| Some(Row::Engine(1, Some(module.to_owned())));
        assert_eq!(row("crates/sm_game/src/lib.rs"), game("lib"));
        assert_eq!(
            row("crates/sm_game/src/actors/bosses/kraid.rs"),
            game("actors")
        );
        assert_eq!(
            row("crates/sm_data/src/table.rs"),
            Some(Row::Engine(0, None))
        );
        assert_eq!(row("crates/sm_app/src/main.rs"), Some(Row::Engine(3, None)));
        assert_eq!(
            row("crates/sm_game/src/verification/checks.rs"),
            Some(Row::Verification)
        );
        assert_eq!(row("crates/sm_tools/src/lib.rs"), Some(Row::Verification));
        assert_eq!(
            row("crates/sm_tools/src/bin/sm_pipeline/fixture/volume/data.rs"),
            None
        );
        assert_eq!(row("sm/src/verification/mod.rs"), Some(Row::Verification));
        assert_eq!(row("sm/src/cpu_infra.rs"), Some(Row::Oracle));
        assert_eq!(row("crates/sm_host/src/game/sm_a9.rs"), Some(Row::Oracle));
        assert_eq!(row("crates/sm_game/tests/replay.rs"), None);
        assert_eq!(row("crates/sm_game/build.rs"), None);
    }

    #[test]
    fn compares_against_the_ratchet_baseline() {
        let found = baseline(
            "BASELINE = {\n    \"engine_logic_tokens\": 40,\n  \"engine_traits\": 9,  # note\n}\n",
        );
        assert_eq!(found.get("engine_logic_tokens"), Some(&40));
        assert_eq!(found.get("engine_traits"), Some(&9));
        let mut rows = BTreeMap::new();
        rows.insert(Row::Engine(1, Some("x".into())), CASES[7].2);
        let (text, held) = compare(&rows, &found);
        assert!(!held, "{text}");
        assert!(
            text.starts_with("engine_logic_tokens: 42 (baseline 40, delta +2)\n"),
            "{text}"
        );
        assert!(
            text.contains("engine_data_tokens: 18 (no baseline)\n"),
            "{text}"
        );
    }
}
