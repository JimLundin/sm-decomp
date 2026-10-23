//! Encoded beam definitions shared by firing, reflection and special attacks.
#![forbid(unsafe_code)]
#[derive(Clone, Copy)]
pub enum Motion {
    NonWave,
    IceWave,
    WavePlasma,
    Hyper,
}
pub const UNCHARGED_SOUND: [u16; 12] = [
    0xb, 0xd, 0xc, 0xe, 0xf, 0x12, 0x10, 0x11, 0x13, 0x16, 0x14, 0x15,
];
pub const CHARGED_SOUND: [u16; 12] = [
    0x17, 0x19, 0x18, 0x1a, 0x1b, 0x1e, 0x1c, 0x1d, 0x1f, 0x22, 0x20, 0x21,
];
pub const AUTO_COOLDOWN: [u8; 12] = [0x19; 12];
pub const MANUAL_COOLDOWN: [u8; 38] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 12, 15, 0, 0, 0, 0, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    30, 30, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub fn motion(charged: bool, index: usize) -> Option<Motion> {
    if index >= 12 {
        None
    } else if index & 1 == 0 {
        Some(Motion::NonWave)
    } else if !charged && index < 4 {
        Some(Motion::IceWave)
    } else {
        Some(Motion::WavePlasma)
    }
}
pub fn instruction(motion: Motion) -> u16 {
    match motion {
        Motion::NonWave => 0xaef3,
        Motion::IceWave => 0xb0e4,
        Motion::WavePlasma => 0xb0c3,
        Motion::Hyper => 0xb159,
    }
}
