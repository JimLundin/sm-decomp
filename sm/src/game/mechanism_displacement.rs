//! Compatibility transport for one active contact; RAM aliases are offsets, never references.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub enum Displacement<'a> {
    Absent,
    Local(&'a mut i32),
    RamOffset(usize),
}
impl Displacement<'_> {
    pub fn present(&self) -> bool {
        !matches!(self, Self::Absent)
    }
    pub fn read(&self, game: &Game) -> i32 {
        match self {
            Self::Absent => panic!("missing contact displacement"),
            Self::Local(value) => **value,
            Self::RamOffset(at) => {
                i32::from_ne_bytes(core::array::from_fn(|i| game.ram().read_byte(*at + i)))
            }
        }
    }
    pub fn write(&mut self, value: i32, game: &mut Game) {
        match self {
            Self::Absent => panic!("missing contact displacement"),
            Self::Local(target) => **target = value,
            Self::RamOffset(at) => game.ram().copy_in(*at, &value.to_ne_bytes()),
        }
    }
}
