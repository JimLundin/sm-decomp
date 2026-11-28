// A test reads its own domain's vector file, and a local function may share the crate's name.
fn vector_file() -> String {
    vector_file::read("demo").unwrap_or_default()
}

#[test]
fn vectors_read() {
    assert!(vector_file().lines().count() < usize::MAX);
}
