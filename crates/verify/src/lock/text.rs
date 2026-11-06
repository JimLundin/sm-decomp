//! The TOML subset a format-2 lock is written in: one `key = value` per line, `[[row]]`
//! section heads, `#` comments and blank lines. Values are strings without escapes,
//! unsigned integers, booleans, arrays and inline tables. The writer
//! (`oracle::relock`) emits nothing else, so anything else is an error rather than a
//! guess.

use super::ErrorKind;

/// One parsed value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Int(u64),
    Bool(bool),
    Array(Vec<Self>),
    /// An inline table's pairs, in the order written.
    Table(Vec<(String, Self)>),
}

impl Value {
    /// What the value is, for a type error.
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Str(_) => "string",
            Self::Int(_) => "integer",
            Self::Bool(_) => "boolean",
            Self::Array(_) => "array",
            Self::Table(_) => "table",
        }
    }
}

/// One meaningful line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    /// `[[name]]`.
    ArrayHead(String),
    /// `key = value`; a dotted key such as `streams.vram` keeps its dots.
    Pair { key: String, value: Value },
}

/// The line, or `None` for a blank or comment line.
pub fn line(text: &str) -> Result<Option<Line>, ErrorKind> {
    let mut cursor = Cursor { rest: text };
    cursor.skip_blank();
    if cursor.at_end() {
        return Ok(None);
    }
    let parsed = if cursor.eat_str("[[") {
        let name = cursor.bare_key()?;
        if !cursor.eat_str("]]") {
            return Err(ErrorKind::Syntax("a section head ends with `]]`"));
        }
        Line::ArrayHead(name)
    } else {
        let key = cursor.key()?;
        cursor.skip_blank();
        if !cursor.eat('=') {
            return Err(ErrorKind::Syntax("a key is followed by `=`"));
        }
        let value = cursor.value()?;
        Line::Pair { key, value }
    };
    cursor.skip_blank();
    if cursor.at_end() {
        Ok(Some(parsed))
    } else {
        Err(ErrorKind::Syntax("unexpected text after the value"))
    }
}

struct Cursor<'a> {
    rest: &'a str,
}

const fn is_bare(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
}

impl Cursor<'_> {
    /// Skips spaces and tabs, and a comment through the end of the line.
    fn skip_blank(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t']);
        if self.rest.starts_with('#') {
            self.rest = "";
        }
    }

    const fn at_end(&self) -> bool {
        self.rest.is_empty()
    }

    fn peek(&self) -> Option<char> {
        self.rest.chars().next()
    }

    fn eat(&mut self, character: char) -> bool {
        self.rest
            .strip_prefix(character)
            .map(|rest| self.rest = rest)
            .is_some()
    }

    fn eat_str(&mut self, prefix: &str) -> bool {
        self.rest
            .strip_prefix(prefix)
            .map(|rest| self.rest = rest)
            .is_some()
    }

    fn bare_key(&mut self) -> Result<String, ErrorKind> {
        let end = self.rest.find(|character| !is_bare(character));
        let (key, rest) = self.rest.split_at(end.unwrap_or(self.rest.len()));
        if key.is_empty() {
            return Err(ErrorKind::Syntax("expected a key"));
        }
        self.rest = rest;
        Ok(key.to_owned())
    }

    fn key(&mut self) -> Result<String, ErrorKind> {
        if self.peek() == Some('"') {
            self.string()
        } else {
            self.bare_key()
        }
    }

    fn string(&mut self) -> Result<String, ErrorKind> {
        if !self.eat('"') {
            return Err(ErrorKind::Syntax("expected `\"`"));
        }
        let Some((body, rest)) = self.rest.split_once('"') else {
            return Err(ErrorKind::Syntax("unterminated string"));
        };
        // The writer escapes nothing, so a backslash or control character was not written
        // by it.
        if body.contains(|character: char| character == '\\' || character.is_control()) {
            return Err(ErrorKind::Syntax(
                "strings hold no escapes or control characters",
            ));
        }
        self.rest = rest;
        Ok(body.to_owned())
    }

    fn value(&mut self) -> Result<Value, ErrorKind> {
        self.skip_blank();
        match self.peek() {
            Some('"') => self.string().map(Value::Str),
            Some('[') => self.array(),
            Some('{') => self.table(),
            Some(character) if character.is_ascii_digit() => self.integer(),
            Some(_) if self.eat_str("true") => Ok(Value::Bool(true)),
            Some(_) if self.eat_str("false") => Ok(Value::Bool(false)),
            Some(_) | None => Err(ErrorKind::Syntax("expected a value")),
        }
    }

    fn integer(&mut self) -> Result<Value, ErrorKind> {
        let end = self
            .rest
            .find(|character: char| !character.is_ascii_digit());
        let (digits, rest) = self.rest.split_at(end.unwrap_or(self.rest.len()));
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(ErrorKind::Syntax("integers have no leading zero"));
        }
        let number = digits
            .parse()
            .map_err(|_| ErrorKind::Syntax("integer out of range"))?;
        self.rest = rest;
        Ok(Value::Int(number))
    }

    /// The items of `[...]` or the pairs of `{...}` up to `close`, comma separated.
    fn items<T>(
        &mut self,
        close: char,
        mut item: impl FnMut(&mut Self) -> Result<T, ErrorKind>,
    ) -> Result<Vec<T>, ErrorKind> {
        let mut items = Vec::new();
        self.skip_blank();
        if self.eat(close) {
            return Ok(items);
        }
        loop {
            items.push(item(self)?);
            self.skip_blank();
            if self.eat(close) {
                return Ok(items);
            }
            if !self.eat(',') {
                return Err(ErrorKind::Syntax("expected `,` or the closing bracket"));
            }
            self.skip_blank();
        }
    }

    fn array(&mut self) -> Result<Value, ErrorKind> {
        self.eat('[');
        self.items(']', Self::value).map(Value::Array)
    }

    fn table(&mut self) -> Result<Value, ErrorKind> {
        self.eat('{');
        self.items('}', |cursor| {
            let key = cursor.key()?;
            cursor.skip_blank();
            if !cursor.eat('=') {
                return Err(ErrorKind::Syntax("a key is followed by `=`"));
            }
            Ok((key, cursor.value()?))
        })
        .map(Value::Table)
    }
}
