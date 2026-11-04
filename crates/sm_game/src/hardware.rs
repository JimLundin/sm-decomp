//! Narrow hardware ports: one typed interface for each hardware job the engine
//! still asks of the console, which the runtime implements. The emulated backend
//! in sm/ implements each port with the original's register writes and reads.

/// The console's multipliers.
pub trait Multiplier {
    /// The PPU's signed multiply: `multiplicand` to M7A (its low byte, then its
    /// high byte) and `multiplier` to M7B, then product bits 8 to 23 read back
    /// from MPYM and MPYH.
    fn mode7_product(&mut self, multiplicand: u16, multiplier: u8) -> u16;
    /// Write `operand` to the CPU multiplier's multiplicand (WRMPYA).
    fn cpu_multiplicand(&mut self, operand: u8);
}

/// `length` times the sine of `angle`, through the PPU multiplier. Was
/// `ComputeSinMult`.
pub fn sine_times(game: &mut (impl Multiplier + ?Sized), length: u16, angle: u16) -> u16 {
    let sine = crate::projectile_geometry::SINE_COSINE[usize::from(angle & 0xff) + 64];
    game.mode7_product(sine.cast_unsigned(), length.to_le_bytes()[0])
}

/// `length` times the cosine of `angle`, through the PPU multiplier. Was
/// `ComputeCosMult`.
pub fn cosine_times(game: &mut (impl Multiplier + ?Sized), length: u16, angle: u16) -> u16 {
    sine_times(game, length, angle.wrapping_add(64))
}

/// The CPU's view of the bus, for the reads the original makes through a long
/// address that can reach past work RAM.
pub trait Bus {
    /// The byte the CPU reads at 24-bit `address`, with the read's side effects.
    fn read_bus_byte(&mut self, address: u32) -> u8;
}
