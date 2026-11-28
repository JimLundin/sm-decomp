///Adds one.
///
///    assert_eq!(demo::next(1), 2);
pub fn next(n: u8) -> u8 {
    n.wrapping_add(1)
}
