//! The random number generator: one 16-bit word, stepped by a multiply and an add.

use crate::state::Rng;

/// What each step multiplies the word by.
const MULTIPLIER: u8 = 5;
/// What each step adds after the multiply.
const INCREMENT: u16 = 17;

impl Rng {
    /// Step the generator and return the new word.
    ///
    /// Bank `$80` log, generate random number `$80:8111`: roughly `word * 5 + 0x111`, but
    /// built a byte at a time on the hardware multiplier. Only the low byte of `high * 5` is
    /// kept, the low product's high byte and one more are added into it, and a carry out of
    /// that byte adds one to the final sum.
    pub fn advance(&mut self) -> u16 {
        let [low, high] = self.word.to_le_bytes();
        let [product_low, product_high] = u16::from(low)
            .wrapping_mul(u16::from(MULTIPLIER))
            .to_le_bytes();
        let (sum, first_carry) = high.wrapping_mul(MULTIPLIER).overflowing_add(product_high);
        let (sum, second_carry) = sum.overflowing_add(1);
        let carry = u16::from(first_carry || second_carry);
        self.word = u16::from_le_bytes([product_low, sum])
            .wrapping_add(INCREMENT)
            .wrapping_add(carry);
        self.calls = self.calls.wrapping_add(1);
        self.word
    }
}

#[cfg(test)]
mod tests {
    use crate::state::Rng;

    fn after(word: u16) -> u16 {
        Rng { word, calls: 0 }.advance()
    }

    #[test]
    fn steps_as_the_bytewise_multiply_and_add() {
        // 4F * 5 = 018B; E8 * 5 = 0488, low byte 88; 88 + 01 + 1 = 8A; 8A8B + 11 = 8A9C.
        // The step the two slice 0 rows take on their first Tick.
        assert_eq!(after(0xE84F), 0x8A9C);
        // 00 * 5 = 0000; 33 * 5 = FF; FF + 00 + 1 carries out, so 0000 + 11 + 1.
        assert_eq!(after(0x3300), 0x0012);
        // FF * 5 = 04FB; FF * 5 = 04FB, low byte FB; FB + 04 + 1 = 00 with a carry.
        assert_eq!(after(0xFFFF), 0x010D);
        assert_eq!(after(0), 0x0111);
    }

    #[test]
    fn counts_its_calls() {
        let mut rng = Rng::default();
        rng.advance();
        rng.advance();
        assert_eq!(rng.calls, 2);
        assert_eq!(rng.word, after(0x0111));
    }
}
