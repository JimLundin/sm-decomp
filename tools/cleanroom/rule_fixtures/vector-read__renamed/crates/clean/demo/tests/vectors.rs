use vector_file as vf;

#[test]
fn reads() {
    assert!(vf::read("world-doors").is_ok());
}
