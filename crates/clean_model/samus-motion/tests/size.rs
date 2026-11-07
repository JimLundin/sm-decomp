//! The size score, as the brief for round 8 gives it: one weighted score over the model's logic.
//! Each identifier, keyword, literal and operator is a token, and so is each bracket pair. Free:
//! comments, layout, tests, `use` and `mod` lines, attributes, trailing commas, type declarations
//! (`struct`, `enum`, `type`, `trait` items) and the values of constants and statics (data).
//! Function signatures count half, `true` and `false` count double, and each nesting level inside
//! a function body adds a quarter. Pairs of parameter types in three or more signatures, and
//! pairs of arguments at three or more call sites, are reported: they cost more.
//!
//! The score is reported by file and by function (run with `--nocapture`). Over the header's
//! budget is a review stop, not a failure here: the root measures the score after the round. Over
//! the old code's score, 12,724, is a strict failure.

#![cfg(test)]

use std::collections::BTreeMap;

/// The header's budget, 6,564 weighted tokens.
const BUDGET: usize = 6_564;
/// The old code's score (the header's areas' `old`), over which the score is a strict failure.
const OLD: usize = 12_724;
/// The score the test holds to: `OLD`, or `BUDGET` to show the report through a runner that
/// prints only failures.
const LIMIT: usize = OLD;

/// The model's files, tests excluded.
const FILES: [(&str, &str); 11] = [
    (
        "clean_types/lib.rs",
        include_str!("../../../clean_types/src/lib.rs"),
    ),
    ("lib.rs", include_str!("../src/lib.rs")),
    ("motion.rs", include_str!("../src/motion.rs")),
    ("fit.rs", include_str!("../src/fit.rs")),
    ("rules.rs", include_str!("../src/rules.rs")),
    ("catalogue.rs", include_str!("../src/catalogue.rs")),
    ("collide.rs", include_str!("../src/collide.rs")),
    ("rows.rs", include_str!("../src/rows.rs")),
    ("situation.rs", include_str!("../src/situation.rs")),
    ("chart.rs", include_str!("../src/chart.rs")),
    ("chart/table.rs", include_str!("../src/chart/table.rs")),
];

/// Operators of more than one character, longest first.
const OPERATORS: [&str; 18] = [
    "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", "<<", ">>", "+=", "-=",
    "|=", "&=", "^=",
];

/// Whether a character continues a word.
fn wordy(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Where the token starting at `at` ends.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "test code: indices into a short text"
)]
fn token_end(chars: &[char], at: usize) -> usize {
    let rest: String = chars[at..chars.len().min(at + 3)].iter().collect();
    let c = chars[at];
    if c == '"' {
        let mut end = at + 1;
        while chars[end] != '"' {
            end += if chars[end] == '\\' { 2 } else { 1 };
        }
        return end + 1;
    }
    if c == '\'' && chars.get(at + 2) == Some(&'\'') {
        return at + 3;
    }
    if c == '\'' || wordy(c) {
        let mut end = at + 1;
        while end < chars.len() && wordy(chars[end]) {
            end += 1;
        }
        return end;
    }
    at + OPERATORS
        .iter()
        .find(|op| rest.starts_with(**op))
        .map_or(1, |op| op.len())
}

/// The tokens of `text`, comments and layout dropped.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "test code: indices into a short text"
)]
fn tokens(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let (mut at, mut out) = (0, Vec::new());
    while at < chars.len() {
        if chars[at].is_whitespace() {
            at += 1;
        } else if chars[at] == '/' && chars.get(at + 1) == Some(&'/') {
            while at < chars.len() && chars[at] != '\n' {
                at += 1;
            }
        } else {
            let end = token_end(&chars, at);
            out.push(chars[at..end].iter().collect());
            at = end;
        }
    }
    out
}

/// Whether a token opens or closes a bracket.
fn opens(t: &str) -> bool {
    matches!(t, "(" | "[" | "{")
}
fn closes(t: &str) -> bool {
    matches!(t, ")" | "]" | "}")
}

/// The index just past the bracket that `at` opens.
#[expect(clippy::arithmetic_side_effects, reason = "test code: a depth count")]
fn past_close(t: &[String], at: usize) -> usize {
    let mut depth = 0_usize;
    for (n, tok) in t.iter().enumerate().skip(at) {
        if opens(tok) {
            depth += 1;
        } else if closes(tok) {
            depth -= 1;
            if depth == 0 {
                return n + 1;
            }
        }
    }
    t.len()
}

