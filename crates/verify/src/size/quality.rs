//! The score's two quality terms (Jim, 2026-11-07, on the quality-score review).
//!
//! **Depth (Q1).** A logic token weighs `1 + 0.25 × d` times its weight, `d` the number of
//! blocks around it, inside a function body, that nest: the block of an `if`, an `else`, a
//! `for`, `while` or `loop` and a let-else; a closure's body, a block or an expression; and a
//! `match` arm written as a block (`=> {`). A `match`'s own braces, its expression arms and a
//! guard's `if` don't nest, so a flat dispatch stays at depth 0.
//!
//! **Clumps (Q2).** Values that travel together want a type, and pay where they recur:
//! - A pair of parameter types other than `bool` that is in three or more signatures loses the
//!   signature's half on those parameters in each of them: their tokens and comma weigh one. A
//!   parameter's type is the first name it holds (`&mut Mover` is `Mover`, `(u16, u16)` is
//!   `u16`), primitives included, so a type can't dodge the term by becoming a `u16`.
//! - A pair of argument expressions (a call's arguments or a struct literal's field values) that
//!   recurs at three or more sites pays the shorter expression's tokens again, at one each, at
//!   every site after the second. A struct built only to carry arguments pays as the arguments
//!   did. An expression that holds a call or no name (a literal) varies from site to site and is
//!   left out, as is `..base`.
//!
//! Clumps are counted over all the sources measured together, in order.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::{Token, counted, group_end, signature_body};

/// What the quality terms know of one token.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Context {
    /// Its nesting depth (Q1).
    pub depth: usize,
    /// Whether it belongs to a parameter whose type is in a clump (Q2).
    pub clump: bool,
}

/// A function's parameters and body, as token ranges of its source.
struct Function<'a> {
    params: Vec<Param<'a>>,
    body: Option<Range<usize>>,
}

/// A parameter other than `self`: the type it names, and its tokens with its comma.
struct Param<'a> {
    ty: &'a str,
    tokens: Range<usize>,
}

/// A call or struct literal whose repeated argument pairs pay again (Q2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pay {
    /// The index of the token that opens it.
    pub at: usize,
    /// The tokens it pays again.
    pub tokens: usize,
}

/// For each source, its tokens' contexts and the calls and struct literals that pay again.
pub fn of<'t, 'a>(sources: &'t [Vec<Token<'a>>]) -> Vec<(Vec<Context>, Vec<Pay>)> {
    let functions: Vec<Vec<Function<'a>>> = sources.iter().map(|t| functions(t)).collect();
    let structs: BTreeSet<&'a str> = sources.iter().flat_map(|t| structs(t)).collect();
    let clumps = type_clumps(functions.iter().flatten());
    let mut seen: BTreeMap<(&'t [Token<'a>], &'t [Token<'a>]), usize> = BTreeMap::new();
    sources
        .iter()
        .zip(&functions)
        .map(|(tokens, functions)| {
            let mut contexts = vec![Context::default(); tokens.len()];
            let mut repeated = Vec::new();
            for body in outermost(functions) {
                nest(tokens, body.clone(), &mut contexts);
                for (at, site) in body.filter_map(|at| site(tokens, at, &structs).map(|s| (at, s)))
                {
                    repeated.push(Pay {
                        at,
                        tokens: pay(site, &mut seen),
                    });
                }
            }
            for function in functions {
                mark_clumps(function, &clumps, &mut contexts);
            }
            (contexts, repeated)
        })
        .collect()
}

/// The functions of a source: each `fn` followed by its name.
fn functions<'a>(tokens: &[Token<'a>]) -> Vec<Function<'a>> {
    tokens
        .iter()
        .enumerate()
        .filter(|&(at, &token)| {
            token == Token::Word("fn")
                && matches!(tokens.get(at.saturating_add(1)), Some(Token::Word(_)))
        })
        .filter_map(|(at, _)| function(tokens, at))
        .collect()
}

/// The function whose `fn` is at `at`.
fn function<'a>(tokens: &[Token<'a>], at: usize) -> Option<Function<'a>> {
    let mut open = at.checked_add(2)?;
    if tokens.get(open) == Some(&Token::Punct("<")) {
        open = after_angles(tokens, open);
    }
    if tokens.get(open) != Some(&Token::Open('(')) {
        return None;
    }
    let inner = open.saturating_add(1)..group_end(tokens, open).saturating_sub(1);
    let params = parts(tokens, inner, true)
        .into_iter()
        .filter_map(|part| param(tokens, part))
        .collect();
    let body_at = tokens
        .len()
        .saturating_sub(signature_body(tokens.get(at.saturating_add(1)..)?).len());
    let body = (tokens.get(body_at) == Some(&Token::Open('{')))
        .then(|| body_at..group_end(tokens, body_at));
    Some(Function { params, body })
}

