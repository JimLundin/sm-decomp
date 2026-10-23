//! Synthetic entries for actual Kraid native boss-word callers; no gameplay model.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub fn parameters(id: u32) -> Result<(bool, u16, u16), String> {
    if id >= 12 {
        return Err("unknown Kraid flags case".into());
    }
    Ok((
        id >= 6,
        [1, 7, 0xffff][(id as usize % 6) / 2],
        (id % 2) as u16,
    ))
}
pub fn put(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}
pub fn word(game: &Game, address: usize) -> u16 {
    u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
}
pub fn seed(id: u32, game: &mut Game) -> Result<bool, String> {
    let (fade, area, bit) = parameters(id)?;
    for (i, byte) in game.g_ram.iter_mut().enumerate() {
        *byte = (i as u8)
            .wrapping_mul(17)
            .wrapping_add((i >> 8) as u8)
            .wrapping_add(0x53);
    }
    for i in 0..8192 {
        game.g_sram
            .write(i, (i as u8).wrapping_mul(3).wrapping_add(0x61));
    }
    put(game, 0x79f, area);
    put(game, 0xd828 + usize::from(area), 0xa65a | bit);
    put(game, 0x5e7, 0xa534);
    put(game, 0xc400, 14);
    put(game, 0xc402, 0x4567);
    put(game, 0xfa8, 0x1234);
    put(game, 0x998, 8);
    put(game, 0x639, 0);
    put(game, 0x63b, 6);
    put(game, 0x619, 0x9999);
    put(game, 0x629, 0x7777);
    Ok(fade)
}
pub fn record(output: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    output.push(tag);
    output.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    output.extend_from_slice(payload);
}
pub fn state(output: &mut Vec<u8>, stage: u8, result: u16, cpu: [u8; 16], game: &Game) {
    let mut payload = vec![stage];
    payload.extend_from_slice(&result.to_le_bytes());
    payload.extend_from_slice(&cpu);
    payload.extend_from_slice(&game.g_ram);
    payload.extend_from_slice(&game.g_sram.bytes(0..8192));
    record(output, 1, &payload);
}
pub fn mask(output: &mut Vec<u8>, stage: u8, addresses: &std::collections::BTreeSet<usize>) {
    let mut payload = vec![stage];
    for &address in addresses {
        payload.extend_from_slice(&(address as u32).to_le_bytes());
    }
    record(output, 2, &payload);
}
