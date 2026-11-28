#[test]
fn read() {
    assert!(vector_file::read("../vectors/demo").is_ok());
}