/// The index just past the `;` ending a statement at `at`'s depth, or past a `{}` block if
/// one comes first (and `block` allows it).
#[expect(clippy::arithmetic_side_effects, reason = "test code: an index")]
fn past_item(t: &[String], at: usize, block: bool) -> usize {
    let mut n = at;
    while n < t.len() {
        match t[n].as_str() {
            ";" => return n + 1,
            "{" if block => return past_close(t, n),
            s if opens(s) => n = past_close(t, n),
            _ => n += 1,
        }
    }
    n
}

/// The score in 32nds: what one file's items add up to, by function, with the pairs seen.
#[derive(Default)]
struct Score {
    /// Per function (and `-` for code outside functions), in 32nds.
    by_fn: BTreeMap<String, usize>,
    /// Parameter type pairs, by the signatures holding them.
    params: BTreeMap<(String, String), usize>,
    /// Argument pairs, by the call sites holding them.
    args: BTreeMap<(String, String), usize>,
}

/// One token's weight in 32nds: 8ths of a token (a signature's 4, `true` and `false` 16,
/// else 8), times `4 + level` quarters.
#[expect(clippy::arithmetic_side_effects, reason = "test code: small weights")]
fn weight(tok: &str, eighths: usize, level: usize) -> usize {
    let base = if matches!(tok, "true" | "false") {
        eighths * 2
    } else {
        eighths
    };
    base * (4 + level)
}

/// The text of a range of tokens.
fn text(t: &[String]) -> String {
    t.concat()
}

/// The top-level comma-separated parts of the bracket opened at `at`.
#[expect(clippy::arithmetic_side_effects, reason = "test code: indices")]
fn parts(t: &[String], at: usize) -> Vec<String> {
    let end = past_close(t, at) - 1;
    let (mut out, mut from, mut n) = (Vec::new(), at + 1, at + 1);
    while n < end {
        if opens(&t[n]) {
            n = past_close(t, n);
        } else {
            if t[n] == "," {
                out.push(text(&t[from..n]));
                from = n + 1;
            }
            n += 1;
        }
    }
    if from < end {
        out.push(text(&t[from..end]));
    }
    out
}

/// Every unordered pair of distinct parts.
fn pairs(parts: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (n, a) in parts.iter().enumerate() {
        for b in parts.iter().skip(n.saturating_add(1)) {
            if a != b {
                let (x, y) = if a < b { (a, b) } else { (b, a) };
                out.push((x.clone(), y.clone()));
            }
        }
    }
    out
}

impl Score {
    /// Adds to a function's score.
    #[expect(clippy::arithmetic_side_effects, reason = "test code: a sum")]
    fn add(&mut self, name: &str, w: usize) {
        *self.by_fn.entry(name.to_owned()).or_default() += w;
    }

    /// A function: its signature at half, its body by nesting.
    #[expect(clippy::arithmetic_side_effects, reason = "test code: indices")]
    fn function(&mut self, t: &[String], from: usize, fn_at: usize) -> usize {
        let name = t[fn_at + 1].clone();
        let mut n = from;
        while t[n] != "{" && t[n] != ";" {
            if t[n] == "(" && n > fn_at {
                let types: Vec<String> = parts(t, n)
                    .iter()
                    .filter_map(|p| p.split_once(':').map(|(_, ty)| ty.to_owned()))
                    .collect();
                for pair in pairs(&types) {
                    *self.params.entry(pair).or_default() += 1;
                }
                let end = past_close(t, n);
                let w: usize = t[n..end].iter().map(|k| weight(k, 4, 0)).sum();
                self.add(&name, w);
                n = end;
                continue;
            }
            self.add(&name, weight(&t[n], 4, 0));
            n += 1;
        }
        if t[n] == ";" {
            return n + 1;
        }
        let end = past_close(t, n);
        self.add(&name, weight("{", 4, 0));
        self.body(t, n + 1, end - 1, &name);
        end
    }

