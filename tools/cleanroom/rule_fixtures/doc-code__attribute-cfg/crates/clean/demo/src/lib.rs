#[cfg_attr(all(), doc = "~~~\nassert!(true);\n~~~")]
pub fn next(n: u8) -> u8 {
    n.wrapping_add(1)
}
