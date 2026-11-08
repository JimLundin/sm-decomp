//! Code lines, for an observer: the lines of a source that hold code.
//!
//! A line counts when it holds anything but whitespace and comments (doc comments included); a
//! line inside a string literal counts, as code, unless it is blank. A `#[cfg(test)]` item's lines
//! don't count, from its attribute to its end, the same items `size` leaves out;
//! `size::sources` already skips `tests` directories and `tests.rs` files.

/// The attribute that marks an item as test code.
const TEST_ATTRIBUTE: &str = "#[cfg(test)]";

/// What a character of the source is, once comments and literals are told apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mark {
    /// A line break.
    Break,
    /// Whitespace, or a comment's text.
    Blank,
    /// A literal's text: code, but no bracket or `;` of the code around it.
    Literal,
    /// Code outside literals.
    Code(char),
}

/// Whether `c` continues an identifier.
fn wordy(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The marks of the block comment that opens at `at`, pushed onto `out`; the index after it.
/// Block comments nest.
fn block_comment(chars: &[char], mut at: usize, out: &mut Vec<Mark>) -> usize {
    let mut depth = 0usize;
    while let Some(&c) = chars.get(at) {
        let next = chars.get(at.saturating_add(1)).copied();
        let pair = match (c, next) {
            ('/', Some('*')) => {
                depth = depth.saturating_add(1);
                true
            }
            ('*', Some('/')) => {
                depth = depth.saturating_sub(1);
                true
            }
            _ => false,
        };
        let width = if pair { 2 } else { 1 };
        for _ in 0..width {
            out.push(if c == '\n' { Mark::Break } else { Mark::Blank });
        }
        at = at.saturating_add(width);
        if depth == 0 {
            break;
        }
    }
    at
}

/// The marks of a literal that ends at the first `close` not escaped by a `\` (when `escapes`),
/// starting after its opening quote at `at`; the index after its closing quote.
fn literal(
    chars: &[char],
    mut at: usize,
    close: &[char],
    escapes: bool,
    out: &mut Vec<Mark>,
) -> usize {
    while let Some(&c) = chars.get(at) {
        if escapes && c == '\\' {
            out.push(Mark::Literal);
            if let Some(&escaped) = chars.get(at.saturating_add(1)) {
                out.push(if escaped == '\n' {
                    Mark::Break
                } else {
                    Mark::Literal
                });
            }
            at = at.saturating_add(2);
            continue;
        }
        let ends = chars
            .get(at..at.saturating_add(close.len()))
            .is_some_and(|run| run == close);
        if ends {
            out.extend(close.iter().map(|_| Mark::Literal));
            return at.saturating_add(close.len());
        }
        out.push(if c == '\n' {
            Mark::Break
        } else {
            Mark::Literal
        });
        at = at.saturating_add(1);
    }
    at
}

/// The closing run of a raw string whose `r` is at `at`, `"` and its hashes, and the index after
/// its opening quote, if a raw string starts there.
fn raw_string(chars: &[char], at: usize) -> Option<(Vec<char>, usize)> {
    let hashes = chars
        .get(at.saturating_add(1)..)?
        .iter()
        .take_while(|&&c| c == '#')
        .count();
    let quote = at.saturating_add(1).saturating_add(hashes);
    (chars.get(quote) == Some(&'"')).then(|| {
        let close = std::iter::once('"')
            .chain(std::iter::repeat_n('#', hashes))
            .collect();
        (close, quote.saturating_add(1))
    })
}

/// Each character's mark.
fn marks(src: &str) -> Vec<Mark> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::with_capacity(chars.len());
    let mut at = 0usize;
    while let Some(&c) = chars.get(at) {
        let next = chars.get(at.saturating_add(1)).copied();
        let word_start = at
            .checked_sub(1)
            .and_then(|before| chars.get(before))
            .is_none_or(|&before| !wordy(before));
        let start = at;
        at = match (c, next) {
            ('/', Some('/')) => {
                let end = chars
                    .get(at..)
                    .and_then(|rest| rest.iter().position(|&c| c == '\n'))
                    .map_or(chars.len(), |offset| at.saturating_add(offset));
                out.extend((at..end).map(|_| Mark::Blank));
                end
            }
            ('/', Some('*')) => block_comment(&chars, at, &mut out),
            ('"', _) => {
                out.push(Mark::Literal);
                literal(&chars, at.saturating_add(1), &['"'], true, &mut out)
            }
            ('r', _) if word_start => {
                if let Some((close, body)) = raw_string(&chars, at) {
                    out.extend((at..body).map(|_| Mark::Literal));
                    literal(&chars, body, &close, false, &mut out)
                } else {
                    out.push(Mark::Code(c));
                    at.saturating_add(1)
                }
            }
            ('\'', Some(quoted)) => {
                // A character literal, `'x'` or `'\n'`; anything else is a lifetime.
                let escaped = quoted == '\\';
                if escaped || chars.get(at.saturating_add(2)) == Some(&'\'') {
                    out.push(Mark::Literal);
                    literal(&chars, at.saturating_add(1), &['\''], true, &mut out)
                } else {
                    out.push(Mark::Code(c));
                    at.saturating_add(1)
                }
            }
            ('\n', _) => {
                out.push(Mark::Break);
                at.saturating_add(1)
            }
            _ if c.is_whitespace() => {
                out.push(Mark::Blank);
                at.saturating_add(1)
            }
            _ => {
                out.push(Mark::Code(c));
                at.saturating_add(1)
            }
        };
        if at == start {
            break;
        }
    }
    out
}

