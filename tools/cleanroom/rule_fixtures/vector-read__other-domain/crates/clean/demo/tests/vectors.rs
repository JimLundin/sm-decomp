#[test]
fn read() {
    assert!(vector_file::read("world-doors").is_ok());
}
