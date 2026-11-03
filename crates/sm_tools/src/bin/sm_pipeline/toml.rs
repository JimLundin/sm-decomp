//! A reader for the TOML the repository commits: `scenarios/*.toml`,
//! `movies/manifest.toml` and `.github/scripts/row_seconds.toml`.
//!
//! It reads comments, `[table]` and `[[array]]` headers, bare and quoted keys,
//! basic and literal strings, integers, floats, booleans, arrays and inline
//! tables. Anything else (dotted keys, multi-line strings, dates) is an error
//! rather than a guess, so a file outside the subset fails loudly.

/// One TOML value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Table(Table),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(number) => Some(*number),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&Table> {
        match self {
            Self::Table(table) => Some(table),
            _ => None,
        }
    }
}

/// A table, keeping its keys in file order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Table(Vec<(String, Value)>);

impl Table {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value))
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(name, _)| name.as_str())
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0
            .iter_mut()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    fn insert(&mut self, key: String, value: Value) -> Result<(), String> {
        if self.contains(&key) {
            return Err(format!("duplicate key {key}"));
        }
        self.0.push((key, value));
        Ok(())
    }
}

/// Where the following `key = value` lines go.
enum Section {
    Root,
    Table(String),
    ArrayItem(String),
}

/// Parse a whole document.
pub fn parse(text: &str) -> Result<Table, String> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        at: 0,
        line: 1,
    };
    parser
        .document()
        .map_err(|error| format!("line {}: {error}", parser.line))
}

/// Read and parse one file.
pub fn read(path: &std::path::Path) -> Result<Table, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    parse(&text).map_err(|error| format!("{}: {error}", path.display()))
}

