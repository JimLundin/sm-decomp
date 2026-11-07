// A size check reads the clean crates' own sources as text.
const OWN: &str = include_str!("../src/lib.rs");
const TYPES: &str = include_str!("../../../clean_types/src/lib.rs");

#[test]
fn sources_are_text() {
    assert!(!OWN.is_empty() && !TYPES.is_empty());
}
