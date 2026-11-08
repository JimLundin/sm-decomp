// The catalogue reads its entries' checked lines from the spec.
const SPEC: &str = include_str!("../../../../docs/behaviours/demo.md");

#[test]
fn the_spec_is_text() {
    assert!(SPEC.contains("checked"));
}
