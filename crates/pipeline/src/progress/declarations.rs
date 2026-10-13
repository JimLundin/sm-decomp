use super::Declaration;
use anyhow::{Result, ensure};

// Progress records declarations, not fixture admission. Decode only the family
// and routine symbols after scanning complete TOML values, so fixture payloads
// never become an AST. The family verifier validates those ignored fields.
// Dotted metadata and unusual metadata tables retain the complete decoder.
pub(super) fn parse(text: &str) -> Result<Declaration> {
    let projected = projection(text)?;
    Ok(toml::from_str(projected.as_deref().unwrap_or(text))?)
}

fn projection(text: &str) -> Result<Option<String>> {
    let mut projected = String::new();
    let mut routine = false;
    let mut root = true;
    let mut start = 0;
    while start < text.len() {
        let end = extent(text.as_bytes(), start, b"\n")?;
        let statement = text[start..end].trim();
        start = (end + 1).min(text.len());
        if statement.is_empty() || statement.starts_with('#') {
            continue;
        }
        if statement.starts_with('[') {
            let header = statement[..delimiter(statement, b'#').unwrap_or(statement.len())].trim_end();
            let array = header.starts_with("[[");
            let closing = if array { "]]" } else { "]" };
            let Some(key) = header.strip_suffix(closing) else { return Ok(None) };
            let key = toml_edit::Key::parse(&key[if array { 2 } else { 1 }..])?;
            routine = array && key.len() == 1 && key[0].get() == "routines";
            root = false;
            if key.first().is_some_and(|key| key.get() == "family")
                || (!array && key.len() == 1 && key[0].get() == "routines")
                || (key.len() > 1 && key[0].get() == "routines" && key[1].get() == "symbol")
            {
                return Ok(None);
            }
            if routine {
                projected.push_str("[[routines]]\n");
            }
            continue;
        }
        if !root && !routine {
            continue;
        }
        let Some(equal) = delimiter(statement, b'=') else { return Ok(None) };
        let key = toml_edit::Key::parse(&statement[..equal])?;
        let Some(first) = key.first() else { return Ok(None) };
        let wanted = if root {
            if first.get() == "routines" {
                if key.len() != 1 {
                    return Ok(None);
                }
                let Some(value) = inline_routines(&statement[equal + 1..])? else { return Ok(None) };
                projected.push_str("routines = ");
                projected.push_str(&value);
                projected.push('\n');
                continue;
            }
            first.get() == "family"
        } else {
            first.get() == "symbol"
        };
        if wanted {
            if key.len() != 1 {
                return Ok(None);
            }
            projected.push_str(statement);
            projected.push('\n');
        }
    }
    Ok(Some(projected))
}

fn inline_routines(value: &str) -> Result<Option<String>> {
    let end = extent(value.as_bytes(), 0, b"#")?;
    let value = value[..end].trim();
    let Some(values) = value.strip_prefix('[').and_then(|value| value.strip_suffix(']')) else {
        return Ok(None);
    };
    let mut projected = String::from("[");
    let mut start = 0;
    while start < values.len() {
        let end = extent(values.as_bytes(), start, b",")?;
        let value = values[start..end].trim();
        start = (end + 1).min(values.len());
        if value.is_empty() {
            continue;
        }
        let Some(fields) = value.strip_prefix('{').and_then(|value| value.strip_suffix('}')) else {
            return Ok(None);
        };
        projected.push('{');
        let mut start = 0;
        while start < fields.len() {
            let end = extent(fields.as_bytes(), start, b",")?;
            let field = fields[start..end].trim();
            start = (end + 1).min(fields.len());
            let Some(equal) = delimiter(field, b'=') else { return Ok(None) };
            let key = toml_edit::Key::parse(&field[..equal])?;
            if key.first().is_some_and(|key| key.get() == "symbol") {
                if key.len() != 1 {
                    return Ok(None);
                }
                if !projected.ends_with('{') {
                    projected.push(',');
                }
                projected.push_str(field);
            }
        }
        projected.push_str("},");
    }
    projected.push(']');
    Ok(Some(projected))
}

fn delimiter(text: &str, delimiter: u8) -> Option<usize> {
    let mut quote = 0;
    let mut escaped = false;
    text.bytes().position(|byte| {
        if escaped {
            escaped = false;
        } else if quote == b'"' && byte == b'\\' {
            escaped = true;
        } else if quote != 0 {
            if byte == quote {
                quote = 0;
            }
        } else if byte == b'\'' || byte == b'"' {
            quote = byte;
        } else if byte == delimiter {
            return true;
        }
        false
    })
}

// A newline inside an array, inline table or multiline string cannot change
// declaration context. Comments and quoted headers in ignored payloads also
// cannot create apparent declarations.
fn extent(bytes: &[u8], start: usize, delimiters: &[u8]) -> Result<usize> {
    let mut index = start;
    let mut quote = 0;
    let mut multiline = false;
    let mut brackets = Vec::new();
    while index < bytes.len() {
        let byte = bytes[index];
        if quote != 0 {
            if quote == b'"' && byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == quote {
                if multiline {
                    let run = bytes[index..].iter().take_while(|byte| **byte == quote).count();
                    if run >= 3 {
                        index += run;
                        quote = 0;
                        continue;
                    }
                } else {
                    quote = 0;
                }
            } else {
                ensure!(multiline || byte != b'\n', "unterminated declaration string");
            }
        } else {
            if brackets.is_empty() && delimiters.contains(&byte) {
                return Ok(index);
            }
            match byte {
                b'\'' | b'"' => {
                    quote = byte;
                    multiline = bytes.get(index..index + 3).is_some_and(|run| run == [byte; 3]);
                    if multiline {
                        index += 3;
                        continue;
                    }
                }
                b'[' | b'{' => brackets.push(byte),
                b']' | b'}' => {
                    ensure!(
                        brackets.pop() == Some(if byte == b']' { b'[' } else { b'{' }),
                        "unbalanced declaration brackets"
                    );
                }
                b'#' => {
                    index += bytes[index..].iter().position(|byte| *byte == b'\n').unwrap_or(bytes.len() - index);
                    continue;
                }
                _ => {}
            }
        }
        index += 1;
    }
    ensure!(quote == 0 && brackets.is_empty(), "unterminated declaration value");
    Ok(bytes.len())
}
