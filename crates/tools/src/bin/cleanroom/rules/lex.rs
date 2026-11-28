#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Word,
    Num,
    Field,
    Lit,
    Life,
    Open,
    Close,
    Punct,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub text: String,
    pub line: usize,
    pub file: usize,
}

const OPS: &[&str] = &[
    "<<=", ">>=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=",
    "*=", "/=", "%=", "^=", "&=", "|=", "<<", ">>", "..",
];

struct Cursor<'a> {
    chars: Vec<char>,
    src: &'a str,
    offsets: Vec<usize>,
    at: usize,
    line: usize,
}

impl Cursor<'_> {
    fn peek(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.at.saturating_add(ahead)).copied()
    }

    fn bump(&mut self) {
        if self.peek(0) == Some('\n') {
            self.line = self.line.saturating_add(1);
        }
        self.at = self.at.saturating_add(1);
    }

    fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn slice(&self, from: usize) -> &str {
        let start = self.offsets.get(from).copied().unwrap_or(self.src.len());
        let end = self.offsets.get(self.at).copied().unwrap_or(self.src.len());
        self.src.get(start..end).unwrap_or_default()
    }

    fn starts_with(&self, text: &str) -> bool {
        text.chars()
            .enumerate()
            .all(|(i, c)| self.peek(i) == Some(c))
    }

    fn line_comment(&mut self) {
        while self.peek(0).is_some_and(|c| c != '\n') {
            self.bump();
        }
    }

    fn block_comment(&mut self) {
        self.bump_n(2);
        let mut depth = 1usize;
        while self.peek(0).is_some() && depth > 0 {
            if self.starts_with("/*") {
                depth = depth.saturating_add(1);
                self.bump_n(2);
            } else if self.starts_with("*/") {
                depth = depth.saturating_sub(1);
                self.bump_n(2);
            } else {
                self.bump();
            }
        }
    }

    fn quoted(&mut self) {
        self.bump();
        while let Some(c) = self.peek(0) {
            self.bump();
            if c == '\\' {
                self.bump();
            } else if c == '"' {
                return;
            }
        }
    }

    fn raw(&mut self, prefix: usize) {
        self.bump_n(prefix);
        let mut hashes = 0usize;
        while self.peek(0) == Some('#') {
            hashes = hashes.saturating_add(1);
            self.bump();
        }
        self.bump();
        while let Some(c) = self.peek(0) {
            self.bump();
            if c == '"' && (0..hashes).all(|n| self.peek(n) == Some('#')) {
                self.bump_n(hashes);
                return;
            }
        }
    }

    fn raw_prefix(&self) -> Option<usize> {
        let prefix = match (self.peek(0), self.peek(1)) {
            (Some('r'), _) => 1,
            (Some('b' | 'c'), Some('r')) => 2,
            _ => return None,
        };
        let mut at = prefix;
        while self.peek(at) == Some('#') {
            at = at.saturating_add(1);
        }
        (self.peek(at) == Some('"')).then_some(prefix)
    }

    fn char_literal(&self) -> Option<usize> {
        let quote = usize::from(self.peek(0) == Some('b'));
        if self.peek(quote) != Some('\'') {
            return None;
        }
        let value = quote.saturating_add(1);
        let close = if self.peek(value) == Some('\\') {
            let escape = value.saturating_add(1);
            match self.peek(escape)? {
                'x' => escape.saturating_add(3),
                'u' => {
                    let mut at = escape;
                    while self.peek(at).is_some_and(|c| c != '}') {
                        at = at.saturating_add(1);
                    }
                    at.saturating_add(1)
                }
                _ => escape.saturating_add(1),
            }
        } else {
            value.saturating_add(1)
        };
        (self.peek(close) == Some('\'')).then(|| close.saturating_add(1))
    }

    fn number(&mut self, field: bool) {
        let hex = self.starts_with("0x");
        let mut prev = ' ';
        while let Some(c) = self.peek(0) {
            let exponent_sign = matches!(c, '+' | '-') && matches!(prev, 'e' | 'E') && !hex;
            let decimal = c == '.' && !field && self.peek(1).is_some_and(|d| d.is_ascii_digit());
            if !(c.is_alphanumeric() || c == '_' || exponent_sign || decimal) {
                break;
            }
            prev = c;
            self.bump();
        }
    }

    fn word(&mut self) {
        while self
            .peek(0)
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            self.bump();
        }
    }
}

