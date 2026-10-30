//! Axis and scan state shared by projectile terrain reactions.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
}
impl Axis {
    pub fn other(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::X,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scan {
    pub displacement: i32,
    pub position: u16,
    pub span: u16,
    pub edge: u16,
    pub missile: u16,
    pub auxiliary: u16,
    pub remaining: u16,
    pub unblocked: u16,
}
