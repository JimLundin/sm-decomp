pub const fn below(a: u16, b: u16) -> bool { a.wrapping_sub(b) > 0x7FFF }