pub fn tokens(src: &str, file: usize) -> Vec<Token> {
    let mut cur = Cursor {
        chars: src.chars().collect(),
        src,
        offsets: src
            .char_indices()
            .map(|(offset, _)| offset)
            .chain([src.len()])
            .collect(),
        at: 0,
        line: 1,
    };
    let mut out: Vec<Token> = Vec::new();
    while let Some(c) = cur.peek(0) {
        let start = cur.at;
        let line = cur.line;
        let kind = if c.is_whitespace() {
            cur.bump();
            continue;
        } else if cur.starts_with("//") {
            cur.line_comment();
            continue;
        } else if cur.starts_with("/*") {
            cur.block_comment();
            continue;
        } else if let Some(prefix) = cur.raw_prefix() {
            cur.raw(prefix);
            Kind::Lit
        } else if c == '"' || (matches!(c, 'b' | 'c') && cur.peek(1) == Some('"')) {
            if c != '"' {
                cur.bump();
            }
            cur.quoted();
            Kind::Lit
        } else if let Some(len) = cur.char_literal() {
            cur.bump_n(len);
            Kind::Lit
        } else if c == '\'' {
            cur.bump();
            cur.word();
            Kind::Life
        } else if c.is_ascii_digit() {
            let field = out
                .last()
                .is_some_and(|t| t.kind == Kind::Punct && t.text == ".");
            cur.number(field);
            if field { Kind::Field } else { Kind::Num }
        } else if c.is_alphanumeric() || c == '_' {
            cur.word();
            Kind::Word
        } else if matches!(c, '(' | '[' | '{') {
            cur.bump();
            Kind::Open
        } else if matches!(c, ')' | ']' | '}') {
            cur.bump();
            Kind::Close
        } else {
            let op = OPS
                .iter()
                .find(|op| cur.starts_with(op))
                .map_or(1, |op| op.len());
            cur.bump_n(op);
            Kind::Punct
        };
        out.push(Token {
            kind,
            text: cur.slice(start).to_owned(),
            line,
            file,
        });
    }
    out
}

#[derive(Clone, Copy)]
pub struct Toks<'a>(pub &'a [Token]);

impl<'a> Toks<'a> {
    pub const fn len(self) -> usize {
        self.0.len()
    }

