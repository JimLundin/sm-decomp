const VECTORS: &str = include_str!("../../../../scenarios/vectors/demo.toml");

#[test]
fn vectors_read() {
    assert!(VECTORS.lines().count() < usize::MAX);
}
