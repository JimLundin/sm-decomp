#[test]
#[expect(clippy::arithmetic_side_effects)]
fn t() { assert_eq!(1 + 1, 2); }
