#[test]
fn read() {
    let domain = "demo";
    assert!(vector_file::read(domain).is_ok());
}
