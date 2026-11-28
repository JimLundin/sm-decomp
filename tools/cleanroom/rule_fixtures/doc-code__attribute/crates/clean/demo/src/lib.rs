#[doc = "Adds one.\n\n```\nassert_eq!(demo::next(1), 2);\n```"]
pub fn next(n: u8) -> u8 {
    n.wrapping_add(1)
}