/// The index just past the `#[cfg(test)]` item whose attribute ends before `at`: past its `;`, or
/// past the brace that closes its body.
fn item_end(marks: &[Mark], at: usize) -> usize {
    let mut depth = 0usize;
    let mut opened = false;
    for (offset, mark) in marks.get(at..).unwrap_or_default().iter().enumerate() {
        let here = at.saturating_add(offset).saturating_add(1);
        match mark {
            Mark::Code('{' | '(' | '[') => {
                depth = depth.saturating_add(1);
                opened = true;
            }
            Mark::Code('}' | ')' | ']') => {
                depth = depth.saturating_sub(1);
                if depth == 0 && opened && *mark == Mark::Code('}') {
                    return here;
                }
            }
            Mark::Code(';') if depth == 0 => return here,
            Mark::Code(_) | Mark::Break | Mark::Blank | Mark::Literal => {}
        }
    }
    marks.len()
}

/// Whether the code at `at` is the test attribute.
fn test_attribute(marks: &[Mark], at: usize) -> bool {
    let pattern: Vec<Mark> = TEST_ATTRIBUTE.chars().map(Mark::Code).collect();
    marks
        .get(at..at.saturating_add(pattern.len()))
        .is_some_and(|run| run == pattern.as_slice())
}

/// How many lines of `src` hold code, its `#[cfg(test)]` items left out.
#[must_use]
pub fn code_lines(src: &str) -> usize {
    let marks = marks(src);
    let mut lines = 0usize;
    let mut holds_code = false;
    let mut at = 0usize;
    while let Some(&mark) = marks.get(at) {
        if test_attribute(&marks, at) {
            // The item's lines go, but code before the attribute on its line counts.
            let end = item_end(&marks, at.saturating_add(TEST_ATTRIBUTE.len()));
            let breaks = marks
                .get(at..end)
                .unwrap_or_default()
                .iter()
                .filter(|&&mark| mark == Mark::Break)
                .count();
            if holds_code && breaks > 0 {
                lines = lines.saturating_add(1);
                holds_code = false;
            }
            at = end;
            continue;
        }
        match mark {
            Mark::Break => {
                lines = lines.saturating_add(usize::from(holds_code));
                holds_code = false;
            }
            Mark::Code(_) | Mark::Literal => holds_code = true,
            Mark::Blank => {}
        }
        at = at.saturating_add(1);
    }
    lines.saturating_add(usize::from(holds_code))
}
