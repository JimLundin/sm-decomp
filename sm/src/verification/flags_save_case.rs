//! Synthetic SaveToSram cases and capture encoding; no save/map algorithm or ROM bytes.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub const STAGES: [u16; 7] = [0x8000, 0x8020, 0x803a, 0x803d, 0x8050, 0x8068, 0x8084];
pub fn put(game: &mut Game, address: usize, value: u16) {
    let _ = &[(); 0x20000][address..address + 2];
    game.ram().copy_in(address, &value.to_le_bytes());
}
pub fn seed(id: u32, game: &mut Game) -> Result<u16, String> {
    let (slot, area, station, salt) = match id {
        0 => (0, 1, 3, 0x39u8),
        1 => (2, 6, 0, 0xa7u8),
        _ => return Err("unknown flags save case".into()),
    };
    for i in 0..game.ram().len() {
        let value = (i as u8)
            .wrapping_mul(29)
            .wrapping_add((i >> 8) as u8)
            .wrapping_add(salt);
        game.ram().write_byte(i, value);
    }
    for i in 0..8192 {
        game.g_sram.write(
            i,
            (i as u8)
                .wrapping_mul(11)
                .wrapping_add((i >> 7) as u8)
                .wrapping_add(salt ^ 0x5a),
        );
    }
    put(game, 0x79f, area);
    put(game, 0x78b, station);
    for i in 0..256usize {
        let value = if i & 1 == 0 {
            (i as u8).wrapping_add(0x19 ^ salt)
        } else {
            0xe2u8.wrapping_sub(i as u8).wrapping_add(salt)
        };
        game.ram().write_byte(0x7f7 + i, value);
    }
    for i in 0..208usize {
        let value = (i as u8).wrapping_mul(7).wrapping_add(salt ^ 0xc3);
        game.ram().write_byte(0xd820 + i, value);
    }
    put(game, 0x5e7, 0xa534);
    Ok(slot)
}
pub fn record(output: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    output.push(tag);
    output.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    output.extend_from_slice(payload);
}
pub fn state(output: &mut Vec<u8>, stage: u8, cpu: [u8; 16], game: &Game) {
    let mut payload = vec![stage];
    payload.extend_from_slice(&cpu);
    payload.extend((0..game.ram().len()).map(|at| game.ram().read_byte(at)));
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
