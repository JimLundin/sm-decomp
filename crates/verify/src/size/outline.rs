//! A source's outline, for the report's structure section.
//!
//! What its module doc says it is, the items it declares with who can name them, its `use` trees
//! and the paths it names, read from the size lexer's tokens with the `#[cfg(test)]` items left
//! out.
//!
//! It reads the shapes rustfmt writes, not all of Rust: items inside a function's body, a
//! trait's or an `impl`'s associated items other than functions, and macro bodies are not
//! listed; a path is read where it is written, not where `use` would resolve it.

use std::ops::Range;

use super::{Token, after_attribute, group_end, lex, split_top, without_tests};

/// Who can name an item.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Visibility {
    /// No `pub`: its module and the modules inside it.
    Private,
    /// `pub(crate)`, `pub(super)` or `pub(in ...)`.
    Restricted,
    /// `pub`.
    Public,
}

/// What an item declares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A `struct`.
    Struct,
    /// An `enum`, with its variant count.
    Enum {
        /// How many variants it has.
        variants: usize,
    },
    /// A `union`.
    Union,
    /// A `trait`.
    Trait,
    /// A `type` alias.
    Alias,
    /// A function, free or in an inherent `impl`.
    Function,
    /// A `const`.
    Constant,
    /// A `static`.
    Static,
    /// A `mod`.
    Module {
        /// Whether its body is in the source (`mod name { ... }`), not a file of its own.
        inline: bool,
    },
}

/// A declared item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item<'a> {
    /// What it declares.
    pub kind: Kind,
    /// Its name.
    pub name: &'a str,
    /// Who can name it.
    pub visibility: Visibility,
    /// The inline modules (`mod name { ... }`) that hold it, outermost first.
    pub module: Vec<&'a str>,
    /// The type whose inherent `impl` holds it, by the type's last name.
    pub owner: Option<&'a str>,
    /// A function's signature, from its name to its body, on one line.
    pub signature: Option<String>,
}

/// An `impl` of a trait: what callers can do with a type beyond its own items.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TraitImpl {
    /// The trait, as written (`fmt::Display`, `From<Pose>`).
    pub trait_name: String,
    /// The type, as written.
    pub self_type: String,
}

/// One leaf of a `use` tree.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Use<'a> {
    /// The path to the leaf, its name last; a glob's path is its module's.
    pub path: Vec<&'a str>,
    /// Whether it imports everything its path holds (`*`).
    pub glob: bool,
    /// The name it takes (`as`), if not its own.
    pub alias: Option<&'a str>,
    /// Who can name what it imports: a `pub use` re-exports.
    pub visibility: Visibility,
}

impl<'a> Use<'a> {
    /// The name it brings in, `None` for a glob.
    #[must_use]
    pub fn name(&self) -> Option<&'a str> {
        if self.glob {
            None
        } else {
            self.alias.or_else(|| self.path.last().copied())
        }
    }
}

/// What a source declares, imports and names.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Outline<'a> {
    /// The first sentence of its module doc (`//!`).
    pub doc: Option<String>,
    /// Its items, in source order.
    pub items: Vec<Item<'a>>,
    /// Its trait impls, in source order.
    pub impls: Vec<TraitImpl>,
    /// Its `use` leaves, in source order.
    pub uses: Vec<Use<'a>>,
    /// Every path outside a `use` item and an attribute (`a::b::c`), each by its names, sorted,
    /// once each.
    pub paths: Vec<Vec<&'a str>>,
}

/// The outline of one source text.
#[must_use]
pub fn of(src: &str) -> Outline<'_> {
    let (tokens, spans): (Vec<Token<'_>>, Vec<Range<usize>>) =
        without_tests(&lex(src)).into_iter().unzip();
    let mut walk = Walk {
        src,
        tokens,
        spans,
        skipped: Vec::new(),
        module: Vec::new(),
        out: Outline {
            doc: doc(src),
            ..Outline::default()
        },
    };
    walk.scope(0, walk.tokens.len(), None);
    walk.out.paths = walk.paths();
    walk.out
}

/// The first sentence of the source's first `//!` paragraph.
fn doc(src: &str) -> Option<String> {
    let lines: Vec<&str> = src
        .lines()
        .map(str::trim_start)
        .skip_while(|line| !line.starts_with("//!"))
        .map_while(|line| line.strip_prefix("//!"))
        .map(str::trim)
        .take_while(|line| !line.is_empty())
        .collect();
    let paragraph = lines.join(" ");
    let sentence = paragraph
        .find(". ")
        .and_then(|at| paragraph.get(..=at))
        .unwrap_or(&paragraph);
    (!sentence.is_empty()).then(|| sentence.to_owned())
}