struct Parser {
    chars: Vec<char>,
    at: usize,
    line: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.at + offset).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let next = self.peek()?;
        self.at += 1;
        if next == '\n' {
            self.line += 1;
        }
        Some(next)
    }

    fn expect(&mut self, wanted: char) -> Result<(), String> {
        match self.bump() {
            Some(found) if found == wanted => Ok(()),
            found => Err(format!("expected {wanted:?}, found {found:?}")),
        }
    }

    fn spaces(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t')) {
            self.bump();
        }
    }

    fn comment(&mut self) {
        if self.peek() == Some('#') {
            while !matches!(self.peek(), None | Some('\n')) {
                self.bump();
            }
        }
    }

    /// Spaces, comments and line breaks, as between array items.
    fn blank(&mut self) {
        loop {
            self.spaces();
            self.comment();
            match self.peek() {
                Some('\n') => {
                    self.bump();
                }
                Some('\r') if self.peek_at(1) == Some('\n') => {
                    self.bump();
                }
                _ => return,
            }
        }
    }

    /// The end of a line after a header or a key's value.
    fn line_end(&mut self) -> Result<(), String> {
        self.spaces();
        self.comment();
        if self.peek() == Some('\r') {
            self.bump();
        }
        match self.peek() {
            None => Ok(()),
            Some('\n') => {
                self.bump();
                Ok(())
            }
            Some(found) => Err(format!("unexpected {found:?} after a value")),
        }
    }

    fn document(&mut self) -> Result<Table, String> {
        let mut root = Table::default();
        let mut section = Section::Root;
        loop {
            self.blank();
            match self.peek() {
                None => return Ok(root),
                Some('[') => {
                    self.bump();
                    let array = self.peek() == Some('[');
                    if array {
                        self.bump();
                    }
                    self.spaces();
                    let key = self.key()?;
                    self.spaces();
                    self.expect(']')?;
                    if array {
                        self.expect(']')?;
                        match root.get_mut(&key) {
                            None => root.insert(
                                key.clone(),
                                Value::Array(vec![Value::Table(Table::default())]),
                            )?,
                            Some(Value::Array(items)) => items.push(Value::Table(Table::default())),
                            Some(_) => {
                                return Err(format!("[[{key}]] names a key that is not an array"));
                            }
                        }
                        section = Section::ArrayItem(key);
                    } else {
                        root.insert(key.clone(), Value::Table(Table::default()))?;
                        section = Section::Table(key);
                    }
                    self.line_end()?;
                }
                Some(_) => {
                    let key = self.key()?;
                    self.spaces();
                    self.expect('=')?;
                    self.spaces();
                    let value = self.value()?;
                    self.line_end()?;
                    let table = match &section {
                        Section::Root => &mut root,
                        Section::Table(name) => match root.get_mut(name) {
                            Some(Value::Table(table)) => table,
                            _ => return Err(format!("table {name} vanished")),
                        },
                        Section::ArrayItem(name) => match root.get_mut(name) {
                            Some(Value::Array(items)) => match items.last_mut() {
                                Some(Value::Table(table)) => table,
                                _ => return Err(format!("[[{name}]] holds no table")),
                            },
                            _ => return Err(format!("array {name} vanished")),
                        },
                    };
                    table.insert(key, value)?;
                }
            }
        }
    }

    fn key(&mut self) -> Result<String, String> {
        let key = match self.peek() {
            Some('"') => self.basic_string()?,
            Some('\'') => self.literal_string()?,
            _ => {
                let mut key = String::new();
                while let Some(next) = self
                    .peek()
                    .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                {
                    key.push(next);
                    self.bump();
                }
                if key.is_empty() {
                    return Err(format!("expected a key, found {:?}", self.peek()));
                }
                key
            }
        };
        self.spaces();
        if self.peek() == Some('.') {
            return Err(format!("dotted key {key}. is outside the supported subset"));
        }
        Ok(key)
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some('"') => self.basic_string().map(Value::String),
            Some('\'') => self.literal_string().map(Value::String),
            Some('[') => self.array(),
            Some('{') => self.inline_table(),
            Some('t' | 'f') => self.boolean(),
            Some(_) => self.number(),
            None => Err("expected a value".into()),
        }
    }

    fn basic_string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        if self.peek() == Some('"') && self.peek_at(1) == Some('"') {
            return Err("multi-line strings are outside the supported subset".into());
        }
        let mut text = String::new();
        loop {
            match self.bump() {
                None | Some('\n') => return Err("unterminated string".into()),
                Some('"') => return Ok(text),
                Some('\\') => {
                    let escaped = match self.bump() {
                        Some('b') => '\u{8}',
                        Some('t') => '\t',
                        Some('n') => '\n',
                        Some('f') => '\u{c}',
                        Some('r') => '\r',
                        Some('"') => '"',
                        Some('\\') => '\\',
                        Some('u') => self.unicode(4)?,
                        Some('U') => self.unicode(8)?,
                        found => return Err(format!("unknown escape {found:?}")),
                    };
                    text.push(escaped);
                }
                Some(control) if control.is_control() && control != '\t' => {
                    return Err(format!("control character {control:?} in a string"));
                }
                Some(next) => text.push(next),
            }
        }
    }

    fn unicode(&mut self, digits: usize) -> Result<char, String> {
        let mut hex = String::new();
        for _ in 0..digits {
            hex.push(self.bump().ok_or("short unicode escape")?);
        }
        u32::from_str_radix(&hex, 16)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| format!("invalid unicode escape {hex}"))
    }

    fn literal_string(&mut self) -> Result<String, String> {
        self.expect('\'')?;
        if self.peek() == Some('\'') && self.peek_at(1) == Some('\'') {
            return Err("multi-line strings are outside the supported subset".into());
        }
        let mut text = String::new();
        loop {
            match self.bump() {
                None | Some('\n') => return Err("unterminated string".into()),
                Some('\'') => return Ok(text),
                Some(next) => text.push(next),
            }
        }
    }

    fn array(&mut self) -> Result<Value, String> {
        self.expect('[')?;
        let mut items = Vec::new();
        loop {
            self.blank();
            if self.peek() == Some(']') {
                self.bump();
                return Ok(Value::Array(items));
            }
            items.push(self.value()?);
            self.blank();
            match self.bump() {
                Some(',') => {}
                Some(']') => return Ok(Value::Array(items)),
                found => return Err(format!("expected , or ] in an array, found {found:?}")),
            }
        }
    }

    fn inline_table(&mut self) -> Result<Value, String> {
        self.expect('{')?;
        let mut table = Table::default();
        self.spaces();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(Value::Table(table));
        }
        loop {
            self.spaces();
            let key = self.key()?;
            self.spaces();
            self.expect('=')?;
            self.spaces();
            let value = self.value()?;
            table.insert(key, value)?;
            self.spaces();
            match self.bump() {
                Some(',') => {}
                Some('}') => return Ok(Value::Table(table)),
                found => {
                    return Err(format!(
                        "expected , or }} in an inline table, found {found:?}"
                    ));
                }
            }
        }
    }

    fn boolean(&mut self) -> Result<Value, String> {
        for (word, value) in [("true", true), ("false", false)] {
            if word
                .chars()
                .enumerate()
                .all(|(offset, c)| self.peek_at(offset) == Some(c))
            {
                self.at += word.len();
                return Ok(Value::Boolean(value));
            }
        }
        Err("expected true or false".into())
    }

    fn number(&mut self) -> Result<Value, String> {
        let mut text = String::new();
        while let Some(next) = self
            .peek()
            .filter(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '_' | '.' | 'e' | 'E'))
        {
            text.push(next);
            self.bump();
        }
        let digits: String = text.chars().filter(|&c| c != '_').collect();
        let parsed = if digits.contains(['.', 'e', 'E']) {
            digits.parse().map(Value::Float).ok()
        } else {
            digits.parse().map(Value::Integer).ok()
        };
        parsed.ok_or_else(|| format!("unsupported value {text:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_subset_the_repository_uses() {
        let table = parse(concat!(
            "# comment\n",
            "version = 1\n",
            "domains = [\"ram\", 'input', # trailing\n  \"samus\",\n]\n",
            "pins = [\n    { id = \"a\", \"quoted\" = \"x\\u00e9\" },\n    {  },\n]\n",
            "[lock]\n",
            "row-a = 15\n",
            "row-b = 1.5\n",
            "[[scenario]]\n",
            "id = \"one\"\n",
            "flag = true\n",
            "[[scenario]]\n",
            "id = \"two\"\n",
        ))
        .unwrap();
        assert_eq!(table.get("version"), Some(&Value::Integer(1)));
        let domains: Vec<_> = table
            .get("domains")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect();
        assert_eq!(domains, ["ram", "input", "samus"]);
        let pins = table.get("pins").unwrap().as_array().unwrap();
        assert_eq!(
            pins[0].as_table().unwrap().get("quoted").unwrap().as_str(),
            Some("x\u{e9}")
        );
        assert_eq!(pins[1], Value::Table(Table::default()));
        let lock = table.get("lock").unwrap().as_table().unwrap();
        assert_eq!(lock.get("row-b"), Some(&Value::Float(1.5)));
        let scenarios = table.get("scenario").unwrap().as_array().unwrap();
        assert_eq!(scenarios.len(), 2);
        assert_eq!(
            scenarios[0].as_table().unwrap().get("flag"),
            Some(&Value::Boolean(true))
        );
    }

    #[test]
    fn rejects_what_it_does_not_read() {
        for text in [
            "a.b = 1\n",
            "a = \"\"\"x\"\"\"\n",
            "a = 1979-05-27\n",
            "a = 1\na = 2\n",
            "a = 1 b\n",
        ] {
            assert!(parse(text).is_err(), "{text:?}");
        }
    }
}