/// The parameter whose tokens are `part`, unless it is `self`.
fn param<'a>(tokens: &[Token<'a>], part: Range<usize>) -> Option<Param<'a>> {
    let span = tokens.get(part.clone())?;
    let colon = span.iter().position(|&t| t == Token::Punct(":"))?;
    let ty = span.get(colon.saturating_add(1)..)?.iter().find_map(|&t| {
        if let Token::Word(word) = t
            && !matches!(word, "mut" | "dyn" | "impl" | "static")
        {
            Some(word)
        } else {
            None
        }
    })?;
    let end = if tokens.get(part.end) == Some(&Token::Punct(",")) {
        part.end.saturating_add(1)
    } else {
        part.end
    };
    Some(Param {
        ty,
        tokens: part.start..end,
    })
}

/// The index after the `>` that closes the `<` at `open`.
fn after_angles(tokens: &[Token<'_>], open: usize) -> usize {
    let mut depth = 0usize;
    for (at, &token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::Punct("<") => depth = depth.saturating_add(1),
            Token::Punct(">") => depth = depth.saturating_sub(1),
            Token::Punct(">>") => depth = depth.saturating_sub(2),
            Token::Open(_)
            | Token::Close
            | Token::Word(_)
            | Token::Punct(_)
            | Token::Literal(_) => {
                continue;
            }
        }
        if depth == 0 {
            return at.saturating_add(1);
        }
    }
    tokens.len()
}

/// The depth-0, comma-separated parts of `tokens[inner]`, commas left out. With `angles`, `<`
/// and `>` nest too, as in a parameter's type.
fn parts(tokens: &[Token<'_>], inner: Range<usize>, angles: bool) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut start = inner.start;
    let (mut depth, mut angle) = (0usize, 0usize);
    for (at, &token) in tokens.iter().enumerate().take(inner.end).skip(inner.start) {
        match token {
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => depth = depth.saturating_sub(1),
            Token::Punct("<") if angles => angle = angle.saturating_add(1),
            Token::Punct(">") if angles => angle = angle.saturating_sub(1),
            Token::Punct(">>") if angles => angle = angle.saturating_sub(2),
            Token::Punct(",") if depth == 0 && angle == 0 => {
                out.push(start..at);
                start = at.saturating_add(1);
            }
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    if start < inner.end {
        out.push(start..inner.end);
    }
    out
}

/// The names of the structs a source declares.
fn structs<'a>(tokens: &[Token<'a>]) -> Vec<&'a str> {
    tokens
        .windows(2)
        .filter_map(|pair| match pair {
            [Token::Word("struct"), Token::Word(name)] => Some(*name),
            _ => None,
        })
        .collect()
}

/// The bodies of the functions that are not inside another's body.
fn outermost(functions: &[Function<'_>]) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    for body in functions.iter().filter_map(|f| f.body.clone()) {
        if out.last().is_none_or(|last| body.start >= last.end) {
            out.push(body);
        }
    }
    out
}

/// Each pair of distinct items of a sorted, deduplicated list, in order.
fn pairs<T: Copy>(items: &[T]) -> Vec<(T, T)> {
    items
        .iter()
        .enumerate()
        .flat_map(|(at, &a)| {
            items
                .iter()
                .skip(at.saturating_add(1))
                .map(move |&b| (a, b))
        })
        .collect()
}

/// A function's parameter types, but `bool` (a flag, which `max-fn-params-bools` gates), each
/// once and sorted.
fn types<'a>(function: &Function<'a>) -> Vec<&'a str> {
    let set: BTreeSet<&str> = function
        .params
        .iter()
        .map(|p| p.ty)
        .filter(|&ty| ty != "bool")
        .collect();
    set.into_iter().collect()
}

/// The pairs of parameter types in three or more signatures.
fn type_clumps<'f, 'a: 'f>(
    functions: impl Iterator<Item = &'f Function<'a>>,
) -> BTreeSet<(&'a str, &'a str)> {
    let mut signatures: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for function in functions {
        for pair in pairs(&types(function)) {
            let n = signatures.entry(pair).or_default();
            *n = n.saturating_add(1);
        }
    }
    signatures
        .into_iter()
        .filter(|&(_, n)| n >= 3)
        .map(|(pair, _)| pair)
        .collect()
}