/// Source text on one line: its whitespace runs as one space, and the space rustfmt leaves inside
/// a bracket and before a trailing comma's bracket gone.
fn one_line(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    [
        (", )", ")"),
        (", >", ">"),
        (", }", "}"),
        ("( ", "("),
        (" )", ")"),
        ("{ ", "{"),
        (" }", "}"),
    ]
    .iter()
    .fold(joined, |text, (from, to)| text.replace(from, to))
}

/// The walk over a source's tokens.
struct Walk<'a> {
    src: &'a str,
    tokens: Vec<Token<'a>>,
    spans: Vec<Range<usize>>,
    /// The token ranges whose paths are not the source's own: `use` items and macro bodies.
    skipped: Vec<Range<usize>>,
    /// The inline modules the walk is in, outermost first.
    module: Vec<&'a str>,
    out: Outline<'a>,
}

impl<'a> Walk<'a> {
    fn word(&self, at: usize) -> Option<&'a str> {
        match self.tokens.get(at) {
            Some(&Token::Word(word)) => Some(word),
            Some(Token::Open(_) | Token::Close | Token::Punct(_) | Token::Literal(_)) | None => {
                None
            }
        }
    }

    /// The source text from the token at `first` through the token at `last`, on one line.
    fn text(&self, first: usize, last: usize) -> String {
        let start = self.spans.get(first).map_or(0, |span| span.start);
        let end = self.spans.get(last).map_or(start, |span| span.end);
        one_line(self.src.get(start..end).unwrap_or_default())
    }

    /// The first `{` (when `braces`) or `;` at or after `at` outside every bracket group, or `end`.
    fn body(&self, mut at: usize, end: usize, braces: bool) -> usize {
        while at < end {
            let Some(&token) = self.tokens.get(at) else {
                break;
            };
            at = match token {
                Token::Open('{') if braces => return at,
                Token::Punct(";") => return at,
                Token::Open(_) => group_end(&self.tokens, at),
                Token::Close | Token::Word(_) | Token::Punct(_) | Token::Literal(_) => {
                    at.saturating_add(1)
                }
            };
        }
        end
    }

    /// The index past the body at `at`: its group when it opens one, else its `;`.
    fn past(&self, at: usize) -> usize {
        if self.tokens.get(at) == Some(&Token::Open('{')) {
            group_end(&self.tokens, at)
        } else {
            at.saturating_add(1)
        }
    }

    /// The tokens inside the group that opens at `open`.
    fn inside(&self, open: usize) -> Range<usize> {
        open.saturating_add(1)..group_end(&self.tokens, open).saturating_sub(1)
    }

    /// The items from `at` to `end`, those in an inherent `impl` with its type as their owner.
    fn scope(&mut self, mut at: usize, end: usize, owner: Option<&'a str>) {
        while at < end {
            let after = self
                .tokens
                .get(at..)
                .and_then(after_attribute)
                .map(|rest| self.tokens.len().saturating_sub(rest.len()));
            at = after.unwrap_or_else(|| self.item(at, end, owner).max(at.saturating_add(1)));
        }
    }

    /// Reads the item at `at`, returning the index past it.
    fn item(&mut self, at: usize, end: usize, owner: Option<&'a str>) -> usize {
        let (visibility, mut at) = self.visibility(at);
        while let Some(after) = self.qualifier(at) {
            at = after;
        }
        let name = self.word(at.saturating_add(1));
        let module = self.module.clone();
        let declared = |kind, name| Item {
            kind,
            name,
            visibility,
            module: module.clone(),
            owner,
            signature: None,
        };
        match self.word(at) {
            Some("fn") => return self.function(at, end, visibility, owner),
            Some("impl") => return self.implementation(at, end),
            Some("use") => {
                let semicolon = self.body(at, end, false);
                self.skipped.push(at..semicolon);
                let tree = self.tokens.get(at.saturating_add(1)..semicolon);
                use_tree(
                    Vec::new(),
                    tree.unwrap_or_default(),
                    visibility,
                    &mut self.out.uses,
                );
                return semicolon.saturating_add(1);
            }
            Some("macro_rules") => {
                let body = self.body(at, end, true);
                let after = self.past(body);
                self.skipped.push(at..after);
                return after;
            }
            Some("mod") => {
                let body = self.body(at, end, true);
                let inline = self.tokens.get(body) == Some(&Token::Open('{'));
                self.out
                    .items
                    .extend(name.map(|name| declared(Kind::Module { inline }, name)));
                if let (true, Some(name)) = (inline, name) {
                    let inside = self.inside(body);
                    self.module.push(name);
                    self.scope(inside.start, inside.end, None);
                    self.module.pop();
                }
                return self.past(body);
            }
            Some("enum") => {
                let body = self.body(at, end, true);
                let variants = self.variants(body);
                self.out
                    .items
                    .extend(name.map(|name| declared(Kind::Enum { variants }, name)));
                return self.past(body);
            }
            _ => {}
        }
        let (kind, braces, name) = match self.word(at) {
            Some("struct") => (Some(Kind::Struct), true, name),
            Some("union") => (Some(Kind::Union), true, name),
            Some("trait") => (Some(Kind::Trait), true, name),
            Some("type") => (Some(Kind::Alias), false, name),
            Some("const") => (Some(Kind::Constant), false, name),
            Some("static") if name == Some("mut") => {
                (Some(Kind::Static), false, self.word(at.saturating_add(2)))
            }
            Some("static") => (Some(Kind::Static), false, name),
            _ => (None, true, None),
        };
        if let (Some(kind), Some(name)) = (kind, name.filter(|&name| name != "_")) {
            self.out.items.push(declared(kind, name));
        }
        self.past(self.body(at, end, braces))
    }

    /// The visibility at `at` and the index past it.
    fn visibility(&self, at: usize) -> (Visibility, usize) {
        if self.word(at) != Some("pub") {
            return (Visibility::Private, at);
        }
        let next = at.saturating_add(1);
        if self.tokens.get(next) == Some(&Token::Open('(')) {
            (Visibility::Restricted, group_end(&self.tokens, next))
        } else {
            (Visibility::Public, next)
        }
    }

    /// The index past a qualifier of the item at `at` (`const fn`, `async`, `unsafe`,
    /// `extern "C"`), if one starts there.
    fn qualifier(&self, at: usize) -> Option<usize> {
        let next = at.saturating_add(1);
        match self.word(at)? {
            "async" | "unsafe" | "default" => Some(next),
            "const" => matches!(self.word(next), Some("fn" | "async" | "unsafe" | "extern"))
                .then_some(next),
            "extern" => Some(
                if matches!(self.tokens.get(next), Some(Token::Literal(_))) {
                    next.saturating_add(1)
                } else {
                    next
                },
            ),
            _ => None,
        }
    }

    /// The function whose `fn` is at `at`, with its signature; the index past it.
    fn function(
        &mut self,
        at: usize,
        end: usize,
        visibility: Visibility,
        owner: Option<&'a str>,
    ) -> usize {
        let name_at = at.saturating_add(1);
        let body = self.body(name_at, end, true);
        if let Some(name) = self.word(name_at) {
            self.out.items.push(Item {
                kind: Kind::Function,
                name,
                visibility,
                module: self.module.clone(),
                owner,
                signature: Some(self.text(name_at, body.saturating_sub(1))),
            });
        }
        self.past(body)
    }

    /// The `impl` whose keyword is at `at`: a trait's is listed, an inherent one's functions
    /// are items owned by its type. The index past it.
    fn implementation(&mut self, at: usize, end: usize) -> usize {
        let body = self.body(at, end, true);
        let generics_end = self.after_generics(at.saturating_add(1), body);
        let mut depth = 0usize;
        let mut trait_end = None;
        let mut stop = body;
        let mut owner = None;
        for (index, &token) in self.tokens.iter().enumerate().take(body).skip(generics_end) {
            let outside = depth == 0;
            match token {
                Token::Punct("<") => depth = depth.saturating_add(1),
                Token::Punct(">") => depth = depth.saturating_sub(1),
                Token::Punct(">>") => depth = depth.saturating_sub(2),
                Token::Word("for") if outside => {
                    trait_end = Some(index);
                    owner = None;
                }
                Token::Word("where") if outside => {
                    stop = index;
                    break;
                }
                Token::Word(word) if outside && word != "mut" && word != "dyn" => {
                    owner = Some(word);
                }
                Token::Open(_)
                | Token::Close
                | Token::Word(_)
                | Token::Punct(_)
                | Token::Literal(_) => {}
            }
        }
        if let Some(for_at) = trait_end {
            self.out.impls.push(TraitImpl {
                trait_name: self.text(generics_end, for_at.saturating_sub(1)),
                self_type: self.text(for_at.saturating_add(1), stop.saturating_sub(1)),
            });
        } else if let (Some(owner), Some(Token::Open('{'))) = (owner, self.tokens.get(body)) {
            let inside = self.inside(body);
            self.scope(inside.start, inside.end, Some(owner));
        }
        self.past(body)
    }

    /// The index past the generic parameters `<...>` at `at`, or `at` without them.
    fn after_generics(&self, at: usize, end: usize) -> usize {
        if self.tokens.get(at) != Some(&Token::Punct("<")) {
            return at;
        }
        let mut depth = 0usize;
        for (index, &token) in self.tokens.iter().enumerate().take(end).skip(at) {
            match token {
                Token::Punct("<") => depth = depth.saturating_add(1),
                Token::Punct(">") => depth = depth.saturating_sub(1),
                Token::Punct(">>") => depth = depth.saturating_sub(2),
                Token::Open(_)
                | Token::Close
                | Token::Word(_)
                | Token::Punct(_)
                | Token::Literal(_) => {}
            }
            if depth == 0 {
                return index.saturating_add(1);
            }
        }
        end
    }

    /// How many variants the enum body at `open` holds.
    fn variants(&self, open: usize) -> usize {
        let inside = self.inside(open);
        let tokens = self.tokens.get(inside).unwrap_or_default();
        split_top(tokens, Token::Punct(","))
            .into_iter()
            .filter(|variant| {
                let mut rest = *variant;
                while let Some(after) = after_attribute(rest) {
                    rest = after;
                }
                !rest.is_empty()
            })
            .count()
    }

    /// Every path outside the skipped ranges, sorted, once each.
    fn paths(&self) -> Vec<Vec<&'a str>> {
        let mut paths: Vec<Vec<&'a str>> = Vec::new();
        // An attribute's paths (`clippy::all`) name lints and tools, not code.
        let attributes: Vec<Range<usize>> = (0..self.tokens.len())
            .filter_map(|at| {
                let after = after_attribute(self.tokens.get(at..)?)?;
                Some(at..self.tokens.len().saturating_sub(after.len()))
            })
            .collect();
        for (at, pair) in self.tokens.windows(2).enumerate() {
            let &[Token::Word(head), Token::Punct("::")] = pair else {
                continue;
            };
            let continues = at
                .checked_sub(1)
                .and_then(|before| self.tokens.get(before))
                .is_some_and(|&token| token == Token::Punct("::"));
            let skipped = self.skipped.iter().chain(&attributes);
            if continues || skipped.into_iter().any(|range| range.contains(&at)) {
                continue;
            }
            let mut path = vec![head];
            let mut next = at.saturating_add(1);
            while let (Some(Token::Punct("::")), Some(word)) =
                (self.tokens.get(next), self.word(next.saturating_add(1)))
            {
                path.push(word);
                next = next.saturating_add(2);
            }
            paths.push(path);
        }
        paths.sort_unstable();
        paths.dedup();
        paths
    }
}

