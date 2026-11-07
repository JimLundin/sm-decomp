//! The spec header's size target: the model, its data tables included, in logical tokens. Each
//! identifier, keyword, literal and operator is one, and so is each bracket pair; comments,
//! layout and tests are free. Generic angle brackets are counted as operators, so the count
//! errs high.

#[cfg(test)]
mod size {
    /// The spec header's target, and its baseline (over it is a strict failure).
    const TARGET: usize = 6_500;
    const BASELINE: usize = 13_500;

    /// The model's files, tests excluded.
    const FILES: [(&str, &str); 7] = [
        (
            "clean_types/lib.rs",
            include_str!("../../../clean_types/src/lib.rs"),
        ),
        ("lib.rs", include_str!("../src/lib.rs")),
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

    /// The logical tokens in `text`.
    #[expect(clippy::arithmetic_side_effects)]
    fn tokens(text: &str) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let (mut at, mut count) = (0, 0);
        while at < chars.len() {
            let c = chars[at];
            let rest: String = chars[at..chars.len().min(at + 3)].iter().collect();
            if c.is_whitespace() || matches!(c, ')' | ']' | '}') {
                at += 1;
                continue;
            }
            if rest.starts_with("//") {
                while at < chars.len() && chars[at] != '\n' {
                    at += 1;
                }
                continue;
            }
            count += 1;
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
        }
        count
    }

    #[test]
    fn the_model_is_within_the_size_target() {
        let counts: Vec<(&str, usize)> = FILES.iter().map(|&(f, t)| (f, tokens(t))).collect();
        let total: usize = counts.iter().map(|c| c.1).sum();
        let report = counts
            .iter()
            .map(|(f, n)| format!("{f}: {n}"))
            .collect::<Vec<_>>()
            .join(", ");
        assert!(
            total <= BASELINE,
            "over the baseline: {total} tokens ({report})"
        );
        assert!(
            total <= TARGET,
            "over the target: {total} tokens ({report})"
        );
    }
}
