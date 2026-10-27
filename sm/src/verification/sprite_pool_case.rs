//! Shared synthetic entry and observation format; contains no sprite algorithm or ROM bytes.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub const PHASES: u16 = 52;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Operation {
    Create = 0,
    Update = 1,
    Draw = 2,
}
pub fn phase(index: u16) -> Option<(Operation, u8)> {
    if index >= PHASES {
        None
    } else if index == 0 {
        Some((Operation::Create, 0))
    } else if index == 1 {
        Some((Operation::Draw, 0))
    } else {
        Some((
            if index % 2 == 0 {
                Operation::Update
            } else {
                Operation::Draw
            },
            ((index - 2) / 2 + 1) as u8,
        ))
    }
}
pub fn put(game: &mut Game, address: usize, value: u16) {
    let _ = &[(); 0x20000][address..address + 2];
    game.ram().copy_in(address, &value.to_le_bytes());
}
pub fn word(game: &Game, address: usize) -> u16 {
    u16::from_le_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ])
}
pub fn seed(game: &mut Game) {
    game.ram().fill(0..game.ram().len(), 0);
    let value = 1;
    game.ram().write_byte(0x1ffff, value);
    for i in 0..512 {
        put(game, 0xef78 + i * 2, 0x5000 ^ i as u16);
    }
    for i in 0..32 {
        put(game, 0xef78 + i * 2, 0);
    }
    put(game, 0xef76, 0xface);
    put(game, 0xf378, 0xcafe);
    put(game, 0x1844, 0xbeef);
    put(game, 0x12, 128);
    put(game, 0x14, 96);
    put(game, 0x16, 6);
    put(game, 0x18, 0x0e20);
}
pub fn prepare(operation: Operation, game: &mut Game) {
    if operation == Operation::Draw {
        // Normal per-frame OAM assembly starts empty. No sprite state is reset.
        game.ram().fill(0x370..0x592, 0);
    }
}
pub fn record(output: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    output.push(tag);
    output.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    output.extend_from_slice(payload);
}
pub fn state(
    output: &mut Vec<u8>,
    tag: u8,
    index: u16,
    operation: Operation,
    tick: u8,
    result: u16,
    cpu: [u8; 16],
    game: &Game,
) {
    let mut payload = index.to_le_bytes().to_vec();
    payload.push(operation as u8);
    payload.push(tick);
    payload.extend_from_slice(&result.to_le_bytes());
    payload.extend_from_slice(&cpu);
    payload.extend((0..game.ram().len()).map(|at| game.ram().read_byte(at)));
    record(output, tag, &payload);
}
pub fn mask(output: &mut Vec<u8>, index: u16, addresses: &std::collections::BTreeSet<usize>) {
    let mut payload = index.to_le_bytes().to_vec();
    for &address in addresses {
        payload.extend_from_slice(&(address as u32).to_le_bytes());
    }
    record(output, 3, &payload);
}
