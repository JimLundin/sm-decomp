use vector_file::read;

#[test]
fn reads() {
    assert!(read("world-doors").is_ok());
}
