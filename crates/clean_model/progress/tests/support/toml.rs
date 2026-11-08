//! A reader for the part of TOML the vector file uses: full-line comments, `[[vector]]` headers,
//! and one `key = value` per line, where a value is a string, a whole number, a bool, an array or
//! an inline table. Anything else in the file stops the test.

/// A value of the file.
#[derive(Clone, Debug)]
pub enum Value {
    /// A string, without escapes.
    Str(String),
    /// A whole number.
    Int(i64),
    /// `true` or `false`.
    Bool(bool),
    /// `[a, b, ...]`.
    Array(Vec<Self>),
    /// `{ key = value, ... }`, its keys in the file's order.
    Table(Table),
}

/// A table's keys and values, in the file's order.
pub type Table = Vec<(String, Value)>;

impl Value {
    /// The string, or the test stops.
    pub fn str(&self) -> &str {
        let Self::Str(s) = self else {
            panic!("expected a string, found {self:?}")
        };
        s
    }

    /// The whole number as 16 bits, or the test stops.
    pub fn u16(&self) -> u16 {
        let Self::Int(n) = self else {
            panic!("expected a whole number, found {self:?}")
        };
        u16::try_from(*n).unwrap()
    }

    /// The bool, or the test stops.
    pub fn bool(&self) -> bool {
        let Self::Bool(b) = self else {
            panic!("expected a bool, found {self:?}")
        };
        *b
    }

    /// The array, or the test stops.
    pub fn array(&self) -> &[Self] {
        let Self::Array(items) = self else {
            panic!("expected an array, found {self:?}")
        };
        items
    }

    /// The table, or the test stops.
    pub fn table(&self) -> &Table {
        let Self::Table(table) = self else {
            panic!("expected a table, found {self:?}")
        };
        table
    }
}

/// Every `[[vector]]` of the file, as a table.
pub fn read_vectors(text: &str) -> Vec<Table> {
    let mut vectors: Vec<Table> = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[vector]]" {
            vectors.push(Table::new());
            continue;
        }
        let vector = vectors
            .last_mut()
            .expect("a key before the first [[vector]]");
        let mut parser = Parser { rest: line };
        let entry = parser.key_value();
        parser.finish();
        vector.push(entry);
    }
    vectors
}

/// What is left of the line being read.
struct Parser<'a> {
    rest: &'a str,
}

impl Parser<'_> {
    fn skip_space(&mut self) {
        self.rest = self.rest.trim_start();
    }

    fn eat(&mut self, c: char) -> bool {
        self.skip_space();
        match self.rest.strip_prefix(c) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }

    fn need(&mut self, c: char) {
        assert!(self.eat(c), "expected {c:?} at {:?}", self.rest);
    }

    fn word(&mut self) -> &str {
        self.skip_space();
        let end = self
            .rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(self.rest.len());
        assert!(end > 0, "expected a key or a value at {:?}", self.rest);
        let (word, rest) = self.rest.split_at(end);
        self.rest = rest;
        word
    }

    fn key_value(&mut self) -> (String, Value) {
        let key = self.word().to_owned();
        self.need('=');
        (key, self.value())
    }

    fn value(&mut self) -> Value {
        self.skip_space();
        match self.rest.chars().next() {
            Some('"') => self.string(),
            Some('[') => Value::Array(self.array()),
            Some('{') => Value::Table(self.table()),
            _ => self.scalar(),
        }
    }

    fn string(&mut self) -> Value {
        self.need('"');
        let end = self.rest.find('"').expect("an unclosed string");
        let (s, rest) = self.rest.split_at(end);
        self.rest = &rest[1..];
        Value::Str(s.to_owned())
    }

    fn array(&mut self) -> Vec<Value> {
        self.need('[');
        let mut items = Vec::new();
        while !self.eat(']') {
            items.push(self.value());
            if !self.eat(',') {
                self.need(']');
                break;
            }
        }
        items
    }

    fn table(&mut self) -> Table {
        self.need('{');
        let mut table = Table::new();
        while !self.eat('}') {
            table.push(self.key_value());
            if !self.eat(',') {
                self.need('}');
                break;
            }
        }
        table
    }

    fn scalar(&mut self) -> Value {
        match self.word() {
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            number => Value::Int(number.parse().expect("a whole number")),
        }
    }

    fn finish(&mut self) {
        self.skip_space();
        assert!(self.rest.is_empty(), "unread text {:?}", self.rest);
    }
}

/// A table being read key by key: each key is taken once, and `done` stops the test on any key
/// left untaken, so no key of the file is skipped.
pub struct Fields {
    rest: Table,
    context: String,
}

impl Fields {
    /// The fields of `table`, named by `context` in a failure.
    pub fn new(table: &Table, context: &str) -> Self {
        Self {
            rest: table.clone(),
            context: context.to_owned(),
        }
    }

    /// The value of `key`, if the table has it.
    pub fn take(&mut self, key: &str) -> Option<Value> {
        let at = self.rest.iter().position(|(k, _)| k == key)?;
        Some(self.rest.remove(at).1)
    }

    /// The value of `key`, which the table must have.
    pub fn need(&mut self, key: &str) -> Value {
        let context = self.context.clone();
        self.take(key)
            .unwrap_or_else(|| panic!("{context}: no key {key:?}"))
    }

    /// Stops the test if any key is left that nothing took.
    pub fn done(self) {
        let unknown: Vec<&str> = self.rest.iter().map(|(k, _)| k.as_str()).collect();
        assert!(
            unknown.is_empty(),
            "{}: unknown keys {unknown:?}",
            self.context
        );
    }
}