/// Marks the parameters of `function` whose type is in a clump it holds.
fn mark_clumps(function: &Function<'_>, clumps: &BTreeSet<(&str, &str)>, contexts: &mut [Context]) {
    let charged: BTreeSet<&str> = pairs(&types(function))
        .into_iter()
        .filter(|pair| clumps.contains(pair))
        .flat_map(<[_; 2]>::from)
        .collect();
    for param in function.params.iter().filter(|p| charged.contains(p.ty)) {
        for context in contexts.get_mut(param.tokens.clone()).into_iter().flatten() {
            context.clump = true;
        }
    }
}

/// Keywords that stand before a `(` without calling anything.
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "dyn",
];

/// The argument expressions of the call or struct literal at `at`, if one is there: a `(` after
/// a function's or a method's name, or a declared struct's name before its `{` in an
/// expression.
fn site<'t, 'a>(
    tokens: &'t [Token<'a>],
    at: usize,
    structs: &BTreeSet<&str>,
) -> Option<Vec<&'t [Token<'a>]>> {
    let previous = at.checked_sub(1).and_then(|p| tokens.get(p)).copied();
    match *tokens.get(at)? {
        Token::Open('(') => {
            let Some(Token::Word(name)) = previous else {
                return None;
            };
            let called = name.starts_with(|c: char| c.is_lowercase() || c == '_')
                && !KEYWORDS.contains(&name)
                && at.checked_sub(2).and_then(|p| tokens.get(p)) != Some(&Token::Word("fn"));
            called.then(|| arguments(tokens, at, false))
        }
        Token::Word(name) if structs.contains(name) => {
            let open = at.saturating_add(1);
            let built = tokens.get(open) == Some(&Token::Open('{'))
                && !matches!(
                    previous,
                    Some(Token::Word("struct" | "impl" | "for" | "let") | Token::Punct("->" | ":"))
                )
                && !matches!(
                    tokens.get(group_end(tokens, open)),
                    Some(Token::Punct("=" | "=>"))
                );
            built.then(|| arguments(tokens, open, true))
        }
        Token::Open(_) | Token::Close | Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {
            None
        }
    }
}

/// The expressions in the bracket group at `open` that can recur: each part, or with `fields`
/// each field's value (after its `:`, or the whole shorthand field).
fn arguments<'t, 'a>(tokens: &'t [Token<'a>], open: usize, fields: bool) -> Vec<&'t [Token<'a>]> {
    let inner = open.saturating_add(1)..group_end(tokens, open).saturating_sub(1);
    parts(tokens, inner, false)
        .into_iter()
        .filter_map(|part| tokens.get(part))
        .map(|part| {
            let colon = part.iter().position(|&t| t == Token::Punct(":"));
            match colon {
                Some(colon) if fields => part.get(colon.saturating_add(1)..).unwrap_or_default(),
                Some(_) | None => part,
            }
        })
        .filter(|expression| recurs(expression))
        .collect()
}

/// Whether an argument expression can recur as itself: it names something, calls nothing and
/// is not a struct update's `..base`.
fn recurs(expression: &[Token<'_>]) -> bool {
    expression.iter().any(|t| matches!(t, Token::Word(_)))
        && !expression.contains(&Token::Open('('))
        && expression.first() != Some(&Token::Punct(".."))
}

/// The tokens a site's argument pairs pay again: for each pair seen at two sites before, the
/// shorter expression's tokens.
fn pay<'t, 'a>(
    mut site: Vec<&'t [Token<'a>]>,
    seen: &mut BTreeMap<(&'t [Token<'a>], &'t [Token<'a>]), usize>,
) -> usize {
    site.sort_unstable();
    site.dedup();
    pairs(&site)
        .into_iter()
        .map(|(a, b)| {
            let n = seen.entry((a, b)).or_default();
            *n = n.saturating_add(1);
            if *n >= 3 {
                counted(a).min(counted(b))
            } else {
                0
            }
        })
        .fold(0, usize::saturating_add)
}

/// The open brackets of a body and the blocks that nest (Q1).
#[derive(Default)]
struct Nesting {
    open: Vec<Bracket>,
    /// The bracket depths at which an `if`, an `else` or a loop waits for its block.
    waiting: Vec<usize>,
    depth: usize,
}

/// One open bracket: whether it nests, and the expression closures opened in it and still open.
#[derive(Clone, Copy)]
struct Bracket {
    nests: bool,
    closures: usize,
}

