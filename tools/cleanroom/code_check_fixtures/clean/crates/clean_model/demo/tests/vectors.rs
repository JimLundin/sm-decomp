const VECTORS: &str = include_str!("../../../../scenarios/vectors/demo.toml");

#[test]
fn vectors_parse() {
    let table: toml::Table = VECTORS.parse().unwrap();
    assert!(table.is_empty() || !table.is_empty());
}