/// The leaves of the `use` tree `tokens`, under `prefix`.
fn use_tree<'a>(
    mut path: Vec<&'a str>,
    tokens: &[Token<'a>],
    visibility: Visibility,
    out: &mut Vec<Use<'a>>,
) {
    let mut rest = tokens;
    let leaf = |path: Vec<&'a str>, glob, alias| Use {
        path,
        glob,
        alias,
        visibility,
    };
    loop {
        match rest {
            [Token::Punct("::"), tail @ ..] => rest = tail,
            [Token::Word(word), Token::Punct("::"), tail @ ..] => {
                path.push(word);
                rest = tail;
            }
            [Token::Word(word), tail @ ..] => {
                if *word != "self" {
                    path.push(word);
                }
                let alias = match tail {
                    [Token::Word("as"), Token::Word(alias), ..] => Some(*alias),
                    _ => None,
                };
                out.push(leaf(path, false, alias));
                return;
            }
            [Token::Punct("*"), ..] => {
                out.push(leaf(path, true, None));
                return;
            }
            [Token::Open('{'), ..] => {
                let close = group_end(rest, 0).saturating_sub(1);
                let inner = rest.get(1..close).unwrap_or_default();
                for part in split_top(inner, Token::Punct(",")) {
                    if !part.is_empty() {
                        use_tree(path.clone(), part, visibility, out);
                    }
                }
                return;
            }
            _ => return,
        }
    }
}

#[cfg(test)]
mod tests;