impl Nesting {
    /// Ends the expression closures open at this bracket depth: a `,`, a `;` or the closing
    /// bracket ends them.
    fn end_closures(&mut self) {
        if let Some(bracket) = self.open.last_mut() {
            self.depth = self.depth.saturating_sub(bracket.closures);
            bracket.closures = 0;
        }
    }

    fn close(&mut self) {
        self.end_closures();
        if self.open.pop().is_some_and(|b| b.nests) {
            self.depth = self.depth.saturating_sub(1);
        }
        let level = self.open.len();
        self.waiting.retain(|&w| w <= level);
    }

    /// Opens a bracket after `previous`: a block that a keyword waits for, a closure's block or
    /// a block arm nests.
    fn open(&mut self, bracket: char, previous: Option<Token<'_>>) {
        let level = self.open.len();
        let awaited = bracket == '{' && self.waiting.last() == Some(&level);
        if awaited {
            self.waiting.pop();
        }
        let nests = awaited
            || (bracket == '{' && matches!(previous, Some(Token::Punct("|" | "||" | "=>"))));
        if nests {
            self.depth = self.depth.saturating_add(1);
        }
        self.open.push(Bracket { nests, closures: 0 });
    }

    fn wait(&mut self) {
        self.waiting.push(self.open.len());
    }

    fn expression_closure(&mut self) {
        if let Some(bracket) = self.open.last_mut() {
            bracket.closures = bracket.closures.saturating_add(1);
            self.depth = self.depth.saturating_add(1);
        }
    }
}

/// Sets each token's depth (Q1) in the function body `tokens[body]`.
fn nest(tokens: &[Token<'_>], body: Range<usize>, contexts: &mut [Context]) {
    let mut nesting = Nesting::default();
    let mut at = body.start;
    while at < body.end {
        let Some(&token) = tokens.get(at) else {
            break;
        };
        let previous = at.checked_sub(1).and_then(|p| tokens.get(p)).copied();
        let after = tokens.get(at.saturating_add(1)..).unwrap_or_default();
        match token {
            Token::Punct("," | ";") => nesting.end_closures(),
            Token::Close => nesting.close(),
            Token::Open(_) | Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
        set_depth(contexts, at..at.saturating_add(1), nesting.depth);
        match token {
            Token::Word("if") if !is_guard(after) => nesting.wait(),
            Token::Word("else") if after.first() != Some(&Token::Word("if")) => nesting.wait(),
            Token::Word("while" | "for" | "loop") => nesting.wait(),
            Token::Open(bracket) => nesting.open(bracket, previous),
            Token::Punct(bar @ ("|" | "||")) if opens_closure(previous) => {
                let end = if bar == "|" {
                    closing_bar(tokens, at)
                } else {
                    at
                };
                set_depth(contexts, at..end.saturating_add(1), nesting.depth);
                if tokens.get(end.saturating_add(1)) != Some(&Token::Open('{')) {
                    nesting.expression_closure();
                }
                at = end;
            }
            Token::Close | Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
        at = at.saturating_add(1);
    }
}

fn set_depth(contexts: &mut [Context], tokens: Range<usize>, depth: usize) {
    for context in contexts.get_mut(tokens).into_iter().flatten() {
        context.depth = depth;
    }
}

/// Whether the `if` that `after` follows is a match arm's guard: its `=>` comes before a block.
fn is_guard(after: &[Token<'_>]) -> bool {
    let mut depth = 0usize;
    for &token in after {
        match token {
            Token::Open('{') | Token::Punct(";") if depth == 0 => return false,
            Token::Punct("=>") if depth == 0 => return true,
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => {
                if depth == 0 {
                    return false;
                }
                depth = depth.saturating_sub(1);
            }
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    false
}

/// Whether a `|` or `||` after `previous` opens a closure rather than an operator or a pattern.
fn opens_closure(previous: Option<Token<'_>>) -> bool {
    matches!(
        previous,
        None | Some(
            Token::Open(_)
                | Token::Punct("," | "=" | "=>" | ";" | ":")
                | Token::Word("move" | "return")
        )
    )
}

/// The index of the `|` that closes a closure's parameters opened at `at`, or `at`.
fn closing_bar(tokens: &[Token<'_>], at: usize) -> usize {
    let mut depth = 0usize;
    for (end, &token) in tokens.iter().enumerate().skip(at.saturating_add(1)) {
        match token {
            Token::Punct("|") if depth == 0 => return end,
            Token::Open(_) => depth = depth.saturating_add(1),
            Token::Close => depth = depth.saturating_sub(1),
            Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {}
        }
    }
    at
}
