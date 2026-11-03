//! JSON as the workflow reads it, written byte for byte as Python's
//! `json.dumps` with its defaults (`", "` and `": "` separators, ASCII
//! escapes), so the planner's output can be compared with the Python one's
//! while both exist.

use std::fmt;

/// One JSON value. The planner writes no floats.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Bool(bool),
    Int(i64),
    Str(String),
    Array(Vec<Json>),
    Object(Object),
}

/// An object, keeping its keys in insertion order as a Python dict does.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Object(Vec<(String, Json)>);

impl Object {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a key: an existing key keeps its place, a new one goes last.
    pub fn set(&mut self, key: &str, value: impl Into<Json>) {
        let value = value.into();
        match self.0.iter_mut().find(|(name, _)| name == key) {
            Some((_, slot)) => *slot = value,
            None => self.0.push((key.to_owned(), value)),
        }
    }

    /// The object with one more key, for building literals.
    pub fn with(mut self, key: &str, value: impl Into<Json>) -> Self {
        self.set(key, value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        self.0
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    pub fn str(&self, key: &str) -> &str {
        match self.get(key) {
            Some(Json::Str(text)) => text,
            _ => "",
        }
    }

    pub fn int(&self, key: &str) -> Option<i64> {
        match self.get(key) {
            Some(Json::Int(number)) => Some(*number),
            _ => None,
        }
    }
}

impl From<bool> for Json {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for Json {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<usize> for Json {
    fn from(value: usize) -> Self {
        Self::Int(i64::try_from(value).unwrap_or(i64::MAX))
    }
}

impl From<&str> for Json {
    fn from(value: &str) -> Self {
        Self::Str(value.to_owned())
    }
}

impl From<String> for Json {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<Object> for Json {
    fn from(value: Object) -> Self {
        Self::Object(value)
    }
}

impl From<Vec<Object>> for Json {
    fn from(value: Vec<Object>) -> Self {
        Self::Array(value.into_iter().map(Self::Object).collect())
    }
}

fn write_string(out: &mut fmt::Formatter<'_>, text: &str) -> fmt::Result {
    out.write_str("\"")?;
    for c in text.chars() {
        match c {
            '"' => out.write_str("\\\"")?,
            '\\' => out.write_str("\\\\")?,
            '\n' => out.write_str("\\n")?,
            '\r' => out.write_str("\\r")?,
            '\t' => out.write_str("\\t")?,
            '\u{8}' => out.write_str("\\b")?,
            '\u{c}' => out.write_str("\\f")?,
            ' '..='~' => write!(out, "{c}")?,
            _ => {
                let mut units = [0; 2];
                for unit in c.encode_utf16(&mut units) {
                    write!(out, "\\u{unit:04x}")?;
                }
            }
        }
    }
    out.write_str("\"")
}

impl fmt::Display for Json {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => out.write_str(if *value { "true" } else { "false" }),
            Self::Int(value) => write!(out, "{value}"),
            Self::Str(text) => write_string(out, text),
            Self::Array(items) => {
                out.write_str("[")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.write_str(", ")?;
                    }
                    write!(out, "{item}")?;
                }
                out.write_str("]")
            }
            Self::Object(object) => write!(out, "{object}"),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str("{")?;
        for (index, (key, value)) in self.0.iter().enumerate() {
            if index > 0 {
                out.write_str(", ")?;
            }
            write_string(out, key)?;
            write!(out, ": {value}")?;
        }
        out.write_str("}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_as_python_json_dumps() {
        let inner = Json::from(vec![
            Object::new().with("kind", "seed").with("segments", "a b"),
        ]);
        let object = Object::new()
            .with("name", "x\"\u{7f}\u{e9}\u{1f600}")
            .with("first", true)
            .with("steps", 12_i64)
            .with("shards", inner.to_string())
            .with("include", Json::Array(Vec::new()));
        assert_eq!(
            object.to_string(),
            concat!(
                r#"{"name": "x\"\u007f\u00e9\ud83d\ude00", "first": true, "steps": 12, "#,
                r#""shards": "[{\"kind\": \"seed\", \"segments\": \"a b\"}]", "include": []}"#
            )
        );
    }

    #[test]
    fn setting_a_key_keeps_its_place() {
        let mut object = Object::new().with("a", 1_i64).with("b", 2_i64);
        object.set("a", 3_i64);
        assert_eq!(object.to_string(), r#"{"a": 3, "b": 2}"#);
    }
}
