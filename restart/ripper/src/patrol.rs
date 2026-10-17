use crate::{Services, memory::*};

fn instruction(jet: bool, right: bool) -> u16 {
    match (jet, right) {
        (true, false) => 0xe2e0,
        (true, true) => 0xe2f4,
        (false, false) => 0xe48b,
        (false, true) => 0xe477,
    }
}

pub(crate) fn init(ram: &mut [u8], rom: &[u8], base: usize, jet: bool) {
    let row = word(ram, base + PARAMETER1).wrapping_mul(8);
    let right = word(ram, base + PARAMETER2) != 0;
    let (integer, fraction) = speed(rom, row, right);
    put(ram, base + INSTRUCTION, instruction(jet, right));
    put(ram, base + E, row);
    put(ram, base + D, integer);
    put(ram, base + C, fraction);
}

pub(crate) fn main(
    ram: &mut [u8],
    rom: &[u8],
    slot: u16,
    base: usize,
    jet: bool,
    services: &mut impl Services,
) -> Result<(), String> {
    let amount = ((u32::from(word(ram, base + D)) << 16) | u32::from(word(ram, base + C))) as i32;
    match services.move_horizontal(ram, slot, amount)? {
        0 => Ok(()),
        1 => {
            // C retains E's record pointer, then reads its row and D after the
            // synchronous native call. Do not cache either across copyback.
            let row = word(ram, base + E);
            if !valid_row(row) {
                return Err("Ripper service returned an unsupported speed row".into());
            }
            let right = word(ram, base + D) & 0x8000 != 0;
            let (integer, fraction) = speed(rom, row, right);
            put(ram, base + D, integer);
            put(ram, base + C, fraction);
            put(ram, base + INSTRUCTION, instruction(jet, right));
            put(ram, base + INSTRUCTION_TIMER, 1);
            put(ram, base + TIMER, 0);
            Ok(())
        }
        other => Err(format!("invalid Ripper native collision result {other}")),
    }
}
