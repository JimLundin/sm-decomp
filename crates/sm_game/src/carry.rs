//! The 65816's 16-bit add and subtract with carry, for the steps whose carry
//! the original chains from one operation into the next.

/// `ADC`: the sum and its carry out.
pub fn adc(a: u16, b: u16, carry: bool) -> (u16, bool) {
    let sum = u32::from(a) + u32::from(b) + u32::from(carry);
    (sum as u16, sum > 0xffff)
}

/// `SBC`: the difference and its carry (no borrow) out.
pub fn sbc(a: u16, b: u16, carry: bool) -> (u16, bool) {
    let difference = i32::from(a) - i32::from(b) - i32::from(!carry);
    (difference as u16, difference >= 0)
}
