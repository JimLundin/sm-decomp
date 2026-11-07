//! The spec header's size count, in logical tokens: each identifier, keyword, literal and
//! operator is one, and so is each bracket pair; comments, layout and tests are free. The values
//! of constants and statics (the chart's rows, the reference rules, the catalogue) are data,
//! counted apart and reported; the target is on everything else, the logic. Generic angle
//! brackets are counted as operators, so the count errs high.
//!
//! Over the header's baseline is a strict failure. Over its target is not: the count is reported
//! against the target (run with `--nocapture` to see it), and the implementer's report says why
//! the rest can't go.

#[cfg(test)]
mod size {
    /// The spec header's target for the logic, and its baseline (over it is a strict failure).
    const TARGET: usize = 7_300;
    const BASELINE: usize = 15_100;

    /// The model's files, tests excluded.
    const FILES: [(&str, &str); 8] = [
        (
            "clean_types/lib.rs",
            include_str!("../../../clean_types/src/lib.rs"),
        ),
        ("lib.rs", include_str!("../src/lib.rs")),
        ("situation.rs", include_str!("../src/situation.rs")),
        ("rules.rs", include_str!("../src/rules.rs")),
        ("catalogue.rs", include_str!("../src/catalogue.rs")),
        ("collide.rs", include_str!("../src/collide.rs")),
        ("chart.rs", include_str!("../src/chart.rs")),
        ("chart/table.rs", include_str!("../src/chart/table.rs")),
    ];

    /// Operators of more than one character, longest first.
    const OPERATORS: [&str; 18] = [
        "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", "<<", ">>", "+=", "-=",
        "|=", "&=", "^=",
    ];

    /// The tokens of `text`, comments and layout dropped: each with its text. A closing bracket
    /// is kept, for the depth of what it closes, but is free (its pair counts at the opener).
    #[expect(clippy::arithmetic_side_effects)]
    fn tokens(text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let (mut at, mut out) = (0, Vec::new());
        while at < chars.len() {
            let c = chars[at];
            let rest: String = chars[at..chars.len().min(at + 3)].iter().collect();
            if c.is_whitespace() {
                at += 1;
                continue;
            }
            if rest.starts_with("//") {
                while at < chars.len() && chars[at] != '\n' {
                    at += 1;
                }
                continue;
            }
            let from = at;
            if c == '"' {
                at += 1;
                while chars[at] != '"' {
                    at += if chars[at] == '\\' { 2 } else { 1 };
                }
                at += 1;
            } else if c == '\'' {
                // A char literal, or a lifetime.
                if chars.get(at + 2) == Some(&'\'') {
                    at += 3;
                } else {
                    at += 1;
                    while chars[at].is_alphanumeric() || chars[at] == '_' {
                        at += 1;
                    }
                }
            } else if c.is_alphanumeric() || c == '_' {
                while at < chars.len() && (chars[at].is_alphanumeric() || chars[at] == '_') {
                    at += 1;
                }
            } else if let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(**op)) {
                at += op.len();
            } else {
                at += 1;
            }
            out.push(chars[from..at].iter().collect());
        }
        out
    }

    /// A file's count: its logic, and its data (the values of its constants and statics).
    #[expect(clippy::arithmetic_side_effects)]
    fn count(text: &str) -> (usize, usize) {
        let tokens = tokens(text);
        let (mut logic, mut data) = (0, 0);
        // Inside a constant's or static's value: the bracket depth it started at.
        let mut value: Option<usize> = None;
        let mut depth = 0_usize;
        let mut item = false;
        for (n, t) in tokens.iter().enumerate() {
            let closes = matches!(t.as_str(), ")" | "]" | "}");
            let opens = matches!(t.as_str(), "(" | "[" | "{");
            if closes {
                depth = depth.saturating_sub(1);
            }
            match value {
                Some(start) if t == ";" && depth == start => {
                    value = None;
                    logic += 1;
                }
                Some(_) => data += usize::from(!closes),
                None => {
                    logic += usize::from(!closes);
                    // `const NAME` or `static NAME`, not `const fn`: an item whose value is data.
                    if matches!(t.as_str(), "const" | "static")
                        && tokens.get(n + 1).is_some_and(|next| next != "fn")
                    {
                        item = true;
                    }
                    if item && t == "=" {
                        item = false;
                        value = Some(depth);
                    }
                }
            }
            if opens {
                depth += 1;
            }
        }
        (logic, data)
    }

    #[test]
    #[expect(clippy::print_stdout)]
    fn the_model_is_within_the_baseline() {
        let counts: Vec<(&str, (usize, usize))> =
            FILES.iter().map(|&(f, t)| (f, count(t))).collect();
        let logic: usize = counts.iter().map(|c| c.1.0).sum();
        let data: usize = counts.iter().map(|c| c.1.1).sum();
        let report = counts
            .iter()
            .map(|(f, (l, d))| format!("{f}: {l} logic, {d} data"))
            .collect::<Vec<_>>()
            .join("; ");
        println!(
            "logic {logic} tokens against the target of {TARGET} (baseline {BASELINE}); data {data} tokens apart ({report})"
        );
        assert!(
            logic <= BASELINE,
            "over the baseline: {logic} tokens of logic ({report})"
        );
    }
}