    /// A function body's tokens, by nesting level.
    #[expect(clippy::arithmetic_side_effects, reason = "test code: a depth count")]
    fn body(&mut self, t: &[String], from: usize, to: usize, name: &str) {
        let mut level = 0_usize;
        for n in from..to {
            let tok = t[n].as_str();
            if tok == "}" {
                level = level.saturating_sub(1);
                continue;
            }
            let trailing = tok == "," && closes(&t[n + 1]);
            if !closes(tok) && !trailing {
                self.add(name, weight(tok, 8, level));
            }
            if tok == "{" {
                level += 1;
            }
            if tok == "(" && n > from && wordy(t[n - 1].chars().next().unwrap_or(' ')) {
                for pair in pairs(&parts(t, n)) {
                    *self.args.entry(pair).or_default() += 1;
                }
            }
        }
    }

    /// The items between `from` and `to`.
    #[expect(clippy::arithmetic_side_effects, reason = "test code: indices")]
    fn items(&mut self, t: &[String], from: usize, to: usize) {
        let mut n = from;
        while n < to {
            let mut k = n;
            while matches!(t[k].as_str(), "pub" | "const" | "unsafe" | "async")
                && t[k + 1] != "="
                && t[k + 1] != ":"
            {
                k += 1;
                if t[k] == "(" {
                    k = past_close(t, k);
                }
            }
            let head = t[k].as_str();
            n = match head {
                "#" => past_close(t, k + 1),
                "use" | "struct" | "enum" | "type" | "trait" | "union" => {
                    past_item(t, k, head != "use")
                }
                "mod" | "impl" => self.block(t, n, k),
                "fn" => self.function(t, n, k),
                _ => self.constant(t, n),
            };
        }
    }

    /// A `mod` (free) or `impl` (its header counted) holding items.
    #[expect(clippy::arithmetic_side_effects, reason = "test code: indices")]
    fn block(&mut self, t: &[String], from: usize, at: usize) -> usize {
        let mut n = at;
        while t[n] != "{" && t[n] != ";" {
            if t[at] == "impl" {
                self.add("-", weight(&t[n], 8, 0));
            }
            n += 1;
        }
        if t[n] == ";" {
            return n + 1;
        }
        let end = past_close(t, n);
        let _ = from;
        self.items(t, n + 1, end - 1);
        end
    }

    /// A constant or static: its head counted, its value free (data).
    #[expect(clippy::arithmetic_side_effects, reason = "test code: indices")]
    fn constant(&mut self, t: &[String], from: usize) -> usize {
        let mut n = from;
        while n < t.len() && t[n] != "=" && t[n] != ";" {
            self.add("-", weight(&t[n], 8, 0));
            n += 1;
        }
        past_item(t, n, false)
    }
}

/// The test module's text cut from a file.
fn without_tests(text: &str) -> String {
    text.split("#[cfg(test)]\nmod tests;").collect()
}

#[test]
#[expect(clippy::print_stdout, reason = "the size report")]
fn the_model_is_scored() {
    let mut lines = Vec::new();
    let mut total = 0_usize;
    let mut all = Score::default();
    for (file, src) in FILES {
        let t = tokens(&without_tests(src));
        let mut s = Score::default();
        s.items(&t, 0, t.len());
        let sum: usize = s.by_fn.values().sum();
        total = total.saturating_add(sum);
        let mut fns: Vec<(&String, &usize)> = s.by_fn.iter().collect();
        fns.sort_by(|a, b| b.1.cmp(a.1));
        let shown: Vec<String> = fns
            .iter()
            .map(|(f, w)| format!("{f} {}", **w / 32))
            .collect();
        lines.push(format!("{file}: {} ({})", sum / 32, shown.join(", ")));
        for (k, v) in s.params {
            *all.params.entry(k).or_default() += v;
        }
        for (k, v) in s.args {
            *all.args.entry(k).or_default() += v;
        }
    }
    let recurring = |m: &BTreeMap<(String, String), usize>| {
        m.iter()
            .filter(|e| *e.1 >= 3)
            .map(|((a, b), v)| format!("({a} , {b}) x{v}"))
            .collect::<Vec<_>>()
            .join("; ")
    };
    let report = format!(
        "score {} against the budget {BUDGET}\n{}\nparameter type pairs: {}\nargument pairs: {}",
        total / 32,
        lines.join("\n"),
        recurring(&all.params),
        recurring(&all.args)
    );
    println!("{report}");
    assert!(
        total / 32 <= LIMIT,
        "over {LIMIT} (old code {OLD}): {report}"
    );
}
