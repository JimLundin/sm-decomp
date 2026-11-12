// A size check reads the clean crates' own sources as text.
const OWN: &str = include_str!("../src/lib.rs");
const TYPES: &str = include_str!("../../types/src/lib.rs");

#[test]
#[expect(clippy::arithmetic_side_effects, reason = "a test may suppress a denied lint")]
fn sources_are_text() {
    assert!(!OWN.is_empty() && !TYPES.is_empty() && OWN.len() + 2 > 3);
}
