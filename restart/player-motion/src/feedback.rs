use crate::memory::*;

// sm_90.c:5918–5929, pinned recovered source. Only Tab0 is annotated 90:E61B.
const FALL: [u8; 28] = [0, 0, 4, 4, 1, 0, 4, 2, 4, 4, 0, 0, 0, 0, 4, 4, 0, 3, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4];
const LAND: [u8; 28] = [4, 4, 0, 0, 4, 4, 0, 4, 1, 2, 0, 4, 4, 0, 4, 4, 4, 4, 3, 3, 0, 4, 4, 4, 4, 0, 4, 4];

pub(crate) fn down_supported(ram: &[u8], collision: u16) -> bool {
    (collision == 0 && ram[FEEDBACK] == 5) || usize::from(ram[MOVEMENT]) < FALL.len()
}

pub(crate) fn up(ram: &mut [u8]) {
    put(ram, FEEDBACK, if word(ram, COLLISION) != 0 { 4 } else { 0 });
}

pub(crate) fn down(ram: &mut [u8]) -> Result<(), String> {
    let table = if word(ram, COLLISION) != 0 {
        &LAND
    } else if ram[FEEDBACK] != 5 {
        &FALL
    } else {
        return Ok(());
    };
    let high = *table.get(usize::from(ram[MOVEMENT])).ok_or("terrain returned an invalid movement type")?;
    put(ram, FEEDBACK, (u16::from(high) << 8) | if word(ram, COLLISION) != 0 { 1 } else { 2 });
    Ok(())
}
