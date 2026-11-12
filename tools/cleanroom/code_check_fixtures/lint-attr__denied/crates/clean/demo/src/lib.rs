#[expect(clippy::indexing_slicing, reason = "in range")]
pub fn f(a: &[u8]) -> u8 { a[0] }