    pub fn get(self, i: usize) -> Option<&'a Token> {
        self.0.get(i)
    }

    pub fn text(self, i: usize) -> &'a str {
        self.0.get(i).map_or("", |t| t.text.as_str())
    }

    pub fn kind(self, i: usize) -> Option<Kind> {
        self.0.get(i).map(|t| t.kind)
    }

    pub fn is(self, i: usize, kind: Kind) -> bool {
        self.kind(i) == Some(kind)
    }

    pub fn punct(self, i: usize, text: &str) -> bool {
        self.is(i, Kind::Punct) && self.text(i) == text
    }

    pub fn line(self, i: usize) -> usize {
        self.0.get(i).map_or(0, |t| t.line)
    }

    pub fn after_group(self, from: usize) -> usize {
        let mut depth = 0usize;
        for i in from..self.len() {
            match self.kind(i) {
                Some(Kind::Open) => depth = depth.saturating_add(1),
                Some(Kind::Close) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return i.saturating_add(1);
                    }
                }
                _ => {}
            }
        }
        self.len()
    }

    pub fn after_item(self, from: usize) -> usize {
        let mut depth = 0usize;
        let mut top = "";
        for i in from..self.len() {
            match self.kind(i) {
                Some(Kind::Open) => {
                    if depth == 0 {
                        top = self.text(i);
                    }
                    depth = depth.saturating_add(1);
                }
                Some(Kind::Close) => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && top == "{" {
                        return i.saturating_add(1);
                    }
                }
                _ if depth == 0 && self.punct(i, ";") => return i.saturating_add(1),
                _ => {}
            }
        }
        self.len()
    }

    pub fn after_angles(self, from: usize) -> usize {
        let mut depth = 0isize;
        for i in from..self.len() {
            depth = match self.text(i) {
                "<" => depth.saturating_add(1),
                ">" => depth.saturating_sub(1),
                ">>" => depth.saturating_sub(2),
                _ => depth,
            };
            if depth <= 0 {
                return i.saturating_add(1);
            }
        }
        self.len()
    }

    pub fn cfg_test_at(self, i: usize) -> bool {
        ["#", "[", "cfg", "(", "test", ")", "]"]
            .iter()
            .enumerate()
            .all(|(k, t)| self.text(i.saturating_add(k)) == *t)
    }

    pub fn attribute_at(self, i: usize) -> Option<usize> {
        if self.text(i) != "#" {
            return None;
        }
        let next = i.saturating_add(1);
        match self.text(next) {
            "[" => Some(next),
            "!" if self.text(i.saturating_add(2)) == "[" => Some(i.saturating_add(2)),
            _ => None,
        }
    }
}

pub fn literal_path(token: &Token) -> Option<&str> {
    if token.kind != Kind::Lit {
        return None;
    }
    let text = token.text.as_str();
    let body = text.strip_prefix('r').unwrap_or(text).trim_matches('#');
    body.strip_prefix('"')?
        .strip_suffix('"')
        .filter(|inner| !inner.contains('"'))
}

#[cfg(test)]
mod tests {
    use super::{Kind, literal_path, tokens};

    fn kinds(src: &str) -> Vec<(Kind, String)> {
        tokens(src, 0)
            .into_iter()
            .map(|t| (t.kind, t.text))
            .collect()
    }

    #[test]
    fn comments_strings_and_chars_hide_their_contents() {
        let toks = kinds("a /* b /* c */ d */ \"e // f\" 'g' '\\n' r#\"h\"# // i\nj");
        let words: Vec<&str> = toks
            .iter()
            .filter(|(k, _)| *k == Kind::Word)
            .map(|(_, t)| t.as_str())
            .collect();
        assert_eq!(words, ["a", "j"]);
    }

    #[test]
    fn numbers_fields_and_operators_split_as_rust_does() {
        assert_eq!(
            kinds("t.2 1.5e-3 0..=15 x>>=2"),
            [
                (Kind::Word, "t".to_owned()),
                (Kind::Punct, ".".to_owned()),
                (Kind::Field, "2".to_owned()),
                (Kind::Num, "1.5e-3".to_owned()),
                (Kind::Num, "0".to_owned()),
                (Kind::Punct, "..=".to_owned()),
                (Kind::Num, "15".to_owned()),
                (Kind::Word, "x".to_owned()),
                (Kind::Punct, ">>=".to_owned()),
                (Kind::Num, "2".to_owned()),
            ]
        );
    }

    #[test]
    fn lifetimes_are_not_chars_and_lines_count() {
        let toks = tokens("fn f<'a>(\n'b' ) {}", 0);
        assert!(toks.iter().any(|t| t.kind == Kind::Life && t.text == "'a"));
        assert!(toks.iter().any(|t| t.kind == Kind::Lit && t.line == 2));
    }

    #[test]
    fn a_literal_path_is_a_plain_or_raw_string() {
        let toks = tokens("\"a/b.toml\" r#\"c.md\"# 'x' \"q\\\"\"", 0);
        let paths: Vec<Option<&str>> = toks.iter().map(literal_path).collect();
        assert_eq!(paths, [Some("a/b.toml"), Some("c.md"), None, None]);
    }
}
