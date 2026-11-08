//! A reader for the part of TOML the vector file uses: comments, `[table]` and `[[array]]`
//! headers with dotted names, and keys (bare or quoted) holding strings, integers, booleans,
//! arrays (across lines) and inline tables.

use std::iter::Peekable;
use std::str::Chars;

/// A TOML value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// A string.
    Str(String),
    /// An integer.
    Int(i64),
    /// A boolean.
    Bool(bool),
    /// An array.
    Array(Vec<Self>),
    /// A table.
    Table(Table),
}

/// A table: its keys in the order written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table(pub Vec<(String, Value)>);

impl Table {
    /// The value at `key`.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// The value at `key`, to change.
    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Adds `key`, refusing a key written twice.
    fn insert(&mut self, key: String, value: Value) -> Result<(), String> {
        if self.get(&key).is_some() {
            return Err(format!("key {key} written twice"));
        }
        self.0.push((key, value));
        Ok(())
    }
}

/// Reads a whole file into its root table.
pub fn parse(text: &str) -> Result<Table, String> {
    let mut reader = Reader {
        chars: text.chars().peekable(),
    };
    let mut root = Table::default();
    let mut path: Vec<String> = Vec::new();
    loop {
        reader.skip_blank();
        match reader.chars.peek() {
            None => return Ok(root),
            Some('[') => path = reader.header(&mut root)?,
            Some(_) => {
                let (key, value) = reader.key_value()?;
                table_at(&mut root, &path)?.insert(key, value)?;
            }
        }
    }
}

/// The table at a dotted path, made where missing; through an array, its last table.
fn table_at<'t>(table: &'t mut Table, path: &[String]) -> Result<&'t mut Table, String> {
    let Some((first, rest)) = path.split_first() else {
        return Ok(table);
    };
    if table.get(first).is_none() {
        table
            .0
            .push((first.clone(), Value::Table(Table::default())));
    }
    let next = match table.get_mut(first) {
        Some(Value::Table(inner)) => inner,
        Some(Value::Array(items)) => match items.last_mut() {
            Some(Value::Table(inner)) => inner,
            _ => return Err(format!("{first} is not an array of tables")),
        },
        _ => return Err(format!("{first} is not a table")),
    };
    table_at(next, rest)
}

/// A cursor over the file's characters.
struct Reader<'a> {
    chars: Peekable<Chars<'a>>,
}

impl Reader<'_> {
    /// Skips spaces, line ends and comments.
    fn skip_blank(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == '#' {
                self.skip_line();
            } else if c.is_whitespace() {
                self.chars.next();
            } else {
                return;
            }
        }
    }

    /// Skips to the end of the line.
    fn skip_line(&mut self) {
        for c in self.chars.by_ref() {
            if c == '\n' {
                return;
            }
        }
    }

    /// Takes `expected`, or fails.
    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.chars.next() {
            Some(c) if c == expected => Ok(()),
            other => Err(format!("expected {expected:?}, found {other:?}")),
        }
    }

    /// Reads a header line, making its table, and returns its path.
    fn header(&mut self, root: &mut Table) -> Result<Vec<String>, String> {
        self.expect_char('[')?;
        let array = self.chars.peek() == Some(&'[');
        if array {
            self.chars.next();
        }
        let name: String = self.chars.by_ref().take_while(|&c| c != ']').collect();
        if array {
            self.expect_char(']')?;
        }
        let path: Vec<String> = name.trim().split('.').map(str::to_owned).collect();
        let Some((last, parent)) = path.split_last() else {
            return Err("empty header".to_owned());
        };
        let table = table_at(root, parent)?;
        if array {
            push_table(table, last)?;
        } else {
            table_at(table, std::slice::from_ref(last))?;
        }
        Ok(path)
    }

    /// Reads `key = value`.
    fn key_value(&mut self) -> Result<(String, Value), String> {
        let key = self.key()?;
        self.skip_blank();
        self.expect_char('=')?;
        self.skip_blank();
        let value = self.value()?;
        Ok((key, value))
    }

    /// Reads a bare or quoted key.
    fn key(&mut self) -> Result<String, String> {
        if self.chars.peek() == Some(&'"') {
            return self.string();
        }
        let mut key = String::new();
        while let Some(&c) = self.chars.peek() {
            if !(c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                break;
            }
            key.push(c);
            self.chars.next();
        }
        if key.is_empty() {
            return Err(format!("expected a key, found {:?}", self.chars.peek()));
        }
        Ok(key)
    }

    /// Reads a value.
    fn value(&mut self) -> Result<Value, String> {
        match self.chars.peek() {
            Some('"') => self.string().map(Value::Str),
            Some('[') => self.array(),
            Some('{') => self.inline_table(),
            Some('t' | 'f') => self.boolean(),
            Some(c) if c.is_ascii_digit() || *c == '-' || *c == '+' => self.integer(),
            other => Err(format!("expected a value, found {other:?}")),
        }
    }

    /// Reads a basic string, with `\"` and `\\` escapes.
    fn string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut text = String::new();
        loop {
            match self.chars.next() {
                Some('"') => return Ok(text),
                Some('\\') => text.push(self.chars.next().ok_or("unended escape")?),
                Some(c) => text.push(c),
                None => return Err("unended string".to_owned()),
            }
        }
    }

    /// Reads an array, which may run across lines.
    fn array(&mut self) -> Result<Value, String> {
        self.expect_char('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_blank();
            if self.chars.peek() == Some(&']') {
                self.chars.next();
                return Ok(Value::Array(items));
            }
            items.push(self.value()?);
            self.skip_blank();
            if self.chars.peek() == Some(&',') {
                self.chars.next();
            }
        }
    }

    /// Reads an inline table.
    fn inline_table(&mut self) -> Result<Value, String> {
        self.expect_char('{')?;
        let mut table = Table::default();
        loop {
            self.skip_blank();
            if self.chars.peek() == Some(&'}') {
                self.chars.next();
                return Ok(Value::Table(table));
            }
            let (key, value) = self.key_value()?;
            table.insert(key, value)?;
            self.skip_blank();
            if self.chars.peek() == Some(&',') {
                self.chars.next();
            }
        }
    }

    /// Reads `true` or `false`.
    fn boolean(&mut self) -> Result<Value, String> {
        let word = self.word();
        match word.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(format!("expected a boolean, found {word}")),
        }
    }

    /// Reads a decimal integer.
    fn integer(&mut self) -> Result<Value, String> {
        let word = self.word().replace('_', "");
        word.parse()
            .map(Value::Int)
            .map_err(|_| format!("expected an integer, found {word}"))
    }

    /// Reads up to the next space, comma or closing bracket.
    fn word(&mut self) -> String {
        let mut word = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() || matches!(c, ',' | ']' | '}') {
                break;
            }
            word.push(c);
            self.chars.next();
        }
        word
    }
}

/// Adds a new table to the array of tables at `key`.
fn push_table(table: &mut Table, key: &str) -> Result<(), String> {
    if table.get(key).is_none() {
        table.0.push((key.to_owned(), Value::Array(Vec::new())));
    }
    match table.get_mut(key) {
        Some(Value::Array(items)) => {
            items.push(Value::Table(Table::default()));
            Ok(())
        }
        _ => Err(format!("{key} is not an array of tables")),
    }
}
